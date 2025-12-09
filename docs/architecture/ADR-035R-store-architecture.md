# ADR-035R: Arquitectura de Store Redux-style

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Como parte de EPIC-03D (State Management), necesitamos una arquitectura robusta para manejar estado global en aplicaciones Vela complejas. El estado global debe ser predecible, testable, y fácil de debuggear, especialmente en aplicaciones con múltiples componentes que necesitan compartir estado.

Los principales desafíos que enfrentamos son:
- Estado compartido entre múltiples widgets/componentes
- Actualizaciones de estado predecibles y rastreables
- Debugging de cambios de estado complejos
- Testing de lógica de estado
- Persistencia y rehidratación de estado
- Time-travel debugging para desarrollo

## Decisión
Implementaremos un patrón Store Redux-style con las siguientes características principales:

### 1. Store<T> - Contenedor del Estado Global
```rust
pub struct Store<T> {
    state: Arc<RwLock<T>>,
    reducer: Box<dyn Fn(&T, &Action) -> T + Send + Sync>,
    subscribers: Vec<Box<dyn Fn(&T) + Send + Sync>>,
    middleware: Vec<Box<dyn Middleware<T> + Send + Sync>>,
}
```

### 2. Actions - Eventos Inmutables
```rust
pub trait Action: std::fmt::Debug + Send + Sync {
    fn action_type(&self) -> &'static str;
}

pub struct CounterAction {
    pub kind: CounterActionKind,
}

pub enum CounterActionKind {
    Increment,
    Decrement,
    Set(i32),
}
```

### 3. Reducers - Funciones Puras
```rust
pub type Reducer<T> = Box<dyn Fn(&T, &dyn Action) -> T + Send + Sync>;

fn counter_reducer(state: &CounterState, action: &dyn Action) -> CounterState {
    match action.downcast_ref::<CounterAction>() {
        Some(counter_action) => match counter_action.kind {
            CounterActionKind::Increment => CounterState { count: state.count + 1 },
            CounterActionKind::Decrement => CounterState { count: state.count - 1 },
            CounterActionKind::Set(value) => CounterState { count: value },
        },
        None => state.clone(),
    }
}
```

### 4. Dispatch - Mecanismo de Envío
```rust
impl<T> Store<T> {
    pub fn dispatch(&self, action: Box<dyn Action>) {
        // Aplicar middleware
        for middleware in &self.middleware {
            middleware.before_dispatch(&action);
        }

        // Aplicar reducer
        let new_state = (self.reducer)(&*self.state.read().unwrap(), &*action);

        // Actualizar estado
        *self.state.write().unwrap() = new_state.clone();

        // Notificar subscribers
        for subscriber in &self.subscribers {
            subscriber(&new_state);
        }

        // Aplicar middleware post-dispatch
        for middleware in &self.middleware {
            middleware.after_dispatch(&action, &new_state);
        }
    }
}
```

### 5. Selectors - Acceso Tipado al Estado
```rust
pub trait Selector<T, R> {
    fn select(&self, state: &T) -> R;
}

struct CounterValueSelector;
impl Selector<CounterState, i32> for CounterValueSelector {
    fn select(&self, state: &CounterState) -> i32 {
        state.count
    }
}
```

### 6. Middleware System
```rust
pub trait Middleware<T> {
    fn before_dispatch(&self, action: &dyn Action);
    fn after_dispatch(&self, action: &dyn Action, new_state: &T);
}

// Middleware incluido por defecto
pub struct LoggingMiddleware;
pub struct DevToolsMiddleware;
pub struct PersistenceMiddleware;
```

### 7. Integration con UI Framework
```rust
// Decorators para conectar widgets al store
#[connect(store = "app_store", selector = "counter_value")]
struct CounterWidget {
    value: i32,
}

// Hooks para acceso programático
let store = use_store::<AppState>();
let counter = use_selector::<AppState, i32>(CounterValueSelector);
```

## Consecuencias

### Positivas
- ✅ **Predecibilidad**: Estado solo cambia a través de actions
- ✅ **Testabilidad**: Reducers son funciones puras fácilmente testeables
- ✅ **Debugging**: Time-travel debugging con DevTools
- ✅ **Performance**: Selectors memoizados evitan re-renders innecesarios
- ✅ **Escalabilidad**: Patrón probado para apps complejas
- ✅ **Developer Experience**: Hot reloading, logging automático
- ✅ **Type Safety**: Sistema de tipos fuerte previene errores

### Negativas
- ⚠️ **Complejidad inicial**: Curva de aprendizaje para developers nuevos
- ⚠️ **Boilerplate**: Más código para operaciones simples
- ⚠️ **Performance overhead**: Middleware y subscribers agregan overhead
- ⚠️ **Memory usage**: Historial para time-travel consume memoria

## Alternativas Consideradas

### 1. Context + Signals (Rechazada)
```rust
// Alternativa: usar signals directamente
let global_state = create_signal(AppState::default());
```
**Rechazada porque:**
- Estado mutable global difícil de rastrear
- No hay historial de cambios
- Difícil de testear
- No hay middleware o debugging avanzado

### 2. Actor Model (Rechazada)
```rust
// Alternativa: actores para state management
struct StateActor {
    state: AppState,
}
```
**Rechazada porque:**
- Complejidad de concurrencia
- No hay patrón estándar para UI
- Difícil integración con widgets

### 3. MVVM Pattern (Rechazada)
```rust
// Alternativa: ViewModels tradicionales
struct AppViewModel {
    state: AppState,
    update_count: fn(i32),
}
```
**Rechazada porque:**
- ViewModels acoplados a vistas específicas
- Difícil de compartir estado entre vistas
- No hay patrón estándar para composición

### 4. Elm Architecture (Considerada pero rechazada)
```rust
// Similar pero sin commands/tasks complejos
type Model = AppState;
type Msg = Action;
fn update(model: &Model, msg: &Msg) -> Model { ... }
```
**Rechazada porque:**
- Elm es funcional puro, Vela permite efectos
- Middleware system más flexible que commands
- Mejor integración con ecosystem existente

## Implementación

### Fases de Implementación
1. **TASK-035R**: Diseño de arquitectura (este ADR)
2. **TASK-035S**: Store<T> base class
3. **TASK-035T**: Action y Reducer types
4. **TASK-035U**: dispatch keyword
5. **TASK-035V**: @connect decorator
6. **TASK-035W**: @select decorator
7. **TASK-035X**: @persistent decorator
8. **TASK-035Y**: Middleware system
9. **TASK-035Z**: DevTools integration
10. **TASK-035AA**: Tests completos

### Archivos a Crear
```
runtime/reactive/src/store.rs          # Store<T> implementation
runtime/reactive/src/action.rs          # Action trait y helpers
runtime/reactive/src/reducer.rs         # Reducer types
runtime/reactive/src/middleware.rs      # Middleware system
runtime/reactive/src/selector.rs        # Selector pattern
runtime/ui/src/store_integration.rs     # UI integration (@connect, etc.)
```

### Integration Points
- **Reactive Engine**: Signals para notificaciones automáticas
- **UI Framework**: Widgets conectados al store
- **DevTools**: Time-travel debugging
- **Persistence**: Estado automático en localStorage/indexedDB

## Referencias
- Jira: [VELA-035R](https://velalang.atlassian.net/browse/VELA-035R)
- Epic: [EPIC-03D](https://velalang.atlassian.net/browse/EPIC-03D)
- Redux Documentation: https://redux.js.org/
- Elm Architecture: https://guide.elm-lang.org/architecture/
- NgRx (Angular): https://ngrx.io/

## Notas Adicionales
Esta arquitectura se inspira en Redux pero está adaptada para Vela:
- **Type Safety**: Usa generics de Rust para type safety fuerte
- **Performance**: Usa Arc<RwLock<>> para acceso concurrente eficiente
- **Integration**: Diseñado para trabajar con el sistema reactivo de Vela
- **Extensibility**: Middleware system permite extensiones como sagas, thunks, etc.</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-035R-store-architecture.md