/*
Tests unitarios para Action trait y Reducer functions

Jira: TASK-035T
Historia: VELA-035R (EPIC-03D State Management)
*/

use vela_runtime::{Action, ReducerBuilder, create_reducer, combine_reducers};
use std::collections::HashMap;
use std::any::Any;

// Test actions
#[derive(Debug, Clone)]
struct IncrementCounter;

vela_runtime::action!(IncrementCounter, CounterState, "INCREMENT_COUNTER");

#[derive(Debug, Clone)]
struct SetCounterValue {
    value: i32,
}

vela_runtime::action!(SetCounterValue, CounterState, "SET_COUNTER_VALUE");

#[derive(Debug, Clone)]
struct UpdateUser {
    user_id: String,
    name: String,
}

vela_runtime::action_with_meta!(UpdateUser, UserState, "UPDATE_USER", |action: &UpdateUser| {
    let mut meta = HashMap::new();
    meta.insert("user_id".to_string(), action.user_id.clone());
    meta.insert("operation".to_string(), "update".to_string());
    meta
});

// Test states
#[derive(Debug, Clone, PartialEq)]
struct CounterState {
    count: i32,
}

impl CounterState {
    fn new() -> Self {
        CounterState { count: 0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UserState {
    users: HashMap<String, String>,
}

impl UserState {
    fn new() -> Self {
        UserState {
            users: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_trait() {
        let action = IncrementCounter;
        assert_eq!(action.action_type(), "INCREMENT_COUNTER");
        assert!(action.metadata().is_none());
    }

    #[test]
    fn test_action_with_payload() {
        let action = SetCounterValue { value: 42 };
        assert_eq!(action.action_type(), "SET_COUNTER_VALUE");
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
    fn test_reducer_builder() {
        let builder = ReducerBuilder::new()
            .add_reducer(|state: &CounterState, _action: &IncrementCounter| {
                CounterState {
                    count: state.count + 1,
                }
            })
            .add_reducer(|state: &CounterState, action: &SetCounterValue| {
                CounterState {
                    count: action.value,
                }
            });

        let reducer = builder.build();
        let initial_state = CounterState::new();

        // Test increment
        let action: Box<dyn Any + Send + Sync> = Box::new(IncrementCounter);
        let new_state = reducer(&initial_state, &action);
        assert_eq!(new_state.count, 1);

        // Test set value
        let action: Box<dyn Any + Send + Sync> = Box::new(SetCounterValue { value: 42 });
        let new_state = reducer(&new_state, &action);
        assert_eq!(new_state.count, 42);
    }

    #[test]
    fn test_create_reducer() {
        let reducer = create_reducer(|state: &CounterState, _action: &IncrementCounter| {
            CounterState {
                count: state.count + 1,
            }
        });

        let initial_state = CounterState::new();
        let action = IncrementCounter;
        let new_state = reducer(&initial_state, &action);

        assert_eq!(new_state.count, 1);
        assert_eq!(initial_state.count, 0); // Immutability
    }

    #[test]
    fn test_combine_reducers() {
        let increment_reducer = create_reducer(|state: &CounterState, _action: &IncrementCounter| {
            CounterState {
                count: state.count + 1,
            }
        });

        let double_reducer = create_reducer(|state: &CounterState, _action: &IncrementCounter| {
            CounterState {
                count: state.count * 2,
            }
        });

        let combined = combine_reducers(vec![increment_reducer, double_reducer]);
        let initial_state = CounterState { count: 5 };

        let action: Box<dyn Any + Send + Sync> = Box::new(IncrementCounter);
        let new_state = combined(&initial_state, &action);

        // Primero incrementa (5 + 1 = 6), luego duplica (6 * 2 = 12)
        assert_eq!(new_state.count, 12);
    }

    #[test]
    fn test_reducer_no_change_for_unknown_action() {
        let builder = ReducerBuilder::new()
            .add_reducer(|state: &CounterState, _action: &IncrementCounter| {
                CounterState {
                    count: state.count + 1,
                }
            });

        let reducer = builder.build();
        let initial_state = CounterState { count: 10 };

        // Acción no manejada por ningún reducer
        let action: Box<dyn Any + Send + Sync> = Box::new(SetCounterValue { value: 42 });
        let new_state = reducer(&initial_state, &action);

        // Estado debe permanecer sin cambios
        assert_eq!(new_state.count, 10);
    }

    #[test]
    fn test_user_reducer() {
        let reducer = create_reducer(|state: &UserState, action: &UpdateUser| {
            let mut new_users = state.users.clone();
            new_users.insert(action.user_id.clone(), action.name.clone());
            UserState { users: new_users }
        });

        let initial_state = UserState::new();
        let action = UpdateUser {
            user_id: "user123".to_string(),
            name: "John Doe".to_string(),
        };

        let new_state = reducer(&initial_state, &action);

        assert_eq!(initial_state.users.len(), 0);
        assert_eq!(new_state.users.len(), 1);
        assert_eq!(new_state.users.get("user123"), Some(&"John Doe".to_string()));
    }

    #[test]
    fn test_action_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<IncrementCounter>();
        assert_send_sync::<SetCounterValue>();
        assert_send_sync::<UpdateUser>();
    }
}