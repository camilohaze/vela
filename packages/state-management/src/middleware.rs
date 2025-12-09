/// Sistema de middleware para el store Redux-style
/// Permite interceptar y modificar dispatch de acciones
use crate::Action;
use std::sync::Arc;
use std::collections::VecDeque;
use std::any::Any;

/// Tipo de función middleware
/// Recibe el store, la función next, y la acción
pub type MiddlewareFn<State> = Arc<dyn Fn(&dyn StoreInterface<State>, &dyn Fn(&dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>>, &dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>;

/// Trait para stores que soportan middleware
pub trait StoreInterface<T>: Send + Sync {
    fn get_state(&self) -> std::sync::RwLockReadGuard<T>;
    fn set_state(&self, state: T);
    fn dispatch_raw(&self, action: &dyn Action<State = T>) -> Result<(), Box<dyn std::error::Error>>;
}

/// Stack de middlewares
pub struct MiddlewareStack<State> {
    middlewares: Vec<MiddlewareFn<State>>,
}

impl<State> MiddlewareStack<State>
where
    State: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        MiddlewareStack {
            middlewares: Vec::new(),
        }
    }

    pub fn add<M: Middleware<State> + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(move |store, next, action| {
            middleware.process(store, next, action)
        }));
        self
    }

    pub fn build_dispatch(&self, store: Arc<dyn StoreInterface<State>>, base_dispatch: Arc<dyn Fn(&dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>) -> impl Fn(&dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>> {
        let mut dispatch = base_dispatch;

        for middleware in self.middlewares.iter().rev() {
            let store_clone = Arc::clone(&store);
            let dispatch_clone = Arc::clone(&dispatch);
            let middleware_clone = Arc::clone(middleware);

            dispatch = Arc::new(move |action| {
                let store_ref = store_clone.as_ref();
                let dispatch_ref = dispatch_clone.as_ref();
                let middleware_ref = middleware_clone.as_ref();
                middleware_ref(store_ref, dispatch_ref, action)
            });
        }

        move |action| dispatch(action)
    }
}

/// Trait para middlewares
pub trait Middleware<State>: Send + Sync {
    fn process(&self, store: &dyn StoreInterface<State>, next: &dyn Fn(&dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>>, action: &dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>>;
}

/// Middleware de logging
pub struct LoggingMiddleware;

impl<State> Middleware<State> for LoggingMiddleware
where
    State: std::fmt::Debug + Send + Sync + 'static,
{
    fn process(&self, store: &dyn StoreInterface<State>, next: &dyn Fn(&dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>>, action: &dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 [ACTION] {}", action.action_type());
        let state_before = format!("{:?}", *store.get_state());
        println!("📊 [STATE BEFORE] {}", state_before.chars().take(100).collect::<String>());

        let result = next(action);

        let state_after = format!("{:?}", *store.get_state());
        println!("📊 [STATE AFTER] {}", state_after.chars().take(100).collect::<String>());

        result
    }
}

/// Middleware para time-travel debugging
pub struct TimeTravelMiddleware<State> {
    history: std::sync::Mutex<VecDeque<State>>,
    max_history: usize,
}

impl<State> TimeTravelMiddleware<State>
where
    State: Clone + Send + Sync,
{
    pub fn new(max_history: usize) -> Self {
        TimeTravelMiddleware {
            history: std::sync::Mutex::new(VecDeque::new()),
            max_history,
        }
    }

    pub fn get_history(&self) -> Vec<State> {
        self.history.lock().unwrap().iter().cloned().collect()
    }

    pub fn jump_to_state(&self, index: usize, store: &dyn StoreInterface<State>) -> Result<(), Box<dyn std::error::Error>> {
        let history = self.history.lock().unwrap();
        if let Some(state) = history.get(index) {
            store.set_state(state.clone());
        }
        Ok(())
    }
}

impl<State> Middleware<State> for TimeTravelMiddleware<State>
where
    State: Clone + Send + Sync,
{
    fn process(&self, store: &dyn StoreInterface<State>, next: &dyn Fn(&dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>>, action: &dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>> {
        // Guardar estado antes
        let state_before = (*store.get_state()).clone();
        let result = next(action);

        // Guardar estado después
        let state_after = (*store.get_state()).clone();

        let mut history = self.history.lock().unwrap();
        history.push_back(state_before);
        history.push_back(state_after);

        // Mantener límite de historial
        while history.len() > self.max_history {
            history.pop_front();
        }

        result
    }
}

/// Middleware para acciones asíncronas (thunks)
pub struct ThunkMiddleware;

impl<State> Middleware<State> for ThunkMiddleware
where
    State: Send + Sync + 'static,
{
    fn process(&self, store: &dyn StoreInterface<State>, next: &dyn Fn(&dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>>, action: &dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>> {
        // Si la acción es una función (thunk), ejecutarla
        if let Some(thunk) = (action as &dyn Any).downcast_ref::<ThunkAction<State>>() {
            return thunk.execute(store);
        }

        // Si no es thunk, pasar al siguiente middleware
        next(action)
    }
}

/// Acción thunk (función que puede dispatch otras acciones)
pub struct ThunkAction<State> {
    thunk_fn: Box<dyn Fn(&dyn StoreInterface<State>) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>,
}

impl<State> ThunkAction<State> {
    pub fn new<F>(thunk_fn: F) -> Self
    where
        F: Fn(&dyn StoreInterface<State>) -> Result<(), Box<dyn std::error::Error>> + Send + Sync + 'static,
    {
        ThunkAction {
            thunk_fn: Box::new(thunk_fn),
        }
    }

    fn execute(&self, store: &dyn StoreInterface<State>) -> Result<(), Box<dyn std::error::Error>> {
        (self.thunk_fn)(store)
    }
}

impl<State: 'static> Action for ThunkAction<State> {
    type State = State;

    fn action_type(&self) -> &'static str {
        "THUNK"
    }
}

/// Función helper para crear store con middlewares
pub fn apply_middleware<State>(
    store: Arc<dyn StoreInterface<State>>,
    middleware_stack: MiddlewareStack<State>,
) -> Arc<dyn Fn(&dyn Action<State = State>) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>
where
    State: Clone + Send + Sync + 'static,
{
    let store_clone = Arc::clone(&store);
    let base_dispatch = Arc::new(move |action: &dyn Action<State = State>| {
        store_clone.dispatch_raw(action)
    });

    Arc::new(middleware_stack.build_dispatch(store, base_dispatch))
}

/// Macros para crear middlewares fácilmente
#[macro_export]
macro_rules! create_middleware {
    ($name:ident, $state:ty, $body:expr) => {
        pub struct $name;

        impl Middleware<$state> for $name {
            fn process(&self, store: &dyn StoreInterface<$state>, next: &dyn Fn(&dyn Action<State = $state>) -> Result<(), Box<dyn std::error::Error>>, action: &dyn Action<State = $state>) -> Result<(), Box<dyn std::error::Error>> {
                $body(store, next, action)
            }
        }
    };
}

#[macro_export]
macro_rules! thunk {
    ($thunk_fn:expr) => {
        ThunkAction::new($thunk_fn)
    };
}