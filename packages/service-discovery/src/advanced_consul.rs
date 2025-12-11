//! Advanced Consul Service Registry Implementation
//!
//! This module provides an advanced Consul-based implementation with service mesh
//! integration, ACL management, multi-datacenter support, and enterprise features.

use super::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Advanced Consul service registry with enterprise features
pub struct AdvancedConsulRegistry {
    client: Client,
    base_url: String,
    datacenter: Option<String>,
    token: Option<String>,
    service_mesh_enabled: bool,
    kv_store: Arc<RwLock<HashMap<String, String>>>,
    intentions_cache: Arc<RwLock<HashMap<String, Vec<ServiceIntention>>>>,
}

impl AdvancedConsulRegistry {
    /// Create a new advanced Consul registry with default settings
    pub fn new() -> Self {
        Self::with_config(AdvancedConsulConfig::default())
    }

    /// Create a new advanced Consul registry with custom configuration
    pub fn with_config(config: AdvancedConsulConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds.unwrap_or(30)))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.base_url,
            datacenter: config.datacenter,
            token: config.token,
            service_mesh_enabled: config.service_mesh_enabled,
            kv_store: Arc::new(RwLock::new(HashMap::new())),
            intentions_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Build request URL for Consul API
    fn build_url(&self, path: &str) -> String {
        let mut url = format!("{}{}", self.base_url, path);
        let mut params = Vec::new();

        if let Some(dc) = &self.datacenter {
            params.push(format!("dc={}", dc));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
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

    /// Convert ServiceInfo to advanced Consul service registration
    fn to_advanced_consul_service(&self, service: &ServiceInfo) -> AdvancedConsulServiceRegistration {
        let mut checks = Vec::new();

        if let Some(hc) = &service.health_check {
            let check = match hc.check_type {
                HealthCheckType::Http => ConsulHealthCheck {
                    name: format!("{}-http", service.id),
                    http: Some(format!("http://{}:{}{}",
                        service.address, service.port, hc.endpoint.as_deref().unwrap_or("/health"))),
                    interval: Some(format!("{}s", hc.interval_seconds)),
                    timeout: Some(format!("{}s", hc.timeout_seconds)),
                    deregister_critical_service_after: Some(format!("{}s", hc.deregister_after_seconds)),
                    ..Default::default()
                },
                HealthCheckType::Tcp => ConsulHealthCheck {
                    name: format!("{}-tcp", service.id),
                    tcp: Some(format!("{}:{}", service.address, service.port)),
                    interval: Some(format!("{}s", hc.interval_seconds)),
                    timeout: Some(format!("{}s", hc.timeout_seconds)),
                    deregister_critical_service_after: Some(format!("{}s", hc.deregister_after_seconds)),
                    ..Default::default()
                },
                HealthCheckType::Ttl => ConsulHealthCheck {
                    name: format!("{}-ttl", service.id),
                    ttl: Some(format!("{}s", hc.interval_seconds)),
                    deregister_critical_service_after: Some(format!("{}s", hc.deregister_after_seconds)),
                    ..Default::default()
                },
            };
            checks.push(check);
        }

        // Add service mesh sidecar if enabled
        if self.service_mesh_enabled {
            checks.push(ConsulHealthCheck {
                name: format!("{}-connect", service.id),
                tcp: Some(format!("{}:{}", service.address, service.port + 1)), // Sidecar port
                interval: Some("10s".to_string()),
                ..Default::default()
            });
        }

        AdvancedConsulServiceRegistration {
            id: Some(service.id.clone()),
            name: service.name.clone(),
            address: service.address.clone(),
            port: Some(service.port),
            tags: Some(service.tags.clone()),
            meta: Some(service.metadata.clone()),
            checks,
            connect: if self.service_mesh_enabled {
                Some(ConsulConnect {
                    sidecar_service: Some(ConsulSidecarService {
                        port: service.port + 1,
                        proxy: Some(ConsulProxyConfig {
                            upstreams: vec![], // Will be populated dynamically
                            config: HashMap::new(),
                        }),
                    }),
                })
            } else {
                None
            },
            ..Default::default()
        }
    }

    /// Convert Consul service to ServiceInstance with mesh info
    fn from_advanced_consul_service(&self, consul_service: &AdvancedConsulService) -> ServiceInstance {
        let mut metadata = consul_service.service.meta.clone().unwrap_or_default();

        // Add service mesh information to metadata
        if let Some(connect) = &consul_service.service.connect {
            metadata.insert("service_mesh_enabled".to_string(), "true".to_string());
            if let Some(sidecar) = &connect.sidecar_service {
                metadata.insert("sidecar_port".to_string(), sidecar.port.to_string());
            }
        }

        ServiceInstance {
            id: consul_service.service.id.clone().unwrap_or_default(),
            name: consul_service.service.name.clone(),
            address: consul_service.service.address.clone(),
            port: consul_service.service.port.unwrap_or(0),
            tags: consul_service.service.tags.clone().unwrap_or_default(),
            metadata,
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

    /// Get ACL token information
    pub async fn get_acl_token(&self, token_accessor: &str) -> Result<ConsulACLToken, RegistryError> {
        let url = self.build_url(&format!("/v1/acl/token/{}", token_accessor));

        let response = self
            .add_auth_header(self.client.get(&url))
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to get ACL token: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("ACL token retrieval failed with status {}: {}", status, body),
            });
        }

        let token: ConsulACLToken = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse ACL token: {}", e),
            })?;

        Ok(token)
    }

    /// Create ACL token
    pub async fn create_acl_token(&self, token: &ConsulACLToken) -> Result<String, RegistryError> {
        let url = self.build_url("/v1/acl/token");

        let response = self
            .add_auth_header(self.client.put(&url))
            .json(token)
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to create ACL token: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("ACL token creation failed with status {}: {}", status, body),
            });
        }

        let result: HashMap<String, String> = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse ACL token creation response: {}", e),
            })?;

        result.get("AccessorID")
            .cloned()
            .ok_or_else(|| RegistryError::SerializationError {
                message: "Missing AccessorID in ACL token creation response".to_string(),
            })
    }

    /// Get service intentions
    pub async fn get_service_intentions(&self, service_name: &str) -> Result<Vec<ServiceIntention>, RegistryError> {
        let url = self.build_url(&format!("/v1/connect/intentions?filter=SourceName=={}", service_name));

        let response = self
            .add_auth_header(self.client.get(&url))
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to get service intentions: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("Service intentions retrieval failed with status {}: {}", status, body),
            });
        }

        let intentions: Vec<ServiceIntention> = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse service intentions: {}", e),
            })?;

        Ok(intentions)
    }

    /// Create service intention
    pub async fn create_service_intention(&self, intention: &ServiceIntention) -> Result<(), RegistryError> {
        let url = self.build_url("/v1/connect/intentions");

        let response = self
            .add_auth_header(self.client.post(&url))
            .json(intention)
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to create service intention: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("Service intention creation failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    /// Get KV store value
    pub async fn get_kv_value(&self, key: &str) -> Result<Option<String>, RegistryError> {
        let url = self.build_url(&format!("/v1/kv/{}", key));

        let response = self
            .add_auth_header(self.client.get(&url))
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to get KV value: {}", e),
            })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("KV get failed with status {}: {}", status, body),
            });
        }

        let kv_entries: Vec<ConsulKVEntry> = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse KV response: {}", e),
            })?;

        if let Some(entry) = kv_entries.first() {
            if let Some(value) = &entry.value {
                let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, value)
                    .map_err(|e| RegistryError::SerializationError {
                        message: format!("Failed to decode KV value: {}", e),
                    })?;
                let value_str = String::from_utf8(decoded)
                    .map_err(|e| RegistryError::SerializationError {
                        message: format!("KV value is not valid UTF-8: {}", e),
                    })?;
                Ok(Some(value_str))
            } else {
                Ok(Some(String::new()))
            }
        } else {
            Ok(None)
        }
    }

    /// Set KV store value
    pub async fn set_kv_value(&self, key: &str, value: &str) -> Result<(), RegistryError> {
        let url = self.build_url(&format!("/v1/kv/{}", key));

        let response = self
            .add_auth_header(self.client.put(&url))
            .body(value.to_string())
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to set KV value: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("KV set failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    /// Get service mesh upstreams for a service
    pub async fn get_service_mesh_upstreams(&self, service_name: &str) -> Result<Vec<ConsulUpstream>, RegistryError> {
        if !self.service_mesh_enabled {
            return Ok(vec![]);
        }

        let url = self.build_url(&format!("/v1/health/service/{}", service_name));

        let response = self
            .add_auth_header(self.client.get(&url))
            .send()
            .await
            .map_err(|e| RegistryError::NetworkError {
                message: format!("Failed to get service mesh upstreams: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RegistryError::BackendError {
                backend: "consul".to_string(),
                message: format!("Service mesh upstreams failed with status {}: {}", status, body),
            });
        }

        let services: Vec<AdvancedConsulService> = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse service mesh response: {}", e),
            })?;

        let mut upstreams = Vec::new();

        for service in services {
            if let Some(connect) = &service.service.connect {
                if let Some(sidecar) = &connect.sidecar_service {
                    if let Some(proxy) = &sidecar.proxy {
                        upstreams.extend(proxy.upstreams.clone());
                    }
                }
            }
        }

        Ok(upstreams)
    }
}

#[async_trait]
impl ServiceRegistry for AdvancedConsulRegistry {
    async fn register(&self, service: ServiceInfo) -> Result<(), RegistryError> {
        let consul_service = self.to_advanced_consul_service(&service);
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

        let consul_services: Vec<AdvancedConsulService> = response
            .json()
            .await
            .map_err(|e| RegistryError::SerializationError {
                message: format!("Failed to parse Consul response: {}", e),
            })?;

        let instances: Vec<ServiceInstance> = consul_services
            .iter()
            .map(|cs| self.from_advanced_consul_service(cs))
            .collect();

        Ok(instances)
    }

    async fn get_service(&self, service_id: &str) -> Result<ServiceInstance, RegistryError> {
        // First try to get all services and find the one with matching ID
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

        let services: HashMap<String, AdvancedConsulServiceInfo> = response
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

        let health_checks: Vec<AdvancedConsulService> = if health_response.status().is_success() {
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

        Ok(self.from_advanced_consul_service(consul_service))
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
        Ok(Box::new(AdvancedConsulWatcher::new(
            self.client.clone(),
            self.base_url.clone(),
            service_name.to_string(),
            self.datacenter.clone(),
            self.token.clone(),
        )))
    }
}

/// Configuration for advanced Consul registry
#[derive(Debug, Clone)]
pub struct AdvancedConsulConfig {
    pub base_url: String,
    pub datacenter: Option<String>,
    pub token: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub service_mesh_enabled: bool,
}

impl Default for AdvancedConsulConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8500".to_string(),
            datacenter: None,
            token: None,
            timeout_seconds: Some(30),
            service_mesh_enabled: false,
        }
    }
}

/// Advanced Consul service registration with service mesh support
#[derive(Debug, Serialize, Deserialize)]
struct AdvancedConsulServiceRegistration {
    id: Option<String>,
    name: String,
    address: String,
    port: Option<u16>,
    tags: Option<Vec<String>>,
    meta: Option<HashMap<String, String>>,
    checks: Vec<ConsulHealthCheck>,
    connect: Option<ConsulConnect>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

impl Default for AdvancedConsulServiceRegistration {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            address: String::new(),
            port: None,
            tags: None,
            meta: None,
            checks: vec![],
            connect: None,
            extra: HashMap::new(),
        }
    }
}

/// Consul service mesh connect configuration
#[derive(Debug, Serialize, Deserialize)]
struct ConsulConnect {
    sidecar_service: Option<ConsulSidecarService>,
}

/// Consul sidecar service configuration
#[derive(Debug, Serialize, Deserialize)]
struct ConsulSidecarService {
    port: u16,
    proxy: Option<ConsulProxyConfig>,
}

/// Consul proxy configuration
#[derive(Debug, Serialize, Deserialize)]
struct ConsulProxyConfig {
    upstreams: Vec<ConsulUpstream>,
    config: HashMap<String, serde_json::Value>,
}

/// Consul upstream configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConsulUpstream {
    destination_name: String,
    local_bind_port: u16,
}

/// Advanced Consul service response
#[derive(Debug, Deserialize)]
struct AdvancedConsulService {
    service: AdvancedConsulServiceInfo,
    checks: Vec<ConsulHealthCheck>,
}

/// Advanced Consul service information
#[derive(Debug, Deserialize)]
struct AdvancedConsulServiceInfo {
    id: Option<String>,
    name: String,
    address: String,
    port: Option<u16>,
    tags: Option<Vec<String>>,
    meta: Option<HashMap<String, String>>,
    connect: Option<ConsulConnect>,
}

/// Consul health check
#[derive(Debug, Serialize, Deserialize, Default)]
struct ConsulHealthCheck {
    name: String,
    http: Option<String>,
    tcp: Option<String>,
    ttl: Option<String>,
    interval: Option<String>,
    timeout: Option<String>,
    deregister_critical_service_after: Option<String>,
    status: String,
}

/// Consul check status
#[derive(Debug, Deserialize)]
struct ConsulCheckStatus {
    status: String,
}

/// Consul ACL token
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsulACLToken {
    pub accessor_id: Option<String>,
    pub secret_id: Option<String>,
    pub description: String,
    pub policies: Vec<ConsulACLPolicy>,
    pub roles: Vec<ConsulACLRole>,
    pub local: bool,
    pub create_time: Option<String>,
    pub hash: Option<String>,
}

/// Consul ACL policy
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsulACLPolicy {
    pub id: Option<String>,
    pub name: String,
}

/// Consul ACL role
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsulACLRole {
    pub id: Option<String>,
    pub name: String,
}

/// Service intention for service mesh
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceIntention {
    pub source_name: String,
    pub destination_name: String,
    pub action: String, // "allow" or "deny"
    pub meta: Option<HashMap<String, String>>,
}

/// Consul KV store entry
#[derive(Debug, Deserialize)]
struct ConsulKVEntry {
    key: String,
    value: Option<String>,
    flags: u64,
    create_index: u64,
    modify_index: u64,
}

/// Advanced watcher for Consul service changes with service mesh support
pub struct AdvancedConsulWatcher {
    client: Client,
    base_url: String,
    service_name: String,
    datacenter: Option<String>,
    token: Option<String>,
    last_index: Option<String>,
}

impl AdvancedConsulWatcher {
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
impl ServiceWatcher for AdvancedConsulWatcher {
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

        let consul_services: Vec<AdvancedConsulService> = response.json().await.ok()?;

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
    fn test_advanced_consul_config_default() {
        let config = AdvancedConsulConfig::default();
        assert_eq!(config.base_url, "http://localhost:8500");
        assert!(config.datacenter.is_none());
        assert!(config.token.is_none());
        assert_eq!(config.timeout_seconds, Some(30));
        assert!(!config.service_mesh_enabled);
    }

    #[test]
    fn test_advanced_consul_registry_creation() {
        let registry = AdvancedConsulRegistry::new();
        // Just test that it can be created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_to_advanced_consul_service_conversion() {
        let registry = AdvancedConsulRegistry::new();

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

        let consul_service = registry.to_advanced_consul_service(&service);

        assert_eq!(consul_service.name, "test-service");
        assert_eq!(consul_service.address, "127.0.0.1");
        assert_eq!(consul_service.port, Some(8080));
        assert!(consul_service.tags.as_ref().unwrap().contains(&"web".to_string()));
        assert_eq!(consul_service.checks.len(), 1);
    }

    #[tokio::test]
    async fn test_service_mesh_conversion() {
        let config = AdvancedConsulConfig {
            service_mesh_enabled: true,
            ..Default::default()
        };
        let registry = AdvancedConsulRegistry::with_config(config);

        let service = ServiceInfo {
            id: "mesh-service-1".to_string(),
            name: "mesh-service".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            tags: vec![],
            metadata: HashMap::new(),
            health_check: None,
        };

        let consul_service = registry.to_advanced_consul_service(&service);

        assert!(consul_service.connect.is_some());
        let connect = consul_service.connect.as_ref().unwrap();
        assert!(connect.sidecar_service.is_some());
        let sidecar = connect.sidecar_service.as_ref().unwrap();
        assert_eq!(sidecar.port, 8081); // sidecar port = service port + 1
    }
}