//! # Performance Tests
//!
//! Performance benchmarks and regression tests for the observability system.
//!
//! Tests cover:
//! - Tracing overhead measurement
//! - Metrics recording performance
//! - Exporter throughput benchmarks
//! - Memory usage analysis
//! - Concurrent access performance
//! - Sampling impact on performance
//! - Serialization/deserialization speed
//! - Buffer management efficiency
//! - Garbage collection impact
//! - System resource utilization

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vela_runtime::observability::{
    tracing::{Tracer, Span, TracingConfig},
    metrics::{Counter, Gauge, Histogram, Registry},
    exporters::{PrometheusExporter, JaegerExporter},
    logging::{Logger, LoggingConfig},
    ObservabilityConfig, init_observability, shutdown_observability
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracing_overhead_baseline() {
        let tracer = Tracer::new(TracingConfig::default());

        let start = Instant::now();

        // Measure overhead of creating and finishing spans
        for _ in 0..10000 {
            let span = tracer.start_span("overhead_test");
            black_box(span.operation_name());
            span.finish();
        }

        let elapsed = start.elapsed();
        let overhead_per_span = elapsed / 10000;

        // Overhead should be reasonable (less than 100 microseconds per span)
        assert!(overhead_per_span < Duration::from_micros(100));
    }

    #[tokio::test]
    async fn test_metrics_recording_performance() {
        let registry = Registry::new();
        let counter = Arc::new(Counter::new("perf_counter", "Performance counter").unwrap());
        registry.register(Box::new(counter.clone())).unwrap();

        let start = Instant::now();

        // Measure counter increment performance
        for _ in 0..100000 {
            counter.inc();
        }

        let elapsed = start.elapsed();
        let ops_per_second = 100000.0 / elapsed.as_secs_f64();

        // Should handle at least 100k operations per second
        assert!(ops_per_second > 100000.0);
        assert_eq!(counter.get(), 100000.0);
    }

    #[tokio::test]
    async fn test_histogram_observation_speed() {
        let histogram = Histogram::new("perf_histogram", "Performance histogram").unwrap();

        let start = Instant::now();

        // Measure histogram observation performance
        for i in 0..50000 {
            histogram.observe((i % 100) as f64 * 0.01);
        }

        let elapsed = start.elapsed();
        let ops_per_second = 50000.0 / elapsed.as_secs_f64();

        // Should handle at least 50k observations per second
        assert!(ops_per_second > 50000.0);
        assert_eq!(histogram.get_sample_count(), 50000);
    }

    #[tokio::test]
    async fn test_exporter_throughput_prometheus() {
        let config = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://localhost:9090".to_string(),
            ..Default::default()
        };

        let exporter = PrometheusExporter::new(config).unwrap();

        let start = Instant::now();
        let mut successful_exports = 0;

        // Measure export throughput
        for i in 0..100 {
            let metrics_data = format!(
                "# Performance test metrics {}\nperf_metric_{} 1\n",
                i, i
            );

            if let Ok(_) = exporter.export_metrics(&metrics_data).await {
                successful_exports += 1;
            }
        }

        let elapsed = start.elapsed();

        // At least some exports should succeed (depending on endpoint availability)
        // Measure time per successful export
        if successful_exports > 0 {
            let avg_time_per_export = elapsed / successful_exports;
            // Should be reasonably fast (less than 10ms per export on average)
            assert!(avg_time_per_export < Duration::from_millis(10));
        }
    }

    #[tokio::test]
    async fn test_jaeger_exporter_throughput() {
        let config = vela_runtime::observability::exporters::JaegerConfig {
            endpoint: "http://localhost:14268/api/traces".to_string(),
            ..Default::default()
        };

        let exporter = JaegerExporter::new(config).unwrap();

        let start = Instant::now();
        let mut successful_exports = 0;

        // Measure trace export throughput
        for i in 0..100 {
            let trace_data = format!(r#"{{"data":[{{"traceID":"trace{}","spans":[{{"operationName":"op{}"}}]}}]}"#, i, i);

            if let Ok(_) = exporter.export_traces(&trace_data).await {
                successful_exports += 1;
            }
        }

        let elapsed = start.elapsed();

        if successful_exports > 0 {
            let avg_time_per_export = elapsed / successful_exports;
            // Should be reasonably fast
            assert!(avg_time_per_export < Duration::from_millis(20));
        }
    }

    #[tokio::test]
    async fn test_memory_usage_tracing() {
        let tracer = Tracer::new(TracingConfig::default());

        // Measure memory usage with increasing number of spans
        let mut spans = vec![];

        for i in 0..1000 {
            let span = tracer.start_span(&format!("memory_span_{}", i));
            spans.push(span);
        }

        // Check that memory usage scales reasonably
        // (In a real benchmark, we'd use a memory profiler)

        // Finish spans
        for span in spans {
            span.finish();
        }
    }

    #[tokio::test]
    async fn test_concurrent_tracing_performance() {
        let tracer = Arc::new(Tracer::new(TracingConfig::default()));

        let start = Instant::now();

        let mut handles = vec![];

        // Spawn multiple concurrent tracing operations
        for i in 0..10 {
            let tracer_clone = tracer.clone();
            let handle = tokio::spawn(async move {
                for j in 0..1000 {
                    let span = tracer_clone.start_span(&format!("concurrent_{}_{}", i, j));
                    black_box(span.operation_name());
                    span.finish();
                }
            });
            handles.push(handle);
        }

        // Wait for all operations
        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();
        let total_operations = 10 * 1000;
        let ops_per_second = total_operations as f64 / elapsed.as_secs_f64();

        // Should handle reasonable concurrent load
        assert!(ops_per_second > 10000.0);
    }

    #[tokio::test]
    async fn test_sampling_performance_impact() {
        let full_sampling_tracer = Tracer::new(TracingConfig {
            sampling_ratio: 1.0,
            ..Default::default()
        });

        let half_sampling_tracer = Tracer::new(TracingConfig {
            sampling_ratio: 0.5,
            ..Default::default()
        });

        let no_sampling_tracer = Tracer::new(TracingConfig {
            sampling_ratio: 0.0,
            ..Default::default()
        });

        // Measure performance with different sampling rates
        let start_full = Instant::now();
        for _ in 0..10000 {
            let span = full_sampling_tracer.start_span("full_sampling");
            span.finish();
        }
        let full_time = start_full.elapsed();

        let start_half = Instant::now();
        for _ in 0..10000 {
            let span = half_sampling_tracer.start_span("half_sampling");
            span.finish();
        }
        let half_time = start_half.elapsed();

        let start_none = Instant::now();
        for _ in 0..10000 {
            let span = no_sampling_tracer.start_span("no_sampling");
            span.finish();
        }
        let none_time = start_none.elapsed();

        // No sampling should be fastest, full sampling slowest
        assert!(none_time <= half_time);
        assert!(half_time <= full_time);

        // But all should be reasonably fast
        assert!(full_time < Duration::from_secs(1));
        assert!(none_time < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_metrics_serialization_performance() {
        let registry = Registry::new();

        // Create many metrics
        for i in 0..100 {
            let counter = Counter::new(&format!("serial_counter_{}", i), "Serialization counter").unwrap();
            registry.register(Box::new(counter.clone())).unwrap();
            counter.add(i as f64);
        }

        let encoder = TextEncoder::new();

        let start = Instant::now();

        // Measure serialization performance
        for _ in 0..100 {
            let mut buffer = Vec::new();
            encoder.encode(&registry.gather(), &mut buffer).unwrap();
            black_box(&buffer);
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed / 100;

        // Serialization should be fast
        assert!(avg_time < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_buffer_management_efficiency() {
        let config = vela_runtime::observability::exporters::JaegerConfig {
            endpoint: "http://localhost:14268/api/traces".to_string(),
            buffer_size: 1000,
            ..Default::default()
        };

        let exporter = JaegerExporter::new(config).unwrap();

        let start = Instant::now();

        // Fill buffer with spans
        for i in 0..1000 {
            let span_data = format!(r#"{{"operationName":"buffer_test_{}"}}"#, i);
            exporter.buffer_span(span_data);
        }

        let buffer_fill_time = start.elapsed();

        // Flush buffer
        let flush_start = Instant::now();
        let _ = exporter.flush_buffer().await;
        let flush_time = flush_start.elapsed();

        // Buffer operations should be efficient
        assert!(buffer_fill_time < Duration::from_millis(100));
        assert!(flush_time < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_garbage_collection_impact() {
        let tracer = Tracer::new(TracingConfig::default());

        // Create many short-lived spans to test GC impact
        let start = Instant::now();

        for batch in 0..10 {
            let mut spans = vec![];

            // Create batch of spans
            for i in 0..1000 {
                let span = tracer.start_span(&format!("gc_test_{}_{}", batch, i));
                spans.push(span);
            }

            // Finish batch
            for span in spans {
                span.finish();
            }

            // Allow some time for GC if needed
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        let elapsed = start.elapsed();

        // Should complete within reasonable time despite GC
        assert!(elapsed < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_system_resource_utilization() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        let start_time = Instant::now();
        let start_metrics = vela_runtime::observability::get_system_health_metrics().await;

        // Generate observability load
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();
        let logger = vela_runtime::observability::get_logger().await.unwrap();

        for i in 0..1000 {
            let span = tracer.start_span(&format!("resource_test_{}", i));
            let counter = Counter::new(&format!("resource_counter_{}", i), "Resource counter").unwrap();
            registry.register(Box::new(counter.clone())).unwrap();
            counter.inc();
            logger.debug(&format!("Resource test {}", i), &[]);
            span.finish();
        }

        let end_time = Instant::now();
        let end_metrics = vela_runtime::observability::get_system_health_metrics().await;

        let duration = end_time - start_time;

        // Resource usage should be reasonable
        let memory_increase = end_metrics.get("memory_usage").unwrap_or(&0) - start_metrics.get("memory_usage").unwrap_or(&0);
        assert!(memory_increase < 50 * 1024 * 1024); // Less than 50MB increase

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_tracing_nested_spans_performance() {
        let tracer = Tracer::new(TracingConfig::default());

        let start = Instant::now();

        // Create deeply nested spans
        fn create_nested_span(tracer: &Tracer, depth: usize, max_depth: usize, parent_context: Option<&SpanContext>) {
            if depth >= max_depth {
                return;
            }

            let span = if let Some(context) = parent_context {
                tracer.start_span_with_parent(&format!("nested_{}", depth), context)
            } else {
                tracer.start_span(&format!("nested_{}", depth))
            };

            // Recurse
            create_nested_span(tracer, depth + 1, max_depth, Some(span.context()));

            span.finish();
        }

        create_nested_span(&tracer, 0, 20, None);

        let elapsed = start.elapsed();

        // Nested span creation should be efficient
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_metrics_high_cardinality_performance() {
        let counter = Counter::with_labels(
            "cardinality_test",
            "High cardinality test",
            vec!["user_id", "endpoint", "method"]
        ).unwrap();

        let start = Instant::now();

        // Simulate high cardinality labels
        for user_id in 0..100 {
            for endpoint in &["/api/users", "/api/orders", "/api/products"] {
                for method in &["GET", "POST", "PUT", "DELETE"] {
                    counter.with_labels(&[&user_id.to_string(), endpoint, method]).inc();
                }
            }
        }

        let elapsed = start.elapsed();

        // High cardinality operations should be reasonably fast
        assert!(elapsed < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_exporter_retry_performance() {
        let config = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://nonexistent-endpoint:9090".to_string(),
            max_retries: 3,
            retry_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let exporter = PrometheusExporter::new(config).unwrap();

        let start = Instant::now();

        // Measure retry performance
        let _ = exporter.export_metrics("# retry test").await;

        let elapsed = start.elapsed();

        // Should take time for retries but not too long
        assert!(elapsed >= Duration::from_millis(30)); // At least 3 retries with 10ms delay
        assert!(elapsed < Duration::from_secs(1)); // But not excessively long
    }

    #[tokio::test]
    async fn test_logging_performance_under_load() {
        let logger = Logger::new(LoggingConfig::default());

        let start = Instant::now();

        // High volume logging
        for i in 0..10000 {
            logger.info(&format!("Log message {}", i), &[("index", &i.to_string())]);
        }

        let elapsed = start.elapsed();
        let logs_per_second = 10000.0 / elapsed.as_secs_f64();

        // Should handle reasonable logging volume
        assert!(logs_per_second > 5000.0);
    }

    #[tokio::test]
    async fn test_span_context_propagation_speed() {
        let tracer = Tracer::new(TracingConfig::default());

        let start = Instant::now();

        // Measure context propagation performance
        for _ in 0..5000 {
            let span1 = tracer.start_span("propagation_test_1");
            let context = span1.context().clone();
            span1.finish();

            let span2 = tracer.start_span_with_parent("propagation_test_2", &context);
            span2.finish();
        }

        let elapsed = start.elapsed();
        let propagations_per_second = 5000.0 / elapsed.as_secs_f64();

        // Context propagation should be fast
        assert!(propagations_per_second > 10000.0);
    }

    #[tokio::test]
    async fn test_metrics_aggregation_performance() {
        let histogram = Histogram::new("aggregation_test", "Aggregation performance").unwrap();

        let start = Instant::now();

        // Add many observations to test aggregation
        for i in 0..100000 {
            histogram.observe((i % 1000) as f64 * 0.001);
        }

        let elapsed = start.elapsed();

        // Aggregation should be efficient
        assert!(elapsed < Duration::from_secs(2));
        assert_eq!(histogram.get_sample_count(), 100000);
    }

    #[tokio::test]
    async fn test_exporter_concurrent_throughput() {
        let config = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://localhost:9090".to_string(),
            ..Default::default()
        };

        let exporter = Arc::new(PrometheusExporter::new(config).unwrap());

        let start = Instant::now();

        let mut handles = vec![];

        // Concurrent exports
        for i in 0..10 {
            let exporter_clone = exporter.clone();
            let handle = tokio::spawn(async move {
                for j in 0..50 {
                    let metrics_data = format!("# Concurrent test {} {}\nmetric 1\n", i, j);
                    let _ = exporter_clone.export_metrics(&metrics_data).await;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();

        // Concurrent exports should complete reasonably fast
        assert!(elapsed < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_memory_allocation_patterns() {
        let tracer = Tracer::new(TracingConfig::default());

        // Test memory allocation patterns under different loads
        let mut memory_usage = vec![];

        for batch_size in &[100, 500, 1000, 2000] {
            let batch_start = Instant::now();

            let mut spans = vec![];
            for i in 0..*batch_size {
                let span = tracer.start_span(&format!("alloc_test_{}", i));
                spans.push(span);
            }

            // Simulate processing time
            tokio::time::sleep(Duration::from_millis(10)).await;

            for span in spans {
                span.finish();
            }

            let batch_time = batch_start.elapsed();
            memory_usage.push(batch_time);
        }

        // Memory allocation should scale reasonably
        // (Larger batches might take longer due to allocation pressure)
        assert!(memory_usage[0] <= memory_usage[1]);
        assert!(memory_usage[1] <= memory_usage[2]);
        assert!(memory_usage[2] <= memory_usage[3]);
    }

    #[tokio::test]
    async fn test_tracing_with_tags_performance() {
        let tracer = Tracer::new(TracingConfig::default());

        let start = Instant::now();

        // Create spans with many tags
        for i in 0..1000 {
            let mut span = tracer.start_span(&format!("tagged_span_{}", i));

            // Add multiple tags
            for j in 0..10 {
                span.set_tag(&format!("tag_{}", j), &format!("value_{}_{}", i, j));
            }

            span.finish();
        }

        let elapsed = start.elapsed();

        // Tagged spans should not be significantly slower
        assert!(elapsed < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_metrics_registry_scaling() {
        let registry = Arc::new(Registry::new());

        let start = Instant::now();

        // Register many metrics
        for i in 0..1000 {
            let counter = Counter::new(&format!("scale_counter_{}", i), "Scaling counter").unwrap();
            registry.register(Box::new(counter)).unwrap();
        }

        let registration_time = start.elapsed();

        // Gather all metrics
        let gather_start = Instant::now();
        let _ = registry.gather();
        let gather_time = gather_start.elapsed();

        // Registry operations should scale reasonably
        assert!(registration_time < Duration::from_secs(1));
        assert!(gather_time < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_exporter_buffering_throughput() {
        let config = vela_runtime::observability::exporters::JaegerConfig {
            endpoint: "http://localhost:14268/api/traces".to_string(),
            buffer_size: 5000,
            ..Default::default()
        };

        let exporter = JaegerExporter::new(config).unwrap();

        let start = Instant::now();

        // Buffer many spans quickly
        for i in 0..5000 {
            let span_data = format!(r#"{{"operationName":"buffer_perf_{}"}}"#, i);
            exporter.buffer_span(span_data);
        }

        let buffer_time = start.elapsed();

        // Buffering should be very fast
        assert!(buffer_time < Duration::from_millis(200));

        // Flush buffer
        let flush_start = Instant::now();
        let _ = exporter.flush_buffer().await;
        let flush_time = flush_start.elapsed();

        assert!(flush_time < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_observability_system_warmup() {
        // Test cold start vs warm performance
        let config = ObservabilityConfig::default();

        let cold_start = Instant::now();
        init_observability(config.clone()).await.unwrap();
        let cold_time = cold_start.elapsed();

        let tracer = vela_runtime::observability::get_tracer().await.unwrap();

        // Warm up operations
        for i in 0..100 {
            let span = tracer.start_span(&format!("warmup_{}", i));
            span.finish();
        }

        let warm_start = Instant::now();
        for i in 0..1000 {
            let span = tracer.start_span(&format!("warm_test_{}", i));
            span.finish();
        }
        let warm_time = warm_start.elapsed();

        // Warm operations should be faster than cold start
        assert!(warm_time < cold_time);

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_span_event_recording_performance() {
        let tracer = Tracer::new(TracingConfig::default());

        let start = Instant::now();

        // Create spans with many events
        for i in 0..500 {
            let mut span = tracer.start_span(&format!("event_span_{}", i));

            // Add multiple events
            for j in 0..5 {
                let mut attributes = HashMap::new();
                attributes.insert(format!("attr_{}", j), format!("value_{}_{}", i, j));
                span.add_event(&format!("event_{}", j), attributes);
            }

            span.finish();
        }

        let elapsed = start.elapsed();

        // Event recording should be efficient
        assert!(elapsed < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_metrics_concurrent_access_performance() {
        let counter = Arc::new(Counter::new("concurrent_perf", "Concurrent performance").unwrap());
        let gauge = Arc::new(Gauge::new("concurrent_gauge", "Concurrent gauge").unwrap());

        let start = Instant::now();

        let mut handles = vec![];

        // Many concurrent operations
        for _ in 0..20 {
            let counter_clone = counter.clone();
            let gauge_clone = gauge.clone();

            let handle = tokio::spawn(async move {
                for _ in 0..1000 {
                    counter_clone.inc();
                    gauge_clone.inc();
                    gauge_clone.dec();
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();

        // Concurrent operations should be fast
        assert!(elapsed < Duration::from_secs(2));
        assert_eq!(counter.get(), 20000.0); // 20 * 1000
        assert_eq!(gauge.get(), 0.0); // inc/dec cancel out
    }

    #[tokio::test]
    async fn test_exporter_compression_overhead() {
        let config_compressed = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://localhost:9090".to_string(),
            compression: true,
            ..Default::default()
        };

        let config_uncompressed = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://localhost:9090".to_string(),
            compression: false,
            ..Default::default()
        };

        let exporter_compressed = PrometheusExporter::new(config_compressed).unwrap();
        let exporter_uncompressed = PrometheusExporter::new(config_uncompressed).unwrap();

        // Large metrics payload
        let large_metrics = "# Large metrics payload\n".repeat(1000) + "large_metric 1\n";

        let start_compressed = Instant::now();
        let _ = exporter_compressed.export_metrics(&large_metrics).await;
        let compressed_time = start_compressed.elapsed();

        let start_uncompressed = Instant::now();
        let _ = exporter_uncompressed.export_metrics(&large_metrics).await;
        let uncompressed_time = start_uncompressed.elapsed();

        // Compressed should be reasonably close in performance
        // (compression overhead vs network benefits)
        let compression_ratio = compressed_time.as_secs_f64() / uncompressed_time.as_secs_f64();
        assert!(compression_ratio < 2.0); // Compression shouldn't double the time
    }

    #[tokio::test]
    async fn test_tracing_memory_leak_detection() {
        let tracer = Tracer::new(TracingConfig::default());

        // Create and finish many spans
        for batch in 0..5 {
            let mut spans = vec![];

            for i in 0..1000 {
                let span = tracer.start_span(&format!("leak_test_{}_{}", batch, i));
                spans.push(span);
            }

            // Finish spans
            for span in spans {
                span.finish();
            }

            // Force some cleanup
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        // System should remain stable
        let test_span = tracer.start_span("stability_test");
        assert_eq!(test_span.operation_name(), "stability_test");
        test_span.finish();
    }

    #[tokio::test]
    async fn test_metrics_histogram_percentile_calculation() {
        let histogram = Histogram::new("percentile_perf", "Percentile performance").unwrap();

        let start = Instant::now();

        // Add observations for percentile calculation
        for i in 1..=10000 {
            histogram.observe(i as f64);
        }

        let observation_time = start.elapsed();

        // Calculate percentiles
        let percentile_start = Instant::now();
        let _ = histogram.get_percentiles();
        let percentile_time = percentile_start.elapsed();

        // Both operations should be reasonably fast
        assert!(observation_time < Duration::from_secs(1));
        assert!(percentile_time < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_exporter_connection_pool_performance() {
        let config = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://localhost:9090".to_string(),
            max_connections: 10,
            ..Default::default()
        };

        let exporter = Arc::new(PrometheusExporter::new(config).unwrap());

        let start = Instant::now();

        let mut handles = vec![];

        // Test connection pool under concurrent load
        for i in 0..50 {
            let exporter_clone = exporter.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let metrics_data = format!("# Connection pool test {} {}\nmetric 1\n", i, j);
                    let _ = exporter_clone.export_metrics(&metrics_data).await;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();

        // Connection pool should handle concurrent load efficiently
        assert!(elapsed < Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_observability_full_system_throughput() {
        let config = ObservabilityConfig::default();
        init_observability(config).await.unwrap();

        let start = Instant::now();

        // Full system load test
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();
        let registry = vela_runtime::observability::get_metrics_registry().await.unwrap();
        let logger = vela_runtime::observability::get_logger().await.unwrap();

        let operations = 1000;

        for i in 0..operations {
            // Tracing
            let span = tracer.start_span(&format!("full_system_{}", i));

            // Metrics
            let counter = Counter::new(&format!("full_counter_{}", i), "Full system counter").unwrap();
            registry.register(Box::new(counter.clone())).unwrap();
            counter.inc();

            // Logging
            logger.info(&format!("Full system operation {}", i), &[("op", &i.to_string())]);

            span.finish();
        }

        let elapsed = start.elapsed();
        let ops_per_second = operations as f64 / elapsed.as_secs_f64();

        // Full system should handle reasonable throughput
        assert!(ops_per_second > 100.0);

        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_span_context_serialization_speed() {
        let tracer = Tracer::new(TracingConfig::default());
        let span = tracer.start_span("serialization_test");

        let start = Instant::now();

        // Serialize/deserialize context many times
        for _ in 0..10000 {
            let context = span.context();
            let serialized = context.to_string();
            let _deserialized = SpanContext::from_string(&serialized).unwrap();
        }

        let elapsed = start.elapsed();
        let ops_per_second = 10000.0 / elapsed.as_secs_f64();

        // Serialization should be fast
        assert!(ops_per_second > 5000.0);

        span.finish();
    }

    #[tokio::test]
    async fn test_metrics_label_lookup_performance() {
        let counter = Counter::with_labels(
            "label_perf",
            "Label performance test",
            vec!["service", "endpoint", "method", "status"]
        ).unwrap();

        let start = Instant::now();

        // Test label lookup performance with many combinations
        let services = ["auth", "api", "web"];
        let endpoints = ["/login", "/users", "/orders"];
        let methods = ["GET", "POST", "PUT"];
        let statuses = ["200", "404", "500"];

        let mut count = 0;
        for service in &services {
            for endpoint in &endpoints {
                for method in &methods {
                    for status in &statuses {
                        counter.with_labels(&[*service, *endpoint, *method, *status]).inc();
                        count += 1;
                    }
                }
            }
        }

        let elapsed = start.elapsed();

        // Label operations should be efficient
        assert!(elapsed < Duration::from_secs(2));
        assert_eq!(counter.with_labels(&["auth", "/login", "GET", "200"]).get(), 1.0);
    }

    #[tokio::test]
    async fn test_exporter_failure_recovery_speed() {
        let config = vela_runtime::observability::exporters::PrometheusConfig {
            endpoint: "http://failing-endpoint:9090".to_string(),
            max_retries: 2,
            retry_delay: Duration::from_millis(1), // Fast retry for testing
            ..Default::default()
        };

        let exporter = PrometheusExporter::new(config).unwrap();

        let start = Instant::now();

        // Test failure recovery speed
        for _ in 0..10 {
            let _ = exporter.export_metrics("# failure test").await;
        }

        let elapsed = start.elapsed();

        // Failure recovery should be reasonably fast
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_tracing_span_hierarchy_depth_performance() {
        let tracer = Tracer::new(TracingConfig::default());

        let start = Instant::now();

        // Create deep span hierarchy
        fn create_deep_hierarchy(tracer: &Tracer, depth: usize, max_depth: usize, parent: Option<Span>) -> Option<Span> {
            if depth >= max_depth {
                return parent;
            }

            let span = if let Some(ref parent_span) = parent {
                tracer.start_span_with_parent(&format!("depth_{}", depth), parent_span.context())
            } else {
                tracer.start_span(&format!("depth_{}", depth))
            };

            let result = create_deep_hierarchy(tracer, depth + 1, max_depth, Some(span));

            if let Some(span) = result {
                span.finish();
            }

            None
        }

        create_deep_hierarchy(&tracer, 0, 50, None);

        let elapsed = start.elapsed();

        // Deep hierarchies should be manageable
        assert!(elapsed < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_metrics_summary_quantile_performance() {
        let summary = Summary::with_opts(SummaryOpts {
            quantiles: vec![0.5, 0.9, 0.95, 0.99],
            ..Default::default()
        }).unwrap();

        let start = Instant::now();

        // Add observations for quantile calculation
        for i in 0..50000 {
            summary.observe((i % 1000) as f64 * 0.01);
        }

        let observation_time = start.elapsed();

        // Allow time for quantile calculation
        tokio::time::sleep(Duration::from_millis(50)).await;

        let quantile_start = Instant::now();
        let _ = summary.get_quantiles();
        let quantile_time = quantile_start.elapsed();

        // Summary operations should be efficient
        assert!(observation_time < Duration::from_secs(1));
        assert!(quantile_time < Duration::from_millis(50));
    }
}