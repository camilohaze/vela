//! # Widget Invalidator - Reactive Rebuild Management
//!
//! Implementation of: VELA-058 - TASK-058
//! Story: Signal Integration
//! Date: 2025-12-03
//!
//! Description:
//! Simple invalidator for tracking which widgets need rebuild
//! when reactive dependencies change.
//!
//! Inspired by:
//! - React's reconciliation algorithm
//! - Flutter's element tree invalidation
//! - Vue's nextTick system

use std::collections::HashSet;

use crate::reactive_widgets::WidgetId;

/// Simple widget invalidator for reactive rebuilds
#[derive(Debug, Clone)]
pub struct WidgetInvalidator {
    /// Widgets that need rebuilding
    pub invalid_widgets: HashSet<WidgetId>,
}

impl WidgetInvalidator {
    /// Create new invalidator
    pub fn new() -> Self {
        Self {
            invalid_widgets: HashSet::new(),
        }
    }

    /// Mark widget as needing rebuild
    pub fn invalidate(&mut self, widget_id: WidgetId) {
        self.invalid_widgets.insert(widget_id);
    }

    /// Mark multiple widgets as needing rebuild
    pub fn invalidate_batch(&mut self, widget_ids: Vec<WidgetId>) {
        for widget_id in widget_ids {
            self.invalid_widgets.insert(widget_id);
        }
    }

    /// Check if widget needs rebuild
    pub fn needs_rebuild(&self, widget_id: &WidgetId) -> bool {
        self.invalid_widgets.contains(widget_id)
    }

    /// Get all widgets that need rebuild
    pub fn widgets_to_rebuild(&self) -> HashSet<WidgetId> {
        self.invalid_widgets.clone()
    }

    /// Clear all invalidations
    pub fn clear(&mut self) {
        self.invalid_widgets.clear();
    }

    /// Remove specific widget from invalidation set
    pub fn clear_widget(&mut self, widget_id: &WidgetId) {
        self.invalid_widgets.remove(widget_id);
    }
}

impl Default for WidgetInvalidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_invalidator_creation() {
        let invalidator = WidgetInvalidator::new();
        assert!(invalidator.invalid_widgets.is_empty());
    }

    #[test]
    fn test_invalidate_single_widget() {
        let mut invalidator = WidgetInvalidator::new();
        let widget_id = WidgetId::from_str("test-widget");

        assert!(!invalidator.needs_rebuild(&widget_id));

        invalidator.invalidate(widget_id.clone());
        assert!(invalidator.needs_rebuild(&widget_id));
    }

    #[test]
    fn test_invalidate_batch() {
        let mut invalidator = WidgetInvalidator::new();
        let widget_ids = vec![
            WidgetId::from_str("widget-1"),
            WidgetId::from_str("widget-2"),
            WidgetId::from_str("widget-3"),
        ];

        invalidator.invalidate_batch(widget_ids.clone());

        for widget_id in &widget_ids {
            assert!(invalidator.needs_rebuild(widget_id));
        }
    }

    #[test]
    fn test_clear_invalidation() {
        let mut invalidator = WidgetInvalidator::new();
        let widget_id = WidgetId::from_str("test-widget");

        invalidator.invalidate(widget_id.clone());
        assert!(invalidator.needs_rebuild(&widget_id));

        invalidator.clear_widget(&widget_id);
        assert!(!invalidator.needs_rebuild(&widget_id));
    }

    #[test]
    fn test_clear_all_invalidations() {
        let mut invalidator = WidgetInvalidator::new();
        let widget_ids = vec![
            WidgetId::from_str("widget-1"),
            WidgetId::from_str("widget-2"),
        ];

        invalidator.invalidate_batch(widget_ids.clone());
        assert_eq!(invalidator.widgets_to_rebuild().len(), 2);

        invalidator.clear();
        assert!(invalidator.widgets_to_rebuild().is_empty());
    }
}
