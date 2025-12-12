//! Tests del API Gateway
//!
//! Tests de integración y unitarios para el sistema de API Gateway.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::{GatewayConfig, Request, Response};
    use std::collections::HashMap;

    fn create_test_config() -> GatewayConfig {
        GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        }
    }

    fn create_test_request(method: &str, path: &str) -> Request {
        Request {
            method: method.to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_gateway_initialization() {
        let config = create_test_config();
        let gateway = ApiGateway::new(config);

        assert_eq!(gateway.config.port, 8080);
        assert_eq!(gateway.config.host, "127.0.0.1");
    }

    #[tokio::test]
    async fn test_gateway_with_plugins() {
        let config = create_test_config();
        let gateway = ApiGateway::new(config)
            .add_plugin(crate::plugins::LoggingPlugin);

        // El gateway debería tener un plugin registrado
        assert_eq!(gateway.plugin_chain.len(), 1);
    }

    #[tokio::test]
    async fn test_gateway_request_processing_without_routes() {
        let config = create_test_config();
        let gateway = ApiGateway::new(config);

        let request = create_test_request("GET", "/nonexistent");

        // Debería fallar porque no hay rutas configuradas
        let result = gateway.process_request(request).await;
        assert!(result.is_err());

        if let Err(crate::gateway::GatewayError::Routing(_)) = result {
            // Error esperado
        } else {
            panic!("Expected routing error");
        }
    }

    #[tokio::test]
    async fn test_gateway_metrics_recording() {
        let config = create_test_config();
        let gateway = ApiGateway::new(config);

        // Simular algunas operaciones que actualizarían métricas
        // En implementación real, esto se haría automáticamente

        let metrics = gateway.metrics.get_metrics().await;
        // Las métricas deberían estar inicializadas en 0
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.total_responses, 0);
        assert_eq!(metrics.total_errors, 0);
    }

    #[tokio::test]
    async fn test_gateway_health_status() {
        let config = create_test_config();
        let gateway = ApiGateway::new(config);

        let health = gateway.metrics.health_check().await;

        // Inicialmente debería estar en Starting
        match health {
            crate::metrics::HealthStatus::Starting => {},
            _ => panic!("Expected Starting status"),
        }
    }
}