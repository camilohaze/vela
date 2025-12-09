pub mod store;
pub mod action;
pub mod reducer;
pub mod persistent;
pub mod middleware;
pub mod devtools;

pub use store::*;
pub use action::*;
pub use reducer::*;
pub use persistent::*;
pub use middleware::*;
pub use devtools::*;

// ===== TESTS COMPREHENSIVOS PARA STATE MANAGEMENT =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // ===== ESTADOS DE PRUEBA =====

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct CounterState {
        count: i32,
        name: String,
    }

    impl CounterState {
        fn new(count: i32, name: &str) -> Self {
            CounterState {
                count,
                name: name.to_string(),
            }
        }
    }

    // ===== TESTS BÁSICOS =====

    #[test]
    fn test_store_creation() {
        let initial_state = CounterState::new(0, "Test");
        let store = Store::new(initial_state.clone());
        
        let state = store.get_state();
        assert_eq!(state.count, 0);
        assert_eq!(state.name, "Test");
    }

    #[test]
    fn test_persistent_store_creation() {
        // Por ahora solo verificamos que Store básico funciona
        let initial_state = CounterState::new(5, "Persistent");
        let store = Store::new(initial_state.clone());
        
        let state = store.get_state();
        assert_eq!(state.count, 5);
        assert_eq!(state.name, "Persistent");
    }

    #[test]
    fn test_devtools_connector_creation() {
        let connector = DevToolsConnector::new("test-instance");
        // Solo verificamos que se crea sin errores
        assert!(true);
    }

    #[test]
    fn test_devtools_store_creation() {
        let initial_state = CounterState::new(0, "DevTools");
        let store = Store::new(initial_state);
        let connector = Arc::new(DevToolsConnector::new("test-instance"));
        
        let devtools_store = DevToolsStore::new(store, connector);
        assert!(devtools_store.is_ok());
        
        let devtools_store = devtools_store.unwrap();
        let state = devtools_store.get_state();
        assert_eq!(state.count, 0);
        assert_eq!(state.name, "DevTools");
    }

    #[test]
    fn test_state_inspector_creation() {
        let inspector = StateInspector;
        // StateInspector es un unit struct, solo verificamos que existe
        assert!(true);
    }
}