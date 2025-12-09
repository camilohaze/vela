/*
Implementación de Store<T> base class para Vela

Historia: VELA-035R (EPIC-03D State Management)
Tarea: TASK-035S
Fecha: 2025-12-09

Descripción:
Clase base Store<T> para gestión de estado global con thread-safety.
Implementa patrón Redux-style con Arc<RwLock<T>> para concurrencia.
*/

use std::sync::{Arc, RwLock};
use crate::StoreInterface;

/// Store<T> base class para gestión de estado global
/// Proporciona acceso thread-safe al estado mediante Arc<RwLock<T>>
pub struct Store<T> {
    /// Estado interno protegido por RwLock para thread-safety
    state: Arc<RwLock<T>>,
}

impl<T> Store<T> {
    /// Crea un nuevo Store con estado inicial
    ///
    /// # Arguments
    /// * `initial_state` - Estado inicial del store
    ///
    /// # Returns
    /// Nueva instancia de Store<T>
    ///
    /// # Example
    /// ```rust
    /// let store = Store::new(AppState { counter: 0 });
    /// ```
    pub fn new(initial_state: T) -> Self {
        Store {
            state: Arc::new(RwLock::new(initial_state)),
        }
    }

    /// Obtiene una referencia de lectura al estado actual
    ///
    /// # Returns
    /// RwLockReadGuard que permite acceso de lectura al estado
    ///
    /// # Panics
    /// Si hay un error al adquirir el lock de lectura
    ///
    /// # Example
    /// ```rust
    /// let state = store.get_state();
    /// println!("Counter: {}", state.counter);
    /// ```
    pub fn get_state(&self) -> std::sync::RwLockReadGuard<T> {
        self.state.read().unwrap()
    }

    /// Reemplaza completamente el estado
    ///
    /// # Arguments
    /// * `new_state` - Nuevo estado que reemplazará al actual
    ///
    /// # Panics
    /// Si hay un error al adquirir el lock de escritura
    ///
    /// # Example
    /// ```rust
    /// store.set_state(AppState { counter: 42 });
    /// ```
    pub fn set_state(&self, new_state: T) {
        *self.state.write().unwrap() = new_state;
    }

    /// Obtiene un clon del Arc para compartir el store
    ///
    /// # Returns
    /// Arc<Store<T>> que puede ser compartido entre threads
    ///
    /// # Example
    /// ```rust
    /// let store_clone = store.clone_arc();
    /// ```
    pub fn clone_arc(&self) -> Arc<Store<T>> {
        Arc::new(Store {
            state: Arc::clone(&self.state),
        })
    }
}

impl<T> Clone for Store<T> {
    fn clone(&self) -> Self {
        Store {
            state: Arc::clone(&self.state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestState {
        counter: i32,
        name: String,
    }

    #[test]
    fn test_store_creation() {
        let initial_state = TestState {
            counter: 0,
            name: "Test".to_string(),
        };
        let store = Store::new(initial_state.clone());
        let state = store.get_state();
        assert_eq!(*state, initial_state);
    }

    #[test]
    fn test_store_set_state() {
        let initial_state = TestState {
            counter: 0,
            name: "Test".to_string(),
        };
        let store = Store::new(initial_state);

        let new_state = TestState {
            counter: 42,
            name: "Updated".to_string(),
        };
        store.set_state(new_state.clone());

        let state = store.get_state();
        assert_eq!(*state, new_state);
    }

    #[test]
    fn test_store_clone() {
        let initial_state = TestState {
            counter: 0,
            name: "Test".to_string(),
        };
        let store = Store::new(initial_state.clone());
        let store_clone = store.clone();

        // Ambos stores deberían compartir el mismo estado
        let state1 = store.get_state();
        let state2 = store_clone.get_state();
        assert_eq!(*state1, *state2);
        assert_eq!(*state1, initial_state);
    }
}

impl<T> StoreInterface<T> for Store<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn get_state(&self) -> std::sync::RwLockReadGuard<T> {
        self.get_state()
    }

    fn set_state(&self, state: T) {
        self.set_state(state);
    }

    fn dispatch_raw(&self, _action: &dyn crate::Action<State = T>) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implementar dispatch con reducers
        // Por ahora, solo un placeholder
        Ok(())
    }
}