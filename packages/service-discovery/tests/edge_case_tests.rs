//! Edge Cases and Error Scenario Tests for Service Discovery
//!
//! This module tests unusual scenarios, error conditions, and edge cases
//! to ensure robustness of the service discovery system.

use service_discovery::{
    InMemoryRegistry, ServiceDiscoveryClient, ServiceInfo,
    HealthCheckConfig, HealthCheckType, RegistryError, HealthStatus,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    /// Test service registration with extreme metadata
    #[tokio::test]
    async fn test_extreme_metadata() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Test with empty metadata
        let service_empty = ServiceInfo {
            id: "empty-meta-service".to_string(),
            name: "empty-meta".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };
        client.register_service(service_empty).await.unwrap();

        // Test with very large metadata values
        let mut large_metadata = HashMap::new();
        large_metadata.insert("large_value".to_string(), "x".repeat(10000)); // 10KB string

        let service_large = ServiceInfo {
            id: "large-meta-service".to_string(),
            name: "large-meta".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8081,
            tags: vec![],
            metadata: large_metadata,
            health_check: None,
        };
        client.register_service(service_large).await.unwrap();

        // Test with special characters in metadata
        let mut special_metadata = HashMap::new();
        special_metadata.insert("special_chars".to_string(), "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string());
        special_metadata.insert("unicode".to_string(), "🚀🔥💯🌟🎉".to_string());
        special_metadata.insert("multiline".to_string(), "line1\nline2\tline3".to_string());

        let service_special = ServiceInfo {
            id: "special-meta-service".to_string(),
            name: "special-meta".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8082,
            tags: vec![],
            metadata: special_metadata,
            health_check: None,
        };
        client.register_service(service_special).await.unwrap();

        // Verify all services are discoverable
        let services = vec!["empty-meta", "large-meta", "special-meta"];
        for service_name in services {
            let instances = client.discover_services(service_name).await.unwrap();
            assert_eq!(instances.len(), 1);
        }
    }

    /// Test service registration with extreme tag scenarios
    #[tokio::test]
    async fn test_extreme_tags() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Service with no tags
        let service_no_tags = ServiceInfo {
            id: "no-tags-service".to_string(),
            name: "no-tags".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };
        client.register_service(service_no_tags).await.unwrap();

        // Service with many tags
        let many_tags: Vec<String> = (0..100).map(|i| format!("tag-{}", i)).collect();
        let service_many_tags = ServiceInfo {
            id: "many-tags-service".to_string(),
            name: "many-tags".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8081,
            tags: many_tags,
            metadata: HashMap::new(),
            health_check: None,
        };
        client.register_service(service_many_tags).await.unwrap();

        // Service with duplicate tags
        let service_dup_tags = ServiceInfo {
            id: "dup-tags-service".to_string(),
            name: "dup-tags".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8082,
            tags: vec!["tag1".to_string(), "tag1".to_string(), "tag2".to_string()],
            metadata: HashMap::new(),
            health_check: None,
        };
        client.register_service(service_dup_tags).await.unwrap();

        // Verify all services work
        let instances_no_tags = client.discover_services("no-tags").await.unwrap();
        assert_eq!(instances_no_tags.len(), 1);
        assert_eq!(instances_no_tags[0].tags.len(), 0);

        let instances_many_tags = client.discover_services("many-tags").await.unwrap();
        assert_eq!(instances_many_tags.len(), 1);
        assert_eq!(instances_many_tags[0].tags.len(), 100);

        let instances_dup_tags = client.discover_services("dup-tags").await.unwrap();
        assert_eq!(instances_dup_tags.len(), 1);
        // Note: duplicate tags are allowed in the current implementation
    }

    /// Test network address edge cases
    #[tokio::test]
    async fn test_network_address_edge_cases() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Test various valid IP addresses
        let addresses = vec![
            "0.0.0.0",      // All interfaces
            "127.0.0.1",    // Localhost
            "10.0.0.1",     // Private network
            "192.168.1.1",  // Private network
            "172.16.0.1",   // Private network
            "255.255.255.255", // Broadcast
        ];

        for (i, address) in addresses.iter().enumerate() {
            let service = ServiceInfo {
                id: format!("address-test-{}", i),
                name: "address-test".to_string(),
                address: address.to_string(),
                port: 8080 + i as u16,
                tags: vec![],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // Verify all services are registered
        let instances = client.discover_services("address-test").await.unwrap();
        assert_eq!(instances.len(), addresses.len());
    }

    /// Test port number edge cases
    #[tokio::test]
    async fn test_port_edge_cases() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Test various port numbers
        let ports = vec![
            1,      // Minimum valid port
            80,     // HTTP
            443,    // HTTPS
            8080,   // Common application port
            65535,  // Maximum valid port
        ];

        for (i, port) in ports.iter().enumerate() {
            let service = ServiceInfo {
                id: format!("port-test-{}", i),
                name: "port-test".to_string(),
                address: "127.0.0.1".to_string(),
                port: *port,
                tags: vec![],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // Verify all services are registered
        let instances = client.discover_services("port-test").await.unwrap();
        assert_eq!(instances.len(), ports.len());

        // Verify ports are stored correctly
        for instance in instances {
            let port_num: u16 = instance.id.strip_prefix("port-test-").unwrap().parse().unwrap();
            assert_eq!(instance.port, ports[port_num as usize]);
        }
    }

    /// Test service ID edge cases
    #[tokio::test]
    async fn test_service_id_edge_cases() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Test various service IDs
        let service_ids = vec![
            "simple-id",
            "id_with_underscores",
            "id-with-dashes",
            "id.with.dots",
            "123numeric",
            "mixed123case",
            "UPPERCASE",
        ];

        for (i, service_id) in service_ids.iter().enumerate() {
            let service = ServiceInfo {
                id: service_id.to_string(),
                name: format!("id-test-{}", i),
                address: "127.0.0.1".to_string(),
                port: 8080 + i as u16,
                tags: vec![],
                metadata: HashMap::new(),
                health_check: None,
            };

            // Should not fail for these IDs
            client.register_service(service).await.unwrap();

            // If registration succeeded, verify we can discover it
            let instances = client.discover_services(&format!("id-test-{}", i)).await.unwrap();
            assert_eq!(instances.len(), 1);
            assert_eq!(instances[0].id, *service_id);
        }
    }

    /// Test concurrent registration and discovery with conflicts
    #[tokio::test]
    async fn test_concurrent_registration_conflicts() {
        let registry = Arc::new(InMemoryRegistry::new());

        // Create multiple clients trying to register the same service ID
        const NUM_CLIENTS: usize = 5;
        let mut handles = vec![];

        for i in 0..NUM_CLIENTS {
            let registry_clone = registry.clone();
            let handle = tokio::spawn(async move {
                let client = ServiceDiscoveryClient::new(registry_clone);
                let service = ServiceInfo {
                    id: "conflict-service".to_string(), // Same ID for all
                    name: format!("conflict-service-{}", i),
                    address: format!("10.0.0.{}", i + 1),
                    port: 8080,
                    tags: vec![],
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("client_id".to_string(), i.to_string());
                        meta
                    },
                    health_check: None,
                };

                client.register_service(service).await
            });
            handles.push(handle);
        }

        // Wait for all attempts
        let mut success_count = 0;
        let mut failure_count = 0;

        for handle in handles {
            let result = handle.await.unwrap();
            match result {
                Ok(_) => success_count += 1,
                Err(_) => failure_count += 1,
            }
        }

        // Depending on the registry implementation, either:
        // - All succeed (registry allows overwrites)
        // - Only one succeeds (registry prevents conflicts)
        // - Some other behavior
        assert!(success_count + failure_count == NUM_CLIENTS);

        // At least one service should be discoverable
        let client = ServiceDiscoveryClient::new(registry.clone());
        let instances = client.discover_services("conflict-service-0").await.unwrap();
        assert!(instances.len() >= 1, "At least one service should be registered");
    }

    /// Test service discovery with empty results
    #[tokio::test]
    async fn test_empty_discovery_results() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Try to discover non-existent services
        let non_existent_names = vec![
            "",
            "non-existent",
            "service-that-does-not-exist",
            "another-fake-service",
            "🤖-emoji-service",
        ];

        for service_name in non_existent_names {
            let result = client.discover_services(service_name).await;
            // The implementation returns an error for non-existent services
            assert!(result.is_err());
            if let Err(RegistryError::ServiceNotFound { service_name: name }) = result {
                assert_eq!(name, service_name);
            }
        }

        // Register one service, then try to discover others
        let service = ServiceInfo {
            id: "single-service".to_string(),
            name: "single-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };
        client.register_service(service).await.unwrap();

        // This should return results
        let instances = client.discover_services("single-service").await.unwrap();
        assert_eq!(instances.len(), 1);

        // These should still return error
        let result = client.discover_services("other-service").await;
        assert!(result.is_err());
    }

    /// Test service instance selection edge cases
    #[tokio::test]
    async fn test_instance_selection_edge_cases() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Try to discover from non-existent service
        let result = client.discover_services("non-existent").await;
        assert!(result.is_err(), "Should return error for non-existent service");

        // Register services
        for i in 1..=3 {
            let service = ServiceInfo {
                id: format!("selection-test-{}", i),
                name: "selection-test".to_string(),
                address: format!("10.0.0.{}", i),
                port: 8080,
                tags: vec![],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        // Discover instances multiple times
        for _ in 0..5 {
            let instances = client.discover_services("selection-test").await.unwrap();
            assert_eq!(instances.len(), 3);
            // All instances should be returned
        }

        // Remove all instances
        for i in 1..=3 {
            client.deregister_service(&format!("selection-test-{}", i)).await.unwrap();
        }

        // Try to discover again (should return error)
        let result = client.discover_services("selection-test").await;
        assert!(result.is_err(), "Should return error when no instances available");
    }

    /// Test health check configuration edge cases
    #[tokio::test]
    async fn test_health_check_config_edge_cases() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Test with various health check configurations
        let configs = vec![
            None, // No health check
            Some(HealthCheckConfig {
                check_type: HealthCheckType::Http,
                endpoint: None,
                interval_seconds: 0, // Invalid interval
                timeout_seconds: 0,  // Invalid timeout
                deregister_after_seconds: 0, // Invalid deregister time
            }),
            Some(HealthCheckConfig {
                check_type: HealthCheckType::Tcp,
                endpoint: Some("/health".to_string()),
                interval_seconds: 3600, // Very long interval
                timeout_seconds: 300,   // Long timeout
                deregister_after_seconds: 86400, // Very long deregister time
            }),
        ];

        for (i, health_check) in configs.iter().enumerate() {
            let service = ServiceInfo {
                id: format!("health-config-test-{}", i),
                name: "health-config-test".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8080 + i as u16,
                tags: vec![],
                metadata: HashMap::new(),
                health_check: health_check.clone(),
            };

            // Should not fail regardless of health check config
            client.register_service(service).await.unwrap();
        }

        // Verify all services are registered
        let instances = client.discover_services("health-config-test").await.unwrap();
        assert_eq!(instances.len(), configs.len());
    }

    /// Test rapid state changes (register -> deregister -> register)
    #[tokio::test]
    async fn test_rapid_state_changes() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        const SERVICE_ID: &str = "rapid-change-service";

        // Perform rapid register/deregister cycles
        for cycle in 0..20 {
            // Register
            let service = ServiceInfo {
                id: SERVICE_ID.to_string(),
                name: "rapid-change".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8080,
                tags: vec![],
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("cycle".to_string(), cycle.to_string());
                    meta
                },
                health_check: None,
            };
            client.register_service(service).await.unwrap();

            // Immediate discovery
            let instances = client.discover_services("rapid-change").await.unwrap();
            assert_eq!(instances.len(), 1);
            assert_eq!(instances[0].metadata.get("cycle"), Some(&cycle.to_string()));

            // Immediate deregister
            client.deregister_service(SERVICE_ID).await.unwrap();

            // Immediate verification of deregistration (should return error)
            let result = client.discover_services("rapid-change").await;
            assert!(result.is_err());
        }
    }

    /// Test memory and resource cleanup
    #[tokio::test]
    async fn test_resource_cleanup() {
        let registry = Arc::new(InMemoryRegistry::new());
        let client = ServiceDiscoveryClient::new(registry);

        // Create many services with resources
        const NUM_SERVICES: usize = 100;

        for i in 0..NUM_SERVICES {
            let service = ServiceInfo {
                id: format!("cleanup-test-{}", i),
                name: "cleanup-test".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8000 + (i % 1000) as u16,
                tags: vec!["cleanup".to_string(); 10], // Many tags
                metadata: {
                    let mut meta = HashMap::new();
                    for j in 0..10 {
                        meta.insert(format!("key-{}", j), format!("value-{}", j).repeat(10));
                    }
                    meta
                },
                health_check: Some(HealthCheckConfig {
                    check_type: HealthCheckType::Http,
                    endpoint: Some("/health".to_string()),
                    interval_seconds: 30,
                    timeout_seconds: 5,
                    deregister_after_seconds: 300,
                }),
            };
            client.register_service(service).await.unwrap();
        }

        // Verify all are there
        let instances = client.discover_services("cleanup-test").await.unwrap();
        assert_eq!(instances.len(), NUM_SERVICES);

        // Clean up all services
        for i in 0..NUM_SERVICES {
            client.deregister_service(&format!("cleanup-test-{}", i)).await.unwrap();
        }

        // Verify all are gone (should return error)
        let result = client.discover_services("cleanup-test").await;
        assert!(result.is_err());

        // Try to allocate again (test for memory leaks)
        for i in 0..NUM_SERVICES / 2 {
            let service = ServiceInfo {
                id: format!("cleanup-test-recreate-{}", i),
                name: "cleanup-test".to_string(),
                address: "127.0.0.1".to_string(),
                port: 9000 + (i % 1000) as u16,
                tags: vec!["cleanup".to_string()],
                metadata: HashMap::new(),
                health_check: None,
            };
            client.register_service(service).await.unwrap();
        }

        let instances = client.discover_services("cleanup-test").await.unwrap();
        assert_eq!(instances.len(), NUM_SERVICES / 2);
    }
}