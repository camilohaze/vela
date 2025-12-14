/*!
# Native Backend Tests for Vela Compiler

This module provides comprehensive testing for the LLVM native backend,
including correctness, performance, and integration tests.
*/

#[cfg(feature = "llvm_backend")]
mod correctness;
#[cfg(feature = "llvm_backend")]
mod performance;
#[cfg(feature = "llvm_backend")]
mod edge_cases;
#[cfg(feature = "llvm_backend")]
mod integration;

mod utils;

#[cfg(feature = "llvm_backend")]
pub use correctness::*;
#[cfg(feature = "llvm_backend")]
pub use performance::*;
#[cfg(feature = "llvm_backend")]
pub use edge_cases::*;
#[cfg(feature = "llvm_backend")]
pub use integration::*;
pub use utils::*;