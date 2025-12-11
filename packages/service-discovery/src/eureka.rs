//! Eureka Service Registry Implementation
//!
//! This module provides an Eureka-based implementation of the ServiceRegistry
//! trait, integrating with Netflix Eureka for service discovery.

use super::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Eureka service registry implementation
pub struct EurekaRegistry {
    client: Client,
    base_url: String,
    app_name: String,
    instance_id: Option<String>,
}

impl EurekaRegistry {
    /// Create a new Eureka registry with default settings
    pub fn new(base_url: &str, app_name: &str) -> Self {
        Self::with_config(EurekaConfig {
            base_url: base_url.to_string(),
            app_name: app_name.to_string(),
            instance_id: None,
            timeout_seconds: Some(30),
        })
    }

    /// Create a new Eureka registry with custom configuration
    pub fn with_config(config: EurekaConfig) -> Self {
        let client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds.unwrap_or(30)));

        let client = client_builder.build().expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.base_url.trim_end_matches('/').to_string(),
            app_name: config.app_name,
            instance_id: config.instance_id,
        }
    }

    /// Build the full URL for Eureka API endpoints
    fn build_url(&self, path: &str) -> String {
        format!("{}/eureka{}", self.base_url, path)
    }

    /// Convert ServiceInfo to Eureka instance registration
    fn to_eureka_instance(&self, service: &ServiceInfo) -> EurekaInstance {
        let instance_id = self.instance_id.clone()
            .unwrap_or_else(|| format!("{}:{}", service.address, service.port));

        let mut metadata = HashMap::new();
        for (key, value) in &service.metadata {
            metadata.insert(key.clone(), value.clone());
        }

        // Add tags as metadata
        for (i, tag) in service.tags.iter().enumerate() {
            metadata.insert(format!("tag{}", i), tag.clone());
        }

        EurekaInstance {
            instance_id,
            host_name: service.address.clone(),
            app: self.app_name.clone(),
            ip_addr: service.address.clone(),
            vip_address: service.name.clone(),
            secure_vip_address: service.name.clone(),
            status: "UP".to_string(),
            port: Some(EurekaPort {
                value: service.port,
                enabled: true,
            }),
            secure_port: None,
            home_page_url: Some(format!("http://{}:{}/", service.address, service.port)),
            status_page_url: Some(format!("http://{}:{}/status", service.address, service.port)),
            health_check_url: service.health_check.as_ref().map(|hc| {
                match hc.check_type {
                    HealthCheckType::Http => format!("http://{}:{}{}",
                        service.address, service.port, hc.endpoint.as_deref().unwrap_or("/health")),
                    _ => format!("http://{}:{}/health", service.address, service.port),
                }
            }),
            data_center_info: EurekaDataCenterInfo {
                class: "com.netflix.appinfo.InstanceInfo$DefaultDataCenterInfo".to_string(),
                name: "MyOwn".to_string(),
            },
            lease_info: service.health_check.as_ref().map(|hc| EurekaLeaseInfo {
                renewal_interval_in_secs: hc.interval_seconds,
                duration_in_secs: hc.deregister_after_seconds,
            }),
            metadata,
        }
    }

    /// Convert Eureka instance to ServiceInstance
    fn from_eureka_instance(&self, instance: &EurekaInstance) -> ServiceInstance {
        let mut tags = Vec::new();
        let mut metadata = HashMap::new();

        for (key, value) in &instance.metadata {
            if key.starts_with("tag") {
                tags.push(value.clone());
            } else {
                metadata.insert(key.clone(), value.clone());
            }
        }

        ServiceInstance {
            id: instance.instance_id.clone(),
            name: instance.app.clone(),
            address: instance.ip_addr.clone(),
            port: instance.port.as_ref().map(|p| p.value).unwrap_or(0),
            tags,
            metadata,
            health_status: match instance.status.as_str() {
                "UP" => HealthStatus::Passing,
                "DOWN" => HealthStatus::Critical,
                "OUT_OF_SERVICE" => HealthStatus::Warning,
                _ => HealthStatus::Unknown,
            },
            last_health_check: Some(chrono::Utc::now()),
        }
    }
}

#[async_trait]
impl ServiceRegistry for EurekaRegistry {
    async fn register(&self, service: ServiceInfo) -> Result<(), RegistryError> {
        let instance = self.to_eureka_instance(&service);
        let url = self.build_url(&format!("/apps/{}", self.app_name));

        let payload = EurekaRegistration {
            instance,
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to register service: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "eureka".to_string(),
                message: format!("Registration failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    async fn deregister(&self, service_id: &str) -> Result<(), RegistryError> {
        let url = self.build_url(&format!("/apps/{}/{}", self.app_name, service_id));

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to deregister service: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "eureka".to_string(),
                message: format!("Deregistration failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    async fn discover(&self, service_name: &str) -> Result<Vec<ServiceInstance>, RegistryError> {
        let url = self.build_url(&format!("/apps/{}", service_name));

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to discover services: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(RegistryError::ServiceNotFound {
                    service_name: service_name.to_string(),
                });
            }
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "eureka".to_string(),
                message: format!("Discovery failed with status {}: {}", status, body),
            });
        }

        let eureka_response: EurekaApplicationResponse = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse Eureka response: {}", e),
            })?;

        let instances: Vec<ServiceInstance> = eureka_response
            .application
            .instance
            .into_iter()
            .map(|instance| self.from_eureka_instance(&instance))
            .collect();

        Ok(instances)
    }

    async fn get_service(&self, service_id: &str) -> Result<ServiceInstance, RegistryError> {
        let url = self.build_url(&format!("/apps/{}/{}", self.app_name, service_id));

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to get service: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(RegistryError::ServiceNotFound {
                    service_name: service_id.to_string(),
                });
            }
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "eureka".to_string(),
                message: format!("Get service failed with status {}: {}", status, body),
            });
        }

        let eureka_response: EurekaInstanceResponse = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse Eureka instance response: {}", e),
            })?;

        Ok(self.from_eureka_instance(&eureka_response.instance))
    }

    async fn health_check(&self, service_id: &str) -> Result<HealthStatus, RegistryError> {
        // Eureka doesn't have a direct health check endpoint like Consul
        // We need to get the instance and check its status
        let service = self.get_service(service_id).await?;
        Ok(service.health_status)
    }

    async fn watch(&self, service_name: &str) -> Result<Box<dyn ServiceWatcher>, RegistryError> {
        Ok(Box::new(EurekaWatcher::new(
            self.client.clone(),
            self.base_url.clone(),
            service_name.to_string(),
        )))
    }
}

/// Configuration for Eureka registry
#[derive(Debug, Clone)]
pub struct EurekaConfig {
    pub base_url: String,
    pub app_name: String,
    pub instance_id: Option<String>,
    pub timeout_seconds: Option<u64>,
}

impl Default for EurekaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8761".to_string(),
            app_name: "VELA-SERVICE".to_string(),
            instance_id: None,
            timeout_seconds: Some(30),
        }
    }
}

/// Eureka instance information
#[derive(Debug, Serialize, Deserialize)]
struct EurekaInstance {
    instance_id: String,
    host_name: String,
    app: String,
    ip_addr: String,
    vip_address: String,
    secure_vip_address: String,
    status: String,
    port: Option<EurekaPort>,
    secure_port: Option<EurekaPort>,
    home_page_url: Option<String>,
    status_page_url: Option<String>,
    health_check_url: Option<String>,
    data_center_info: EurekaDataCenterInfo,
    lease_info: Option<EurekaLeaseInfo>,
    metadata: HashMap<String, String>,
}

/// Eureka port information
#[derive(Debug, Serialize, Deserialize)]
struct EurekaPort {
    #[serde(rename = "$")]
    value: u16,
    #[serde(rename = "@enabled")]
    enabled: bool,
}

/// Eureka data center information
#[derive(Debug, Serialize, Deserialize)]
struct EurekaDataCenterInfo {
    #[serde(rename = "@class")]
    class: String,
    name: String,
}

/// Eureka lease information
#[derive(Debug, Serialize, Deserialize)]
struct EurekaLeaseInfo {
    renewal_interval_in_secs: u64,
    duration_in_secs: u64,
}

/// Eureka registration payload
#[derive(Debug, Serialize)]
struct EurekaRegistration {
    instance: EurekaInstance,
}

/// Eureka application response
#[derive(Debug, Deserialize)]
struct EurekaApplicationResponse {
    application: EurekaApplication,
}

/// Eureka application
#[derive(Debug, Deserialize)]
struct EurekaApplication {
    name: String,
    instance: Vec<EurekaInstance>,
}

/// Eureka instance response
#[derive(Debug, Deserialize)]
struct EurekaInstanceResponse {
    instance: EurekaInstance,
}

/// Watcher for Eureka service changes
pub struct EurekaWatcher {
    client: Client,
    base_url: String,
    service_name: String,
    last_timestamp: Option<u64>,
}

impl EurekaWatcher {
    fn new(client: Client, base_url: String, service_name: String) -> Self {
        Self {
            client,
            base_url,
            service_name,
            last_timestamp: None,
        }
    }

    fn build_url(&self) -> String {
        format!("{}/eureka/apps/{}", self.base_url, self.service_name)
    }
}

#[async_trait]
impl ServiceWatcher for EurekaWatcher {
    async fn next(&mut self) -> Option<ServiceDiscoveryEvent> {
        let url = self.build_url();

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None; // Error, but we'll retry
        }

        let eureka_response: EurekaApplicationResponse = response.json().await.ok()?;

        let current_count = eureka_response.application.instance.len();
        let previous_count = self.last_timestamp.unwrap_or(0) as usize;

        self.last_timestamp = Some(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs());

        if current_count != previous_count {
            Some(ServiceDiscoveryEvent::ServiceUpdated {
                service_name: self.service_name.clone(),
                instances: current_count,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eureka_config_default() {
        let config = EurekaConfig::default();
        assert_eq!(config.base_url, "http://localhost:8761");
        assert_eq!(config.app_name, "VELA-SERVICE");
        assert!(config.instance_id.is_none());
        assert_eq!(config.timeout_seconds, Some(30));
    }

    #[test]
    fn test_build_url() {
        let registry = EurekaRegistry::new("http://localhost:8761", "test-app");
        let url = registry.build_url("/apps/test-app");
        assert_eq!(url, "http://localhost:8761/eureka/apps/test-app");
    }

    #[test]
    fn test_eureka_registry_creation() {
        let registry = EurekaRegistry::new("http://localhost:8761", "test-app");
        // Just test that it can be created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_to_eureka_instance_conversion() {
        let registry = EurekaRegistry::new("http://localhost:8761", "test-app");

        let service = ServiceInfo {
            id: "test-service-1".to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec!["web".to_string()],
            metadata: HashMap::from([("version".to_string(), "1.0.0".to_string())]),
            health_check: Some(HealthCheckConfig {
                check_type: HealthCheckType::Http,
                endpoint: Some("/health".to_string()),
                interval_seconds: 30,
                timeout_seconds: 5,
                deregister_after_seconds: 300,
            }),
        };

        let instance = registry.to_eureka_instance(&service);

        assert_eq!(instance.app, "test-app");
        assert_eq!(instance.ip_addr, "127.0.0.1");
        assert_eq!(instance.port.as_ref().unwrap().value, 8080);
        assert_eq!(instance.status, "UP");
        assert!(instance.metadata.contains_key("version"));
        assert!(instance.metadata.contains_key("tag0"));
    }
}