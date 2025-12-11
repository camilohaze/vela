//! In-Memory Service Registry Implementation
//!
//! This module provides an in-memory implementation of the ServiceRegistry
//! trait, useful for testing and development environments.

use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory service registry for testing and development
pub struct InMemoryRegistry {
    services: Arc<RwLock<HashMap<String, ServiceInstance>>>,
    service_names: Arc<RwLock<HashMap<String, Vec<String>>>>, // name -> [service_ids]
}

impl InMemoryRegistry {
    /// Create a new in-memory registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new in-memory registry wrapped in Arc
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

impl Default for InMemoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServiceRegistry for InMemoryRegistry {
    async fn register(&self, service: ServiceInfo) -> Result<(), RegistryError> {
        let instance = ServiceInstance {
            id: service.id.clone(),
            name: service.name.clone(),
            address: service.address.clone(),
            port: service.port,
            tags: service.tags.clone(),
            metadata: service.metadata.clone(),
            health_status: HealthStatus::Passing,
            last_health_check: Some(chrono::Utc::now()),
        };

        let mut services = self.services.write().await;
        let mut service_names = self.service_names.write().await;

        // Check if service ID already exists
        if services.contains_key(&service.id) {
            return Err(RegistryError::ServiceAlreadyExists {
                service_id: service.id,
            });
        }

        // Add to services map
        services.insert(service.id.clone(), instance);

        // Add to service names index
        service_names
            .entry(service.name.clone())
            .or_insert_with(Vec::new)
            .push(service.id.clone());

        Ok(())
    }

    async fn deregister(&self, service_id: &str) -> Result<(), RegistryError> {
        let mut services = self.services.write().await;
        let mut service_names = self.service_names.write().await;

        // Remove from services map
        let service = services.remove(service_id).ok_or_else(|| {
            RegistryError::ServiceNotFound {
                service_name: service_id.to_string(),
            }
        })?;

        // Remove from service names index
        if let Some(service_ids) = service_names.get_mut(&service.name) {
            service_ids.retain(|id| id != service_id);
            if service_ids.is_empty() {
                service_names.remove(&service.name);
            }
        }

        Ok(())
    }

    async fn discover(&self, service_name: &str) -> Result<Vec<ServiceInstance>, RegistryError> {
        let services = self.services.read().await;
        let service_names = self.service_names.read().await;

        let service_ids = service_names.get(service_name).ok_or_else(|| {
            RegistryError::ServiceNotFound {
                service_name: service_name.to_string(),
            }
        })?;

        let instances: Vec<ServiceInstance> = service_ids
            .iter()
            .filter_map(|id| services.get(id).cloned())
            .collect();

        Ok(instances)
    }

    async fn get_service(&self, service_id: &str) -> Result<ServiceInstance, RegistryError> {
        let services = self.services.read().await;

        services
            .get(service_id)
            .cloned()
            .ok_or_else(|| RegistryError::ServiceNotFound {
                service_name: service_id.to_string(),
            })
    }

    async fn health_check(&self, service_id: &str) -> Result<HealthStatus, RegistryError> {
        let mut services = self.services.write().await;

        let service = services.get_mut(service_id).ok_or_else(|| {
            RegistryError::ServiceNotFound {
                service_name: service_id.to_string(),
            }
        })?;

        // Simulate health check - in real implementations this would
        // actually check the service health
        service.last_health_check = Some(chrono::Utc::now());

        // For demo purposes, randomly change health status
        // In real implementations, this would be based on actual health checks
        let new_status = match service.health_status {
            HealthStatus::Passing => {
                if rand::random::<f32>() < 0.1 {
                    HealthStatus::Warning
                } else {
                    HealthStatus::Passing
                }
            }
            HealthStatus::Warning => {
                if rand::random::<f32>() < 0.05 {
                    HealthStatus::Critical
                } else if rand::random::<f32>() < 0.2 {
                    HealthStatus::Passing
                } else {
                    HealthStatus::Warning
                }
            }
            HealthStatus::Critical => {
                if rand::random::<f32>() < 0.3 {
                    HealthStatus::Warning
                } else {
                    HealthStatus::Critical
                }
            }
            HealthStatus::Unknown => HealthStatus::Passing,
        };

        service.health_status = new_status.clone();
        Ok(new_status)
    }

    async fn watch(&self, _service_name: &str) -> Result<Box<dyn ServiceWatcher>, RegistryError> {
        // For simplicity, return a basic watcher that doesn't actually watch
        // In a real implementation, this would return a watcher that streams events
        Ok(Box::new(NoOpWatcher))
    }
}

/// A no-op watcher for the in-memory registry
struct NoOpWatcher;

#[async_trait]
impl ServiceWatcher for NoOpWatcher {
    async fn next(&mut self) -> Option<ServiceDiscoveryEvent> {
        // This is a simple implementation that never returns events
        // In a real implementation, this would wait for actual service changes
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_discover_service() {
        let registry = InMemoryRegistry::new();

        let service = ServiceInfo {
            id: "web-service-1".to_string(),
            name: "web-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec!["web".to_string()],
            metadata: HashMap::new(),
            health_check: None,
        };

        // Register service
        registry.register(service).await.unwrap();

        // Discover service
        let instances = registry.discover("web-service").await.unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id, "web-service-1");
        assert_eq!(instances[0].name, "web-service");
        assert_eq!(instances[0].port, 8080);
    }

    #[tokio::test]
    async fn test_deregister_service() {
        let registry = InMemoryRegistry::new();

        let service = ServiceInfo {
            id: "api-service-1".to_string(),
            name: "api-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 3000,
            tags: vec!["api".to_string()],
            metadata: HashMap::new(),
            health_check: None,
        };

        // Register service
        registry.register(service).await.unwrap();

        // Verify it's registered
        let instances = registry.discover("api-service").await.unwrap();
        assert_eq!(instances.len(), 1);

        // Deregister service
        registry.deregister("api-service-1").await.unwrap();

        // Verify it's deregistered
        let result = registry.discover("api-service").await;
        assert!(matches!(result, Err(RegistryError::ServiceNotFound { .. })));
    }

    #[tokio::test]
    async fn test_multiple_services_same_name() {
        let registry = InMemoryRegistry::new();

        let service1 = ServiceInfo {
            id: "web-1".to_string(),
            name: "web-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };

        let service2 = ServiceInfo {
            id: "web-2".to_string(),
            name: "web-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8081,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };

        // Register both services
        registry.register(service1).await.unwrap();
        registry.register(service2).await.unwrap();

        // Discover should return both
        let instances = registry.discover("web-service").await.unwrap();
        assert_eq!(instances.len(), 2);

        let ports: Vec<u16> = instances.iter().map(|s| s.port).collect();
        assert!(ports.contains(&8080));
        assert!(ports.contains(&8081));
    }

    #[tokio::test]
    async fn test_service_not_found() {
        let registry = InMemoryRegistry::new();

        let result = registry.discover("non-existent-service").await;
        assert!(matches!(result, Err(RegistryError::ServiceNotFound { .. })));
    }

    #[tokio::test]
    async fn test_duplicate_service_registration() {
        let registry = InMemoryRegistry::new();

        let service = ServiceInfo {
            id: "duplicate-service".to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };

        // Register first time - should succeed
        registry.register(service.clone()).await.unwrap();

        // Register second time - should fail
        let result = registry.register(service).await;
        assert!(matches!(result, Err(RegistryError::ServiceAlreadyExists { .. })));
    }
}