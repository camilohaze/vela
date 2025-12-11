//! Observability module for Vela services
//!
//! This module provides comprehensive observability features including:
//! - Distributed tracing with OpenTelemetry
//! - Metrics collection (Prometheus-compatible)
//! - Structured logging with multiple sinks
//! - Metrics and trace exporters (Prometheus, Jaeger, Grafana)

pub mod tracing;
pub mod metrics;
pub mod logging;
pub mod exporters;

// Re-export main types for convenience
pub use tracing::{Tracer, Span, SpanContext, TracingConfig, init_tracing, get_tracer, get_propagation};
pub use metrics::{MetricsRegistry, Counter, Gauge, Histogram, Summary, MetricsConfig, init_metrics, get_metrics};
pub use logging::{Logger, LogRecord, Level, LoggerConfig, LogSink, ConsoleSink, FileSink, MemorySink, init_logging, get_logger, info, error};
pub use exporters::{ExporterRegistry, ExporterConfig, PrometheusExporter, JaegerExporter, GrafanaIntegration, init_exporters, get_exporter_registry, shutdown_exporters};

/// Configuration for the entire observability system
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Tracing configuration
    pub tracing: TracingConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Logging configuration
    pub logging: LoggerConfig,
    /// Exporters configuration
    pub exporters: ExporterConfig,
    /// Whether to enable observability (can be disabled for testing)
    pub enabled: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            tracing: TracingConfig::default(),
            metrics: MetricsConfig::default(),
            logging: LoggerConfig::default(),
            exporters: ExporterConfig::default(),
            enabled: true,
        }
    }
}

/// Initialize the entire observability system
pub async fn init_observability(config: ObservabilityConfig) -> Result<(), Box<dyn std::error::Error>> {
    if !config.enabled {
        return Ok(());
    }

    // Initialize tracing
    init_tracing(config.tracing).await?;

    // Initialize metrics
    init_metrics(config.metrics).await?;

    // Initialize exporters
    init_exporters(config.exporters).await?;

    // Initialize logging with console sink by default
    let sinks: Vec<Box<dyn LogSink>> = vec![Box::new(ConsoleSink::new())];
    init_logging(config.logging, sinks).await?;

    Ok(())
}

/// Shutdown the observability system
pub async fn shutdown_observability() -> Result<(), Box<dyn std::error::Error>> {
    // Shutdown exporters first
    shutdown_exporters().await?;

    if let Some(logger) = get_logger().await {
        logger.flush().await?;
        logger.close().await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_initialization() {
        let config = ObservabilityConfig {
            enabled: false, // Disable for testing
            ..Default::default()
        };

        let result = init_observability(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_without_initialization() {
        let result = shutdown_observability().await;
        assert!(result.is_ok());
    }
}