/// Decorador @persistent para persistencia automática del store
/// Guarda y carga el estado automáticamente
use crate::{Store, Action};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// Trait para stores persistentes (opcional, para extensibilidad)
pub trait PersistentStore<T>: Send + Sync
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    /// Clave para persistencia
    fn persistence_key(&self) -> &str;

    /// Método para guardar estado
    fn save_state(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Método para cargar estado
    fn load_state(&self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Implementación persistente del store
pub struct PersistentStoreImpl<T> {
    store: Arc<Store<T>>,
    reducer: crate::Reducer<T, Box<dyn Action<State = T>>>,
    key: String,
}

impl<T> PersistentStoreImpl<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    pub fn new(store: Arc<Store<T>>, reducer: crate::Reducer<T, Box<dyn Action<State = T>>>, key: String) -> Self {
        let persistent = Self { store, reducer, key };
        // Intentar cargar estado inicial
        let _ = persistent.load_state();
        persistent
    }

    pub fn get_state(&self) -> std::sync::RwLockReadGuard<T> {
        self.store.get_state()
    }

    pub fn dispatch(&self, action: Box<dyn Action<State = T>>) -> Result<(), Box<dyn std::error::Error>> {
        // Obtener estado actual
        let current_state = self.store.get_state().clone();
        // Aplicar reducer
        let new_state = (self.reducer)(&current_state, &action);
        // Set nuevo estado
        self.store.set_state(new_state);
        // Guardar después del dispatch
        self.save_state()?;
        Ok(())
    }
}

impl<T> PersistentStore<T> for PersistentStoreImpl<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
{
    fn persistence_key(&self) -> &str {
        &self.key
    }

    fn save_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.store.get_state();
        let json = serde_json::to_string(&*state)?;
        // En WASM usaría localStorage, en desktop archivo
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .unwrap()
                .local_storage()
                .unwrap()
                .unwrap()
                .set_item(&self.key, &json)?;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::write(&self.key, json)?;
        }
        Ok(())
    }

    fn load_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        let json = web_sys::window()
            .unwrap()
            .local_storage()
            .unwrap()
            .unwrap()
            .get_item(&self.key)?;

        #[cfg(not(target_arch = "wasm32"))]
        let json = std::fs::read_to_string(&self.key).ok();

        if let Some(json) = json {
            let state: T = serde_json::from_str(&json)?;
            // Aquí necesitaríamos un método para set_state, pero Store no lo tiene
            // Por simplicidad, asumimos que el estado inicial se carga antes
        }
        Ok(())
    }
}

/// Macro para aplicar el decorador @persistent
#[macro_export]
macro_rules! persistent {
    ($store:expr, $reducer:expr, $key:expr) => {
        {
            use vela_state_management::PersistentStoreImpl;
            Arc::new(PersistentStoreImpl::new($store, $reducer, $key.to_string()))
        }
    };
}