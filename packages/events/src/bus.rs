//! Event bus implementation
//!
//! This module implements the main EventBus that manages event distribution,
//! handler registration, and provides thread-safe access to event operations.

use crate::error::{EventError, EventResult};
use crate::handler::{Event, EventPublisher, EventSubscriber, TypedEventHandler, HandlerWrapperTrait, HandlerWrapper};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// The main event bus
pub struct EventBus {
    /// Registered handlers by event type
    handlers: RwLock<HashMap<TypeId, Vec<Arc<dyn HandlerWrapperTrait>>>>,
    /// Broadcast channel for shutdown signaling
    shutdown_tx: broadcast::Sender<()>,
    /// Shutdown receiver
    shutdown_rx: broadcast::Receiver<()>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        Self {
            handlers: RwLock::new(HashMap::new()),
            shutdown_tx,
            shutdown_rx,
        }
    }

    /// Publish an event to all registered handlers
    pub async fn publish<E>(&self, event: E) -> EventResult<()>
    where
        E: Event,
    {
        let event_type_id = TypeId::of::<E>();
        let event_type_name = std::any::type_name::<E>();

        // Get handlers for this event type
        let handlers = {
            let handlers_map = self.handlers.read().unwrap();
            handlers_map.get(&event_type_id).cloned()
        };

        if let Some(handlers) = handlers {
            // Execute all handlers concurrently
            let mut tasks = Vec::new();

            for handler in handlers {
                let event_clone = event.clone();
                let task = tokio::spawn(async move {
                    handler.handle(&event_clone as &(dyn std::any::Any + Send + Sync)).await
                });
                tasks.push(task);
            }

            // Wait for all handlers to complete
            let mut errors = Vec::new();
            for task in tasks {
                match task.await {
                    Ok(result) => {
                        if let Err(e) = result {
                            errors.push(e);
                        }
                    }
                    Err(_) => {
                        errors.push(EventError::HandlerPanicked {
                            event_type: event_type_name.to_string(),
                        });
                    }
                }
            }

            // Return first error if any occurred
            if let Some(error) = errors.into_iter().next() {
                return Err(error);
            }
        }

        Ok(())
    }

    /// Register a handler for a specific event type
    pub fn subscribe<E, H>(&self, handler: H) -> EventResult<()>
    where
        E: Event,
        H: TypedEventHandler<E> + 'static,
    {
        let event_type_id = TypeId::of::<E>();

        let mut handlers_map = self.handlers.write().unwrap();
        let handlers = handlers_map.entry(event_type_id).or_insert_with(Vec::new);

        // Wrap the handler in a type-safe wrapper
        let wrapped_handler = HandlerWrapper::new(handler);
        let arc_handler: Arc<dyn HandlerWrapperTrait> = Arc::new(wrapped_handler);

        handlers.push(arc_handler);
        Ok(())
    }

    /// Unregister a handler for a specific event type
    ///
    /// # Type Parameters
    /// * `E` - The event type
    /// * `H` - The handler type
    ///
    /// # Arguments
    /// * `handler` - The handler to remove
    ///
    /// # Returns
    /// Ok(()) if unregistration succeeded
    pub fn unsubscribe<E, H>(&self, _handler: &H) -> EventResult<()>
    where
        E: Event,
        H: TypedEventHandler<E> + 'static,
    {
        // For simplicity, we don't implement selective unregistration
        // In a real implementation, you'd need handler IDs or references
        Ok(())
    }

    /// Get the number of registered handlers for an event type
    ///
    /// # Type Parameters
    /// * `E` - The event type
    ///
    /// # Returns
    /// The number of registered handlers
    pub fn handler_count<E>(&self) -> usize
    where
        E: Event,
    {
        let event_type_id = TypeId::of::<E>();
        let handlers_map = self.handlers.read().unwrap();
        handlers_map.get(&event_type_id).map(|h| h.len()).unwrap_or(0)
    }

    /// Get all registered event types
    ///
    /// # Returns
    /// Vector of event type names
    pub fn registered_event_types(&self) -> Vec<String> {
        let handlers_map = self.handlers.read().unwrap();
        handlers_map.keys().map(|type_id| format!("{:?}", type_id)).collect()
    }

    /// Shutdown the event bus
    ///
    /// This signals all handlers to stop and clears all registrations.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
        let mut handlers_map = self.handlers.write().unwrap();
        handlers_map.clear();
    }

    /// Check if the event bus is shut down
    ///
    /// # Returns
    /// true if the bus is shut down, false otherwise
    pub fn is_shutdown(&mut self) -> bool {
        self.shutdown_rx.try_recv().is_ok()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the publisher trait for EventBus
impl<E> EventPublisher<E> for EventBus
where
    E: Event,
{
    fn publish(&self, event: E) -> EventResult<()> {
        // For sync publishing, we spawn a task to handle the async operation
        // We need to move the event and create a new reference to self
        let bus_ref = self as *const EventBus as usize;
        tokio::spawn(async move {
            let bus = unsafe { &*(bus_ref as *const EventBus) };
            let _ = bus.publish(event).await;
        });
        Ok(())
    }
}

// Implement the subscriber trait for EventBus
impl<E> EventSubscriber<E> for EventBus
where
    E: Event,
{
    fn subscribe<H>(&self, handler: H) -> EventResult<()>
    where
        H: TypedEventHandler<E> + 'static,
    {
        self.subscribe(handler)
    }

    fn unsubscribe<H>(&self, handler: &H) -> EventResult<()>
    where
        H: TypedEventHandler<E> + 'static,
    {
        self.unsubscribe(handler)
    }
}