//! Consul Service Registry Implementation
//!
//! This module provides a Consul-based implementation of the ServiceRegistry
//! trait, integrating with HashiCorp Consul for service discovery.

use super::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Consul service registry implementation
pub struct ConsulRegistry {
    client: Client,
    base_url: String,
    datacenter: Option<String>,
    token: Option<String>,
}

impl ConsulRegistry {
    /// Create a new Consul registry with default settings
    pub fn new() -> Self {
        Self::with_config(ConsulConfig::default())
    }

    /// Create a new Consul registry with custom configuration
    pub fn with_config(config: ConsulConfig) -> Self {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(30));

        if let Some(timeout) = config.timeout_seconds {
            client_builder = client_builder.timeout(Duration::from_secs(timeout));
        }

        let client = client_builder.build().expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.base_url,
            datacenter: config.datacenter,
            token: config.token,
        }
    }

    /// Build request URL for Consul API
    fn build_url(&self, path: &str) -> String {
        let mut url = format!("{}{}", self.base_url, path);

        if let Some(dc) = &self.datacenter {
            url.push_str(&format!("?dc={}", dc));
        }

        url
    }

    /// Add authorization header if token is provided
    fn add_auth_header(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.token {
            request.header("X-Consul-Token", token)
        } else {
            request
        }
    }

    /// Convert ServiceInfo to Consul service registration payload
    fn to_consul_service(&self, service: &ServiceInfo) -> ConsulServiceRegistration {
        let check = service.health_check.as_ref().map(|hc| {
            match hc.check_type {
                HealthCheckType::Http => ConsulCheck {
                    http: Some(format!("http://{}:{}{}", service.address, service.port, hc.endpoint.as_deref().unwrap_or("/health"))),
                    interval: Some(format!("{}s", hc.interval_seconds)),
                    timeout: Some(format!("{}s", hc.timeout_seconds)),
                    deregister_critical_service_after: Some(format!("{}s", hc.deregister_after_seconds)),
                    ..Default::default()
                },
                HealthCheckType::Tcp => ConsulCheck {
                    tcp: Some(format!("{}:{}", service.address, service.port)),
                    interval: Some(format!("{}s", hc.interval_seconds)),
                    timeout: Some(format!("{}s", hc.timeout_seconds)),
                    deregister_critical_service_after: Some(format!("{}s", hc.deregister_after_seconds)),
                    ..Default::default()
                },
                HealthCheckType::Ttl => ConsulCheck {
                    ttl: Some(format!("{}s", hc.interval_seconds)),
                    deregister_critical_service_after: Some(format!("{}s", hc.deregister_after_seconds)),
                    ..Default::default()
                },
            }
        });

        ConsulServiceRegistration {
            id: Some(service.id.clone()),
            name: service.name.clone(),
            address: service.address.clone(),
            port: Some(service.port),
            tags: Some(service.tags.clone()),
            meta: Some(service.metadata.clone()),
            check,
        }
    }

    /// Convert Consul service to ServiceInstance
    fn from_consul_service(&self, consul_service: &ConsulService) -> ServiceInstance {
        ServiceInstance {
            id: consul_service.service.id.clone().unwrap_or_default(),
            name: consul_service.service.name.clone(),
            address: consul_service.service.address.clone(),
            port: consul_service.service.port.unwrap_or(0),
            tags: consul_service.service.tags.clone().unwrap_or_default(),
            metadata: consul_service.service.meta.clone().unwrap_or_default(),
            health_status: self.map_consul_health_status(&consul_service.checks),
            last_health_check: Some(chrono::Utc::now()),
        }
    }

    /// Map Consul health checks to our HealthStatus
    fn map_consul_health_status(&self, checks: &[ConsulHealthCheck]) -> HealthStatus {
        if checks.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut has_passing = false;
        let mut has_warning = false;
        let mut has_critical = false;

        for check in checks {
            match check.status.as_str() {
                "passing" => has_passing = true,
                "warning" => has_warning = true,
                "critical" => has_critical = true,
                _ => {}
            }
        }

        if has_critical {
            HealthStatus::Critical
        } else if has_warning {
            HealthStatus::Warning
        } else if has_passing {
            HealthStatus::Passing
        } else {
            HealthStatus::Unknown
        }
    }
}

#[async_trait]
impl ServiceRegistry for ConsulRegistry {
    async fn register(&self, service: ServiceInfo) -> Result<(), RegistryError> {
        let consul_service = self.to_consul_service(&service);
        let url = self.build_url("/v1/agent/service/register");

        let response = self
            .add_auth_header(self.client.put(&url))
            .json(&consul_service)
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to register service: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("Registration failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    async fn deregister(&self, service_id: &str) -> Result<(), RegistryError> {
        let url = self.build_url(&format!("/v1/agent/service/deregister/{}", service_id));

        let response = self
            .add_auth_header(self.client.put(&url))
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to deregister service: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("Deregistration failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    async fn discover(&self, service_name: &str) -> Result<Vec<ServiceInstance>, RegistryError> {
        let url = self.build_url(&format!("/v1/health/service/{}", service_name));

        let response = self
            .add_auth_header(self.client.get(&url))
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
                backend: "consul".to_string(),
                message: format!("Discovery failed with status {}: {}", status, body),
            });
        }

        let consul_services: Vec<ConsulService> = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse Consul response: {}", e),
            })?;

        let instances: Vec<ServiceInstance> = consul_services
            .iter()
            .map(|cs| self.from_consul_service(cs))
            .collect();

        Ok(instances)
    }

    async fn get_service(&self, service_id: &str) -> Result<ServiceInstance, RegistryError> {
        // First try to get all services and find the one with matching ID
        // Note: Consul doesn't have a direct API to get a service by ID,
        // so we need to discover all services and filter
        let url = self.build_url("/v1/agent/services");

        let response = self
            .add_auth_header(self.client.get(&url))
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to get services: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("Get service failed with status {}: {}", status, body),
            });
        }

        let services: HashMap<String, ConsulServiceInfo> = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse services response: {}", e),
            })?;

        let service_info = services.get(service_id).ok_or_else(|| {
            RegistryError::ServiceNotFound {
                service_name: service_id.to_string(),
            }
        })?;

        // Get health checks for this service
        let health_url = self.build_url(&format!("/v1/health/service/{}", service_info.name));
        let health_response = self
            .add_auth_header(self.client.get(&health_url))
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to get health checks: {}", e),
            })?;

        let health_checks: Vec<ConsulService> = if health_response.status().is_success() {
            health_response.json().await.unwrap_or_default()
        } else {
            vec![]
        };

        // Find the matching service with health info
        let consul_service = health_checks
            .iter()
            .find(|cs| cs.service.id.as_ref() == Some(&service_id.to_string()))
            .ok_or_else(|| RegistryError::ServiceNotFound {
                service_name: service_id.to_string(),
            })?;

        Ok(self.from_consul_service(consul_service))
    }

    async fn health_check(&self, service_id: &str) -> Result<HealthStatus, RegistryError> {
        let url = self.build_url(&format!("/v1/agent/checks"));

        let response = self
            .add_auth_header(self.client.get(&url))
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to get health checks: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("Health check failed with status {}: {}", status, body),
            });
        }

        let checks: HashMap<String, ConsulCheckStatus> = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse health checks: {}", e),
            })?;

        // Find the service check
        let service_check_key = format!("service:{}", service_id);
        let check = checks.get(&service_check_key).ok_or_else(|| {
            RegistryError::ServiceNotFound {
                service_name: service_id.to_string(),
            }
        })?;

        match check.status.as_str() {
            "passing" => Ok(HealthStatus::Passing),
            "warning" => Ok(HealthStatus::Warning),
            "critical" => Ok(HealthStatus::Critical),
            _ => Ok(HealthStatus::Unknown),
        }
    }

    async fn watch(&self, service_name: &str) -> Result<Box<dyn ServiceWatcher>, RegistryError> {
        Ok(Box::new(ConsulWatcher::new(
            self.client.clone(),
            self.base_url.clone(),
            service_name.to_string(),
            self.datacenter.clone(),
            self.token.clone(),
        )))
    }
}

/// Configuration for Consul registry
#[derive(Debug, Clone)]
pub struct ConsulConfig {
    pub base_url: String,
    pub datacenter: Option<String>,
    pub token: Option<String>,
    pub timeout_seconds: Option<u64>,
}

impl Default for ConsulConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8500".to_string(),
            datacenter: None,
            token: None,
            timeout_seconds: Some(30),
        }
    }
}

/// Consul service registration payload
#[derive(Debug, Serialize, Deserialize)]
struct ConsulServiceRegistration {
    id: Option<String>,
    name: String,
    address: String,
    port: Option<u16>,
    tags: Option<Vec<String>>,
    meta: Option<HashMap<String, String>>,
    check: Option<ConsulCheck>,
}

/// Consul check configuration
#[derive(Debug, Serialize, Deserialize, Default)]
struct ConsulCheck {
    http: Option<String>,
    tcp: Option<String>,
    ttl: Option<String>,
    interval: Option<String>,
    timeout: Option<String>,
    deregister_critical_service_after: Option<String>,
}

/// Consul service response
#[derive(Debug, Deserialize)]
struct ConsulService {
    service: ConsulServiceInfo,
    checks: Vec<ConsulHealthCheck>,
}

/// Consul service information
#[derive(Debug, Deserialize)]
struct ConsulServiceInfo {
    id: Option<String>,
    name: String,
    address: String,
    port: Option<u16>,
    tags: Option<Vec<String>>,
    meta: Option<HashMap<String, String>>,
}

/// Consul health check
#[derive(Debug, Deserialize)]
struct ConsulHealthCheck {
    status: String,
}

/// Consul check status
#[derive(Debug, Deserialize)]
struct ConsulCheckStatus {
    status: String,
}

/// Watcher for Consul service changes
pub struct ConsulWatcher {
    client: Client,
    base_url: String,
    service_name: String,
    datacenter: Option<String>,
    token: Option<String>,
    last_index: Option<String>,
}

impl ConsulWatcher {
    fn new(
        client: Client,
        base_url: String,
        service_name: String,
        datacenter: Option<String>,
        token: Option<String>,
    ) -> Self {
        Self {
            client,
            base_url,
            service_name,
            datacenter,
            token,
            last_index: None,
        }
    }

    fn build_url(&self) -> String {
        let mut url = format!("{}/v1/health/service/{}", self.base_url, self.service_name);

        let mut params = Vec::new();

        if let Some(dc) = &self.datacenter {
            params.push(format!("dc={}", dc));
        }

        if let Some(index) = &self.last_index {
            params.push(format!("index={}", index));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        url
    }

    fn add_auth_header(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.token {
            request.header("X-Consul-Token", token)
        } else {
            request
        }
    }
}

#[async_trait]
impl ServiceWatcher for ConsulWatcher {
    async fn next(&mut self) -> Option<ServiceDiscoveryEvent> {
        let url = self.build_url();

        let response = self
            .add_auth_header(self.client.get(&url))
            .send()
            .await
            .ok()?;

        // Update last index for next request
        if let Some(index) = response.headers().get("x-consul-index") {
            self.last_index = index.to_str().ok().map(|s| s.to_string());
        }

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return None; // No changes
        }

        if !response.status().is_success() {
            return None; // Error, but we'll retry
        }

        let consul_services: Vec<ConsulService> = response.json().await.ok()?;

        // For simplicity, we'll emit a ServiceUpdated event
        // In a more sophisticated implementation, we'd track changes
        Some(ServiceDiscoveryEvent::ServiceUpdated {
            service_name: self.service_name.clone(),
            instances: consul_services.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consul_config_default() {
        let config = ConsulConfig::default();
        assert_eq!(config.base_url, "http://localhost:8500");
        assert!(config.datacenter.is_none());
        assert!(config.token.is_none());
        assert_eq!(config.timeout_seconds, Some(30));
    }

    #[test]
    fn test_build_url_without_datacenter() {
        let registry = ConsulRegistry::new();
        let url = registry.build_url("/v1/agent/services");
        assert_eq!(url, "http://localhost:8500/v1/agent/services");
    }

    #[test]
    fn test_build_url_with_datacenter() {
        let config = ConsulConfig {
            datacenter: Some("dc1".to_string()),
            ..Default::default()
        };
        let registry = ConsulRegistry::with_config(config);
        let url = registry.build_url("/v1/agent/services");
        assert_eq!(url, "http://localhost:8500/v1/agent/services?dc=dc1");
    }

    #[tokio::test]
    async fn test_consul_registry_creation() {
        let registry = ConsulRegistry::new();
        // Just test that it can be created without panicking
        assert!(true);
    }
}