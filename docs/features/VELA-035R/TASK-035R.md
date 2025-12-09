# TASK-035R: Diseñar arquitectura de Store

## 📋 Información General
- **Historia:** VELA-035R (Diseñar arquitectura de Store)
- **Estado:** Completada ✅
- **Fecha:** 2025-12-09
- **Dependencia:** Ninguna

## 🎯 Objetivo
Diseñar una arquitectura robusta de Store pattern Redux-style que permita manejar estado global predecible, testable y debuggable en aplicaciones Vela complejas.

## 🔨 Implementación

### Arquitectura Diseñada

#### 1. Store<T> - Núcleo del Sistema
El Store actúa como contenedor único del estado global de la aplicación:

```rust
pub struct Store<T> {
    state: Arc<RwLock<T>>,                    // Estado thread-safe
    reducer: Reducer<T>,                      // Función reductora pura
    subscribers: Vec<Subscriber<T>>,          // Observadores de cambios
    middleware: Vec<Box<dyn Middleware<T>>>,  // Pipeline de efectos
}
```

#### 2. Actions - Eventos Inmutables
Sistema de acciones tipadas que representan todas las posibles modificaciones de estado:

```rust
pub trait Action: std::fmt::Debug + Send + Sync {
    fn action_type(&self) -> &'static str;
}

// Ejemplo de action
#[derive(Debug, Clone)]
pub struct CounterAction {
    pub kind: CounterActionKind,
}

#[derive(Debug, Clone)]
pub enum CounterActionKind {
    Increment,
    Decrement,
    Set(i32),
    Reset,
}
```

#### 3. Reducers - Transformaciones Puras
Funciones puras que toman estado anterior y action, retornando nuevo estado:

```rust
pub type Reducer<T> = Box<dyn Fn(&T, &dyn Action) -> T + Send + Sync>;

fn counter_reducer(state: &CounterState, action: &dyn Action) -> CounterState {
    match action.downcast_ref::<CounterAction>() {
        Some(counter_action) => match counter_action.kind {
            CounterActionKind::Increment => CounterState {
                count: state.count + 1,
                ..*state
            },
            CounterActionKind::Decrement => CounterState {
                count: state.count - 1,
                ..*state
            },
            CounterActionKind::Set(value) => CounterState {
                count: value,
                ..*state
            },
            CounterActionKind::Reset => CounterState {
                count: 0,
                ..*state
            },
        },
        None => state.clone(),
    }
}
```

#### 4. Dispatch Pipeline
Mecanismo central para enviar actions con pipeline completo:

```rust
impl<T> Store<T> {
    pub fn dispatch(&self, action: Box<dyn Action>) {
        // 1. Pre-dispatch middleware
        for middleware in &self.middleware {
            middleware.before_dispatch(&action);
        }

        // 2. Aplicar reducer (función pura)
        let new_state = (self.reducer)(&*self.state.read().unwrap(), &*action);

        // 3. Actualizar estado de forma atómica
        *self.state.write().unwrap() = new_state.clone();

        // 4. Notificar subscribers
        for subscriber in &self.subscribers {
            subscriber(&new_state);
        }

        // 5. Post-dispatch middleware
        for middleware in &self.middleware {
            middleware.after_dispatch(&action, &new_state);
        }
    }
}
```

#### 5. Selectors - Acceso Optimizado
Sistema de selectores para acceder a partes específicas del estado:

```rust
pub trait Selector<T, R> {
    fn select(&self, state: &T) -> R;
    fn equality_fn(&self) -> Option<Box<dyn Fn(&R, &R) -> bool>>;
}

// Selector memoizado
struct CounterValueSelector;
impl Selector<AppState, i32> for CounterValueSelector {
    fn select(&self, state: &AppState) -> i32 {
        state.counter.count
    }

    fn equality_fn(&self) -> Option<Box<dyn Fn(&i32, &i32) -> bool>> {
        Some(Box::new(|a, b| a == b))
    }
}
```

#### 6. Middleware System Extensible
Pipeline de middleware para logging, async actions, etc.:

```rust
pub trait Middleware<T> {
    fn before_dispatch(&self, action: &dyn Action);
    fn after_dispatch(&self, action: &dyn Action, new_state: &T);
}

// Middleware incluido
pub struct LoggingMiddleware;      // Logging automático
pub struct DevToolsMiddleware;     // Time-travel debugging
pub struct PersistenceMiddleware;  // Auto-save a localStorage
pub struct ThunkMiddleware;        // Async actions
```

### Beneficios de la Arquitectura

#### Predecibilidad
- ✅ Estado solo cambia a través de actions
- ✅ Reducers son funciones puras (sin side effects)
- ✅ Historial completo de cambios

#### Testabilidad
- ✅ Reducers fácilmente testeables
- ✅ Actions serializables
- ✅ Selectors puros

#### Debugging
- ✅ Time-travel debugging
- ✅ Action logging automático
- ✅ State snapshots

#### Performance
- ✅ Selectors memoizados
- ✅ Atomic updates
- ✅ Lazy evaluation

### Integration con Vela

#### Decorators para UI
```rust
// Conectar widget al store
#[connect(store = "app_store")]
struct AppWidget {
    #[select(selector = "counter_value")]
    counter: i32,

    #[select(selector = "user_name")]
    user_name: String,
}

// Uso programático
let store = use_store::<AppState>();
let counter = use_selector::<AppState, i32>(CounterValueSelector::new());
```

#### Reactive Integration
```rust
// Store integrado con signals
let store_signal = create_store_signal(store);
let derived_value = create_derived(|| {
    let state = store_signal.get();
    state.counter.count * 2
});
```

## ✅ Criterios de Aceptación
- [x] Arquitectura Store<T> definida con thread-safety
- [x] Sistema de Actions tipadas diseñado
- [x] Reducers como funciones puras especificadas
- [x] Pipeline de dispatch con middleware diseñado
- [x] Sistema de selectors memoizados definido
- [x] Integration con UI framework planificada
- [x] ADR completo creado con alternativas consideradas
- [x] Documentación técnica completa generada

## 🔗 Referencias
- **Jira:** [VELA-035R](https://velalang.atlassian.net/browse/VELA-035R)
- **ADR:** `docs/architecture/ADR-035R-store-architecture.md`
- **Redux Pattern:** Inspiración principal
- **NgRx/Akita:** Patrones similares en otros frameworks

## 📝 Decisiones Clave

### Redux-style sobre Context API
- **Razones:** Predecibilidad, testabilidad, debugging superior
- **Trade-off:** Mayor complejidad inicial vs mejor mantenibilidad

### Middleware Pipeline
- **Razones:** Extensibilidad, separación de concerns
- **Beneficios:** Logging, persistence, async actions sin modificar core

### Type Safety Primero
- **Razones:** Prevenir errores en runtime
- **Implementación:** Generics fuertes, traits bien definidos

### Reactive Integration
- **Razones:** Mejor DX con sistema reactivo existente
- **Beneficios:** Automatic UI updates, lazy evaluation</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-035R\TASK-035R.md