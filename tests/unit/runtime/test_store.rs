/*
Tests unitarios para Store<T> base class

Jira: TASK-035S
Historia: VELA-035R (EPIC-03D State Management)
*/

use vela_runtime::Store;

#[derive(Debug, Clone, PartialEq)]
struct TestState {
    counter: i32,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_store_thread_safety() {
        use std::thread;
        use std::sync::Arc;

        let store = Arc::new(Store::new(TestState {
            counter: 0,
            name: "Test".to_string(),
        }));

        let store_clone1 = Arc::clone(&store);
        let store_clone2 = Arc::clone(&store);

        let handle1 = thread::spawn(move || {
            store_clone1.set_state(TestState {
                counter: 1,
                name: "Thread1".to_string(),
            });
        });

        let handle2 = thread::spawn(move || {
            store_clone2.set_state(TestState {
                counter: 2,
                name: "Thread2".to_string(),
            });
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // El último set_state debería prevalecer (Thread2)
        let state = store.get_state();
        assert_eq!(state.counter, 2);
        assert_eq!(state.name, "Thread2");
    }

    #[test]
    fn test_store_clone_arc() {
        let store = Store::new(TestState {
            counter: 0,
            name: "Test".to_string(),
        });

        let store_arc = store.clone_arc();

        // Cambiar estado en el original
        store.set_state(TestState {
            counter: 42,
            name: "Updated".to_string(),
        });

        // El clon debería ver el cambio
        let state = store_arc.get_state();
        assert_eq!(state.counter, 42);
        assert_eq!(state.name, "Updated");
    }
}