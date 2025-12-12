//! Tests exhaustivos del API Gateway
//!
//! Suite completa de tests para validar:
//! - Routing dinámico
//! - Load balancing
//! - Rate limiting
//! - Manejo de errores
//! - Concurrencia

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::gateway::{
    ApiGateway, GatewayConfig, Request, Response, GatewayError,
    Context, Plugin
};
use crate::rate_limiter::{RateLimiter, RateLimitRule, RateLimitKeyType};
use crate::load_balancer::{LoadBalancer, LoadBalancingStrategy};
use crate::router::{Router, Route};

/// Mock plugin para testing
struct MockPlugin {
    pub should_fail: bool,
    pub execution_count: Arc<std::sync::Mutex<usize>>,
}

#[async_trait::async_trait]
impl Plugin for MockPlugin {
    async fn execute(&self, _ctx: &mut Context) -> Result<(), GatewayError> {
        let mut count = self.execution_count.lock().unwrap();
        *count += 1;

        if self.should_fail {
            Err(GatewayError::Internal("Mock plugin failed".to_string()))
        } else {
            Ok(())
        }
    }
}

mod rate_limiting_tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting_by_ip() {
        let mut rate_limiter = RateLimiter::new();

        // Configurar regla de rate limiting por IP
        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Ip,
            pattern: "*".to_string(),
            requests_per_second: 2,
            burst_size: 2,
        };
        rate_limiter.add_rule(rule);

        let mut request = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Forwarded-For".to_string(), "192.168.1.100".to_string());
                h
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Primeras 2 requests deberían pasar
        assert!(rate_limiter.check_limit(&request).await.unwrap());
        assert!(rate_limiter.check_limit(&request).await.unwrap());

        // Tercera debería fallar
        assert!(!rate_limiter.check_limit(&request).await.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limiting_by_user() {
        let mut rate_limiter = RateLimiter::new();

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::User,
            pattern: "*".to_string(),
            requests_per_second: 1,
            burst_size: 1,
        };
        rate_limiter.add_rule(rule);

        let mut request = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Authorization".to_string(), "Bearer user123".to_string());
                h
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Primera request pasa
        assert!(rate_limiter.check_limit(&request).await.unwrap());

        // Segunda falla
        assert!(!rate_limiter.check_limit(&request).await.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limiting_by_endpoint() {
        let mut rate_limiter = RateLimiter::new();

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Endpoint,
            pattern: "/api/sensitive".to_string(),
            requests_per_second: 1,
            burst_size: 1,
        };
        rate_limiter.add_rule(rule);

        let request_sensitive = Request {
            method: "GET".to_string(),
            path: "/api/sensitive".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let request_normal = Request {
            method: "GET".to_string(),
            path: "/api/normal".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Request a endpoint sensible limitado
        assert!(rate_limiter.check_limit(&request_sensitive).await.unwrap());
        assert!(!rate_limiter.check_limit(&request_sensitive).await.unwrap());

        // Request a endpoint normal no limitado
        assert!(rate_limiter.check_limit(&request_normal).await.unwrap());
        assert!(rate_limiter.check_limit(&request_normal).await.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limiting_with_wildcards() {
        let mut rate_limiter = RateLimiter::new();

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Endpoint,
            pattern: "/api/v1/*".to_string(),
            requests_per_second: 1,
            burst_size: 1,
        };
        rate_limiter.add_rule(rule);

        let request1 = Request {
            method: "GET".to_string(),
            path: "/api/v1/users".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let request2 = Request {
            method: "GET".to_string(),
            path: "/api/v1/posts".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Ambos endpoints deberían estar limitados
        assert!(rate_limiter.check_limit(&request1).await.unwrap());
        assert!(!rate_limiter.check_limit(&request1).await.unwrap());

        assert!(rate_limiter.check_limit(&request2).await.unwrap());
        assert!(!rate_limiter.check_limit(&request2).await.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limiting_combined_keys() {
        let mut rate_limiter = RateLimiter::new();

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::IpAndEndpoint,
            pattern: "*".to_string(),
            requests_per_second: 1,
            burst_size: 1,
        };
        rate_limiter.add_rule(rule);

        let request = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Forwarded-For".to_string(), "192.168.1.100".to_string());
                h
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Primera request pasa
        assert!(rate_limiter.check_limit(&request).await.unwrap());

        // Segunda falla (misma IP y endpoint)
        assert!(!rate_limiter.check_limit(&request).await.unwrap());

        // Request con diferente IP debería pasar
        let request_different_ip = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Forwarded-For".to_string(), "192.168.1.101".to_string());
                h
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        assert!(rate_limiter.check_limit(&request_different_ip).await.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limiting_token_expiration() {
        let mut rate_limiter = RateLimiter::new();

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Ip,
            pattern: "*".to_string(),
            requests_per_second: 1,
            burst_size: 1,
        };
        rate_limiter.add_rule(rule);

        let request = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Forwarded-For".to_string(), "192.168.1.100".to_string());
                h
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Consumir el token
        assert!(rate_limiter.check_limit(&request).await.unwrap());
        assert!(!rate_limiter.check_limit(&request).await.unwrap());

        // Simular paso del tiempo (esperar más de 1 segundo)
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Debería permitir request nuevamente
        assert!(rate_limiter.check_limit(&request).await.unwrap());
    }
}

mod routing_tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_routing() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let gateway = ApiGateway::new(config);

        // Agregar ruta de prueba al router
        let route = Route {
            path: "/api/test".to_string(),
            methods: vec!["GET".to_string()],
            service: "test-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        let request = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Debería encontrar la ruta (aunque falle en load balancing por no tener servicios)
        let result = gateway.process_request(request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GatewayError::LoadBalancing(_) => {} // Esperado
            _ => panic!("Expected LoadBalancing error"),
        }
    }

    #[tokio::test]
    async fn test_routing_with_parameters() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let gateway = ApiGateway::new(config);

        let route = Route {
            path: "/api/users/{id}".to_string(),
            methods: vec!["GET".to_string()],
            service: "user-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        let request = Request {
            method: "GET".to_string(),
            path: "/api/users/123".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GatewayError::LoadBalancing(_) => {} // Esperado
            _ => panic!("Expected LoadBalancing error"),
        }
    }

    #[tokio::test]
    async fn test_routing_method_not_allowed() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let gateway = ApiGateway::new(config);

        let route = Route {
            path: "/api/test".to_string(),
            methods: vec!["GET".to_string()],
            service: "test-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        let request = Request {
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GatewayError::Routing(_) => {} // Esperado
            _ => panic!("Expected Routing error"),
        }
    }
}

mod load_balancing_tests {
    use super::*;
    use crate::load_balancer::Backend;

    #[tokio::test]
    async fn test_round_robin_load_balancing() {
        let mut load_balancer = LoadBalancer::new();
        load_balancer.set_strategy(LoadBalancingStrategy::RoundRobin);

        let backends = vec![
            Backend { url: "http://service1:8080".to_string(), weight: 1, healthy: true },
            Backend { url: "http://service2:8080".to_string(), weight: 1, healthy: true },
            Backend { url: "http://service3:8080".to_string(), weight: 1, healthy: true },
        ];

        load_balancer.add_service("test-service".to_string(), backends);

        // Probar distribución round-robin
        let mut selected_backends = Vec::new();
        for _ in 0..6 {
            let backend = load_balancer.select_backend("test-service").unwrap();
            selected_backends.push(backend);
        }

        // Deberíamos ver distribución uniforme
        let service1_count = selected_backends.iter().filter(|b| b.contains("service1")).count();
        let service2_count = selected_backends.iter().filter(|b| b.contains("service2")).count();
        let service3_count = selected_backends.iter().filter(|b| b.contains("service3")).count();

        assert_eq!(service1_count, 2);
        assert_eq!(service2_count, 2);
        assert_eq!(service3_count, 2);
    }

    #[tokio::test]
    async fn test_weighted_load_balancing() {
        let mut load_balancer = LoadBalancer::new();
        load_balancer.set_strategy(LoadBalancingStrategy::Weighted);

        let backends = vec![
            Backend { url: "http://service1:8080".to_string(), weight: 3, healthy: true },
            Backend { url: "http://service2:8080".to_string(), weight: 1, healthy: true },
        ];

        load_balancer.add_service("test-service".to_string(), backends);

        // Probar distribución weighted
        let mut selected_backends = Vec::new();
        for _ in 0..8 {
            let backend = load_balancer.select_backend("test-service").unwrap();
            selected_backends.push(backend);
        }

        let service1_count = selected_backends.iter().filter(|b| b.contains("service1")).count();
        let service2_count = selected_backends.iter().filter(|b| b.contains("service2")).count();

        // service1 debería tener ~6 requests (3/4 del total), service2 ~2
        assert!(service1_count >= 5);
        assert!(service2_count >= 1);
    }

    #[tokio::test]
    async fn test_unhealthy_backend_skip() {
        let mut load_balancer = LoadBalancer::new();
        load_balancer.set_strategy(LoadBalancingStrategy::RoundRobin);

        let backends = vec![
            Backend { url: "http://service1:8080".to_string(), weight: 1, healthy: false },
            Backend { url: "http://service2:8080".to_string(), weight: 1, healthy: true },
        ];

        load_balancer.add_service("test-service".to_string(), backends);

        // Solo debería seleccionar el backend healthy
        for _ in 0..5 {
            let backend = load_balancer.select_backend("test-service").unwrap();
            assert!(backend.contains("service2"));
        }
    }
}

mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_request_flow() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let mut gateway = ApiGateway::new(config);

        // Configurar routing
        let route = Route {
            path: "/api/health".to_string(),
            methods: vec!["GET".to_string()],
            service: "health-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        // Configurar load balancing
        let backends = vec![
            Backend { url: "http://health1:8080".to_string(), weight: 1, healthy: true },
        ];
        gateway.load_balancer.write().await.add_service("health-service".to_string(), backends);

        // Agregar plugin de logging
        let execution_count = Arc::new(std::sync::Mutex::new(0));
        let plugin = MockPlugin {
            should_fail: false,
            execution_count: execution_count.clone(),
        };
        gateway = gateway.add_plugin(plugin);

        let request = Request {
            method: "GET".to_string(),
            path: "/api/health".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(request).await;

        // Debería procesar exitosamente
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, 200);

        // Plugin debería haber sido ejecutado
        assert_eq!(*execution_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_rate_limiting_integration() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: Some("2req/sec".to_string()),
            auth: None,
            services: HashMap::new(),
        };

        let mut gateway = ApiGateway::new(config);

        // Configurar rate limiting agresivo para test
        {
            let mut rate_limiter = gateway.rate_limiter.write().await;
            let rule = RateLimitRule {
                key_type: RateLimitKeyType::Ip,
                pattern: "*".to_string(),
                requests_per_second: 1,
                burst_size: 1,
            };
            rate_limiter.add_rule(rule);
        }

        // Configurar routing básico
        let route = Route {
            path: "/api/test".to_string(),
            methods: vec!["GET".to_string()],
            service: "test-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        let backends = vec![
            Backend { url: "http://test:8080".to_string(), weight: 1, healthy: true },
        ];
        gateway.load_balancer.write().await.add_service("test-service".to_string(), backends);

        let request = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Forwarded-For".to_string(), "192.168.1.100".to_string());
                h
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        // Primera request debería pasar
        let result1 = gateway.process_request(request.clone()).await;
        assert!(result1.is_ok());

        // Segunda debería ser rate limited
        let result2 = gateway.process_request(request).await;
        assert!(result2.is_err());
        match result2.err().unwrap() {
            GatewayError::RateLimitExceeded => {} // Esperado
            _ => panic!("Expected RateLimitExceeded error"),
        }
    }

    #[tokio::test]
    async fn test_plugin_failure_propagation() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let mut gateway = ApiGateway::new(config);

        // Agregar plugin que falla
        let plugin = MockPlugin {
            should_fail: true,
            execution_count: Arc::new(std::sync::Mutex::new(0)),
        };
        gateway = gateway.add_plugin(plugin);

        let request = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GatewayError::Internal(_) => {} // Esperado
            _ => panic!("Expected Internal error"),
        }
    }
}

mod concurrency_tests {
    use super::*;
    use tokio::task;

    #[tokio::test]
    async fn test_concurrent_requests() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let gateway = Arc::new(ApiGateway::new(config));

        // Configurar routing
        let route = Route {
            path: "/api/test".to_string(),
            methods: vec!["GET".to_string()],
            service: "test-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        let backends = vec![
            Backend { url: "http://test:8080".to_string(), weight: 1, healthy: true },
        ];
        gateway.load_balancer.write().await.add_service("test-service".to_string(), backends);

        // Ejecutar múltiples requests concurrentemente
        let mut handles = vec![];

        for i in 0..10 {
            let gateway_clone = Arc::clone(&gateway);
            let handle = task::spawn(async move {
                let request = Request {
                    method: "GET".to_string(),
                    path: "/api/test".to_string(),
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("X-Request-Id".to_string(), format!("req-{}", i));
                        h
                    },
                    body: None,
                    query_params: HashMap::new(),
                    path_params: HashMap::new(),
                };

                gateway_clone.process_request(request).await
            });
            handles.push(handle);
        }

        // Esperar que todas las requests terminen
        let mut success_count = 0;
        for handle in handles {
            let result = handle.await.unwrap();
            if result.is_ok() {
                success_count += 1;
            }
        }

        // Todas deberían haber tenido éxito
        assert_eq!(success_count, 10);
    }

    #[tokio::test]
    async fn test_rate_limiting_concurrency() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: Some("5req/sec".to_string()),
            auth: None,
            services: HashMap::new(),
        };

        let gateway = Arc::new(ApiGateway::new(config));

        // Configurar rate limiting
        {
            let mut rate_limiter = gateway.rate_limiter.write().await;
            let rule = RateLimitRule {
                key_type: RateLimitKeyType::Ip,
                pattern: "*".to_string(),
                requests_per_second: 2,
                burst_size: 2,
            };
            rate_limiter.add_rule(rule);
        }

        // Configurar routing
        let route = Route {
            path: "/api/test".to_string(),
            methods: vec!["GET".to_string()],
            service: "test-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        let backends = vec![
            Backend { url: "http://test:8080".to_string(), weight: 1, healthy: true },
        ];
        gateway.load_balancer.write().await.add_service("test-service".to_string(), backends);

        // Ejecutar requests concurrentemente desde la misma IP
        let mut handles = vec![];

        for i in 0..5 {
            let gateway_clone = Arc::clone(&gateway);
            let handle = task::spawn(async move {
                let request = Request {
                    method: "GET".to_string(),
                    path: "/api/test".to_string(),
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("X-Forwarded-For".to_string(), "192.168.1.100".to_string());
                        h
                    },
                    body: None,
                    query_params: HashMap::new(),
                    path_params: HashMap::new(),
                };

                gateway_clone.process_request(request).await
            });
            handles.push(handle);
        }

        // Esperar resultados
        let mut success_count = 0;
        let mut rate_limit_count = 0;

        for handle in handles {
            let result = handle.await.unwrap();
            match result {
                Ok(_) => success_count += 1,
                Err(GatewayError::RateLimitExceeded) => rate_limit_count += 1,
                _ => {} // Otros errores
            }
        }

        // Deberíamos tener algunas requests exitosas y algunas rate limited
        assert!(success_count >= 1);
        assert!(rate_limit_count >= 1);
        assert_eq!(success_count + rate_limit_count, 5);
    }
}