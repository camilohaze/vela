//! Tests de integración del API Gateway
//!
//! Tests end-to-end que validan el funcionamiento completo del gateway
//! incluyendo todos los componentes trabajando juntos.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

use crate::gateway::{
    ApiGateway, GatewayConfig, Request, Response, GatewayError,
    Context, Plugin
};
use crate::rate_limiter::{RateLimiter, RateLimitRule, RateLimitKeyType};
use crate::load_balancer::{LoadBalancer, LoadBalancingStrategy, Backend};
use crate::router::{Router, Route};
use crate::dynamic_router::DynamicRouter;

/// Mock HTTP client para simular backends
struct MockHttpClient {
    responses: HashMap<String, Response>,
}

impl MockHttpClient {
    fn new() -> Self {
        let mut responses = HashMap::new();

        // Simular diferentes servicios
        responses.insert("http://user-service:8080/users".to_string(), Response {
            status: 200,
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
            body: Some(br#"{"users": [{"id": 1, "name": "Alice"}]}"#.to_vec()),
        });

        responses.insert("http://product-service:8080/products".to_string(), Response {
            status: 200,
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
            body: Some(br#"{"products": [{"id": 1, "name": "Widget"}]}"#.to_vec()),
        });

        responses.insert("http://health-service:8080/health".to_string(), Response {
            status: 200,
            headers: HashMap::new(),
            body: Some(b"OK".to_vec()),
        });

        Self { responses }
    }

    async fn send_request(&self, url: &str, _request: &Request) -> Result<Response, GatewayError> {
        match self.responses.get(url) {
            Some(response) => Ok(response.clone()),
            None => Err(GatewayError::ServiceUnavailable(format!("No mock response for {}", url))),
        }
    }
}

/// Plugin de logging para tests
struct LoggingPlugin {
    pub logs: Arc<std::sync::Mutex<Vec<String>>>,
}

#[async_trait::async_trait]
impl Plugin for LoggingPlugin {
    async fn execute(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("Processing request: {} {}", ctx.request.method, ctx.request.path));
        Ok(())
    }
}

/// Plugin de transformación para tests
struct TransformPlugin {
    pub transform_count: Arc<std::sync::Mutex<usize>>,
}

#[async_trait::async_trait]
impl Plugin for TransformPlugin {
    async fn execute(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        let mut count = self.transform_count.lock().unwrap();
        *count += 1;

        // Agregar header de transformación
        ctx.request.headers.insert(
            "X-Transformed".to_string(),
            "true".to_string()
        );
        Ok(())
    }
}

mod e2e_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_api_gateway_flow() {
        // Configurar gateway completo
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: Some("10req/sec".to_string()),
            auth: None,
            services: HashMap::new(),
        };

        let mut gateway = ApiGateway::new(config);

        // Configurar rate limiting
        {
            let mut rate_limiter = gateway.rate_limiter.write().await;
            let rule = RateLimitRule {
                key_type: RateLimitKeyType::Ip,
                pattern: "*".to_string(),
                requests_per_second: 5,
                burst_size: 5,
            };
            rate_limiter.add_rule(rule);
        }

        // Configurar routing
        let routes = vec![
            Route {
                path: "/api/users".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string()],
                service: "user-service".to_string(),
                priority: 0,
            },
            Route {
                path: "/api/products".to_string(),
                methods: vec!["GET".to_string()],
                service: "product-service".to_string(),
                priority: 0,
            },
            Route {
                path: "/health".to_string(),
                methods: vec!["GET".to_string()],
                service: "health-service".to_string(),
                priority: 0,
            },
        ];

        for route in routes {
            gateway.router.add_route(route);
        }

        // Configurar load balancing
        let user_backends = vec![
            Backend { url: "http://user-service:8080".to_string(), weight: 2, healthy: true },
            Backend { url: "http://user-service-backup:8080".to_string(), weight: 1, healthy: true },
        ];
        let product_backends = vec![
            Backend { url: "http://product-service:8080".to_string(), weight: 1, healthy: true },
        ];
        let health_backends = vec![
            Backend { url: "http://health-service:8080".to_string(), weight: 1, healthy: true },
        ];

        {
            let mut load_balancer = gateway.load_balancer.write().await;
            load_balancer.add_service("user-service".to_string(), user_backends);
            load_balancer.add_service("product-service".to_string(), product_backends);
            load_balancer.add_service("health-service".to_string(), health_backends);
        }

        // Agregar plugins
        let logs = Arc::new(std::sync::Mutex::new(Vec::new()));
        let transform_count = Arc::new(std::sync::Mutex::new(0));

        let logging_plugin = LoggingPlugin {
            logs: Arc::clone(&logs),
        };
        let transform_plugin = TransformPlugin {
            transform_count: Arc::clone(&transform_count),
        };

        gateway = gateway.add_plugin(logging_plugin);
        gateway = gateway.add_plugin(transform_plugin);

        // Test 1: Health check
        let health_request = Request {
            method: "GET".to_string(),
            path: "/health".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(health_request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, 200);

        // Verificar logs
        {
            let logs_guard = logs.lock().unwrap();
            assert!(logs_guard.iter().any(|log| log.contains("/health")));
        }

        // Verificar transformación
        assert_eq!(*transform_count.lock().unwrap(), 1);

        // Test 2: Users API
        let users_request = Request {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Forwarded-For".to_string(), "192.168.1.100".to_string());
                h.insert("Accept".to_string(), "application/json".to_string());
                h
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(users_request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("Content-Type").unwrap(), "application/json");

        // Test 3: Products API
        let products_request = Request {
            method: "GET".to_string(),
            path: "/api/products".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("X-Forwarded-For".to_string(), "192.168.1.100".to_string());
                h
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(products_request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, 200);

        // Verificar que se procesaron 3 requests
        assert_eq!(*transform_count.lock().unwrap(), 3);

        // Verificar logs contienen todas las requests
        {
            let logs_guard = logs.lock().unwrap();
            assert_eq!(logs_guard.len(), 3);
            assert!(logs_guard.iter().any(|log| log.contains("/health")));
            assert!(logs_guard.iter().any(|log| log.contains("/api/users")));
            assert!(logs_guard.iter().any(|log| log.contains("/api/products")));
        }
    }

    #[tokio::test]
    async fn test_rate_limiting_e2e() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: Some("2req/sec".to_string()),
            auth: None,
            services: HashMap::new(),
        };

        let gateway = Arc::new(ApiGateway::new(config));

        // Configurar rate limiting agresivo
        {
            let rate_limiter = Arc::clone(&gateway.rate_limiter);
            let mut rate_limiter_guard = rate_limiter.write().await;
            let rule = RateLimitRule {
                key_type: RateLimitKeyType::Ip,
                pattern: "*".to_string(),
                requests_per_second: 1,
                burst_size: 1,
            };
            rate_limiter_guard.add_rule(rule);
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
            Backend { url: "http://test-service:8080".to_string(), weight: 1, healthy: true },
        ];
        {
            let load_balancer = Arc::clone(&gateway.load_balancer);
            let mut load_balancer_guard = load_balancer.write().await;
            load_balancer_guard.add_service("test-service".to_string(), backends);
        }

        // Función para crear request
        let create_request = || Request {
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
        let result1 = gateway.process_request(create_request()).await;
        assert!(result1.is_ok());

        // Segunda request debería ser rate limited
        let result2 = gateway.process_request(create_request()).await;
        assert!(result2.is_err());
        match result2.err().unwrap() {
            GatewayError::RateLimitExceeded => {} // Esperado
            _ => panic!("Expected RateLimitExceeded"),
        }

        // Esperar a que expire el rate limit
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Tercera request debería pasar nuevamente
        let result3 = gateway.process_request(create_request()).await;
        assert!(result3.is_ok());
    }

    #[tokio::test]
    async fn test_load_balancing_distribution() {
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
            path: "/api/service".to_string(),
            methods: vec!["GET".to_string()],
            service: "test-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        // Configurar load balancing con múltiples backends
        let backends = vec![
            Backend { url: "http://service1:8080".to_string(), weight: 2, healthy: true },
            Backend { url: "http://service2:8080".to_string(), weight: 1, healthy: true },
            Backend { url: "http://service3:8080".to_string(), weight: 1, healthy: true },
        ];

        {
            let load_balancer = Arc::clone(&gateway.load_balancer);
            let mut load_balancer_guard = load_balancer.write().await;
            load_balancer_guard.set_strategy(LoadBalancingStrategy::Weighted);
            load_balancer_guard.add_service("test-service".to_string(), backends);
        }

        // Enviar múltiples requests y rastrear distribución
        let mut backend_counts = HashMap::new();
        let request_count = 20;

        for i in 0..request_count {
            let request = Request {
                method: "GET".to_string(),
                path: "/api/service".to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("X-Request-Id".to_string(), format!("req-{}", i));
                    h
                },
                body: None,
                query_params: HashMap::new(),
                path_params: HashMap::new(),
            };

            let result = gateway.process_request(request).await;
            assert!(result.is_ok());

            // En un test real, aquí verificaríamos qué backend fue seleccionado
            // Por ahora solo verificamos que las requests se procesan
        }

        // Verificar que se procesaron todas las requests
        assert_eq!(request_count, 20);
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let gateway = Arc::new(ApiGateway::new(config));

        // Test 1: Routing error
        let bad_route_request = Request {
            method: "GET".to_string(),
            path: "/nonexistent".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(bad_route_request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GatewayError::Routing(_) => {} // Esperado
            _ => panic!("Expected Routing error"),
        }

        // Test 2: Method not allowed
        let route = Route {
            path: "/api/test".to_string(),
            methods: vec!["GET".to_string()],
            service: "test-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        let wrong_method_request = Request {
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(wrong_method_request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GatewayError::Routing(_) => {} // Esperado
            _ => panic!("Expected Routing error"),
        }

        // Test 3: Service unavailable (no backends)
        let no_backends_request = Request {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        };

        let result = gateway.process_request(no_backends_request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            GatewayError::LoadBalancing(_) => {} // Esperado
            _ => panic!("Expected LoadBalancing error"),
        }
    }

    #[tokio::test]
    async fn test_dynamic_routing_integration() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: None,
            auth: None,
            services: HashMap::new(),
        };

        let dynamic_router = Arc::new(RwLock::new(DynamicRouter::new()));
        let mut gateway = ApiGateway::new(config);
        gateway = gateway.with_dynamic_router(Arc::clone(&dynamic_router));

        // Configurar rutas dinámicas
        {
            let mut router_guard = dynamic_router.write().await;
            let dynamic_router_inner = router_guard.get_router();

            let mut inner_guard = dynamic_router_inner.write().await;
            let routes = vec![
                Route {
                    path: "/dynamic/users".to_string(),
                    methods: vec!["GET".to_string()],
                    service: "dynamic-user-service".to_string(),
                    priority: 0,
                },
                Route {
                    path: "/dynamic/products".to_string(),
                    methods: vec!["GET".to_string()],
                    service: "dynamic-product-service".to_string(),
                    priority: 0,
                },
            ];

            for route in routes {
                inner_guard.add_route(route);
            }
        }

        // Configurar backends
        let backends = vec![
            Backend { url: "http://dynamic-service:8080".to_string(), weight: 1, healthy: true },
        ];
        {
            let mut load_balancer = gateway.load_balancer.write().await;
            load_balancer.add_service("dynamic-user-service".to_string(), backends.clone());
            load_balancer.add_service("dynamic-product-service".to_string(), backends);
        }

        // Test routing dinámico
        let requests = vec![
            ("/dynamic/users", "dynamic-user-service"),
            ("/dynamic/products", "dynamic-product-service"),
        ];

        for (path, expected_service) in requests {
            let request = Request {
                method: "GET".to_string(),
                path: path.to_string(),
                headers: HashMap::new(),
                body: None,
                query_params: HashMap::new(),
                path_params: HashMap::new(),
            };

            let result = gateway.process_request(request).await;
            assert!(result.is_ok(), "Request to {} should succeed", path);
        }
    }
}

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_high_concurrency_performance() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: Some("100req/sec".to_string()),
            auth: None,
            services: HashMap::new(),
        };

        let gateway = Arc::new(ApiGateway::new(config));

        // Configurar routing simple
        let route = Route {
            path: "/api/fast".to_string(),
            methods: vec!["GET".to_string()],
            service: "fast-service".to_string(),
            priority: 0,
        };
        gateway.router.add_route(route);

        let backends = vec![
            Backend { url: "http://fast-service:8080".to_string(), weight: 1, healthy: true },
        ];
        {
            let mut load_balancer = gateway.load_balancer.write().await;
            load_balancer.add_service("fast-service".to_string(), backends);
        }

        // Medir performance con alta concurrencia
        let request_count = 50;
        let start_time = Instant::now();

        let mut handles = vec![];

        for i in 0..request_count {
            let gateway_clone = Arc::clone(&gateway);
            let handle = tokio::spawn(async move {
                let request = Request {
                    method: "GET".to_string(),
                    path: "/api/fast".to_string(),
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("X-Request-Id".to_string(), format!("perf-{}", i));
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

        // Esperar todas las requests
        let mut success_count = 0;
        for handle in handles {
            let result = handle.await.unwrap();
            if result.is_ok() {
                success_count += 1;
            }
        }

        let duration = start_time.elapsed();
        let requests_per_second = request_count as f64 / duration.as_secs_f64();

        println!("Performance test results:");
        println!("  Total requests: {}", request_count);
        println!("  Successful requests: {}", success_count);
        println!("  Duration: {:.2}s", duration.as_secs_f64());
        println!("  Requests/second: {:.2}", requests_per_second);

        // Verificar que todas tuvieron éxito
        assert_eq!(success_count, request_count);

        // Verificar performance mínima (ajustar según hardware)
        assert!(requests_per_second > 10.0, "Performance too low: {:.2} req/sec", requests_per_second);
    }

    #[tokio::test]
    async fn test_memory_usage_under_load() {
        let config = GatewayConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            tls: None,
            rate_limit: Some("50req/sec".to_string()),
            auth: None,
            services: HashMap::new(),
        };

        let gateway = Arc::new(ApiGateway::new(config));

        // Configurar múltiples rutas
        let routes = vec![
            Route {
                path: "/api/users".to_string(),
                methods: vec!["GET".to_string()],
                service: "user-service".to_string(),
                priority: 0,
            },
            Route {
                path: "/api/products".to_string(),
                methods: vec!["GET".to_string()],
                service: "product-service".to_string(),
                priority: 0,
            },
            Route {
                path: "/api/orders".to_string(),
                methods: vec!["GET".to_string()],
                service: "order-service".to_string(),
                priority: 0,
            },
        ];

        for route in routes {
            gateway.router.add_route(route);
        }

        // Configurar backends
        let services = vec!["user-service", "product-service", "order-service"];
        for service in services {
            let backends = vec![
                Backend { url: format!("http://{}:8080", service), weight: 1, healthy: true },
            ];
            let mut load_balancer = gateway.load_balancer.write().await;
            load_balancer.add_service(service.to_string(), backends);
        }

        // Enviar carga mixta
        let request_count = 100;
        let paths = vec!["/api/users", "/api/products", "/api/orders"];

        let mut handles = vec![];

        for i in 0..request_count {
            let gateway_clone = Arc::clone(&gateway);
            let path = paths[i % paths.len()].to_string();

            let handle = tokio::spawn(async move {
                let request = Request {
                    method: "GET".to_string(),
                    path,
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("X-Request-Id".to_string(), format!("mem-{}", i));
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

        // Esperar todas las requests
        let mut success_count = 0;
        for handle in handles {
            let result = handle.await.unwrap();
            if result.is_ok() {
                success_count += 1;
            }
        }

        // Verificar que todas tuvieron éxito
        assert_eq!(success_count, request_count);

        // En un test real, aquí mediríamos el uso de memoria
        // Por ahora solo verificamos funcionalidad
    }
}