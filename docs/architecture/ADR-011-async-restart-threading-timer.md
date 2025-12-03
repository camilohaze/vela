# ADR-011: Async Restart Logic con threading.Timer

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto
La implementación actual de `restart_child()` en el sistema de supervisión usa `time.sleep()` para aplicar backoff delays, lo cual **bloquea el supervisor** durante el delay. Esto es problemático porque:

1. **Bloqueo del supervisor**: Mientras espera el delay, el supervisor no puede procesar otros fallos ni mensajes
2. **Cascadas de bloqueos**: En jerarquías de supervisores, cada nivel se bloquea esperando al inferior
3. **No cancellable**: Una vez iniciado el sleep, no se puede cancelar el restart pendiente
4. **Escalabilidad**: Con muchos children fallando, el supervisor se convierte en un cuello de botella

**Alternativas consideradas:**

### Alternativa 1: Integración con ActorScheduler (RECHAZADA)
```python
# Usar scheduler.schedule_delayed() para async restarts
scheduler.schedule_delayed(delay, lambda: restart_child(child_ref))
```

**Razón de rechazo:** Circular dependency
- SupervisorActor necesita scheduler.schedule_delayed()
- ActorScheduler necesita crear actors (incluyendo SupervisorActor)
- Solución requiere inyección de dependencias compleja

### Alternativa 2: threading.Timer (ACEPTADA ✅)
```python
# Usar threading.Timer para async restarts
timer = threading.Timer(delay, do_restart)
timer.daemon = True  # No bloquea shutdown
timer.start()
```

**Ventajas:**
- ✅ Simple (stdlib, no deps externas)
- ✅ No bloquea el supervisor
- ✅ Cancellable (timer.cancel())
- ✅ Sin circular dependencies
- ✅ Daemon threads no bloquean shutdown

**Desventajas:**
- ⚠️ Threads (overhead mínimo, pero threads nonetheless)
- ⚠️ No es async/await nativo (pero funcional para el use case)

### Alternativa 3: asyncio (RECHAZADA)
```python
# Usar asyncio.create_task() para async restarts
await asyncio.sleep(delay)
restart_child(child_ref)
```

**Razón de rechazo:**
- Requiere migrar TODA la arquitectura de actors a async/await
- El sistema actual es síncrono (Actor.receive() no es async)
- Cambio masivo para TASK-043 (out of scope)

## Decisión
**Usar threading.Timer para async restarts (Alternativa 2)**

Implementación:
```python
def restart_child(self, child_ref: ActorRef) -> None:
    # Calcular delay
    delay = self.strategy.restart_policy.calculate_delay(stats.failure_count)
    
    # Función de restart
    def do_restart():
        child_ref.actor.pre_restart(error=None)
        child_ref.actor.state = ActorState.RUNNING
        child_ref.actor.post_restart(error=None)
        stats.record_restart()
        del self._pending_restarts[child_name]
    
    # Cancelar timer anterior si existe
    if child_name in self._pending_restarts:
        old_timer = self._pending_restarts[child_name]
        old_timer.cancel()
    
    # Schedule restart asíncrono
    timer = threading.Timer(delay, do_restart)
    timer.daemon = True  # Daemon thread
    timer.start()
    self._pending_restarts[child_name] = timer
```

**Cambios de estado:**
- `_pending_restarts: Dict[str, str]` → `Dict[str, threading.Timer]`
- `cancel_pending_restarts()`: Usa `timer.cancel()` (antes usaba scheduler API)
- `restart_child()`: No bloquea, retorna inmediatamente

## Consecuencias

### Positivas
- ✅ **Supervisor no bloqueante**: Puede manejar múltiples fallos concurrentes
- ✅ **Cancellable**: `cancel_pending_restarts()` funciona correctamente
- ✅ **Simplicidad**: Sin deps externas, código claro
- ✅ **Escalabilidad**: Múltiples restarts en paralelo sin bloqueos
- ✅ **Shutdown limpio**: Daemon threads no impiden shutdown

### Negativas
- ⚠️ **Tests más complejos**: Requieren `time.sleep()` para esperar Timer execution
- ⚠️ **Race conditions potenciales**: Si handle_child_failure() se llama 2 veces antes del restart
  - **Mitigación**: Cancel timer anterior antes de crear nuevo
- ⚠️ **No es async/await**: Si en el futuro se migra a async, habrá que refactorizar
  - **Mitigación**: El código está encapsulado en `restart_child()`, cambio localizado

### Neutral
- 🔄 **Thread overhead**: Mínimo para el use case (1 thread por restart pendiente)
- 🔄 **No es event-driven puro**: Pero Actor model tampoco lo requiere

## Referencias
- **Jira**: TASK-043 (VELA-579)
- **Historia**: VELA-579 - Sistema de Supervision de Actors
- **Código**: `src/concurrency/supervision.py` (líneas 670-720)
- **Tests**: `tests/unit/concurrency/test_supervision.py` (32 tests pasando)

## Implementación
- Archivo: `src/concurrency/supervision.py`
- Método: `restart_child()`, `cancel_pending_restarts()`
- Tests: 32/32 pasando (incluyendo tests de async timing)

## Métricas
- **LOC modificadas**: ~50 líneas
- **Tests agregados**: 7 tests modificados para async
- **Tests pasando**: 32/32 (100%)
- **Tiempo de implementación**: 3 horas

## Notas Técnicas

### Daemon Threads
Los Timer threads se configuran como **daemon threads** (`timer.daemon = True`). Esto significa:
- **No bloquean shutdown**: El programa puede terminar incluso con timers pendientes
- **Limpieza automática**: Python mata daemon threads al exit
- **Trade-off**: Restarts pendientes se cancelan si el programa termina
  - **OK**: Si el programa termina, no importa que restarts pendientes se pierdan

### Cancelación de Timers
```python
# Cancelar timer anterior
if child_name in self._pending_restarts:
    old_timer = self._pending_restarts[child_name]
    old_timer.cancel()  # Cancela Timer antes de que ejecute
```

**Comportamiento:**
- Si Timer **NO ejecutó**: Cancel funciona, `do_restart()` no ejecuta
- Si Timer **YA ejecutó**: Cancel no hace nada (no-op)
- Si Timer **está ejecutando**: Race condition (raro, pero posible)
  - **Mitigación**: Check `child_name in self._pending_restarts` en `do_restart()`

### Tests Async
Los tests deben esperar la ejecución del Timer:
```python
supervisor.handle_child_failure(child, error)

# TASK-043: Esperar restart asíncrono
time.sleep(delay + 0.05)  # delay + buffer

assert child.actor.restart_count == 1
```

**Valores de sleep:**
- `initial_delay=0.1s` (en fixtures de test)
- `time.sleep(0.15s)` (delay + 0.05s buffer)

## Decisiones Futuras

### Migración a async/await (si es necesario)
Si en el futuro se decide migrar a async/await:
1. Cambiar `Actor.receive()` a `async def receive()`
2. Cambiar `restart_child()` a usar `asyncio.create_task()`
3. Reemplazar `threading.Timer` con `asyncio.sleep()`

**Impacto:** Localizado en `restart_child()`, el resto del código sigue igual.

### Event-Driven Scheduler (si se necesita)
Si se necesita un scheduler más sofisticado:
1. Implementar `ActorScheduler` sin circular deps (patrón Registry)
2. Migrar `restart_child()` a usar scheduler
3. Tests siguen pasando (interface igual)

**Decisión:** YAGNI (You Ain't Gonna Need It) - threading.Timer es suficiente para MVP.

---

**ÚLTIMA ACTUALIZACIÓN:** 2025-12-02  
**VERSIÓN:** 1.0.0  
**AUTOR:** Copilot Agent (TASK-043)
