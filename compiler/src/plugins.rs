//! Plugin System para API Gateway
//!
//! Sistema extensible de plugins/middleware que permite:
//! - Pre-processing de requests
//! - Post-processing de responses
//! - Error handling
//! - Custom business logic

use crate::gateway::{Context, GatewayError};

/// Tipos de plugins disponibles
#[derive(Debug, Clone)]
pub enum PluginType {
    Logging,
    Cors(Vec<String>), // allowed origins
    RateLimit,
    Auth,
    Custom(String), // custom plugin name
}

/// Plugin configuration
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub plugin_type: PluginType,
    pub priority: i32,
    pub enabled: bool,
}

/// Plugin trait - define la interfaz para todos los plugins
pub trait Plugin: Send + Sync {
    /// Nombre del plugin
    fn name(&self) -> &str;

    /// Ejecutar el plugin
    fn execute(&self, ctx: &mut Context) -> Result<(), GatewayError>;

    /// Prioridad de ejecución (menor número = mayor prioridad)
    fn priority(&self) -> i32 {
        0
    }
}

/// Plugin de logging básico
pub struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        "logging"
    }

    fn execute(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        println!("📨 {} {} - Headers: {} - Body size: {}",
            ctx.request.method,
            ctx.request.path,
            ctx.request.headers.len(),
            ctx.request.body.as_ref().map(|b| b.len()).unwrap_or(0)
        );
        Ok(())
    }
}

#[cfg(feature = "gateway-async-plugin")]
#[async_trait::async_trait]
impl crate::gateway::Plugin for LoggingPlugin {
    async fn execute(&self, ctx: &mut crate::gateway::Context) -> Result<(), crate::gateway::GatewayError> {
        // Call the sync version for compatibility
        Plugin::execute(self, ctx)
    }
}

/// Plugin de CORS
pub struct CorsPlugin {
    allowed_origins: Vec<String>,
}

impl CorsPlugin {
    pub fn new(allowed_origins: Vec<String>) -> Self {
        Self { allowed_origins }
    }
}

impl Plugin for CorsPlugin {
    fn name(&self) -> &str {
        "cors"
    }

    fn priority(&self) -> i32 {
        -10 // Alta prioridad
    }

    fn execute(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        if ctx.request.method == "OPTIONS" {
            // Handle preflight request
            let mut response = crate::gateway::Response {
                status: 200,
                headers: std::collections::HashMap::new(),
                body: None,
            };

            response.headers.insert("Access-Control-Allow-Origin".to_string(),
                self.allowed_origins.join(", "));
            response.headers.insert("Access-Control-Allow-Methods".to_string(),
                "GET, POST, PUT, DELETE, OPTIONS".to_string());
            response.headers.insert("Access-Control-Allow-Headers".to_string(),
                "Content-Type, Authorization, X-API-Key".to_string());

            ctx.response = Some(response);
            return Ok(());
        }

        // Add CORS headers to normal responses
        if let Some(ref mut response) = ctx.response {
            response.headers.insert("Access-Control-Allow-Origin".to_string(),
                self.allowed_origins.join(", "));
        }

        Ok(())
    }
}

/// Plugin de rate limiting (integra con RateLimiter)
pub struct RateLimitPlugin;

impl Plugin for RateLimitPlugin {
    fn name(&self) -> &str {
        "rate_limit"
    }

    fn priority(&self) -> i32 {
        -5 // Después de CORS, antes de auth
    }

    fn execute(&self, _ctx: &mut Context) -> Result<(), GatewayError> {
        // La lógica de rate limiting se maneja en el gateway principal
        // Este plugin podría agregar headers informativos
        Ok(())
    }
}

/// Plugin de error handling
pub struct ErrorHandlingPlugin;

impl Plugin for ErrorHandlingPlugin {
    fn name(&self) -> &str {
        "error_handling"
    }

    fn priority(&self) -> i32 {
        100 // Baja prioridad - maneja errores de otros plugins
    }

    fn execute(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        // Si hay un error en el contexto, crear response de error
        if let Some(error_key) = ctx.metadata.get("error") {
            if let Some(error_msg) = error_key.as_str() {
                let response = crate::gateway::Response {
                    status: 500,
                    headers: {
                        let mut headers = std::collections::HashMap::new();
                        headers.insert("Content-Type".to_string(), "application/json".to_string());
                        headers
                    },
                    body: Some(format!("{{\"error\": \"{}\"}}", error_msg).into_bytes()),
                };
                ctx.response = Some(response);
            }
        }
        Ok(())
    }
}

/// Plugin personalizado de ejemplo
pub struct CustomHeaderPlugin {
    header_name: String,
    header_value: String,
}

impl CustomHeaderPlugin {
    pub fn new(header_name: String, header_value: String) -> Self {
        Self { header_name, header_value }
    }
}

impl Plugin for CustomHeaderPlugin {
    fn name(&self) -> &str {
        "custom_header"
    }

    fn execute(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        if let Some(ref mut response) = ctx.response {
            response.headers.insert(self.header_name.clone(), self.header_value.clone());
        }
        Ok(())
    }
}

/// Registry de plugins
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Registrar un plugin
    pub fn register<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        // Ordenar por prioridad
        self.plugins.sort_by_key(|p| p.priority());
        self
    }

    /// Obtener plugins ordenados por prioridad
    pub fn get_plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// Ejecutar todos los plugins
    pub fn execute_all(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        for plugin in &self.plugins {
            plugin.execute(ctx)?;
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::Request;
    use std::collections::HashMap;

    fn create_test_context() -> Context {
        Context {
            request: Request {
                method: "GET".to_string(),
                path: "/test".to_string(),
                headers: HashMap::new(),
                body: None,
                query_params: HashMap::new(),
                path_params: HashMap::new(),
            },
            response: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_logging_plugin() {
        let plugin = LoggingPlugin;
        let mut ctx = create_test_context();

        let result = plugin.execute(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(plugin.name(), "logging");
    }

    #[tokio::test]
    async fn test_cors_plugin_preflight() {
        let plugin = CorsPlugin::new(vec!["http://localhost:3000".to_string()]);
        let mut ctx = create_test_context();
        ctx.request.method = "OPTIONS".to_string();

        let result = plugin.execute(&mut ctx);
        assert!(result.is_ok());
        assert!(ctx.response.is_some());

        let response = ctx.response.unwrap();
        assert_eq!(response.status, 200);
        assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
    }

    #[tokio::test]
    async fn test_plugin_registry() {
        let registry = PluginRegistry::new()
            .register(LoggingPlugin)
            .register(CorsPlugin::new(vec!["*".to_string()]));

        assert_eq!(registry.get_plugins().len(), 2);
    }

    #[tokio::test]
    async fn test_plugin_execution_order() {
        let registry = PluginRegistry::new()
            .register(CustomHeaderPlugin::new("X-Test".to_string(), "value1".to_string()))
            .register(CustomHeaderPlugin::new("X-Test".to_string(), "value2".to_string()));

        let mut ctx = create_test_context();
        ctx.response = Some(crate::gateway::Response {
            status: 200,
            headers: HashMap::new(),
            body: None,
        });

        let result = registry.execute_all(&mut ctx);
        assert!(result.is_ok());

        // El último plugin debería sobrescribir
        let response = ctx.response.unwrap();
        assert_eq!(response.headers.get("X-Test"), Some(&"value2".to_string()));
    }

    #[tokio::test]
    async fn test_error_handling_plugin() {
        let plugin = ErrorHandlingPlugin;
        let mut ctx = create_test_context();
        ctx.metadata.insert("error".to_string(), serde_json::Value::String("Test error".to_string()));

        let result = plugin.execute(&mut ctx);
        assert!(result.is_ok());
        assert!(ctx.response.is_some());

        let response = ctx.response.unwrap();
        assert_eq!(response.status, 500);
        assert!(response.body.is_some());
    }
}