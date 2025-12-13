/*!
# Integration Testing

Framework for integration testing of multiple components.

## Example

```rust,no_run
use vela_testing::integration::*;

// Create integration test
let mut test = IntegrationTest::new();
test.add_service("auth", auth_service);
test.add_service("user", user_service);

// Run integration scenario
test.run_scenario("user_registration").await;
```

*/

use std::collections::HashMap;

/// Integration test runner
pub struct IntegrationTest {
    services: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    scenarios: HashMap<String, Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync + 'static>> + Send + Sync>>,
}

impl IntegrationTest {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            scenarios: HashMap::new(),
        }
    }

    /// Add service to test environment
    pub fn add_service<T: 'static + Send + Sync>(&mut self, name: &str, service: T) {
        self.services.insert(name.to_string(), Box::new(service));
    }

    /// Get service from test environment
    pub fn get_service<T: 'static>(&self, name: &str) -> Option<&T> {
        self.services.get(name)?
            .downcast_ref::<T>()
    }

    /// Add test scenario
    pub fn add_scenario<F, Fut>(&mut self, name: &str, scenario: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + Sync + 'static,
    {
        self.scenarios.insert(name.to_string(), Box::new(move || Box::pin(scenario())));
    }

    /// Run test scenario
    pub async fn run_scenario(&self, name: &str) -> Result<(), String> {
        if let Some(scenario) = self.scenarios.get(name) {
            scenario().await;
            Ok(())
        } else {
            Err(format!("Scenario '{}' not found", name))
        }
    }

    /// Setup test environment
    pub async fn setup(&mut self) {
        // Setup logic for integration test
    }

    /// Cleanup test environment
    pub async fn cleanup(&mut self) {
        // Cleanup logic for integration test
    }
}

/// Integration test utilities
pub mod utils {
    use super::*;

    /// Setup database for integration tests
    pub async fn setup_database() -> Result<(), String> {
        // Database setup logic
        Ok(())
    }

    /// Setup external services
    pub async fn setup_external_services() -> Result<(), String> {
        // External services setup logic
        Ok(())
    }

    /// Wait for services to be ready
    pub async fn wait_for_services(timeout_secs: u64) -> Result<(), String> {
        // Wait logic
        tokio::time::sleep(std::time::Duration::from_secs(timeout_secs)).await;
        Ok(())
    }
}