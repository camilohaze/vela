//! Metrics y Observabilidad para API Gateway
//!
//! Implementa métricas de observabilidad con:
//! - Contadores de requests/responses
//! - Latencias y percentiles
//! - Errores por tipo
//! - Health checks

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use crate::gateway::{Request, Response};

/// Métricas del gateway
#[derive(Debug, Clone)]
pub struct GatewayMetrics {
    pub total_requests: u64,
    pub total_responses: u64,
    pub total_errors: u64,
    pub requests_by_method: HashMap<String, u64>,
    pub requests_by_path: HashMap<String, u64>,
    pub errors_by_type: HashMap<String, u64>,
    pub response_times: Vec<Duration>,
    pub active_connections: u64,
}

/// Estadísticas de un endpoint
#[derive(Debug, Clone)]
pub struct EndpointStats {
    pub path: String,
    pub method: String,
    pub request_count: u64,
    pub error_count: u64,
    pub avg_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
}

/// Sistema de métricas
#[derive(Debug)]
pub struct Metrics {
    metrics: Arc<RwLock<GatewayMetrics>>,
    start_time: Instant,
}

impl Metrics {
    /// Crear nuevo sistema de métricas
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(GatewayMetrics {
                total_requests: 0,
                total_responses: 0,
                total_errors: 0,
                requests_by_method: HashMap::new(),
                requests_by_path: HashMap::new(),
                errors_by_type: HashMap::new(),
                response_times: Vec::new(),
                active_connections: 0,
            })),
            start_time: Instant::now(),
        }
    }

    /// Registrar un request
    pub async fn record_request(&self, request: &Request, response: &Response) {
        let mut metrics = self.metrics.write().await;

        metrics.total_requests += 1;
        *metrics.requests_by_method.entry(request.method.clone()).or_insert(0) += 1;
        *metrics.requests_by_path.entry(request.path.clone()).or_insert(0) += 1;

        if response.status >= 400 {
            metrics.total_errors += 1;
            let error_type = match response.status {
                400..=499 => "client_error",
                500..=599 => "server_error",
                _ => "unknown_error",
            };
            *metrics.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
        } else {
            metrics.total_responses += 1;
        }
    }

    /// Registrar tiempo de respuesta
    pub async fn record_response_time(&self, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.response_times.push(duration);

        // Mantener solo las últimas 1000 mediciones para no consumir mucha memoria
        if metrics.response_times.len() > 1000 {
            metrics.response_times.remove(0);
        }
    }

    /// Incrementar conexiones activas
    pub async fn increment_active_connections(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.active_connections += 1;
    }

    /// Decrementar conexiones activas
    pub async fn decrement_active_connections(&self) {
        let mut metrics = self.metrics.write().await;
        if metrics.active_connections > 0 {
            metrics.active_connections -= 1;
        }
    }

    /// Obtener métricas actuales
    pub async fn get_metrics(&self) -> GatewayMetrics {
        self.metrics.read().await.clone()
    }

    /// Calcular percentiles de response time
    pub async fn calculate_percentiles(&self) -> (Duration, Duration, Duration) {
        let metrics = self.metrics.read().await;

        if metrics.response_times.is_empty() {
            return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
        }

        let mut times: Vec<Duration> = metrics.response_times.clone();
        times.sort();

        let p50_idx = (times.len() as f64 * 0.5) as usize;
        let p95_idx = (times.len() as f64 * 0.95) as usize;
        let p99_idx = (times.len() as f64 * 0.99) as usize;

        let p50 = times.get(p50_idx).cloned().unwrap_or(Duration::ZERO);
        let p95 = times.get(p95_idx).cloned().unwrap_or(Duration::ZERO);
        let p99 = times.get(p99_idx).cloned().unwrap_or(Duration::ZERO);

        (p50, p95, p99)
    }

    /// Obtener uptime del gateway
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Generar reporte de métricas en formato Prometheus
    pub async fn prometheus_metrics(&self) -> String {
        let metrics = self.get_metrics().await;
        let uptime = self.uptime().as_secs();
        let (p50, p95, p99) = self.calculate_percentiles().await;

        format!(
            "# HELP vela_gateway_requests_total Total number of requests processed
# TYPE vela_gateway_requests_total counter
vela_gateway_requests_total {} 

# HELP vela_gateway_responses_total Total number of successful responses
# TYPE vela_gateway_responses_total counter
vela_gateway_responses_total {}

# HELP vela_gateway_errors_total Total number of errors
# TYPE vela_gateway_errors_total counter
vela_gateway_errors_total {}

# HELP vela_gateway_active_connections Number of active connections
# TYPE vela_gateway_active_connections gauge
vela_gateway_active_connections {}

# HELP vela_gateway_uptime_seconds Gateway uptime in seconds
# TYPE vela_gateway_uptime_seconds counter
vela_gateway_uptime_seconds {}

# HELP vela_gateway_response_time_p50 Response time P50 in milliseconds
# TYPE vela_gateway_response_time_p50 gauge
vela_gateway_response_time_p50 {}

# HELP vela_gateway_response_time_p95 Response time P95 in milliseconds
# TYPE vela_gateway_response_time_p95 gauge
vela_gateway_response_time_p95 {}

# HELP vela_gateway_response_time_p99 Response time P99 in milliseconds
# TYPE vela_gateway_response_time_p99 gauge
vela_gateway_response_time_p99 {}
",
            metrics.total_requests,
            metrics.total_responses,
            metrics.total_errors,
            metrics.active_connections,
            uptime,
            p50.as_millis(),
            p95.as_millis(),
            p99.as_millis()
        )
    }

    /// Obtener estadísticas por endpoint
    pub async fn get_endpoint_stats(&self) -> Vec<EndpointStats> {
        let metrics = self.get_metrics().await;
        let mut stats = Vec::new();

        for (path, count) in &metrics.requests_by_path {
            // Para simplificar, asumimos método GET
            // En implementación real, trackear por path+method
            let error_count = 0; // Implementar lógica real
            let avg_response_time = Duration::from_millis(100); // Mock
            let p95_response_time = Duration::from_millis(200); // Mock
            let p99_response_time = Duration::from_millis(500); // Mock

            stats.push(EndpointStats {
                path: path.clone(),
                method: "GET".to_string(),
                request_count: *count,
                error_count,
                avg_response_time,
                p95_response_time,
                p99_response_time,
            });
        }

        stats
    }

    /// Health check del gateway
    pub async fn health_check(&self) -> HealthStatus {
        let metrics = self.get_metrics().await;
        let uptime = self.uptime();

        // Lógica simple de health check
        let error_rate = if metrics.total_requests > 0 {
            metrics.total_errors as f64 / metrics.total_requests as f64
        } else {
            0.0
        };

        if error_rate > 0.5 {
            HealthStatus::Unhealthy("High error rate".to_string())
        } else if uptime < Duration::from_secs(30) {
            HealthStatus::Starting
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Estado de health del gateway
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Starting,
    Unhealthy(String),
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::Request;
    use std::collections::HashMap;

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

    fn create_test_response(status: u16) -> Response {
        Response {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = Metrics::new();
        let m = metrics.get_metrics().await;
        assert_eq!(m.total_requests, 0);
        assert_eq!(m.total_responses, 0);
        assert_eq!(m.total_errors, 0);
    }

    #[tokio::test]
    async fn test_record_successful_request() {
        let metrics = Metrics::new();
        let request = create_test_request("GET", "/api/users");
        let response = create_test_response(200);

        metrics.record_request(&request, &response).await;

        let m = metrics.get_metrics().await;
        assert_eq!(m.total_requests, 1);
        assert_eq!(m.total_responses, 1);
        assert_eq!(m.total_errors, 0);
        assert_eq!(m.requests_by_method.get("GET"), Some(&1));
        assert_eq!(m.requests_by_path.get("/api/users"), Some(&1));
    }

    #[tokio::test]
    async fn test_record_error_request() {
        let metrics = Metrics::new();
        let request = create_test_request("POST", "/api/users");
        let response = create_test_response(500);

        metrics.record_request(&request, &response).await;

        let m = metrics.get_metrics().await;
        assert_eq!(m.total_requests, 1);
        assert_eq!(m.total_responses, 0);
        assert_eq!(m.total_errors, 1);
        assert_eq!(m.errors_by_type.get("server_error"), Some(&1));
    }

    #[tokio::test]
    async fn test_response_time_recording() {
        let metrics = Metrics::new();

        metrics.record_response_time(Duration::from_millis(100)).await;
        metrics.record_response_time(Duration::from_millis(200)).await;
        metrics.record_response_time(Duration::from_millis(150)).await;

        let (p50, p95, p99) = metrics.calculate_percentiles().await;
        assert_eq!(p50.as_millis(), 150); // Mediana de [100, 150, 200]
    }

    #[tokio::test]
    async fn test_active_connections() {
        let metrics = Metrics::new();

        metrics.increment_active_connections().await;
        metrics.increment_active_connections().await;

        {
            let m = metrics.get_metrics().await;
            assert_eq!(m.active_connections, 2);
        }

        metrics.decrement_active_connections().await;

        {
            let m = metrics.get_metrics().await;
            assert_eq!(m.active_connections, 1);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let metrics = Metrics::new();

        // Initially starting
        let status = metrics.health_check().await;
        assert_eq!(status, HealthStatus::Starting);

        // Simulate some time passing and add successful requests
        tokio::time::sleep(Duration::from_secs(35)).await; // Wait longer than 30 seconds

        for _ in 0..10 {
            let request = create_test_request("GET", "/health");
            let response = create_test_response(200);
            metrics.record_request(&request, &response).await;
        }

        let status = metrics.health_check().await;
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_uptime() {
        let metrics = Metrics::new();
        let uptime = metrics.uptime();
        assert!(uptime.as_millis() >= 0);
    }
}