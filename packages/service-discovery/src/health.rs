//! Health Check Endpoints for Kubernetes
//!
//! This module provides HTTP endpoints for Kubernetes health checks,
//! specifically /health/live (liveness) and /health/ready (readiness) probes.
//! These endpoints integrate with the service discovery system to provide
//! comprehensive health monitoring.

use super::*;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::future::FutureExt;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub checks: HashMap<String, HealthCheckResult>,
    pub version: String,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: String,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
}

/// Health check status for health endpoints
#[derive(Debug, Clone, PartialEq)]
pub enum HealthEndpointStatus {
    Healthy,
    Unhealthy,
    Degraded,
}

impl From<HealthEndpointStatus> for String {
    fn from(status: HealthEndpointStatus) -> String {
        match status {
            HealthEndpointStatus::Healthy => "healthy".to_string(),
            HealthEndpointStatus::Unhealthy => "unhealthy".to_string(),
            HealthEndpointStatus::Degraded => "degraded".to_string(),
        }
    }
}

/// Health check function type
pub type HealthCheckFn = Box<dyn Fn() -> futures::future::BoxFuture<'static, HealthCheckResult> + Send + Sync>;

/// Health check server configuration
#[derive(Debug, Clone)]
pub struct HealthServerConfig {
    pub port: u16,
    pub host: String,
    pub enable_cors: bool,
    pub enable_tracing: bool,
    pub readiness_timeout_seconds: u64,
    pub liveness_timeout_seconds: u64,
}

impl Default for HealthServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            enable_cors: true,
            enable_tracing: true,
            readiness_timeout_seconds: 30,
            liveness_timeout_seconds: 10,
        }
    }
}

/// Health check server
#[derive(Clone)]
pub struct HealthCheckServer {
    config: HealthServerConfig,
    liveness_checks: Arc<RwLock<HashMap<String, HealthCheckFn>>>,
    readiness_checks: Arc<RwLock<HashMap<String, HealthCheckFn>>>,
    service_client: Option<Arc<ServiceDiscoveryClient>>,
}

impl HealthCheckServer {
    /// Create a new health check server with default configuration
    pub fn new() -> Self {
        Self::with_config(HealthServerConfig::default())
    }

    /// Create a new health check server with custom configuration
    pub fn with_config(config: HealthServerConfig) -> Self {
        Self {
            config,
            liveness_checks: Arc::new(RwLock::new(HashMap::new())),
            readiness_checks: Arc::new(RwLock::new(HashMap::new())),
            service_client: None,
        }
    }

    /// Create a server with service discovery client integration
    pub fn with_service_client(
        config: HealthServerConfig,
        service_client: Arc<ServiceDiscoveryClient>,
    ) -> Self {
        let mut server = Self::with_config(config);
        server.service_client = Some(service_client.clone());

        // Add default service discovery readiness check
        server.add_readiness_check(
            "service_discovery".to_string(),
            Box::new(move || {
                let client = service_client.clone();
                async move {
                    let start = std::time::Instant::now();

                    // Check if we can discover services (basic connectivity test)
                    match client.discover_services("health-check-service").await {
                        Ok(_) => HealthCheckResult {
                            status: "healthy".to_string(),
                            message: Some("Service discovery is operational".to_string()),
                            timestamp: chrono::Utc::now(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        },
                        Err(e) => HealthCheckResult {
                            status: "unhealthy".to_string(),
                            message: Some(format!("Service discovery error: {}", e)),
                            timestamp: chrono::Utc::now(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        },
                    }
                }.boxed()
            }),
        );

        server
    }

    /// Add a liveness health check
    pub fn add_liveness_check(&self, name: String, check: HealthCheckFn) {
        let mut checks = self.liveness_checks.try_write().unwrap();
        checks.insert(name, check);
    }

    /// Add a readiness health check
    pub fn add_readiness_check(&self, name: String, check: HealthCheckFn) {
        let mut checks = self.readiness_checks.try_write().unwrap();
        checks.insert(name, check);
    }

    /// Add a database connectivity check
    pub fn add_database_check(&self, name: String, check_fn: Arc<dyn Fn() -> futures::future::BoxFuture<'static, Result<(), String>> + Send + Sync>) {
        let check = Box::new(move || {
            let check_fn = Arc::clone(&check_fn);
            async move {
                let start = std::time::Instant::now();
                match check_fn().await {
                    Ok(()) => HealthCheckResult {
                        status: "healthy".to_string(),
                        message: Some("Database connection successful".to_string()),
                        timestamp: chrono::Utc::now(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    },
                    Err(e) => HealthCheckResult {
                        status: "unhealthy".to_string(),
                        message: Some(format!("Database error: {}", e)),
                        timestamp: chrono::Utc::now(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    },
                }
            }.boxed()
        });

        self.add_readiness_check(name, check);
    }

    /// Add an external service dependency check
    pub fn add_service_dependency_check(&self, service_name: String) {
        if let Some(client) = &self.service_client {
            let client = Arc::clone(client);
            let service_name_clone = service_name.clone();

            let check = Box::new(move || {
                let client = Arc::clone(&client);
                let service_name = service_name_clone.clone();
                async move {
                    let start = std::time::Instant::now();

                    match client.discover_services(&service_name).await {
                        Ok(instances) if !instances.is_empty() => {
                            let healthy_count = instances.iter()
                                .filter(|inst| inst.health_status == super::HealthStatus::Passing)
                                .count();

                            if healthy_count > 0 {
                                HealthCheckResult {
                                    status: "healthy".to_string(),
                                    message: Some(format!("Found {} healthy instances of {}", healthy_count, service_name)),
                                    timestamp: chrono::Utc::now(),
                                    duration_ms: start.elapsed().as_millis() as u64,
                                }
                            } else {
                                HealthCheckResult {
                                    status: "degraded".to_string(),
                                    message: Some(format!("Service {} has no healthy instances", service_name)),
                                    timestamp: chrono::Utc::now(),
                                    duration_ms: start.elapsed().as_millis() as u64,
                                }
                            }
                        }
                        Ok(_) => HealthCheckResult {
                            status: "unhealthy".to_string(),
                            message: Some(format!("Service {} not found", service_name)),
                            timestamp: chrono::Utc::now(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        },
                        Err(e) => HealthCheckResult {
                            status: "unhealthy".to_string(),
                            message: Some(format!("Service discovery error for {}: {}", service_name, e)),
                            timestamp: chrono::Utc::now(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        },
                    }
                }.boxed()
            });

            self.add_readiness_check(service_name, check);
        }
    }

    /// Start the health check server
    pub async fn start(self) -> Result<(), HealthCheckError> {
        let app = self.create_router();

        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse::<SocketAddr>()
            .map_err(|e| HealthCheckError::ConfigError(format!("Invalid address: {}", e)))?;

        println!("Health check server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| HealthCheckError::ServerError(format!("Failed to bind to {}: {}", addr, e)))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| HealthCheckError::ServerError(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Create the Axum router with health check endpoints
    fn create_router(&self) -> Router {
        let self_arc = Arc::new(self.clone());
        let mut router = Router::new()
            .route("/health/live", get(Self::liveness_handler))
            .route("/health/ready", get(Self::readiness_handler))
            .route("/health", get(Self::combined_health_handler))
            .with_state(self_arc);

        if self.config.enable_cors {
            router = router.layer(CorsLayer::permissive());
        }

        if self.config.enable_tracing {
            router = router.layer(TraceLayer::new_for_http());
        }

        router
    }

    /// Liveness probe handler - checks if the application is alive
    async fn liveness_handler(
        State(server): State<Arc<HealthCheckServer>>,
    ) -> (StatusCode, Json<HealthCheckResponse>) {
        let start_time = std::time::Instant::now();
        let mut checks = HashMap::new();
        let mut overall_status = HealthEndpointStatus::Healthy;

        // Run all liveness checks
        let liveness_checks = server.liveness_checks.read().await;
        for (name, check_fn) in liveness_checks.iter() {
            let result = check_fn().await;
            checks.insert(name.clone(), result.clone());

            if result.status == "unhealthy" {
                overall_status = HealthEndpointStatus::Unhealthy;
            }
        }

        // If no liveness checks are configured, assume healthy
        if liveness_checks.is_empty() {
            checks.insert(
                "process".to_string(),
                HealthCheckResult {
                    status: "healthy".to_string(),
                    message: Some("Process is running".to_string()),
                    timestamp: chrono::Utc::now(),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                },
            );
        }

        let status_code = match overall_status {
            HealthEndpointStatus::Healthy => StatusCode::OK,
            HealthEndpointStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
            HealthEndpointStatus::Degraded => StatusCode::OK, // Liveness allows degraded state
        };

        let response = HealthCheckResponse {
            status: overall_status.into(),
            timestamp: chrono::Utc::now(),
            checks,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        (status_code, Json(response))
    }

    /// Readiness probe handler - checks if the application is ready to serve traffic
    async fn readiness_handler(
        State(server): State<Arc<HealthCheckServer>>,
    ) -> (StatusCode, Json<HealthCheckResponse>) {
        let start_time = std::time::Instant::now();
        let mut checks = HashMap::new();
        let mut overall_status = HealthEndpointStatus::Healthy;

        // Run all readiness checks
        let readiness_checks = server.readiness_checks.read().await;
        for (name, check_fn) in readiness_checks.iter() {
            let result = check_fn().await;
            checks.insert(name.clone(), result.clone());

            match result.status.as_str() {
                "unhealthy" => overall_status = HealthEndpointStatus::Unhealthy,
                "degraded" => {
                    if overall_status == HealthEndpointStatus::Healthy {
                        overall_status = HealthEndpointStatus::Degraded;
                    }
                }
                _ => {}
            }
        }

        // If no readiness checks are configured, assume healthy
        if readiness_checks.is_empty() {
            checks.insert(
                "application".to_string(),
                HealthCheckResult {
                    status: "healthy".to_string(),
                    message: Some("Application is ready".to_string()),
                    timestamp: chrono::Utc::now(),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                },
            );
        }

        let status_code = match overall_status {
            HealthEndpointStatus::Healthy => StatusCode::OK,
            HealthEndpointStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
            HealthEndpointStatus::Degraded => StatusCode::OK, // Readiness allows degraded state
        };

        let response = HealthCheckResponse {
            status: overall_status.into(),
            timestamp: chrono::Utc::now(),
            checks,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        (status_code, Json(response))
    }

    /// Combined health check handler - returns both liveness and readiness
    async fn combined_health_handler(
        State(server): State<Arc<HealthCheckServer>>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let liveness_response = Self::liveness_handler(State(Arc::clone(&server))).await;
        let readiness_response = Self::readiness_handler(State(Arc::clone(&server))).await;

        let combined = serde_json::json!({
            "liveness": liveness_response.1 .0,
            "readiness": readiness_response.1 .0,
            "overall_status": if liveness_response.0 == StatusCode::OK && readiness_response.0 == StatusCode::OK {
                "healthy"
            } else {
                "unhealthy"
            }
        });

        let status_code = if liveness_response.0 == StatusCode::OK && readiness_response.0 == StatusCode::OK {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };

        (status_code, Json(combined))
    }
}

impl Default for HealthCheckServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during health check operations
#[derive(Error, Debug)]
pub enum HealthCheckError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Health check error: {0}")]
    HealthCheckError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_server_creation() {
        let server = HealthCheckServer::new();
        // Just test that it can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_health_check_config_default() {
        let config = HealthServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert!(config.enable_cors);
        assert!(config.enable_tracing);
    }

    #[tokio::test]
    async fn test_add_liveness_check() {
        let server = HealthCheckServer::new();

        server.add_liveness_check(
            "test".to_string(),
            Box::new(|| {
                Box::pin(async {
                    HealthCheckResult {
                        status: "healthy".to_string(),
                        message: Some("Test check".to_string()),
                        timestamp: chrono::Utc::now(),
                        duration_ms: 1,
                    }
                })
            }),
        );

        let checks = server.liveness_checks.read().await;
        assert_eq!(checks.len(), 1);
        assert!(checks.contains_key("test"));
    }

    #[tokio::test]
    async fn test_add_readiness_check() {
        let server = HealthCheckServer::new();

        server.add_readiness_check(
            "test".to_string(),
            Box::new(|| {
                Box::pin(async {
                    HealthCheckResult {
                        status: "healthy".to_string(),
                        message: Some("Test check".to_string()),
                        timestamp: chrono::Utc::now(),
                        duration_ms: 1,
                    }
                })
            }),
        );

        let checks = server.readiness_checks.read().await;
        assert_eq!(checks.len(), 1);
        assert!(checks.contains_key("test"));
    }

    #[tokio::test]
    async fn test_health_status_conversion() {
        assert_eq!(String::from(HealthEndpointStatus::Healthy), "healthy");
        assert_eq!(String::from(HealthEndpointStatus::Unhealthy), "unhealthy");
        assert_eq!(String::from(HealthEndpointStatus::Degraded), "degraded");
    }

    #[tokio::test]
    async fn test_health_check_response_serialization() {
        let response = HealthCheckResponse {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            checks: HashMap::new(),
            version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("version"));
    }
}