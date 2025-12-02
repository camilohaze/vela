# TASK-035K: Diseñar Arquitectura del Event Bus

## 📋 Información General
- **Epic:** EPIC-03C - Event System
- **User Story:** US-07C - Sistema de eventos genérico para comunicación desacoplada
- **Sprint:** 14
- **Estimación:** 24 horas
- **Estado:** ✅ Completada
- **Fecha:** 2025-12-02

---

## 🎯 Objetivo

Diseñar la arquitectura completa del sistema de eventos de Vela, incluyendo:
- EventBus<T> type-safe con subscriptions
- Event propagation (bubbling, capturing)
- Event filtering y routing
- Memory management (leak prevention)
- Integration con sistema reactivo (signals)

---

## 🔨 Implementación

### Archivos Creados

1. **ADR-035K**: `docs/architecture/ADR-035K-event-system.md` (~1,200 LOC)
   - Arquitectura completa del sistema de eventos
   - Decisiones de diseño y trade-offs
   - Comparación con alternativas (Observable, Signals, Actors)
   - Performance considerations

### Componentes Diseñados

#### 1. EventBus<T> (Core)
```vela
class EventBus<T> {
  listeners: Dict<String, List<EventListener<T>>> = {}
  
  fn on(eventType: String, listener: (event: T) -> void) -> Subscription
  fn emit(eventType: String, payload: T) -> void
  fn off(eventType: String, listener: (event: T) -> void) -> void
  fn clear(eventType: Option<String> = None) -> void
  fn listenerCount(eventType: String) -> Number
}
```

**Características**:
- ✅ Generic `EventBus<T>` para type safety
- ✅ Múltiples listeners por evento
- ✅ Error handling per listener (no crash)
- ✅ Subscription pattern para auto-cleanup

#### 2. Event<T> (Event Object)
```vela
class Event<T> {
  type: String
  payload: T
  timestamp: Number
  target: Option<Any> = None
  propagationStopped: Bool = false
  defaultPrevented: Bool = false
  
  fn stopPropagation() -> void
  fn preventDefault() -> void
}
```

#### 3. Subscription (Disposable Pattern)
```vela
class Subscription {
  eventType: String
  listener: Function
  bus: EventBus
  disposed: Bool = false
  
  fn unsubscribe() -> void
  fn destroy() -> void  # Auto-dispose on destroy
}
```

#### 4. EventEmitter Interface
```vela
interface EventEmitter<T> {
  fn on(eventType: String, listener: (event: T) -> void) -> Subscription
  fn emit(eventType: String, payload: T) -> void
  fn off(eventType: String, listener: (event: T) -> void) -> void
  fn once(eventType: String, listener: (event: T) -> void) -> Subscription
}
```

#### 5. Event Propagation (DOM-style)

**Fases**:
1. **CAPTURING**: parent → child (top-down)
2. **AT_TARGET**: event on target element
3. **BUBBLING**: child → parent (bottom-up)

```vela
enum PropagationPhase {
  CAPTURING = 1,
  AT_TARGET = 2,
  BUBBLING = 3
}

class PropagatingEvent<T> extends Event<T> {
  currentTarget: Option<EventTarget> = None
  phase: PropagationPhase = PropagationPhase.AT_TARGET
  
  fn stopImmediatePropagation() -> void
}
```

#### 6. Event Filtering

```vela
class EventFilter {
  static fn matchPattern(pattern: String, eventType: String) -> Bool
  static fn applyPredicate<T>(event: Event<T>, predicate: (Event<T>) -> Bool) -> Bool
  static fn matchTags(event: Event, requiredTags: List<String>) -> Bool
}

class FilteredEventBus<T> extends EventBus<T> {
  fn onPattern(pattern: String, listener: (event: T) -> void) -> Subscription
  fn onWhere(eventType: String, predicate: (event: T) -> Bool, listener: (event: T) -> void) -> Subscription
  fn onTag(tag: String, listener: (event: T) -> void) -> Subscription
}
```

**Ejemplos**:
```vela
bus.on("user.*", handler)        # Wildcard pattern
bus.onWhere("user.*", (e) => e.payload.role == "admin", handler)  # Predicate
bus.onTag("audit", handler)      # By tag
```

#### 7. Memory Management

**Auto-Dispose Pattern**:
```vela
class AutoDisposeEventBus<T> extends EventBus<T> {
  subscriptionsByOwner: Dict<Any, List<Subscription>> = {}
  
  fn on(eventType: String, listener: (event: T) -> void, owner: Option<Any> = None) -> Subscription
  fn disposeAll(owner: Any) -> void  # Dispose all subscriptions for owner
}
```

**Integration con Lifecycle**:
```vela
component UserList {
  mount() {
    eventBus.on("user.created", this.onUserCreated, owner=this)
  }
  
  destroy() {
    eventBus.disposeAll(this)  # Auto-cleanup
  }
}
```

#### 8. Integration con Sistema Reactivo

```vela
fn eventToSignal<T>(bus: EventBus<T>, eventType: String, initialValue: T) -> Signal<T> {
  sig = signal(initialValue)
  
  bus.on(eventType, (event: Event<T>) => {
    sig.value = event.payload
  })
  
  return sig
}

# Usage
userCreatedSignal = eventToSignal(eventBus, "user.created", None)
```

---

## 📊 Decisiones de Diseño

### ✅ Decisiones Tomadas

| Decisión | Razón | Alternativa Rechazada |
|----------|-------|----------------------|
| **EventBus<T> generic** | Type safety en compile-time | Event bus dinámico sin tipos |
| **Subscription pattern** | Auto-dispose, memory leak prevention | Manual unsubscribe everywhere |
| **DOM-style propagation** | Familiar para desarrolladores web | Custom propagation model |
| **Dict<String, List> storage** | O(1) emit, simple implementation | Tree structure (overhead) |
| **Error isolation** | Un listener crasheado no afecta otros | Propagate errors (crash all) |
| **Optional propagation** | Complejidad solo cuando se necesita | Always propagate (overhead) |

### ⚠️ Alternativas Consideradas

#### 1. Observable Pattern (RxJS-style)
```vela
class Observable<T> {
  fn subscribe(observer: Observer<T>) -> Subscription
  fn map<U>(fn: (T) -> U) -> Observable<U>
  fn filter(predicate: (T) -> Bool) -> Observable<T>
}
```

**Rechazado porque**:
- ❌ Más complejo que necesario para eventos simples
- ❌ Learning curve alta
- ✅ Puede agregarse después como `ObservableEventBus` wrapper

#### 2. Signal-based Events
```vela
userCreated = signal<Event<User>>(None)
```

**Rechazado porque**:
- ❌ Signals son para estado, no notificaciones one-time
- ❌ No soporta múltiples listeners nativamente
- ✅ Pero sí integramos: `eventToSignal()` helper

#### 3. Actor Message Passing
```vela
actor UserManager {
  receive {
    case CreateUser(user) => // ...
  }
}
```

**Rechazado porque**:
- ❌ Actors son para concurrencia/aislamiento, no events generales
- ❌ Overhead de mailbox innecesario
- ✅ Actors pueden USAR EventBus internamente

---

## 🔬 Performance Considerations

### Complejidad Algorítmica

| Operación | Complejidad | Notas |
|-----------|-------------|-------|
| `on()` subscribe | O(1) average | Hash map insert |
| `emit()` | O(n) | n = listener count |
| `off()` unsubscribe | O(n) | Linear search in listener list |
| `listenerCount()` | O(1) | Direct access |

### Thread Safety

**Decisión**: Lock-free para single-threaded, Mutex para multi-threaded

```python
class ThreadSafeEventBus(EventBus):
    def emit(self, event_type: str, payload: Any):
        with self._lock:
            listeners = self.listeners.get(event_type, []).copy()
        
        # Notify outside lock (prevent deadlock)
        for listener in listeners:
            try:
                listener(Event(event_type, payload))
            except Exception as e:
                logger.error(f"Listener error: {e}")
```

### Memory Management

**Estrategias**:
1. **Auto-dispose pattern**: Owner-based cleanup
2. **Weak references**: Para listeners de objetos temporales
3. **Subscription tracking**: Detectar leaks en DevTools

---

## 📚 Referencias e Inspiración

| Framework | Aspecto que inspira |
|-----------|---------------------|
| **Node.js EventEmitter** | API simple (on/emit/off), múltiples listeners |
| **RxJS Observables** | Type safety, operators, backpressure |
| **DOM Events** | Bubbling, capturing, preventDefault, stopPropagation |
| **Vue.js $emit** | Type-safe custom events en componentes |
| **Angular EventEmitter** | Generic EventEmitter<T> con type safety |
| **Akka Event Bus** | Clasificación de eventos, subscriptions por tipo |

**Documentación externa**:
- Node.js EventEmitter: https://nodejs.org/api/events.html
- DOM Events Level 3: https://www.w3.org/TR/DOM-Level-3-Events/
- RxJS: https://rxjs.dev/
- Akka Event Bus: https://doc.akka.io/docs/akka/current/event-bus.html

---

## ✅ Criterios de Aceptación

- [x] ADR-035K creado con arquitectura completa (~1,200 LOC)
- [x] EventBus<T> diseñado con type safety
- [x] Event propagation (bubbling, capturing) especificado
- [x] Event filtering diseñado (wildcard, predicates, tags)
- [x] Memory management strategy definida (auto-dispose)
- [x] Integration con signals diseñada
- [x] Performance considerations documentadas
- [x] Alternativas evaluadas y justificadas
- [x] Documentación TASK-035K.md completa

---

## 🎓 Lecciones Aprendidas

### ✅ Aciertos

1. **Type Safety First**: Generic `EventBus<T>` previene errores en compile-time
2. **Simple por Default**: EventBus básico es simple, features avanzadas son opcionales
3. **Memory Safety**: Auto-dispose pattern previene memory leaks comunes
4. **Familiar API**: Inspirado en Node.js y DOM, fácil de aprender
5. **Integration**: Se integra bien con signals para reactividad

### ⚠️ Desafíos

1. **Propagation Complexity**: DOM-style propagation es complejo, pero necesario para UI
2. **Performance Trade-offs**: O(n) emit es aceptable, pero optimizaciones posibles
3. **Debugging**: Event flow puede ser difícil de rastrear (mitigado con DevTools)

### 🔄 Mejoras Futuras

1. **Observable Wrapper**: Agregar `ObservableEventBus` para reactive programming avanzado
2. **DevTools Integration**: Visualizar event flow en tiempo real
3. **Performance Optimizations**: Listener pooling, batch emit
4. **Async Events**: Soporte para listeners async con backpressure

---

## 🔗 Referencias

- **Jira**: [TASK-035K](https://velalang.atlassian.net/browse/VELA-575)
- **Epic**: EPIC-03C - Event System
- **Sprint**: 14
- **ADR**: `docs/architecture/ADR-035K-event-system.md`
- **Siguiente Tarea**: TASK-035L - Implementar EventBus<T> core

---

**Estado**: ✅ COMPLETADA  
**Fecha de Inicio**: 2025-12-02  
**Fecha de Fin**: 2025-12-02  
**Tiempo Real**: ~4 horas (vs 24h estimadas)
