/*!
# Cross-Framework Integration Tests

Tests that validate different testing frameworks can work together
and don't have conflicts when used in combination.
*/

use vela_testing::widget_testing::*;
use vela_testing::mocking::*;
use vela_testing::property::*;
use vela_testing::snapshot::*;
use vela_testing::integration::*;
use std::collections::HashMap;

/// Test using widget testing with mocking
#[test]
fn test_widget_testing_with_mocking() {
    // Create a mock service
    mock!(UserService {
        users: HashMap<String, String>,
    });

    let mut mock_service = UserService::new();
    mock_service.when().get_user("alice").returns(Some("Alice Cooper".to_string()));

    // Create a widget that uses the service
    let mut tester = WidgetTester::new();
    tester.add_widget("user_display", UserDisplayWidget {
        service: mock_service,
        username: "alice".to_string(),
    });

    // Simulate interaction
    tester.simulate_event("user_display", TestEvent::Refresh).unwrap();
    tester.process_events();

    // Verify the widget used the mock
    let widget = tester.get_widget("user_display").unwrap();
    assert!(widget.displayed_user().is_some());
    assert_eq!(widget.displayed_user().unwrap(), "Alice Cooper");
}

/// Test using property testing with snapshot testing
#[test]
fn test_property_testing_with_snapshots() {
    // Property: Generated UI trees should have consistent snapshots
    let result = property_test(|tree_config: UITreeConfig| {
        let tree = generate_ui_tree(tree_config);
        let snapshot1 = Snapshot::capture(&tree).unwrap();
        let snapshot2 = Snapshot::capture(&tree).unwrap();

        // Same tree should produce identical snapshots
        let diff = snapshot1.compare(&snapshot2).unwrap();
        !diff.has_changes()
    });

    match result {
        PropertyTestResult::Passed { .. } => {
            // Should pass - snapshots should be deterministic
        }
        PropertyTestResult::Failed { .. } => {
            panic!("Property testing with snapshots should have passed");
        }
    }
}

/// Test using integration testing with mocking
#[test]
fn test_integration_testing_with_mocking() {
    // Create mock services
    mock!(AuthService {
        tokens: HashMap<String, String>,
    });

    mock!(UserRepository {
        users: Vec<String>,
    });

    let mut mock_auth = AuthService::new();
    let mut mock_repo = UserRepository::new();

    // Configure mocks
    mock_auth.when().validate_token("valid_token").returns(true);
    mock_repo.when().get_user_by_id(1).returns(Some("Alice".to_string()));

    // Create integration test environment
    let mut env = TestEnvironment::builder()
        .with_service("auth", "http://localhost:8080")
        .with_service("api", "http://localhost:8081")
        .build();

    // Add mocks to environment
    env.add_mock("auth_service", mock_auth).unwrap();
    env.add_mock("user_repo", mock_repo).unwrap();

    // Verify mocks are accessible
    assert!(env.get_mock::<AuthService>("auth_service").is_some());
    assert!(env.get_mock::<UserRepository>("user_repo").is_some());
}

/// Test using all frameworks together
#[test]
fn test_complete_framework_integration() {
    // Setup integration environment
    let mut env = TestEnvironment::builder()
        .with_service("api", "http://localhost:8080")
        .with_database("postgresql://test:test@localhost/test")
        .build();

    // Create mocks for services
    mock!(APIService {
        endpoints: Vec<String>,
    });

    let mut mock_api = APIService::new();
    mock_api.when().get_users().returns(vec!["Alice", "Bob", "Charlie"]);

    env.add_mock("api_service", mock_api).unwrap();

    // Create widget testing setup
    let mut tester = WidgetTester::new();
    tester.add_widget("user_list", UserListWidget {
        api_service: env.get_mock::<APIService>("api_service").unwrap().clone(),
    });

    // Take initial snapshot
    let initial_snapshot = Snapshot::capture(&tester.widget_tree()).unwrap();

    // Simulate user interaction
    tester.simulate_event("user_list", TestEvent::LoadData).unwrap();
    tester.process_events();

    // Take snapshot after interaction
    let after_snapshot = Snapshot::capture(&tester.widget_tree()).unwrap();

    // Verify snapshots are different (UI changed)
    let diff = initial_snapshot.compare(&after_snapshot).unwrap();
    assert!(diff.has_changes());

    // Use property testing to verify UI consistency
    let property_result = property_test(|config: UIConfig| {
        let test_tree = generate_test_ui(config);
        let snapshot = Snapshot::capture(&test_tree).unwrap();

        // Property: All generated UIs should be valid JSON
        snapshot.validate().is_ok()
    });

    match property_result {
        PropertyTestResult::Passed { .. } => {
            // Integration successful
        }
        PropertyTestResult::Failed { .. } => {
            panic!("Complete framework integration test failed");
        }
    }
}

/// Test framework isolation (no conflicts)
#[test]
fn test_framework_isolation() {
    // Test that different frameworks don't interfere with each other

    // Widget testing
    let mut widget_tester = WidgetTester::new();
    widget_tester.add_widget("test", TestWidget { value: 42 });

    // Mocking
    mock!(TestMock {
        data: i32,
    });
    let mock = TestMock::new();

    // Property testing
    let property_result = property_test(|x: i32| x >= 0 || x < 0); // Always true

    // Snapshot testing
    let snapshot = Snapshot::from_data(serde_json::json!({"test": "data"}));

    // Integration testing
    let env = TestEnvironment::new(Default::default());

    // All should work without conflicts
    assert!(widget_tester.widgets().contains_key("test"));
    assert!(mock.is_mock());
    assert!(matches!(property_result, PropertyTestResult::Passed { .. }));
    assert!(snapshot.validate().is_ok());
    assert!(env.fixtures().is_empty());
}

/// Test framework cleanup and isolation
#[test]
fn test_framework_cleanup_isolation() {
    // Test that frameworks clean up properly and don't leak state

    // Create multiple instances
    let mut tester1 = WidgetTester::new();
    let mut tester2 = WidgetTester::new();

    tester1.add_widget("widget1", TestWidget { value: 1 });
    tester2.add_widget("widget2", TestWidget { value: 2 });

    // They should be independent
    assert_eq!(tester1.widgets().len(), 1);
    assert_eq!(tester2.widgets().len(), 1);
    assert_ne!(
        tester1.get_widget("widget1").unwrap().value(),
        tester2.get_widget("widget2").unwrap().value()
    );

    // Cleanup
    tester1.cleanup();
    tester2.cleanup();

    // Should be clean
    assert!(tester1.widgets().is_empty());
    assert!(tester2.widgets().is_empty());
}

/// Test concurrent framework usage
#[test]
fn test_concurrent_framework_usage() {
    use std::thread;
    use std::sync::Arc;
    use std::sync::Mutex;

    let results = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];

    // Spawn multiple threads using different frameworks
    for i in 0..5 {
        let results_clone = Arc::clone(&results);
        let handle = thread::spawn(move || {
            match i % 4 {
                0 => {
                    // Widget testing
                    let mut tester = WidgetTester::new();
                    tester.add_widget(&format!("widget_{}", i), TestWidget { value: i as i32 });
                    results_clone.lock().unwrap().push(format!("widget_{}", i));
                }
                1 => {
                    // Mocking
                    mock!(ThreadMock {
                        id: i32,
                    });
                    let mock = ThreadMock::new();
                    results_clone.lock().unwrap().push(format!("mock_{}", i));
                }
                2 => {
                    // Snapshot testing
                    let snapshot = Snapshot::from_data(serde_json::json!({
                        "thread": i,
                        "data": format!("test_{}", i)
                    }));
                    results_clone.lock().unwrap().push(format!("snapshot_{}", i));
                }
                3 => {
                    // Integration testing
                    let env = TestEnvironment::builder()
                        .with_service(&format!("service_{}", i), "http://localhost:8080")
                        .build();
                    results_clone.lock().unwrap().push(format!("env_{}", i));
                }
                _ => unreachable!(),
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all operations completed
    let final_results = results.lock().unwrap();
    assert_eq!(final_results.len(), 5);

    // Should have results from all framework types
    assert!(final_results.iter().any(|r| r.starts_with("widget_")));
    assert!(final_results.iter().any(|r| r.starts_with("mock_")));
    assert!(final_results.iter().any(|r| r.starts_with("snapshot_")));
    assert!(final_results.iter().any(|r| r.starts_with("env_")));
}

/// Test framework performance in combination
#[test]
fn test_combined_framework_performance() {
    let start = std::time::Instant::now();

    // Setup all frameworks
    let mut tester = WidgetTester::new();
    mock!(PerfMock { counter: i64 });
    let mock = PerfMock::new();
    let env = TestEnvironment::new(Default::default());

    // Perform operations with all frameworks
    for i in 0..100 {
        // Widget testing
        tester.add_widget(&format!("perf_widget_{}", i), TestWidget { value: i });

        // Mocking
        let _ = mock.is_mock();

        // Integration testing
        let _ = env.fixtures().is_empty();

        // Snapshot testing
        let snapshot = Snapshot::from_data(serde_json::json!({
            "iteration": i,
            "data": format!("perf_test_{}", i)
        }));
        let _ = snapshot.validate();
    }

    let duration = start.elapsed();

    // Should complete in reasonable time (less than 2 seconds for 100 iterations)
    assert!(duration.as_secs() < 2);

    // Cleanup
    tester.cleanup();
}

/// Test framework error propagation
#[test]
fn test_framework_error_propagation() {
    // Test that errors in one framework don't affect others

    // Widget testing with error
    let mut tester = WidgetTester::new();
    let result = tester.simulate_event("non_existent", TestEvent::Click);
    assert!(result.is_err());

    // Other frameworks should still work
    mock!(ErrorTestMock { data: String });
    let mock = ErrorTestMock::new();
    assert!(mock.is_mock());

    let snapshot = Snapshot::from_data(serde_json::json!({"test": "data"}));
    assert!(snapshot.validate().is_ok());

    let env = TestEnvironment::new(Default::default());
    assert!(env.fixtures().is_empty());
}

// Supporting types and functions

#[derive(Debug, Clone)]
struct TestWidget {
    value: i32,
}

impl Widget for TestWidget {
    fn id(&self) -> &str { "test" }
    fn widget_type(&self) -> &str { "Test" }
    fn text(&self) -> Option<&str> { None }
    fn is_enabled(&self) -> bool { true }
    fn children(&self) -> Vec<&str> { vec![] }
}

impl TestWidget {
    fn value(&self) -> i32 { self.value }
}

#[derive(Debug, Clone)]
struct UserDisplayWidget {
    service: UserService,
    username: String,
}

impl Widget for UserDisplayWidget {
    fn id(&self) -> &str { "user_display" }
    fn widget_type(&self) -> &str { "UserDisplay" }
    fn text(&self) -> Option<&str> { self.displayed_user().as_deref() }
    fn is_enabled(&self) -> bool { true }
    fn children(&self) -> Vec<&str> { vec![] }
}

impl UserDisplayWidget {
    fn displayed_user(&self) -> Option<String> {
        // In real implementation, this would call the service
        None // Simplified for test
    }
}

#[derive(Debug, Clone)]
struct UserListWidget {
    api_service: APIService,
}

impl Widget for UserListWidget {
    fn id(&self) -> &str { "user_list" }
    fn widget_type(&self) -> &str { "UserList" }
    fn text(&self) -> Option<&str> { None }
    fn is_enabled(&self) -> bool { true }
    fn children(&self) -> Vec<&str> { vec![] }
}

#[derive(Debug, Clone)]
struct UITreeConfig {
    depth: u8,
    branching_factor: u8,
}

#[derive(Debug, Clone)]
struct UIConfig {
    width: u32,
    height: u32,
}

fn generate_ui_tree(_config: UITreeConfig) -> serde_json::Value {
    serde_json::json!({
        "type": "Container",
        "children": [
            {"type": "Button", "text": "OK"},
            {"type": "TextInput", "placeholder": "Enter text"}
        ]
    })
}

fn generate_test_ui(_config: UIConfig) -> serde_json::Value {
    serde_json::json!({
        "type": "Container",
        "width": 800,
        "height": 600,
        "children": []
    })
}