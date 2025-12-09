/*
Tests unitarios para el sistema de DevTools

Historia: VELA-035R (EPIC-03D State Management)
Tarea: TASK-035Z
Fecha: 2025-12-09

Tests para:
- DevToolsConnector
- DevToolsMiddleware
- DevToolsStore
- StateInspector
- Protocolo de mensajes
*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Estado de prueba
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestState {
        counter: i32,
        name: String,
    }

    impl TestState {
        fn new(counter: i32, name: &str) -> Self {
            TestState {
                counter,
                name: name.to_string(),
            }
        }
    }

    // Acción de prueba
    #[derive(Debug)]
    struct IncrementAction;

    impl Action for IncrementAction {
        type State = TestState;

        fn action_type(&self) -> &'static str {
            "INCREMENT"
        }
    }

    #[test]
    fn test_devtools_connector_creation() {
        let connector = DevToolsConnector::new("test-instance");
        assert_eq!(connector.instance_id, "test-instance");
        assert!(!connector.is_connected());
    }

    #[test]
    fn test_devtools_connector_connect() {
        let connector = DevToolsConnector::new("test-instance");
        let result = connector.connect();
        assert!(result.is_ok());
        assert!(connector.is_connected());
    }

    #[test]
    fn test_devtools_connector_send_message() {
        let connector = DevToolsConnector::new("test-instance");
        connector.connect().unwrap();

        let message = DevToolsMessage::Ping;
        let result = connector.send_message(message);
        assert!(result.is_ok());

        // Verificar que el mensaje fue encolado
        let messages = connector.receive_messages();
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_devtools_middleware_creation() {
        let connector = Arc::new(DevToolsConnector::new("test"));
        let middleware = DevToolsMiddleware::new(Arc::clone(&connector));

        // Verificar que se creó correctamente
        assert_eq!(middleware.connector.instance_id, "test");
    }

    #[test]
    fn test_devtools_middleware_init() {
        let connector = Arc::new(DevToolsConnector::new("test"));
        let middleware = DevToolsMiddleware::new(Arc::clone(&connector));
        connector.connect().unwrap();

        let initial_state = TestState::new(0, "test");

        let result = middleware.init(&initial_state);
        assert!(result.is_ok());

        // Verificar que se envió mensaje de inicialización
        let messages = connector.receive_messages();
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            DevToolsMessage::Init { instance_id, state, features } => {
                assert_eq!(instance_id, "test");
                assert!(!state.is_empty());
                assert!(features.contains(&"timeTravel".to_string()));
            }
            _ => panic!("Expected Init message"),
        }
    }

    #[test]
    fn test_devtools_middleware_process_action() {
        let connector = Arc::new(DevToolsConnector::new("test"));
        let middleware = DevToolsMiddleware::new(Arc::clone(&connector));
        connector.connect().unwrap();

        let state_before = TestState::new(0, "before");
        let state_after = TestState::new(1, "after");
        let action = IncrementAction;

        let result = middleware.process_action(&action, &state_before, &state_after);
        assert!(result.is_ok());

        // Verificar que se envió mensaje de acción
        let messages = connector.receive_messages();
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            DevToolsMessage::ActionDispatched { action_type, state_before: sb, state_after: sa, .. } => {
                assert_eq!(action_type, "INCREMENT");
                assert!(!sb.is_empty());
                assert!(!sa.is_empty());
            }
            _ => panic!("Expected ActionDispatched message"),
        }
    }

    #[test]
    fn test_devtools_store_creation() {
        let initial_state = TestState::new(0, "test");
        let store = Store::new(initial_state);
        let connector = Arc::new(DevToolsConnector::new("test-instance"));

        let devtools_store = DevToolsStore::new(store, Arc::clone(&connector));
        assert!(devtools_store.is_ok());
    }

    #[test]
    fn test_devtools_store_dispatch() {
        let initial_state = TestState::new(0, "test");
        let store = Store::new(initial_state);
        let connector = Arc::new(DevToolsConnector::new("test-instance"));

        let devtools_store = DevToolsStore::new(store, Arc::clone(&connector)).unwrap();

        let action = IncrementAction;
        let result = devtools_store.dispatch(&action);
        assert!(result.is_ok());

        // Verificar que se enviaron mensajes
        let messages = connector.receive_messages();
        assert!(messages.len() >= 1); // Al menos Init
    }

    #[test]
    fn test_devtools_store_export_import_state() {
        let initial_state = TestState::new(42, "test");
        let store = Store::new(initial_state.clone());
        let connector = Arc::new(DevToolsConnector::new("test-instance"));

        let devtools_store = DevToolsStore::new(store, Arc::clone(&connector)).unwrap();

        // Exportar estado
        let exported = devtools_store.export_state();
        assert!(exported.is_ok());

        // Crear nuevo store e importar
        let new_store = Store::new(TestState::new(0, "empty"));
        let new_connector = Arc::new(DevToolsConnector::new("new-instance"));
        let new_devtools_store = DevToolsStore::new(new_store, new_connector).unwrap();

        let result = new_devtools_store.import_state(exported.unwrap());
        assert!(result.is_ok());

        // Verificar que el estado fue importado
        let imported_state = new_devtools_store.get_state();
        assert_eq!(*imported_state, initial_state);
    }

    #[test]
    fn test_state_inspector_diff() {
        let state1 = TestState::new(1, "first");
        let state2 = TestState::new(2, "second");

        let diff = StateInspector::diff_states(&state1, &state2);
        assert!(diff.is_ok());
        // Diff debería mostrar algún cambio
        assert!(!diff.unwrap().is_empty());
    }

    #[test]
    fn test_state_inspector_format() {
        let state = TestState::new(42, "test");

        let formatted = StateInspector::format_state(&state);
        assert!(formatted.is_ok());

        let json = formatted.unwrap();
        assert!(json.contains("42"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_devtools_message_serialization() {
        let message = DevToolsMessage::Ping;

        // Verificar que se puede serializar
        let serialized = serde_json::to_string(&message);
        assert!(serialized.is_ok());

        // Verificar que se puede deserializar
        let deserialized: Result<DevToolsMessage, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
        assert!(matches!(deserialized.unwrap(), DevToolsMessage::Ping));
    }

    #[test]
    fn test_devtools_command_processing() {
        let connector = DevToolsConnector::new("test");

        // Test Ping -> Pong
        let ping_result = connector.process_command(DevToolsMessage::Ping);
        assert!(ping_result.is_ok());
        assert!(ping_result.unwrap().is_none()); // No command returned

        // Verificar que se envió Pong
        let messages = connector.receive_messages();
        assert!(messages.contains(&DevToolsMessage::Pong));

        // Test TimeTravel command
        let time_travel_result = connector.process_command(DevToolsMessage::TimeTravel { target_index: 5 });
        assert!(time_travel_result.is_ok());

        match time_travel_result.unwrap() {
            Some(DevToolsCommand::TimeTravel { target_index }) => assert_eq!(target_index, 5),
            _ => panic!("Expected TimeTravel command"),
        }
    }
}