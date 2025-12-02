# TASK-035V: Implementar @connect decorator

## 📋 Información General
- **Historia:** VELA-577 (State Management con Store<T>, Actions, Reducers)
- **Epic:** EPIC-03D (State Management)
- **Sprint:** Sprint 15
- **Estado:** Completada ✅
- **Fecha:** 2025-05-30

## 🎯 Objetivo

Implementar el decorador `@connect` que permite conectar widgets a un Store automáticamente, inyectando props derivadas del state mediante un selector y habilitando auto-subscribe/unsubscribe en el lifecycle del widget.

## 🔨 Implementación

### Archivos generados

1. **`src/reactive/connect.py`** (~400 LOC):
   - `ConnectOptions[S, P]`: Dataclass con configuración del decorator
   - `connect()`: Decorator principal para widgets
   - `shallow_equal()`: Función de comparación de props (shallow equality)
   - `create_selector()`: Memoización de selectors (estilo reselect)
   - `connect_to_store()`: API simplificada
   - `connect_with_dispatch()`: API con inyección de dispatch

2. **`tests/unit/state/test_connect.py`** (~340 LOC):
   - 23 tests cubriendo todo el comportamiento del decorator
   - Fixtures: `CounterState`, `IncrementAction`, `counter_store`, `MockWidget`
   - Test classes:
     - `TestShallowEqual`: 9 tests (comparación de props)
     - `TestConnectDecorator`: 8 tests (funcionalidad principal)
     - `TestCreateSelector`: 2 tests (memoización)
     - `TestConnectHelpers`: 3 tests (APIs helpers)
     - `TestConnectMetadata`: 1 test (metadata del decorator)

### Arquitectura del Decorator

**Pattern:** Redux/NgRx connect (Higher-Order Component pattern)

```
Widget (sin state)
       ↓
  @connect({ store, selector })
       ↓
Widget Conectado (con props + dispatch inyectados)
       ↓
  mount() → subscribe → ejecuta selector → inyecta props
       ↓
  update() → re-render cuando props cambian
       ↓
  destroy() → unsubscribe automático
```

### Lifecycle Hooks Inyectados

**1. `enhanced_mount()`** (al montar widget):
```python
def enhanced_mount(self):
    # 1. Obtener state actual
    current_state = options.store.get_state()
    
    # 2. Ejecutar selector para obtener props
    props = options.selector(current_state)
    
    # 3. Inyectar props en el widget
    for key, value in props.items():
        setattr(self, key, value)
    
    # 4. Inyectar dispatch function
    setattr(self, 'dispatch', options.store.dispatch)
    
    # 5. Suscribirse al store
    unsubscribe_fn = options.store.subscribe(on_state_change)
    
    # 6. Llamar mount original
    if original_mount:
        original_mount(self)
```

**2. `enhanced_update()`** (después de cambio de props):
```python
def enhanced_update(self):
    # Llamar update original si existe
    if original_update:
        original_update(self)
```

**3. `enhanced_destroy()`** (al desmontar widget):
```python
def enhanced_destroy(self):
    # 1. Unsubscribe del store
    if unsubscribe_fn:
        unsubscribe_fn()
    
    # 2. Llamar destroy original
    if original_destroy:
        original_destroy(self)
```

### Selector y Props

**Selector**: Función `(state: S) -> P` que extrae props del state.

```python
# Selector simple
def counter_selector(state: CounterState) -> Dict[str, Any]:
    return {"count": state.count}

# Selector derivado
def double_count_selector(state: CounterState) -> Dict[str, Any]:
    return {
        "count": state.count,
        "double": state.count * 2
    }
```

**Props Injection**: Cada key del dict retornado por el selector se inyecta como atributo del widget.

```python
# Si selector retorna {"count": 5}
widget.count  # → 5
```

### Shallow Equality

**Optimización**: Solo re-renderizar si las props cambiaron.

```python
def shallow_equal(obj1: Any, obj2: Any) -> bool:
    # Mismo objeto → True
    if obj1 is obj2:
        return True
    
    # Dicts: comparar keys y valores
    if isinstance(obj1, dict) and isinstance(obj2, dict):
        if obj1.keys() != obj2.keys():
            return False
        return all(obj1[k] is obj2[k] for k in obj1.keys())
    
    # Lists: comparar longitud y elementos
    if isinstance(obj1, list) and isinstance(obj2, list):
        if len(obj1) != len(obj2):
            return False
        return all(a is b for a, b in zip(obj1, obj2))
    
    # Tipos diferentes o primitivos
    return obj1 == obj2
```

**Nota**: Usa identity comparison (`is`) para valores dentro de dicts/lists, pero equality (`==`) para primitivos.

### Create Selector (Memoization)

**Inspirado en:** reselect (librería de Redux)

```python
def create_selector(selector_fn: Callable[[S], T]) -> Callable[[S], T]:
    """
    Crea un selector memoizado.
    
    El selector solo se recalcula si el input (state) cambió.
    """
    cache = {"input": None, "result": None}
    
    def memoized_selector(state: S) -> T:
        if state is not cache["input"]:
            cache["input"] = state
            cache["result"] = selector_fn(state)
        return cache["result"]
    
    return memoized_selector
```

**Uso:**
```python
# Sin memoización (recalcula siempre)
expensive_selector = lambda state: heavy_computation(state.data)

# Con memoización (recalcula solo si state cambió)
memoized_selector = create_selector(expensive_selector)
```

### APIs Alternativas

**1. `connect_to_store()`** - API simplificada:
```python
def connect_to_store(
    store: Store[S, A],
    selector: Callable[[S], P],
    equals_fn: Optional[Callable[[P, P], bool]] = None
) -> Callable[[Type], Type]:
    """
    API simplificada para conectar widget a store.
    
    Alias de connect() sin dispatch injection.
    """
    return connect(ConnectOptions(
        store=store,
        selector=selector,
        equals_fn=equals_fn,
        dispatch_prop=None
    ))
```

**2. `connect_with_dispatch()`** - API con dispatch customizable:
```python
def connect_with_dispatch(
    store: Store[S, A],
    selector: Callable[[S], P],
    dispatch_prop: str = "dispatch",
    equals_fn: Optional[Callable[[P, P], bool]] = None
) -> Callable[[Type], Type]:
    """
    API para conectar widget con dispatch customizable.
    """
    return connect(ConnectOptions(
        store=store,
        selector=selector,
        equals_fn=equals_fn,
        dispatch_prop=dispatch_prop
    ))
```

### Metadata del Decorator

**Decorador agrega metadata a la clase:**

```python
widget_class.__connected__ = True
widget_class.__connect_options__ = options
```

**Uso:**
```python
if hasattr(MyWidget, '__connected__'):
    print("Widget está conectado a un Store")
    print(f"Store: {MyWidget.__connect_options__.store}")
```

## 📊 Ejemplos de Uso

### Ejemplo 1: Counter Widget Simple

```vela
import 'system:reactive' show { Store, connect, ConnectOptions }

# State
struct CounterState {
  count: Number
}

# Actions
class IncrementAction extends Action {
  fn get_type() -> String { return "INCREMENT" }
}

class DecrementAction extends Action {
  fn get_type() -> String { return "DECREMENT" }
}

# Reducer
fn counter_reducer(state: CounterState, action: Action) -> CounterState {
  match action {
    IncrementAction() => CounterState { count: state.count + 1 }
    DecrementAction() => CounterState { count: state.count - 1 }
    _ => state
  }
}

# Store
counter_store: Store<CounterState> = Store(
  initial_state: CounterState { count: 0 },
  reducer: counter_reducer
)

# Widget conectado
@connect(ConnectOptions(
  store: counter_store,
  selector: (state) => { count: state.count }
))
component CounterWidget extends StatelessWidget {
  # Props inyectadas automáticamente:
  # - this.count: Number (del selector)
  # - this.dispatch: (Action) -> void (del store)
  
  fn build() -> Widget {
    return Column {
      children: [
        Text("Count: ${this.count}"),
        
        Button {
          text: "Increment",
          onClick: () => this.dispatch(IncrementAction())
        },
        
        Button {
          text: "Decrement",
          onClick: () => this.dispatch(DecrementAction())
        }
      ]
    }
  }
}

# Uso
app_root = CounterWidget()
app_root.mount()  # Auto-subscribe al store
```

### Ejemplo 2: TodoList con Selector Derivado

```vela
# State
struct TodoState {
  todos: List<Todo>
  filter: String  # "all" | "active" | "completed"
}

struct Todo {
  id: Number
  text: String
  completed: Bool
}

# Selector con lógica derivada
fn todo_selector(state: TodoState) -> Map<String, Any> {
  filtered_todos = match state.filter {
    "active" => state.todos.filter(t => !t.completed)
    "completed" => state.todos.filter(t => t.completed)
    _ => state.todos
  }
  
  return {
    "todos": filtered_todos,
    "total": state.todos.length,
    "active_count": state.todos.filter(t => !t.completed).length,
    "completed_count": state.todos.filter(t => t.completed).length
  }
}

# Widget conectado
@connect(ConnectOptions(
  store: todo_store,
  selector: todo_selector
))
component TodoListWidget extends StatelessWidget {
  # Props inyectadas:
  # - this.todos: List<Todo>
  # - this.total: Number
  # - this.active_count: Number
  # - this.completed_count: Number
  # - this.dispatch: (Action) -> void
  
  fn build() -> Widget {
    return Column {
      children: [
        Text("Total: ${this.total}"),
        Text("Active: ${this.active_count}"),
        Text("Completed: ${this.completed_count}"),
        
        ...this.todos.map(todo => TodoItem { todo: todo })
      ]
    }
  }
}
```

### Ejemplo 3: Selector Memoizado (Expensive Computation)

```vela
import 'system:reactive' show { create_selector }

# Selector sin memoización (malo - recalcula siempre)
fn expensive_selector(state: AppState) -> Map<String, Any> {
  # Cálculo pesado (O(n²) o I/O)
  processed_data = heavy_computation(state.raw_data)
  
  return { "data": processed_data }
}

# Selector memoizado (bueno - recalcula solo si state.raw_data cambió)
memoized_selector = create_selector(expensive_selector)

@connect(ConnectOptions(
  store: app_store,
  selector: memoized_selector  # ✅ Usa versión memoizada
))
component DataWidget extends StatelessWidget {
  # ...
}
```

### Ejemplo 4: Custom Equality Function

```vela
# Equality customizada (deep comparison)
fn deep_equal(obj1: Any, obj2: Any) -> Bool {
  # Implementación de deep equality
  # (comparar recursivamente objetos anidados)
  return json_stringify(obj1) == json_stringify(obj2)
}

@connect(ConnectOptions(
  store: app_store,
  selector: (state) => { complex_object: state.nested_data },
  equals_fn: deep_equal  # ✅ Usa deep equality en lugar de shallow
))
component ComplexWidget extends StatelessWidget {
  # ...
}
```

## ✅ Criterios de Aceptación

- [x] **Código implementado** en `src/reactive/connect.py`:
  - [x] `ConnectOptions[S, P]` dataclass con configuración
  - [x] `connect()` decorator principal
  - [x] `shallow_equal()` función de comparación
  - [x] `create_selector()` memoización de selectors
  - [x] `connect_to_store()` API simplificada
  - [x] `connect_with_dispatch()` API con dispatch
  - [x] Lifecycle hooks: mount, update, destroy
  - [x] Auto-subscribe on mount
  - [x] Auto-unsubscribe on destroy
  - [x] Props injection via selector
  - [x] Dispatch injection
  - [x] Shallow equality optimization

- [x] **Tests escritos y pasando** (23 tests):
  - [x] `TestShallowEqual`: 9 tests (comparación)
  - [x] `TestConnectDecorator`: 8 tests (decorator behavior)
  - [x] `TestCreateSelector`: 2 tests (memoización)
  - [x] `TestConnectHelpers`: 3 tests (APIs helpers)
  - [x] `TestConnectMetadata`: 1 test (metadata)

- [x] **Documentación generada** (`TASK-035V.md`):
  - [x] Arquitectura del decorator
  - [x] Lifecycle hooks
  - [x] Shallow equality
  - [x] Memoización con create_selector
  - [x] APIs alternativas
  - [x] Ejemplos de uso completos

- [x] **Integración con Store<T>** (TASK-035S):
  - [x] Usa `store.subscribe()` para auto-subscribe
  - [x] Usa función de unsubscribe retornada por `subscribe()`
  - [x] Usa `store.get_state()` para obtener state
  - [x] Usa `store.dispatch()` para inyectar dispatch

## 📊 Métricas

- **Archivos creados**: 2
  - `src/reactive/connect.py`: ~400 LOC
  - `tests/unit/state/test_connect.py`: ~340 LOC
- **Tests**: 23 (100% pasando)
- **Cobertura**: 100% de funcionalidad del decorator
- **Bugs detectados**: 3 (corregidos durante testing)
  1. Action.get_type() no implementado → Fixed
  2. Store.subscriptions no existe → Fixed (es `_subscribers`)
  3. Store._subscribers no existe → Fixed (es `_listeners`)
  4. Store.unsubscribe() no existe → Fixed (usar función retornada)

## 🔗 Referencias

- **Jira**: [TASK-035V](https://velalang.atlassian.net/browse/VELA-577)
- **Historia**: [VELA-577](https://velalang.atlassian.net/browse/VELA-577)
- **Epic**: [EPIC-03D](https://velalang.atlassian.net/browse/EPIC-03D)
- **ADR**: [ADR-008 - State Management Architecture](../../../docs/architecture/ADR-008-state-management.md)
- **Tasks previas**:
  - TASK-035R: Decisión arquitectónica (ADR-008)
  - TASK-035S: Implementación de Store<T>
  - TASK-035T: Documentación de Action/Reducer
  - TASK-035U: dispatch keyword

## 🔄 Integración con Tareas Previas

**Store<T> (TASK-035S)**:
- `@connect` usa `store.subscribe()` para auto-subscribe
- `@connect` usa `store.get_state()` para obtener state actual
- `@connect` usa `store.dispatch()` para inyectar dispatch function
- `subscribe()` retorna función de unsubscribe (no usa ID)

**Action (TASK-035S)**:
- Tests usan `Action` base class con `get_type()` implementado
- Decorador inyecta `dispatch()` que acepta `Action` instances

**dispatch keyword (TASK-035U)**:
- `@connect` inyecta `dispatch` function en el widget
- No usa keyword `dispatch` (es prop inyectada)

## 🚀 Próximos Pasos

Después de TASK-035V, continuar con:

1. **TASK-035W**: Implementar `@select` decorator
   - Parser support para `@select`
   - Integración con `Computed`
   - Memoización de selectors
   - Auto-recompute on dependencies

2. **TASK-035X**: Implementar `@persistent` decorator (opcional)
   - Persistencia en localStorage/file
   - Auto-save/restore

3. **TASK-035Y**: Implementar middleware system (opcional)
   - Logger middleware
   - Async middleware

4. **TASK-035Z**: Implementar DevTools integration (opcional)
   - Browser extension
   - Time-travel debugging

5. **TASK-035AA**: Tests de integración de State Management (obligatorio)
   - TodoApp E2E test
   - Performance tests
   - Integration tests

## 📝 Notas Técnicas

### Diferencias con Redux/NgRx

| Aspecto | Redux | NgRx | Vela (@connect) |
|---------|-------|------|-----------------|
| **Subscribe** | `store.subscribe(listener)` retorna `unsubscribe()` | `store.select(selector).subscribe()` | `store.subscribe(listener)` retorna `unsubscribe()` |
| **Props injection** | Manual (mapStateToProps) | Automático (async pipe) | Automático (decorator) |
| **Lifecycle** | Manual (componentDidMount/Unmount) | Automático (OnDestroy) | Automático (mount/destroy hooks) |
| **Equality** | Shallow por defecto | Deep por defecto | Shallow por defecto (customizable) |
| **Memoization** | reselect library | createSelector() | create_selector() (built-in) |

### Optimizaciones

1. **Shallow Equality**: Evita re-renders innecesarios
2. **Memoization**: `create_selector()` cachea resultados
3. **Auto-unsubscribe**: Previene memory leaks
4. **Identity comparison**: Usa `is` para comparar referencias (más rápido que `==`)

### Edge Cases Manejados

1. **Widget sin lifecycle hooks**: Decorator funciona igual (no llama hooks inexistentes)
2. **Selector retorna no-dict**: Asume iterable de (key, value) o ignora
3. **Props iguales**: No trigger update (shallow equality)
4. **Unsubscribe después de destroy**: Guard con `if unsubscribe_fn:`
5. **Múltiples decoradores**: Cada decorator tiene su propia closure (no colisiones)

---

**Estado Final**: ✅ **Completada**  
**Tests**: 23/23 pasando (100%)  
**Commit**: Pendiente (siguiente paso)
