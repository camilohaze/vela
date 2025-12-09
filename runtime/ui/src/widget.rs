//! Widget trait and base implementations

use crate::vdom::{VDomNode, VDomTree};
use crate::context::BuildContext;
use crate::key::Key;
use crate::lifecycle::{Lifecycle, LifecycleState};
use std::any::Any;

/// Core trait for all widgets in Vela UI
pub trait Widget: std::fmt::Debug {
    /// Build the widget into a VDOM node
    fn build(&self, context: &BuildContext) -> VDomNode;

    /// Optional key for efficient reconciliation
    fn key(&self) -> Option<Key> {
        None
    }

    /// Type name for debugging
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Extension methods for widgets
pub trait WidgetExt: Widget + Sized {
    /// Convert widget to VDOM tree
    fn into_tree(self) -> VDomTree
    where
        Self: 'static,
    {
        VDomTree::new(self)
    }

    /// Build widget with context
    fn build_with_context(self, context: &BuildContext) -> VDomNode
    where
        Self: 'static,
    {
        self.build(context)
    }
}

impl<T: Widget> WidgetExt for T {}

/// Base widget class with lifecycle hooks
#[derive(Debug)]
pub struct BaseWidget {
    pub key: Option<Key>,
    lifecycle_state: LifecycleState,
}

impl BaseWidget {
    /// Create a new base widget
    pub fn new() -> Self {
        Self {
            key: None,
            lifecycle_state: LifecycleState::Unmounted,
        }
    }

    /// Create with key
    pub fn with_key(key: Key) -> Self {
        Self {
            key: Some(key),
            lifecycle_state: LifecycleState::Unmounted,
        }
    }

    /// Get current lifecycle state
    pub fn lifecycle_state(&self) -> LifecycleState {
        self.lifecycle_state.clone()
    }

    /// Protected method for subclasses to override mount behavior
    pub fn on_mount(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Protected method for subclasses to override pre-update behavior
    pub fn on_will_update(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Protected method for subclasses to override post-update behavior
    pub fn on_did_update(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Protected method for subclasses to override pre-unmount behavior
    pub fn on_will_unmount(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Protected method for subclasses to override update decision
    pub fn should_update(&self, _old_widget: &dyn Widget) -> bool {
        true // Default: always update
    }
}

impl Widget for BaseWidget {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        // Default implementation - subclasses should override
        VDomNode::empty()
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

impl Lifecycle for BaseWidget {
    fn mount(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Mounting;
        self.on_mount(context);
        self.lifecycle_state = LifecycleState::Mounted;
    }

    fn will_update(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Updating;
        self.on_will_update(context);
    }

    fn did_update(&mut self, context: &BuildContext) {
        self.on_did_update(context);
        self.lifecycle_state = LifecycleState::Mounted;
    }

    fn will_unmount(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Unmounting;
        self.on_will_unmount(context);
        // Note: State remains Unmounting until unmount is complete
    }

    fn should_update(&self, old_widget: &dyn Widget) -> bool {
        self.should_update(old_widget)
    }
}

/// Stateless widget base class
#[derive(Debug)]
pub struct StatelessWidget {
    pub key: Option<Key>,
}

impl StatelessWidget {
    pub fn new() -> Self {
        Self { key: None }
    }

    pub fn with_key(key: Key) -> Self {
        Self { key: Some(key) }
    }
}

impl Widget for StatelessWidget {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        VDomNode::empty()
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

/// Stateful widget base class
#[derive(Debug)]
pub struct StatefulWidget {
    pub key: Option<Key>,
    // State would be managed through signals
}

impl StatefulWidget {
    pub fn new() -> Self {
        Self { key: None }
    }

    pub fn with_key(key: Key) -> Self {
        Self { key: Some(key) }
    }
}

impl Widget for StatefulWidget {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        VDomNode::empty()
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

/// Container widget for composition
#[derive(Debug)]
pub struct Container {
    pub child: Option<Box<dyn Widget>>,
    pub children: Vec<Box<dyn Widget>>,
    pub key: Option<Key>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            child: None,
            children: Vec::new(),
            key: None,
        }
    }

    pub fn child<W: Widget + 'static>(&mut self, child: W) -> &mut Self {
        self.child = Some(Box::new(child));
        self
    }

    pub fn children(&mut self, children: Vec<Box<dyn Widget>>) -> &mut Self {
        self.children = children;
        self
    }

    pub fn with_key(&mut self, key: Key) -> &mut Self {
        self.key = Some(key);
        self
    }
}

impl Widget for Container {
    fn build(&self, context: &BuildContext) -> VDomNode {
        let mut node = VDomNode::element("div");

        // Add single child if present (replaces any children)
        if let Some(child) = &self.child {
            node.children.push(child.build(context));
        } else {
            // Add children if no single child
            for child in &self.children {
                node.children.push(child.build(context));
            }
        }

        node
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

/// Text widget
#[derive(Debug)]
pub struct Text {
    pub content: String,
    pub key: Option<Key>,
}

impl Text {
    pub fn new<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
            key: None,
        }
    }

    pub fn with_key(mut self, key: Key) -> Self {
        self.key = Some(key);
        self
    }
}

impl Widget for Text {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        VDomNode::text(&self.content)
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_widget() {
        let text = Text::new("Hello Vela");
        let context = BuildContext::new();
        let node = text.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Text);
        assert_eq!(node.text_content, Some("Hello Vela".to_string()));
    }

    #[test]
    fn test_container_widget() {
        let mut container = Container::new();
        container.child(Text::new("Hello"));
        container.child(Text::new("World"));

        let context = BuildContext::new();
        let node = container.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.children.len(), 1); // Only the last child is kept
    }

    #[test]
    fn test_widget_keys() {
        let key = Key::String("test-key".to_string());
        let text = Text::new("Test").with_key(key.clone());

        assert_eq!(text.key(), Some(key));
    }

    #[test]
    fn test_base_widget_creation() {
        let widget = BaseWidget::new();
        assert_eq!(widget.lifecycle_state(), LifecycleState::Unmounted);
        assert!(widget.key().is_none());
    }

    #[test]
    fn test_base_widget_with_key() {
        let key = Key::String("test-key".to_string());
        let widget = BaseWidget::with_key(key.clone());
        assert_eq!(widget.key(), Some(key));
        assert_eq!(widget.lifecycle_state(), LifecycleState::Unmounted);
    }

    #[test]
    fn test_base_widget_build_default() {
        let widget = BaseWidget::new();
        let context = BuildContext::new();
        let node = widget.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Empty);
    }

    #[derive(Debug)]
    struct TestLifecycleWidget {
        base: BaseWidget,
        mounted: bool,
        updated: bool,
        unmounted: bool,
    }

    impl TestLifecycleWidget {
        fn new() -> Self {
            Self {
                base: BaseWidget::new(),
                mounted: false,
                updated: false,
                unmounted: false,
            }
        }
    }

    impl Widget for TestLifecycleWidget {
        fn build(&self, _context: &BuildContext) -> VDomNode {
            VDomNode::text("Test Widget")
        }

        fn key(&self) -> Option<Key> {
            self.base.key()
        }
    }

    impl Lifecycle for TestLifecycleWidget {
        fn mount(&mut self, context: &BuildContext) {
            self.base.mount(context);
            self.mounted = true;
        }

        fn will_update(&mut self, context: &BuildContext) {
            self.base.will_update(context);
        }

        fn did_update(&mut self, context: &BuildContext) {
            self.base.did_update(context);
            self.updated = true;
        }

        fn will_unmount(&mut self, context: &BuildContext) {
            self.base.will_unmount(context);
            self.unmounted = true;
        }
    }

    #[test]
    fn test_base_widget_lifecycle_hooks() {
        let mut widget = TestLifecycleWidget::new();
        let context = BuildContext::new();

        // Test mount
        widget.mount(&context);
        assert!(widget.mounted);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Mounted);

        // Test will_update
        widget.will_update(&context);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Updating);

        // Test did_update
        widget.did_update(&context);
        assert!(widget.updated);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Mounted);

        // Test will_unmount
        widget.will_unmount(&context);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Unmounting);
        assert!(widget.unmounted);
    }

    #[test]
    fn test_base_widget_should_update() {
        let widget = BaseWidget::new();
        let old_widget = BaseWidget::new();

        // Default implementation always returns true
        assert!(widget.should_update(&old_widget));
    }

    #[test]
    fn test_base_widget_lifecycle_manager_integration() {
        use crate::lifecycle::LifecycleManager;

        let mut manager = LifecycleManager::new();
        let mut widget = TestLifecycleWidget::new();
        let context = BuildContext::new();

        // Test mounting through lifecycle manager
        manager.transition(
            "test-widget".to_string(),
            &mut widget,
            LifecycleState::Mounting,
            &context
        ).unwrap();

        assert!(widget.mounted);
        assert_eq!(manager.get_state("test-widget"), LifecycleState::Mounted);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Mounted);

        // Test updating through lifecycle manager
        manager.transition(
            "test-widget".to_string(),
            &mut widget,
            LifecycleState::Updating,
            &context
        ).unwrap();

        assert!(widget.updated);
        assert_eq!(manager.get_state("test-widget"), LifecycleState::Mounted);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Mounted);
    }
}