//! Service Discovery for Vela Programming Language
//!
//! This crate provides service discovery capabilities for microservices
//! built with Vela, supporting multiple registry backends like Consul,
//! Eureka, and etcd.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Errors that can occur during service discovery operations
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Service not found: {service_name}")]
    ServiceNotFound { service_name: String },

    #[error("Service already registered: {service_id}")]
    ServiceAlreadyExists { service_id: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Health check failed: {service_id}")]
    HealthCheckFailed { service_id: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Backend error: {backend} - {message}")]
    BackendError { backend: String, message: String },
}

/// Health status of a service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Passing,
    Warning,
    Critical,
    Unknown,
}

/// Service instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub health_status: HealthStatus,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub health_check: Option<HealthCheckConfig>,
}

/// Type of health check to perform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthCheckType {
    Http,
    Tcp,
    Ttl,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub check_type: HealthCheckType,
    pub endpoint: Option<String>, // For HTTP checks
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub deregister_after_seconds: u64,
}

/// Service watcher for receiving updates about service changes
#[async_trait]
pub trait ServiceWatcher: Send + Sync {
    async fn next(&mut self) -> Option<ServiceDiscoveryEvent>;
}

/// Events that can occur in service discovery
#[derive(Debug, Clone)]
pub enum ServiceDiscoveryEvent {
    ServiceRegistered(ServiceInstance),
    ServiceDeregistered { service_id: String, service_name: String },
    ServiceHealthChanged { service_id: String, new_status: HealthStatus },
    ServiceUpdated { service_name: String, instances: usize },
}

/// Main trait for service registry implementations
#[async_trait]
pub trait ServiceRegistry: Send + Sync {
    /// Register a service with the registry
    async fn register(&self, service: ServiceInfo) -> Result<(), RegistryError>;

    /// Deregister a service from the registry
    async fn deregister(&self, service_id: &str) -> Result<(), RegistryError>;

    /// Discover all instances of a service
    async fn discover(&self, service_name: &str) -> Result<Vec<ServiceInstance>, RegistryError>;

    /// Get a specific service instance by ID
    async fn get_service(&self, service_id: &str) -> Result<ServiceInstance, RegistryError>;

    /// Perform health check on a service
    async fn health_check(&self, service_id: &str) -> Result<HealthStatus, RegistryError>;

    /// Watch for changes to services
    async fn watch(&self, service_name: &str) -> Result<Box<dyn ServiceWatcher>, RegistryError>;
}

/// Configuration for service registry clients
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub endpoints: Vec<String>,
    pub timeout: std::time::Duration,
    pub retry_attempts: u32,
    pub retry_delay: std::time::Duration,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            endpoints: vec!["http://localhost:8500".to_string()], // Default Consul
            timeout: std::time::Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: std::time::Duration::from_millis(100),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_info_creation() {
        let service = ServiceInfo {
            id: "test-service-1".to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec!["web".to_string(), "api".to_string()],
            metadata: HashMap::from([
                ("version".to_string(), "1.0.0".to_string()),
                ("environment".to_string(), "test".to_string()),
            ]),
            health_check: Some(HealthCheckConfig {
                check_type: HealthCheckType::Http,
                endpoint: Some("/health".to_string()),
                interval_seconds: 30,
                timeout_seconds: 5,
                deregister_after_seconds: 300,
            }),
        };

        assert_eq!(service.id, "test-service-1");
        assert_eq!(service.name, "test-service");
        assert_eq!(service.port, 8080);
        assert!(service.tags.contains(&"web".to_string()));
        assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
    }
}

// Re-export modules
pub mod advanced_consul;
pub mod client;
pub mod consul;
pub mod eureka;
pub mod in_memory;

pub use advanced_consul::{AdvancedConsulRegistry, AdvancedConsulConfig, ConsulACLToken, ServiceIntention};
pub use client::{ServiceDiscoveryHttpClient, ServiceDiscoveryClientConfig, HttpRequest, HttpResponse, HttpMethod, ServiceDiscoveryError, LoadBalancerStrategy, CircuitBreakerState};
pub use consul::{ConsulRegistry, ConsulConfig};
pub use eureka::{EurekaRegistry, EurekaConfig};
pub use in_memory::InMemoryRegistry;

/// Service discovery client that manages service registration and discovery
pub struct ServiceDiscoveryClient {
    registry: Arc<dyn ServiceRegistry + Send + Sync>,
    registered_services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    health_check_interval: std::time::Duration,
    auto_deregister: bool,
}

impl ServiceDiscoveryClient {
    /// Create a new service discovery client with the given registry
    pub fn new(registry: Arc<dyn ServiceRegistry + Send + Sync>) -> Self {
        Self {
            registry,
            registered_services: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval: std::time::Duration::from_secs(30),
            auto_deregister: true,
        }
    }

    /// Create a new client with custom configuration
    pub fn with_config(
        registry: Arc<dyn ServiceRegistry + Send + Sync>,
        health_check_interval: std::time::Duration,
        auto_deregister: bool,
    ) -> Self {
        Self {
            registry,
            registered_services: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval,
            auto_deregister,
        }
    }

    /// Register a service with the registry
    pub async fn register_service(&self, service: ServiceInfo) -> Result<(), RegistryError> {
        // Register with the backend
        self.registry.register(service.clone()).await?;

        // Store locally for management
        let mut registered = self.registered_services.write().await;
        registered.insert(service.id.clone(), service);

        Ok(())
    }

    /// Deregister a service from the registry
    pub async fn deregister_service(&self, service_id: &str) -> Result<(), RegistryError> {
        // Deregister from backend
        self.registry.deregister(service_id).await?;

        // Remove from local storage
        let mut registered = self.registered_services.write().await;
        registered.remove(service_id);

        Ok(())
    }

    /// Discover services by name
    pub async fn discover_services(&self, service_name: &str) -> Result<Vec<ServiceInstance>, RegistryError> {
        self.registry.discover(service_name).await
    }

    /// Get a specific service instance
    pub async fn get_service(&self, service_id: &str) -> Result<ServiceInstance, RegistryError> {
        self.registry.get_service(service_id).await
    }

    /// Perform health check on a service
    pub async fn check_service_health(&self, service_id: &str) -> Result<HealthStatus, RegistryError> {
        self.registry.health_check(service_id).await
    }

    /// Watch for service changes
    pub async fn watch_services(&self, service_name: &str) -> Result<Box<dyn ServiceWatcher>, RegistryError> {
        self.registry.watch(service_name).await
    }

    /// Start the health check loop for all registered services
    pub async fn start_health_checks(&self) -> Result<(), RegistryError> {
        let client = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(client.health_check_interval);

            loop {
                interval.tick().await;

                let registered_services = {
                    let registered = client.registered_services.read().await;
                    registered.keys().cloned().collect::<Vec<_>>()
                };

                for service_id in registered_services {
                    match client.check_service_health(&service_id).await {
                        Ok(HealthStatus::Critical) if client.auto_deregister => {
                            if let Err(e) = client.deregister_service(&service_id).await {
                                eprintln!("Failed to auto-deregister unhealthy service {}: {}", service_id, e);
                            } else {
                                println!("Auto-deregistered unhealthy service: {}", service_id);
                            }
                        }
                        Ok(status) => {
                            println!("Service {} health: {:?}", service_id, status);
                        }
                        Err(e) => {
                            eprintln!("Health check failed for service {}: {}", service_id, e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Get all locally registered services
    pub async fn get_registered_services(&self) -> HashMap<String, ServiceInfo> {
        let registered = self.registered_services.read().await;
        registered.clone()
    }

    /// Generate a unique service ID
    pub fn generate_service_id(service_name: &str) -> String {
        format!("{}-{}", service_name, Uuid::new_v4().simple())
    }
}

impl Clone for ServiceDiscoveryClient {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            registered_services: Arc::clone(&self.registered_services),
            health_check_interval: self.health_check_interval,
            auto_deregister: self.auto_deregister,
        }
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;

    #[tokio::test]
    async fn test_client_registration() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        let service = ServiceInfo {
            id: "test-service-1".to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec!["web".to_string()],
            metadata: HashMap::new(),
            health_check: None,
        };

        // Register service
        client.register_service(service.clone()).await.unwrap();

        // Verify it's registered locally
        let registered = client.get_registered_services().await;
        assert_eq!(registered.len(), 1);
        assert!(registered.contains_key("test-service-1"));

        // Discover service
        let instances = client.discover_services("test-service").await.unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id, "test-service-1");
    }

    #[tokio::test]
    async fn test_client_deregistration() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        let service = ServiceInfo {
            id: "test-service-2".to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };

        // Register service
        client.register_service(service).await.unwrap();

        // Deregister service
        client.deregister_service("test-service-2").await.unwrap();

        // Verify it's deregistered locally
        let registered = client.get_registered_services().await;
        assert_eq!(registered.len(), 0);

        // Verify it's deregistered from registry
        let result = client.discover_services("test-service").await;
        assert!(matches!(result, Err(RegistryError::ServiceNotFound { .. })));
    }

    #[test]
    fn test_generate_service_id() {
        let id1 = ServiceDiscoveryClient::generate_service_id("web-service");
        let id2 = ServiceDiscoveryClient::generate_service_id("web-service");

        assert!(id1.starts_with("web-service-"));
        assert!(id2.starts_with("web-service-"));
        assert_ne!(id1, id2); // Should be unique
    }
}