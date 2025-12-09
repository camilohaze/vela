//! # Reactive Widgets - Signal Integration for UI
//!
//! Implementation of: VELA-058 - TASK-058
//! Story: Signal Integration
//! Date: 2025-12-03
//!
//! Description:
//! Integrates reactive signals with the widget system to enable
//! automatic UI updates when signal values change.
//!
//! Inspired by:
//! - Flutter's StatefulWidget with reactive state
//! - React hooks with dependency tracking
//! - SolidJS components with signals

use uuid::Uuid;

use crate::vdom::VDomNode;
use crate::widget::Widget;
use crate::widget_invalidator::WidgetInvalidator;

/// Unique identifier for widgets in the reactive system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WidgetId(pub String);

impl WidgetId {
    /// Create a new unique widget ID
    pub fn new() -> Self {
        Self(format!("widget-{}", Uuid::new_v4()))
    }

    /// Create widget ID from string
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for widgets that can react to signal changes
#[cfg(feature = "reactive")]
pub trait ReactiveWidget: Widget {
    /// Build the widget reactively with automatic dependency tracking
    fn build_reactive(&self, ctx: &mut crate::reactive_context::ReactiveBuildContext) -> VDomNode;

    /// Get the widget's unique ID for dependency tracking
    fn widget_id(&self) -> WidgetId {
        WidgetId::new()
    }
}

#[cfg(test)]
#[cfg(feature = "reactive")]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id_creation() {
        let id1 = WidgetId::new();
        let id2 = WidgetId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_widget_id_from_str() {
        let id = WidgetId::from_str("test");
        assert_eq!(id.0, "test");
    }

    #[test]
    fn test_widget_invalidator() {
        let mut invalidator = WidgetInvalidator::new();
        let widget_id = WidgetId::from_str("test-widget");

        assert!(!invalidator.needs_rebuild(&widget_id));

        invalidator.invalidate(widget_id.clone());
        assert!(invalidator.needs_rebuild(&widget_id));

        invalidator.clear();
        assert!(!invalidator.needs_rebuild(&widget_id));
    }
}