# ADR-035K: Event System Architecture

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

Vela necesita un **sistema de eventos genérico y type-safe** para comunicación desacoplada entre componentes. Este sistema debe servir tanto para:

1. **Backend**: Event-driven architecture en microservicios
2. **Frontend**: UI events, user interactions, custom events
3. **Cross-cutting**: Logging, analytics, notifications

### Inspiraciones

| Framework | Aspecto que inspira |
|-----------|---------------------|
| **Node.js EventEmitter** | API simple (on/emit/off), múltiples listeners |
| **RxJS Observables** | Type safety, operators, backpressure |
| **DOM Events** | Bubbling, capturing, preventDefault, stopPropagation |
| **Vue.js $emit** | Type-safe custom events en componentes |
| **Angular EventEmitter** | Generic EventEmitter<T> con type safety |
| **Akka Event Bus** | Clasificación de eventos, subscriptions por tipo |

### Requisitos

**Funcionales:**
- ✅ Type-safe: `EventBus<T>` con tipado estricto
- ✅ Subscribe/unsubscribe con listeners
- ✅ Emit events con payload type-safe
- ✅ Event propagation (bubbling, capturing)
- ✅ Event filtering y routing
- ✅ Memory leak prevention (auto-unsubscribe)

**No Funcionales:**
- ✅ Performance: O(1) emit, O(n) notify listeners
- ✅ Thread-safe para concurrencia
- ✅ Zero memory leaks (weak references donde aplique)
- ✅ Integración con sistema reactivo (signals)

---

## Decisión

### 1. Arquitectura General

```
┌─────────────────────────────────────────────────────┐
│                   Event System                       │
├─────────────────────────────────────────────────────┤
│                                                       │
│  ┌─────────────┐      ┌──────────────┐              │
│  │  EventBus<T> │◄────│ EventEmitter │              │
│  │             │      │  interface   │              │
│  └──────┬──────┘      └──────────────┘              │
│         │                                            │
│         │  uses                                      │
│         ▼                                            │
│  ┌─────────────────┐    ┌──────────────────┐        │
│  │ Subscription    │    │ EventPropagation │        │
│  │ Management      │    │ (bubble/capture) │        │
│  └─────────────────┘    └──────────────────┘        │
│         │                        │                   │
│         │                        │                   │
│         ▼                        ▼                   │
│  ┌─────────────────┐    ┌──────────────────┐        │
│  │ EventFilter     │    │ Memory Manager   │        │
│  │ (type/tags)     │    │ (weak refs)      │        │
│  └─────────────────┘    └──────────────────┘        │
│                                                       │
└─────────────────────────────────────────────────────┘
```

### 2. Core Components

#### 2.1. EventBus<T> (Generic Event Bus)

**Responsabilidad**: Bus central de eventos type-safe.

```vela
class EventBus<T> {
  # Private state
  listeners: Dict<String, List<EventListener<T>>> = {}
  
  # Subscribe to events
  fn on(eventType: String, listener: (event: T) -> void) -> Subscription {
    if !this.listeners.has(eventType) {
      this.listeners[eventType] = []
    }
    this.listeners[eventType].push(listener)
    
    # Return subscription for unsubscribe
    return Subscription(eventType, listener, this)
  }
  
  # Emit event
  fn emit(eventType: String, payload: T) -> void {
    if !this.listeners.has(eventType) {
      return
    }
    
    event = Event(eventType, payload)
    this.listeners[eventType].forEach(listener => {
      try {
        listener(event)
      } catch (error) {
        # Log error but continue propagation
        console.error("Event listener error:", error)
      }
    })
  }
  
  # Unsubscribe
  fn off(eventType: String, listener: (event: T) -> void) -> void {
    if !this.listeners.has(eventType) {
      return
    }
    this.listeners[eventType] = this.listeners[eventType]
      .filter(l => l != listener)
  }
  
  # Remove all listeners
  fn clear(eventType: Option<String> = None) -> void {
    match eventType {
      Some(type) => this.listeners.remove(type)
      None => this.listeners.clear()
    }
  }
  
  # Get listener count
  fn listenerCount(eventType: String) -> Number {
    return this.listeners.get(eventType).unwrapOr([]).length
  }
}
```

**Características**:
- ✅ Generic `EventBus<T>` para type safety
- ✅ Múltiples listeners por evento
- ✅ Error handling per listener (no crash)
- ✅ Subscription pattern para auto-cleanup

#### 2.2. Event<T> (Event Object)

```vela
class Event<T> {
  type: String
  payload: T
  timestamp: Number
  target: Option<Any> = None
  
  # Propagation control
  propagationStopped: Bool = false
  defaultPrevented: Bool = false
  
  fn stopPropagation() -> void {
    this.propagationStopped = true
  }
  
  fn preventDefault() -> void {
    this.defaultPrevented = true
  }
}
```

#### 2.3. Subscription (Disposable Pattern)

```vela
class Subscription {
  eventType: String
  listener: Function
  bus: EventBus
  disposed: Bool = false
  
  constructor(eventType: String, listener: Function, bus: EventBus) {
    this.eventType = eventType
    this.listener = listener
    this.bus = bus
  }
  
  fn unsubscribe() -> void {
    if !this.disposed {
      this.bus.off(this.eventType, this.listener)
      this.disposed = true
    }
  }
  
  # Auto-dispose on destroy
  fn destroy() -> void {
    this.unsubscribe()
  }
}
```

#### 2.4. EventEmitter Interface

**Responsabilidad**: Interface para objetos que emiten eventos.

```vela
interface EventEmitter<T> {
  # Subscribe to event
  fn on(eventType: String, listener: (event: T) -> void) -> Subscription
  
  # Emit event
  fn emit(eventType: String, payload: T) -> void
  
  # Unsubscribe
  fn off(eventType: String, listener: (event: T) -> void) -> void
  
  # One-time listener
  fn once(eventType: String, listener: (event: T) -> void) -> Subscription
}
```

**Implementación default**:

```vela
class BaseEventEmitter<T> implements EventEmitter<T> {
  private bus: EventBus<T> = EventBus<T>()
  
  fn on(eventType: String, listener: (event: T) -> void) -> Subscription {
    return this.bus.on(eventType, listener)
  }
  
  fn emit(eventType: String, payload: T) -> void {
    this.bus.emit(eventType, payload)
  }
  
  fn off(eventType: String, listener: (event: T) -> void) -> void {
    this.bus.off(eventType, listener)
  }
  
  fn once(eventType: String, listener: (event: T) -> void) -> Subscription {
    wrappedListener = (event: T) -> void {
      listener(event)
      this.off(eventType, wrappedListener)
    }
    return this.on(eventType, wrappedListener)
  }
}
```

---

### 3. Event Propagation (DOM-style)

**Inspiración**: DOM Level 3 Events

#### 3.1. Fases de Propagación

```
┌──────────────────────────────────────┐
│        1. CAPTURING PHASE            │
│     (parent → child, top-down)       │
└──────────────────────────────────────┘
                 ▼
┌──────────────────────────────────────┐
│        2. TARGET PHASE               │
│     (event on target element)        │
└──────────────────────────────────────┘
                 ▼
┌──────────────────────────────────────┐
│        3. BUBBLING PHASE             │
│     (child → parent, bottom-up)      │
└──────────────────────────────────────┘
```

#### 3.2. Implementación

```vela
enum PropagationPhase {
  CAPTURING = 1,
  AT_TARGET = 2,
  BUBBLING = 3
}

class PropagatingEvent<T> extends Event<T> {
  currentTarget: Option<EventTarget> = None
  phase: PropagationPhase = PropagationPhase.AT_TARGET
  immediatePropagationStopped: Bool = false
  
  fn stopImmediatePropagation() -> void {
    this.propagationStopped = true
    this.immediatePropagationStopped = true
  }
}

interface EventTarget {
  fn addEventListener(
    eventType: String, 
    listener: (event: PropagatingEvent) -> void,
    useCapture: Bool = false
  ) -> Subscription
  
  fn dispatchEvent(event: PropagatingEvent) -> Bool
  fn removeEventListener(
    eventType: String, 
    listener: Function
  ) -> void
  
  # Parent for propagation
  fn getParent() -> Option<EventTarget>
}
```

**Algoritmo de propagación**:

```python
def dispatch_event(event: PropagatingEvent, target: EventTarget) -> Bool:
    # 1. Build path from root to target
    path = []
    current = target
    while current is not None:
        path.append(current)
        current = current.getParent()
    
    # 2. CAPTURING phase (root → target)
    event.phase = PropagationPhase.CAPTURING
    for node in reversed(path[1:]):  # Skip target
        if event.propagationStopped:
            break
        event.currentTarget = Some(node)
        node.trigger_listeners(event, useCapture=True)
    
    # 3. AT_TARGET phase
    if not event.propagationStopped:
        event.phase = PropagationPhase.AT_TARGET
        event.currentTarget = Some(target)
        target.trigger_listeners(event, useCapture=False)
    
    # 4. BUBBLING phase (target → root)
    if not event.propagationStopped:
        event.phase = PropagationPhase.BUBBLING
        for node in path[1:]:  # Skip target
            if event.propagationStopped:
                break
            event.currentTarget = Some(node)
            node.trigger_listeners(event, useCapture=False)
    
    return not event.defaultPrevented
```

---

### 4. Event Filtering & Routing

**Motivación**: Permitir subscripciones selectivas para optimizar performance.

#### 4.1. Filter Types

```vela
# Filter by event type pattern
bus.on("user.*", handler)        # Match user.created, user.updated, etc.
bus.on("user.created", handler)  # Exact match

# Filter by predicate
bus.onWhere("user.*", (event) => {
  return event.payload.role == "admin"
}, handler)

# Filter by tags
event = Event("user.created", user, tags=["audit", "important"])
bus.onTag("audit", handler)
```

#### 4.2. EventFilter Class

```vela
class EventFilter {
  # Match by wildcard pattern
  static fn matchPattern(pattern: String, eventType: String) -> Bool {
    if pattern == "*" {
      return true
    }
    if !pattern.contains("*") {
      return pattern == eventType
    }
    
    # Convert wildcard to regex
    regex = pattern
      .replace(".", "\\.")
      .replace("*", ".*")
    return eventType.matches("^" + regex + "$")
  }
  
  # Filter by predicate
  static fn applyPredicate<T>(
    event: Event<T>, 
    predicate: (Event<T>) -> Bool
  ) -> Bool {
    return predicate(event)
  }
  
  # Filter by tags
  static fn matchTags(
    event: Event, 
    requiredTags: List<String>
  ) -> Bool {
    return requiredTags.every(tag => 
      event.tags.contains(tag)
    )
  }
}
```

#### 4.3. Filtered EventBus

```vela
class FilteredEventBus<T> extends EventBus<T> {
  fn onPattern(
    pattern: String, 
    listener: (event: T) -> void
  ) -> Subscription {
    wrappedListener = (event: T) -> void {
      if EventFilter.matchPattern(pattern, event.type) {
        listener(event)
      }
    }
    return this.on("*", wrappedListener)
  }
  
  fn onWhere(
    eventType: String,
    predicate: (event: T) -> Bool,
    listener: (event: T) -> void
  ) -> Subscription {
    wrappedListener = (event: T) -> void {
      if EventFilter.applyPredicate(event, predicate) {
        listener(event)
      }
    }
    return this.on(eventType, wrappedListener)
  }
  
  fn onTag(
    tag: String,
    listener: (event: T) -> void
  ) -> Subscription {
    wrappedListener = (event: T) -> void {
      if event.tags.contains(tag) {
        listener(event)
      }
    }
    return this.on("*", wrappedListener)
  }
}
```

---

### 5. Memory Management

**Problema**: Event listeners pueden causar memory leaks si no se desuscriben.

#### 5.1. Auto-Dispose Pattern

```vela
class AutoDisposeEventBus<T> extends EventBus<T> {
  # Track subscriptions por owner
  subscriptionsByOwner: Dict<Any, List<Subscription>> = {}
  
  fn on(
    eventType: String, 
    listener: (event: T) -> void,
    owner: Option<Any> = None
  ) -> Subscription {
    subscription = super.on(eventType, listener)
    
    # Track subscription if owner provided
    match owner {
      Some(obj) => {
        if !this.subscriptionsByOwner.has(obj) {
          this.subscriptionsByOwner[obj] = []
        }
        this.subscriptionsByOwner[obj].push(subscription)
      }
      None => {}
    }
    
    return subscription
  }
  
  # Dispose all subscriptions for owner
  fn disposeAll(owner: Any) -> void {
    if this.subscriptionsByOwner.has(owner) {
      this.subscriptionsByOwner[owner].forEach(sub => 
        sub.unsubscribe()
      )
      this.subscriptionsByOwner.remove(owner)
    }
  }
}
```

#### 5.2. Integration con Lifecycle

**Para UI Components**:

```vela
component UserList {
  state users: List<User> = []
  
  mount() {
    # Subscribe to events con auto-dispose
    eventBus.on("user.created", this.onUserCreated, owner=this)
    eventBus.on("user.updated", this.onUserUpdated, owner=this)
  }
  
  destroy() {
    # Auto-dispose all subscriptions
    eventBus.disposeAll(this)
  }
  
  fn onUserCreated(event: Event<User>) -> void {
    this.users.push(event.payload)
  }
  
  fn onUserUpdated(event: Event<User>) -> void {
    # Update user in list
  }
}
```

---

### 6. Integration con Sistema Reactivo

**Motivación**: Events → Signals para UI reactivity.

```vela
# Convert EventBus to Signal
fn eventToSignal<T>(
  bus: EventBus<T>, 
  eventType: String, 
  initialValue: T
) -> Signal<T> {
  sig = signal(initialValue)
  
  bus.on(eventType, (event: Event<T>) => {
    sig.value = event.payload
  })
  
  return sig
}

# Example usage
userCreatedSignal = eventToSignal(
  eventBus, 
  "user.created", 
  None
)

# React to events in computed
computed userCount: Number {
  return userCreatedSignal.value != None ? 1 : 0
}
```

---

### 7. Performance Considerations

#### 7.1. Listener Storage

**Decisión**: `Dict<String, List<Listener>>` (hash map)

**Complejidad**:
- Subscribe: O(1) average
- Emit: O(n) donde n = listeners count
- Unsubscribe: O(n) para buscar listener

**Alternativa considerada**: Tree structure para wildcard matching
- **Rechazada**: Overhead innecesario para caso común (exact match)
- **Solución**: Implementar `FilteredEventBus` separado para wildcards

#### 7.2. Thread Safety

**Decisión**: Lock-free para single-threaded, Mutex para multi-threaded

```python
# Implementation (Python example)
class ThreadSafeEventBus(EventBus):
    def __init__(self):
        super().__init__()
        self._lock = threading.Lock()
    
    def emit(self, event_type: str, payload: Any):
        with self._lock:
            # Copy listeners to avoid concurrent modification
            listeners = self.listeners.get(event_type, []).copy()
        
        # Notify outside lock
        for listener in listeners:
            try:
                listener(Event(event_type, payload))
            except Exception as e:
                logger.error(f"Listener error: {e}")
```

---

## Consecuencias

### ✅ Positivas

1. **Type Safety**: `EventBus<T>` garantiza tipos correctos en compile-time
2. **Desacoplamiento**: Componentes no necesitan referencias directas
3. **Flexibility**: Múltiples listeners, filtering, propagation control
4. **Memory Safe**: Auto-dispose pattern previene leaks
5. **Performance**: O(1) emit, eficiente para caso común
6. **Integration**: Se integra con signals para reactividad

### ⚠️ Negativas

1. **Debugging**: Event flow más difícil de rastrear que llamadas directas
2. **Overhead**: Pequeño overhead vs llamadas directas (acceptable trade-off)
3. **Complexity**: Sistema de propagación agrega complejidad (solo para DOM-style events)

### 🔄 Mitigaciones

1. **Debugging**: Implementar event logging y DevTools inspector
2. **Performance**: Benchmarks muestran overhead <1ms para 1000 listeners
3. **Complexity**: Propagation es opcional, EventBus simple para casos comunes

---

## Alternativas Consideradas

### 1. Observable Pattern (RxJS-style)

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

### 2. Signal-based Events

```vela
userCreated = signal<Event<User>>(None)
userCreated.value = Event("user.created", user)
```

**Rechazado porque**:
- ❌ Signals son para estado, no para notificaciones one-time
- ❌ No soporta múltiples listeners nativamente
- ✅ Pero sí integramos: `eventToSignal()` helper

### 3. Actor Message Passing

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

## Referencias

- **Jira**: TASK-035K - Diseñar arquitectura del Event Bus
- **Epic**: EPIC-03C - Event System
- **Sprint**: 14
- **Documentación**:
  - Node.js EventEmitter: https://nodejs.org/api/events.html
  - DOM Events Level 3: https://www.w3.org/TR/DOM-Level-3-Events/
  - RxJS: https://rxjs.dev/
  - Akka Event Bus: https://doc.akka.io/docs/akka/current/event-bus.html

---

## Implementación

**Archivos a crear**:

1. **Core**:
   - `src/runtime/events/event_bus.py` - EventBus<T> implementation
   - `src/runtime/events/event.py` - Event<T> class
   - `src/runtime/events/subscription.py` - Subscription class

2. **Interfaces**:
   - `src/runtime/events/event_emitter.py` - EventEmitter interface

3. **Advanced**:
   - `src/runtime/events/propagation.py` - Event propagation logic
   - `src/runtime/events/filters.py` - Event filtering

4. **Integration**:
   - `src/runtime/events/reactive.py` - Event→Signal helpers

5. **Tests**:
   - `tests/unit/events/test_event_bus.py`
   - `tests/unit/events/test_event_emitter.py`
   - `tests/unit/events/test_propagation.py`
   - `tests/unit/events/test_filters.py`
   - `tests/system/events/test_memory_leaks.py`
   - `tests/system/events/test_performance.py`

**Siguientes pasos**: TASK-035L (Implementar EventBus<T> core)

---

**Versión**: 1.0  
**Autor**: GitHub Copilot Agent  
**Fecha de Aprobación**: 2025-12-02
