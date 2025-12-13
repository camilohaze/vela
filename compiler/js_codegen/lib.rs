//! JavaScript Code Generator for Vela
//!
//! This module provides functionality to generate JavaScript code from Vela's
//! Intermediate Representation (IR). It includes:
//!
//! - Code generation from IR modules and functions
//! - Type mapping from Vela types to JavaScript types
//! - Runtime support for Vela constructs (signals, Option, Result)
//! - DOM rendering for UI widgets
//! - Expression and statement generation
//!
//! The generated JavaScript code requires the vela-runtime.js file for
//! runtime support of Vela-specific features.

pub mod codegen;
pub mod dom_renderer;
pub mod dom_renderer_tests;
pub mod expressions;
pub mod generator;
pub mod runtime;
pub mod statements;
pub mod types;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod reactive_tests;
#[cfg(test)]
mod backend_tests;
#[cfg(test)]
mod codegen_correctness_tests;
#[cfg(test)]
mod runtime_integration_tests;
#[cfg(test)]
mod end_to_end_tests;

pub use self::generator::JSGenerator;
pub use self::runtime::generate_runtime_file as generate_runtime;