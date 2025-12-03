//! Event handler traits and types
//!
//! This module defines the core traits for event handling, including
//! the Event trait for event types and EventHandler trait for processors.

use super::error::{EventError, EventResult};

/// Trait that all events must implement
///
/// Events are data structures that can be published to the event bus.
/// They must be Send, Sync, and Clone for thread-safe distribution.
pub trait Event: Send + Sync + Clone + 'static {
    /// Get the type ID of this event
    fn event_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    /// Get the event type name for debugging
    fn event_type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Blanket implementation for all types that meet the requirements
impl<T> Event for T where T: Send + Sync + Clone + 'static {}

/// Trait for event handlers
///
/// Handlers process events asynchronously. They can be registered with
/// the event bus to receive specific types of events.
#[async_trait::async_trait]
pub trait EventHandler<E: Event>: Send + Sync + 'static {
    /// Handle an event
    ///
    /// This method is called when an event is published.
    /// The event is passed as a typed reference.
    ///
    /// # Arguments
    /// * `event` - The event to handle
    ///
    /// # Returns
    /// Ok(()) if handling succeeded, Err if it failed
    async fn handle(&self, event: &E) -> EventResult<()>;

    /// Get the handler name for debugging
    fn handler_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Type alias for event handlers (for convenience)
pub trait TypedEventHandler<E: Event>: EventHandler<E> {}

/// Blanket implementation for all EventHandler implementations
impl<E: Event, T: EventHandler<E>> TypedEventHandler<E> for T {}

/// Type-erased event handler trait for storage in collections
#[async_trait::async_trait]
pub trait HandlerWrapperTrait: Send + Sync {
    /// Handle an event with type erasure
    async fn handle(&self, event: &(dyn std::any::Any + Send + Sync)) -> EventResult<()>;
}

/// Wrapper to type-erase event handlers for storage
pub struct HandlerWrapper<E: Event, H: EventHandler<E>> {
    handler: H,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: Event, H: EventHandler<E>> HandlerWrapper<E, H> {
    /// Create a new handler wrapper
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<E: Event, H: EventHandler<E>> HandlerWrapperTrait for HandlerWrapper<E, H> {
    async fn handle(&self, event: &(dyn std::any::Any + Send + Sync)) -> EventResult<()> {
        // Downcast the event to the expected type
        let typed_event = event.downcast_ref::<E>().ok_or_else(|| {
            EventError::TypeMismatch {
                expected: std::any::type_name::<E>().to_string(),
                actual: std::any::type_name_of_val(event).to_string(),
            }
        })?;

        self.handler.handle(typed_event).await
    }
}

/// Trait for event publishers
///
/// Publishers can send events to the event bus for distribution
/// to registered handlers.
pub trait EventPublisher<E: Event> {
    /// Publish an event
    ///
    /// Sends the event to all registered handlers for this event type.
    ///
    /// # Arguments
    /// * `event` - The event to publish
    ///
    /// # Returns
    /// Ok(()) if publishing succeeded, Err if it failed
    fn publish(&self, event: E) -> EventResult<()>;
}

/// Trait for event subscribers
///
/// Subscribers can register handlers to receive events of specific types.
pub trait EventSubscriber<E: Event> {
    /// Subscribe to events of type E
    ///
    /// Registers a handler to receive events of the specified type.
    ///
    /// # Arguments
    /// * `handler` - The handler to register
    ///
    /// # Returns
    /// Ok(()) if subscription succeeded, Err if it failed
    fn subscribe<H>(&self, handler: H) -> EventResult<()>
    where
        H: EventHandler<E> + 'static;

    /// Unsubscribe a handler
    ///
    /// Removes a previously registered handler.
    ///
    /// # Arguments
    /// * `handler` - The handler to remove
    ///
    /// # Returns
    /// Ok(()) if unsubscription succeeded, Err if it failed
    fn unsubscribe<H>(&self, handler: &H) -> EventResult<()>
    where
        H: EventHandler<E> + 'static;
}