/*!
# Mock Framework

Framework for mocking services and dependencies in tests.

## Example

```rust,no_run
use vela_testing::mock::*;

// Create mock service
let mut mock_service = MockService::new();
mock_service.expect_get_user().returning(User { id: 1, name: "Test" });

// Use in test
let result = mock_service.get_user(1).await;
assert_eq!(result.name, "Test");
```

*/

use std::collections::HashMap;

/// Mock service for testing
pub struct MockService {
    expectations: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl MockService {
    pub fn new() -> Self {
        Self {
            expectations: HashMap::new(),
        }
    }

    /// Set expectation for method
    pub fn expect_get_user(&mut self) -> MockExpectation {
        MockExpectation::new("get_user", &mut self.expectations)
    }

    /// Mock method implementation
    pub async fn get_user(&self, id: u32) -> User {
        if let Some(expectation) = self.expectations.get("get_user") {
            if let Some(user) = expectation.downcast_ref::<User>() {
                return user.clone();
            }
        }
        // Default behavior
        User { id, name: "Default".to_string() }
    }
}

/// Mock expectation builder
pub struct MockExpectation {
    method: String,
    expectations: *mut HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl MockExpectation {
    pub fn new(method: &str, expectations: *mut HashMap<String, Box<dyn std::any::Any + Send + Sync>>) -> Self {
        Self {
            method: method.to_string(),
            expectations,
        }
    }

    pub fn returning<T>(self, value: T) where T: 'static + Send + Sync {
        unsafe {
            (*self.expectations).insert(self.method, Box::new(value));
        }
    }
}

/// Test user struct
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: u32,
    pub name: String,
}