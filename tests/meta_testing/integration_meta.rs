/*!
# Meta-Tests for Integration Testing Framework

Tests that validate the integration testing framework itself works correctly.
These tests use integration testing to test itself (self-hosting).
*/

use vela_testing::integration::*;
use std::collections::HashMap;
use std::time::Duration;

/// Test that TestEnvironment creation works
#[test]
fn test_test_environment_creation() {
    let config = TestEnvironmentConfig {
        database_url: Some("postgresql://test:test@localhost/test".to_string()),
        services: {
            let mut services = HashMap::new();
            services.insert("api".to_string(), "http://localhost:8080".to_string());
            services.insert("db".to_string(), "http://localhost:5432".to_string());
            services
        },
        fixtures_path: Some("test_fixtures".to_string()),
        timeout: Duration::from_secs(30),
    };

    let env = TestEnvironment::new(config.clone());

    assert_eq!(env.config().database_url, config.database_url);
    assert_eq!(env.config().services.len(), 2);
    assert_eq!(env.services_health().len(), 2);
}

/// Test that TestEnvironment builder pattern works
#[test]
fn test_test_environment_builder() {
    let env = TestEnvironment::builder()
        .with_database("postgresql://test:test@localhost/test")
        .with_service("api", "http://localhost:8080")
        .with_service("auth", "http://localhost:8081")
        .with_fixtures_path("test_fixtures")
        .with_timeout(Duration::from_secs(60))
        .build();

    assert_eq!(env.config().database_url.as_ref().unwrap(), "postgresql://test:test@localhost/test");
    assert_eq!(env.config().services.get("api").unwrap(), "http://localhost:8080");
    assert_eq!(env.config().services.get("auth").unwrap(), "http://localhost:8081");
    assert_eq!(env.config().fixtures_path.as_ref().unwrap(), "test_fixtures");
    assert_eq!(env.config().timeout, Duration::from_secs(60));
}

/// Test service health checking
#[test]
fn test_service_health_checking() {
    let mut env = TestEnvironment::builder()
        .with_service("healthy", "http://httpbin.org/status/200")
        .with_service("unhealthy", "http://httpbin.org/status/500")
        .build();

    // Initially all services should be unhealthy (not checked yet)
    for health in env.services_health().values() {
        assert!(!health.healthy);
    }

    // Note: Actual health checking would require real HTTP calls
    // In a real test, we would mock the HTTP client
}

/// Test fixture loading
#[test]
fn test_fixture_loading() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fixtures_path = temp_dir.path().to_str().unwrap();

    // Create test fixture files
    std::fs::create_dir_all(fixtures_path).unwrap();
    std::fs::write(
        format!("{}/users.json", fixtures_path),
        r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#
    ).unwrap();

    let mut env = TestEnvironment::builder()
        .with_fixtures_path(fixtures_path)
        .build();

    // Load fixtures
    env.load_fixtures().unwrap();

    // Verify fixtures were loaded
    let users_fixture = env.fixtures().get("users").unwrap();
    let users: Vec<serde_json::Value> = serde_json::from_value(users_fixture.clone()).unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0]["name"], "Alice");
}

/// Test fixture modification
#[test]
fn test_fixture_modification() {
    let mut env = TestEnvironment::new(Default::default());

    // Add fixture data
    let mut user_data = HashMap::new();
    user_data.insert("id".to_string(), serde_json::json!(1));
    user_data.insert("name".to_string(), serde_json::json!("Test User"));

    env.fixtures_mut().insert("test_user".to_string(), serde_json::json!(user_data));

    // Verify fixture was added
    let fixture = env.fixtures().get("test_user").unwrap();
    assert_eq!(fixture["name"], "Test User");
}

/// Test DatabaseHelper creation and configuration
#[test]
fn test_database_helper_creation() {
    let db_url = "postgresql://test:test@localhost/test";
    let helper = DatabaseHelper::new(db_url);

    assert_eq!(helper.connection_string(), db_url);
    assert!(!helper.is_connected());
}

/// Test HTTP client extensions
#[test]
fn test_http_client_extensions() {
    let mut env = TestEnvironment::new(Default::default());

    // Test that HTTP client is available
    let client = env.http_client();
    assert!(client.is_some());

    // Note: Actual HTTP calls would be tested with mocks in real scenarios
}

/// Test parallel runner creation
#[test]
fn test_parallel_runner_creation() {
    let runner = ParallelRunner::new(4);

    assert_eq!(runner.max_concurrency(), 4);
    assert_eq!(runner.active_tasks(), 0);
}

/// Test parallel execution
#[test]
fn test_parallel_execution() {
    let runner = ParallelRunner::new(2);

    let mut env = TestEnvironment::new(Default::default());

    // Add some test tasks
    let task1 = runner.add_environment(env.clone());
    let task2 = runner.add_environment(env.clone());

    assert!(task1.is_ok());
    assert!(task2.is_ok());

    // Note: Actual parallel execution would be tested in integration tests
}

/// Test assertion helpers
#[test]
fn test_assertion_helpers() {
    // Test HTTP status assertions
    assert!(assert_status(200).check(&200));
    assert!(!assert_status(200).check(&404));

    // Test JSON path assertions
    let json = serde_json::json!({"user": {"name": "Alice", "age": 30}});
    assert!(assert_json_path("$.user.name", "Alice").check(&json));
    assert!(!assert_json_path("$.user.name", "Bob").check(&json));
}

/// Test cleanup functionality
#[test]
fn test_environment_cleanup() {
    let mut env = TestEnvironment::builder()
        .with_service("test", "http://localhost:9999")
        .build();

    // Add some fixtures
    env.fixtures_mut().insert("temp".to_string(), serde_json::json!({"data": "test"}));

    // Cleanup
    env.cleanup().unwrap();

    // Fixtures should be cleared
    assert!(env.fixtures().is_empty());

    // Services should be marked as unhealthy
    for health in env.services_health().values() {
        assert!(!health.healthy);
    }
}

/// Test configuration validation
#[test]
fn test_configuration_validation() {
    // Valid configuration
    let valid_config = TestEnvironmentConfig {
        database_url: Some("postgresql://user:pass@localhost/db".to_string()),
        services: HashMap::new(),
        fixtures_path: Some("/valid/path".to_string()),
        timeout: Duration::from_secs(30),
    };

    assert!(valid_config.validate().is_ok());

    // Invalid configuration
    let invalid_config = TestEnvironmentConfig {
        database_url: Some("invalid-url".to_string()),
        services: HashMap::new(),
        fixtures_path: Some("/valid/path".to_string()),
        timeout: Duration::from_secs(30),
    };

    assert!(invalid_config.validate().is_err());
}

/// Test environment serialization
#[test]
fn test_environment_serialization() {
    let config = TestEnvironmentConfig {
        database_url: Some("postgresql://test:test@localhost/test".to_string()),
        services: {
            let mut services = HashMap::new();
            services.insert("api".to_string(), "http://localhost:8080".to_string());
            services
        },
        fixtures_path: Some("fixtures".to_string()),
        timeout: Duration::from_secs(30),
    };

    let env = TestEnvironment::new(config);

    // Test serialization
    let serialized = env.to_json().unwrap();
    assert!(serialized.contains("database_url"));
    assert!(serialized.contains("services"));

    // Test deserialization
    let deserialized = TestEnvironment::from_json(&serialized).unwrap();
    assert_eq!(deserialized.config().database_url, env.config().database_url);
}

/// Test fixture builder pattern
#[test]
fn test_fixture_builder_pattern() {
    let mut env = TestEnvironment::new(Default::default());

    // Use fixture builder
    let user_fixture = FixtureBuilder::new("user")
        .with_field("id", 1)
        .with_field("name", "Alice")
        .with_field("email", "alice@example.com")
        .build();

    env.fixtures_mut().insert("test_user".to_string(), user_fixture);

    // Verify fixture
    let fixture = env.fixtures().get("test_user").unwrap();
    assert_eq!(fixture["id"], 1);
    assert_eq!(fixture["name"], "Alice");
    assert_eq!(fixture["email"], "alice@example.com");
}

/// Test service health monitoring
#[test]
fn test_service_health_monitoring() {
    let mut env = TestEnvironment::builder()
        .with_service("api", "http://localhost:8080")
        .with_service("db", "http://localhost:5432")
        .build();

    // Initially unhealthy
    assert!(!env.services_health()["api"].healthy);
    assert!(!env.services_health()["db"].healthy);

    // Simulate health check
    env.update_service_health("api", true).unwrap();

    assert!(env.services_health()["api"].healthy);
    assert!(!env.services_health()["db"].healthy);
}

/// Test timeout handling
#[test]
fn test_timeout_handling() {
    let config = TestEnvironmentConfig {
        database_url: None,
        services: HashMap::new(),
        fixtures_path: None,
        timeout: Duration::from_millis(100),
    };

    let env = TestEnvironment::new(config);

    assert_eq!(env.config().timeout, Duration::from_millis(100));
}

/// Test error handling
#[test]
fn test_error_handling() {
    let mut env = TestEnvironment::new(Default::default());

    // Test invalid fixture loading
    let result = env.load_fixtures_from_path("non_existent_path");
    assert!(result.is_err());

    // Test invalid service health update
    let result = env.update_service_health("non_existent_service", true);
    assert!(result.is_err());
}

/// Test performance under load
#[test]
fn test_performance_under_load() {
    let start = std::time::Instant::now();

    let mut env = TestEnvironment::builder()
        .with_service("service1", "http://localhost:8081")
        .with_service("service2", "http://localhost:8082")
        .with_service("service3", "http://localhost:8083")
        .build();

    // Add many fixtures
    for i in 0..100 {
        let fixture = serde_json::json!({
            "id": i,
            "data": format!("test_data_{}", i)
        });
        env.fixtures_mut().insert(format!("fixture_{}", i), fixture);
    }

    let setup_time = start.elapsed();

    // Should complete quickly
    assert!(setup_time.as_millis() < 500);

    // Verify fixtures were added
    assert_eq!(env.fixtures().len(), 100);
}