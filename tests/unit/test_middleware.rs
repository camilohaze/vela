/*
Tests unitarios para el sistema de middleware

Historia: VELA-035R (EPIC-03D State Management)
Tarea: TASK-035Y
Fecha: 2025-12-09

Tests para:
- LoggingMiddleware
- TimeTravelMiddleware
- ThunkMiddleware
- MiddlewareStack
- apply_middleware function
*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use vela_state_management::*;

    // Estado de prueba
    #[derive(Debug, Clone, PartialEq)]
    struct TestState {
        counter: i32,
    }

    impl TestState {
        fn new(counter: i32) -> Self {
            TestState { counter }
        }
    }

    // Acciones de prueba
    #[derive(Debug)]
    struct IncrementAction;

    impl Action for IncrementAction {
        type State = TestState;

        fn action_type(&self) -> &'static str {
            "INCREMENT"
        }
    }

    #[derive(Debug)]
    struct DecrementAction;

    impl Action for DecrementAction {
        type State = TestState;

        fn action_type(&self) -> &'static str {
            "DECREMENT"
        }
    }

    #[test]
    fn test_logging_middleware() {
        let initial_state = TestState::new(0);
        let store = Store::new(initial_state);
        let middleware = LoggingMiddleware;

        // Crear dispatch con middleware
        let dispatch = apply_middleware(
            Arc::new(store) as Arc<dyn StoreInterface<TestState>>,
            MiddlewareStack::new().add(middleware)
        );

        // Dispatch acción
        let action = IncrementAction;
        let result = dispatch(&action);

        assert!(result.is_ok());
    }

    #[test]
    fn test_time_travel_middleware() {
        let initial_state = TestState::new(0);
        let store = Store::new(initial_state);
        let time_travel = TimeTravelMiddleware::new(10);

        let dispatch = apply_middleware(
            Arc::new(store) as Arc<dyn StoreInterface<TestState>>,
            MiddlewareStack::new().add(time_travel.clone())
        );

        // Dispatch varias acciones
        dispatch(&IncrementAction).unwrap();
        dispatch(&IncrementAction).unwrap();
        dispatch(&DecrementAction).unwrap();

        // Verificar historial
        let history = time_travel.get_history();
        assert!(history.len() > 0);
    }

    #[test]
    fn test_thunk_middleware() {
        let initial_state = TestState::new(0);
        let store = Store::new(initial_state);

        let dispatch = apply_middleware(
            Arc::new(store) as Arc<dyn StoreInterface<TestState>>,
            MiddlewareStack::new().add(ThunkMiddleware)
        );

        // Crear thunk que incrementa contador
        let thunk_action = ThunkAction::new(|_store| {
            // Thunk vacío para test
            Ok(())
        });

        // Dispatch thunk
        let result = dispatch(&thunk_action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_middleware_stack_builder_pattern() {
        let stack = MiddlewareStack::<TestState>::new()
            .add(LoggingMiddleware)
            .add(ThunkMiddleware);

        // Verificar que se pueden encadenar
        assert_eq!(stack.middlewares.len(), 2);
    }

    #[test]
    fn test_thunk_action_creation() {
        let thunk = ThunkAction::new(|_store| {
            // Thunk vacío para test
            Ok(())
        });

        assert_eq!(thunk.action_type(), "THUNK");
    }
}