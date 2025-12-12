//! API Gateway para Vela
//!
//! Implementación de API Gateway con routing, load balancing, rate limiting
//! y autenticación para microservicios Vela.
//!
//! # Arquitectura
//!
//! El gateway sigue una arquitectura modular con los siguientes componentes:
//!
//! - **Router**: Motor de routing basado en tries con wildcards
//! - **LoadBalancer**: Balanceo de carga con múltiples estrategias
//! - **RateLimiter**: Limitación de tasa con algoritmos configurables
//! - **AuthEngine**: Motor de autenticación y autorización
//! - **PluginSystem**: Sistema de plugins/middleware extensible
//! - **Metrics**: Observabilidad con métricas y tracing
//!
//! # Configuración Declarativa
//!
//! ```vela
//! @gateway({
//!   port: 8080,
//!   tls: true,
//!   rateLimit: "1000req/min",
//!   auth: "jwt"
//! })
//! class ApiGateway {
//!   // Routes se definen con decoradores
//! }
//!
//! @route("/api/v1/users", methods: ["GET", "POST"])
//! @rateLimit("100req/min")
//! @auth("required")
//! async fn handleUsers(req: Request) -> Response {
//!   // Routing logic
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// Re-export modules for easier access
pub use crate::router;
pub use crate::load_balancer;
pub use crate::rate_limiter;
pub use crate::auth;
pub use crate::plugins;
pub use crate::metrics;

/// Configuración principal del API Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub port: u16,
    pub host: String,
    pub tls: Option<TlsConfig>,
    pub rate_limit: Option<String>,
    pub auth: Option<String>,
    pub services: HashMap<String, ServiceConfig>,
}

/// Configuración TLS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

/// Configuración de un servicio backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub endpoints: Vec<String>,
    pub health_check: Option<String>,
    pub timeout: Option<u64>,
}

/// Request HTTP
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub query_params: HashMap<String, String>,
    pub path_params: HashMap<String, String>,
}

/// Response HTTP
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// Contexto de ejecución de un request
#[derive(Debug)]
pub struct Context {
    pub request: Request,
    pub response: Option<Response>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Plugin trait para middleware
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    async fn execute(&self, ctx: &mut Context) -> Result<(), GatewayError>;
}

/// Error del gateway
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Routing error: {0}")]
    Routing(String),
    #[error("Load balancing error: {0}")]
    LoadBalancing(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// API Gateway principal
pub struct ApiGateway {
    config: GatewayConfig,
    router: Arc<router::Router>,
    load_balancer: Arc<RwLock<load_balancer::LoadBalancer>>,
    rate_limiter: Arc<RwLock<rate_limiter::RateLimiter>>,
    auth_engine: Arc<RwLock<auth::AuthEngine>>,
    plugin_chain: Vec<Box<dyn Plugin>>,
    metrics: Arc<metrics::Metrics>,
}

impl ApiGateway {
    /// Crear nuevo gateway con configuración
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            router: Arc::new(router::Router::new()),
            load_balancer: Arc::new(RwLock::new(load_balancer::LoadBalancer::new())),
            rate_limiter: Arc::new(RwLock::new(rate_limiter::RateLimiter::new())),
            auth_engine: Arc::new(RwLock::new(auth::AuthEngine::new())),
            plugin_chain: Vec::new(),
            metrics: Arc::new(metrics::Metrics::new()),
            config,
        }
    }

    /// Agregar un plugin al pipeline
    pub fn add_plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.plugin_chain.push(Box::new(plugin));
        self
    }

    /// Procesar un request HTTP
    pub async fn process_request(&self, mut request: Request) -> Result<Response, GatewayError> {
        let mut ctx = Context {
            request,
            response: None,
            metadata: HashMap::new(),
        };

        // Ejecutar plugins en orden
        for plugin in &self.plugin_chain {
            plugin.execute(&mut ctx).await?;
        }

        // Routing
        let route = self.router.match_route(&ctx.request.path, &ctx.request.method)
            .ok_or_else(|| GatewayError::Routing(format!("No route found for {}", ctx.request.path)))?;

        // Rate limiting
        {
            let mut rate_limiter = self.rate_limiter.write().await;
            if !rate_limiter.check_limit(&ctx.request).await? {
                return Err(GatewayError::RateLimitExceeded);
            }
        }

        // Authentication
        {
            let auth_engine = self.auth_engine.read().await;
            auth_engine.authenticate(&mut ctx).await?;
        }

        // Load balancing
        let backend_url = {
            let mut load_balancer = self.load_balancer.write().await;
            load_balancer.select_backend(&route.service)?
        };

        // Forward request
        let response = self.forward_request(&ctx.request, &backend_url).await?;

        // Update metrics
        self.metrics.record_request(&ctx.request, &response).await;

        Ok(response)
    }

    /// Forward request al backend
    async fn forward_request(&self, request: &Request, backend_url: &str) -> Result<Response, GatewayError> {
        // Implementación simplificada - en producción usar reqwest o similar
        // Por ahora retornamos una respuesta mock
        Ok(Response {
            status: 200,
            headers: HashMap::new(),
            body: Some(b"OK".to_vec()),
        })
    }

    /// Iniciar el servidor HTTP
    pub async fn start(self) -> Result<(), GatewayError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        println!("🚀 API Gateway listening on {}", addr);

        // Implementación del servidor HTTP aquí
        // Por ahora solo imprimimos
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gateway_creation() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let gateway = ApiGateway::new(config);
        assert_eq!(gateway.config.port, 8080);
    }

    #[tokio::test]
    async fn test_request_processing() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let gateway = ApiGateway::new(config);

        let request = Request {
            method: "GET".to_string(),
            path: "/health".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Debería fallar porque no hay rutas configuradas
        let result = gateway.process_request(request).await;
        assert!(result.is_err());
    }
}