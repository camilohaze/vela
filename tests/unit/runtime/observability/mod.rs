//! # Observability Tests Module
//!
//! This module contains comprehensive tests for the Vela observability system,
//! including tracing, metrics, exporters, and integration tests.
//!
//! ## Test Coverage
//!
//! - **Unit Tests**: Individual component testing
//! - **Integration Tests**: End-to-end observability workflows
//! - **Performance Tests**: Benchmarks and load testing
//! - **Concurrency Tests**: Multi-threaded scenarios
//!
//! ## Test Categories
//!
//! - `test_tracing.rs`: Distributed tracing functionality
//! - `test_metrics.rs`: Metrics collection and aggregation
//! - `test_exporters.rs`: Data export to monitoring backends
//! - `test_integration.rs`: Full-stack observability testing
//! - `test_performance.rs`: Performance benchmarks and profiling

pub mod test_tracing;
pub mod test_metrics;
pub mod test_exporters;
pub mod test_integration;
pub mod test_performance;