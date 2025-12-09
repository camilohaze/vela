//! Virtual DOM implementation

use crate::widget::Widget;
use crate::context::BuildContext;
use crate::key::Key;
use std::collections::HashMap;

/// Virtual DOM tree
#[derive(Debug, Clone)]
pub struct VDomTree {
    pub root: VDomNode,
    pub needs_update: bool,
}

impl VDomTree {
    /// Create a new VDOM tree from a root widget
    pub fn new<W: Widget + 'static>(root_widget: W) -> Self {
        let context = BuildContext::new();
        let root = root_widget.build(&context);

        Self {
            root,
            needs_update: false,
        }
    }

    /// Check if the tree needs updating
    pub fn needs_update(&self) -> bool {
        self.needs_update
    }

    /// Mark tree as needing update
    pub fn mark_for_update(&mut self) {
        self.needs_update = true;
    }

    /// Rebuild the tree (in real implementation this would be reactive)
    pub fn rebuild(&self) -> Result<VDomTree, Box<dyn std::error::Error>> {
        // For now, return a copy. In real implementation this would rebuild
        // based on reactive signals
        Ok(self.clone())
    }
}

/// Virtual DOM node
#[derive(Debug, Clone)]
pub struct VDomNode {
    /// Type of node
    pub node_type: NodeType,
    /// Tag name for elements
    pub tag_name: Option<String>,
    /// Text content for text nodes
    pub text_content: Option<String>,
    /// Attributes for elements
    pub attributes: HashMap<String, String>,
    /// Properties for elements
    pub properties: HashMap<String, serde_json::Value>,
    /// Event listeners
    pub event_listeners: HashMap<String, String>, // In real impl, this would be function pointers
    /// Child nodes
    pub children: Vec<VDomNode>,
    /// Widget key for reconciliation
    pub key: Option<Key>,
}

impl VDomNode {
    /// Create an empty node
    pub fn empty() -> Self {
        Self {
            node_type: NodeType::Empty,
            tag_name: None,
            text_content: None,
            attributes: HashMap::new(),
            properties: HashMap::new(),
            event_listeners: HashMap::new(),
            children: Vec::new(),
            key: None,
        }
    }

    /// Create an element node
    pub fn element<S: Into<String>>(tag_name: S) -> Self {
        Self {
            node_type: NodeType::Element,
            tag_name: Some(tag_name.into()),
            text_content: None,
            attributes: HashMap::new(),
            properties: HashMap::new(),
            event_listeners: HashMap::new(),
            children: Vec::new(),
            key: None,
        }
    }

    /// Create a text node
    pub fn text<S: Into<String>>(content: S) -> Self {
        Self {
            node_type: NodeType::Text,
            tag_name: None,
            text_content: Some(content.into()),
            attributes: HashMap::new(),
            properties: HashMap::new(),
            event_listeners: HashMap::new(),
            children: Vec::new(),
            key: None,
        }
    }

    /// Add an attribute
    pub fn attr<S: Into<String>>(mut self, name: S, value: S) -> Self {
        self.attributes.insert(name.into(), value.into());
        self
    }

    /// Add a property
    pub fn prop<S: Into<String>>(mut self, name: S, value: serde_json::Value) -> Self {
        self.properties.insert(name.into(), value);
        self
    }

    /// Add an event listener
    pub fn on<S: Into<String>>(mut self, event: S, handler: S) -> Self {
        self.event_listeners.insert(event.into(), handler.into());
        self
    }

    /// Add a child node
    pub fn child(mut self, child: VDomNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: Vec<VDomNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set key
    pub fn key(mut self, key: &Key) -> Self {
        self.key = Some(key.clone());
        self
    }

    /// Check if node is empty
    pub fn is_empty(&self) -> bool {
        matches!(self.node_type, NodeType::Empty)
    }

    /// Get all descendant keys
    pub fn collect_keys(&self) -> Vec<Key> {
        let mut keys = Vec::new();

        if let Some(key) = &self.key {
            keys.push(key.clone());
        }

        for child in &self.children {
            keys.extend(child.collect_keys());
        }

        keys
    }
}

/// Type of virtual DOM node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Empty node (no-op)
    Empty,
    /// HTML element
    Element,
    /// Text node
    Text,
    /// Component node (for custom widgets)
    Component,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vdom_element() {
        let element = VDomNode::element("div")
            .attr("class", "container")
            .attr("id", "main");

        assert_eq!(element.node_type, NodeType::Element);
        assert_eq!(element.tag_name, Some("div".to_string()));
        assert_eq!(element.attributes.get("class"), Some(&"container".to_string()));
        assert_eq!(element.attributes.get("id"), Some(&"main".to_string()));
    }

    #[test]
    fn test_vdom_text() {
        let text = VDomNode::text("Hello World");

        assert_eq!(text.node_type, NodeType::Text);
        assert_eq!(text.text_content, Some("Hello World".to_string()));
    }

    #[test]
    fn test_vdom_with_children() {
        let child1 = VDomNode::text("Child 1");
        let child2 = VDomNode::text("Child 2");

        let parent = VDomNode::element("div")
            .child(child1)
            .child(child2);

        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0].text_content, Some("Child 1".to_string()));
        assert_eq!(parent.children[1].text_content, Some("Child 2".to_string()));
    }

    #[test]
    fn test_vdom_with_key() {
        let key = crate::key::Key::string("test-key");
        let node = VDomNode::element("div").key(&key);

        assert_eq!(node.key, Some(key));
    }

    #[test]
    fn test_collect_keys() {
        let key1 = crate::key::Key::string("key1");
        let key2 = crate::key::Key::string("key2");

        let child = VDomNode::element("span").key(&key2);
        let parent = VDomNode::element("div")
            .key(&key1)
            .child(child);

        let keys = parent.collect_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&key1));
        assert!(keys.contains(&key2));
    }

    #[test]
    fn test_vdom_tree() {
        let tree = VDomTree::new(crate::widget::TestText::new("Root"));

        assert!(!tree.needs_update());
        assert_eq!(tree.root.node_type, NodeType::Text);
        assert_eq!(tree.root.text_content, Some("Root".to_string()));
    }
}