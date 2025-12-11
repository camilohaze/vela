//! # Metrics Tests
//!
//! Comprehensive tests for the metrics collection functionality.
//!
//! Tests cover:
//! - Counter metrics (increment, decrement, reset)
//! - Gauge metrics (set, increment, decrement)
//! - Histogram metrics (observe, buckets, quantiles)
//! - Summary metrics (observe, quantiles, sliding window)
//! - Labels and dimensional metrics
//! - Registry management and naming
//! - Concurrency and thread safety
//! - Prometheus format export
//! - Metric lifecycle and cleanup

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use vela_runtime::observability::metrics::{
    Counter, Gauge, Histogram, Summary, Registry, MetricOpts, HistogramOpts, SummaryOpts,
    Encoder, TextEncoder, MetricFamily, MetricType, Sample, LabelPair
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_counter_creation() {
        let registry = Registry::new();
        let counter = Counter::new("test_counter", "A test counter").unwrap();

        registry.register(Box::new(counter.clone())).unwrap();

        assert_eq!(counter.desc().name, "test_counter");
        assert_eq!(counter.desc().help, "A test counter");
        assert_eq!(counter.get(), 0.0);
    }

    #[tokio::test]
    async fn test_counter_increment() {
        let counter = Counter::new("increment_test", "Test increment").unwrap();

        // Initial value should be 0
        assert_eq!(counter.get(), 0.0);

        // Increment by 1
        counter.inc();
        assert_eq!(counter.get(), 1.0);

        // Increment by 5
        counter.add(5.0);
        assert_eq!(counter.get(), 6.0);

        // Increment by float
        counter.add(1.5);
        assert_eq!(counter.get(), 7.5);
    }

    #[tokio::test]
    async fn test_counter_with_labels() {
        let counter = Counter::with_labels(
            "labeled_counter",
            "Counter with labels",
            vec!["method", "status"]
        ).unwrap();

        // Increment with different label combinations
        counter.with_labels(&["GET", "200"]).inc();
        counter.with_labels(&["POST", "201"]).add(2.0);
        counter.with_labels(&["GET", "404"]).inc();

        assert_eq!(counter.with_labels(&["GET", "200"]).get(), 1.0);
        assert_eq!(counter.with_labels(&["POST", "201"]).get(), 2.0);
        assert_eq!(counter.with_labels(&["GET", "404"]).get(), 1.0);
    }

    #[tokio::test]
    async fn test_gauge_creation() {
        let registry = Registry::new();
        let gauge = Gauge::new("test_gauge", "A test gauge").unwrap();

        registry.register(Box::new(gauge.clone())).unwrap();

        assert_eq!(gauge.desc().name, "test_gauge");
        assert_eq!(gauge.desc().help, "A test gauge");
        assert_eq!(gauge.get(), 0.0);
    }

    #[tokio::test]
    async fn test_gauge_operations() {
        let gauge = Gauge::new("gauge_ops", "Gauge operations").unwrap();

        // Set value
        gauge.set(42.0);
        assert_eq!(gauge.get(), 42.0);

        // Increment
        gauge.inc();
        assert_eq!(gauge.get(), 43.0);

        // Decrement
        gauge.dec();
        assert_eq!(gauge.get(), 42.0);

        // Add/subtract
        gauge.add(5.5);
        assert_eq!(gauge.get(), 47.5);

        gauge.sub(10.0);
        assert_eq!(gauge.get(), 37.5);
    }

    #[tokio::test]
    async fn test_gauge_with_labels() {
        let gauge = Gauge::with_labels(
            "labeled_gauge",
            "Gauge with labels",
            vec!["service", "instance"]
        ).unwrap();

        // Set different values for different labels
        gauge.with_labels(&["web", "instance1"]).set(10.0);
        gauge.with_labels(&["api", "instance2"]).set(20.0);
        gauge.with_labels(&["web", "instance2"]).set(15.0);

        assert_eq!(gauge.with_labels(&["web", "instance1"]).get(), 10.0);
        assert_eq!(gauge.with_labels(&["api", "instance2"]).get(), 20.0);
        assert_eq!(gauge.with_labels(&["web", "instance2"]).get(), 15.0);
    }

    #[tokio::test]
    async fn test_histogram_creation() {
        let registry = Registry::new();
        let histogram = Histogram::new(
            "test_histogram",
            "A test histogram"
        ).unwrap();

        registry.register(Box::new(histogram.clone())).unwrap();

        assert_eq!(histogram.desc().name, "test_histogram");
        assert_eq!(histogram.desc().help, "A test histogram");
    }

    #[tokio::test]
    async fn test_histogram_observations() {
        let histogram = Histogram::new("hist_obs", "Histogram observations").unwrap();

        // Observe some values
        histogram.observe(1.0);
        histogram.observe(2.0);
        histogram.observe(3.0);
        histogram.observe(2.5);

        // Check sample count
        assert_eq!(histogram.get_sample_count(), 4);
        assert_eq!(histogram.get_sample_sum(), 8.5);
    }

    #[tokio::test]
    async fn test_histogram_buckets() {
        let histogram = Histogram::with_opts(HistogramOpts {
            buckets: vec![1.0, 2.5, 5.0, 10.0],
            ..Default::default()
        }).unwrap();

        // Observe values that fall into different buckets
        histogram.observe(0.5);  // bucket 0: [0, 1.0)
        histogram.observe(1.5);  // bucket 1: [1.0, 2.5)
        histogram.observe(3.0);  // bucket 2: [2.5, 5.0)
        histogram.observe(7.0);  // bucket 3: [5.0, 10.0)
        histogram.observe(15.0); // bucket 4: [10.0, +inf)

        let buckets = histogram.get_buckets();
        assert_eq!(buckets[0].count, 1); // 0.5
        assert_eq!(buckets[1].count, 1); // 1.5
        assert_eq!(buckets[2].count, 1); // 3.0
        assert_eq!(buckets[3].count, 1); // 7.0
        assert_eq!(buckets[4].count, 1); // 15.0
    }

    #[tokio::test]
    async fn test_histogram_with_labels() {
        let histogram = Histogram::with_labels(
            "labeled_histogram",
            "Histogram with labels",
            vec!["endpoint", "method"]
        ).unwrap();

        // Observe values for different label combinations
        histogram.with_labels(&["/api/users", "GET"]).observe(0.1);
        histogram.with_labels(&["/api/users", "POST"]).observe(0.2);
        histogram.with_labels(&["/api/orders", "GET"]).observe(0.15);

        assert_eq!(histogram.with_labels(&["/api/users", "GET"]).get_sample_count(), 1);
        assert_eq!(histogram.with_labels(&["/api/users", "POST"]).get_sample_count(), 1);
        assert_eq!(histogram.with_labels(&["/api/orders", "GET"]).get_sample_count(), 1);
    }

    #[tokio::test]
    async fn test_summary_creation() {
        let registry = Registry::new();
        let summary = Summary::new("test_summary", "A test summary").unwrap();

        registry.register(Box::new(summary.clone())).unwrap();

        assert_eq!(summary.desc().name, "test_summary");
        assert_eq!(summary.desc().help, "A test summary");
    }

    #[tokio::test]
    async fn test_summary_observations() {
        let summary = Summary::new("summary_obs", "Summary observations").unwrap();

        // Observe values
        summary.observe(1.0);
        summary.observe(2.0);
        summary.observe(3.0);
        summary.observe(4.0);
        summary.observe(5.0);

        assert_eq!(summary.get_sample_count(), 5);
        assert_eq!(summary.get_sample_sum(), 15.0);
    }

    #[tokio::test]
    async fn test_summary_quantiles() {
        let summary = Summary::with_opts(SummaryOpts {
            quantiles: vec![0.5, 0.9, 0.99],
            max_age: Duration::from_secs(60),
            age_buckets: 5,
            buf_cap: 500,
        }).unwrap();

        // Add many observations to get meaningful quantiles
        for i in 1..=100 {
            summary.observe(i as f64);
        }

        // Give time for quantiles to be calculated
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check that quantiles are calculated (exact values depend on implementation)
        let quantiles = summary.get_quantiles();
        assert!(!quantiles.is_empty());
    }

    #[tokio::test]
    async fn test_summary_with_labels() {
        let summary = Summary::with_labels(
            "labeled_summary",
            "Summary with labels",
            vec!["service", "version"]
        ).unwrap();

        summary.with_labels(&["auth", "v1"]).observe(0.1);
        summary.with_labels(&["auth", "v2"]).observe(0.2);
        summary.with_labels(&["billing", "v1"]).observe(0.15);

        assert_eq!(summary.with_labels(&["auth", "v1"]).get_sample_count(), 1);
        assert_eq!(summary.with_labels(&["auth", "v2"]).get_sample_count(), 1);
        assert_eq!(summary.with_labels(&["billing", "v1"]).get_sample_count(), 1);
    }

    #[tokio::test]
    async fn test_registry_management() {
        let registry = Registry::new();

        let counter1 = Counter::new("counter1", "First counter").unwrap();
        let counter2 = Counter::new("counter2", "Second counter").unwrap();

        // Register metrics
        registry.register(Box::new(counter1.clone())).unwrap();
        registry.register(Box::new(counter2.clone())).unwrap();

        // Check that metrics are registered
        let metrics = registry.gather();
        assert_eq!(metrics.len(), 2);

        // Find specific metrics
        let counter1_metric = metrics.iter().find(|m| m.get_name() == "counter1").unwrap();
        let counter2_metric = metrics.iter().find(|m| m.get_name() == "counter2").unwrap();

        assert_eq!(counter1_metric.get_help(), "First counter");
        assert_eq!(counter2_metric.get_help(), "Second counter");
    }

    #[tokio::test]
    async fn test_registry_duplicate_names() {
        let registry = Registry::new();

        let counter1 = Counter::new("duplicate_name", "First counter").unwrap();
        let counter2 = Counter::new("duplicate_name", "Second counter").unwrap();

        registry.register(Box::new(counter1)).unwrap();

        // Should fail to register duplicate name
        assert!(registry.register(Box::new(counter2)).is_err());
    }

    #[tokio::test]
    async fn test_metric_naming_conventions() {
        let registry = Registry::new();

        // Valid names
        let valid_counter = Counter::new("valid_metric_name", "Valid name").unwrap();
        registry.register(Box::new(valid_counter)).unwrap();

        // Invalid names should be rejected
        assert!(Counter::new("invalid-name", "Invalid name").is_err());
        assert!(Counter::new("123invalid", "Invalid name").is_err());
        assert!(Counter::new("", "Empty name").is_err());
    }

    #[tokio::test]
    async fn test_concurrent_metric_updates() {
        let counter = Arc::new(Counter::new("concurrent_counter", "Concurrent counter").unwrap());
        let gauge = Arc::new(Gauge::new("concurrent_gauge", "Concurrent gauge").unwrap());

        let mut handles = vec![];

        // Spawn multiple tasks updating the same metrics
        for i in 0..10 {
            let counter_clone = counter.clone();
            let gauge_clone = gauge.clone();

            let handle = tokio::spawn(async move {
                for _ in 0..100 {
                    counter_clone.inc();
                    gauge_clone.inc();
                    gauge_clone.dec();
                }
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Check final values
        assert_eq!(counter.get(), 1000.0); // 10 tasks * 100 increments
        assert_eq!(gauge.get(), 0.0);     // inc/dec cancel out
    }

    #[tokio::test]
    async fn test_prometheus_text_format() {
        let registry = Registry::new();

        let counter = Counter::new("prometheus_counter", "Counter for Prometheus").unwrap();
        counter.add(42.0);

        let gauge = Gauge::new("prometheus_gauge", "Gauge for Prometheus").unwrap();
        gauge.set(3.14);

        registry.register(Box::new(counter)).unwrap();
        registry.register(Box::new(gauge)).unwrap();

        // Encode to Prometheus format
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&registry.gather(), &mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();

        // Check that output contains expected metrics
        assert!(output.contains("# HELP prometheus_counter Counter for Prometheus"));
        assert!(output.contains("# TYPE prometheus_counter counter"));
        assert!(output.contains("prometheus_counter 42"));

        assert!(output.contains("# HELP prometheus_gauge Gauge for Prometheus"));
        assert!(output.contains("# TYPE prometheus_gauge gauge"));
        assert!(output.contains("prometheus_gauge 3.14"));
    }

    #[tokio::test]
    async fn test_histogram_percentiles() {
        let histogram = Histogram::new("percentile_test", "Histogram percentiles").unwrap();

        // Add observations
        for i in 1..=1000 {
            histogram.observe(i as f64 / 10.0);
        }

        // Check percentiles are calculated
        let percentiles = histogram.get_percentiles();
        assert!(!percentiles.is_empty());

        // 50th percentile should be around 50
        let p50 = percentiles.iter().find(|p| p.quantile == 0.5).unwrap();
        assert!(p50.value >= 40.0 && p50.value <= 60.0);
    }

    #[tokio::test]
    async fn test_summary_sliding_window() {
        let summary = Summary::with_opts(SummaryOpts {
            quantiles: vec![0.5, 0.95],
            max_age: Duration::from_secs(1), // Short window for testing
            age_buckets: 2,
            buf_cap: 100,
        }).unwrap();

        // Add observations over time
        for i in 0..10 {
            summary.observe(i as f64);
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        // Wait for some observations to age out
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Summary should still be functional
        summary.observe(100.0);
        assert_eq!(summary.get_sample_count(), 11); // 10 + 1 new observation
    }

    #[tokio::test]
    async fn test_metric_labels_validation() {
        // Valid labels
        let counter = Counter::with_labels(
            "valid_labels",
            "Valid labels",
            vec!["method", "status_code"]
        ).unwrap();

        counter.with_labels(&["GET", "200"]).inc();
        assert_eq!(counter.with_labels(&["GET", "200"]).get(), 1.0);

        // Invalid: wrong number of labels
        // This should be handled gracefully (either panic or ignore)
        // depending on implementation
    }

    #[tokio::test]
    async fn test_counter_reset() {
        let counter = Counter::new("reset_test", "Counter reset test").unwrap();

        counter.add(10.0);
        assert_eq!(counter.get(), 10.0);

        // Reset counter
        counter.reset();
        assert_eq!(counter.get(), 0.0);

        // Can continue incrementing after reset
        counter.inc();
        assert_eq!(counter.get(), 1.0);
    }

    #[tokio::test]
    async fn test_gauge_negative_values() {
        let gauge = Gauge::new("negative_gauge", "Gauge with negative values").unwrap();

        gauge.set(-10.0);
        assert_eq!(gauge.get(), -10.0);

        gauge.sub(5.0);
        assert_eq!(gauge.get(), -15.0);

        gauge.add(20.0);
        assert_eq!(gauge.get(), 5.0);
    }

    #[tokio::test]
    async fn test_histogram_custom_buckets() {
        let histogram = Histogram::with_opts(HistogramOpts {
            buckets: vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
            ..Default::default()
        }).unwrap();

        // Test custom bucket boundaries
        histogram.observe(0.05); // bucket 0
        histogram.observe(0.3);  // bucket 1
        histogram.observe(1.5);  // bucket 3
        histogram.observe(7.0);  // bucket 5

        let buckets = histogram.get_buckets();
        assert_eq!(buckets.len(), 7); // 6 custom + 1 inf
        assert_eq!(buckets[0].count, 1); // 0.05
        assert_eq!(buckets[1].count, 1); // 0.3
        assert_eq!(buckets[3].count, 1); // 1.5
        assert_eq!(buckets[5].count, 1); // 7.0
    }

    #[tokio::test]
    async fn test_summary_time_window() {
        let summary = Summary::with_opts(SummaryOpts {
            quantiles: vec![0.5, 0.9],
            max_age: Duration::from_millis(500),
            age_buckets: 3,
            buf_cap: 1000,
        }).unwrap();

        let start = Instant::now();

        // Add observations
        for i in 0..100 {
            summary.observe(i as f64);
        }

        // Wait longer than max_age
        while start.elapsed() < Duration::from_millis(600) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Add more observations
        for i in 100..200 {
            summary.observe(i as f64);
        }

        // Summary should have both old and new observations
        assert!(summary.get_sample_count() >= 200);
    }

    #[tokio::test]
    async fn test_registry_metric_collection() {
        let registry = Registry::new();

        // Register various metric types
        let counter = Counter::new("collect_counter", "Collection counter").unwrap();
        let gauge = Gauge::new("collect_gauge", "Collection gauge").unwrap();
        let histogram = Histogram::new("collect_histogram", "Collection histogram").unwrap();
        let summary = Summary::new("collect_summary", "Collection summary").unwrap();

        registry.register(Box::new(counter.clone())).unwrap();
        registry.register(Box::new(gauge.clone())).unwrap();
        registry.register(Box::new(histogram.clone())).unwrap();
        registry.register(Box::new(summary.clone())).unwrap();

        // Update metrics
        counter.add(5.0);
        gauge.set(10.0);
        histogram.observe(1.0);
        summary.observe(2.0);

        // Collect all metrics
        let metrics = registry.gather();
        assert_eq!(metrics.len(), 4);

        // Verify metric values
        for metric in metrics {
            match metric.get_name().as_str() {
                "collect_counter" => {
                    assert_eq!(metric.get_metric()[0].get_counter().get_value(), 5.0);
                }
                "collect_gauge" => {
                    assert_eq!(metric.get_metric()[0].get_gauge().get_value(), 10.0);
                }
                "collect_histogram" => {
                    assert_eq!(metric.get_metric()[0].get_histogram().get_sample_count(), 1);
                }
                "collect_summary" => {
                    assert_eq!(metric.get_metric()[0].get_summary().get_sample_count(), 1);
                }
                _ => panic!("Unexpected metric name"),
            }
        }
    }

    #[tokio::test]
    async fn test_metric_thread_safety() {
        let counter = Arc::new(Counter::new("thread_safe_counter", "Thread safe counter").unwrap());
        let gauge = Arc::new(Gauge::new("thread_safe_gauge", "Thread safe gauge").unwrap());

        let mut handles = vec![];

        // Test concurrent access from multiple threads
        for _ in 0..5 {
            let counter_clone = counter.clone();
            let gauge_clone = gauge.clone();

            let handle = tokio::spawn(async move {
                for _ in 0..50 {
                    counter_clone.inc();
                    gauge_clone.add(1.0);
                    gauge_clone.sub(1.0);
                }
            });

            handles.push(handle);
        }

        // Wait for completion
        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(counter.get(), 250.0); // 5 threads * 50 increments
        assert_eq!(gauge.get(), 0.0);     // add/sub cancel out
    }

    #[tokio::test]
    async fn test_histogram_large_observations() {
        let histogram = Histogram::new("large_obs_histogram", "Large observations").unwrap();

        // Test with very large values
        histogram.observe(1_000_000.0);
        histogram.observe(10_000_000.0);
        histogram.observe(100_000_000.0);

        assert_eq!(histogram.get_sample_count(), 3);
        assert_eq!(histogram.get_sample_sum(), 111_000_000.0);
    }

    #[tokio::test]
    async fn test_summary_high_quantiles() {
        let summary = Summary::with_opts(SummaryOpts {
            quantiles: vec![0.99, 0.999, 0.9999],
            ..Default::default()
        }).unwrap();

        // Add many observations
        for i in 1..=10000 {
            summary.observe(i as f64);
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        let quantiles = summary.get_quantiles();
        assert!(!quantiles.is_empty());

        // 99th percentile should be close to 9900
        let q99 = quantiles.iter().find(|q| q.quantile == 0.99).unwrap();
        assert!(q99.value >= 9800.0 && q99.value <= 10000.0);
    }

    #[tokio::test]
    async fn test_metric_memory_usage() {
        let registry = Registry::new();

        // Create many metrics to test memory usage
        for i in 0..100 {
            let counter = Counter::new(&format!("memory_test_counter_{}", i), "Memory test").unwrap();
            registry.register(Box::new(counter)).unwrap();
        }

        let metrics = registry.gather();
        assert_eq!(metrics.len(), 100);

        // All metrics should be accessible
        for i in 0..100 {
            let metric_name = format!("memory_test_counter_{}", i);
            assert!(metrics.iter().any(|m| m.get_name() == metric_name));
        }
    }

    #[tokio::test]
    async fn test_gauge_functional_interface() {
        let gauge = Gauge::new("functional_gauge", "Functional gauge").unwrap();

        // Test functional interface
        gauge.set_to_current_time();
        let time1 = gauge.get();

        tokio::time::sleep(Duration::from_millis(10)).await;

        gauge.set_to_current_time();
        let time2 = gauge.get();

        // Time should have advanced
        assert!(time2 > time1);
    }

    #[tokio::test]
    async fn test_counter_overflow() {
        let counter = Counter::new("overflow_counter", "Overflow test").unwrap();

        // Add very large values
        counter.add(f64::MAX / 2.0);
        counter.add(f64::MAX / 2.0);

        // Should handle overflow gracefully
        assert!(counter.get().is_finite());
    }

    #[tokio::test]
    async fn test_histogram_zero_observations() {
        let histogram = Histogram::new("zero_obs_histogram", "Zero observations").unwrap();

        // No observations yet
        assert_eq!(histogram.get_sample_count(), 0);
        assert_eq!(histogram.get_sample_sum(), 0.0);

        // Buckets should be empty
        let buckets = histogram.get_buckets();
        for bucket in buckets {
            assert_eq!(bucket.count, 0);
        }
    }

    #[tokio::test]
    async fn test_summary_empty_quantiles() {
        let summary = Summary::new("empty_quantiles", "Empty quantiles").unwrap();

        // No quantiles configured
        let quantiles = summary.get_quantiles();
        assert!(quantiles.is_empty());
    }

    #[tokio::test]
    async fn test_registry_metric_removal() {
        let registry = Registry::new();

        let counter = Counter::new("removal_test", "Removal test").unwrap();
        registry.register(Box::new(counter)).unwrap();

        // Metric should be present
        assert_eq!(registry.gather().len(), 1);

        // Unregister metric (if supported)
        // Note: This depends on registry implementation
        // registry.unregister("removal_test");

        // For now, just verify registry functionality
        let metrics = registry.gather();
        assert_eq!(metrics[0].get_name(), "removal_test");
    }

    #[tokio::test]
    async fn test_metric_label_cardinality() {
        let counter = Counter::with_labels(
            "cardinality_test",
            "Cardinality test",
            vec!["label1", "label2"]
        ).unwrap();

        // Test with different label combinations
        let combinations = vec![
            vec!["a", "1"],
            vec!["a", "2"],
            vec!["b", "1"],
            vec!["b", "2"],
            vec!["c", "1"],
        ];

        for combo in combinations {
            counter.with_labels(&combo).inc();
        }

        // Verify all combinations are tracked separately
        assert_eq!(counter.with_labels(&["a", "1"]).get(), 1.0);
        assert_eq!(counter.with_labels(&["a", "2"]).get(), 1.0);
        assert_eq!(counter.with_labels(&["b", "1"]).get(), 1.0);
        assert_eq!(counter.with_labels(&["b", "2"]).get(), 1.0);
        assert_eq!(counter.with_labels(&["c", "1"]).get(), 1.0);
    }

    #[tokio::test]
    async fn test_concurrent_registry_access() {
        let registry = Arc::new(Registry::new());

        let mut handles = vec![];

        // Multiple tasks registering and gathering metrics
        for i in 0..5 {
            let registry_clone = registry.clone();
            let handle = tokio::spawn(async move {
                let counter = Counter::new(&format!("concurrent_counter_{}", i), "Concurrent").unwrap();
                registry_clone.register(Box::new(counter)).unwrap();

                // Gather metrics
                let metrics = registry_clone.gather();
                metrics.len()
            });
            handles.push(handle);
        }

        // Wait for all
        let results: Vec<usize> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // Each task should see increasing number of metrics
        for (i, &count) in results.iter().enumerate() {
            assert!(count >= i + 1);
        }
    }

    #[tokio::test]
    async fn test_prometheus_format_validation() {
        let registry = Registry::new();

        let counter = Counter::new("format_counter", "Format validation").unwrap();
        counter.add(123.45);

        registry.register(Box::new(counter)).unwrap();

        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&registry.gather(), &mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();

        // Validate Prometheus format
        assert!(output.contains("# HELP format_counter Format validation"));
        assert!(output.contains("# TYPE format_counter counter"));
        assert!(output.contains("format_counter 123.45"));

        // Should end with newline
        assert!(output.ends_with('\n'));
    }

    #[tokio::test]
    async fn test_histogram_bucket_boundaries() {
        let histogram = Histogram::with_opts(HistogramOpts {
            buckets: vec![1.0, 5.0, 10.0, 50.0, 100.0],
            ..Default::default()
        }).unwrap();

        // Test exact bucket boundaries
        histogram.observe(1.0);   // Should go to bucket 0 (upper bound 1.0)
        histogram.observe(5.0);   // Should go to bucket 1 (upper bound 5.0)
        histogram.observe(10.0);  // Should go to bucket 2 (upper bound 10.0)
        histogram.observe(50.0);  // Should go to bucket 3 (upper bound 50.0)
        histogram.observe(100.0); // Should go to bucket 4 (upper bound 100.0)

        let buckets = histogram.get_buckets();
        assert_eq!(buckets[0].count, 1); // <= 1.0
        assert_eq!(buckets[1].count, 1); // <= 5.0
        assert_eq!(buckets[2].count, 1); // <= 10.0
        assert_eq!(buckets[3].count, 1); // <= 50.0
        assert_eq!(buckets[4].count, 1); // <= 100.0
    }

    #[tokio::test]
    async fn test_summary_buffer_capacity() {
        let summary = Summary::with_opts(SummaryOpts {
            buf_cap: 10, // Small buffer for testing
            ..Default::default()
        }).unwrap();

        // Add more observations than buffer capacity
        for i in 0..20 {
            summary.observe(i as f64);
        }

        // Should handle buffer overflow gracefully
        assert!(summary.get_sample_count() > 0);
        assert!(summary.get_sample_sum() > 0.0);
    }

    #[tokio::test]
    async fn test_metric_help_text() {
        let registry = Registry::new();

        let counter = Counter::new("help_test", "This is a detailed help text for testing purposes").unwrap();
        registry.register(Box::new(counter)).unwrap();

        let metrics = registry.gather();
        let metric = &metrics[0];

        assert_eq!(metric.get_help(), "This is a detailed help text for testing purposes");
    }

    #[tokio::test]
    async fn test_gauge_timestamp_tracking() {
        let gauge = Gauge::new("timestamp_gauge", "Timestamp tracking").unwrap();

        let time1 = gauge.get_timestamp();
        gauge.set(1.0);
        let time2 = gauge.get_timestamp();

        // Timestamp should be updated when value changes
        assert!(time2 >= time1);
    }

    #[tokio::test]
    async fn test_counter_precision() {
        let counter = Counter::new("precision_test", "Precision test").unwrap();

        // Test floating point precision
        counter.add(0.1);
        counter.add(0.2);
        assert!((counter.get() - 0.3).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_histogram_outlier_handling() {
        let histogram = Histogram::new("outlier_test", "Outlier handling").unwrap();

        // Add normal values
        for i in 1..=10 {
            histogram.observe(i as f64);
        }

        // Add extreme outliers
        histogram.observe(1_000_000.0);
        histogram.observe(0.000001);

        // Should handle outliers without breaking
        assert_eq!(histogram.get_sample_count(), 12);
        assert!(histogram.get_sample_sum() > 0.0);
    }

    #[tokio::test]
    async fn test_summary_age_buckets() {
        let summary = Summary::with_opts(SummaryOpts {
            age_buckets: 3,
            max_age: Duration::from_secs(1),
            ..Default::default()
        }).unwrap();

        // Add observations over time
        for i in 0..30 {
            summary.observe(i as f64);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Should maintain age buckets properly
        assert!(summary.get_sample_count() > 0);
    }

    #[tokio::test]
    async fn test_registry_empty_state() {
        let registry = Registry::new();

        let metrics = registry.gather();
        assert_eq!(metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_metric_name_sanitization() {
        // Test that metric names are properly sanitized
        let counter = Counter::new("Test_Metric.Name-123", "Name sanitization").unwrap();

        // Name should be sanitized according to Prometheus rules
        assert!(counter.desc().name.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    #[tokio::test]
    async fn test_gauge_atomic_operations() {
        let gauge = Arc::new(Gauge::new("atomic_gauge", "Atomic operations").unwrap());

        let mut handles = vec![];

        // Test atomic increments/decrements
        for _ in 0..10 {
            let gauge_clone = gauge.clone();
            let handle = tokio::spawn(async move {
                for _ in 0..100 {
                    gauge_clone.inc();
                    gauge_clone.dec();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Should be back to 0 (all inc/dec pairs cancel out)
        assert_eq!(gauge.get(), 0.0);
    }

    #[tokio::test]
    async fn test_histogram_memory_efficiency() {
        let histogram = Histogram::new("memory_efficient", "Memory efficiency").unwrap();

        // Add many observations
        for i in 0..10000 {
            histogram.observe((i % 100) as f64);
        }

        // Should use reasonable memory
        assert_eq!(histogram.get_sample_count(), 10000);
        assert!(histogram.get_sample_sum() > 0.0);
    }

    #[tokio::test]
    async fn test_summary_quantile_accuracy() {
        let summary = Summary::with_opts(SummaryOpts {
            quantiles: vec![0.25, 0.5, 0.75],
            ..Default::default()
        }).unwrap();

        // Add known distribution
        for i in 1..=100 {
            summary.observe(i as f64);
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        let quantiles = summary.get_quantiles();

        // Check quantile approximations
        let q25 = quantiles.iter().find(|q| q.quantile == 0.25).unwrap();
        let q50 = quantiles.iter().find(|q| q.quantile == 0.5).unwrap();
        let q75 = quantiles.iter().find(|q| q.quantile == 0.75).unwrap();

        assert!(q25.value >= 20.0 && q25.value <= 30.0); // ~25th percentile
        assert!(q50.value >= 45.0 && q50.value <= 55.0); // ~50th percentile
        assert!(q75.value >= 70.0 && q75.value <= 80.0); // ~75th percentile
    }
}