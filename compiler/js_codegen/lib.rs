//! JavaScript Code Generator for Vela
//!
//! This module provides functionality to generate JavaScript code from Vela's
//! Intermediate Representation (IR). It includes:
//!
//! - Code generation from IR modules and functions
//! - Type mapping from Vela types to JavaScript types
//! - Runtime support for Vela constructs (signals, Option, Result)
//! - Expression and statement generation
//!
//! The generated JavaScript code requires the vela-runtime.js file for
//! runtime support of Vela-specific features.

pub mod codegen;
pub mod expressions;
pub mod runtime;
pub mod statements;
pub mod types;

#[cfg(test)]
mod tests;

pub use codegen::JSGenerator;
pub use runtime::generate_runtime;
pub use types::JSTypeMapper;