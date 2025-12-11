//! # Observability Integration Tests
//!
//! End-to-end tests for the complete observability stack.
//!
//! ## Test Scenarios
//!
//! - Full request tracing from HTTP to database
//! - Metrics collection across service boundaries
//! - Log aggregation and correlation
//! - Exporter pipeline validation
//! - Configuration loading and validation

pub mod test_full_stack;
pub mod test_prometheus_export;
pub mod test_jaeger_export;
pub mod test_grafana_integration;