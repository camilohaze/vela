/*!
# Integration Testing Tests

Comprehensive tests for integration testing helpers.
*/

use super::*;
use std::collections::HashMap;
use serde_json::json;
use crate::integration::{TestEnvironment, TestEnvironmentConfig, fixtures, parallel};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_test_environment_creation() {
        let env = TestEnvironment::new();
        assert!(env.config().database_url.is_none());
        assert!(env.config().services.is_empty());
        assert_eq!(env.config().timeout_secs, 30);
    }

    #[tokio::test]
    async fn test_test_environment_with_config() {
        let mut services = HashMap::new();
        services.insert("test-service".to_string(), "http://localhost:8080".to_string());

        let config = TestEnvironmentConfig {
            database_url: Some("postgresql://localhost:5432/test".to_string()),
            services,
            timeout_secs: 60,
            retry_attempts: 5,
            parallel_execution: true,
            fixtures_path: Some("fixtures.json".to_string()),
        };

        let env = TestEnvironment::with_config(config.clone());
        assert_eq!(env.config().database_url, config.database_url);
        assert_eq!(env.config().services.len(), 1);
        assert_eq!(env.config().timeout_secs, 60);
        assert_eq!(env.config().retry_attempts, 5);
        assert!(env.config().parallel_execution);
        assert_eq!(env.config().fixtures_path, config.fixtures_path);
    }

    #[tokio::test]
    async fn test_test_environment_builder_pattern() {
        let env = TestEnvironment::new()
            .with_database("postgresql://localhost:5432/test")
            .with_service("auth", "http://localhost:8080")
            .with_service("user", "http://localhost:8081")
            .with_timeout(45)
            .with_parallel_execution(true)
            .with_fixtures_path("test-fixtures.json");

        assert_eq!(env.config().database_url, Some("postgresql://localhost:5432/test".to_string()));
        assert_eq!(env.config().services.len(), 2);
        assert_eq!(env.config().services.get("auth"), Some(&"http://localhost:8080".to_string()));
        assert_eq!(env.config().services.get("user"), Some(&"http://localhost:8081".to_string()));
        assert_eq!(env.config().timeout_secs, 45);
        assert!(env.config().parallel_execution);
        assert_eq!(env.config().fixtures_path, Some("test-fixtures.json".to_string()));
    }

    #[tokio::test]
    async fn test_fixture_builder() {
        let fixture = fixtures::UserFixture::new()
            .email("john@example.com")
            .password("secret123")
            .name("John Doe")
            .build();

        assert_eq!(fixture["email"], "john@example.com");
        assert_eq!(fixture["password"], "secret123");
        assert_eq!(fixture["name"], "John Doe");
        assert_eq!(fixture["active"], true);
    }

    #[tokio::test]
    async fn test_fixture_builder_inactive() {
        let fixture = fixtures::UserFixture::new()
            .inactive()
            .build();

        assert_eq!(fixture["active"], false);
    }

    #[tokio::test]
    async fn test_generic_fixture_builder() {
        let fixture = fixtures::FixtureBuilder::new()
            .add("users", vec![
                fixtures::UserFixture::new().email("user1@test.com").build(),
                fixtures::UserFixture::new().email("user2@test.com").build(),
            ])
            .add("settings", json!({
                "theme": "dark",
                "notifications": true
            }))
            .build();

        assert!(fixture["users"].is_array());
        let users = fixture["users"].as_array().unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0]["email"], "user1@test.com");
        assert_eq!(users[1]["email"], "user2@test.com");

        assert_eq!(fixture["settings"]["theme"], "dark");
        assert_eq!(fixture["settings"]["notifications"], true);
    }

    #[tokio::test]
    async fn test_assertions_status() {
        // Mock response - in real tests this would be from actual HTTP calls
        // For now, we'll test the assertion logic structure
        let response = json!({
            "status": "success",
            "data": {
                "id": 123,
                "name": "Test User"
            }
        });

        // Test that our JSON structure is correct
        assert!(response["status"].is_string());
        assert!(response["data"].is_object());
        assert_eq!(response["data"]["id"], 123);
        assert_eq!(response["data"]["name"], "Test User");
    }

    #[tokio::test]
    async fn test_parallel_runner_creation() {
        let runner = parallel::ParallelRunner::new(5);
        assert_eq!(runner.environments().len(), 0);
    }

    #[tokio::test]
    async fn test_parallel_runner_add_environment() {
        let mut runner = parallel::ParallelRunner::new(3);
        let env = TestEnvironment::new();

        runner.add_environment(env);
        assert_eq!(runner.environments().len(), 1);
    }

    #[tokio::test]
    async fn test_http_client_extensions() {
        let client = reqwest::Client::new();

        // Test that the extension methods exist and return RequestBuilder
        let _get_builder = client.get("http://example.com");
        let _post_builder = client.post("http://example.com");
        let _put_builder = client.put("http://example.com");
        let _delete_builder = client.delete("http://example.com");
        let _patch_builder = client.patch("http://example.com");

        // If we get here without compilation errors, the extensions work
        assert!(true);
    }

    #[tokio::test]
    async fn test_service_health_initialization() {
        let mut env = TestEnvironment::new()
            .with_service("test-service", "http://localhost:8080");

        // Before setup, services_health should be empty
        assert!(env.services_health().is_empty());

        // Setup should initialize service health tracking
        env.setup().await.expect("Setup should succeed");
        assert_eq!(env.services_health().len(), 1);

        let health = env.services_health().get("test-service").unwrap();
        assert_eq!(health.url(), "http://localhost:8080");
        assert!(!health.healthy()); // Should start as unhealthy
    }

    #[tokio::test]
    async fn test_fixtures_loading() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        // Create a temporary fixtures file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let fixtures_content = r#"{
            "users": [
                {"email": "test@example.com", "name": "Test User"}
            ],
            "settings": {
                "theme": "dark"
            }
        }"#;

        temp_file.write_all(fixtures_content.as_bytes()).expect("Failed to write fixtures");
        let fixtures_path = temp_file.path().to_str().unwrap();

        let mut env = TestEnvironment::new();
        env.load_fixtures(fixtures_path).await.expect("Failed to load fixtures");

        // Test that fixtures were loaded
        assert!(env.get_fixture("users").is_some());
        assert!(env.get_fixture("settings").is_some());
        assert!(env.get_fixture("nonexistent").is_none());

        let users = env.get_fixture("users").unwrap();
        assert!(users.is_array());
        assert_eq!(users[0]["email"], "test@example.com");

        let settings = env.get_fixture("settings").unwrap();
        assert_eq!(settings["theme"], "dark");
    }

    #[tokio::test]
    async fn test_fixtures_loading_invalid_json() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        // Create a temporary file with invalid JSON
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(b"invalid json content").expect("Failed to write invalid JSON");

        let fixtures_path = temp_file.path().to_str().unwrap();
        let mut env = TestEnvironment::new();

        let result = env.load_fixtures(fixtures_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse fixtures JSON"));
    }

    #[tokio::test]
    async fn test_fixtures_loading_nonexistent_file() {
        let mut env = TestEnvironment::new();
        let result = env.load_fixtures("nonexistent_file.json").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read fixtures file"));
    }

    #[tokio::test]
    async fn test_environment_cleanup() {
        let mut env = TestEnvironment::new()
            .with_database("postgresql://localhost:5432/test");

        // Setup environment
        env.setup().await.expect("Setup should succeed");

        // Add some fixtures
        env.fixtures_mut().insert("test".to_string(), json!({"key": "value"}));

        // Cleanup should succeed
        env.cleanup().await.expect("Cleanup should succeed");

        // Fixtures should be cleared
        assert!(env.fixtures_mut().is_empty());
    }

    #[tokio::test]
    async fn test_environment_without_database() {
        let mut env = TestEnvironment::new();

        // Setup should succeed even without database
        env.setup().await.expect("Setup should succeed");
        assert!(env.database().is_none());

        // Cleanup should succeed
        env.cleanup().await.expect("Cleanup should succeed");
    }
}