//! # Vela Validation
//!
//! Declarative validation framework for Vela with support for
//! custom validators, schema validation, and error aggregation.

pub mod error;
pub mod errors;
pub mod validators;
pub mod decorator;
pub mod schema;
pub mod integration;

#[cfg(test)]
pub mod integration_tests;

/// Re-export commonly used types
pub use error::ValidationError;
pub use errors::ValidationErrors;
pub use error::ValidationResult;
pub use decorator::ValidationDecorator;
pub use schema::Schema;