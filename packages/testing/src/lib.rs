/*!
# Vela Testing Framework

Advanced testing framework for Vela applications providing:
- Widget testing for UI components
- Snapshot testing for visual regression
- Mocking framework for services
- Property-based testing
- Integration testing helpers

## Example

```rust,no_run
use vela_testing::*;

#[tokio::test]
async fn test_counter_widget() {
    // Create test app
    let mut app = TestApp::new();

    // Mount counter widget
    app.mount(CounterWidget::new()).await;

    // Verify initial state
    app.expect(find::by_text("Count: 0")).await;

    // Simulate user interaction
    app.tap(find::by_key("increment")).await;

    // Verify reactive update
    app.expect(find::by_text("Count: 1")).await;
}
```
*/

pub mod widget_testing;
pub mod matchers;
pub mod finders;
pub mod interactions;
pub mod snapshot;
pub mod mock;
pub mod mocking;
pub mod property;
pub mod integration;
#[cfg(test)]
mod widget_testing_tests;
#[cfg(test)]
mod mocking_tests;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod integration_tests;

// Note: widget_testing module is UI-agnostic and provides the core testing infrastructure
// UI-specific integrations should be implemented in the vela-ui package
pub use widget_testing::*;
pub use matchers::*;
pub use finders::*;
pub use interactions::*;
pub use mocking::*;
pub use property::*;