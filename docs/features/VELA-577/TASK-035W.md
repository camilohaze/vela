# TASK-035W: Implementar @select decorator

## 📋 Información General
- **Historia:** VELA-577 (State Management con Store<T>, Actions, Reducers)
- **Epic:** EPIC-03D (State Management)
- **Sprint:** Sprint 15
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02

## 🎯 Objetivo

Implementar el decorador `@select` para crear selectors memoizados que extraen datos derivados del Store con reactividad automática mediante integración con `Computed<T>`.

## 🔨 Implementación

### Archivos generados

1. **`src/reactive/select.py`** (~470 LOC):
   - `SelectOptions`: Dataclass con configuración del decorator
   - `@select`: Decorator principal para métodos computed
   - `create_selector()`: Selector compuesto (reselect-style)
   - `create_structured_selector()`: Selector que retorna dict
   - `create_parametric_selector()`: Selector con parámetros
   - `SelectorComposer`: Fluent API para componer selectors
   - `is_selector()`, `get_selector_name()`, `get_selector_options()`: Metadata helpers

2. **`tests/unit/state/test_select.py`** (~440 LOC):
   - 22 tests cubriendo funcionalidad completa
   - Fixtures: `Todo`, `TodoState`, `MockTodoStore`
   - Test classes:
     - `TestSelectDecorator`: 7 tests (decorator behavior)
     - `TestCreateSelector`: 3 tests (selector composition)
     - `TestCreateStructuredSelector`: 2 tests (structured selectors)
     - `TestCreateParametricSelector`: 3 tests (parametric selectors)
     - `TestSelectorComposer`: 4 tests (fluent API)
     - `TestMetadataHelpers`: 3 tests (metadata)

### Arquitectura del Decorator

**Pattern:** Memoized computed properties con reactividad

```
Store State
    ↓
  @select
    ↓
Computed<T> (cached)
    ↓
Property getter
    ↓
Re-compute solo si dependencias cambian
```

### Integración con Computed<T>

**@select usa `Computed` internamente:**

```python
@select()
def completed_todos(self) -> List[Todo]:
    return [t for t in self.todos if t.completed]

# Internamente crea:
# Computed(lambda: self.todos.filter(...))
```

**Beneficios:**
- ✅ Reactividad automática (dependency tracking)
- ✅ Lazy evaluation (solo computa al acceder)
- ✅ Caching (resultado cacheado hasta que dependencias cambien)

### SelectOptions

**Configuración del decorator:**

```python
@dataclass
class SelectOptions:
    memoize: bool = True           # Habilitar memoization
    max_size: int = 1000           # Tamaño máximo del cache
    ttl: Optional[float] = None    # Time to live en segundos
    equals: Optional[Callable] = None  # Función de comparación
    name: Optional[str] = None     # Nombre del selector
```

### APIs Principales

#### 1. **@select Decorator**

```vela
store AppStore {
  state todos: List<Todo> = []
  
  # Selector simple
  @select
  computed completed_todos: List<Todo> {
    return self.todos.filter(t => t.completed)
  }
  
  # Selector con opciones
  @select(SelectOptions(max_size=100, ttl=60.0))
  computed expensive_data: Any {
    return heavyComputation(self.raw_data)
  }
}
```

#### 2. **create_selector()** - Composición de selectors

```vela
# Input selectors
todos_selector = (state) => state.todos
filter_selector = (state) => state.filter

# Selector compuesto
filtered_todos = create_selector(
  todos_selector,
  filter_selector,
  combiner: (todos, filter) => {
    return todos.filter(t =>
      filter == "all" ||
      (filter == "active" && !t.completed) ||
      (filter == "completed" && t.completed)
    )
  }
)

# Uso
result = filtered_todos(app_state)
```

#### 3. **create_structured_selector()** - Props objects

```vela
# Selector estructurado (para @connect)
props_selector = create_structured_selector({
  "todos": (state) => state.todos,
  "filter": (state) => state.filter,
  "completed_count": (state) => state.todos.filter(t => t.completed).length
})

# Uso con @connect
@connect(ConnectOptions(
  store: app_store,
  selector: props_selector
))
widget TodoList { }
```

#### 4. **create_parametric_selector()** - Selectors con parámetros

```vela
# Selector parametrizado
todo_by_id = create_parametric_selector(
  (state, todo_id) => state.todos.find(t => t.id == todo_id)
)

# Crear selector para ID específico
selector = todo_by_id(42)

# Usar
todo = selector(app_state)
```

#### 5. **SelectorComposer** - Fluent API

```vela
composer = SelectorComposer((state) => state.todos)

# Encadenar transformaciones
completed_texts = (composer
  .filter(t => t.completed)
  .map(todos => todos.map(t => t.text))
  .build()
)

result = completed_texts(app_state)
```

## 📊 Ejemplos de Uso

### Ejemplo 1: TodoList Store con Selectors

```vela
import 'system:reactive' show { Store, select, SelectOptions }

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

# Store con selectors
store TodoStore {
  state todos: List<Todo> = []
  state filter: String = "all"
  
  # Selector 1: Todos completados
  @select
  computed completed_todos: List<Todo> {
    return this.todos.filter(t => t.completed)
  }
  
  # Selector 2: Todos activos
  @select
  computed active_todos: List<Todo> {
    return this.todos.filter(t => !t.completed)
  }
  
  # Selector 3: Contador de completados (usa completed_todos)
  @select
  computed completed_count: Number {
    return this.completed_todos.length
  }
  
  # Selector 4: Contador de activos (usa active_todos)
  @select
  computed active_count: Number {
    return this.active_todos.length
  }
  
  # Selector 5: Todos filtrados (según this.filter)
  @select
  computed filtered_todos: List<Todo> {
    match this.filter {
      "active" => this.active_todos
      "completed" => this.completed_todos
      _ => this.todos
    }
  }
  
  # Selector 6: Stats completos
  @select
  computed stats: Map<String, Number> {
    return {
      "total": this.todos.length,
      "completed": this.completed_count,
      "active": this.active_count
    }
  }
}

# Uso
store = TodoStore(initial_state: TodoState {
  todos: [
    Todo { id: 1, text: "Buy milk", completed: false },
    Todo { id: 2, text: "Write code", completed: true },
  ],
  filter: "all"
})

# Acceder a selectors
print(store.completed_count)  # 1
print(store.active_count)     # 1
print(store.filtered_todos)   # [Todo(1), Todo(2)]
```

### Ejemplo 2: Selector Compuesto con create_selector

```vela
import 'system:reactive' show { create_selector }

# Input selectors
todos_selector = (state: TodoState) => state.todos
search_selector = (state: TodoState) => state.search_text

# Selector compuesto: buscar en todos
search_results = create_selector(
  todos_selector,
  search_selector,
  combiner: (todos, search) => {
    if !search or search.isEmpty() {
      return todos
    }
    
    return todos.filter(t => 
      t.text.toLowerCase().contains(search.toLowerCase())
    )
  }
)

# Uso
results = search_results(app_state)
```

### Ejemplo 3: Selector Estructurado para @connect

```vela
import 'system:reactive' show { 
  connect, 
  ConnectOptions, 
  create_structured_selector 
}

# Selector estructurado
todo_props = create_structured_selector({
  "todos": (state) => state.todos,
  "filter": (state) => state.filter,
  "stats": (state) => {
    completed = state.todos.filter(t => t.completed).length
    active = state.todos.filter(t => !t.completed).length
    
    return {
      "total": state.todos.length,
      "completed": completed,
      "active": active
    }
  }
})

# Widget conectado con selector estructurado
@connect(ConnectOptions(
  store: todo_store,
  selector: todo_props
))
component TodoListWidget extends StatelessWidget {
  # Props inyectadas automáticamente:
  # - this.todos: List<Todo>
  # - this.filter: String
  # - this.stats: Map<String, Number>
  
  fn build() -> Widget {
    return Column {
      children: [
        Text("Total: ${this.stats['total']}"),
        Text("Completed: ${this.stats['completed']}"),
        Text("Active: ${this.stats['active']}"),
        
        ...this.todos.map(todo => TodoItem { todo: todo })
      ]
    }
  }
}
```

### Ejemplo 4: Selector Parametrizado

```vela
import 'system:reactive' show { create_parametric_selector }

# Selector parametrizado: obtener usuario por ID
user_by_id = create_parametric_selector(
  (state: AppState, user_id: Number) => {
    return state.users.find(u => u.id == user_id)
  }
)

# Crear selector para usuario específico
get_user_42 = user_by_id(42)

# Usar en widget
@connect(ConnectOptions(
  store: app_store,
  selector: (state) => {
    user = get_user_42(state)
    return { "user": user }
  }
))
component UserProfile extends StatelessWidget {
  # this.user: Option<User>
  
  fn build() -> Widget {
    match this.user {
      Some(user) => Text("Name: ${user.name}")
      None => Text("User not found")
    }
  }
}
```

### Ejemplo 5: SelectorComposer con Fluent API

```vela
import 'system:reactive' show { SelectorComposer }

# Composer con múltiples transformaciones
composer = SelectorComposer((state) => state.todos)

# Fluent API
high_priority_texts = (composer
  .filter(todo => todo.priority == "high")
  .filter(todo => !todo.completed)
  .map(todos => todos.map(t => t.text))
  .build()
)

# Uso
texts = high_priority_texts(app_state)
```

## ✅ Criterios de Aceptación

- [x] **Código implementado** en `src/reactive/select.py`:
  - [x] `SelectOptions` dataclass
  - [x] `@select` decorator con integración a Computed
  - [x] `create_selector()` para composición
  - [x] `create_structured_selector()` para props objects
  - [x] `create_parametric_selector()` para selectors con parámetros
  - [x] `SelectorComposer` fluent API
  - [x] Metadata helpers (is_selector, get_selector_name, get_selector_options)

- [x] **Tests escritos y pasando** (22 tests):
  - [x] `TestSelectDecorator`: 7 tests (decorator behavior)
  - [x] `TestCreateSelector`: 3 tests (composition)
  - [x] `TestCreateStructuredSelector`: 2 tests (structured)
  - [x] `TestCreateParametricSelector`: 3 tests (parametric)
  - [x] `TestSelectorComposer`: 4 tests (fluent API)
  - [x] `TestMetadataHelpers`: 3 tests (metadata)

- [x] **Documentación generada** (`TASK-035W.md`):
  - [x] Arquitectura e integración con Computed
  - [x] SelectOptions configuración
  - [x] APIs principales
  - [x] 5 ejemplos completos de uso

- [x] **Integración con Computed<T>** (Sprint 11-12):
  - [x] `@select` crea instancias de Computed
  - [x] Dependency tracking automático
  - [x] Lazy evaluation y caching

## 📊 Métricas

- **Archivos creados**: 2
  - `src/reactive/select.py`: ~470 LOC
  - `tests/unit/state/test_select.py`: ~440 LOC
- **Tests**: 22 (100% pasando)
- **APIs**: 7 funciones principales
  - `@select` decorator
  - `create_selector()`
  - `create_structured_selector()`
  - `create_parametric_selector()`
  - `SelectorComposer`
  - 3 metadata helpers
- **Cobertura**: 100% de funcionalidad del decorator

## 🔗 Referencias

- **Jira**: [TASK-035W](https://velalang.atlassian.net/browse/VELA-577)
- **Historia**: [VELA-577](https://velalang.atlassian.net/browse/VELA-577)
- **Epic**: [EPIC-03D](https://velalang.atlassian.net/browse/EPIC-03D)
- **ADR**: [ADR-008 - State Management Architecture](../../../docs/architecture/ADR-008-state-management-architecture.md)
- **Tasks previas**:
  - TASK-035R: Decisión arquitectónica (ADR-008)
  - TASK-035S: Implementación de Store<T>
  - TASK-035T: Documentación de Action/Reducer
  - TASK-035U: dispatch keyword
  - TASK-035V: @connect decorator

## 🔄 Integración con Tareas Previas

**Computed<T> (Sprint 11-12)**:
- `@select` usa `Computed` internamente para reactividad
- Computed maneja dependency tracking automático
- Lazy evaluation y caching integrados

**Store<T> (TASK-035S)**:
- `@select` se usa dentro de Store classes
- Selectors acceden a state fields reactivos

**@connect (TASK-035V)**:
- `@connect` usa selectors para extraer props
- `create_structured_selector()` crea props objects
- Integración fluida entre @select y @connect

## 🚀 Próximos Pasos

Después de TASK-035W, continuar con:

1. **TASK-035X**: Implementar `@persistent` decorator (opcional P1)
   - Persistencia en localStorage/file
   - Auto-save/restore

2. **TASK-035Y**: Implementar middleware system (opcional P1)
   - Logger middleware
   - Async middleware

3. **TASK-035Z**: Implementar DevTools integration (opcional P2)
   - Browser extension
   - Time-travel debugging

4. **TASK-035AA**: Tests de integración de State Management (obligatorio P0)
   - TodoApp E2E test
   - Performance tests
   - Integration tests

## 📝 Notas Técnicas

### Comparación con reselect (Redux)

| Aspecto | reselect | Vela @select |
|---------|----------|--------------|
| **API** | `createSelector()` function | `@select` decorator + `create_selector()` |
| **Memoization** | Manual (input selector IDs) | Automático (Computed + id() comparison) |
| **Composición** | `createSelector()` anidado | `create_selector()` + SelectorComposer |
| **Type Safety** | TypeScript optional | Nativo con generics |
| **Reactividad** | No (Redux es pull-based) | Sí (Computed es push-based) |

### Optimizaciones

1. **ID-based caching**: Usa `id()` en lugar de comparación de valores (más rápido)
2. **Instance-level cache**: Cada instancia tiene su propio cache de Computed
3. **Lazy evaluation**: Selectors solo se ejecutan cuando se acceden
4. **Composed selectors**: Memoizan inputs intermedios

### Edge Cases Manejados

1. **Selectors sin dependencias**: Funcionan igual (siempre retornan mismo valor)
2. **Selectors que retornan objetos mutables**: Cache usa `id()` en lugar de `==`
3. **Multiple accesos al mismo selector**: Reutiliza mismo Computed
4. **State mutation vs immutable update**: Computed detecta cambios por identity

### Diferencias con @ngrx/store selectors

| Aspecto | @ngrx/store | Vela @select |
|---------|-------------|--------------|
| **Definición** | `createSelector()` functions | `@select` decorator en Store |
| **Ubicación** | Archivos separados | Dentro del Store class |
| **Memoization** | Automático (input functions) | Automático (Computed) |
| **Composición** | Functional (createSelector) | OOP (decorator) + Functional (helpers) |

---

**Estado Final**: ✅ **Completada**  
**Tests**: 22/22 pasando (100%)  
**Commit**: Pendiente (siguiente paso)
