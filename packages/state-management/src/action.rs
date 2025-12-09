/*
Implementación de Action trait para el sistema de Store Redux-style

Historia: VELA-035R (EPIC-03D State Management)
Tarea: TASK-035T
Fecha: 2025-12-09

Descripción:
Trait Action para eventos tipados que modifican el estado del store.
Implementa pattern matching y type safety para acciones Redux-style.
*/

use std::any::Any;

/// Trait base para todas las acciones que pueden ser enviadas al store
/// Proporciona type safety y pattern matching para eventos de estado
pub trait Action: Any + Send + Sync + 'static {
    /// Tipo del estado que esta acción puede modificar
    type State;

    /// Identificador único de la acción (útil para debugging y logging)
    fn action_type(&self) -> &'static str;

    /// Método opcional para metadata adicional de la acción
    fn metadata(&self) -> Option<std::collections::HashMap<String, String>> {
        None
    }
}

/// Macro helper para implementar Action con type safety
#[macro_export]
macro_rules! action {
    ($action_name:ident, $state_type:ty, $type_str:expr) => {
        impl Action for $action_name {
            type State = $state_type;

            fn action_type(&self) -> &'static str {
                $type_str
            }
        }
    };
}

/// Macro helper para acciones con metadata
#[macro_export]
macro_rules! action_with_meta {
    ($action_name:ident, $state_type:ty, $type_str:expr, $meta_fn:expr) => {
        impl Action for $action_name {
            type State = $state_type;

            fn action_type(&self) -> &'static str {
                $type_str
            }

            fn metadata(&self) -> Option<std::collections::HashMap<String, String>> {
                Some($meta_fn(self))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Ejemplo de acción simple
    #[derive(Debug, Clone)]
    struct IncrementCounter;

    action!(IncrementCounter, i32, "INCREMENT_COUNTER");

    // Ejemplo de acción con payload
    #[derive(Debug, Clone)]
    struct SetCounter {
        pub value: i32,
    }

    action!(SetCounter, i32, "SET_COUNTER");

    // Ejemplo de acción con metadata
    #[derive(Debug, Clone)]
    struct UpdateUser {
        pub user_id: String,
        pub name: String,
    }

    action_with_meta!(UpdateUser, String, "UPDATE_USER", |action: &UpdateUser| {
        let mut meta = HashMap::new();
        meta.insert("user_id".to_string(), action.user_id.clone());
        meta.insert("operation".to_string(), "update".to_string());
        meta
    });

    #[test]
    fn test_action_type() {
        let action = IncrementCounter;
        assert_eq!(action.action_type(), "INCREMENT_COUNTER");
    }

    #[test]
    fn test_action_with_payload() {
        let action = SetCounter { value: 42 };
        assert_eq!(action.action_type(), "SET_COUNTER");
        assert!(action.metadata().is_none());
    }

    #[test]
    fn test_action_with_metadata() {
        let action = UpdateUser {
            user_id: "user123".to_string(),
            name: "John Doe".to_string(),
        };
        assert_eq!(action.action_type(), "UPDATE_USER");

        let meta = action.metadata().unwrap();
        assert_eq!(meta.get("user_id"), Some(&"user123".to_string()));
        assert_eq!(meta.get("operation"), Some(&"update".to_string()));
    }

    #[test]
    fn test_action_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<IncrementCounter>();
        assert_send_sync::<SetCounter>();
        assert_send_sync::<UpdateUser>();
    }
}