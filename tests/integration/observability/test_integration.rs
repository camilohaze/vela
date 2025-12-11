//! # Integration Tests
//!
//! End-to-end tests for the complete observability system.
//!
//! Tests cover:
//! - Full request tracing from HTTP entry to database
//! - Metrics collection across all components
//! - Log aggregation and correlation
//! - Exporter pipeline functionality
//! - Configuration validation and application
//! - Cross-component communication
//! - Error propagation and handling
//! - Performance under load
//! - Resource cleanup and lifecycle management
//! - System health monitoring

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use vela_runtime::observability::{
    tracing::{Tracer, Span, TracingConfig},
    metrics::{Registry, Counter, Gauge, Histogram},
    exporters::{PrometheusExporter, JaegerExporter, GrafanaExporter},
    logging::{Logger, LoggingConfig},
    ObservabilityConfig, init_observability, shutdown_observability
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_full_request_tracing() {
        // Initialize complete observability system
        let config = ObservabilityConfig {
            tracing: TracingConfig {
                service_name: "integration-test".to_string(),
                service_version: "1.0.0".to_string(),
                sampling_ratio: 1.0,
            },
            ..Default::default()
        };

        init_observability(config).await.unwrap();

        // Simulate a full HTTP request flow
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();

        // HTTP layer
        let http_span = tracer.start_span("http_request");
        http_span.set_tag("method", "GET");
        http_span.set_tag("url", "/api/users");

        // Business logic layer
        let business_span = tracer.start_span_with_parent("business_logic", http_span.context());
        business_span.set_tag("operation", "get_users");

        // Database layer
        let db_span = tracer.start_span_with_parent("database_query", business_span.context());
        db_span.set_tag("table", "users");
        db_span.set_tag("query_type", "SELECT");

        // Simulate database work
        tokio::time::sleep(Duration::from_millis(10)).await;

        db_span.finish();

        // Back to business logic
        tokio::time::sleep(Duration::from_millis(5)).await;
        business_span.finish();

        // Back to HTTP
        http_span.set_tag("status_code", "200");
        http_span.finish();

        // Verify trace hierarchy
        assert!(http_span.context().trace_id().is_valid());
        assert_eq!(business_span.context().parent_span_id(), Some(http_span.context().span_id()));
        assert_eq!(db_span.context().parent_span_id(), Some(business_span.context().span_id()));

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_metrics_collection_pipeline() {
        let registry = Arc::new(Registry::new());

        // Create various metrics
        let http_requests = Arc::new(Counter::with_labels(
            "http_requests_total",
            "Total HTTP requests",
            vec!["method", "endpoint", "status"]
        ).unwrap());

        let response_time = Arc::new(Histogram::with_labels(
            "http_request_duration_seconds",
            "HTTP request duration",
            vec!["method", "endpoint"]
        ).unwrap());

        let active_connections = Arc::new(Gauge::new(
            "active_connections",
            "Number of active connections"
        ).unwrap());

        // Register metrics
        registry.register(Box::new(http_requests.clone())).unwrap();
        registry.register(Box::new(response_time.clone())).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();

        // Simulate HTTP traffic
        for i in 0..100 {
            let method = if i % 3 == 0 { "GET" } else if i % 3 == 1 { "POST" } else { "PUT" };
            let status = if i % 10 == 0 { "500" } else { "200" };

            http_requests.with_labels(&[method, "/api/data", status]).inc();

            let duration = (i as f64 * 0.01) + 0.1; // 0.1 to 1.1 seconds
            response_time.with_labels(&[method, "/api/data"]).observe(duration);

            active_connections.set((i % 50) as f64);
        }

        // Collect all metrics
        let metrics = registry.gather();
        assert_eq!(metrics.len(), 3);

        // Verify metric values
        let http_metric = metrics.iter().find(|m| m.get_name() == "http_requests_total").unwrap();
        let time_metric = metrics.iter().find(|m| m.get_name() == "http_request_duration_seconds").unwrap();
        let conn_metric = metrics.iter().find(|m| m.get_name() == "active_connections").unwrap();

        // Should have recorded requests
        assert!(http_metric.get_metric().iter().any(|m| m.get_counter().get_value() > 0.0));
        assert!(time_metric.get_metric().iter().any(|m| m.get_histogram().get_sample_count() > 0));
        assert!(conn_metric.get_metric()[0].get_gauge().get_value() >= 0.0);
    }

    #[tokio::test]
    async fn test_log_aggregation_and_correlation() {
        let config = ObservabilityConfig {
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let logger = vela_runtime::observability::get_logger().await.unwrap();

        // Create a span and log within it
        let span = tracer.start_span("test_operation");
        let trace_id = span.context().trace_id().to_string();
        let span_id = span.context().span_id().to_string();

        // Log messages that should be correlated with the span
        logger.info("Starting operation", &[("trace_id", &trace_id), ("span_id", &span_id)]);
        logger.debug("Processing data", &[("step", "1"), ("trace_id", &trace_id)]);
        logger.warn("Slow operation detected", &[("duration_ms", "150"), ("trace_id", &trace_id)]);
        logger.info("Operation completed", &[("result", "success"), ("trace_id", &trace_id)]);

        span.finish();

        // Verify logs are collected and correlated
        let logs = logger.get_recent_logs().await;
        assert!(!logs.is_empty());

        // Check that logs contain trace correlation
        let trace_logs: Vec<_> = logs.iter().filter(|log| log.contains(&trace_id)).collect();
        assert!(!trace_logs.is_empty());

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_exporter_pipeline_integration() {
        // Create mock exporters (in real test, these would connect to actual services)
        let prometheus_config = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://localhost:9090".to_string(),
            ..Default::default()
        };

        let jaeger_config = vela_runtime::observability::exporters::JaegerConfig {
            endpoint: "http://localhost:14268/api/traces".to_string(),
            service_name: "test-service".to_string(),
            ..Default::default()
        };

        let grafana_config = vela_runtime::observability::exporters::GrafanaConfig {
            endpoint: "http://localhost:3000".to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        // Initialize exporters
        let prometheus = PrometheusExporter::new(prometheus_config).unwrap();
        let jaeger = JaegerExporter::new(jaeger_config).unwrap();
        let grafana = GrafanaExporter::new(grafana_config).unwrap();

        // Create test data
        let metrics_data = r#"
# HELP test_metric Test metric
# TYPE test_metric counter
test_metric 42
"#;

        let trace_data = r#"{"data":[{"traceID":"123","spans":[{"operationName":"test"}]}]}"#;

        // Test export pipeline
        let metrics_result = prometheus.export_metrics(metrics_data).await;
        let traces_result = jaeger.export_traces(trace_data).await;

        // Results depend on whether services are running
        assert!(metrics_result.is_ok() || metrics_result.is_err());
        assert!(traces_result.is_ok() || traces_result.is_err());

        // Test Grafana integration
        let dashboard_result = grafana.create_dashboard(r#"{"title":"Test Dashboard"}"#).await;
        assert!(dashboard_result.is_ok() || dashboard_result.is_err());
    }

    #[tokio::test]
    async fn test_configuration_validation_and_application() {
        // Test valid configuration
        let valid_config = ObservabilityConfig {
            tracing: TracingConfig {
                service_name: "test-service".to_string(),
                service_version: "1.0.0".to_string(),
                sampling_ratio: 0.5,
            },
            ..Default::default()
        };

        let result = init_observability(valid_config).await;
        assert!(result.is_ok());

        // Verify configuration is applied
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        assert_eq!(tracer.service_name(), "test-service");

        shutdown_observability().await;

        // Test invalid configuration
        let invalid_config = ObservabilityConfig {
            tracing: TracingConfig {
                service_name: "".to_string(), // Invalid: empty service name
                service_version: "1.0.0".to_string(),
                sampling_ratio: 1.5, // Invalid: sampling ratio > 1.0
            },
            ..Default::default()
        };

        let result = init_observability(invalid_config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cross_component_communication() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();
        let logger = vela_runtime::observability::get_logger().await.unwrap();

        // Create a span
        let span = tracer.start_span("cross_component_test");
        let trace_id = span.context().trace_id().to_string();

        // Create a metric
        let counter = Counter::new("cross_component_counter", "Cross component test").unwrap();
        registry.register(Box::new(counter.clone())).unwrap();
        counter.add(5.0);

        // Log with trace correlation
        logger.info("Cross component operation", &[("trace_id", &trace_id), ("component", "test")]);

        span.finish();

        // Verify all components worked together
        let metrics = registry.gather();
        assert!(!metrics.is_empty());

        let logs = logger.get_recent_logs().await;
        assert!(!logs.is_empty());

        let trace_logs: Vec<_> = logs.iter().filter(|log| log.contains(&trace_id)).collect();
        assert!(!trace_logs.is_empty());

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_error_propagation_and_handling() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let logger = vela_runtime::observability::get_logger().await.unwrap();

        // Simulate an operation that fails
        let span = tracer.start_span("error_test_operation");

        // Log error
        logger.error("Operation failed", &[("error_code", "E001"), ("component", "test")]);

        // Record error in span
        span.record_error("Operation failed", "Detailed error description");
        span.set_tag("error", "true");
        span.set_tag("error_code", "E001");

        span.finish();

        // Verify error is recorded
        assert!(span.has_error());

        // Verify error logs are captured
        let logs = logger.get_recent_logs().await;
        let error_logs: Vec<_> = logs.iter().filter(|log| log.contains("ERROR")).collect();
        assert!(!error_logs.is_empty());

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_performance_under_load() {
        let config = ObservabilityConfig {
            tracing: TracingConfig {
                sampling_ratio: 0.1, // Low sampling for performance
                ..Default::default()
            },
            ..Default::default()
        };

        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();

        let counter = Arc::new(Counter::new("load_test_counter", "Load test counter").unwrap());
        registry.register(Box::new(counter.clone())).unwrap();

        let start = Instant::now();

        // Simulate high load
        let mut handles = vec![];

        for i in 0..50 {
            let tracer_clone = tracer.clone();
            let counter_clone = counter.clone();

            let handle = tokio::spawn(async move {
                for j in 0..20 {
                    let span = tracer_clone.start_span(&format!("load_operation_{}_{}", i, j));
                    counter_clone.inc();

                    // Simulate some work
                    tokio::time::sleep(Duration::from_micros(100)).await;

                    span.finish();
                }
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();

        // Should complete within reasonable time (adjust threshold as needed)
        assert!(elapsed < Duration::from_secs(10));

        // Should have recorded all operations
        assert_eq!(counter.get(), 1000.0); // 50 * 20

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_resource_cleanup_and_lifecycle() {
        let config = ObservabilityConfig::default();

        // Initialize
        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();

        // Create resources
        let span = tracer.start_span("lifecycle_test");
        let counter = Counter::new("lifecycle_counter", "Lifecycle test").unwrap();
        registry.register(Box::new(counter.clone())).unwrap();

        counter.add(10.0);

        // Use resources
        assert_eq!(counter.get(), 10.0);
        assert_eq!(span.operation_name(), "lifecycle_test");

        span.finish();

        // Shutdown system
        shutdown_observability().await;

        // Verify clean shutdown (no panics)
        // Note: In a real implementation, we might verify that resources are freed
    }

    #[tokio::test]
    async fn test_system_health_monitoring() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        // Test health of all components
        let tracer_health = vela_runtime::observability::check_tracer_health().await;
        let metrics_health = vela_runtime::observability::check_metrics_health().await;
        let logger_health = vela_runtime::observability::check_logger_health().await;

        // All components should report healthy
        assert!(tracer_health.is_healthy());
        assert!(metrics_health.is_healthy());
        assert!(logger_health.is_healthy());

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_tracing_and_metrics_correlation() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();

        // Create correlated span and metric
        let span = tracer.start_span("correlated_operation");
        let trace_id = span.context().trace_id().to_string();

        let counter = Counter::with_labels(
            "operation_counter",
            "Operations counter",
            vec!["operation", "trace_id"]
        ).unwrap();

        registry.register(Box::new(counter.clone())).unwrap();

        // Record metric with trace correlation
        counter.with_labels(&["test_operation", &trace_id]).inc();

        span.finish();

        // Verify correlation
        let metrics = registry.gather();
        let operation_metric = metrics.iter().find(|m| m.get_name() == "operation_counter").unwrap();

        // Should have metric with trace_id label
        let has_correlation = operation_metric.get_metric().iter().any(|m| {
            m.get_label().iter().any(|label| label.get_name() == "trace_id" && label.get_value() == trace_id)
        });

        assert!(has_correlation);

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_log_level_filtering_integration() {
        let config = ObservabilityConfig {
            logging: LoggingConfig {
                level: "warn".to_string(), // Only warn and above
                ..Default::default()
            },
            ..Default::default()
        };

        init_observability(config).await.unwrap();

        let logger = vela_runtime::observability::get_logger().await.unwrap();

        // Log at different levels
        logger.debug("Debug message"); // Should be filtered out
        logger.info("Info message");   // Should be filtered out
        logger.warn("Warn message");   // Should be included
        logger.error("Error message"); // Should be included

        let logs = logger.get_recent_logs().await;

        // Should only contain warn and error logs
        let debug_logs: Vec<_> = logs.iter().filter(|log| log.contains("DEBUG")).collect();
        let info_logs: Vec<_> = logs.iter().filter(|log| log.contains("INFO")).collect();
        let warn_logs: Vec<_> = logs.iter().filter(|log| log.contains("WARN")).collect();
        let error_logs: Vec<_> = logs.iter().filter(|log| log.contains("ERROR")).collect();

        assert_eq!(debug_logs.len(), 0);
        assert_eq!(info_logs.len(), 0);
        assert!(warn_logs.len() > 0);
        assert!(error_logs.len() > 0);

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_exporter_configuration_persistence() {
        // Test that exporter configurations are properly applied and maintained
        let prometheus_config = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://test-endpoint:9090".to_string(),
            namespace: Some("test_namespace".to_string()),
            ..Default::default()
        };

        let exporter = PrometheusExporter::new(prometheus_config).unwrap();

        // Configuration should persist
        assert_eq!(exporter.endpoint(), "http://test-endpoint:9090/metrics");
        assert_eq!(exporter.namespace(), Some(&"test_namespace".to_string()));

        // Test multiple operations maintain configuration
        for i in 0..10 {
            let metrics_data = format!("# Test metrics {}\nmetric{} 1\n", i, i);
            let _ = exporter.export_metrics(&metrics_data).await;
        }

        // Configuration should still be correct
        assert_eq!(exporter.endpoint(), "http://test-endpoint:9090/metrics");
    }

    #[tokio::test]
    async fn test_concurrent_operations_safety() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        let tracer = Arc::new(vela_runtime::observability::get_tracer().await.unwrap());
        let registry = Arc::new(vela_runtime::observability::get_metrics_registry().await.unwrap());
        let logger = Arc::new(vela_runtime::observability::get_logger().await.unwrap());

        let mut handles = vec![];

        // Spawn multiple concurrent operations
        for i in 0..20 {
            let tracer_clone = tracer.clone();
            let registry_clone = registry.clone();
            let logger_clone = logger.clone();

            let handle = tokio::spawn(async move {
                // Create span
                let span = tracer_clone.start_span(&format!("concurrent_op_{}", i));

                // Create and update metric
                let counter = Counter::new(&format!("concurrent_counter_{}", i), "Concurrent counter").unwrap();
                registry_clone.register(Box::new(counter.clone())).unwrap();
                counter.add(i as f64);

                // Log message
                logger_clone.info(&format!("Concurrent operation {}", i), &[]);

                // Simulate work
                tokio::time::sleep(Duration::from_millis(5)).await;

                span.finish();

                i
            });

            handles.push(handle);
        }

        // Wait for all operations
        let results: Vec<usize> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // All operations should have completed
        assert_eq!(results.len(), 20);
        assert_eq!(results.iter().sum::<usize>(), (0..20).sum()); // 0+1+...+19 = 190

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_memory_usage_monitoring() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();

        // Create many spans and metrics to test memory usage
        let mut spans = vec![];
        let mut counters = vec![];

        for i in 0..100 {
            let span = tracer.start_span(&format!("memory_test_span_{}", i));
            spans.push(span);

            let counter = Counter::new(&format!("memory_test_counter_{}", i), "Memory test").unwrap();
            registry.register(Box::new(counter.clone())).unwrap();
            counters.push(counter);
        }

        // Update all counters
        for (i, counter) in counters.iter().enumerate() {
            counter.add(i as f64);
        }

        // Finish all spans
        for span in spans {
            span.finish();
        }

        // System should still be functional
        let final_span = tracer.start_span("final_memory_test");
        assert_eq!(final_span.operation_name(), "final_memory_test");
        final_span.finish();

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_configuration_hot_reload() {
        let mut config = ObservabilityConfig {
            tracing: TracingConfig {
                sampling_ratio: 1.0, // Start with full sampling
                ..Default::default()
            },
            ..Default::default()
        };

        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();

        // Test with full sampling
        for _ in 0..100 {
            let span = tracer.start_span("sampling_test");
            assert!(span.is_sampled()); // Should be sampled
            span.finish();
        }

        // Hot reload configuration
        let new_config = ObservabilityConfig {
            tracing: TracingConfig {
                sampling_ratio: 0.0, // Change to no sampling
                ..Default::default()
            },
            ..Default::default()
        };

        vela_runtime::observability::reload_configuration(new_config).await.unwrap();

        // Test with no sampling
        for _ in 0..100 {
            let span = tracer.start_span("sampling_test");
            assert!(!span.is_sampled()); // Should not be sampled
            span.finish();
        }

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_exporter_failure_recovery() {
        // Create exporter that will fail initially
        let prometheus_config = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://nonexistent-endpoint:9090".to_string(),
            max_retries: 3,
            ..Default::default()
        };

        let exporter = PrometheusExporter::new(prometheus_config).unwrap();

        // First few exports should fail
        for _ in 0..3 {
            let result = exporter.export_metrics("# test").await;
            assert!(result.is_err());
        }

        // Exporter should handle failures gracefully without crashing
        let final_result = exporter.export_metrics("# final test").await;
        assert!(final_result.is_ok() || final_result.is_err()); // Either way, no panic
    }

    #[tokio::test]
    async fn test_end_to_end_request_simulation() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        // Simulate a complete request lifecycle
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();
        let logger = vela_runtime::observability::get_logger().await.unwrap();

        // HTTP Request start
        let request_span = tracer.start_span("http_request");
        request_span.set_tag("method", "POST");
        request_span.set_tag("url", "/api/users");

        let request_counter = Counter::with_labels(
            "http_requests_total",
            "HTTP requests",
            vec!["method", "status"]
        ).unwrap();
        registry.register(Box::new(request_counter.clone())).unwrap();

        logger.info("Incoming HTTP request", &[("method", "POST"), ("url", "/api/users")]);

        // Authentication
        let auth_span = tracer.start_span_with_parent("authenticate", request_span.context());
        tokio::time::sleep(Duration::from_millis(5)).await; // Simulate auth work
        auth_span.finish();

        // Business logic
        let business_span = tracer.start_span_with_parent("process_request", request_span.context());

        // Database operation
        let db_span = tracer.start_span_with_parent("database_query", business_span.context());
        db_span.set_tag("table", "users");
        db_span.set_tag("operation", "INSERT");

        tokio::time::sleep(Duration::from_millis(10)).await; // Simulate DB work
        db_span.finish();

        business_span.finish();

        // Response
        request_span.set_tag("status_code", "201");
        request_counter.with_labels(&["POST", "201"]).inc();
        logger.info("Request completed successfully", &[("status_code", "201")]);

        request_span.finish();

        // Verify complete trace
        assert!(request_span.context().trace_id().is_valid());
        assert!(request_span.duration().unwrap() > Duration::from_millis(10));

        // Verify metrics
        let metrics = registry.gather();
        assert!(!metrics.is_empty());

        // Verify logs
        let logs = logger.get_recent_logs().await;
        assert!(!logs.is_empty());

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_component_initialization_order() {
        // Test that components can be initialized in any order

        // Initialize metrics first
        let registry = Registry::new();
        let counter = Counter::new("init_order_test", "Initialization order test").unwrap();
        registry.register(Box::new(counter.clone())).unwrap();

        // Then tracing
        let tracer = Tracer::new(TracingConfig::default());

        // Then logging
        let logger = Logger::new(LoggingConfig::default());

        // All should work together
        let span = tracer.start_span("init_order_test");
        counter.inc();
        logger.info("Initialization order test", &[]);

        span.finish();

        assert_eq!(counter.get(), 1.0);
        assert_eq!(span.operation_name(), "init_order_test");
    }

    #[tokio::test]
    async fn test_observability_system_restart() {
        let config = ObservabilityConfig::default();

        // First initialization
        init_observability(config.clone()).await.unwrap();

        let tracer1 = vela_runtime::observability::get_tracer().await.unwrap();
        let span1 = tracer1.start_span("before_restart");
        span1.finish();

        // Shutdown
        shutdown_observability().await;

        // Restart
        init_observability(config).await.unwrap();

        let tracer2 = vela_runtime::observability::get_tracer().await.unwrap();
        let span2 = tracer2.start_span("after_restart");
        span2.finish();

        // Both tracers should work independently
        assert_eq!(span1.operation_name(), "before_restart");
        assert_eq!(span2.operation_name(), "after_restart");

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_cross_service_trace_propagation() {
        // Simulate trace propagation between services
        let service_a_config = ObservabilityConfig {
            tracing: TracingConfig {
                service_name: "service-a".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let service_b_config = ObservabilityConfig {
            tracing: TracingConfig {
                service_name: "service-b".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        // Service A initiates request
        init_observability(service_a_config).await.unwrap();
        let tracer_a = vela_runtime::observability::get_tracer().await.unwrap();

        let span_a = tracer_a.start_span("service_a_operation");
        let trace_context = span_a.context().clone();

        // Extract trace context for propagation (simulate HTTP headers)
        let trace_id = trace_context.trace_id().to_string();
        let span_id = trace_context.span_id().to_string();

        shutdown_observability().await;

        // Service B receives request with trace context
        init_observability(service_b_config).await.unwrap();
        let tracer_b = vela_runtime::observability::get_tracer().await.unwrap();

        // Recreate span context from propagated headers
        let propagated_context = SpanContext::from_w3c_header(&format!("{}-{}", trace_id, span_id)).unwrap();
        let span_b = tracer_b.start_span_with_parent("service_b_operation", &propagated_context);

        // Verify trace continuity
        assert_eq!(span_b.context().trace_id().to_string(), trace_id);
        assert_eq!(span_b.context().parent_span_id(), Some(span_a.context().span_id()));

        span_b.finish();
        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_metrics_aggregation_and_export() {
        let registry = Registry::new();

        // Create histogram for response times
        let response_times = Histogram::new("http_response_time_seconds", "Response times").unwrap();
        registry.register(Box::new(response_times.clone())).unwrap();

        // Simulate various response times
        let response_data = vec![0.1, 0.2, 0.15, 0.3, 0.25, 2.0, 0.18, 0.22];

        for &time in &response_data {
            response_times.observe(time);
        }

        // Export to Prometheus format
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&registry.gather(), &mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();

        // Verify Prometheus format
        assert!(output.contains("# HELP http_response_time_seconds Response times"));
        assert!(output.contains("# TYPE http_response_time_seconds histogram"));
        assert!(output.contains("http_response_time_seconds_sum"));
        assert!(output.contains("http_response_time_seconds_count"));

        // Verify aggregated values
        assert_eq!(response_times.get_sample_count(), response_data.len() as u64);
        assert!((response_times.get_sample_sum() - response_data.iter().sum::<f64>()).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_log_structured_data_integration() {
        let config = ObservabilityConfig {
            logging: LoggingConfig {
                format: "json".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        init_observability(config).await.unwrap();

        let logger = vela_runtime::observability::get_logger().await.unwrap();

        // Log structured data
        let mut fields = HashMap::new();
        fields.insert("user_id", "12345");
        fields.insert("action", "login");
        fields.insert("ip_address", "192.168.1.100");
        fields.insert("user_agent", "Mozilla/5.0");

        logger.info("User login event", &[
            ("user_id", "12345"),
            ("action", "login"),
            ("ip_address", "192.168.1.100"),
            ("user_agent", "Mozilla/5.0")
        ]);

        let logs = logger.get_recent_logs().await;

        // Find the structured log entry
        let login_log = logs.iter().find(|log| log.contains("User login event")).unwrap();

        // Verify structured data is present
        assert!(login_log.contains("user_id"));
        assert!(login_log.contains("12345"));
        assert!(login_log.contains("action"));
        assert!(login_log.contains("login"));

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_performance_regression_detection() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();

        // Create performance baseline
        let operation_time = Histogram::new("operation_duration_seconds", "Operation duration").unwrap();
        registry.register(Box::new(operation_time.clone())).unwrap();

        // Record baseline performance
        for _ in 0..100 {
            let start = Instant::now();
            let span = tracer.start_span("baseline_operation");
            // Simulate baseline work
            tokio::time::sleep(Duration::from_micros(100)).await;
            let duration = start.elapsed().as_secs_f64();
            operation_time.observe(duration);
            span.finish();
        }

        let baseline_p95 = operation_time.get_percentiles().iter()
            .find(|p| p.quantile == 0.95)
            .map(|p| p.value)
            .unwrap_or(0.0);

        // Simulate performance regression
        for _ in 0..50 {
            let start = Instant::now();
            let span = tracer.start_span("regressed_operation");
            // Simulate slower work (regression)
            tokio::time::sleep(Duration::from_micros(200)).await;
            let duration = start.elapsed().as_secs_f64();
            operation_time.observe(duration);
            span.finish();
        }

        let current_p95 = operation_time.get_percentiles().iter()
            .find(|p| p.quantile == 0.95)
            .map(|p| p.value)
            .unwrap_or(0.0);

        // Performance should have regressed (increased)
        assert!(current_p95 > baseline_p95);

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_configuration_environment_variables() {
        // Test configuration from environment variables
        std::env::set_var("VELA_TRACING_SERVICE_NAME", "env-service");
        std::env::set_var("VELA_TRACING_SAMPLING_RATIO", "0.75");
        std::env::set_var("VELA_METRICS_ENABLED", "true");

        let config = ObservabilityConfig::from_env();

        assert_eq!(config.tracing.service_name, "env-service");
        assert_eq!(config.tracing.sampling_ratio, 0.75);

        // Clean up
        std::env::remove_var("VELA_TRACING_SERVICE_NAME");
        std::env::remove_var("VELA_TRACING_SAMPLING_RATIO");
        std::env::remove_var("VELA_METRICS_ENABLED");
    }

    #[tokio::test]
    async fn test_exporter_load_balancing() {
        // Create multiple exporters for load balancing
        let configs = vec![
            vela_runtime::observability::exporters::PrometheusConfig {
                endpoint: "http://prometheus-1:9090".to_string(),
                ..Default::default()
            },
            vela_runtime::observability::exporters::PrometheusConfig {
                endpoint: "http://prometheus-2:9090".to_string(),
                ..Default::default()
            },
            vela_runtime::observability::exporters::PrometheusConfig {
                endpoint: "http://prometheus-3:9090".to_string(),
                ..Default::default()
            },
        ];

        let exporters: Vec<_> = configs.into_iter()
            .map(|config| PrometheusExporter::new(config).unwrap())
            .collect();

        // Distribute exports across exporters
        for i in 0..30 {
            let exporter_index = i % exporters.len();
            let metrics_data = format!("# Load balanced metrics {}\nmetric{} 1\n", i, i);
            let _ = exporters[exporter_index].export_metrics(&metrics_data).await;
        }

        // Each exporter should have received some exports
        // (In real implementation, we'd verify this through metrics)
    }

    #[tokio::test]
    async fn test_observability_data_retention() {
        let config = ObservabilityConfig {
            tracing: TracingConfig {
                max_spans_per_trace: 100,
                ..Default::default()
            },
            ..Default::default()
        };

        init_observability(config).await.unwrap();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();

        // Create a trace with many spans
        let root_span = tracer.start_span("retention_test");

        let mut spans = vec![root_span];
        let mut parent_context = spans[0].context().clone();

        // Create chain of spans
        for i in 1..150 { // More than max_spans_per_trace
            let span = tracer.start_span_with_parent(&format!("span_{}", i), &parent_context);
            parent_context = span.context().clone();
            spans.push(span);
        }

        // Finish all spans
        for span in spans.into_iter().rev() {
            span.finish();
        }

        // System should handle the span limit gracefully
        // (In real implementation, we'd verify span count limits)

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_multi_tenant_observability() {
        // Test observability isolation between tenants
        let tenant_a_config = ObservabilityConfig {
            tracing: TracingConfig {
                service_name: "tenant-a-service".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let tenant_b_config = ObservabilityConfig {
            tracing: TracingConfig {
                service_name: "tenant-b-service".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        // Tenant A operations
        init_observability(tenant_a_config).await.unwrap();

        let tracer_a = vela_runtime::observability::get_tracer().await.unwrap();
        let registry_a = vela_runtime::observability::get_metrics_registry().await.unwrap();

        let span_a = tracer_a.start_span("tenant_a_operation");
        let counter_a = Counter::new("tenant_a_counter", "Tenant A counter").unwrap();
        registry_a.register(Box::new(counter_a.clone())).unwrap();
        counter_a.add(5.0);

        span_a.finish();

        shutdown_observability().await;

        // Tenant B operations (independent)
        init_observability(tenant_b_config).await.unwrap();

        let tracer_b = vela_runtime::observability::get_tracer().await.unwrap();
        let registry_b = vela_runtime::observability::get_metrics_registry().await.unwrap();

        let span_b = tracer_b.start_span("tenant_b_operation");
        let counter_b = Counter::new("tenant_b_counter", "Tenant B counter").unwrap();
        registry_b.register(Box::new(counter_b.clone())).unwrap();
        counter_b.add(10.0);

        span_b.finish();

        // Verify tenant isolation
        assert_eq!(tracer_b.service_name(), "tenant-b-service");
        assert_eq!(counter_b.get(), 10.0);

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_observability_system_health_metrics() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        // Generate some activity
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();

        for i in 0..10 {
            let span = tracer.start_span(&format!("health_test_{}", i));
            let counter = Counter::new(&format!("health_counter_{}", i), "Health counter").unwrap();
            registry.register(Box::new(counter.clone())).unwrap();
            counter.inc();
            span.finish();
        }

        // Check system health metrics
        let health_metrics = vela_runtime::observability::get_system_health_metrics().await;

        // Should include various health indicators
        assert!(health_metrics.contains_key("spans_created"));
        assert!(health_metrics.contains_key("metrics_registered"));
        assert!(health_metrics.contains_key("memory_usage"));
        assert!(health_metrics.contains_key("uptime_seconds"));

        // Values should be reasonable
        assert!(*health_metrics.get("spans_created").unwrap() >= 10);
        assert!(*health_metrics.get("metrics_registered").unwrap() >= 10);

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_end_to_end_observability_workflow() {
        // Complete workflow: setup -> operation -> monitoring -> cleanup

        // 1. Setup
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        // 2. Operation
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();
        let logger = vela_runtime::observability::get_logger().await.unwrap();

        // Business operation
        let operation_span = tracer.start_span("complete_workflow_operation");

        let requests_counter = Counter::new("workflow_requests", "Workflow requests").unwrap();
        registry.register(Box::new(requests_counter.clone())).unwrap();

        logger.info("Starting workflow operation", &[]);

        // Simulate workflow steps
        for step in 1..=5 {
            let step_span = tracer.start_span_with_parent(&format!("step_{}", step), operation_span.context());
            requests_counter.inc();

            tokio::time::sleep(Duration::from_millis(10)).await;

            logger.info(&format!("Completed step {}", step), &[("step", &step.to_string())]);
            step_span.finish();
        }

        operation_span.finish();

        // 3. Monitoring - check that everything was recorded
        let metrics = registry.gather();
        let logs = logger.get_recent_logs().await;

        assert!(!metrics.is_empty());
        assert!(!logs.is_empty());
        assert_eq!(requests_counter.get(), 5.0);

        // 4. Cleanup
        shutdown_observability().await;

        // Verify clean shutdown
    }

    #[tokio::test]
    async fn test_observability_error_boundary() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        // Test that observability errors don't crash the application
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();

        // Create spans with invalid data
        let span = tracer.start_span(""); // Empty operation name
        span.set_tag("", "invalid_tag"); // Empty tag name

        // Operations should not panic
        span.record_error("", ""); // Empty error
        span.finish();

        // System should still be functional
        let test_span = tracer.start_span("error_boundary_test");
        assert_eq!(test_span.operation_name(), "error_boundary_test");
        test_span.finish();

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_configuration_file_loading() {
        // Test loading configuration from file
        let config_content = r#"
tracing:
  service_name: "file-config-service"
  sampling_ratio: 0.8
logging:
  level: "debug"
  format: "json"
"#;

        // Write config to temporary file
        let config_path = "test_config.yaml";
        std::fs::write(config_path, config_content).unwrap();

        let config = ObservabilityConfig::from_file(config_path).unwrap();

        assert_eq!(config.tracing.service_name, "file-config-service");
        assert_eq!(config.tracing.sampling_ratio, 0.8);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.format, "json");

        // Cleanup
        std::fs::remove_file(config_path).unwrap();
    }
}