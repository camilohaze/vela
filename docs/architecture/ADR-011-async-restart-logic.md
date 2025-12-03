# ADR-011: Async Restart Logic con ActorScheduler Integration

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

En **TASK-042** implementamos el sistema de supervisión con 3 estrategias (OneForOne, OneForAll, RestForOne) y RestartPolicy con backoff. Sin embargo, la implementación actual usa `time.sleep()` para aplicar el delay de backoff:

```python
# ❌ PROBLEMA: Blocking
def restart_child(self, child_ref: ActorRef) -> None:
    delay = self.strategy.restart_policy.calculate_delay(stats.failure_count)
    time.sleep(delay)  # BLOQUEA el thread del supervisor
    child_ref.actor.pre_restart()
    # ... reiniciar child
```

**Problemas con el enfoque actual:**

1. **Blocking**: `time.sleep()` bloquea el thread del supervisor, impidiendo procesar otros mensajes
2. **No escalable**: Con múltiples children fallando simultáneamente, los restarts serían secuenciales
3. **No cancellable**: No hay forma de cancelar un restart scheduled
4. **Sin visibilidad**: El scheduler no tiene conocimiento de restarts pendientes

**Requisitos para TASK-043:**

- Restarts asíncronos que NO bloqueen el supervisor
- Integración con ActorScheduler para scheduling de restarts
- Soporte para cancelación de restarts pendientes
- Métricas de restarts scheduled/pending
- Backoff strategies aplicadas correctamente

## Decisión

**Implementar scheduled tasks en ActorScheduler con integración de restart logic.**

### 1. Agregar `schedule_delayed()` a ActorScheduler

```python
class ActorScheduler:
    def schedule_delayed(
        self, 
        callback: Callable[[], None], 
        delay_seconds: float,
        task_id: Optional[str] = None
    ) -> str:
        """
        Ejecutar callback después de un delay.
        
        Args:
            callback: Función a ejecutar
            delay_seconds: Delay en segundos
            task_id: ID opcional para la tarea
            
        Returns:
            Task ID (para cancelación)
        """
```

**Implementación:**
- Thread dedicado para scheduled tasks (Timer thread)
- Priority queue ordenada por `execute_at` time
- Task IDs únicos para rastreo y cancelación
- Integración con métricas del scheduler

### 2. Modificar `restart_child()` para usar `schedule_delayed()`

```python
# ✅ SOLUCIÓN: Non-blocking
def restart_child(self, child_ref: ActorRef) -> None:
    delay = self.strategy.restart_policy.calculate_delay(stats.failure_count)
    
    # Schedule restart asíncrono
    def do_restart():
        child_ref.actor.pre_restart()
        # ... reiniciar child
        child_ref.actor.post_restart()
    
    # NO bloquea
    self.scheduler.schedule_delayed(do_restart, delay)
```

### 3. Estructura de ScheduledTask

```python
@dataclass
class ScheduledTask:
    task_id: str
    callback: Callable[[], None]
    scheduled_at: float        # timestamp cuando se creó
    execute_at: float          # timestamp cuando debe ejecutarse
    delay: float              # delay original (para debug)
    cancelled: bool = False   # flag de cancelación
```

### 4. API de Scheduled Tasks

```python
# Schedule task
task_id = scheduler.schedule_delayed(callback, delay=5.0)

# Cancel task
scheduler.cancel_scheduled_task(task_id)

# Get pending tasks
pending = scheduler.get_pending_tasks()

# Metrics
metrics = scheduler.get_scheduled_task_metrics()
# → {pending: 5, executed: 100, cancelled: 2, avg_delay: 1.5}
```

## Consecuencias

### Positivas

1. **Non-blocking**: Supervisores pueden procesar otros mensajes mientras esperan restart
2. **Escalabilidad**: Múltiples restarts simultáneos manejados eficientemente
3. **Visibilidad**: Scheduler tiene conocimiento de restarts pendientes (métricas)
4. **Cancelación**: Restarts pueden cancelarse si el child se detiene antes
5. **Testing**: Tests pueden simular delays sin esperar tiempo real
6. **Consistencia**: Todos los delays usan el mismo mecanismo (no time.sleep scattered)

### Negativas

1. **Complejidad**: Requiere thread adicional para scheduled tasks
2. **Thread safety**: Necesita sincronización cuidadosa con locks
3. **Testing**: Tests más complejos (async timing)
4. **Overhead**: Memory overhead de pending tasks queue

### Mitigaciones

- Thread safety garantizado con locks en scheduled tasks queue
- Tests usan `mock time` para evitar waits reales
- Scheduled tasks queue tiene límite máximo (prevenir memory leak)
- Cleanup automático de tasks completados/cancelled

## Alternativas Consideradas

### Alternativa 1: Timer threads individuales por restart

```python
def restart_child(self, child_ref: ActorRef) -> None:
    delay = calculate_delay()
    timer = threading.Timer(delay, lambda: self._do_restart(child_ref))
    timer.start()
```

**Rechazada porque:**
- Crea thread por restart (no escala)
- No hay visibilidad centralizada
- Difícil cancelar múltiples timers
- No hay métricas agregadas

### Alternativa 2: `asyncio` con `await asyncio.sleep()`

```python
async def restart_child(self, child_ref: ActorRef) -> None:
    delay = calculate_delay()
    await asyncio.sleep(delay)
    self._do_restart(child_ref)
```

**Rechazada porque:**
- Requiere asyncio event loop
- Actor model de Vela es thread-based, no async/await
- Mixing threads + asyncio es complejo
- Vela NO usa async/await por diseño (actor model puro)

### Alternativa 3: Message-based delays (enviar mensaje futuro)

```python
def restart_child(self, child_ref: ActorRef) -> None:
    delay = calculate_delay()
    self.self().send_delayed(RestartMessage(child_ref), delay)
```

**Rechazada porque:**
- Requiere message loop del supervisor procesando activamente
- Si supervisor está bloqueado, no procesa mensaje
- Semántica confusa (mensaje "del futuro")
- No hay benefit claro sobre scheduled tasks

## Referencias

- **Erlang/OTP Timer**: `erlang:send_after/3` para mensajes delayed
- **Akka Scheduler**: `system.scheduler.scheduleOnce()` para tasks delayed
- **Java ScheduledExecutorService**: API de Java para scheduled tasks
- **Python threading.Timer**: Timer threads en stdlib

## Implementación

### Archivos modificados:
1. `src/concurrency/scheduler.py` - Agregar scheduled tasks support
2. `src/concurrency/supervision.py` - Integrar con schedule_delayed()
3. `tests/unit/concurrency/test_supervision.py` - Tests async restart
4. `tests/unit/concurrency/test_scheduler.py` - Tests de scheduled tasks

### API Final:

```python
# ActorScheduler
scheduler.schedule_delayed(callback, delay=5.0) -> str  # task_id
scheduler.cancel_scheduled_task(task_id) -> bool
scheduler.get_pending_tasks() -> List[ScheduledTask]
scheduler.get_scheduled_task_metrics() -> Dict[str, Any]

# SupervisorActor (internamente usa schedule_delayed)
supervisor.restart_child(child_ref)  # NON-BLOCKING ahora
supervisor.cancel_pending_restarts(child_ref) -> int  # cancelar restarts pendientes
```

### Métricas de Scheduled Tasks:

```python
{
    "pending_tasks": 5,          # Tasks esperando ejecución
    "executed_tasks": 100,       # Tasks ejecutados
    "cancelled_tasks": 2,        # Tasks cancelados
    "failed_tasks": 1,           # Tasks que lanzaron exception
    "avg_delay": 1.5,            # Delay promedio
    "max_delay": 30.0,           # Delay máximo
    "oldest_pending": 25.5       # Task más antiguo pendiente (segundos)
}
```

## Decisiones de Diseño

### 1. Priority Queue vs Lista

**Decisión**: Usar `heapq` (min-heap) ordenado por `execute_at`

**Razón**: O(log n) para insert, O(1) para peek, eficiente para popping tasks ready

### 2. Thread dedicado vs Thread pool

**Decisión**: Thread dedicado (`TimerThread`) que wake up para ejecutar tasks

**Razón**:
- Simplicity: Un thread vs coordinar pool
- Efficiency: Tasks son lightweight callbacks
- Escalabilidad: ThreadPoolExecutor sigue manejando message loops

### 3. Granularidad de wake ups

**Decisión**: Timer thread wake up cada 100ms para check tasks

**Razón**:
- Balance entre responsiveness y CPU usage
- 100ms es suficiente para backoff delays (usualmente > 500ms)
- Configurable si se necesita más precisión

### 4. Task ID format

**Decisión**: `f"task-{timestamp}-{counter}"`

**Razón**:
- Único (timestamp + counter)
- Ordenable (timestamp primero)
- Debuggable (timestamp legible)

## Testing Strategy

### Unit Tests:
- `test_schedule_delayed_executes_callback`
- `test_schedule_delayed_respects_delay`
- `test_cancel_scheduled_task`
- `test_multiple_scheduled_tasks`
- `test_scheduled_task_metrics`

### Integration Tests:
- `test_supervisor_restart_non_blocking`
- `test_multiple_children_restart_concurrent`
- `test_cancel_restart_on_child_stop`
- `test_restart_backoff_applied_correctly`

### Performance Tests:
- `test_schedule_1000_tasks_performance`
- `test_concurrent_schedule_and_cancel`

## Migration Path

1. ✅ TASK-042: Implementar supervision con `time.sleep()` (blocking)
2. → TASK-043: Agregar scheduled tasks + integrar (non-blocking)
3. → TASK-044: Integration tests completos

## Conclusión

Scheduled tasks integrados en ActorScheduler permiten restart logic asíncrono y eficiente. Supervisores no bloquean, múltiples restarts concurrentes son manejados correctamente, y el sistema es escalable y testeable.

Esta arquitectura sigue el patrón de Erlang/OTP (`send_after`) y Akka (`scheduleOnce`), adaptado al diseño thread-based de Vela.
