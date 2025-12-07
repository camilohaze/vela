//! # Effect - Reactive Side Effects
//!
//! Implementation of: US-06 - TASK-028
//! Story: Reactive System
//! Date: 2025-12-03
//!
//! Description:
//! Implements Effect, side effects that run when dependencies change.

use std::sync::Arc;

use crate::graph::{ReactiveGraph, ReactiveNode, NodeState};

/// Reactive side effect
pub struct Effect {
    /// The reactive node backing this effect
    node: Arc<ReactiveNode>,
    /// The graph this effect belongs to
    graph: Arc<ReactiveGraph>,
}

impl Effect {
    /// Create a new effect (runs immediately)
    pub fn new<F>(effect_fn: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let graph = Arc::new(ReactiveGraph::new());
        Self::with_graph(effect_fn, graph)
    }

    /// Create a new effect with custom graph
    pub fn with_graph<F>(effect_fn: F, graph: Arc<ReactiveGraph>) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = format!("effect-{}", uuid::Uuid::new_v4());

        let effect_fn_boxed = Box::new(move || {
            effect_fn();
        });

        let node = Arc::new(ReactiveNode::new_effect(id, effect_fn_boxed));
        graph.register_node(Arc::clone(&node));

        let effect = Effect {
            node: Arc::clone(&node),
            graph: Arc::clone(&graph),
        };

        // Run immediately
        effect.run();

        effect
    }

    /// Run the effect
    pub fn run(&self) {
        if self.is_stopped() {
            return;
        }

        // Run cleanup if exists
        if let Some(cleanup) = self.node.cleanup_fn.lock().unwrap().take() {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(cleanup));
        }

        // Run effect function
        if let Some(ref compute_fn) = self.node.compute_fn {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _result = compute_fn();
            }));
        }
    }

    /// Stop the effect
    pub fn stop(&self) {
        // Mark as disposed to stop
        *self.node.state.write() = NodeState::Disposed;
    }

    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        matches!(*self.node.state.read(), NodeState::Disposed)
    }

    /// Resume the effect (if stopped)
    pub fn resume(&self) {
        if self.is_stopped() {
            *self.node.state.write() = NodeState::Dirty;
            self.run();
        }
    }

    /// Dispose the effect
    pub fn dispose(&self) {
        self.node.dispose();
        self.graph.unregister_node(&self.node.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_effect_creation() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let _effect = Effect::new(move || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Effect should run immediately
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_effect_stop_resume() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let effect = Effect::new(move || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        effect.stop();
        assert!(effect.is_stopped());

        effect.resume();
        assert!(!effect.is_stopped());
        // Should have run again
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_effect_dispose() {
        let effect = Effect::new(|| {});
        effect.dispose();
        assert!(effect.is_stopped());
    }
}