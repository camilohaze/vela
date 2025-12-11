/*!
Error types for Vela ORM.

This module defines the error types used throughout the ORM,
providing consistent error handling and reporting.
*/

use std::fmt;

/// Result type alias for ORM operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Vela ORM
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Database connection errors
    #[error("Connection error: {message}")]
    Connection { message: String },

    /// Query execution errors
    #[error("Query error: {message}")]
    Query { message: String },

    /// Entity mapping errors
    #[error("Entity error: {message}")]
    Entity { message: String },

    /// Migration errors
    #[error("Migration error: {message}")]
    Migration { message: String },

    /// Transaction errors
    #[error("Transaction error: {message}")]
    Transaction { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Driver-specific errors
    #[error("Driver error: {message}")]
    Driver { message: String },

    /// Pool errors
    #[error("Pool error: {message}")]
    Pool { message: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// Generic errors
    #[error("ORM error: {message}")]
    Other { message: String },
}

impl Error {
    /// Create a connection error
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Create a query error
    pub fn query<S: Into<String>>(message: S) -> Self {
        Self::Query {
            message: message.into(),
        }
    }

    /// Create an entity error
    pub fn entity<S: Into<String>>(message: S) -> Self {
        Self::Entity {
            message: message.into(),
        }
    }

    /// Create a migration error
    pub fn migration<S: Into<String>>(message: S) -> Self {
        Self::Migration {
            message: message.into(),
        }
    }

    /// Create a transaction error
    pub fn transaction<S: Into<String>>(message: S) -> Self {
        Self::Transaction {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a driver error
    pub fn driver<S: Into<String>>(message: S) -> Self {
        Self::Driver {
            message: message.into(),
        }
    }

    /// Create a pool error
    pub fn pool<S: Into<String>>(message: S) -> Self {
        Self::Pool {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: message.into(),
        }
    }

    /// Check if this is a connection error
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Connection { .. })
    }

    /// Check if this is a query error
    pub fn is_query(&self) -> bool {
        matches!(self, Self::Query { .. })
    }

    /// Check if this is an entity error
    pub fn is_entity(&self) -> bool {
        matches!(self, Self::Entity { .. })
    }

    /// Check if this is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation { .. })
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Other {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "postgres")]
impl From<tokio_postgres::Error> for Error {
    fn from(err: tokio_postgres::Error) -> Self {
        Self::Driver {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "mysql")]
impl From<mysql_async::Error> for Error {
    fn from(err: mysql_async::Error) -> Self {
        Self::Driver {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Self::Driver {
            message: err.to_string(),
        }
    }
}

/// Validation error with field information
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Validation rule that was violated
    pub rule: String,
    /// Error message
    pub message: String,
    /// Actual value that failed validation
    pub value: Option<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Validation failed for field '{}': {} (rule: {})",
            self.field, self.message, self.rule
        )
    }
}

/// Collection of validation errors
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Create a new validation errors collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a validation error
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Check if there are any errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Get all errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Convert to ORM error
    pub fn into_orm_error(self) -> Error {
        let messages: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();
        Error::validation(messages.join("; "))
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let messages: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();
        write!(f, "Validation errors: {}", messages.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::connection("Failed to connect");
        assert!(err.is_connection());
        assert!(!err.is_query());

        let err = Error::query("Invalid SQL");
        assert!(err.is_query());
        assert!(!err.is_connection());
    }

    #[test]
    fn test_validation_errors() {
        let mut errors = ValidationErrors::new();

        errors.add(ValidationError {
            field: "email".to_string(),
            rule: "email".to_string(),
            message: "Invalid email format".to_string(),
            value: Some("invalid-email".to_string()),
        });

        assert_eq!(errors.len(), 1);
        assert!(!errors.is_empty());

        let orm_error = errors.into_orm_error();
        assert!(orm_error.is_validation());
    }
}