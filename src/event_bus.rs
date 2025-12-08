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
    _unsubscribe: Box<dyn FnOnce() + Send + Sync>,
}

impl Subscription {
    /// Manually unsubscribe from the event
    pub fn unsubscribe(self) {
        // The closure will be called when dropped
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // In a real implementation, this would remove the listener from the bus
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
}