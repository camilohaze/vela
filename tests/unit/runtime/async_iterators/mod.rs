//! Test suite completa para async iterators
//!
//! Esta suite incluye tests exhaustivos de:
//! - Correctness: Validación funcional de todos los operadores
//! - Performance: Benchmarks de throughput y latency
//! - Stress: Tests bajo alta carga y escenarios extremos
//! - Integration: Pipelines completos y escenarios del mundo real

pub mod correctness_tests;
pub mod performance_tests;
pub mod stress_tests;
pub mod integration_tests;

// Re-export para facilitar el acceso
pub use correctness_tests::*;
pub use performance_tests::*;
pub use stress_tests::*;
pub use integration_tests::*;