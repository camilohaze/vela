//! Event system error types
//!
//! This module defines all error types that can occur in the event system,
//! providing comprehensive error handling for event publishing, subscription,
//! and processing operations.

use std::fmt;

/// Errors that can occur in the event system
#[derive(Debug, Clone, PartialEq)]
pub enum EventError {
    /// Handler execution failed
    HandlerFailed {
        /// The event type that failed
        event_type: String,
        /// The error message from the handler
        cause: String,
    },

    /// Event publishing failed
    PublishFailed {
        /// The event type being published
        event_type: String,
        /// The underlying cause
        cause: String,
    },

    /// Handler registration failed
    RegistrationFailed {
        /// The event type for registration
        event_type: String,
        /// The underlying cause
        cause: String,
    },

    /// Handler unregistration failed
    UnregistrationFailed {
        /// The event type for unregistration
        event_type: String,
        /// The underlying cause
        cause: String,
    },

    /// Invalid event type
    InvalidEventType {
        /// The invalid type name
        type_name: String,
    },

    /// Handler panicked during execution
    HandlerPanicked {
        /// The event type being processed
        event_type: String,
    },

    /// Event bus is shut down
    BusShutdown,

    /// Timeout occurred while waiting for handler
    Timeout {
        /// The operation that timed out
        operation: String,
    },

    /// Type mismatch during downcast
    TypeMismatch {
        /// Expected type name
        expected: String,
        /// Actual type name
        actual: String,
    },
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::HandlerFailed { event_type, cause } => {
                write!(f, "Handler failed for event '{}': {}", event_type, cause)
            }
            EventError::PublishFailed { event_type, cause } => {
                write!(f, "Failed to publish event '{}': {}", event_type, cause)
            }
            EventError::RegistrationFailed { event_type, cause } => {
                write!(f, "Failed to register handler for event '{}': {}", event_type, cause)
            }
            EventError::UnregistrationFailed { event_type, cause } => {
                write!(f, "Failed to unregister handler for event '{}': {}", event_type, cause)
            }
            EventError::InvalidEventType { type_name } => {
                write!(f, "Invalid event type: {}", type_name)
            }
            EventError::HandlerPanicked { event_type } => {
                write!(f, "Handler panicked while processing event '{}'", event_type)
            }
            EventError::BusShutdown => {
                write!(f, "Event bus has been shut down")
            }
            EventError::Timeout { operation } => {
                write!(f, "Timeout occurred during '{}'", operation)
            }
            EventError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected '{}', got '{}'", expected, actual)
            }
        }
    }
}

impl std::error::Error for EventError {}

/// Result type alias for event operations
pub type EventResult<T> = Result<T, EventError>;