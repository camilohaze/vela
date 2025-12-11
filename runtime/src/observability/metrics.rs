//! Metrics collection and export for Vela services
//!
//! This module provides metrics collection using Prometheus-compatible
//! metric types: Counter, Gauge, Histogram, and Summary.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Labels for metrics dimensions
pub type Labels = HashMap<String, String>;

/// Metric value types
#[derive(Debug, Clone)]
pub enum Value {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary(f64),
}

/// Configuration for the metrics system
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Service name for metrics
    pub service_name: String,
    /// Default labels applied to all metrics
    pub default_labels: Labels,
    /// Histogram buckets for latency metrics
    pub histogram_buckets: Vec<f64>,
    /// Summary quantiles
    pub summary_quantiles: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            service_name: "vela-service".to_string(),
            default_labels: HashMap::new(),
            histogram_buckets: vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            summary_quantiles: vec![0.5, 0.9, 0.95, 0.99],
        }
    }
}

/// Counter metric - monotonically increasing value
#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    help: String,
    labels: Labels,
    value: Arc<RwLock<u64>>,
}

impl Counter {
    /// Create a new counter
    pub fn new(name: &str, help: &str) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            labels: HashMap::new(),
            value: Arc::new(RwLock::new(0)),
        }
    }

    /// Increment the counter by 1
    pub async fn increment(&self) {
        *self.value.write().await += 1;
    }

    /// Increment the counter by a specific amount
    pub async fn increment_by(&self, amount: u64) {
        *self.value.write().await += amount;
    }

    /// Get the current value
    pub async fn get(&self) -> u64 {
        *self.value.read().await
    }

    /// Reset the counter to 0
    pub async fn reset(&self) {
        *self.value.write().await = 0;
    }
}

/// Gauge metric - value that can go up and down
#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    help: String,
    labels: Labels,
    value: Arc<RwLock<f64>>,
}

impl Gauge {
    /// Create a new gauge
    pub fn new(name: &str, help: &str) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            labels: HashMap::new(),
            value: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Set the gauge to a specific value
    pub async fn set(&self, value: f64) {
        *self.value.write().await = value;
    }

    /// Increment the gauge by 1
    pub async fn increment(&self) {
        *self.value.write().await += 1.0;
    }

    /// Decrement the gauge by 1
    pub async fn decrement(&self) {
        *self.value.write().await -= 1.0;
    }

    /// Add a value to the gauge
    pub async fn add(&self, value: f64) {
        *self.value.write().await += value;
    }

    /// Subtract a value from the gauge
    pub async fn subtract(&self, value: f64) {
        *self.value.write().await -= value;
    }

    /// Get the current value
    pub async fn get(&self) -> f64 {
        *self.value.read().await
    }
}

/// Histogram metric - samples observations and counts them in buckets
#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    help: String,
    labels: Labels,
    buckets: Vec<f64>,
    observations: Arc<RwLock<Vec<f64>>>,
    count: Arc<RwLock<u64>>,
    sum: Arc<RwLock<f64>>,
}

impl Histogram {
    /// Create a new histogram
    pub fn new(name: &str, help: &str, buckets: Vec<f64>) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            labels: HashMap::new(),
            buckets,
            observations: Arc::new(RwLock::new(Vec::new())),
            count: Arc::new(RwLock::new(0)),
            sum: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Observe a value
    pub async fn observe(&self, value: f64) {
        self.observations.write().await.push(value);
        *self.count.write().await += 1;
        *self.sum.write().await += value;
    }

    /// Get the count of observations
    pub async fn count(&self) -> u64 {
        *self.count.read().await
    }

    /// Get the sum of all observations
    pub async fn sum(&self) -> f64 {
        *self.sum.read().await
    }

    /// Get bucket counts
    pub async fn bucket_counts(&self) -> Vec<(f64, u64)> {
        let observations = self.observations.read().await;
        let mut counts = vec![0u64; self.buckets.len() + 1];

        for &obs in observations.iter() {
            let mut bucket_index = 0;
            for (i, &bucket) in self.buckets.iter().enumerate() {
                if obs <= bucket {
                    bucket_index = i;
                    break;
                }
                bucket_index = i + 1;
            }
            counts[bucket_index] += 1;
        }

        self.buckets.iter().cloned()
            .chain(std::iter::once(f64::INFINITY))
            .zip(counts)
            .collect()
    }
}

/// Summary metric - calculates quantiles over a sliding time window
#[derive(Debug, Clone)]
pub struct Summary {
    name: String,
    help: String,
    labels: Labels,
    quantiles: Vec<f64>,
    observations: Arc<RwLock<Vec<f64>>>,
    count: Arc<RwLock<u64>>,
    sum: Arc<RwLock<f64>>,
}

impl Summary {
    /// Create a new summary
    pub fn new(name: &str, help: &str, quantiles: Vec<f64>) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            labels: HashMap::new(),
            quantiles,
            observations: Arc::new(RwLock::new(Vec::new())),
            count: Arc::new(RwLock::new(0)),
            sum: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Observe a value
    pub async fn observe(&self, value: f64) {
        self.observations.write().await.push(value);
        *self.count.write().await += 1;
        *self.sum.write().await += value;
    }

    /// Get the count of observations
    pub async fn count(&self) -> u64 {
        *self.count.read().await
    }

    /// Get the sum of all observations
    pub async fn sum(&self) -> f64 {
        *self.sum.read().await
    }

    /// Get quantile values
    pub async fn quantiles(&self) -> Vec<(f64, f64)> {
        let mut observations = self.observations.read().await.clone();
        observations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.quantiles.iter().map(|&q| {
            let index = (q * (observations.len() - 1) as f64) as usize;
            let value = if observations.is_empty() {
                0.0
            } else {
                observations[index]
            };
            (q, value)
        }).collect()
    }
}

/// Metrics registry for managing all metrics
#[derive(Debug)]
pub struct MetricsRegistry {
    config: MetricsConfig,
    counters: Arc<RwLock<HashMap<String, Counter>>>,
    gauges: Arc<RwLock<HashMap<String, Gauge>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
    summaries: Arc<RwLock<HashMap<String, Summary>>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            summaries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a counter
    pub async fn register_counter(&self, name: &str, help: &str) -> Result<(), Box<dyn std::error::Error>> {
        let counter = Counter::new(name, help);
        self.counters.write().await.insert(name.to_string(), counter);
        Ok(())
    }

    /// Register a gauge
    pub async fn register_gauge(&self, name: &str, help: &str) -> Result<(), Box<dyn std::error::Error>> {
        let gauge = Gauge::new(name, help);
        self.gauges.write().await.insert(name.to_string(), gauge);
        Ok(())
    }

    /// Register a histogram
    pub async fn register_histogram(&self, name: &str, help: &str) -> Result<(), Box<dyn std::error::Error>> {
        let histogram = Histogram::new(name, help, self.config.histogram_buckets.clone());
        self.histograms.write().await.insert(name.to_string(), histogram);
        Ok(())
    }

    /// Register a summary
    pub async fn register_summary(&self, name: &str, help: &str) -> Result<(), Box<dyn std::error::Error>> {
        let summary = Summary::new(name, help, self.config.summary_quantiles.clone());
        self.summaries.write().await.insert(name.to_string(), summary);
        Ok(())
    }

    /// Get a counter by name
    pub async fn get_counter(&self, name: &str) -> Option<Counter> {
        self.counters.read().await.get(name).cloned()
    }

    /// Get a gauge by name
    pub async fn get_gauge(&self, name: &str) -> Option<Gauge> {
        self.gauges.read().await.get(name).cloned()
    }

    /// Get a histogram by name
    pub async fn get_histogram(&self, name: &str) -> Option<Histogram> {
        self.histograms.read().await.get(name).cloned()
    }

    /// Get a summary by name
    pub async fn get_summary(&self, name: &str) -> Option<Summary> {
        self.summaries.read().await.get(name).cloned()
    }

    /// Export all metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Export counters
        for (name, counter) in self.counters.read().await.iter() {
            output.push_str(&format!("# HELP {} {}\n", name, counter.help));
            output.push_str(&format!("# TYPE {} counter\n", name));
            output.push_str(&format!("{} {}\n", name, counter.get().await));
        }

        // Export gauges
        for (name, gauge) in self.gauges.read().await.iter() {
            output.push_str(&format!("# HELP {} {}\n", name, gauge.help));
            output.push_str(&format!("# TYPE {} gauge\n", name));
            output.push_str(&format!("{} {}\n", name, gauge.get().await));
        }

        // Export histograms
        for (name, histogram) in self.histograms.read().await.iter() {
            output.push_str(&format!("# HELP {} {}\n", name, histogram.help));
            output.push_str(&format!("# TYPE {} histogram\n", name));

            let bucket_counts = histogram.bucket_counts().await;
            for (bucket, count) in bucket_counts {
                if bucket == f64::INFINITY {
                    output.push_str(&format!("{}_bucket{{le=\"+Inf\"}} {}\n", name, count));
                } else {
                    output.push_str(&format!("{}_bucket{{le=\"{}\"}} {}\n", name, bucket, count));
                }
            }
            output.push_str(&format!("{}_count {}\n", name, histogram.count().await));
            output.push_str(&format!("{}_sum {}\n", name, histogram.sum().await));
        }

        // Export summaries
        for (name, summary) in self.summaries.read().await.iter() {
            output.push_str(&format!("# HELP {} {}\n", name, summary.help));
            output.push_str(&format!("# TYPE {} summary\n", name));

            let quantiles = summary.quantiles().await;
            for (quantile, value) in quantiles {
                output.push_str(&format!("{}{{quantile=\"{}\"}} {}\n", name, quantile, value));
            }
            output.push_str(&format!("{}_count {}\n", name, summary.count().await));
            output.push_str(&format!("{}_sum {}\n", name, summary.sum().await));
        }

        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new(MetricsConfig::default())
    }
}

/// Global metrics registry
pub struct GlobalMetricsRegistry {
    registry: Arc<RwLock<Option<MetricsRegistry>>>,
}

impl GlobalMetricsRegistry {
    /// Create a new global registry
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the global registry
    pub async fn init(&self, config: MetricsConfig) -> Result<(), Box<dyn std::error::Error>> {
        let registry = MetricsRegistry::new(config);
        *self.registry.write().await = Some(registry);
        Ok(())
    }

    /// Get the global registry
    pub async fn get(&self) -> Option<MetricsRegistry> {
        self.registry.read().await.clone()
    }
}

impl Default for GlobalMetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global static registry instance
static GLOBAL_METRICS: once_cell::sync::Lazy<GlobalMetricsRegistry> = once_cell::sync::Lazy::new(|| {
    GlobalMetricsRegistry::new()
});

/// Get the global metrics registry
pub fn global_metrics() -> &'static GlobalMetricsRegistry {
    &GLOBAL_METRICS
}

/// Initialize global metrics
pub async fn init_metrics(config: MetricsConfig) -> Result<(), Box<dyn std::error::Error>> {
    global_metrics().init(config).await
}

/// Get the global metrics registry instance
pub async fn get_metrics() -> Option<MetricsRegistry> {
    global_metrics().get().await
}