//! # Vela Events
//!
//! Sistema de eventos pub/sub para aplicaciones Rust.
//!
//! Este crate proporciona un bus de eventos thread-safe para manejo asíncrono
//! de eventos con soporte para tipos seguros y handlers con inyección de dependencias.

pub mod error;
pub mod handler;
pub mod bus;

pub use error::{EventError, EventResult};
pub use handler::{Event, EventHandler, EventPublisher, EventSubscriber, TypedEventHandler, HandlerWrapperTrait, HandlerWrapper};
pub use bus::EventBus;