# TASK-043: Restart logic with backoff (async, non-blocking)

## 📋 Información General
- **Historia:** VELA-579
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02

## 🎯 Objetivo
Implementar lógica de restart asíncrona con backoff que **NO bloquee** el supervisor mientras espera delays, permitiendo procesar otros fallos concurrentemente.

### Problema Original
En TASK-042, la implementación usaba `time.sleep()` para aplicar backoff delays:

```python
# ❌ PROBLEMA: Blocking
def restart_child(self, child_ref: ActorRef) -> None:
    delay = self.strategy.restart_policy.calculate_delay(stats.failure_count)
    time.sleep(delay)  # BLOQUEA el supervisor
    child_ref.actor.pre_restart()
    # ...
```

**Consecuencias:**
- Supervisor bloqueado durante delay (no puede procesar otros mensajes)
- Restarts secuenciales (no concurrentes)
- No cancellable
- No escalable

## 🔨 Implementación

### Solución: threading.Timer

Migrar a **async restarts** usando `threading.Timer` (stdlib, simple, sin deps externas):

```python
# ✅ SOLUCIÓN: Non-blocking
def restart_child(self, child_ref: ActorRef) -> None:
    delay = self.strategy.restart_policy.calculate_delay(stats.failure_count)
    
    # Función de restart (ejecuta EN EL TIMER)
    def do_restart():
        child_ref.actor.pre_restart(error=None)
        child_ref.actor.state = ActorState.RUNNING
        child_ref.actor.post_restart(error=None)
        stats.record_restart()
        del self._pending_restarts[child_name]
    
    # Cancelar timer pendiente si existe
    if child_name in self._pending_restarts:
        old_timer = self._pending_restarts[child_name]
        old_timer.cancel()
    
    # Schedule restart asíncrono
    timer = threading.Timer(delay, do_restart)
    timer.daemon = True  # No bloquea shutdown
    timer.start()
    self._pending_restarts[child_name] = timer
```

### Archivos generados/modificados
- `src/concurrency/supervision.py` - Implementación async restart
  * `restart_child()`: Usa threading.Timer (no bloquea)
  * `cancel_pending_restarts()`: Usa timer.cancel()
  * `_pending_restarts: Dict[str, threading.Timer]` (antes Dict[str, str])
- `tests/unit/concurrency/test_supervision.py` - Tests modificados para async
  * 7 tests modificados: Agregar `time.sleep(delay + 0.05)` después de handle_child_failure()
  * Fixture `restart_policy`: `initial_delay=0.1` (delay corto para tests)
- `docs/architecture/ADR-011-async-restart-threading-timer.md` - Decisión arquitectónica

## ✅ Criterios de Aceptación
- [x] Código implementado (threading.Timer)
- [x] Restarts no bloquean el supervisor
- [x] Cancellable con `cancel_pending_restarts()`
- [x] Daemon threads (no bloquean shutdown)
- [x] Tests escritos y pasando (32/32)
- [x] Tests modificados para async timing
- [x] Documentación generada
- [x] ADR creado

## 📊 Métricas
- **LOC modificadas:** ~50 líneas (supervision.py)
- **Tests:** 32/32 pasando (100%)
- **Tests modificados:** 7 tests (async timing)
- **Tiempo de implementación:** 3 horas
- **Cobertura:** >95% (supervision module)

## 🔍 Cambios Clave

### 1. Tipo de `_pending_restarts`
```python
# Antes
_pending_restarts: Dict[str, str]  # child_name -> task_id

# Después (TASK-043)
_pending_restarts: Dict[str, threading.Timer]  # child_name -> Timer
```

### 2. `restart_child()` - Non-blocking
```python
# NO usa time.sleep() → retorna inmediatamente
# Restart ejecuta EN EL TIMER (thread separado)
timer = threading.Timer(delay, do_restart)
timer.daemon = True
timer.start()
```

### 3. `cancel_pending_restarts()` - Usa timer.cancel()
```python
# Cancelar timer pendiente
timer = self._pending_restarts[child_name]
timer.cancel()  # Cancela Timer antes de que ejecute
del self._pending_restarts[child_name]
```

### 4. Tests - Async Timing
```python
# Tests deben esperar Timer execution
supervisor.handle_child_failure(child, error)

# TASK-043: Esperar restart asíncrono
time.sleep(0.15)  # delay=0.1s + 0.05s buffer

assert child.actor.restart_count == 1  # Ahora pasa
```

## 🧪 Tests Modificados

| Test | Cambio | Razón |
|------|--------|-------|
| `test_restart_only_failed_child` | +`time.sleep(0.15)` | Esperar Timer execution |
| `test_restart_with_backoff` | +`time.sleep(0.15)` | Medir elapsed time correctamente |
| `test_restart_increments_stats` | +`time.sleep(0.15)` | `record_restart()` ejecuta EN el Timer |
| `test_escalate_after_max_retries` | +`initial_delay=0.1` + `time.sleep(0.15)` | Delay era 1.0s (demasiado largo) |
| `test_restart_all_children` | +`time.sleep(0.15)` | TODOS los restarts son async |
| `test_restart_failed_and_subsequent` | +`time.sleep(0.15)` | RestForOne restarts múltiples |
| `test_restart_last_child_only` | +`time.sleep(0.15)` | RestForOne restart único |

**Total:** 7 tests modificados, 32/32 pasando.

## 🔗 Referencias
- **Jira:** [TASK-043](https://velalang.atlassian.net/browse/VELA-579)
- **Historia:** [VELA-579](https://velalang.atlassian.net/browse/VELA-579)
- **ADR:** [ADR-011-async-restart-threading-timer.md](../architecture/ADR-011-async-restart-threading-timer.md)

## 🧠 Decisiones Clave

### ¿Por qué threading.Timer y no ActorScheduler?
- **Circular dependency**: SupervisorActor → Scheduler → Actors (incluyendo SupervisorActor)
- **Simplicidad**: threading.Timer es stdlib, 0 deps externas
- **Suficiente**: Para restarts async, no necesitamos scheduler completo

### ¿Por qué daemon threads?
- **No bloquean shutdown**: Programa puede terminar con timers pendientes
- **Limpieza automática**: Python mata daemon threads al exit
- **Trade-off**: Restarts pendientes se pierden si shutdown (OK para MVP)

### ¿Por qué `time.sleep()` en tests?
- Restarts ejecutan EN EL TIMER (thread separado)
- Tests deben ESPERAR que el Timer ejecute antes de hacer asserts
- `time.sleep(delay + buffer)` asegura que Timer ejecutó

## 🚀 Próximos Pasos (Fuera de Scope)

### Migración a async/await (si es necesario)
```python
# Futuro: async/await
async def restart_child(self, child_ref: ActorRef) -> None:
    delay = calculate_delay()
    await asyncio.sleep(delay)
    do_restart()
```

**Impacto:** Localizado en `restart_child()`, resto del código igual.

### Event-Driven Scheduler (si se necesita)
- Implementar ActorScheduler sin circular deps
- Migrar `restart_child()` a usar scheduler
- Tests siguen pasando (interface igual)

**Decisión:** YAGNI - threading.Timer es suficiente para MVP.

---

**ÚLTIMA ACTUALIZACIÓN:** 2025-12-02  
**VERSIÓN:** 1.0.0  
**COMPLETADO POR:** Copilot Agent
