//! Widget lifecycle hooks

use crate::widget::Widget;
use crate::context::BuildContext;

/// Lifecycle hooks for widgets
pub trait Lifecycle: Widget {
    /// Called when widget is first mounted to the DOM
    fn mount(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Called when widget is about to update
    fn will_update(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Called after widget has updated
    fn did_update(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Called when widget is about to be unmounted from DOM
    fn will_unmount(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Called when widget should update (reactive trigger)
    fn should_update(&self, _old_widget: &dyn Widget) -> bool {
        true // Default: always update
    }
}

/// Lifecycle state management
#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleState {
    /// Widget not yet mounted
    Unmounted,
    /// Widget is mounting
    Mounting,
    /// Widget is mounted and active
    Mounted,
    /// Widget is updating
    Updating,
    /// Widget is unmounting
    Unmounting,
}

impl Default for LifecycleState {
    fn default() -> Self {
        LifecycleState::Unmounted
    }
}

/// Lifecycle manager for coordinating widget lifecycles
#[derive(Debug)]
pub struct LifecycleManager {
    states: std::collections::HashMap<String, LifecycleState>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            states: std::collections::HashMap::new(),
        }
    }

    /// Get lifecycle state for a widget
    pub fn get_state(&self, widget_id: &str) -> LifecycleState {
        self.states
            .get(widget_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Set lifecycle state for a widget
    pub fn set_state(&mut self, widget_id: String, state: LifecycleState) {
        self.states.insert(widget_id, state);
    }

    /// Transition widget through lifecycle
    pub fn transition<W: Lifecycle + ?Sized>(
        &mut self,
        widget_id: String,
        widget: &mut W,
        new_state: LifecycleState,
        context: &BuildContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let current_state = self.get_state(&widget_id);

        match (&current_state, &new_state) {
            (LifecycleState::Unmounted, LifecycleState::Mounting) => {
                widget.mount(context);
                self.set_state(widget_id, LifecycleState::Mounted);
            }
            (LifecycleState::Mounted, LifecycleState::Updating) => {
                widget.will_update(context);
                // Update happens here
                widget.did_update(context);
                self.set_state(widget_id, LifecycleState::Mounted);
            }
            (LifecycleState::Mounted, LifecycleState::Unmounting) => {
                widget.will_unmount(context);
                self.set_state(widget_id, LifecycleState::Unmounted);
            }
            _ => {
                return Err(format!(
                    "Invalid lifecycle transition from {:?} to {:?}",
                    current_state, new_state
                ).into());
            }
        }

        Ok(())
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for widgets that need lifecycle management
pub trait StatefulLifecycle: Lifecycle {
    /// Get unique widget ID
    fn widget_id(&self) -> String;

    /// Called when widget state changes
    fn on_state_change(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Called when widget receives new props
    fn on_props_change(&mut self, _old_props: &dyn std::any::Any, _context: &BuildContext) {
        // Default: no-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{Widget, StatelessWidget};
    use crate::vdom::VDomNode;

    #[derive(Debug)]
    struct TestLifecycleWidget {
        mounted: bool,
        updated: bool,
        unmounted: bool,
    }

    impl TestLifecycleWidget {
        fn new() -> Self {
            Self {
                mounted: false,
                updated: false,
                unmounted: false,
            }
        }
    }

    impl Widget for TestLifecycleWidget {
        fn build(&self, _context: &BuildContext) -> VDomNode {
            VDomNode::text("Test")
        }
    }

    impl Lifecycle for TestLifecycleWidget {
        fn mount(&mut self, _context: &BuildContext) {
            self.mounted = true;
        }

        fn did_update(&mut self, _context: &BuildContext) {
            self.updated = true;
        }

        fn will_unmount(&mut self, _context: &BuildContext) {
            self.unmounted = true;
        }
    }

    impl StatefulLifecycle for TestLifecycleWidget {
        fn widget_id(&self) -> String {
            "test-widget".to_string()
        }
    }

    #[test]
    fn test_lifecycle_manager() {
        let mut manager = LifecycleManager::new();
        let mut widget = TestLifecycleWidget::new();
        let context = BuildContext::new();

        // Test mounting
        assert_eq!(manager.get_state("test-widget"), LifecycleState::Unmounted);

        manager.transition("test-widget".to_string(), &mut widget, LifecycleState::Mounting, &context).unwrap();
        assert_eq!(manager.get_state("test-widget"), LifecycleState::Mounted);
        assert!(widget.mounted);

        // Test updating
        manager.transition("test-widget".to_string(), &mut widget, LifecycleState::Updating, &context).unwrap();
        assert_eq!(manager.get_state("test-widget"), LifecycleState::Mounted);
        assert!(widget.updated);

        // Test unmounting
        manager.transition("test-widget".to_string(), &mut widget, LifecycleState::Unmounting, &context).unwrap();
        assert_eq!(manager.get_state("test-widget"), LifecycleState::Unmounted);
        assert!(widget.unmounted);
    }

    #[test]
    fn test_invalid_transition() {
        let mut manager = LifecycleManager::new();
        let mut widget = TestLifecycleWidget::new();
        let context = BuildContext::new();

        // Try to update unmounted widget
        let result = manager.transition("test-widget".to_string(), &mut widget, LifecycleState::Updating, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_lifecycle_states() {
        let state = LifecycleState::Mounted;
        assert!(matches!(state, LifecycleState::Mounted));

        let default_state = LifecycleState::default();
        assert!(matches!(default_state, LifecycleState::Unmounted));
    }
}