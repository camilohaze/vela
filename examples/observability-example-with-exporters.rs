//! Example: Complete observability setup with exporters
//!
//! This example demonstrates how to set up the complete Vela observability
//! stack including metrics collection, distributed tracing, and exporters
//! for Prometheus, Jaeger, and Grafana integration.

use std::time::Duration;
use vela_runtime::observability::{
    ObservabilityConfig, MetricsConfig, TracingConfig, ExporterConfig,
    init_observability, shutdown_observability, get_metrics, get_tracer
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Vela Observability Example with Exporters");

    // Configure complete observability system
    let config = ObservabilityConfig {
        tracing: TracingConfig {
            service_name: "vela-observability-example".to_string(),
            service_version: "1.0.0".to_string(),
            sampling_ratio: 1.0, // Sample all traces in development
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            ..Default::default()
        },
        metrics: MetricsConfig {
            service_name: "vela-observability-example".to_string(),
            default_labels: [
                ("environment".to_string(), "development".to_string()),
                ("version".to_string(), "1.0.0".to_string()),
            ].into_iter().collect(),
            ..Default::default()
        },
        exporters: ExporterConfig {
            prometheus_addr: "127.0.0.1:9090".parse()?,
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            service_name: "vela-observability-example".to_string(),
            service_version: "1.0.0".to_string(),
        },
        ..Default::default()
    };

    // Initialize observability
    println!("📊 Initializing observability system...");
    init_observability(config).await?;
    println!("✅ Observability initialized successfully");

    // Get metrics registry and register some metrics
    if let Some(metrics) = get_metrics().await {
        println!("📈 Registering metrics...");

        // Register HTTP request counter
        metrics.register_counter("http_requests_total", "Total HTTP requests").await?;
        let request_counter = metrics.get_counter("http_requests_total").unwrap();

        // Register response time histogram
        metrics.register_histogram("http_request_duration_seconds", "HTTP request duration").await?;
        let duration_histogram = metrics.get_histogram("http_request_duration_seconds").unwrap();

        // Register active connections gauge
        metrics.register_gauge("active_connections", "Number of active connections").await?;
        let active_connections = metrics.get_gauge("active_connections").unwrap();

        println!("✅ Metrics registered");
    }

    // Get tracer and create some spans
    if let Some(tracer) = get_tracer().await {
        println!("🔍 Creating trace spans...");

        // Create a root span for the request
        let mut root_span = tracer.start_span("handle_request");
        root_span.set_attribute("method", "GET".into());
        root_span.set_attribute("path", "/api/users".into());

        // Simulate some work
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create child span for database operation
        let mut db_span = tracer.start_span("query_database");
        db_span.set_attribute("table", "users".into());
        db_span.set_attribute("operation", "SELECT".into());

        // Simulate database work
        tokio::time::sleep(Duration::from_millis(50)).await;

        db_span.end();

        // Create child span for response processing
        let mut response_span = tracer.start_span("process_response");
        response_span.set_attribute("status_code", 200.into());
        response_span.set_attribute("response_size", 1024.into());

        // Simulate response processing
        tokio::time::sleep(Duration::from_millis(25)).await;

        response_span.end();
        root_span.end();

        println!("✅ Trace spans created and exported");
    }

    println!("🌐 Exporters running:");
    println!("   📊 Prometheus metrics: http://127.0.0.1:9090/metrics");
    println!("   🏥 Health check: http://127.0.0.1:9090/health");
    println!("   🔍 Jaeger traces: http://localhost:14268 (if running)");
    println!("");
    println!("📋 Grafana setup:");
    println!("   1. Add Prometheus data source: http://127.0.0.1:9090");
    println!("   2. Import dashboard from: docs/features/VELA-602/examples/grafana-dashboard.json");
    println!("");
    println!("⏹️  Press Ctrl+C to stop...");

    // Keep running to allow scraping
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 Shutting down observability system...");

    // Shutdown
    shutdown_observability().await?;
    println!("✅ Observability shutdown complete");

    Ok(())
}