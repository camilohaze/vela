//! Event system for the Vela runtime
//!
//! This module provides a thread-safe event bus for asynchronous event handling.
//! It supports type-safe events and handlers with automatic dependency injection.

pub mod error;
pub mod handler;
pub mod bus;

pub use error::{EventError, EventResult};
pub use handler::{Event, EventHandler, EventPublisher, EventSubscriber, TypedEventHandler, HandlerWrapperTrait, HandlerWrapper};
pub use bus::EventBus;