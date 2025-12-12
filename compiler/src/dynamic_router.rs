//! Dynamic Routing System para API Gateway
//!
//! Implementación de: TASK-113BW
//! Historia: VELA-611
//! Fecha: 2025-01-30
//!
//! Descripción:
//! Sistema de routing dinámico que permite configurar rutas desde archivos
//! externos, service discovery, health checks y load balancing dinámico.

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::router::{Router, Route};
use crate::gateway::{GatewayConfig, ServiceConfig, GatewayError};
use crate::config_loader::ConfigLoader;

/// Configuración de routing dinámico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRoutingConfig {
    /// Habilitar routing dinámico
    pub enabled: bool,
    /// Archivo de configuración de rutas
    pub routes_file: Option<String>,
    /// Intervalo de health checks (segundos)
    pub health_check_interval: u64,
    /// Timeout para health checks (segundos)
    pub health_check_timeout: u64,
    /// Service discovery configuration
    pub service_discovery: Option<ServiceDiscoveryConfig>,
}

/// Configuración de service discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// Tipo de discovery (consul, etcd, kubernetes, static)
    pub discovery_type: String,
    /// Endpoint del service discovery
    pub endpoint: String,
    /// Prefijo para keys de servicios
    pub service_prefix: String,
    /// Intervalo de polling (segundos)
    pub poll_interval: u64,
}

/// Información de salud de un servicio
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub service_name: String,
    pub endpoint: String,
    pub healthy: bool,
    pub last_check: Instant,
    pub response_time: Duration,
    pub consecutive_failures: u32,
}

/// Estado de un servicio registrado
#[derive(Debug, Clone)]
pub struct RegisteredService {
    pub name: String,
    pub endpoints: Vec<String>,
    pub healthy_endpoints: Vec<String>,
    pub routes: Vec<Route>,
    pub health_checks: Vec<ServiceHealth>,
}

/// Manager para routing dinámico
pub struct DynamicRouter {
    /// Router base
    router: Arc<RwLock<Router>>,
    /// Configuración
    config: DynamicRoutingConfig,
    /// Servicios registrados
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,
    /// Health checker
    health_checker: Arc<HealthChecker>,
    /// Service discovery
    service_discovery: Option<Box<dyn ServiceDiscovery>>,
    /// Config loader para rutas dinámicas
    config_loader: Option<Arc<Mutex<ConfigLoader>>>,
}

impl DynamicRouter {
    /// Crear nuevo dynamic router
    pub fn new(config: DynamicRoutingConfig) -> Self {
        let router = Arc::new(RwLock::new(Router::new()));
        let services = Arc::new(RwLock::new(HashMap::new()));
        let health_checker = HealthChecker::new(
            Duration::from_secs(config.health_check_interval),
            Duration::from_secs(config.health_check_timeout),
        );

        Self {
            router,
            config,
            services,
            health_checker: Arc::new(health_checker),
            service_discovery: None,
            config_loader: None,
        }
    }

    /// Configurar service discovery
    pub fn with_service_discovery(mut self, discovery: Box<dyn ServiceDiscovery>) -> Self {
        self.service_discovery = Some(discovery);
        self
    }

    /// Configurar config loader para rutas dinámicas
    pub fn with_config_loader(mut self, loader: Arc<Mutex<ConfigLoader>>) -> Self {
        self.config_loader = Some(loader);
        self
    }

    /// Inicializar el routing dinámico
    pub async fn initialize(&mut self) -> Result<(), GatewayError> {
        // Cargar rutas desde archivo si está configurado
        if let Some(routes_file) = &self.config.routes_file {
            self.load_routes_from_file(routes_file).await?;
        }

        // Inicializar service discovery si está configurado
        if let Some(discovery) = &self.service_discovery {
            self.initialize_service_discovery(discovery).await?;
        }

        // Iniciar health checks
        self.start_health_checks().await;

        Ok(())
    }

    /// Cargar rutas desde archivo de configuración
    async fn load_routes_from_file(&self, file_path: &str) -> Result<(), GatewayError> {
        let routes_config: RoutesConfig = if let Some(loader) = &self.config_loader {
            // Usar config loader si está disponible
            let loader = loader.lock().unwrap();
            let content = std::fs::read_to_string(file_path)
                .map_err(|e| GatewayError::Internal(format!("Cannot read routes file: {}", e)))?;
            serde_json::from_str(&content)
                .map_err(|e| GatewayError::Internal(format!("Invalid routes config: {}", e)))?
        } else {
            // Cargar directamente desde archivo
            let content = std::fs::read_to_string(file_path)
                .map_err(|e| GatewayError::Internal(format!("Cannot read routes file: {}", e)))?;
            serde_json::from_str(&content)
                .map_err(|e| GatewayError::Internal(format!("Invalid routes config: {}", e)))?
        };

        // Aplicar rutas dinámicas
        self.apply_routes_config(routes_config).await?;

        Ok(())
    }

    /// Aplicar configuración de rutas
    async fn apply_routes_config(&self, config: RoutesConfig) -> Result<(), GatewayError> {
        let mut router = self.router.write().await;
        let mut services = self.services.write().await;

        for service_config in config.services {
            // Registrar servicio
            let service = RegisteredService {
                name: service_config.name.clone(),
                endpoints: service_config.endpoints.clone(),
                healthy_endpoints: service_config.endpoints.clone(), // Inicialmente todos healthy
                routes: Vec::new(),
                health_checks: Vec::new(),
            };

            services.insert(service_config.name.clone(), service);

            // Agregar rutas al router
            for route_config in &service_config.routes {
                let route = Route {
                    path: route_config.path.clone(),
                    methods: route_config.methods.clone(),
                    service: service_config.name.clone(),
                    middlewares: route_config.middlewares.clone(),
                };

                router.add_route(route.clone());

                // Actualizar servicio con la ruta
                if let Some(service) = services.get_mut(&service_config.name) {
                    service.routes.push(route);
                }
            }
        }

        Ok(())
    }

    /// Inicializar service discovery
    async fn initialize_service_discovery(&self, discovery: &Box<dyn ServiceDiscovery>) -> Result<(), GatewayError> {
        // Descubrir servicios iniciales
        let services = discovery.discover_services().await
            .map_err(|e| GatewayError::Internal(format!("Service discovery error: {}", e)))?;

        // Registrar servicios descubiertos
        for service in services {
            self.register_service(service).await?;
        }

        Ok(())
    }

    /// Registrar un servicio dinámicamente
    pub async fn register_service(&self, service: ServiceInfo) -> Result<(), GatewayError> {
        let mut services = self.services.write().await;
        let mut router = self.router.write().await;

        let registered_service = RegisteredService {
            name: service.name.clone(),
            endpoints: service.endpoints.clone(),
            healthy_endpoints: service.endpoints.clone(),
            routes: Vec::new(),
            health_checks: Vec::new(),
        };

        services.insert(service.name.clone(), registered_service);

        // Agregar rutas por defecto para el servicio
        let default_route = Route {
            path: format!("/api/{}", service.name),
            methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            service: service.name.clone(),
            middlewares: vec!["cors".to_string(), "logging".to_string()],
        };

        router.add_route(default_route.clone());

        if let Some(service_mut) = services.get_mut(&service.name) {
            service_mut.routes.push(default_route);
        }

        Ok(())
    }

    /// Desregistrar un servicio
    pub async fn unregister_service(&self, service_name: &str) -> Result<(), GatewayError> {
        let mut services = self.services.write().await;
        let mut router = self.router.write().await;

        if let Some(service) = services.remove(service_name) {
            // Remover todas las rutas del servicio
            for route in service.routes {
                // Nota: El router actual no tiene método para remover rutas
                // Esto sería una mejora futura
            }
        }

        Ok(())
    }

    /// Obtener servicios registrados
    pub async fn get_services(&self) -> HashMap<String, RegisteredService> {
        let services = self.services.read().await;
        services.clone()
    }

    /// Obtener router subyacente
    pub fn get_router(&self) -> Arc<RwLock<Router>> {
        Arc::clone(&self.router)
    }

    /// Iniciar health checks en background
    async fn start_health_checks(&self) {
        let services = Arc::clone(&self.services);
        let health_checker = self.health_checker.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(health_checker.check_interval);

            loop {
                interval.tick().await;

                let mut services_write = services.write().await;
                for (service_name, service) in services_write.iter_mut() {
                    for endpoint in &service.endpoints {
                        let health = health_checker.check_endpoint(endpoint).await;

                        // Actualizar health checks
                        service.health_checks.retain(|h| h.endpoint != *endpoint);
                        service.health_checks.push(health.clone());

                        // Actualizar healthy endpoints
                        if health.healthy {
                            if !service.healthy_endpoints.contains(endpoint) {
                                service.healthy_endpoints.push(endpoint.clone());
                            }
                        } else {
                            service.healthy_endpoints.retain(|e| e != endpoint);
                        }
                    }
                }
            }
        });
    }

    /// Actualizar rutas dinámicamente (para hot reload)
    pub async fn reload_routes(&self, routes_config: RoutesConfig) -> Result<(), GatewayError> {
        // Limpiar rutas existentes
        let mut router = self.router.write().await;
        let mut new_router = Router::new();

        // Aplicar nueva configuración
        let mut services = self.services.write().await;
        *services = HashMap::new();

        // Recrear todo desde la configuración
        drop(services);
        drop(router);

        self.apply_routes_config(routes_config).await?;

        Ok(())
    }
}

/// Configuración de rutas desde archivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutesConfig {
    pub services: Vec<ServiceRoutesConfig>,
}

/// Configuración de rutas para un servicio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRoutesConfig {
    pub name: String,
    pub endpoints: Vec<String>,
    pub routes: Vec<RouteConfig>,
}

/// Configuración de una ruta individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub path: String,
    pub methods: Vec<String>,
    pub middlewares: Vec<String>,
}

/// Información de un servicio para discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub endpoints: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Trait para service discovery
#[async_trait::async_trait]
pub trait ServiceDiscovery: Send + Sync {
    async fn discover_services(&self) -> Result<Vec<ServiceInfo>, GatewayError>;
    async fn watch_services(&self) -> Result<(), GatewayError>;
}

/// Health checker para servicios
#[derive(Debug, Clone)]
pub struct HealthChecker {
    check_interval: Duration,
    timeout: Duration,
}

impl HealthChecker {
    pub fn new(check_interval: Duration, timeout: Duration) -> Self {
        Self {
            check_interval,
            timeout,
        }
    }

    /// Verificar health de un endpoint
    pub async fn check_endpoint(&self, endpoint: &str) -> ServiceHealth {
        let start = Instant::now();
        let service_name = extract_service_name_from_endpoint(endpoint);

        // Health check simple (HTTP GET)
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .unwrap_or_default();

        let health = match client.get(&format!("{}/health", endpoint)).send().await {
            Ok(response) if response.status().is_success() => {
                ServiceHealth {
                    service_name,
                    endpoint: endpoint.to_string(),
                    healthy: true,
                    last_check: start,
                    response_time: start.elapsed(),
                    consecutive_failures: 0,
                }
            }
            _ => {
                ServiceHealth {
                    service_name,
                    endpoint: endpoint.to_string(),
                    healthy: false,
                    last_check: start,
                    response_time: start.elapsed(),
                    consecutive_failures: 1, // TODO: track consecutive failures
                }
            }
        };

        health
    }
}

/// Extraer nombre del servicio desde endpoint
fn extract_service_name_from_endpoint(endpoint: &str) -> String {
    // Simple extraction: assume format "http://service-name:port"
    endpoint
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split(':')
        .next()
        .unwrap_or("unknown")
        .split('.')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dynamic_router_creation() {
        let config = DynamicRoutingConfig {
            enabled: true,
            routes_file: None,
            health_check_interval: 30,
            health_check_timeout: 5,
            service_discovery: None,
        };

        let router = DynamicRouter::new(config);
        let services = router.get_services().await;
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn test_service_registration() {
        let config = DynamicRoutingConfig {
            enabled: true,
            routes_file: None,
            health_check_interval: 30,
            health_check_timeout: 5,
            service_discovery: None,
        };

        let router = DynamicRouter::new(config);

        let service = ServiceInfo {
            name: "user-service".to_string(),
            endpoints: vec!["http://localhost:8081".to_string()],
            metadata: HashMap::new(),
        };

        router.register_service(service).await.unwrap();

        let services = router.get_services().await;
        assert!(services.contains_key("user-service"));

        let service = &services["user-service"];
        assert_eq!(service.endpoints.len(), 1);
        assert!(service.routes.len() > 0);
    }

    #[tokio::test]
    async fn test_routes_config_application() {
        let config = DynamicRoutingConfig {
            enabled: true,
            routes_file: None,
            health_check_interval: 30,
            health_check_timeout: 5,
            service_discovery: None,
        };

        let router = DynamicRouter::new(config);

        let routes_config = RoutesConfig {
            services: vec![ServiceRoutesConfig {
                name: "api-service".to_string(),
                endpoints: vec!["http://localhost:8080".to_string()],
                routes: vec![RouteConfig {
                    path: "/api/v1/*".to_string(),
                    methods: vec!["GET".to_string(), "POST".to_string()],
                    middlewares: vec!["auth".to_string()],
                }],
            }],
        };

        router.apply_routes_config(routes_config).await.unwrap();

        let services = router.get_services().await;
        assert!(services.contains_key("api-service"));

        // Verificar que la ruta fue agregada al router
        let router_guard = router.get_router().read().await;
        let matched = router_guard.match_route("/api/v1/users", "GET");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().service, "api-service");
    }
}