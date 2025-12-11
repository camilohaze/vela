//! Advanced Service Discovery HTTP Client
//!
//! This module provides an HTTP client that integrates with service discovery
//! to automatically discover and call registered services with load balancing,
//! circuit breaker, and retry capabilities.

use super::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// HTTP method for service calls
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

/// HTTP request configuration
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub query_params: HashMap<String, String>,
    pub timeout: Option<Duration>,
}

/// HTTP response from service call
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub duration: Duration,
    pub service_instance: ServiceInstance,
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,        // Failures before opening
    pub recovery_timeout: Duration,    // Time before trying again
    pub success_threshold: u32,        // Successes needed to close
    pub timeout: Duration,            // Request timeout
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Circuit breaker for service calls
#[derive(Debug)]
struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
        }
    }

    fn should_attempt(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.recovery_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        match self.state {
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                }
            }
            _ => {}
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
}

/// Load balancer strategy
#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancerStrategy {
    RoundRobin,
    Random,
    LeastConnections, // Simplified, doesn't track actual connections
    WeightedRandom,
}

/// Service discovery HTTP client configuration
#[derive(Debug, Clone)]
pub struct ServiceDiscoveryClientConfig {
    pub load_balancer_strategy: LoadBalancerStrategy,
    pub circuit_breaker_config: CircuitBreakerConfig,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub service_mesh_enabled: bool,
    pub default_timeout: Duration,
}

impl Default for ServiceDiscoveryClientConfig {
    fn default() -> Self {
        Self {
            load_balancer_strategy: LoadBalancerStrategy::RoundRobin,
            circuit_breaker_config: CircuitBreakerConfig::default(),
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            service_mesh_enabled: false,
            default_timeout: Duration::from_secs(30),
        }
    }
}

/// Advanced service discovery HTTP client
pub struct ServiceDiscoveryHttpClient {
    registry: Arc<dyn ServiceRegistry + Send + Sync>,
    http_client: Client,
    config: ServiceDiscoveryClientConfig,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    round_robin_index: Arc<RwLock<HashMap<String, usize>>>,
}

impl ServiceDiscoveryHttpClient {
    /// Create a new service discovery HTTP client
    pub fn new(
        registry: Arc<dyn ServiceRegistry + Send + Sync>,
        config: ServiceDiscoveryClientConfig,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(config.default_timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            registry,
            http_client,
            config,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            round_robin_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a client with default configuration
    pub fn with_registry(registry: Arc<dyn ServiceRegistry + Send + Sync>) -> Self {
        Self::new(registry, ServiceDiscoveryClientConfig::default())
    }

    /// Call a service with automatic discovery and load balancing
    pub async fn call_service(
        &self,
        service_name: &str,
        request: HttpRequest,
    ) -> Result<HttpResponse, ServiceDiscoveryError> {
        let mut attempts = 0;
        let max_attempts = self.config.max_retries + 1;

        while attempts < max_attempts {
            attempts += 1;

            // Discover service instances
            let instances = self.registry.discover(service_name).await
                .map_err(|e| ServiceDiscoveryError::RegistryError(e.to_string()))?;

            if instances.is_empty() {
                return Err(ServiceDiscoveryError::ServiceNotFound(service_name.to_string()));
            }

            // Filter healthy instances
            let healthy_instances: Vec<_> = instances.into_iter()
                .filter(|inst| inst.health_status == HealthStatus::Passing)
                .collect();

            if healthy_instances.is_empty() {
                return Err(ServiceDiscoveryError::NoHealthyInstances(service_name.to_string()));
            }

            // Select instance using load balancer
            let selected_instance = self.select_instance(service_name, &healthy_instances).await?;

            // Check circuit breaker
            if !self.check_circuit_breaker(service_name).await {
                if attempts == max_attempts {
                    return Err(ServiceDiscoveryError::CircuitBreakerOpen(service_name.to_string()));
                }
                tokio::time::sleep(self.config.retry_delay).await;
                continue;
            }

            // Make the HTTP call
            match self.make_http_call(&selected_instance, &request).await {
                Ok(response) => {
                    self.record_success(service_name).await;
                    return Ok(response);
                }
                Err(e) => {
                    self.record_failure(service_name).await;

                    if attempts == max_attempts {
                        return Err(e);
                    }

                    // Wait before retry
                    tokio::time::sleep(self.config.retry_delay * attempts as u32).await;
                }
            }
        }

        Err(ServiceDiscoveryError::MaxRetriesExceeded(service_name.to_string()))
    }

    /// Call a service using service mesh (if available)
    pub async fn call_service_mesh(
        &self,
        service_name: &str,
        request: HttpRequest,
    ) -> Result<HttpResponse, ServiceDiscoveryError> {
        if !self.config.service_mesh_enabled {
            return self.call_service(service_name, request).await;
        }

        // For service mesh, we might want to use sidecar proxy
        // This is a simplified implementation
        let mesh_request = HttpRequest {
            path: format!("/{}", service_name).to_string() + &request.path,
            ..request
        };

        // In a real implementation, this would route through the sidecar
        self.call_service(service_name, mesh_request).await
    }

    /// Get circuit breaker status for a service
    pub async fn get_circuit_breaker_status(&self, service_name: &str) -> CircuitBreakerState {
        let breakers = self.circuit_breakers.read().await;
        breakers.get(service_name)
            .map(|cb| cb.state.clone())
            .unwrap_or(CircuitBreakerState::Closed)
    }

    /// Reset circuit breaker for a service
    pub async fn reset_circuit_breaker(&self, service_name: &str) {
        let mut breakers = self.circuit_breakers.write().await;
        if let Some(cb) = breakers.get_mut(service_name) {
            cb.state = CircuitBreakerState::Closed;
            cb.failure_count = 0;
            cb.success_count = 0;
            cb.last_failure_time = None;
        }
    }

    /// Select instance using configured load balancer strategy
    async fn select_instance(
        &self,
        service_name: &str,
        instances: &[ServiceInstance],
    ) -> Result<ServiceInstance, ServiceDiscoveryError> {
        match self.config.load_balancer_strategy {
            LoadBalancerStrategy::RoundRobin => {
                self.select_round_robin(service_name, instances).await
            }
            LoadBalancerStrategy::Random => {
                self.select_random(instances)
            }
            LoadBalancerStrategy::LeastConnections => {
                // Simplified: just use round robin for now
                self.select_round_robin(service_name, instances).await
            }
            LoadBalancerStrategy::WeightedRandom => {
                // Simplified: ignore weights for now
                self.select_random(instances)
            }
        }
    }

    /// Round-robin instance selection
    async fn select_round_robin(
        &self,
        service_name: &str,
        instances: &[ServiceInstance],
    ) -> Result<ServiceInstance, ServiceDiscoveryError> {
        let mut indices = self.round_robin_index.write().await;
        let index = indices.entry(service_name.to_string()).or_insert(0);

        let instance = instances[*index % instances.len()].clone();
        *index = (*index + 1) % instances.len();

        Ok(instance)
    }

    /// Random instance selection
    fn select_random(&self, instances: &[ServiceInstance]) -> Result<ServiceInstance, ServiceDiscoveryError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..instances.len());
        Ok(instances[index].clone())
    }

    /// Check if circuit breaker allows the call
    async fn check_circuit_breaker(&self, service_name: &str) -> bool {
        let mut breakers = self.circuit_breakers.write().await;
        let breaker = breakers.entry(service_name.to_string())
            .or_insert_with(|| CircuitBreaker::new(self.config.circuit_breaker_config.clone()));

        breaker.should_attempt()
    }

    /// Record successful call
    async fn record_success(&self, service_name: &str) {
        let mut breakers = self.circuit_breakers.write().await;
        if let Some(breaker) = breakers.get_mut(service_name) {
            breaker.record_success();
        }
    }

    /// Record failed call
    async fn record_failure(&self, service_name: &str) {
        let mut breakers = self.circuit_breakers.write().await;
        if let Some(breaker) = breakers.get_mut(service_name) {
            breaker.record_failure();
        }
    }

    /// Make the actual HTTP call to the service instance
    async fn make_http_call(
        &self,
        instance: &ServiceInstance,
        request: &HttpRequest,
    ) -> Result<HttpResponse, ServiceDiscoveryError> {
        let start_time = Instant::now();

        // Build URL
        let mut url = format!("http://{}:{}", instance.address, instance.port);
        if !request.path.starts_with('/') {
            url.push('/');
        }
        url.push_str(&request.path);

        // Add query parameters
        if !request.query_params.is_empty() {
            let query_string = request.query_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            url.push('?');
            url.push_str(&query_string);
        }

        // Build request
        let mut req_builder = match request.method {
            HttpMethod::GET => self.http_client.get(&url),
            HttpMethod::POST => self.http_client.post(&url),
            HttpMethod::PUT => self.http_client.put(&url),
            HttpMethod::DELETE => self.http_client.delete(&url),
            HttpMethod::PATCH => self.http_client.patch(&url),
            HttpMethod::HEAD => self.http_client.head(&url),
            HttpMethod::OPTIONS => self.http_client.request(reqwest::Method::OPTIONS, &url),
        };

        // Add headers
        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        // Add service mesh headers if enabled
        if self.config.service_mesh_enabled {
            req_builder = req_builder.header("x-service-mesh", "true");
        }

        // Add body for POST/PUT/PATCH
        if let Some(body) = &request.body {
            req_builder = req_builder.body(body.clone());
        }

        // Set timeout
        if let Some(timeout) = request.timeout {
            req_builder = req_builder.timeout(timeout);
        }

        // Execute request
        let response = req_builder
            .send()
            .await
            .map_err(|e| ServiceDiscoveryError::HttpError(e.to_string()))?;

        let status_code = response.status().as_u16();

        // Read headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // Read body
        let body = if response.status().is_success() {
            Some(response.text().await
                .map_err(|e| ServiceDiscoveryError::HttpError(e.to_string()))?)
        } else {
            None
        };

        let duration = start_time.elapsed();

        Ok(HttpResponse {
            status_code,
            headers,
            body,
            duration,
            service_instance: instance.clone(),
        })
    }
}

/// Errors that can occur during service discovery HTTP calls
#[derive(Error, Debug)]
pub enum ServiceDiscoveryError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("No healthy instances available for service: {0}")]
    NoHealthyInstances(String),

    #[error("Circuit breaker is open for service: {0}")]
    CircuitBreakerOpen(String),

    #[error("Maximum retries exceeded for service: {0}")]
    MaxRetriesExceeded(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_http_client_creation() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryHttpClient::with_registry(registry);

        // Just test that it can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_circuit_breaker_states() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 1,
            timeout: Duration::from_secs(1),
        };

        let mut breaker = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(breaker.state, CircuitBreakerState::Closed);
        assert!(breaker.should_attempt());

        // Record failures
        breaker.record_failure();
        assert_eq!(breaker.state, CircuitBreakerState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state, CircuitBreakerState::Open);
        assert!(!breaker.should_attempt());

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(breaker.should_attempt());
        assert_eq!(breaker.state, CircuitBreakerState::HalfOpen);

        // Record success to close
        breaker.record_success();
        assert_eq!(breaker.state, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryHttpClient::with_registry(registry);

        let instances = vec![
            ServiceInstance {
                id: "inst1".to_string(),
                name: "test-service".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8080,
                tags: vec![],
                metadata: HashMap::new(),
                health_status: HealthStatus::Passing,
                last_health_check: None,
            },
            ServiceInstance {
                id: "inst2".to_string(),
                name: "test-service".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8081,
                tags: vec![],
                metadata: HashMap::new(),
                health_status: HealthStatus::Passing,
                last_health_check: None,
            },
        ];

        // Test round-robin selection
        let inst1 = client.select_round_robin("test-service", &instances).await.unwrap();
        assert_eq!(inst1.id, "inst1");

        let inst2 = client.select_round_robin("test-service", &instances).await.unwrap();
        assert_eq!(inst2.id, "inst2");

        let inst1_again = client.select_round_robin("test-service", &instances).await.unwrap();
        assert_eq!(inst1_again.id, "inst1");
    }

    #[tokio::test]
    async fn test_random_selection() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryHttpClient::with_registry(registry);

        let instances = vec![
            ServiceInstance {
                id: "inst1".to_string(),
                name: "test-service".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8080,
                tags: vec![],
                metadata: HashMap::new(),
                health_status: HealthStatus::Passing,
                last_health_check: None,
            },
            ServiceInstance {
                id: "inst2".to_string(),
                name: "test-service".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8081,
                tags: vec![],
                metadata: HashMap::new(),
                health_status: HealthStatus::Passing,
                last_health_check: None,
            },
        ];

        // Test random selection (just check it returns a valid instance)
        let selected = client.select_random(&instances).unwrap();
        assert!(selected.id == "inst1" || selected.id == "inst2");
    }

    #[test]
    fn test_http_request_creation() {
        let request = HttpRequest {
            method: HttpMethod::GET,
            path: "/api/users".to_string(),
            headers: HashMap::from([("Authorization".to_string(), "Bearer token".to_string())]),
            body: None,
            query_params: HashMap::from([("limit".to_string(), "10".to_string())]),
            timeout: Some(Duration::from_secs(5)),
        };

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/api/users");
        assert_eq!(request.headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(request.query_params.get("limit"), Some(&"10".to_string()));
    }
}