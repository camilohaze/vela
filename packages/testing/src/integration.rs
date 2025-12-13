/*!
# Integration Testing Helpers

Comprehensive framework for integration testing of Vela applications.

## Features

- Database setup and teardown helpers
- HTTP client testing utilities
- Service health checks and waiting
- Test fixtures and data seeding
- Configuration management for test environments
- Parallel test execution support
- Comprehensive assertion helpers

## Example

```rust,no_run
use vela_testing::integration::*;

#[tokio::test]
async fn test_user_registration_flow() {
    // Setup test environment
    let mut env = TestEnvironment::new()
        .with_database("postgresql://localhost:5432/testdb")
        .with_service("auth-service", "http://localhost:8080")
        .with_service("user-service", "http://localhost:8081");

    // Initialize environment
    env.setup().await.expect("Failed to setup test environment");

    // Wait for services to be ready
    env.wait_for_services(30).await.expect("Services not ready");

    // Create HTTP client for testing
    let client = env.http_client();

    // Test user registration API
    let response = client
        .post("/api/users")
        .json(&serde_json::json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register user");

    assert_eq!(response.status(), 201);

    // Verify user was created in database
    let user_count = env.query_count("SELECT COUNT(*) FROM users WHERE email = $1", &["test@example.com"])
        .await
        .expect("Failed to query database");

    assert_eq!(user_count, 1);

    // Cleanup
    env.cleanup().await.expect("Failed to cleanup");
}
```

*/

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

/// Configuration for test environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironmentConfig {
    pub database_url: Option<String>,
    pub services: HashMap<String, String>,
    pub timeout_secs: u64,
    pub retry_attempts: u32,
    pub parallel_execution: bool,
    pub fixtures_path: Option<String>,
}

impl Default for TestEnvironmentConfig {
    fn default() -> Self {
        Self {
            database_url: None,
            services: HashMap::new(),
            timeout_secs: 30,
            retry_attempts: 3,
            parallel_execution: false,
            fixtures_path: None,
        }
    }
}

/// Test environment for integration testing
pub struct TestEnvironment {
    config: TestEnvironmentConfig,
    database: Option<DatabaseHelper>,
    http_client: reqwest::Client,
    services_health: HashMap<String, ServiceHealth>,
    fixtures: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub url: String,
    pub healthy: bool,
    pub last_check: std::time::Instant,
}

impl ServiceHealth {
    // Getters for testing
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn healthy(&self) -> bool {
        self.healthy
    }
}

impl TestEnvironment {
    /// Create new test environment with default config
    pub fn new() -> Self {
        Self::with_config(TestEnvironmentConfig::default())
    }

    /// Create test environment with custom config
    pub fn with_config(config: TestEnvironmentConfig) -> Self {
        Self {
            config,
            database: None,
            http_client: reqwest::Client::new(),
            services_health: HashMap::new(),
            fixtures: HashMap::new(),
        }
    }

    /// Configure database connection
    pub fn with_database(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = Some(url.into());
        self
    }

    /// Add service to test environment
    pub fn with_service(mut self, name: impl Into<String>, url: impl Into<String>) -> Self {
        self.config.services.insert(name.into(), url.into());
        self
    }

    /// Set timeout for operations
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.config.timeout_secs = secs;
        self
    }

    /// Enable parallel execution
    pub fn with_parallel_execution(mut self, enabled: bool) -> Self {
        self.config.parallel_execution = enabled;
        self
    }

    /// Set fixtures path
    pub fn with_fixtures_path(mut self, path: impl Into<String>) -> Self {
        self.config.fixtures_path = Some(path.into());
        self
    }

    /// Setup test environment
    pub async fn setup(&mut self) -> Result<(), String> {
        // Initialize database if configured
        if let Some(url) = self.config.database_url.clone() {
            self.database = Some(DatabaseHelper::new(url).await?);
        }

        // Initialize service health tracking
        for (name, url) in &self.config.services {
            self.services_health.insert(name.clone(), ServiceHealth {
                url: url.clone(),
                healthy: false,
                last_check: std::time::Instant::now(),
            });
        }

        // Load fixtures if path is configured
        if let Some(path) = self.config.fixtures_path.clone() {
            self.load_fixtures(&path).await?;
        }

        Ok(())
    }

    /// Cleanup test environment
    pub async fn cleanup(&mut self) -> Result<(), String> {
        // Cleanup database
        if let Some(db) = &mut self.database {
            db.cleanup().await?;
        }

        // Clear fixtures
        self.fixtures.clear();

        Ok(())
    }

    /// Wait for all services to be healthy
    pub async fn wait_for_services(&mut self, timeout_secs: u64) -> Result<(), String> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            let mut all_healthy = true;
            let service_urls: Vec<(String, String)> = self.services_health.iter()
                .map(|(name, health)| (name.clone(), health.url.clone()))
                .collect();

            for (name, url) in service_urls {
                let needs_check = if let Some(health) = self.services_health.get(&name) {
                    !health.healthy
                } else {
                    false
                };

                if needs_check {
                    match self.check_service_health(&url).await {
                        Ok(is_healthy) => {
                            if let Some(health) = self.services_health.get_mut(&name) {
                                health.healthy = is_healthy;
                                health.last_check = std::time::Instant::now();
                            }
                        }
                        Err(_) => {
                            all_healthy = false;
                        }
                    }
                }
            }

            if all_healthy {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err("Timeout waiting for services to be healthy".to_string())
    }

    /// Get HTTP client for making requests
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Get database helper
    pub fn database(&self) -> Option<&DatabaseHelper> {
        self.database.as_ref()
    }

    /// Get database helper mutably
    pub fn database_mut(&mut self) -> Option<&mut DatabaseHelper> {
        self.database.as_mut()
    }

    /// Load test fixtures from file
    pub async fn load_fixtures(&mut self, path: &str) -> Result<(), String> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read fixtures file: {}", e))?;

        let fixtures: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse fixtures JSON: {}", e))?;

        self.fixtures = fixtures;
        Ok(())
    }

    /// Get fixture by name
    pub fn get_fixture(&self, name: &str) -> Option<&serde_json::Value> {
        self.fixtures.get(name)
    }

    /// Seed database with fixture data
    pub async fn seed_database(&mut self, fixture_name: &str) -> Result<(), String> {
        let db = self.database.as_mut()
            .ok_or("Database not configured")?;

        let fixture = self.fixtures.get(fixture_name)
            .ok_or(format!("Fixture '{}' not found", fixture_name))?
            .clone();

        db.seed_data(&fixture).await
    }

    /// Check if service is healthy
    async fn check_service_health(&self, url: &str) -> Result<bool, String> {
        match self.http_client
            .get(&format!("{}/health", url.trim_end_matches('/')))
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false), // Consider service unhealthy if request fails
        }
    }

    // Getters for testing
    pub fn config(&self) -> &TestEnvironmentConfig {
        &self.config
    }

    pub fn services_health(&self) -> &HashMap<String, ServiceHealth> {
        &self.services_health
    }

    pub fn fixtures_mut(&mut self) -> &mut HashMap<String, serde_json::Value> {
        &mut self.fixtures
    }
}

/// Database helper for integration tests
pub struct DatabaseHelper {
    client: tokio_postgres::Client,
}

impl DatabaseHelper {
    /// Create new database helper
    pub async fn new(url: String) -> Result<Self, String> {
        let (client, connection) = tokio_postgres::connect(&url, tokio_postgres::tls::NoTls)
            .await
            .map_err(|e| format!("Failed to connect to database: {}", e))?;

        // Spawn connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        Ok(Self { client })
    }

    /// Execute query and return count
    pub async fn query_count(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<i64, String> {
        let row = self.client.query_one(query, params)
            .await
            .map_err(|e| format!("Database query failed: {}", e))?;

        row.try_get(0)
            .map_err(|e| format!("Failed to get count from result: {}", e))
    }

    /// Execute query and return rows
    pub async fn query(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<tokio_postgres::Row>, String> {
        self.client.query(query, params)
            .await
            .map_err(|e| format!("Database query failed: {}", e))
    }

    /// Execute statement
    pub async fn execute(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64, String> {
        self.client.execute(query, params)
            .await
            .map_err(|e| format!("Database execute failed: {}", e))
    }

    /// Seed database with test data
    pub async fn seed_data(&self, data: &serde_json::Value) -> Result<(), String> {
        if let Some(tables) = data.as_object() {
            for (table_name, rows) in tables {
                if let Some(rows_array) = rows.as_array() {
                    for row in rows_array {
                        self.insert_row(table_name, row).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Insert a single row into table
    async fn insert_row(&self, table: &str, row: &serde_json::Value) -> Result<(), String> {
        if let Some(obj) = row.as_object() {
            let columns: Vec<String> = obj.keys().cloned().collect();
            let values: Vec<String> = obj.values()
                .map(|v| match v {
                    serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => "NULL".to_string(),
                })
                .collect();

            let query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                columns.join(", "),
                values.join(", ")
            );

            self.execute(&query, &[]).await?;
        }
        Ok(())
    }

    /// Cleanup database (truncate tables)
    pub async fn cleanup(&self) -> Result<(), String> {
        // Get all table names (simplified - in real implementation you'd query information_schema)
        let tables = vec!["users", "posts", "comments"]; // Example tables

        for table in tables {
            self.execute(&format!("TRUNCATE TABLE {} CASCADE", table), &[]).await?;
        }

        Ok(())
    }
}

/// HTTP client extensions for testing
pub trait HttpTestExt {
    fn get(&self, url: &str) -> reqwest::RequestBuilder;
    fn post(&self, url: &str) -> reqwest::RequestBuilder;
    fn put(&self, url: &str) -> reqwest::RequestBuilder;
    fn delete(&self, url: &str) -> reqwest::RequestBuilder;
    fn patch(&self, url: &str) -> reqwest::RequestBuilder;
}

impl HttpTestExt for reqwest::Client {
    fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.get(url)
    }

    fn post(&self, url: &str) -> reqwest::RequestBuilder {
        self.post(url)
    }

    fn put(&self, url: &str) -> reqwest::RequestBuilder {
        self.put(url)
    }

    fn delete(&self, url: &str) -> reqwest::RequestBuilder {
        self.delete(url)
    }

    fn patch(&self, url: &str) -> reqwest::RequestBuilder {
        self.patch(url)
    }
}

/// Assertion helpers for integration tests
pub mod assertions {
    use super::*;

    /// Assert that HTTP response has expected status
    pub fn assert_status(response: &reqwest::Response, expected: u16) -> Result<(), String> {
        let actual = response.status().as_u16();
        if actual == expected {
            Ok(())
        } else {
            Err(format!("Expected status {}, got {}", expected, actual))
        }
    }

    /// Assert that HTTP response is successful (2xx)
    pub fn assert_success(response: &reqwest::Response) -> Result<(), String> {
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Expected success status, got {}", response.status()))
        }
    }

    /// Assert that JSON response contains expected field
    pub async fn assert_json_contains(response: reqwest::Response, field: &str, expected: &serde_json::Value) -> Result<(), String> {
        let json: serde_json::Value = response.json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        if let Some(actual) = json.get(field) {
            if actual == expected {
                Ok(())
            } else {
                Err(format!("Field '{}' has value {:?}, expected {:?}", field, actual, expected))
            }
        } else {
            Err(format!("Field '{}' not found in response", field))
        }
    }

    /// Assert that database has expected row count
    pub async fn assert_row_count(db: &DatabaseHelper, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)], expected: i64) -> Result<(), String> {
        let actual = db.query_count(query, params).await?;
        if actual == expected {
            Ok(())
        } else {
            Err(format!("Expected {} rows, got {}", expected, actual))
        }
    }
}

/// Test fixtures and data builders
pub mod fixtures {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// User fixture builder
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserFixture {
        pub email: String,
        pub password: String,
        pub name: Option<String>,
        pub active: bool,
    }

    impl Default for UserFixture {
        fn default() -> Self {
            Self {
                email: "test@example.com".to_string(),
                password: "password123".to_string(),
                name: Some("Test User".to_string()),
                active: true,
            }
        }
    }

    impl UserFixture {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn email(mut self, email: impl Into<String>) -> Self {
            self.email = email.into();
            self
        }

        pub fn password(mut self, password: impl Into<String>) -> Self {
            self.password = password.into();
            self
        }

        pub fn name(mut self, name: impl Into<String>) -> Self {
            self.name = Some(name.into());
            self
        }

        pub fn inactive(mut self) -> Self {
            self.active = false;
            self
        }

        pub fn build(self) -> serde_json::Value {
            serde_json::to_value(self).unwrap()
        }
    }

    /// Generic fixture builder
    pub struct FixtureBuilder {
        data: HashMap<String, serde_json::Value>,
    }

    impl FixtureBuilder {
        pub fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }

        pub fn add<T: Serialize>(mut self, key: impl Into<String>, value: T) -> Self {
            self.data.insert(key.into(), serde_json::to_value(value).unwrap());
            self
        }

        pub fn build(self) -> serde_json::Value {
            serde_json::to_value(self.data).unwrap()
        }
    }
}

/// Parallel test execution helpers
pub mod parallel {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    /// Parallel test runner
    pub struct ParallelRunner {
        semaphore: Arc<Semaphore>,
        environments: Vec<Arc<Mutex<TestEnvironment>>>,
    }

    impl ParallelRunner {
        /// Create new parallel runner with concurrency limit
        pub fn new(concurrency: usize) -> Self {
            Self {
                semaphore: Arc::new(Semaphore::new(concurrency)),
                environments: Vec::new(),
            }
        }

        /// Add test environment
        pub fn add_environment(&mut self, env: TestEnvironment) {
            self.environments.push(Arc::new(Mutex::new(env)));
        }

        /// Run test function in parallel
        pub async fn run_parallel<F, Fut>(&self, test_fn: F) -> Result<(), String>
        where
            F: Fn(Arc<Mutex<TestEnvironment>>) -> Fut + Send + Sync + Clone + 'static,
            Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
        {
            let mut handles = Vec::new();

            for env in &self.environments {
                let env = Arc::clone(env);
                let test_fn = test_fn.clone();
                let permit = self.semaphore.clone().acquire_owned().await.unwrap();

                let handle = tokio::spawn(async move {
                    let _permit = permit; // Hold permit until test completes
                    test_fn(env).await
                });

                handles.push(handle);
            }

            // Wait for all tests to complete
            for handle in handles {
                handle.await
                    .map_err(|e| format!("Task panicked: {}", e))?
                    .map_err(|e| format!("Test failed: {}", e))?;
            }

            Ok(())
        }

        // Getter for testing
        pub fn environments(&self) -> &Vec<Arc<Mutex<TestEnvironment>>> {
            &self.environments
        }
    }
}

// Re-export commonly used items
pub use assertions::*;
pub use fixtures::*;
pub use parallel::*;