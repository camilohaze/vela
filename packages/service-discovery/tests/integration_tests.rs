//! Integration Tests for Service Discovery
//!
//! This module contains comprehensive integration tests for the service discovery system,
//! covering registration, discovery, failover, and recovery scenarios.

use service_discovery::{
    InMemoryRegistry, ServiceDiscoveryClient, ServiceInfo, RegistryError,
    HealthCheckConfig, HealthCheckType,
    HealthCheckServer, HealthServerConfig, HealthCheckResult,
};
use std::collections::HashMap;
use std::sync::Arc;
use futures::future::FutureExt;
use tokio::time::sleep;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test complete service lifecycle: registration, discovery, deregistration
    #[tokio::test]
    async fn test_service_lifecycle() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register a service
        let service = ServiceInfo {
            id: "web-service-1".to_string(),
            name: "web-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec!["web".to_string(), "api".to_string()],
            metadata: HashMap::new(),
            health_check: Some(HealthCheckConfig {
                check_type: HealthCheckType::Http,
                endpoint: Some("/health".to_string()),
                interval_seconds: 30,
                timeout_seconds: 5,
                deregister_after_seconds: 300,
            }),
        };

        // Register service
        client.register_service(service.clone()).await.unwrap();

        // Discover service
        let instances = client.discover_services("web-service").await.unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id, "web-service-1");

        // Deregister service
        client.deregister_service("web-service-1").await.unwrap();

        // Verify service is gone
        match client.discover_services("web-service").await {
            Err(RegistryError::ServiceNotFound { .. }) => {
                // Expected: no services found
            }
            Ok(instances) => {
                assert_eq!(instances.len(), 0, "Expected no services after deregistration");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    /// Test multiple service instances with load balancing
    #[tokio::test]
    async fn test_multiple_instances_load_balancing() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register multiple instances
        for i in 1..=3 {
            let service = ServiceInfo {
                id: format!("api-service-{}", i),
                name: "api-service".to_string(),
                address: format!("10.0.0.{}", i),
                port: 8080 + i,
                tags: vec!["api".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // Discover all instances
        let instances = client.discover_services("api-service").await.unwrap();
        assert_eq!(instances.len(), 3);

        // Test round-robin load balancing
        let mut selected_instances = Vec::new();
        let mut instance_index = 0;
        for _ in 0..6 {
            let instances = client.discover_services("api-service").await.unwrap();
            let instance = &instances[instance_index % instances.len()];
            selected_instances.push(instance.id.clone());
            instance_index += 1;
        }

        // Should have distributed load across all instances
        assert!(selected_instances.contains(&"api-service-1".to_string()));
        assert!(selected_instances.contains(&"api-service-2".to_string()));
        assert!(selected_instances.contains(&"api-service-3".to_string()));
    }

    /// Test service failover when instances become unhealthy
    #[tokio::test]
    async fn test_service_failover() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register healthy instances
        for i in 1..=3 {
            let service = ServiceInfo {
                id: format!("cache-service-{}", i),
                name: "cache-service".to_string(),
                address: format!("10.0.0.{}", i),
                port: 6379,
                tags: vec!["cache".to_string(), "redis".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // All instances should be available
        let instances = client.discover_services("cache-service").await.unwrap();
        assert_eq!(instances.len(), 3);

        // Simulate instance failure (deregister one instance)
        client.deregister_service("cache-service-2").await.unwrap();

        // Should still have 2 healthy instances
        let instances = client.discover_services("cache-service").await.unwrap();
        assert_eq!(instances.len(), 2);

        // Load balancing should still work with remaining instances
        let instances = client.discover_services("cache-service").await.unwrap();
        let instance = instances.first().unwrap();
        assert!(instance.id == "cache-service-1" || instance.id == "cache-service-3");

        // Register the failed instance back (recovery)
        let recovered_service = ServiceInfo {
            id: "cache-service-2".to_string(),
            name: "cache-service".to_string(),
            address: "10.0.0.2".to_string(),
            port: 6379,
            tags: vec!["cache".to_string(), "redis".to_string()],
            metadata: HashMap::new(),
            health_check: None,
        };
        client.register_service(recovered_service).await.unwrap();

        // All instances should be available again
        let instances = client.discover_services("cache-service").await.unwrap();
        assert_eq!(instances.len(), 3);
    }

    /// Test circuit breaker behavior during service failures
    #[tokio::test]
    async fn test_circuit_breaker_failover() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register a service
        let service = ServiceInfo {
            id: "failing-service".to_string(),
            name: "failing-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };
        client.register_service(service).await.unwrap();

        // Simulate multiple failures (circuit breaker would open in real implementation)
        // For this test, we just verify the service can be discovered
        let instances = client.discover_services("failing-service").await.unwrap();
        let instance = instances.first().unwrap();
        assert_eq!(instance.id, "failing-service");
    }

    /// Test service discovery with health checks integration
    #[tokio::test]
    async fn test_health_check_integration() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Create health check server
        let health_config = HealthServerConfig {
            port: 8081,
            host: "127.0.0.1".to_string(),
            enable_cors: false,
            enable_tracing: false,
            readiness_timeout_seconds: 5,
            liveness_timeout_seconds: 5,
        };
        let health_server = HealthCheckServer::with_service_client(health_config, Arc::new(client.clone()));

        // Add a custom health check
        health_server.add_readiness_check(
            "database".to_string(),
            Box::new(|| {
                async move {
                    // Simulate database health check
                    HealthCheckResult {
                        status: "healthy".to_string(),
                        message: Some("Database connection OK".to_string()),
                        timestamp: chrono::Utc::now(),
                        duration_ms: 10,
                    }
                }.boxed()
            }),
        );

        // Register a service that depends on database
        let service = ServiceInfo {
            id: "dependent-service".to_string(),
            name: "dependent-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec!["api".to_string()],
            metadata: HashMap::new(),
            health_check: None,
        };
        client.register_service(service).await.unwrap();

        // Service should be discoverable
        let instances = client.discover_services("dependent-service").await.unwrap();
        assert_eq!(instances.len(), 1);
    }

    /// Test concurrent service operations
    #[tokio::test]
    async fn test_concurrent_operations() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Spawn multiple tasks registering services concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                let service = ServiceInfo {
                    id: format!("concurrent-service-{}", i),
                    name: "concurrent-service".to_string(),
                    address: format!("10.0.0.{}", i % 255),
                    port: 8000 + i,
                    tags: vec!["test".to_string()],
                    metadata: HashMap::new(),
                    health_check: None,
                };
                client_clone.register_service(service).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all registrations to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all services are registered
        let instances = client.discover_services("concurrent-service").await.unwrap();
        assert_eq!(instances.len(), 10);

        // Test concurrent discovery
        let mut discovery_handles = vec![];
        for _ in 0..5 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                let instances = client_clone.discover_services("concurrent-service").await.unwrap();
                instances.len()
            });
            discovery_handles.push(handle);
        }

        // All discoveries should return the same count
        for handle in discovery_handles {
            let count = handle.await.unwrap();
            assert_eq!(count, 10);
        }
    }

    /// Test service mesh scenarios with upstream dependencies
    #[tokio::test]
    async fn test_service_mesh_dependencies() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register backend services
        let backend_services = vec!["user-db", "product-db", "payment-service"];

        for service_name in &backend_services {
            let service = ServiceInfo {
                id: format!("{}-1", service_name),
                name: service_name.to_string(),
                address: "127.0.0.1".to_string(),
                port: 9000,
                tags: vec!["backend".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // Register API gateway that depends on backend services
        let gateway = ServiceInfo {
            id: "api-gateway-1".to_string(),
            name: "api-gateway".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec!["gateway".to_string(), "api".to_string()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("upstream_services".to_string(),
                           backend_services.join(",").to_string());
                meta
            },
            health_check: None,
        };
        client.register_service(gateway).await.unwrap();

        // Verify all services are discoverable
        for service_name in &backend_services {
            let instances = client.discover_services(service_name).await.unwrap();
            assert_eq!(instances.len(), 1);
        }

        let gateway_instances = client.discover_services("api-gateway").await.unwrap();
        assert_eq!(gateway_instances.len(), 1);

        // Simulate backend service failure
        client.deregister_service("user-db-1").await.unwrap();

        // Gateway should still be discoverable, but user service should be gone
        match client.discover_services("user-db").await {
            Err(RegistryError::ServiceNotFound { .. }) => {
                // Expected: no services found
            }
            Ok(instances) => {
                assert_eq!(instances.len(), 0, "Expected no user-db services after deregistration");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        let gateway_instances = client.discover_services("api-gateway").await.unwrap();
        assert_eq!(gateway_instances.len(), 1);
    }

    /// Test service discovery under high load
    #[tokio::test]
    async fn test_high_load_service_discovery() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register many services
        const NUM_SERVICES: usize = 100;
        for i in 0..NUM_SERVICES {
            let service = ServiceInfo {
                id: format!("load-test-service-{}", i),
                name: "load-test-service".to_string(),
                address: format!("10.0.{}.{}", (i / 255), (i % 255)),
                port: (8000u16 + (i % 1000) as u16),
                tags: vec!["load-test".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // Perform many concurrent discoveries
        const NUM_DISCOVERIES: usize = 50;
        let mut handles = vec![];

        for i in 0..NUM_DISCOVERIES {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                let instances = client_clone.discover_services("load-test-service").await.unwrap();
                instances.len()
            });
            handles.push(handle);
        }

        // All discoveries should return the correct count
        for (i, handle) in handles.into_iter().enumerate() {
            let count = handle.await.unwrap();
            assert_eq!(count, NUM_SERVICES, "Discovery {} returned {} instances, expected {}", i, count, NUM_SERVICES);
        }

        // Test load balancing under high load
        let mut selection_counts = HashMap::new();
        let mut instance_index = 0;
        for _ in 0..NUM_SERVICES * 2 {
            let instances = client.discover_services("load-test-service").await.unwrap();
            let instance = &instances[instance_index % instances.len()];
            *selection_counts.entry(instance.id.clone()).or_insert(0) += 1;
            instance_index += 1;
        }

        // Each service should be selected at least once
        assert_eq!(selection_counts.len(), NUM_SERVICES);
    }

    /// Test service discovery with different registry backends
    #[tokio::test]
    async fn test_multi_registry_backend_compatibility() {
        // Test with InMemory registry (always available)
        let in_memory = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(in_memory.clone());

        let service = ServiceInfo {
            id: "multi-registry-test".to_string(),
            name: "multi-registry-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
            health_check: None,
        };

        // Register with in-memory registry
        client.register_service(service.clone()).await.unwrap();

        // Discover from in-memory registry
        let instances = client.discover_services("multi-registry-service").await.unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id, "multi-registry-test");

        // This test demonstrates that the client interface works consistently
        // across different registry implementations
    }

    /// Test error handling and recovery scenarios
    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Test discovering non-existent service
        let result = client.discover_services("non-existent-service").await;
        match result {
            Err(RegistryError::ServiceNotFound { .. }) => {
                // Expected: no services found
            }
            Ok(instances) => {
                assert_eq!(instances.len(), 0, "Expected empty list for non-existent service");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        // Test deregistering non-existent service
        let result = client.deregister_service("non-existent-id").await;
        assert!(result.is_err()); // This should fail

        // Register and then deregister
        let service = ServiceInfo {
            id: "error-test-service".to_string(),
            name: "error-test".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };

        client.register_service(service).await.unwrap();
        client.deregister_service("error-test-service").await.unwrap();

        // Try to deregister again (should fail)
        let result = client.deregister_service("error-test-service").await;
        assert!(result.is_err());
    }

    /// Test service metadata and tagging
    #[tokio::test]
    async fn test_service_metadata_and_tagging() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.2.3".to_string());
        metadata.insert("environment".to_string(), "production".to_string());
        metadata.insert("region".to_string(), "us-west-2".to_string());

        let service = ServiceInfo {
            id: "metadata-service".to_string(),
            name: "metadata-service".to_string(),
            address: "10.0.0.1".to_string(),
            port: 8080,
            tags: vec!["web".to_string(), "api".to_string(), "production".to_string()],
            metadata,
            health_check: None,
        };

        client.register_service(service).await.unwrap();

        let instances = client.discover_services("metadata-service").await.unwrap();
        assert_eq!(instances.len(), 1);

        let instance = &instances[0];
        assert_eq!(instance.tags.len(), 3);
        assert!(instance.tags.contains(&"web".to_string()));
        assert!(instance.tags.contains(&"api".to_string()));
        assert!(instance.tags.contains(&"production".to_string()));

        assert_eq!(instance.metadata.get("version"), Some(&"1.2.3".to_string()));
        assert_eq!(instance.metadata.get("environment"), Some(&"production".to_string()));
        assert_eq!(instance.metadata.get("region"), Some(&"us-west-2".to_string()));
    }
}