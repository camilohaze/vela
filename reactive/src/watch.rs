//! # Watch - Reactive Watcher
//!
//! Implementation of: US-06 - TASK-029
//! Story: Reactive System
//! Date: 2025-12-03
//!
//! Description:
//! Implements Watch, a helper for observing changes in reactive values.

use std::sync::Arc;

use crate::graph::ReactiveGraph;
use crate::signal::Signal;

/// Reactive watcher for observing changes
pub struct Watch<T> {
    /// The signal being watched
    signal: Arc<Signal<T>>,
    /// The graph this watch belongs to
    graph: Arc<ReactiveGraph>,
    /// Whether watching immediately
    immediate: bool,
    /// Whether currently stopped
    stopped: Arc<std::sync::Mutex<bool>>,
}

impl<T> Watch<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new watch (immediate by default)
    pub fn new<F>(signal: Arc<Signal<T>>, callback: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        Self::new_with_options(signal, callback, true)
    }

    /// Create a new watch with options
    pub fn new_with_options<F>(signal: Arc<Signal<T>>, callback: F, immediate: bool) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let graph = Arc::new(ReactiveGraph::new());

        let watch = Watch {
            signal: Arc::clone(&signal),
            graph: Arc::clone(&graph),
            immediate,
            stopped: Arc::new(std::sync::Mutex::new(false)),
        };

        // Create callback arc first
        let callback_arc = Arc::new(callback);

        // Subscribe to signal changes
        let callback_clone = Arc::clone(&callback_arc);
        let stopped = watch.stopped.clone();

        signal.subscribe(move |old_value, new_value| {
            if !*stopped.lock().unwrap() {
                callback_clone(&new_value);
            }
        });

        // Run immediately if requested
        if immediate {
            callback_arc(&signal.get());
        }

        watch
    }

    /// Stop watching
    pub fn stop(&self) {
        *self.stopped.lock().unwrap() = true;
    }

    /// Resume watching
    pub fn resume(&self) {
        *self.stopped.lock().unwrap() = false;
    }

    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        *self.stopped.lock().unwrap()
    }

    /// Get the current value
    pub fn value(&self) -> T {
        self.signal.get()
    }

    /// Dispose the watch
    pub fn dispose(&self) {
        self.stop();
        // Note: In a full implementation, we'd clean up the subscription
    }
}

/// Helper function to create a watch
pub fn watch<T, F>(signal: Arc<Signal<T>>, callback: F) -> Watch<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn(&T) + Send + Sync + 'static,
{
    Watch::new(signal, callback)
}

/// Helper function to create a watch with options
pub fn watch_with_options<T, F>(signal: Arc<Signal<T>>, callback: F, immediate: bool) -> Watch<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn(&T) + Send + Sync + 'static,
{
    Watch::new_with_options(signal, callback, immediate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_watch_creation() {
        let signal = Arc::new(Signal::new(42));
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let _watch = Watch::new(Arc::clone(&signal), move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Should have called immediately
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_watch_changes() {
        let signal = Arc::new(Signal::new(42));
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let _watch = Watch::new(Arc::clone(&signal), move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Initial call
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Change signal
        signal.set(43);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_watch_stop_resume() {
        let signal = Arc::new(Signal::new(42));
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let watch = Watch::new(Arc::clone(&signal), move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        watch.stop();
        signal.set(43);
        // Should not have called again
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        watch.resume();
        signal.set(44);
        // Should have called again
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_watch_value() {
        let signal = Arc::new(Signal::new(42));
        let watch = Watch::new(Arc::clone(&signal), |_| {});

        assert_eq!(watch.value(), 42);

        signal.set(43);
        assert_eq!(watch.value(), 43);
    }
}