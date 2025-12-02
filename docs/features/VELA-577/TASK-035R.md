# TASK-035R: Diseñar arquitectura de Store

## 📋 Información General
- **Historia:** VELA-577 - State Management
- **Epic:** EPIC-03D - State Management
- **Sprint:** Sprint 15
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Prioridad:** P0

## 🎯 Objetivo

Diseñar la **arquitectura completa del sistema de State Management** de Vela, siguiendo el **patrón Redux/NgRx** con mejoras específicas del lenguaje:

- Store pattern Redux-style con actions y reducers
- Integración con Signal System (reactividad)
- Integración con DI System (inyección de stores)
- Type-safe actions, reducers y selectors
- Middleware system para cross-cutting concerns
- DevTools integration para debugging

## 🔨 Implementación

### Decisiones Arquitectónicas (ADR-008)

Se creó el **ADR-008: Arquitectura de State Management** con las siguientes decisiones:

#### 1. **Patrón Redux/NgRx con mejoras de Vela**

**Flujo unidireccional:**
```
Action → Middleware → Reducer → State → Selectors → UI
   ↑                                                   │
   └───────────────────dispatch()────────────────────┘
```

#### 2. **Componentes Principales**

##### a) **Store<T>** - Contenedor de estado global

```vela
store AppStore {
  # Estado reactivo (Signal System)
  state count: Number = 0
  state todos: List<Todo> = []
  
  # Reducer: (State, Action) → State
  reducer(state: AppState, action: Action) -> AppState {
    match action {
      IncrementAction => { ...state, count: state.count + 1 }
      AddTodoAction(todo) => { ...state, todos: state.todos.push(todo) }
      _ => state
    }
  }
}
```

**Características:**
- ✅ Estado inmutable (spread operator `...state`)
- ✅ State fields son Signals (reactividad automática)
- ✅ Type-safe: `Store<AppState>`
- ✅ Single source of truth

##### b) **Action** - Intención de cambiar estado

```vela
# Enum con datos asociados
enum TodoAction {
  Add(text: String)
  Toggle(id: Number)
  Remove(id: Number)
  Clear
}

# Union type
type CounterAction = IncrementAction | DecrementAction
```

##### c) **Reducer** - Función pura: (State, Action) → State

```vela
fn counterReducer(state: CounterState, action: CounterAction) -> CounterState {
  match action {
    IncrementAction => { count: state.count + 1 }
    DecrementAction(amount) => { count: state.count - amount }
  }
}
```

**Características:**
- ✅ Funciones puras (sin side effects)
- ✅ Pattern matching exhaustivo
- ✅ Testables sin mocks

##### d) **dispatch keyword** - Enviar acciones

```vela
dispatch(IncrementAction)
dispatch(AddTodoAction(text: "Learn Vela"))
```

**Características:**
- ✅ Keyword nativo (como `await`, `yield`)
- ✅ Type-checked
- ✅ Pasa por middleware chain

##### e) **@connect decorator** - Conectar widgets al store

```vela
@connect(store: AppStore, selector: (state) => state.count)
widget Counter {
  count: Number  # Inyectado desde selector
  
  fn build() -> Widget {
    return Button(
      text: "+",
      onPressed: () => dispatch(IncrementAction)
    )
  }
}
```

**Características:**
- ✅ Auto-subscribe al store
- ✅ Re-render solo cuando selector cambia
- ✅ Auto-cleanup al destruir widget

##### f) **@select decorator** - Memoización de selectores

```vela
store AppStore {
  state todos: List<Todo> = []
  
  @select
  computed completedTodos: List<Todo> {
    return this.todos.filter(t => t.completed)
  }
}
```

**Características:**
- ✅ Memoización automática
- ✅ Recompute solo si dependencias cambian
- ✅ Evita renders innecesarios

##### g) **Middleware System** - Interceptores

```vela
fn loggerMiddleware(store: Store, next: Dispatch, action: Action) -> void {
  print("Dispatching: ${action}")
  next(action)
  print("New state: ${store.getState()}")
}

store AppStore {
  middlewares: [loggerMiddleware, asyncMiddleware]
}
```

**Casos de uso:**
- Logging de acciones
- Async side effects (HTTP requests)
- Time-travel debugging
- Analytics tracking

##### h) **@persistent decorator** - Persistencia automática

```vela
@persistent(key: "app-state", storage: localStorage)
store AppStore {
  state user: Option<User> = None
}
```

##### i) **DevTools Integration** - Time-travel debugging

```vela
store AppStore {
  devTools: true
}
```

**Features:**
- Historial de acciones
- Time-travel (undo/redo)
- State snapshots
- Action replay

#### 3. **Integración con Sistemas Existentes**

##### Signal System (Sprint 11-12)

```vela
store AppStore {
  # state fields son Signals automáticamente
  state count: Number = 0  # Signal<Number>
  
  # computed usa Signal.computed()
  @select
  computed doubled: Number {
    return this.count * 2
  }
}
```

**Beneficios:**
- ✅ Reactividad automática
- ✅ Dependency tracking
- ✅ Updates eficientes con batch()

##### DI System (Sprint 13)

```vela
@injectable
store AppStore { ... }

@injectable
service TodoService {
  store: AppStore = inject(AppStore)
  
  fn addTodo(text: String) -> void {
    this.store.dispatch(AddTodoAction(text))
  }
}
```

#### 4. **Comparación con Alternativas**

| Feature | Redux | MobX | Vela Store ✅ |
|---------|-------|------|---------------|
| Patrón | Flux | Observable | Redux + Signals |
| Mutabilidad | Inmutable | Mutable | **Inmutable** |
| Boilerplate | Alto | Bajo | **Bajo (keywords)** |
| Type Safety | TS only | Débil | **Nativo** |
| Middleware | ✅ | ❌ | ✅ |
| DevTools | ✅ | ✅ | ✅ |
| Reactividad | External | Built-in | **Built-in (Signals)** |

#### 5. **Alternativas Rechazadas**

1. **MobX-style (mutabilidad)**:
   - ❌ Dificulta time-travel debugging
   - ❌ Cambios de estado no rastreables

2. **Recoil-style (atoms distribuidos)**:
   - ❌ Estado fragmentado
   - ❌ No single source of truth

3. **Context API-style**:
   - ❌ No pattern de acciones/reductores
   - ❌ Testing más difícil

### Arquitectura Visual

```
┌─────────────────────────────────────────────────────┐
│                   VELA STORE                        │
│                                                     │
│  ┌─────────────┐     ┌──────────────┐            │
│  │   State<T>  │────▶│  Reducers    │            │
│  │  (Signals)  │     │  (Pure fns)  │            │
│  └─────────────┘     └──────────────┘            │
│         ▲                    ▲                      │
│         │                    │                      │
│  ┌──────┴──────┐      ┌─────┴─────┐              │
│  │  @select    │      │  Actions   │              │
│  │ (Selectors) │      │ (Enums/    │              │
│  │             │      │  Types)    │              │
│  └─────────────┘      └───────────┘              │
│         ▲                    ▲                      │
│         │                    │                      │
│  ┌──────┴──────────────────┴─────────┐          │
│  │        Middleware Chain           │          │
│  │  (Logging, Async, DevTools, etc.) │          │
│  └────────────────────────────────────┘          │
│                                                     │
└─────────────────────────────────────────────────────┘
         ▲                            │
         │                            │
    @connect                      dispatch()
         │                            │
┌────────┴────────────────────────────▼─────────────┐
│              UI Components (Widgets)               │
│                                                     │
│  ┌──────────────┐      ┌──────────────┐          │
│  │   Counter    │      │   TodoList   │          │
│  │   Widget     │      │    Widget    │          │
│  └──────────────┘      └──────────────┘          │
└────────────────────────────────────────────────────┘
```

### Plan de Implementación

**10 tareas en orden:**

1. ✅ **TASK-035R**: Diseñar arquitectura (ADR-008)
2. **TASK-035S**: Implementar Store<T> base class
3. **TASK-035T**: Implementar Action y Reducer types
4. **TASK-035U**: Implementar dispatch keyword
5. **TASK-035V**: Implementar @connect decorator
6. **TASK-035W**: Implementar @select decorator
7. **TASK-035X**: Implementar @persistent decorator (P1)
8. **TASK-035Y**: Implementar middleware system (P1)
9. **TASK-035Z**: Implementar DevTools integration (P2)
10. **TASK-035AA**: Tests de State Management

### Estructura de Archivos

```
src/reactive/
├── store.py          # Store<T> base class
├── action.py         # Action types
├── reducer.py        # Reducer types
└── middleware.py     # Middleware system

src/stdlib/
└── store.vela        # Store APIs en Vela

src/lexer/
├── token.py          # + DISPATCH keyword
└── lexer.py          # Reconocer dispatch

src/parser/
└── parser.py         # + parse_dispatch_statement()

tests/unit/state/
├── test_store.py
├── test_reducers.py
├── test_middleware.py
├── test_selectors.py
└── test_persistence.py

tests/integration/
└── test_state_management.py

docs/features/VELA-577/
├── README.md
├── TASK-035R.md      # Este archivo
└── ...
```

## ✅ Criterios de Aceptación

- [x] ✅ ADR-008 creado con decisiones arquitectónicas
- [x] ✅ Arquitectura definida: Store, Action, Reducer, dispatch
- [x] ✅ Comparación con alternativas (Redux, MobX, Recoil)
- [x] ✅ Integración con Signal System diseñada
- [x] ✅ Integración con DI System diseñada
- [x] ✅ Middleware system diseñado
- [x] ✅ DevTools integration diseñado
- [x] ✅ Documentación TASK-035R.md completa
- [x] ✅ Estructura de archivos planificada
- [x] ✅ Plan de implementación definido

## 📊 Métricas

- **Archivos creados**:
  - `docs/architecture/ADR-008-state-management-architecture.md` (~550 líneas)
  - `docs/features/VELA-577/TASK-035R.md` (este archivo, ~350 líneas)
- **Decisiones arquitectónicas**: 9 componentes principales
- **Alternativas evaluadas**: 3 (MobX, Recoil, Context API)
- **Integraciones planificadas**: Signal System, DI System, Event System

## 🔗 Referencias

- **Jira**: [VELA-577](https://velalang.atlassian.net/browse/VELA-577)
- **ADR**: [ADR-008](../../architecture/ADR-008-state-management-architecture.md)
- **Epic**: EPIC-03D - State Management
- **Dependencies**:
  - ✅ Signal System (Sprint 11-12)
  - ✅ DI System (Sprint 13)
  - ✅ Event System (Sprint 14)

## 📝 Notas Adicionales

### Inspiración de Frameworks

**Vela Store combina lo mejor de:**

1. **Redux** (JavaScript):
   - Flujo unidireccional: Action → Reducer → State
   - Middleware chain
   - DevTools integration

2. **NgRx** (Angular):
   - Type-safe actions con discriminated unions
   - Selectors con memoization
   - Effects para async (middleware en Vela)

3. **Vuex** (Vue):
   - Integración profunda con reactividad
   - State mutations simples

4. **Zustand** (React):
   - API minimalista
   - Menos boilerplate

**Mejoras de Vela:**
- ✅ `dispatch` como keyword nativo (no función importada)
- ✅ `@connect`, `@select`, `@persistent` como decoradores first-class
- ✅ Reactividad built-in con Signal System
- ✅ Type safety nativa (no necesita TypeScript)

### Ejemplo Completo: TodoApp

```vela
# 1. Definir Actions
enum TodoAction {
  Add(text: String)
  Toggle(id: Number)
  Remove(id: Number)
  SetFilter(filter: TodoFilter)
}

enum TodoFilter {
  All
  Active
  Completed
}

# 2. Definir State
struct Todo {
  id: Number
  text: String
  completed: Bool
}

struct AppState {
  todos: List<Todo>
  filter: TodoFilter
}

# 3. Definir Store
@injectable
store TodoStore {
  state todos: List<Todo> = []
  state filter: TodoFilter = TodoFilter.All
  
  # Reducer
  reducer(state: AppState, action: TodoAction) -> AppState {
    match action {
      TodoAction.Add(text) => {
        newTodo = Todo {
          id: state.todos.length + 1,
          text: text,
          completed: false
        }
        return { ...state, todos: state.todos.push(newTodo) }
      }
      
      TodoAction.Toggle(id) => {
        updatedTodos = state.todos.map(todo => {
          if todo.id == id {
            return { ...todo, completed: !todo.completed }
          }
          return todo
        })
        return { ...state, todos: updatedTodos }
      }
      
      TodoAction.Remove(id) => {
        return { ...state, todos: state.todos.filter(t => t.id != id) }
      }
      
      TodoAction.SetFilter(filter) => {
        return { ...state, filter: filter }
      }
    }
  }
  
  # Selectors memoizados
  @select
  computed filteredTodos: List<Todo> {
    match this.filter {
      TodoFilter.All => this.todos
      TodoFilter.Active => this.todos.filter(t => !t.completed)
      TodoFilter.Completed => this.todos.filter(t => t.completed)
    }
  }
  
  @select
  computed activeCount: Number {
    return this.todos.filter(t => !t.completed).length
  }
}

# 4. UI Components
@connect(store: TodoStore, selector: (state) => state.filteredTodos)
widget TodoList {
  todos: List<Todo>
  
  fn build() -> Widget {
    return Column(
      children: this.todos.map(todo => {
        return Row(
          children: [
            Checkbox(
              value: todo.completed,
              onChange: () => dispatch(TodoAction.Toggle(todo.id))
            ),
            Text(todo.text),
            Button(
              text: "Delete",
              onPressed: () => dispatch(TodoAction.Remove(todo.id))
            )
          ]
        )
      })
    )
  }
}

@connect(store: TodoStore)
widget AddTodoForm {
  state inputText: String = ""
  
  fn build() -> Widget {
    return Row(
      children: [
        TextField(
          value: this.inputText,
          onChange: (text) => { this.inputText = text }
        ),
        Button(
          text: "Add",
          onPressed: () => {
            if this.inputText.trim().length > 0 {
              dispatch(TodoAction.Add(this.inputText))
              this.inputText = ""
            }
          }
        )
      ]
    )
  }
}

# 5. App principal
@connect(store: TodoStore)
widget TodoApp {
  fn build() -> Widget {
    return Container(
      children: [
        Text("Todo App"),
        AddTodoForm(),
        TodoList(),
        FilterButtons()
      ]
    )
  }
}
```

---

**Última actualización:** 2025-12-02  
**Versión:** 1.0.0  
**Estado:** ✅ Completada
