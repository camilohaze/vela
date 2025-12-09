//! Error types for the Dependency Injection system
//!
//! This module defines all error types that can occur during dependency
//! injection operations, providing detailed error information for debugging.

use std::fmt;

/// Errors that can occur during dependency injection operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DIError {
    /// Service not registered in the container
    ServiceNotRegistered {
        service_type: String,
    },

    /// Circular dependency detected during resolution
    CircularDependency {
        service_chain: Vec<String>,
    },

    /// Failed to resolve a dependency
    ResolutionFailed {
        service_type: String,
        cause: String,
    },

    /// Invalid scope transition attempted
    InvalidScopeTransition {
        from_scope: String,
        to_scope: String,
    },

    /// Service already registered
    ServiceAlreadyRegistered {
        service_type: String,
    },

    /// Provider creation failed
    ProviderCreationFailed {
        service_type: String,
        cause: String,
    },
}

impl fmt::Display for DIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DIError::ServiceNotRegistered { service_type } => {
                write!(f, "Service '{}' is not registered in the container", service_type)
            }
            DIError::CircularDependency { service_chain } => {
                write!(f, "Circular dependency detected: {}", service_chain.join(" -> "))
            }
            DIError::ResolutionFailed { service_type, cause } => {
                write!(f, "Failed to resolve service '{}': {}", service_type, cause)
            }
            DIError::InvalidScopeTransition { from_scope, to_scope } => {
                write!(f, "Invalid scope transition from '{}' to '{}'", from_scope, to_scope)
            }
            DIError::ServiceAlreadyRegistered { service_type } => {
                write!(f, "Service '{}' is already registered", service_type)
            }
            DIError::ProviderCreationFailed { service_type, cause } => {
                write!(f, "Failed to create provider for '{}': {}", service_type, cause)
            }
        }
    }
}

impl std::error::Error for DIError {}

/// Result type alias for DI operations
pub type DIResult<T> = Result<T, DIError>;