/*!
# Meta-Testing Module

This module contains comprehensive meta-tests that validate the entire testing framework.
Meta-tests ensure that each testing component works correctly by testing the frameworks
themselves using self-hosting approaches.

## Test Categories

- **widget_testing_meta.rs**: Tests for the widget testing framework
- **mocking_meta.rs**: Tests for the mocking framework
- **property_meta.rs**: Tests for property-based testing
- **snapshot_meta.rs**: Tests for snapshot testing
- **integration_meta.rs**: Tests for integration testing framework
- **cross_framework_meta.rs**: Tests for framework interoperability
- **performance_meta.rs**: Performance validation tests
- **error_handling_meta.rs**: Error handling and recovery tests

## Purpose

These meta-tests provide confidence that the testing frameworks are reliable
and don't contain bugs that could lead to false positives or negatives in
actual project tests.
*/

pub mod widget_testing_meta;
pub mod mocking_meta;
pub mod property_meta;
pub mod snapshot_meta;
pub mod integration_meta;
pub mod cross_framework_meta;
pub mod performance_meta;
pub mod error_handling_meta;

// Re-export for convenience
pub use widget_testing_meta::*;
pub use mocking_meta::*;
pub use property_meta::*;
pub use snapshot_meta::*;
pub use integration_meta::*;
pub use cross_framework_meta::*;
pub use performance_meta::*;
pub use error_handling_meta::*;