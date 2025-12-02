# TASK-035Y: Implementar Middleware System

## 📋 Información General
- **Historia:** VELA-577 - State Management
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Prioridad:** P1 (opcional)

## 🎯 Objetivo

Implementar un **sistema de middleware** para interceptar y modificar acciones antes de que lleguen al reducer. Permite side effects, logging, async operations, throttling, error handling, etc.

**Inspirado en:**
- Redux Middleware (redux-thunk, redux-saga, redux-logger)
- Express.js Middleware
- Koa Middleware
- ASP.NET Core Middleware

## 🏗️ Arquitectura

### Flujo de Middleware Chain

```
┌─────────────────────────────────────────────────────┐
│                 MIDDLEWARE CHAIN                    │
│                                                     │
│  dispatch(action)                                   │
│         │                                           │
│         ▼                                           │
│  ┌─────────────┐                                   │
│  │ Middleware 1│ (Logger)                          │
│  └──────┬──────┘                                   │
│         │ next(action)                              │
│         ▼                                           │
│  ┌─────────────┐                                   │
│  │ Middleware 2│ (Async)                           │
│  └──────┬──────┘                                   │
│         │ next(action)                              │
│         ▼                                           │
│  ┌─────────────┐                                   │
│  │ Middleware 3│ (Error Handler)                   │
│  └──────┬──────┘                                   │
│         │ next(action)                              │
│         ▼                                           │
│  ┌─────────────┐                                   │
│  │   Reducer   │                                   │
│  └─────────────┘                                   │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Componentes Principales

#### 1. **Middleware Base Class**

```python
class Middleware:
    def handle(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        # Por defecto, pasar al siguiente
        next(action)
```

#### 2. **MiddlewareContext**

```python
@dataclass
class MiddlewareContext:
    store: Any                     # Referencia al Store
    get_state: Callable[[], Any]   # Función para obtener estado
    dispatch: Callable[[Any], None] # Función para dispatch
```

#### 3. **Middleware Chain**

El sistema soporta **composición de middlewares** en cadena, donde cada middleware puede:
- Inspeccionar la acción
- Modificar la acción
- Detener la propagación (no llamar `next()`)
- Dispatch acciones adicionales
- Ejecutar side effects

## 🔧 Implementación

### Archivos Creados

1. **src/reactive/middleware.py** (~580 LOC)
   - `Middleware` base class
   - `MiddlewareContext` dataclass
   - **6 middlewares prebuilts:**
     - `LoggerMiddleware` - Logging de acciones y estado
     - `AsyncMiddleware` - Soporte para thunks async
     - `ThrottleMiddleware` - Rate limiting
     - `DebounceMiddleware` - Debouncing
     - `ErrorHandlerMiddleware` - Manejo de errores
     - `CacheMiddleware` - Caching de resultados
   - Helper functions: `compose_middleware()`, `apply_middleware()`, `create_middleware()`

2. **tests/unit/state/test_middleware.py** (~410 LOC, 17 tests pasando)
   - `TestMiddlewareBase`: 2 tests
   - `TestLoggerMiddleware`: 2 tests
   - `TestAsyncMiddleware`: 2 tests
   - `TestThrottleMiddleware`: 3 tests
   - `TestDebounceMiddleware`: 1 test
   - `TestErrorHandlerMiddleware`: 2 tests
   - `TestCacheMiddleware`: 2 tests
   - `TestHelperFunctions`: 3 tests

## 📝 APIs Principales

### 1. LoggerMiddleware - Logging

```vela
# Logger básico
store AppStore {
  middlewares: [LoggerMiddleware()]
}

# Logger con opciones
store AppStore {
  middlewares: [
    LoggerMiddleware(
      log_actions: true,    # Registrar acciones
      log_state: true,      # Registrar cambios de estado
      collapsed: false      # Logs expandidos
    )
  ]
}

# Output:
# [12:34:56.789] action INCREMENT
#   prev state: {"count": 0}
#   next state: {"count": 1}
```

### 2. AsyncMiddleware - Operaciones Asíncronas

```vela
# Thunks async (Redux-thunk style)
async fn fetchUsers() -> Thunk {
  return async fn (dispatch, getState) {
    # Dispatch loading
    dispatch(SetLoadingAction(true))
    
    try {
      # Fetch async
      users = await api.fetchUsers()
      
      # Dispatch success
      dispatch(SetUsersAction(users))
      dispatch(SetLoadingAction(false))
    } catch (error) {
      # Dispatch error
      dispatch(SetErrorAction(error))
      dispatch(SetLoadingAction(false))
    }
  }
}

# Configurar store
store AppStore {
  middlewares: [AsyncMiddleware()]
}

# Dispatch async
store.dispatch(fetchUsers())
```

### 3. ThrottleMiddleware - Rate Limiting

```vela
# Throttle: máximo 1 acción por segundo
store AppStore {
  middlewares: [
    ThrottleMiddleware(delay: 1000)  # ms
  ]
}

# Dispatch rápido
store.dispatch(UpdateAction)  # ✅ Pasa
store.dispatch(UpdateAction)  # ❌ Bloqueado (< 1000ms)
# ... esperar 1000ms ...
store.dispatch(UpdateAction)  # ✅ Pasa
```

### 4. DebounceMiddleware - Debouncing

```vela
# Debounce: espera 300ms de inactividad
store AppStore {
  middlewares: [
    DebounceMiddleware(delay: 300)  # ms
  ]
}

# Dispatch múltiple
store.dispatch(SearchAction("a"))    # Timer started
store.dispatch(SearchAction("ab"))   # Timer reset
store.dispatch(SearchAction("abc"))  # Timer reset
# ... esperar 300ms sin dispatches ...
# ✅ Solo ejecuta el último: SearchAction("abc")
```

### 5. ErrorHandlerMiddleware - Manejo de Errores

```vela
# Error handler básico
store AppStore {
  middlewares: [
    ErrorHandlerMiddleware()
  ]
}

# Con callback custom
store AppStore {
  middlewares: [
    ErrorHandlerMiddleware(
      on_error: (error, action) => {
        # Log a service externo
        errorLogger.log(error, action)
        
        # Mostrar toast
        showToast("Error: ${error.message}")
      }
    )
  ]
}

# Si reducer lanza error:
# [Error] Action failed: IncrementAction
#   Error: Division by zero
# → Dispatch automático de ERROR action
```

### 6. CacheMiddleware - Caching

```vela
# Cache con tamaño límite
store AppStore {
  middlewares: [
    CacheMiddleware(max_size: 100)
  ]
}

# Primera ejecución → ejecuta reducer
store.dispatch(FetchUserAction(id: 1))

# Segunda ejecución con mismo payload → cache hit
store.dispatch(FetchUserAction(id: 1))  # No ejecuta reducer

# [Cache HIT] FetchUserAction:{"id":1}
```

## 📚 Ejemplos de Uso

### Ejemplo 1: Stack completo de middlewares

```vela
store AppStore {
  middlewares: [
    # 1. Error handling (primero, para capturar todo)
    ErrorHandlerMiddleware(
      on_error: (error, action) => {
        analytics.trackError(error)
      }
    ),
    
    # 2. Logging (segundo, para ver todas las acciones)
    LoggerMiddleware(
      log_actions: true,
      log_state: true
    ),
    
    # 3. Async support
    AsyncMiddleware(),
    
    # 4. Throttle (rate limiting)
    ThrottleMiddleware(delay: 100),
    
    # 5. Cache (último, cerca del reducer)
    CacheMiddleware(max_size: 50)
  ]
}
```

### Ejemplo 2: Custom middleware

```vela
# Analytics middleware
class AnalyticsMiddleware extends Middleware {
  fn handle(context: MiddlewareContext, next: NextFn, action: Action) -> void {
    # Track acción en analytics
    analytics.track("store_action", {
      type: action.type,
      timestamp: Date.now(),
      user_id: context.get_state().user?.id
    })
    
    # Pasar al siguiente
    next(action)
  }
}

# Usar
store AppStore {
  middlewares: [
    AnalyticsMiddleware(),
    LoggerMiddleware()
  ]
}
```

### Ejemplo 3: Conditional middleware

```vela
# Middleware que solo ejecuta en desarrollo
class DevOnlyMiddleware extends Middleware {
  middleware: Middleware
  
  constructor(middleware: Middleware) {
    this.middleware = middleware
  }
  
  fn handle(context, next, action) -> void {
    if process.env.NODE_ENV == "development" {
      this.middleware.handle(context, next, action)
    } else {
      next(action)
    }
  }
}

# Usar
store AppStore {
  middlewares: [
    DevOnlyMiddleware(LoggerMiddleware()),
    AsyncMiddleware()
  ]
}
```

### Ejemplo 4: Async thunk con error handling

```vela
# Todo service con async thunks
service TodoService {
  fn fetchTodos() -> Thunk {
    return async fn (dispatch, getState) {
      dispatch(SetLoadingAction(true))
      dispatch(ClearErrorAction())
      
      try {
        response = await fetch("/api/todos")
        
        if !response.ok {
          throw Error("Failed to fetch todos")
        }
        
        todos = await response.json()
        dispatch(SetTodosAction(todos))
        dispatch(SetLoadingAction(false))
        
      } catch (error) {
        dispatch(SetErrorAction(error.message))
        dispatch(SetLoadingAction(false))
      }
    }
  }
  
  fn addTodo(text: String) -> Thunk {
    return async fn (dispatch, getState) {
      dispatch(SetLoadingAction(true))
      
      try {
        response = await fetch("/api/todos", {
          method: "POST",
          body: JSON.stringify({ text })
        })
        
        todo = await response.json()
        dispatch(AddTodoAction(todo))
        dispatch(SetLoadingAction(false))
        
      } catch (error) {
        dispatch(SetErrorAction(error.message))
        dispatch(SetLoadingAction(false))
      }
    }
  }
}

# Configurar store con async middleware
@injectable
store TodoStore {
  state todos: List<Todo> = []
  state loading: Bool = false
  state error: Option<String> = None
  
  middlewares: [
    ErrorHandlerMiddleware(),
    LoggerMiddleware(),
    AsyncMiddleware()
  ]
}

# Usar
service = inject(TodoService)
store = inject(TodoStore)

# Dispatch async
store.dispatch(service.fetchTodos())
store.dispatch(service.addTodo("Learn Vela"))
```

### Ejemplo 5: Composición manual de middlewares

```vela
# Crear middleware compuesto
composed = compose_middleware(
  LoggerMiddleware(),
  AsyncMiddleware(),
  ErrorHandlerMiddleware()
)

# Aplicar a store existente
store = Store(initial_state, reducer)
apply_middleware(store, composed)

# O crear middleware custom
custom = create_middleware((context, next, action) => {
  print("Before: ${action.type}")
  next(action)
  print("After: ${action.type}")
})

apply_middleware(store, custom)
```

## 🔄 Integración con Store<T>

El middleware system **se integra nativamente** con Store<T>:

```vela
store AppStore {
  state count: Number = 0
  
  # Configurar middlewares
  middlewares: [
    LoggerMiddleware(),
    AsyncMiddleware(),
    ThrottleMiddleware(delay: 1000)
  ]
  
  reducer(state: AppState, action: Action) -> AppState {
    match action {
      IncrementAction => { count: state.count + 1 }
      _ => state
    }
  }
}

# Al dispatch, la acción pasa por todos los middlewares
store.dispatch(IncrementAction)

# Output (Logger):
# [12:34:56.789] action IncrementAction
#   prev state: {"count": 0}
#   next state: {"count": 1}
```

## 📊 Comparación con Redux Middleware

| Feature | Redux Middleware | Vela Middleware ✅ |
|---------|------------------|-------------------|
| **Chain composition** | ✅ | ✅ |
| **Async support** | redux-thunk | ✅ AsyncMiddleware |
| **Logger** | redux-logger | ✅ LoggerMiddleware |
| **Error handling** | Manual | ✅ ErrorHandlerMiddleware |
| **Throttle/Debounce** | Manual | ✅ Built-in |
| **Caching** | Manual | ✅ CacheMiddleware |
| **Type safety** | TS only | ✅ Nativo |
| **Configuration** | Complex | ✅ Simple (array) |

## ✅ Tests

### Cobertura

- **17 tests pasando** (100%)
- **8 test classes**:
  1. `TestMiddlewareBase`: 2 tests (base behavior, callable)
  2. `TestLoggerMiddleware`: 2 tests (logs actions, logs state)
  3. `TestAsyncMiddleware`: 2 tests (handles functions, passes actions)
  4. `TestThrottleMiddleware`: 3 tests (allows first, blocks rapid, allows after delay)
  5. `TestDebounceMiddleware`: 1 test (delays action)
  6. `TestErrorHandlerMiddleware`: 2 tests (catches exceptions, calls callback)
  7. `TestCacheMiddleware`: 2 tests (stores results, evicts oldest)
  8. `TestHelperFunctions`: 3 tests (compose, apply, create)

### Tests Destacados

#### Middleware Chain
- ✅ Composición de múltiples middlewares
- ✅ Orden de ejecución (before/after pattern)
- ✅ Next function propagation

#### Logger
- ✅ Registra acciones con timestamp
- ✅ Registra cambios de estado (prev/next)
- ✅ Serialización de estado

#### Async
- ✅ Ejecuta funciones thunk con dispatch/getState
- ✅ Pasa acciones normales sin modificar

#### Throttle
- ✅ Permite primera acción
- ✅ Bloquea acciones rápidas
- ✅ Permite después del delay

#### Error Handler
- ✅ Captura excepciones sin crashear
- ✅ Llama callback custom
- ✅ Dispatch ERROR action

## 🔗 Referencias

- **Jira:** [VELA-577](https://velalang.atlassian.net/browse/VELA-577)
- **Sprint:** Sprint 15
- **ADR:** [ADR-008](../../architecture/ADR-008-state-management-architecture.md)

## 📦 Entregables

- ✅ `src/reactive/middleware.py` (~580 LOC)
- ✅ `tests/unit/state/test_middleware.py` (~410 LOC)
- ✅ 17 tests pasando (100%)
- ✅ 6 middlewares prebuilts
- ✅ 3 helper functions
- ✅ Documentación completa

## 🎯 Criterios de Aceptación

- [x] `Middleware` base class
- [x] `MiddlewareContext` dataclass
- [x] `LoggerMiddleware` con configuración
- [x] `AsyncMiddleware` para thunks
- [x] `ThrottleMiddleware` con delay configurable
- [x] `DebounceMiddleware` con timer cancelable
- [x] `ErrorHandlerMiddleware` con callback custom
- [x] `CacheMiddleware` con eviction policy
- [x] `compose_middleware()` helper
- [x] `apply_middleware()` helper
- [x] `create_middleware()` helper
- [x] 17 tests pasando (100%)
- [x] Documentación completa

---

**Última actualización:** 2025-12-02  
**Versión:** 1.0.0  
**Estado:** ✅ Completada
