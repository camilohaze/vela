# TASK-035T: Documentar Action y Reducer Types

## 📋 Información General
- **Historia:** VELA-577 - State Management
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Nota:** Action y Reducer fueron implementados en TASK-035S, esta tarea documenta su uso

## 🎯 Objetivo
Documentar en detalle los tipos Action y Reducer del State Management, proporcionando guías de uso, ejemplos y mejores prácticas para desarrolladores que trabajen con el Store de Vela.

## 📚 Documentación

### 1. Action Types

**Actions** son objetos inmutables que describen **"qué pasó"** en la aplicación. Son la única fuente de información para el Store.

#### 1.1. Action (Abstract Base Class)

```python
from abc import ABC, abstractmethod
from typing import Dict, Any

class Action(ABC):
    """
    Clase base abstracta para todas las acciones.
    
    Todas las acciones deben implementar:
    - get_type(): Retorna el tipo de acción
    - to_dict(): Serializa la acción a diccionario
    """
    
    @abstractmethod
    def get_type(self) -> str:
        """Retorna el tipo de acción como string."""
        pass
    
    @abstractmethod
    def to_dict(self) -> Dict[str, Any]:
        """Serializa la acción a diccionario para DevTools."""
        pass
```

**Propósito:**
- ✅ Contrato común para todas las acciones
- ✅ Serialización para DevTools
- ✅ Type safety mediante herencia

---

#### 1.2. SimpleAction - Acciones sin payload

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class SimpleAction(Action):
    """
    Acción simple sin datos adicionales.
    
    Ejemplo: "LOGIN", "LOGOUT", "RESET", "CLEAR"
    """
    type: str
    
    def get_type(self) -> str:
        return self.type
    
    def to_dict(self) -> Dict[str, Any]:
        return {"type": self.type}
```

**Características:**
- ✅ `@dataclass(frozen=True)` → Inmutable
- ✅ Solo contiene el tipo de acción
- ✅ Ideal para eventos sin datos

**Ejemplo de uso:**

```python
# Crear acciones simples
login = SimpleAction("USER_LOGIN")
logout = SimpleAction("USER_LOGOUT")
reset = SimpleAction("RESET_STATE")

# Dispatch
store.dispatch(login)
store.dispatch(logout)
store.dispatch(reset)
```

**En Vela (futuro):**
```vela
# Acciones simples como enums
enum AppAction {
  UserLogin,
  UserLogout,
  ResetState
}

dispatch(AppAction.UserLogin)
```

---

#### 1.3. PayloadAction<T> - Acciones con payload tipado

```python
from typing import Generic, TypeVar

T = TypeVar('T')

@dataclass(frozen=True)
class PayloadAction(Action, Generic[T]):
    """
    Acción con payload tipado.
    
    El payload puede ser cualquier tipo: Number, String, Dict, Object, etc.
    """
    type: str
    payload: T
    
    def get_type(self) -> str:
        return self.type
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": self.type,
            "payload": self.payload
        }
```

**Características:**
- ✅ Genérico: `PayloadAction[T]` para cualquier tipo
- ✅ Type-safe: El payload está tipado
- ✅ Inmutable

**Ejemplo de uso:**

```python
# Payload simple (Number)
increment = PayloadAction("INCREMENT_BY", 5)
store.dispatch(increment)

# Payload complejo (Dict)
add_todo = PayloadAction("ADD_TODO", {
    "id": 1,
    "text": "Learn Vela State Management",
    "completed": False
})
store.dispatch(add_todo)

# Payload con objeto custom
set_user = PayloadAction("SET_USER", User(
    id=123,
    name="Alice",
    email="alice@vela.dev"
))
store.dispatch(set_user)
```

**En Vela (futuro):**
```vela
# Acciones con datos asociados
enum TodoAction {
  Add(text: String),
  Toggle(id: Number),
  Remove(id: Number),
  Update(id: Number, text: String)
}

dispatch(TodoAction.Add("Learn Vela"))
dispatch(TodoAction.Toggle(1))
dispatch(TodoAction.Update(2, "Updated text"))
```

---

#### 1.4. ActionCreator - Factory Pattern

```python
class ActionCreator:
    """
    Factory para crear action creators reusables.
    
    Evita duplicar strings de tipos de acciones.
    """
    
    @staticmethod
    def simple(action_type: str) -> Callable[[], SimpleAction]:
        """Crea un action creator para SimpleAction."""
        def creator() -> SimpleAction:
            return SimpleAction(action_type)
        return creator
    
    @staticmethod
    def payload(action_type: str) -> Callable[[T], PayloadAction[T]]:
        """Crea un action creator para PayloadAction."""
        def creator(payload: T) -> PayloadAction[T]:
            return PayloadAction(action_type, payload)
        return creator
```

**Propósito:**
- ✅ Evitar typos en strings de tipos
- ✅ Reutilizar action creators
- ✅ API más limpia

**Ejemplo de uso:**

```python
# Definir action creators (una sola vez)
increment = ActionCreator.simple("INCREMENT")
decrement = ActionCreator.simple("DECREMENT")
set_value = ActionCreator.payload("SET_VALUE")
add_todo = ActionCreator.payload("ADD_TODO")

# Usar en toda la app (reusable)
store.dispatch(increment())
store.dispatch(decrement())
store.dispatch(set_value(42))
store.dispatch(add_todo({"text": "Learn Vela"}))
```

**Ventajas:**
- ✅ No más strings mágicos dispersos
- ✅ Auto-complete en IDE
- ✅ Refactoring seguro

---

#### 1.5. Actions Especiales

```python
# InitAction - Inicialización del store
class InitAction(SimpleAction):
    def __init__(self):
        super().__init__("@@INIT")

# ResetAction - Reset al estado inicial
class ResetAction(SimpleAction):
    def __init__(self):
        super().__init__("@@RESET")
```

**Uso interno:**
- `InitAction`: Dispatched automáticamente al crear el Store
- `ResetAction`: Usar con `store.reset()`

---

### 2. Reducer Types

**Reducers** son funciones puras que toman el estado actual y una acción, y retornan el **nuevo estado**.

**Regla de oro:** `(State, Action) → New State`

#### 2.1. Reducer Type Alias

```python
from typing import Callable, TypeVar

S = TypeVar('S')  # State type
A = TypeVar('A')  # Action type

# Reducer es simplemente un callable
Reducer = Callable[[S, A], S]
```

**Ejemplo básico:**

```python
def counter_reducer(state: Dict, action: Action) -> Dict:
    """
    Reducer simple para contador.
    
    IMPORTANTE: NO mutar el estado, retornar uno nuevo.
    """
    if action.get_type() == "INCREMENT":
        # ✅ CORRECTO: Spread operator para inmutabilidad
        return {**state, "count": state["count"] + 1}
    
    elif action.get_type() == "DECREMENT":
        return {**state, "count": state["count"] - 1}
    
    elif action.get_type() == "SET_VALUE":
        return {**state, "count": action.payload}
    
    # Siempre retornar estado actual si no coincide
    return state
```

**Reglas para Reducers:**

1. ✅ **Funciones PURAS** - Sin side effects:
   ```python
   # ❌ MAL: Side effects
   def bad_reducer(state, action):
       fetch_data()  # ❌ HTTP call
       print("action")  # ❌ Logging
       Math.random()  # ❌ No determinístico
       return state
   
   # ✅ BIEN: Puro
   def good_reducer(state, action):
       if action.get_type() == "INCREMENT":
           return {**state, "count": state["count"] + 1}
       return state
   ```

2. ✅ **NO MUTAR el estado** - Retornar uno nuevo:
   ```python
   # ❌ MAL: Mutación
   def bad_reducer(state, action):
       state["count"] += 1  # ❌ Mutando directamente
       return state
   
   # ✅ BIEN: Inmutabilidad
   def good_reducer(state, action):
       return {**state, "count": state["count"] + 1}  # Nuevo objeto
   ```

3. ✅ **Siempre retornar estado** - Incluso si no hay cambio:
   ```python
   def reducer(state, action):
       if action.get_type() == "KNOWN_ACTION":
           return new_state
       # ✅ Retornar estado actual para acciones desconocidas
       return state
   ```

---

#### 2.2. CombinedReducer - combineReducers Pattern

```python
class CombinedReducer:
    """
    Combina múltiples reducers en uno solo.
    
    Similar a combineReducers de Redux.
    Cada reducer maneja una slice del estado.
    """
    
    def __init__(self, reducers: Dict[str, Reducer]):
        self.reducers = reducers
    
    def reduce(self, state: S, action: A) -> S:
        """
        Aplica cada reducer a su slice.
        
        Optimización: Retorna el mismo objeto si ningún
        reducer cambió su slice (reference equality).
        """
        changed = False
        new_state = {}
        
        for key, reducer in self.reducers.items():
            prev_state_for_key = state.get(key)
            next_state_for_key = reducer(prev_state_for_key, action)
            
            new_state[key] = next_state_for_key
            
            # Detectar cambios por referencia
            if next_state_for_key is not prev_state_for_key:
                changed = True
        
        # Optimización: retornar mismo objeto si nada cambió
        return new_state if changed else state
```

**Ejemplo de uso:**

```python
# Reducers individuales
def counter_reducer(state, action):
    if state is None:
        state = {"count": 0}
    
    if action.get_type() == "INCREMENT":
        return {**state, "count": state["count"] + 1}
    return state

def todos_reducer(state, action):
    if state is None:
        state = {"todos": []}
    
    if action.get_type() == "ADD_TODO":
        return {**state, "todos": state["todos"] + [action.payload]}
    return state

def user_reducer(state, action):
    if state is None:
        state = {"user": None}
    
    if action.get_type() == "SET_USER":
        return {**state, "user": action.payload}
    return state

# Combinar en root reducer
root_reducer = CombinedReducer({
    "counter": counter_reducer,
    "todos": todos_reducer,
    "user": user_reducer
})

# Estado resultante:
# {
#   "counter": {"count": 0},
#   "todos": {"todos": []},
#   "user": {"user": None}
# }

# Usar en Store
store = Store(initial_state, root_reducer.reduce)
```

**Ventajas:**
- ✅ Separación de concerns (cada reducer maneja su dominio)
- ✅ Reusabilidad de reducers
- ✅ Testing más fácil (testear cada reducer por separado)
- ✅ Optimización automática (reference equality check)

---

#### 2.3. ReducerBuilder - Fluent API

```python
class ReducerBuilder:
    """
    Builder pattern para crear reducers con fluent API.
    
    Más legible que if/elif largo.
    """
    
    def __init__(self):
        self._handlers: Dict[str, Callable] = {}
        self._default_handler: Optional[Callable] = None
    
    def case(self, action_type: str, handler: Callable) -> 'ReducerBuilder':
        """Registra handler para un tipo de acción."""
        self._handlers[action_type] = handler
        return self  # Chaining
    
    def default(self, handler: Callable) -> 'ReducerBuilder':
        """Handler por defecto (opcional)."""
        self._default_handler = handler
        return self
    
    def build(self) -> Reducer:
        """Construye el reducer final."""
        def reducer(state: S, action: A) -> S:
            action_type = action.get_type()
            
            if action_type in self._handlers:
                return self._handlers[action_type](state, action)
            
            if self._default_handler:
                return self._default_handler(state, action)
            
            return state  # No match
        
        return reducer
```

**Ejemplo de uso:**

```python
# Fluent API - más legible
counter_reducer = (
    ReducerBuilder()
    .case("INCREMENT", lambda state, action: {
        **state, "count": state["count"] + 1
    })
    .case("DECREMENT", lambda state, action: {
        **state, "count": state["count"] - 1
    })
    .case("SET_VALUE", lambda state, action: {
        **state, "count": action.payload
    })
    .case("RESET", lambda state, action: {
        "count": 0
    })
    .default(lambda state, action: state)  # Opcional
    .build()
)

# Usar en Store
store = Store({"count": 0}, counter_reducer)
```

**Ventajas:**
- ✅ Más legible que if/elif largo
- ✅ Method chaining (fluent)
- ✅ Pattern matching style

---

#### 2.4. create_reducer() - Helper Function

```python
def create_reducer(
    handlers: Dict[str, Callable],
    default: Optional[Callable] = None
) -> Reducer:
    """
    Helper para crear reducer desde dict de handlers.
    
    Alternativa más simple que ReducerBuilder.
    """
    def reducer(state: S, action: A) -> S:
        action_type = action.get_type()
        
        if action_type in handlers:
            return handlers[action_type](state, action)
        
        if default:
            return default(state, action)
        
        return state
    
    return reducer
```

**Ejemplo de uso:**

```python
# Dict de handlers
counter_reducer = create_reducer({
    "INCREMENT": lambda state, action: {**state, "count": state["count"] + 1},
    "DECREMENT": lambda state, action: {**state, "count": state["count"] - 1},
    "SET_VALUE": lambda state, action: {**state, "count": action.payload},
    "RESET": lambda state, action: {"count": 0}
}, default=lambda state, action: state)

# Más compacto que ReducerBuilder
store = Store({"count": 0}, counter_reducer)
```

---

### 3. Ejemplo Completo: TodoApp

```python
from typing import Dict, List
from src.reactive.action import SimpleAction, PayloadAction, ActionCreator
from src.reactive.reducer import create_reducer
from src.reactive.store import Store

# ============================================
# 1. DEFINIR ESTADO
# ============================================

# Type alias para claridad
TodoState = Dict[str, List[Dict]]

initial_state: TodoState = {
    "todos": []
}

# ============================================
# 2. DEFINIR ACTIONS
# ============================================

# Action creators
add_todo = ActionCreator.payload("ADD_TODO")
toggle_todo = ActionCreator.payload("TOGGLE_TODO")
remove_todo = ActionCreator.payload("REMOVE_TODO")
clear_completed = ActionCreator.simple("CLEAR_COMPLETED")

# ============================================
# 3. DEFINIR REDUCER
# ============================================

def todos_reducer(state: TodoState, action) -> TodoState:
    """Reducer para lista de TODOs."""
    
    if action.get_type() == "ADD_TODO":
        new_todo = {
            "id": len(state["todos"]) + 1,
            "text": action.payload,
            "completed": False
        }
        return {
            **state,
            "todos": state["todos"] + [new_todo]
        }
    
    elif action.get_type() == "TOGGLE_TODO":
        todo_id = action.payload
        new_todos = [
            {**todo, "completed": not todo["completed"]}
            if todo["id"] == todo_id else todo
            for todo in state["todos"]
        ]
        return {**state, "todos": new_todos}
    
    elif action.get_type() == "REMOVE_TODO":
        todo_id = action.payload
        new_todos = [
            todo for todo in state["todos"]
            if todo["id"] != todo_id
        ]
        return {**state, "todos": new_todos}
    
    elif action.get_type() == "CLEAR_COMPLETED":
        new_todos = [
            todo for todo in state["todos"]
            if not todo["completed"]
        ]
        return {**state, "todos": new_todos}
    
    return state

# ============================================
# 4. CREAR STORE
# ============================================

store = Store(initial_state, todos_reducer, enable_devtools=True)

# ============================================
# 5. SUBSCRIBE A CAMBIOS
# ============================================

def on_state_change(state):
    print(f"📝 TODOs: {len(state['todos'])} items")
    for todo in state["todos"]:
        status = "✅" if todo["completed"] else "⬜"
        print(f"  {status} {todo['text']}")

unsubscribe = store.subscribe(on_state_change)

# ============================================
# 6. DISPATCH ACTIONS
# ============================================

# Agregar TODOs
store.dispatch(add_todo("Learn Vela"))
store.dispatch(add_todo("Build awesome app"))
store.dispatch(add_todo("Deploy to production"))

# Output:
# 📝 TODOs: 1 items
#   ⬜ Learn Vela
# 📝 TODOs: 2 items
#   ⬜ Learn Vela
#   ⬜ Build awesome app
# 📝 TODOs: 3 items
#   ⬜ Learn Vela
#   ⬜ Build awesome app
#   ⬜ Deploy to production

# Toggle completado
store.dispatch(toggle_todo(1))
# Output:
# 📝 TODOs: 3 items
#   ✅ Learn Vela
#   ⬜ Build awesome app
#   ⬜ Deploy to production

# Remover TODO
store.dispatch(remove_todo(2))
# Output:
# 📝 TODOs: 2 items
#   ✅ Learn Vela
#   ⬜ Deploy to production

# Limpiar completados
store.dispatch(clear_completed())
# Output:
# 📝 TODOs: 1 items
#   ⬜ Deploy to production

# ============================================
# 7. TIME-TRAVEL (DevTools)
# ============================================

# Deshacer último cambio
store.undo()
# Output:
# 📝 TODOs: 2 items
#   ✅ Learn Vela
#   ⬜ Deploy to production

# Ver historial
history = store.get_history()
for entry in history:
    print(f"{entry['action']['type']} → {len(entry['state']['todos'])} todos")

# ============================================
# 8. CLEANUP
# ============================================

unsubscribe()
```

---

## ✅ Mejores Prácticas

### 1. Naming Conventions

```python
# ✅ ACTIONS: SCREAMING_SNAKE_CASE
"USER_LOGIN"
"ADD_TODO"
"FETCH_DATA_SUCCESS"

# ✅ Action creators: camelCase
add_todo = ActionCreator.payload("ADD_TODO")
login_user = ActionCreator.simple("USER_LOGIN")

# ✅ Reducers: snake_case con sufijo _reducer
def todos_reducer(state, action): ...
def user_reducer(state, action): ...
```

### 2. Organización de Archivos

```
src/store/
├── actions/
│   ├── todos.py       # add_todo, toggle_todo, etc.
│   ├── user.py        # login, logout, etc.
│   └── ui.py          # open_modal, close_modal, etc.
│
├── reducers/
│   ├── todos.py       # todos_reducer
│   ├── user.py        # user_reducer
│   ├── ui.py          # ui_reducer
│   └── root.py        # root_reducer (CombinedReducer)
│
└── store.py           # Store instance
```

### 3. Testing

```python
# Test de reducer (simple)
def test_counter_increment():
    state = {"count": 0}
    action = SimpleAction("INCREMENT")
    
    new_state = counter_reducer(state, action)
    
    assert new_state["count"] == 1
    assert state["count"] == 0  # Estado original no mutado

# Test de reducer con payload
def test_add_todo():
    state = {"todos": []}
    action = PayloadAction("ADD_TODO", "Learn Vela")
    
    new_state = todos_reducer(state, action)
    
    assert len(new_state["todos"]) == 1
    assert new_state["todos"][0]["text"] == "Learn Vela"
```

---

## 🔗 Referencias

- **Implementación:** `src/reactive/action.py`, `src/reactive/reducer.py`
- **Tests:** `tests/unit/state/test_action.py`, `tests/unit/state/test_reducer.py`
- **Historia:** [VELA-577](https://velalang.atlassian.net/browse/VELA-577)
- **ADR:** [ADR-008 - State Management Architecture](../../architecture/ADR-008-state-management-architecture.md)

---

**Última actualización:** 2025-12-02  
**Versión:** 1.0.0
