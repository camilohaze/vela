//! Metrics and tracing exporters for Vela observability
//!
//! This module provides exporters for different observability backends:
//! - Prometheus exporter for metrics
//! - Jaeger exporter for traces
//! - Grafana integration via Prometheus

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

/// Exporter configuration
#[derive(Debug, Clone)]
pub struct ExporterConfig {
    /// Prometheus metrics endpoint
    pub prometheus_addr: SocketAddr,
    /// Jaeger endpoint for traces
    pub jaeger_endpoint: Option<String>,
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            prometheus_addr: "127.0.0.1:9090".parse().unwrap(),
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            service_name: "vela-service".to_string(),
            service_version: "1.0.0".to_string(),
        }
    }
}

/// Global exporter registry
pub struct ExporterRegistry {
    config: ExporterConfig,
    prometheus_server: Option<tokio::task::JoinHandle<()>>,
    jaeger_client: Option<reqwest::Client>,
}

impl ExporterRegistry {
    /// Create a new exporter registry
    pub fn new(config: ExporterConfig) -> Self {
        Self {
            config,
            prometheus_server: None,
            jaeger_client: Some(reqwest::Client::new()),
        }
    }

    /// Start Prometheus metrics server
    pub async fn start_prometheus_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.config.prometheus_addr;

        // Create metrics endpoint
        let metrics_route = warp::path("metrics")
            .and_then(|| async {
                let metrics = generate_prometheus_metrics().await;
                Ok::<_, warp::Rejection>(warp::reply::with_header(
                    metrics,
                    "content-type",
                    "text/plain; version=0.0.4; charset=utf-8"
                ))
            });

        let health_route = warp::path("health")
            .map(|| "OK");

        let routes = metrics_route.or(health_route);

        let server = tokio::spawn(async move {
            println!("Starting Prometheus metrics server on {}", addr);
            warp::serve(routes)
                .run(addr)
                .await;
        });

        self.prometheus_server = Some(server);
        Ok(())
    }

    /// Stop Prometheus metrics server
    pub async fn stop_prometheus_server(&mut self) {
        if let Some(server) = self.prometheus_server.take() {
            server.abort();
            println!("Prometheus metrics server stopped");
        }
    }

    /// Export span to Jaeger
    pub async fn export_span_to_jaeger(
        &self,
        span_name: &str,
        trace_id: &str,
        span_id: &str,
        start_time: u64,
        duration: u64,
        attributes: &HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(client), Some(endpoint)) = (&self.jaeger_client, &self.config.jaeger_endpoint) {
            // Create Jaeger Thrift format span
            let jaeger_span = create_jaeger_span(
                span_name,
                trace_id,
                span_id,
                start_time,
                duration,
                attributes,
                &self.config.service_name,
            );

            // Send to Jaeger
            let response = client
                .post(endpoint)
                .header("Content-Type", "application/x-thrift")
                .body(jaeger_span)
                .send()
                .await?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(format!("Jaeger export failed: {}", response.status()).into())
            }
        } else {
            // Jaeger not configured, skip
            Ok(())
        }
    }

    /// Get Prometheus endpoint URL
    pub fn prometheus_endpoint(&self) -> String {
        format!("http://{}/metrics", self.config.prometheus_addr)
    }

    /// Get Jaeger endpoint
    pub fn jaeger_endpoint(&self) -> Option<&str> {
        self.config.jaeger_endpoint.as_deref()
    }
}

/// Generate metrics in Prometheus format
async fn generate_prometheus_metrics() -> String {
    let mut output = String::new();

    // Try to get metrics from global registry
    if let Some(registry) = crate::observability::get_metrics().await {
        output = registry.export_prometheus().await;
    } else {
        // Fallback to basic metrics if registry not initialized
        output.push_str("# HELP vela_up Service uptime\n");
        output.push_str("# TYPE vela_up gauge\n");
        output.push_str("vela_up 1\n");
    }

    output
}

/// Create Jaeger span in Thrift format
fn create_jaeger_span(
    span_name: &str,
    trace_id: &str,
    span_id: &str,
    start_time: u64,
    duration: u64,
    attributes: &HashMap<String, String>,
    service_name: &str,
) -> Vec<u8> {
    // This is a simplified implementation
    // In a real implementation, you'd use the jaeger-client crate
    // or implement proper Jaeger Thrift protocol

    // For now, return empty vec to indicate not implemented
    // TODO: Implement proper Jaeger Thrift encoding
    Vec::new()
}

/// Prometheus exporter for metrics
pub struct PrometheusExporter {
    registry: Arc<ExporterRegistry>,
}

impl PrometheusExporter {
    pub fn new(registry: Arc<ExporterRegistry>) -> Self {
        Self { registry }
    }

    /// Export metrics to Prometheus format
    pub async fn export(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Get metrics from registry and format as Prometheus
        generate_prometheus_metrics().await
    }
}

/// Jaeger exporter for traces
pub struct JaegerExporter {
    registry: Arc<ExporterRegistry>,
}

impl JaegerExporter {
    pub fn new(registry: Arc<ExporterRegistry>) -> Self {
        Self { registry }
    }

    /// Export span to Jaeger
    pub async fn export_span(
        &self,
        span_name: &str,
        trace_id: &str,
        span_id: &str,
        start_time: u64,
        duration: u64,
        attributes: &HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.registry
            .export_span_to_jaeger(span_name, trace_id, span_id, start_time, duration, attributes)
            .await
    }
}

/// Grafana integration (via Prometheus)
pub struct GrafanaIntegration {
    prometheus_endpoint: String,
    dashboards: HashMap<String, String>,
}

impl GrafanaIntegration {
    pub fn new(prometheus_endpoint: String) -> Self {
        Self {
            prometheus_endpoint,
            dashboards: HashMap::new(),
        }
    }

    /// Add a Grafana dashboard
    pub fn add_dashboard(&mut self, name: &str, dashboard_json: &str) {
        self.dashboards.insert(name.to_string(), dashboard_json.to_string());
    }

    /// Get dashboard JSON
    pub fn get_dashboard(&self, name: &str) -> Option<&str> {
        self.dashboards.get(name).map(|s| s.as_str())
    }

    /// Generate default Vela dashboard
    pub fn generate_default_dashboard(&self) -> String {
        format!(r#"
{{
  "dashboard": {{
    "title": "Vela Service Observability",
    "tags": ["vela", "observability"],
    "timezone": "browser",
    "panels": [
      {{
        "title": "HTTP Requests per Second",
        "type": "graph",
        "targets": [{{
          "expr": "rate(vela_http_requests_total[5m])",
          "legendFormat": "{{{{method}}}} {{{{endpoint}}}}"
        }}]
      }},
      {{
        "title": "HTTP Request Duration",
        "type": "heatmap",
        "targets": [{{
          "expr": "vela_http_request_duration_seconds",
          "legendFormat": "Duration"
        }}]
      }},
      {{
        "title": "Service Uptime",
        "type": "stat",
        "targets": [{{
          "expr": "vela_up",
          "legendFormat": "Uptime"
        }}]
      }}
    ],
    "time": {{
      "from": "now-1h",
      "to": "now"
    }},
    "refresh": "5s"
  }}
}}
"#)
    }

    /// Get Prometheus data source configuration
    pub fn prometheus_data_source_config(&self) -> String {
        format!(r#"
{{
  "name": "Vela Prometheus",
  "type": "prometheus",
  "url": "{}",
  "access": "proxy",
  "isDefault": true
}}
"#, self.prometheus_endpoint)
    }
}

/// Global exporter instance
static mut EXPORTER_REGISTRY: Option<Arc<RwLock<ExporterRegistry>>> = None;

/// Initialize global exporter registry
pub async fn init_exporters(config: ExporterConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = ExporterRegistry::new(config);
    registry.start_prometheus_server().await?;

    let registry = Arc::new(RwLock::new(registry));
    unsafe {
        EXPORTER_REGISTRY = Some(registry);
    }

    Ok(())
}

/// Get global exporter registry
pub async fn get_exporter_registry() -> Option<Arc<RwLock<ExporterRegistry>>> {
    unsafe { EXPORTER_REGISTRY.clone() }
}

/// Shutdown exporters
pub async fn shutdown_exporters() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(registry) = unsafe { &EXPORTER_REGISTRY } {
        let mut registry = registry.write().await;
        registry.stop_prometheus_server().await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_exporter_registry_creation() {
        let config = ExporterConfig::default();
        let registry = ExporterRegistry::new(config);
        assert_eq!(registry.config.service_name, "vela-service");
    }

    #[tokio::test]
    async fn test_prometheus_exporter() {
        let config = ExporterConfig::default();
        let registry = Arc::new(ExporterRegistry::new(config));
        let exporter = PrometheusExporter::new(registry);

        let metrics = exporter.export().await.unwrap();
        assert!(metrics.contains("# HELP") || metrics.contains("# TYPE") || metrics.contains("vela_up"));
    }

    #[tokio::test]
    async fn test_jaeger_exporter_creation() {
        let config = ExporterConfig::default();
        let registry = Arc::new(ExporterRegistry::new(config));
        let exporter = JaegerExporter::new(registry);

        // Test that exporter can be created
        assert!(exporter.registry.config.jaeger_endpoint.is_some());
    }

    #[tokio::test]
    async fn test_grafana_integration() {
        let integration = GrafanaIntegration::new("http://localhost:9090".to_string());

        let dashboard = integration.generate_default_dashboard();
        assert!(dashboard.contains("Vela Service Observability"));
        assert!(dashboard.contains("HTTP Requests per Second"));

        let data_source = integration.prometheus_data_source_config();
        assert!(data_source.contains("http://localhost:9090"));
        assert!(data_source.contains("Vela Prometheus"));
    }

    #[tokio::test]
    async fn test_exporter_config_defaults() {
        let config = ExporterConfig::default();
        assert_eq!(config.service_name, "vela-service");
        assert_eq!(config.service_version, "1.0.0");
        assert!(config.jaeger_endpoint.is_some());
        assert_eq!(config.prometheus_addr.port(), 9090);
    }

    #[tokio::test]
    async fn test_global_exporter_initialization() {
        // Test that global initialization works
        let config = ExporterConfig {
            prometheus_addr: "127.0.0.1:9091".parse().unwrap(),
            ..Default::default()
        };

        let result = init_exporters(config).await;
        assert!(result.is_ok());

        // Test shutdown
        let shutdown_result = shutdown_exporters().await;
        assert!(shutdown_result.is_ok());
    }

    #[tokio::test]
    async fn test_prometheus_endpoint_url_generation() {
        let config = ExporterConfig {
            prometheus_addr: "192.168.1.100:8080".parse().unwrap(),
            ..Default::default()
        };
        let registry = ExporterRegistry::new(config);
        assert_eq!(registry.prometheus_endpoint(), "http://192.168.1.100:8080/metrics");
    }
}