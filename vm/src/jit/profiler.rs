//! Hotspot Profiler
//!
//! Detects frequently executed functions (hotspots) that are candidates
//! for JIT compilation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Function identifier type
pub type FunctionId = String;

/// Hotspot profiler for identifying frequently executed functions
#[derive(Debug)]
pub struct HotspotProfiler {
    /// Call counters for each function
    call_counts: HashMap<FunctionId, Arc<AtomicU64>>,
    /// Threshold for considering a function a hotspot
    threshold: u32,
    /// Total profiling overhead (for monitoring)
    profiling_overhead_ns: Arc<AtomicU64>,
}

impl HotspotProfiler {
    /// Create a new hotspot profiler
    pub fn new(threshold: u32) -> Self {
        Self {
            call_counts: HashMap::new(),
            threshold,
            profiling_overhead_ns: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a function call
    pub fn record_call(&mut self, function_id: FunctionId) {
        let start = std::time::Instant::now();

        let counter = self.call_counts
            .entry(function_id)
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));

        counter.fetch_add(1, Ordering::Relaxed);

        let elapsed = start.elapsed().as_nanos() as u64;
        self.profiling_overhead_ns.fetch_add(elapsed, Ordering::Relaxed);
    }

    /// Check if a function is a hotspot
    pub fn is_hotspot(&self, function_id: &FunctionId) -> bool {
        self.call_counts
            .get(function_id)
            .map(|counter| counter.load(Ordering::Relaxed) >= self.threshold as u64)
            .unwrap_or(false)
    }

    /// Get call count for a function
    pub fn get_call_count(&self, function_id: &FunctionId) -> u64 {
        self.call_counts
            .get(function_id)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get all hotspots above the threshold
    pub fn get_hotspots(&self) -> Vec<(FunctionId, u64)> {
        self.call_counts
            .iter()
            .filter_map(|(id, counter)| {
                let count = counter.load(Ordering::Relaxed);
                if count >= self.threshold as u64 {
                    Some((id.clone(), count))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Reset profiling data
    pub fn reset(&mut self) {
        self.call_counts.clear();
        self.profiling_overhead_ns.store(0, Ordering::Relaxed);
    }

    /// Get profiling statistics
    pub fn stats(&self) -> ProfilingStats {
        let total_calls: u64 = self.call_counts
            .values()
            .map(|counter| counter.load(Ordering::Relaxed))
            .sum();

        let hotspots = self.get_hotspots().len();

        ProfilingStats {
            total_functions: self.call_counts.len(),
            total_calls,
            hotspots,
            threshold: self.threshold,
            profiling_overhead_ns: self.profiling_overhead_ns.load(Ordering::Relaxed),
        }
    }

    /// Update the hotspot threshold
    pub fn set_threshold(&mut self, threshold: u32) {
        self.threshold = threshold;
    }
}

/// Profiling statistics
#[derive(Debug, Clone)]
pub struct ProfilingStats {
    /// Total number of functions profiled
    pub total_functions: usize,
    /// Total number of calls recorded
    pub total_calls: u64,
    /// Number of hotspots detected
    pub hotspots: usize,
    /// Current hotspot threshold
    pub threshold: u32,
    /// Profiling overhead in nanoseconds
    pub profiling_overhead_ns: u64,
}

impl Default for HotspotProfiler {
    fn default() -> Self {
        Self::new(1000) // Default threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = HotspotProfiler::new(100);
        assert_eq!(profiler.threshold, 100);
        assert!(profiler.call_counts.is_empty());
    }

    #[test]
    fn test_call_recording() {
        let mut profiler = HotspotProfiler::new(5);

        profiler.record_call("func1".to_string());
        profiler.record_call("func1".to_string());
        profiler.record_call("func2".to_string());

        assert_eq!(profiler.get_call_count(&"func1".to_string()), 2);
        assert_eq!(profiler.get_call_count(&"func2".to_string()), 1);
        assert_eq!(profiler.get_call_count(&"func3".to_string()), 0);
    }

    #[test]
    fn test_hotspot_detection() {
        let mut profiler = HotspotProfiler::new(3);

        // Record calls below threshold
        profiler.record_call("func1".to_string());
        profiler.record_call("func1".to_string());
        assert!(!profiler.is_hotspot(&"func1".to_string()));

        // Record one more call to reach threshold
        profiler.record_call("func1".to_string());
        assert!(profiler.is_hotspot(&"func1".to_string()));
    }

    #[test]
    fn test_get_hotspots() {
        let mut profiler = HotspotProfiler::new(2);

        profiler.record_call("func1".to_string());
        profiler.record_call("func1".to_string());
        profiler.record_call("func1".to_string()); // hotspot

        profiler.record_call("func2".to_string());
        profiler.record_call("func2".to_string()); // hotspot

        profiler.record_call("func3".to_string()); // not hotspot

        let hotspots = profiler.get_hotspots();
        assert_eq!(hotspots.len(), 2);

        let func1 = hotspots.iter().find(|(id, _)| id == "func1").unwrap();
        assert_eq!(func1.1, 3);

        let func2 = hotspots.iter().find(|(id, _)| id == "func2").unwrap();
        assert_eq!(func2.1, 2);
    }

    #[test]
    fn test_stats() {
        let mut profiler = HotspotProfiler::new(2);

        profiler.record_call("func1".to_string());
        profiler.record_call("func1".to_string());
        profiler.record_call("func2".to_string());

        let stats = profiler.stats();
        assert_eq!(stats.total_functions, 2);
        assert_eq!(stats.total_calls, 3);
        assert_eq!(stats.hotspots, 1); // only func1
        assert_eq!(stats.threshold, 2);
    }

    #[test]
    fn test_reset() {
        let mut profiler = HotspotProfiler::new(1);

        profiler.record_call("func1".to_string());
        assert_eq!(profiler.get_call_count(&"func1".to_string()), 1);

        profiler.reset();
        assert_eq!(profiler.get_call_count(&"func1".to_string()), 0);
        assert!(profiler.call_counts.is_empty());
    }
}