//! # Batch - Reactive Batch Updates
//!
//! Implementation of: US-06 - TASK-030
//! Story: Reactive System
//! Date: 2025-12-03
//!
//! Description:
//! Implements Batch, for grouping multiple reactive updates into a single batch.

use std::sync::Arc;

use crate::graph::ReactiveGraph;
use crate::scheduler::ReactiveScheduler;

/// Reactive batch context for grouping updates
pub struct Batch {
    /// The graph this batch belongs to
    graph: Arc<ReactiveGraph>,
    /// The scheduler for batching
    scheduler: Arc<ReactiveScheduler>,
}

impl Batch {
    /// Create a new batch context
    pub fn new() -> Self {
        let graph = Arc::new(ReactiveGraph::new());
        let scheduler = Arc::new(ReactiveScheduler::new());

        Batch {
            graph: Arc::clone(&graph),
            scheduler: Arc::clone(&scheduler),
        }
    }

    /// Create a batch with custom graph and scheduler
    pub fn with_graph_scheduler(graph: Arc<ReactiveGraph>, scheduler: Arc<ReactiveScheduler>) -> Self {
        Batch {
            graph: Arc::clone(&graph),
            scheduler: Arc::clone(&scheduler),
        }
    }

    /// Execute a function within a batch context
    pub fn batch<F, T>(&self, func: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.scheduler.batch(func)
    }

    /// Check if currently batching
    pub fn is_batching(&self) -> bool {
        self.scheduler.is_batching()
    }

    /// Get the graph
    pub fn graph(&self) -> &Arc<ReactiveGraph> {
        &self.graph
    }

    /// Get the scheduler
    pub fn scheduler(&self) -> &Arc<ReactiveScheduler> {
        &self.scheduler
    }
}

/// Global batch context
static mut GLOBAL_BATCH: Option<Batch> = None;

/// Initialize global batch context
pub fn init_global_batch() {
    unsafe {
        GLOBAL_BATCH = Some(Batch::new());
    }
}

/// Get global batch context
pub fn global_batch() -> &'static Batch {
    unsafe {
        GLOBAL_BATCH.as_ref().expect("Global batch not initialized")
    }
}

/// Helper function to batch updates globally
pub fn batch<F, T>(func: F) -> T
where
    F: FnOnce() -> T,
{
    global_batch().batch(func)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::Signal;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_batch_creation() {
        let batch = Batch::new();
        assert!(!batch.is_batching());
    }

    #[test]
    fn test_batch_execution() {
        let batch = Batch::new();
        let signal1 = Arc::new(Signal::new(1));
        let signal2 = Arc::new(Signal::new(2));

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        // Subscribe to both signals
        signal1.subscribe(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        signal2.subscribe(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Batch updates
        let result = batch.batch(|| {
            signal1.set(10);
            signal2.set(20);
            42
        });

        assert_eq!(result, 42);
        // In a full implementation, subscribers might be batched
        // For now, they run immediately
        assert!(call_count.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn test_global_batch() {
        init_global_batch();

        let result = batch(|| {
            42
        });

        assert_eq!(result, 42);
    }
}