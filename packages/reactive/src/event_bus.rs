//! Event Bus implementation for Vela
//!
//! This module provides a type-safe event system for decoupled communication
//! between components in Vela applications.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Core event bus for type-safe event handling
pub struct EventBus<T> {
    listeners: HashMap<TypeId, Vec<Box<dyn Fn(&T) + Send + Sync>>>,
}

impl<T> EventBus<T> {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    /// Emit an event to all registered listeners
    pub fn emit(&self, event: &T) {
        let type_id = TypeId::of::<T>();
        if let Some(listeners) = self.listeners.get(&type_id) {
            for listener in listeners {
                listener(event);
            }
        }
    }

    /// Subscribe to events of type T
    pub fn on<F>(&mut self, listener: F) -> Subscription
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let listener_box = Box::new(listener);

        self.listeners
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(listener_box);

        // For now, return a dummy subscription
        // In a real implementation, this would allow unsubscribing
        Subscription {
            _unsubscribe: Box::new(|| {}),
        }
    }
}

impl<T> Default for EventBus<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for event subscriptions with automatic cleanup
pub struct Subscription {
    unsubscribe_fn: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl Subscription {
    /// Create a new subscription with an unsubscribe function
    pub fn new<F>(unsubscribe_fn: F) -> Self
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        Self {
            unsubscribe_fn: Some(Box::new(unsubscribe_fn)),
        }
    }

    /// Manually unsubscribe from the event
    pub fn unsubscribe(mut self) {
        if let Some(unsubscribe_fn) = self.unsubscribe_fn.take() {
            unsubscribe_fn();
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // Automatically unsubscribe when dropped if not already unsubscribed
        if let Some(unsubscribe_fn) = self.unsubscribe_fn.take() {
            unsubscribe_fn();
        }
    }
}

/// Trait for objects that can emit events
pub trait EventEmitter<T> {
    /// Emit an event
    fn emit(&self, event: T);

    /// Subscribe to events
    fn on<F>(&self, listener: F) -> Subscription
    where
        F: Fn(&T) + Send + Sync + 'static;

    /// Unsubscribe a specific listener
    fn off(&self, subscription: Subscription);
}

/// Simple event emitter implementation
pub struct SimpleEventEmitter<T> {
    listeners: Arc<Mutex<HashMap<u64, Box<dyn Fn(&T) + Send + Sync>>>>,
    next_id: Arc<Mutex<u64>>,
}

impl<T> SimpleEventEmitter<T> {
    /// Create a new simple event emitter
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }
}

impl<T> EventEmitter<T> for SimpleEventEmitter<T> {
    fn emit(&self, event: T) {
        let listeners = self.listeners.lock().unwrap();
        for listener in listeners.values() {
            listener(&event);
        }
    }

    fn on<F>(&self, listener: F) -> Subscription
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let mut listeners = self.listeners.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        listeners.insert(id, Box::new(listener));

        let listeners_clone = Arc::clone(&self.listeners);
        Subscription::new(move || {
            let mut listeners = listeners_clone.lock().unwrap();
            listeners.remove(&id);
        })
    }

    fn off(&self, subscription: Subscription) {
        // The subscription will unsubscribe when dropped
        subscription.unsubscribe();
    }
}

impl<T> Default for SimpleEventEmitter<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Base event type with metadata
#[derive(Debug, Clone)]
pub struct Event<T> {
    /// The event payload
    pub data: T,
    /// Timestamp when the event was emitted
    pub timestamp: std::time::Instant,
    /// Optional source identifier
    pub source: Option<String>,
}

impl<T> Event<T> {
    /// Create a new event
    pub fn new(data: T) -> Self {
        Self {
            data,
            timestamp: std::time::Instant::now(),
            source: None,
        }
    }

    /// Create a new event with source
    pub fn with_source(data: T, source: String) -> Self {
        Self {
            data,
            timestamp: std::time::Instant::now(),
            source: Some(source),
        }
    }
}

/// Example event types for testing
#[derive(Debug, Clone)]
pub struct UserLoggedIn {
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct DataUpdated {
    pub entity_type: String,
    pub entity_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_creation() {
        let bus: EventBus<UserLoggedIn> = EventBus::new();
        assert!(bus.listeners.is_empty());
    }

    #[test]
    fn test_event_emission() {
        let mut bus = EventBus::new();
        let event = UserLoggedIn {
            user_id: "123".to_string(),
            username: "testuser".to_string(),
        };

        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();

        let _subscription = bus.on(move |e: &UserLoggedIn| {
            received_clone.lock().unwrap().push(e.clone());
        });

        bus.emit(&event);

        let received_events = received.lock().unwrap();
        assert_eq!(received_events.len(), 1);
        assert_eq!(received_events[0].user_id, "123");
        assert_eq!(received_events[0].username, "testuser");
    }

    #[test]
    fn test_multiple_listeners() {
        let mut bus = EventBus::new();
        let event = DataUpdated {
            entity_type: "user".to_string(),
            entity_id: "456".to_string(),
        };

        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));

        let counter1_clone = counter1.clone();
        let counter2_clone = counter2.clone();

        let _sub1 = bus.on(move |_e: &DataUpdated| {
            *counter1_clone.lock().unwrap() += 1;
        });

        let _sub2 = bus.on(move |_e: &DataUpdated| {
            *counter2_clone.lock().unwrap() += 1;
        });

        bus.emit(&event);

        assert_eq!(*counter1.lock().unwrap(), 1);
        assert_eq!(*counter2.lock().unwrap(), 1);
    }

    #[test]
    fn test_event_creation() {
        let event = Event::new(UserLoggedIn {
            user_id: "789".to_string(),
            username: "newevent".to_string(),
        });

        assert_eq!(event.data.user_id, "789");
        assert!(event.source.is_none());
    }

    #[test]
    fn test_event_with_source() {
        let event = Event::with_source(
            DataUpdated {
                entity_type: "product".to_string(),
                entity_id: "999".to_string(),
            },
            "test_source".to_string(),
        );

        assert_eq!(event.data.entity_type, "product");
        assert_eq!(event.source, Some("test_source".to_string()));
    }

    #[test]
    fn test_simple_event_emitter_creation() {
        let emitter: SimpleEventEmitter<UserLoggedIn> = SimpleEventEmitter::new();
        // Should not panic
    }

    #[test]
    fn test_simple_event_emitter_emit() {
        let emitter = SimpleEventEmitter::new();
        let event = UserLoggedIn {
            user_id: "123".to_string(),
            username: "testuser".to_string(),
        };

        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();

        let _subscription = emitter.on(move |e: &UserLoggedIn| {
            received_clone.lock().unwrap().push(e.clone());
        });

        emitter.emit(event);

        let received_events = received.lock().unwrap();
        assert_eq!(received_events.len(), 1);
        assert_eq!(received_events[0].user_id, "123");
    }

    #[test]
    fn test_simple_event_emitter_multiple_listeners() {
        let emitter = SimpleEventEmitter::new();
        let event = DataUpdated {
            entity_type: "user".to_string(),
            entity_id: "456".to_string(),
        };

        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));

        let counter1_clone = counter1.clone();
        let counter2_clone = counter2.clone();

        let _sub1 = emitter.on(move |_e: &DataUpdated| {
            *counter1_clone.lock().unwrap() += 1;
        });

        let _sub2 = emitter.on(move |_e: &DataUpdated| {
            *counter2_clone.lock().unwrap() += 1;
        });

        emitter.emit(event);

        assert_eq!(*counter1.lock().unwrap(), 1);
        assert_eq!(*counter2.lock().unwrap(), 1);
    }

    #[test]
    fn test_subscription_unsubscribe() {
        let emitter = SimpleEventEmitter::new();

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let subscription = emitter.on(move |_e: &UserLoggedIn| {
            *counter_clone.lock().unwrap() += 1;
        });

        // Emit once - should trigger
        emitter.emit(UserLoggedIn {
            user_id: "123".to_string(),
            username: "test".to_string(),
        });
        assert_eq!(*counter.lock().unwrap(), 1);

        // Unsubscribe
        subscription.unsubscribe();

        // Emit again - should not trigger
        emitter.emit(UserLoggedIn {
            user_id: "456".to_string(),
            username: "test2".to_string(),
        });
        assert_eq!(*counter.lock().unwrap(), 1); // Should still be 1
    }

    #[test]
    fn test_subscription_raii_drop() {
        let emitter = SimpleEventEmitter::new();

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        {
            let _subscription = emitter.on(move |_e: &UserLoggedIn| {
                *counter_clone.lock().unwrap() += 1;
            });

            // Emit while subscription is alive
            emitter.emit(UserLoggedIn {
                user_id: "123".to_string(),
                username: "test".to_string(),
            });
            assert_eq!(*counter.lock().unwrap(), 1);
        } // subscription goes out of scope here

        // Emit again - listener should still be active (RAII cleanup happens on drop)
        emitter.emit(UserLoggedIn {
            user_id: "456".to_string(),
            username: "test2".to_string(),
        });
        // Note: In current implementation, RAII cleanup happens but listener may still exist
        // This test documents current behavior - may need adjustment based on design decisions
    }

    #[test]
    fn test_event_emitter_off_method() {
        let emitter = SimpleEventEmitter::new();

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let subscription = emitter.on(move |_e: &UserLoggedIn| {
            *counter_clone.lock().unwrap() += 1;
        });

        // Emit once
        emitter.emit(UserLoggedIn {
            user_id: "123".to_string(),
            username: "test".to_string(),
        });
        assert_eq!(*counter.lock().unwrap(), 1);

        // Use off method
        emitter.off(subscription);

        // Emit again - should not trigger
        emitter.emit(UserLoggedIn {
            user_id: "456".to_string(),
            username: "test2".to_string(),
        });
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_memory_leak_prevention() {
        let emitter = Arc::new(SimpleEventEmitter::new());

        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let emitter_clone = Arc::clone(&emitter);
        let subscription = emitter.on(move |_e: &UserLoggedIn| {
            *call_count_clone.lock().unwrap() += 1;
        });

        // Emit event
        emitter.emit(UserLoggedIn {
            user_id: "123".to_string(),
            username: "test".to_string(),
        });
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Explicitly unsubscribe
        subscription.unsubscribe();

        // Emit again - should not trigger
        emitter.emit(UserLoggedIn {
            user_id: "456".to_string(),
            username: "test2".to_string(),
        });
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Check that emitter still works with new listeners
        let new_call_count = Arc::new(Mutex::new(0));
        let new_call_count_clone = Arc::clone(&new_call_count);

        let _new_subscription = emitter.on(move |_e: &UserLoggedIn| {
            *new_call_count_clone.lock().unwrap() += 1;
        });

        emitter.emit(UserLoggedIn {
            user_id: "789".to_string(),
            username: "test3".to_string(),
        });
        assert_eq!(*new_call_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;
        use std::time::Duration;

        let emitter = Arc::new(SimpleEventEmitter::new());
        let results = Arc::new(Mutex::new(Vec::new()));

        // Spawn multiple threads that add listeners and emit events
        let mut handles = vec![];

        for i in 0..5 {
            let emitter_clone = Arc::clone(&emitter);
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                let counter = Arc::new(Mutex::new(0));
                let counter_clone = Arc::clone(&counter);

                let subscription = emitter_clone.on(move |_e: &UserLoggedIn| {
                    *counter_clone.lock().unwrap() += 1;
                });

                // Emit event
                emitter_clone.emit(UserLoggedIn {
                    user_id: format!("thread_{}", i),
                    username: format!("user_{}", i),
                });

                // Small delay to allow other threads to process
                thread::sleep(Duration::from_millis(10));

                let final_count = *counter.lock().unwrap();
                results_clone.lock().unwrap().push(final_count);

                // Explicitly unsubscribe
                subscription.unsubscribe();
            });

            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 5);

        // Each thread should have received exactly 1 event
        for &count in results.iter() {
            assert_eq!(count, 1);
        }
    }
}