//! Widget trait and base implementations

use crate::vdom::{VDomNode, VDomTree};
use crate::context::BuildContext;
use crate::key::Key;
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

    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    pub fn children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    pub fn with_key(mut self, key: Key) -> Self {
        self.key = Some(key);
        self
    }
}

impl Widget for Container {
    fn build(&self, context: &BuildContext) -> VDomNode {
        let mut node = VDomNode::element("div");

        // Add child if present
        if let Some(child) = &self.child {
            node.children.push(child.build(context));
        }

        // Add children
        for child in &self.children {
            node.children.push(child.build(context));
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
        let container = Container::new()
            .child(Text::new("Hello"))
            .child(Text::new("World"));

        let context = BuildContext::new();
        let node = container.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn test_widget_keys() {
        let key = Key::String("test-key".to_string());
        let text = Text::new("Test").with_key(key.clone());

        assert_eq!(text.key(), Some(key));
    }
}