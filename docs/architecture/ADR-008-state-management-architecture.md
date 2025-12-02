# ADR-008: Arquitectura de State Management

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

Vela necesita un **sistema de gestión de estado global** para aplicaciones complejas con múltiples componentes que comparten estado. Actualmente:

- ✅ **Signal System** (Sprint 11-12): Reactividad local dentro de componentes
- ✅ **DI System** (Sprint 13): Inyección de dependencias para servicios
- ✅ **Event System** (Sprint 14): Comunicación desacoplada entre componentes

Sin embargo, **NO existe un patrón estándar para estado compartido** entre múltiples widgets o componentes que necesitan sincronizarse.

### Problemas a resolver:

1. **Prop drilling**: Pasar estado por múltiples niveles de componentes
2. **Sincronización**: Múltiples componentes leyendo/escribiendo el mismo estado
3. **Debugging**: Rastrear cambios de estado y entender flujo de datos
4. **Persistencia**: Guardar/restaurar estado de la aplicación
5. **Time-travel**: Deshacer/rehacer cambios de estado
6. **Testing**: Testear lógica de estado separada de la UI

### Requisitos del sistema:

- ✅ **Type-safe**: Inferencia de tipos para acciones y estado
- ✅ **Reactivo**: Integración con Signal System para updates automáticos
- ✅ **Predecible**: Flujo unidireccional de datos
- ✅ **Debuggable**: Logs de acciones y cambios de estado
- ✅ **Testable**: Reductores puros fáciles de testear
- ✅ **Extensible**: Sistema de middleware para cross-cutting concerns
- ✅ **Performance**: Updates eficientes con memoization

---

## Decisión

### ✅ **Adoptar patrón Redux/NgRx con mejoras de Vela**

**Arquitectura elegida:**

```
┌─────────────────────────────────────────────────────┐
│                   VELA STORE                        │
│                                                     │
│  ┌─────────────┐     ┌──────────────┐            │
│  │   State<T>  │────▶│  Reducers    │            │
│  └─────────────┘     └──────────────┘            │
│         ▲                    ▲                      │
│         │                    │                      │
│  ┌──────┴──────┐      ┌─────┴─────┐              │
│  │  Selectors  │      │  Actions   │              │
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
└────────────────────────────────────────────────────┘
```

### Componentes principales:

#### 1. **Store<T>** - Contenedor de estado global

```vela
store AppStore {
  # Estado tipado y reactivo
  state count: Number = 0
  state user: Option<User> = None
  state todos: List<Todo> = []
  
  # Reducer: state + action → new state
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
- ✅ Integración con Signal System (state fields son Signals)
- ✅ Type-safe: `Store<AppState>` garantiza tipos correctos
- ✅ Single source of truth

#### 2. **Action** - Intención de cambiar estado

```vela
# Acciones como enums con datos asociados
enum TodoAction {
  Add(text: String)
  Toggle(id: Number)
  Remove(id: Number)
  Clear
}

# O como types simples
type IncrementAction = { type: "INCREMENT" }
type DecrementAction = { type: "DECREMENT", amount: Number }
type CounterAction = IncrementAction | DecrementAction
```

**Características:**
- ✅ Discriminated unions para type safety
- ✅ Payload tipado
- ✅ Inmutables

#### 3. **Reducer** - Función pura: (State, Action) → State

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
- ✅ Testables sin mock
- ✅ Pattern matching exhaustivo

#### 4. **dispatch keyword** - Enviar acciones al store

```vela
# En cualquier componente conectado
dispatch(IncrementAction)
dispatch(AddTodoAction(text: "Learn Vela"))
```

**Características:**
- ✅ Type-checked: acción debe ser del tipo correcto
- ✅ Keyword nativo del lenguaje (como `await`, `yield`)
- ✅ Pasa por middleware chain antes de llegar al reducer

#### 5. **@connect decorator** - Conectar widgets al store

```vela
@connect(store: AppStore, selector: (state) => state.count)
widget Counter {
  # Props inyectadas automáticamente
  count: Number  # Del selector
  
  fn build() -> Widget {
    return Column(
      children: [
        Text("Count: ${this.count}"),
        Button(
          text: "+",
          onPressed: () => dispatch(IncrementAction)
        )
      ]
    )
  }
}
```

**Características:**
- ✅ Auto-subscribe al store
- ✅ Re-render solo cuando selector cambia (shallow equality)
- ✅ Auto-cleanup cuando widget se destruye

#### 6. **@select decorator** - Memoización de selectores

```vela
store AppStore {
  state todos: List<Todo> = []
  
  # Computed memoizado (solo recalcula si todos cambia)
  @select
  computed completedTodos: List<Todo> {
    return this.todos.filter(t => t.completed)
  }
  
  @select
  computed completedCount: Number {
    return this.completedTodos.length
  }
}
```

**Características:**
- ✅ Memoización automática (caching)
- ✅ Recompute solo si dependencias cambian
- ✅ Evita renders innecesarios

#### 7. **Middleware System** - Interceptores de acciones

```vela
# Logger middleware (inspección)
fn loggerMiddleware(store: Store, next: Dispatch, action: Action) -> void {
  print("Dispatching: ${action}")
  prevState = store.getState()
  
  next(action)  # Pasar al siguiente middleware o reducer
  
  nextState = store.getState()
  print("New state: ${nextState}")
}

# Async middleware (side effects)
fn asyncMiddleware(store: Store, next: Dispatch, action: Action) -> void {
  match action {
    FetchTodosAction => {
      # Dispatch loading
      next(SetLoadingAction(true))
      
      # Fetch async
      todos = await fetchTodos()
      
      # Dispatch success
      next(SetTodosAction(todos))
      next(SetLoadingAction(false))
    }
    _ => next(action)
  }
}

# Configurar store con middleware
store AppStore {
  middlewares: [loggerMiddleware, asyncMiddleware]
}
```

**Características:**
- ✅ Chain of responsibility pattern
- ✅ Composición de middlewares
- ✅ Side effects (HTTP, timers, logging)
- ✅ Acceso a `store.getState()` y `store.dispatch()`

#### 8. **@persistent decorator** - Persistencia automática

```vela
@persistent(key: "app-state", storage: localStorage)
store AppStore {
  state user: Option<User> = None
  state settings: Settings = defaultSettings
}
```

**Características:**
- ✅ Guardado automático en localStorage/file
- ✅ Restauración al inicializar
- ✅ Serialización JSON automática
- ✅ Opción de persistir solo parte del estado

#### 9. **DevTools Integration** - Time-travel debugging

```vela
store AppStore {
  devTools: true  # Habilita Redux DevTools
}
```

**Características:**
- ✅ Historial de acciones
- ✅ Time-travel (undo/redo)
- ✅ State snapshots
- ✅ Action replay
- ✅ Integración con browser DevTools

---

## Comparación con alternativas

| Feature | **Redux** | **MobX** | **Recoil** | **Vela Store** ✅ |
|---------|-----------|----------|------------|-------------------|
| **Patrón** | Flux | Observable | Atom-based | Redux + Signals |
| **Mutabilidad** | Inmutable | Mutable | Inmutable | Inmutable |
| **Boilerplate** | Alto | Bajo | Medio | **Bajo** (keywords) |
| **Type Safety** | TS only | Débil | TS only | **Nativo** |
| **Middleware** | ✅ | ❌ | ❌ | ✅ |
| **DevTools** | ✅ | ✅ | ❌ | ✅ |
| **Reactividad** | External | Built-in | External | **Built-in (Signals)** |
| **Selectors** | Reselect | Computed | Selectors | **@select (built-in)** |
| **Async** | Redux-thunk | Built-in | Built-in | **Middleware** |

---

## Consecuencias

### ✅ Positivas

1. **Predecibilidad**:
   - Flujo unidireccional: Action → Reducer → State → UI
   - Estado inmutable: fácil rastrear cambios
   - Reductores puros: fáciles de testear

2. **Type Safety completa**:
   - Actions, State, Reducers 100% tipados
   - Pattern matching exhaustivo

3. **Integración profunda con Vela**:
   - `dispatch` como keyword nativo
   - `@connect`, `@select`, `@persistent` como decoradores first-class
   - Integración con Signal System (reactividad automática)

4. **Developer Experience**:
   - DevTools con time-travel
   - Logging de acciones
   - Hot reload de reductores

5. **Performance**:
   - Memoización con `@select`
   - Updates eficientes con Signals
   - Re-renders granulares

6. **Testing**:
   - Reductores son funciones puras → fáciles de testear
   - Sin necesidad de mock del store
   - Testear acciones/reductores por separado

### ⚠️ Negativas

1. **Boilerplate**:
   - Requiere definir Actions, Reducers, Store
   - Más código que MobX (aunque menos que Redux tradicional)

2. **Curva de aprendizaje**:
   - Conceptos nuevos: reducer, immutability, middleware
   - Debugging requiere entender flujo de datos

3. **Overkill para apps simples**:
   - Para 2-3 componentes, Signal System es suficiente
   - Store global agrega complejidad innecesaria

4. **Async handling**:
   - Requiere middleware o acciones separadas (FETCH_START, FETCH_SUCCESS, FETCH_ERROR)
   - No tan directo como async/await en componentes

---

## Alternativas Consideradas

### 1. **MobX-style (Observables mutables)**

```vela
store AppStore {
  @observable
  state count: Number = 0  # Mutable
  
  fn increment() -> void {
    this.count += 1  # Mutación directa
  }
}
```

**Rechazado porque:**
- ❌ Mutabilidad dificulta time-travel debugging
- ❌ Cambios de estado no rastreables
- ❌ Dificulta testing (side effects ocultos)

### 2. **Recoil-style (Atoms distribuidos)**

```vela
atom countAtom: Number = 0

widget Counter {
  count = useAtom(countAtom)
  
  fn increment() -> void {
    count.set(count.value + 1)
  }
}
```

**Rechazado porque:**
- ❌ Estado fragmentado (no single source of truth)
- ❌ Dificulta debuggear flujo de datos global
- ❌ No hay middleware/logging centralizado

### 3. **Context API-style (Vela context)**

```vela
context AppContext {
  state count: Number = 0
  
  fn increment() -> void {
    this.count += 1
  }
}
```

**Rechazado porque:**
- ❌ No hay pattern de acciones/reductores
- ❌ Testing más difícil
- ❌ No DevTools integration

---

## Referencias

- **Jira**: [VELA-577](https://velalang.atlassian.net/browse/VELA-577)
- **Sprint**: Sprint 15
- **Inspiración**:
  - Redux: https://redux.js.org/
  - NgRx: https://ngrx.io/
  - Vuex: https://vuex.vuejs.org/
  - Zustand: https://github.com/pmndrs/zustand

---

## Implementación

### Archivos a crear:

```
src/reactive/
├── store.py          # Store<T> base class
├── action.py         # Action types
├── reducer.py        # Reducer types
└── middleware.py     # Middleware system

src/stdlib/
└── store.vela        # Store APIs en Vela

src/lexer/
└── token.py          # + DISPATCH keyword

src/parser/
└── parser.py         # + dispatch statement

tests/unit/state/
├── test_store.py
├── test_reducers.py
├── test_middleware.py
└── test_selectors.py

tests/integration/
└── test_state_management.py

docs/features/VELA-577/
├── README.md
├── TASK-035R.md
├── TASK-035S.md
└── ...
```

---

## Plan de Implementación (11 tareas)

### Sprint 15 - State Management

| Task | Descripción | Prioridad |
|------|-------------|-----------|
| **TASK-035R** | Diseñar arquitectura (este ADR) | P0 ✅ |
| **TASK-035S** | Implementar Store<T> base | P0 |
| **TASK-035T** | Implementar Action y Reducer types | P0 |
| **TASK-035U** | Implementar dispatch keyword | P0 |
| **TASK-035V** | Implementar @connect decorator | P0 |
| **TASK-035W** | Implementar @select decorator | P0 |
| **TASK-035X** | Implementar @persistent decorator | **P1** |
| **TASK-035Y** | Implementar middleware system | **P1** |
| **TASK-035Z** | Implementar DevTools integration | **P2** |
| **TASK-035AA** | Tests completos | P0 |

**Orden de implementación:**
1. TASK-035R (ADR) ✅
2. TASK-035S (Store<T>)
3. TASK-035T (Action/Reducer)
4. TASK-035U (dispatch keyword)
5. TASK-035V (@connect)
6. TASK-035W (@select)
7. TASK-035X (@persistent) - **Opcional P1**
8. TASK-035Y (middleware) - **Opcional P1**
9. TASK-035Z (DevTools) - **Opcional P2**
10. TASK-035AA (Tests finales)

---

## Criterios de Aceptación

- [x] ✅ ADR completado con decisiones arquitectónicas
- [ ] Store<T> implementado con state reactivo
- [ ] Actions y Reducers type-safe
- [ ] dispatch keyword funcional
- [ ] @connect decorator conecta widgets al store
- [ ] @select decorator memoiza selectores
- [ ] @persistent decorator (opcional P1)
- [ ] Middleware system (opcional P1)
- [ ] DevTools integration (opcional P2)
- [ ] Tests completos (>= 80% coverage)
- [ ] Documentación completa
- [ ] Ejemplo funcional (TodoApp con Store)

---

## Notas Adicionales

### Integración con Signal System

El Store<T> **reutiliza el Signal System** (Sprint 11-12):

```vela
store AppStore {
  # state fields son Signals automáticamente
  state count: Number = 0  # Signal<Number>
  
  # computed fields usan computed()
  @select
  computed doubled: Number {
    return this.count * 2  # Recompute cuando count cambia
  }
}
```

**Beneficios:**
- ✅ No reinventar reactividad
- ✅ Re-use de Signal dependency tracking
- ✅ Updates eficientes con batch()

### Integración con DI System

Stores se pueden inyectar:

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

---

**Última actualización:** 2025-12-02  
**Versión:** 1.0.0  
**Estado:** ✅ Aceptado
