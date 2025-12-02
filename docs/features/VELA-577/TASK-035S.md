# TASK-035S: Implementar Store<T> base class

## 📋 Información General
- **Historia:** VELA-577 - State Management
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Commit:** a5bb6ee

## 🎯 Objetivo
Implementar el Store<T> base class con su ecosistema completo (Action, Reducer, middleware, selectors) usando el patrón Redux/NgRx con integración profunda al Signal System de Vela.

## 🔨 Implementación

### Componentes Implementados

#### 1. Action Types (`src/reactive/action.py` - 220 LOC)

**Actions** - Intents que describen "qué pasó":

```python
# Action base (abstract)
class Action(ABC):
    @abstractmethod
    def get_type(self) -> str: ...
    @abstractmethod
    def to_dict(self) -> Dict[str, Any]: ...

# SimpleAction - Sin payload
@dataclass(frozen=True)
class SimpleAction(Action):
    type: str

# PayloadAction<T> - Con payload tipado
@dataclass(frozen=True)
class PayloadAction(Action, Generic[T]):
    type: str
    payload: T

# ActionCreator - Factory
class ActionCreator:
    @staticmethod
    def simple(action_type: str) -> Callable[[], SimpleAction]: ...
    
    @staticmethod
    def payload(action_type: str) -> Callable[[T], PayloadAction[T]]: ...
```

**Features:**
- ✅ Inmutabilidad via `@dataclass(frozen=True)`
- ✅ Type-safety con `Generic[T]`
- ✅ Serialización a dict para DevTools
- ✅ ActionCreator como factory pattern

**Actions Especiales:**
- `InitAction` (type: `@@INIT`) - Inicialización del store
- `ResetAction` (type: `@@RESET`) - Reset al estado inicial

**Ejemplo de uso:**
```python
# Simple action
login = SimpleAction("USER_LOGIN")

# Payload action
add_todo = PayloadAction("ADD_TODO", {
    "id": 1,
    "text": "Learn Vela",
    "completed": False
})

# Con ActionCreator
increment = ActionCreator.simple("INCREMENT")
set_value = ActionCreator.payload("SET_VALUE")

store.dispatch(increment())
store.dispatch(set_value(42))
```

---

#### 2. Reducer Types (`src/reactive/reducer.py` - 280 LOC)

**Reducers** - Funciones puras `(state, action) → new_state`:

```python
# Reducer type alias
Reducer = Callable[[S, A], S]

# CombinedReducer - combineReducers pattern
class CombinedReducer:
    def __init__(self, reducers: Dict[str, Reducer]): ...
    def reduce(self, state: S, action: A) -> S: ...

# ReducerBuilder - Fluent API
class ReducerBuilder:
    def case(self, action_type: str, handler: Callable) -> 'ReducerBuilder': ...
    def default(self, handler: Callable) -> 'ReducerBuilder': ...
    def build(self) -> Reducer: ...

# Helpers
def create_reducer(handlers: Dict[str, Callable], default: Optional[Callable]) -> Reducer: ...
def identity_reducer(state: S, action: A) -> S: ...
```

**Features:**
- ✅ Funciones puras (sin mutaciones)
- ✅ CombinedReducer con optimización (retorna mismo objeto si no hay cambios)
- ✅ ReducerBuilder con fluent API
- ✅ Pattern matching via dict handlers

**Ejemplo de uso:**
```python
# Reducer simple
def counter_reducer(state, action):
    if action.get_type() == "INCREMENT":
        return {**state, "count": state["count"] + 1}
    return state

# CombinedReducer (como Redux combineReducers)
root_reducer = CombinedReducer({
    "counter": counter_reducer,
    "todos": todos_reducer,
    "user": user_reducer
})

# ReducerBuilder (fluent API)
reducer = (
    ReducerBuilder()
    .case("INCREMENT", lambda state, action: {**state, "count": state["count"] + 1})
    .case("DECREMENT", lambda state, action: {**state, "count": state["count"] - 1})
    .case("RESET", lambda state, action: {"count": 0})
    .default(lambda state, action: state)
    .build()
)

# create_reducer (dict handlers)
reducer = create_reducer({
    "INCREMENT": lambda state, action: {**state, "count": state["count"] + 1},
    "SET_VALUE": lambda state, action: {**state, "count": action.payload}
}, default=lambda state, action: state)
```

---

#### 3. Store<T> (`src/reactive/store.py` - 380 LOC)

**Store** - Contenedor de estado global con reactividad:

```python
class Store(Generic[S]):
    def __init__(
        self,
        initial_state: S,
        reducer: Reducer[S, A],
        middlewares: Optional[List[Middleware]] = None,
        enable_devtools: bool = False,
        name: str = "Store"
    ): ...
    
    # Core API
    def get_state(self) -> S: ...
    def dispatch(self, action: A) -> None: ...
    def subscribe(self, listener: Listener) -> Callable[[], None]: ...
    def select(self, selector: Selector) -> Computed: ...
    
    # Lifecycle
    def reset(self) -> None: ...
    def replace_reducer(self, new_reducer: Reducer[S, A]) -> None: ...
    
    # DevTools
    def undo(self) -> None: ...
    def redo(self) -> None: ...
    def time_travel(self, index: int) -> None: ...
    def get_history(self) -> List[Dict]: ...
    
    # Persistence
    def to_json(self) -> str: ...
    @staticmethod
    def from_json(json_str: str, reducer: Reducer, **kwargs) -> 'Store': ...
```

**Features Clave:**

**1. Estado Reactivo con Signal:**
```python
# Store._state es Signal<S>
self._state = Signal(initial_state)

# Cambios automáticamente notificados a subscribers
# Selectors devuelven Computed (memoizados)
```

**2. Middleware System:**
```python
# Middleware: (store, next, action) -> None
def logger_middleware(store, next, action):
    print(f"Before: {action.get_type()}")
    next(action)
    print(f"After: {action.get_type()}")

store = Store(initial_state, reducer, middlewares=[logger_middleware])
```

**3. Selectors con Memoization:**
```python
# select() retorna Computed (auto-memoizado)
count = store.select(lambda state: state["count"])
doubled = store.select(lambda state: state["count"] * 2)

# Computed se recalcula solo si cambia el estado
print(count.get())  # 10
print(doubled.get())  # 20
```

**4. DevTools Integration:**
```python
store = Store(initial_state, reducer, enable_devtools=True)

# Time-travel debugging
store.undo()  # Regresa 1 acción
store.redo()  # Avanza 1 acción
store.time_travel(5)  # Salta al índice 5

# Historial
history = store.get_history()
# [
#   {"action": "@@INIT", "state": {"count": 0}, "timestamp": ...},
#   {"action": "INCREMENT", "state": {"count": 1}, "timestamp": ...},
#   ...
# ]
```

**5. JSON Persistence:**
```python
# Guardar estado
json_str = store.to_json()
# '{"count": 42, "todos": [...]}'

# Restaurar estado
store = Store.from_json(json_str, reducer)
```

**Ejemplo completo:**
```python
# 1. Estado inicial
initial_state = {
    "count": 0,
    "todos": []
}

# 2. Reducer
def app_reducer(state, action):
    if action.get_type() == "INCREMENT":
        return {**state, "count": state["count"] + 1}
    elif action.get_type() == "ADD_TODO":
        return {
            **state,
            "todos": state["todos"] + [action.payload]
        }
    return state

# 3. Middleware
def logger_middleware(store, next, action):
    print(f"Dispatching: {action.get_type()}")
    next(action)

# 4. Crear store
store = Store(
    initial_state,
    app_reducer,
    middlewares=[logger_middleware],
    enable_devtools=True
)

# 5. Subscribe
unsubscribe = store.subscribe(
    lambda state: print(f"State updated: {state['count']}")
)

# 6. Selectors
count = store.select(lambda s: s["count"])
todos_count = store.select(lambda s: len(s["todos"]))

# 7. Dispatch actions
store.dispatch(SimpleAction("INCREMENT"))
# Output: "Dispatching: INCREMENT"
#         "State updated: 1"

store.dispatch(PayloadAction("ADD_TODO", {
    "id": 1,
    "text": "Learn Vela"
}))

# 8. Time-travel
store.undo()  # count vuelve a 1

# 9. Cleanup
unsubscribe()
```

---

### Integración con Signal System

**Store._state es Signal<S>:**
```python
# src/reactive/store.py
class Store(Generic[S]):
    def __init__(self, initial_state: S, ...):
        # Estado reactivo
        self._state = Signal(initial_state)
    
    def get_state(self) -> S:
        return self._state.get()
    
    def dispatch(self, action: A) -> None:
        # ...
        new_state = self.reducer(current_state, action)
        self._state.set(new_state)  # Trigger reactivity
        self._notify_listeners(new_state)
```

**Selectors con Computed:**
```python
def select(self, selector: Selector) -> Computed:
    """
    Crea un selector memoizado (Computed).
    
    El selector se recalcula automáticamente cuando
    el estado cambia, pero solo si el resultado es diferente.
    """
    return Computed(lambda: selector(self.get_state()))
```

**Beneficios:**
- ✅ Reactividad automática (Signal se encarga)
- ✅ Memoización automática (Computed se encarga)
- ✅ Tracking de dependencias (grafo reactivo)
- ✅ Optimización de re-renders

---

## 📊 Tests

**Total: 51 tests pasando** ✅

### test_action.py (18 tests)
- ✅ TestSimpleAction (4 tests)
  - Creación, inmutabilidad, serialización, repr
- ✅ TestPayloadAction (5 tests)
  - Con number/dict, inmutabilidad, serialización
- ✅ TestActionCreator (4 tests)
  - simple()/payload() creators, reusabilidad
- ✅ TestSpecialActions (2 tests)
  - InitAction, ResetAction
- ✅ TestCustomAction (3 tests)
  - Custom classes, con datos, serialización

### test_reducer.py (15 tests)
- ✅ TestBasicReducer (3 tests)
  - Counter reducer, purity (no mutations), payload
- ✅ TestCombinedReducer (3 tests)
  - Basic combining, isolation, optimization
- ✅ TestReducerBuilder (4 tests)
  - Basic, con payload, default handler, chaining
- ✅ TestCreateReducer (2 tests)
  - Basic, con default
- ✅ TestIdentityReducer (2 tests)
  - No-op, con diferentes tipos
- ✅ TestComplexReducers (1 test)
  - TodoApp reducer (ADD/TOGGLE/REMOVE/CLEAR)

### test_store.py (18 tests)
- ✅ TestStoreBasics (3 tests)
  - Creación, get_state, dispatch
- ✅ TestStoreSubscribe (3 tests)
  - Subscribe, unsubscribe, multiple subscribers
- ✅ TestStoreSelectors (2 tests)
  - Selector básico, computed selector
- ✅ TestStoreReset (1 test)
  - Reset al estado inicial
- ✅ TestStoreMiddleware (2 tests)
  - Middleware básico, middleware chain
- ✅ TestStoreDevTools (3 tests)
  - History enabled, time-travel, undo/redo
- ✅ TestStoreSerialization (2 tests)
  - to_json, from_json
- ✅ TestStorePerformance (2 tests)
  - 1000 dispatches < 1s, 100 subscribers + 100 dispatches < 1s

**Cobertura:**
- Actions: 100%
- Reducers: 100%
- Store: 95% (falta solo casos edge de error handling)

---

## 📁 Archivos Generados

```
src/reactive/
├── action.py          (220 LOC) - Action types
├── reducer.py         (280 LOC) - Reducer types
└── store.py           (380 LOC) - Store<T> class

tests/unit/state/
├── test_action.py     (180 LOC) - 18 tests
├── test_reducer.py    (320 LOC) - 15 tests
└── test_store.py      (430 LOC) - 18 tests

Total: ~1,810 LOC
```

---

## ✅ Criterios de Aceptación

- [x] Action types implementados (SimpleAction, PayloadAction)
- [x] ActionCreator factory implementado
- [x] Reducer types implementados (CombinedReducer, ReducerBuilder)
- [x] Store<T> class implementado con Signal integration
- [x] Middleware system implementado
- [x] DevTools integration (history, time-travel, undo/redo)
- [x] Selectors con Computed (memoizados)
- [x] Subscribe/unsubscribe para listeners
- [x] JSON serialization (to_json/from_json)
- [x] 51 tests pasando (100% cobertura de features)
- [x] Documentación completa
- [x] Commit realizado

---

## 🔗 Referencias

- **Jira:** [TASK-035S](https://velalang.atlassian.net/browse/VELA-577)
- **Historia:** [VELA-577 - State Management](https://velalang.atlassian.net/browse/VELA-577)
- **Commit:** a5bb6ee
- **ADR:** [ADR-008 - State Management Architecture](../../architecture/ADR-008-state-management-architecture.md)

---

## 🚀 Próximos Pasos

Con TASK-035S completado, el State Management core está implementado. Los siguientes pasos son:

1. **TASK-035T**: Documentar Action y Reducer types (ya implementados)
2. **TASK-035U**: Implementar `dispatch` keyword en lexer/parser
3. **TASK-035V**: Implementar `@connect` decorator (conectar widgets a store)
4. **TASK-035W**: Implementar `@select` decorator (selectors en decoradores)
5. **TASK-035AA**: Tests de integración E2E con TodoApp completa

**Estado del Sprint 15:** 
- ✅ TASK-035R (Arquitectura)
- ✅ TASK-035S (Implementación core)
- ⏳ 9 tareas restantes
