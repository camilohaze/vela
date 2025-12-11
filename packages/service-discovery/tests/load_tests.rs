//! Load and Stress Tests for Service Discovery
//!
//! This module contains performance and stress tests to ensure the service discovery
//! system can handle high loads and edge cases.

use service_discovery::{
    InMemoryRegistry, ServiceDiscoveryClient, ServiceInfo, RegistryError,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[cfg(test)]
mod load_tests {
    use super::*;

    /// Test service discovery performance under load
    #[tokio::test]
    async fn test_service_discovery_performance() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register many services
        const NUM_SERVICES: usize = 1000;
        let start = Instant::now();

        for i in 0..NUM_SERVICES {
            let service = ServiceInfo {
                id: format!("perf-service-{}", i),
                name: "perf-service".to_string(),
                address: format!("10.0.{}.{}", (i / 255), (i % 255)),
                port: (8000u16 + (i % 1000) as u16),
                tags: vec!["performance".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        let registration_time = start.elapsed();
        println!("Registered {} services in {:?}", NUM_SERVICES, registration_time);

        // Test discovery performance
        let discovery_start = Instant::now();
        const NUM_DISCOVERIES: usize = 100;

        for _ in 0..NUM_DISCOVERIES {
            let _instances = client.discover_services("perf-service").await.unwrap();
        }

        let discovery_time = discovery_start.elapsed();
        let avg_discovery_time = discovery_time / NUM_DISCOVERIES as u32;

        println!("Performed {} discoveries in {:?}, avg: {:?}", NUM_DISCOVERIES, discovery_time, avg_discovery_time);

        // Performance assertions
        assert!(registration_time < Duration::from_secs(5), "Registration should be fast");
        assert!(avg_discovery_time < Duration::from_millis(10), "Discovery should be fast");
    }

    /// Test concurrent load with many clients
    #[tokio::test]
    async fn test_concurrent_load() {
        let registry = Arc::new(InMemoryRegistry::new());

        // Create multiple clients
        const NUM_CLIENTS: usize = 10;
        let mut clients = vec![];

        for _ in 0..NUM_CLIENTS {
            clients.push(ServiceDiscoveryClient::new(registry.clone()));
        }

        // Register services concurrently
        let mut handles = vec![];

        for (client_idx, client) in clients.into_iter().enumerate() {
            let handle = tokio::spawn(async move {
                for service_idx in 0..10 {
                    let service = ServiceInfo {
                        id: format!("concurrent-service-{}-{}", client_idx, service_idx),
                        name: "concurrent-service".to_string(),
                        address: format!("10.{}.0.{}", client_idx, service_idx),
                        port: 8000 + service_idx,
                        tags: vec!["concurrent".to_string()],
                        metadata: HashMap::new(),
                        health_check: None,
                    };
                    client.register_service(service).await.unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all registrations
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all services are registered
        let verification_client = ServiceDiscoveryClient::new(registry.clone());
        let instances = verification_client.discover_services("concurrent-service").await.unwrap();
        assert_eq!(instances.len(), NUM_CLIENTS * 10);
    }

    /// Test memory usage and cleanup
    #[tokio::test]
    async fn test_memory_cleanup() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register many services
        const NUM_SERVICES: usize = 500;

        for i in 0..NUM_SERVICES {
            let service = ServiceInfo {
                id: format!("memory-test-{}", i),
                name: "memory-test".to_string(),
                address: "127.0.0.1".to_string(),
                port: (8000u16 + (i % 1000) as u16),
                tags: vec!["memory".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // Verify services are there
        let instances = client.discover_services("memory-test").await.unwrap();
        assert_eq!(instances.len(), NUM_SERVICES);

        // Deregister all services
        for i in 0..NUM_SERVICES {
            client.deregister_service(&format!("memory-test-{}", i)).await.unwrap();
        }

        // Verify all services are gone
        match client.discover_services("memory-test").await {
            Err(RegistryError::ServiceNotFound { .. }) => {
                // Expected: no services found
            }
            Ok(instances) => {
                assert_eq!(instances.len(), 0, "Expected no services after deregistration");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    /// Test rapid registration/deregistration cycles
    #[tokio::test]
    async fn test_rapid_registration_cycles() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        const NUM_CYCLES: usize = 50;
        const SERVICE_ID: &str = "cycle-test-service";

        for cycle in 0..NUM_CYCLES {
            // Register service
            let service = ServiceInfo {
                id: SERVICE_ID.to_string(),
                name: "cycle-test".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8080,
                tags: vec!["cycle".to_string()],
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("cycle".to_string(), cycle.to_string());
                    meta
                },
                health_check: None,
            };

            client.register_service(service).await.unwrap();

            // Verify it's registered
            let instances = client.discover_services("cycle-test").await.unwrap();
            assert_eq!(instances.len(), 1);
            assert_eq!(instances[0].metadata.get("cycle"), Some(&cycle.to_string()));

            // Deregister service
            client.deregister_service(SERVICE_ID).await.unwrap();

            // Verify it's gone
            match client.discover_services("cycle-test").await {
                Err(RegistryError::ServiceNotFound { .. }) => {
                    // Expected: no services found
                }
                Ok(instances) => {
                    assert_eq!(instances.len(), 0, "Expected no services after deregistration");
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
    }

    /// Test service discovery with network-like delays
    #[tokio::test]
    async fn test_network_delay_simulation() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register services
        for i in 1..=3 {
            let service = ServiceInfo {
                id: format!("delay-service-{}", i),
                name: "delay-service".to_string(),
                address: format!("10.0.0.{}", i),
                port: 8080,
                tags: vec!["delay".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();

            // Simulate network delay
            sleep(Duration::from_millis(10)).await;
        }

        // Test that all services are eventually discoverable
        let mut total_instances = 0;
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 10;

        while total_instances < 3 && attempts < MAX_ATTEMPTS {
            let instances = client.discover_services("delay-service").await.unwrap();
            total_instances = instances.len();
            attempts += 1;

            if total_instances < 3 {
                sleep(Duration::from_millis(50)).await;
            }
        }

        assert_eq!(total_instances, 3, "All services should be discoverable after delays");
    }

    /// Test service discovery under memory pressure
    #[tokio::test]
    async fn test_memory_pressure() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Create services with large metadata to simulate memory pressure
        const NUM_SERVICES: usize = 200;

        for i in 0..NUM_SERVICES {
            let mut metadata = HashMap::new();

            // Add large metadata
            for j in 0..10 {
                metadata.insert(
                    format!("key-{}-{}", i, j),
                    format!("value-{}-{}", i, j).repeat(100), // 1000+ chars per value
                );
            }

            let service = ServiceInfo {
                id: format!("memory-pressure-{}", i),
                name: "memory-pressure".to_string(),
                address: "127.0.0.1".to_string(),
                port: (8000u16 + (i % 1000) as u16),
                tags: vec!["memory".to_string(), "pressure".to_string()],
                metadata,
                health_check: None,
            };

            client.register_service(service).await.unwrap();
        }

        // Verify all services are discoverable
        let instances = client.discover_services("memory-pressure").await.unwrap();
        assert_eq!(instances.len(), NUM_SERVICES);

        // Verify metadata integrity
        for instance in instances {
            let service_num: usize = instance.id.strip_prefix("memory-pressure-").unwrap().parse().unwrap();
            for j in 0..10 {
                let key = format!("key-{}-{}", service_num, j);
                let expected_value = format!("value-{}-{}", service_num, j).repeat(100);
                assert_eq!(instance.metadata.get(&key), Some(&expected_value));
            }
        }
    }

    /// Test failover under high frequency operations
    #[tokio::test]
    async fn test_high_frequency_failover() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Register initial services
        for i in 1..=5 {
            let service = ServiceInfo {
                id: format!("failover-service-{}", i),
                name: "failover-service".to_string(),
                address: format!("10.0.0.{}", i),
                port: 8080,
                tags: vec!["failover".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // Perform rapid failover simulation
        for round in 0..10 {
            // Fail a service
            let failing_id = format!("failover-service-{}", (round % 5) + 1);
            client.deregister_service(&failing_id).await.unwrap();

            // Immediately try to discover
            let instances = client.discover_services("failover-service").await.unwrap();
            assert_eq!(instances.len(), 4, "Should have 4 services after failure in round {}", round);

            // Recover the service
            let recovered_service = ServiceInfo {
                id: failing_id,
                name: "failover-service".to_string(),
                address: format!("10.0.0.{}", (round % 5) + 1),
                port: 8080,
                tags: vec!["failover".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(recovered_service).await.unwrap();

            // Verify recovery
            let instances = client.discover_services("failover-service").await.unwrap();
            assert_eq!(instances.len(), 5, "Should have 5 services after recovery in round {}", round);
        }
    }

    /// Test service discovery with very large service sets
    #[tokio::test]
    async fn test_large_scale_service_discovery() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry.clone());

        // Simulate a large microservices environment
        const NUM_SERVICES: usize = 2000;
        const NUM_SERVICE_TYPES: usize = 20;

        for service_type in 0..NUM_SERVICE_TYPES {
            for instance in 0..(NUM_SERVICES / NUM_SERVICE_TYPES) {
                let service = ServiceInfo {
                    id: format!("large-service-{}-{}", service_type, instance),
                    name: format!("large-service-{}", service_type),
                    address: format!("10.{}.{}", service_type, instance / 255),
                    port: (8000u16 + (instance % 1000) as u16),
                    tags: vec!["large-scale".to_string(), format!("type-{}", service_type)],
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("service_type".to_string(), service_type.to_string());
                        meta.insert("instance_id".to_string(), instance.to_string());
                        meta.insert("environment".to_string(), "large-scale-test".to_string());
                        meta
                    },
                    health_check: None,
                };
                client.register_service(service).await.unwrap();
            }
        }

        // Test discovery of each service type
        for service_type in 0..NUM_SERVICE_TYPES {
            let service_name = format!("large-service-{}", service_type);
            let instances = client.discover_services(&service_name).await.unwrap();
            assert_eq!(instances.len(), NUM_SERVICES / NUM_SERVICE_TYPES);

            // Verify metadata
            for instance in instances {
                assert_eq!(instance.metadata.get("service_type"), Some(&service_type.to_string()));
                assert_eq!(instance.metadata.get("environment"), Some(&"large-scale-test".to_string()));
            }
        }

        // Test load balancing across large service sets
        for service_type in 0..NUM_SERVICE_TYPES {
            let service_name = format!("large-service-{}", service_type);
            let mut selected_ids = HashSet::new();

            // Select instances multiple times
            for i in 0..(NUM_SERVICES / NUM_SERVICE_TYPES) {
                let instances = client.discover_services(&service_name).await.unwrap();
                let instance_index = i % instances.len();
                let instance = &instances[instance_index];
                selected_ids.insert(instance.id.clone());
            }

            // Should have selected different instances
            assert!(selected_ids.len() > 1, "Load balancing should distribute across instances for service type {}", service_type);
        }
    }
}