//! # Reactive Build Context - Signal Integration for Widgets
//!
//! Implementation of: VELA-058 - TASK-058
//! Story: Signal Integration
//! Date: 2025-12-03
//!
//! Description:
//! Provides ReactiveBuildContext for automatic dependency tracking
//! during widget builds. Tracks which signals are read during build()
//! and enables selective widget rebuilds when dependencies change.
//!
//! Inspired by:
//! - Flutter's BuildContext with InheritedWidget
//! - React's useContext with dependency arrays
//! - SolidJS createContext with reactive scope

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::context::BuildContext;
use crate::reactive_widgets::WidgetId;

#[cfg(feature = "reactive")]
use vela_reactive::{Signal, Computed};

/// Reactive build context for automatic dependency tracking
#[cfg(feature = "reactive")]
#[derive(Clone)]
pub struct ReactiveBuildContext {
    /// Base build context
    pub base_context: BuildContext,
    /// Currently building widget ID
    pub current_widget_id: WidgetId,
    /// Signals read during current build
    pub signals_read: Arc<Mutex<HashSet<String>>>,
    /// Computed values read during current build
    pub computed_read: Arc<Mutex<HashSet<String>>>,
}

#[cfg(feature = "reactive")]
impl ReactiveBuildContext {
    /// Create new reactive build context
    pub fn new(base_context: BuildContext) -> Self {
        Self {
            base_context,
            current_widget_id: WidgetId::from_str("default"),
            signals_read: Arc::new(Mutex::new(HashSet::new())),
            computed_read: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Create context for specific widget
    pub fn for_widget(base_context: BuildContext, widget_id: WidgetId) -> Self {
        Self {
            base_context,
            current_widget_id: widget_id,
            signals_read: Arc::new(Mutex::new(HashSet::new())),
            computed_read: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Get current widget ID
    pub fn widget_id(&self) -> &WidgetId {
        &self.current_widget_id
    }

    /// Read a signal and track dependency
    pub fn read_signal<T: Clone>(&self, signal: &Signal<T>) -> T {
        // Track that this signal was read during build
        // Use a simple string representation for tracking
        let signal_key = format!("{:p}", signal as *const _);
        if let Ok(mut signals) = self.signals_read.lock() {
            signals.insert(signal_key);
        }
        signal.get().clone()
    }

    /// Read a computed value and track dependency
    pub fn read_computed<T: Clone>(&self, computed: &Computed<T>) -> T {
        // Track that this computed was read during build
        let computed_key = format!("{:p}", computed as *const _);
        if let Ok(mut computed_set) = self.computed_read.lock() {
            computed_set.insert(computed_key);
        }
        computed.get().clone()
    }

    /// Get signals read during this build
    pub fn signals_read(&self) -> HashSet<String> {
        self.signals_read.lock().unwrap().clone()
    }

    /// Get computed values read during this build
    pub fn computed_read(&self) -> HashSet<String> {
        self.computed_read.lock().unwrap().clone()
    }

    /// Clear tracked dependencies
    pub fn clear_dependencies(&self) {
        if let Ok(mut signals) = self.signals_read.lock() {
            signals.clear();
        }
        if let Ok(mut computed_set) = self.computed_read.lock() {
            computed_set.clear();
        }
    }
}

#[cfg(test)]
#[cfg(feature = "reactive")]
mod tests {
    use super::*;
    use crate::context::BuildContext;
    use vela_reactive::Signal;

    #[test]
    fn test_reactive_build_context_creation() {
        let base_ctx = BuildContext::new();
        let ctx = ReactiveBuildContext::new(base_ctx);
        assert!(ctx.signals_read().is_empty());
        assert!(ctx.computed_read().is_empty());
    }

    #[test]
    fn test_signal_reading_tracks_dependency() {
        let base_ctx = BuildContext::new();
        let ctx = ReactiveBuildContext::new(base_ctx);
        let signal = Signal::new(42);

        let value = ctx.read_signal(&signal);
        assert_eq!(value, 42);

        let signals = ctx.signals_read();
        assert_eq!(signals.len(), 1);
        // Check that the signal pointer is tracked
        let signal_key = format!("{:p}", &signal as *const _);
        assert!(signals.contains(&signal_key));
    }

    #[test]
    fn test_clear_dependencies() {
        let base_ctx = BuildContext::new();
        let ctx = ReactiveBuildContext::new(base_ctx);
        let signal = Signal::new(42);

        ctx.read_signal(&signal);
        assert!(!ctx.signals_read().is_empty());

        ctx.clear_dependencies();
        assert!(ctx.signals_read().is_empty());
        assert!(ctx.computed_read().is_empty());
    }
}
