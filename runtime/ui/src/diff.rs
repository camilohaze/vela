//! Virtual DOM diffing algorithm

use crate::vdom::{VDomNode, VDomTree, VDomPath, NodeType};
use crate::key::Key;
use std::collections::HashMap;

/// Result of diffing two VDOM trees
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub patches: Vec<Patch>,
}

/// Types of patches that can be applied to the DOM
#[derive(Debug, Clone)]
pub enum Patch {
    /// Insert a new node
    Insert {
        parent_path: VDomPath,
        index: usize,
        node: VDomNode,
    },
    /// Remove a node
    Remove {
        path: VDomPath,
    },
    /// Replace a node
    Replace {
        path: VDomPath,
        new_node: VDomNode,
    },
    /// Update text content
    UpdateText {
        path: VDomPath,
        new_text: String,
    },
    /// Update attributes
    UpdateAttributes {
        path: VDomPath,
        attributes: HashMap<String, Option<String>>, // None = remove, Some = set
    },
    /// Update properties
    UpdateProperties {
        path: VDomPath,
        properties: HashMap<String, Option<serde_json::Value>>,
    },
    /// Update event listeners
    UpdateEvents {
        path: VDomPath,
        events: HashMap<String, Option<String>>,
    },
}

/// Diff two VDOM trees
pub fn diff_trees(old_tree: &VDomTree, new_tree: &VDomTree) -> Vec<Patch> {
    let mut patches = Vec::new();
    diff_nodes(&old_tree.root, &new_tree.root, VDomPath::root(), &mut patches);
    patches
}

/// Diff two VDOM nodes recursively
fn diff_nodes(old_node: &VDomNode, new_node: &VDomNode, path: VDomPath, patches: &mut Vec<Patch>) {
    // For Fragment nodes, always diff children since they have no content of their own
    if old_node.node_type == NodeType::Fragment && new_node.node_type == NodeType::Fragment {
        diff_children(old_node, new_node, path, patches);
        return;
    }

    // If nodes have different types or keys, replace entirely
    if old_node.node_type != new_node.node_type ||
       old_node.key != new_node.key ||
       (old_node.tag_name != new_node.tag_name && old_node.node_type == NodeType::Element) {
        patches.push(Patch::Replace {
            path: path.clone(),
            new_node: new_node.clone(),
        });
        return;
    }

    // Handle different node types
    match (&old_node.node_type, &new_node.node_type) {
        (NodeType::Text, NodeType::Text) => {
            if old_node.text_content != new_node.text_content {
                if let Some(new_text) = &new_node.text_content {
                    patches.push(Patch::UpdateText {
                        path: path.clone(),
                        new_text: new_text.clone(),
                    });
                }
            }
        }
        (NodeType::Element, NodeType::Element) => {
            // Diff attributes
            let attr_patches = diff_attributes(&old_node.attributes, &new_node.attributes);
            if !attr_patches.is_empty() {
                patches.push(Patch::UpdateAttributes {
                    path: path.clone(),
                    attributes: attr_patches,
                });
            }

            // Diff properties
            let prop_patches = diff_properties(&old_node.properties, &new_node.properties);
            if !prop_patches.is_empty() {
                patches.push(Patch::UpdateProperties {
                    path: path.clone(),
                    properties: prop_patches,
                });
            }

            // Diff event listeners
            let event_patches = diff_events(&old_node.event_listeners, &new_node.event_listeners);
            if !event_patches.is_empty() {
                patches.push(Patch::UpdateEvents {
                    path: path.clone(),
                    events: event_patches,
                });
            }

            // Diff children using key-based reconciliation
            diff_children(old_node, new_node, path.clone(), patches);
        }
        _ => {
            // For other cases, replace the node
            patches.push(Patch::Replace {
                path: path.clone(),
                new_node: new_node.clone(),
            });
        }
    }
}

/// Diff attributes between two nodes
fn diff_attributes(old_attrs: &HashMap<String, String>, new_attrs: &HashMap<String, String>) -> HashMap<String, Option<String>> {
    let mut patches = HashMap::new();

    // Find removed attributes
    for (key, _) in old_attrs {
        if !new_attrs.contains_key(key) {
            patches.insert(key.clone(), None);
        }
    }

    // Find added/changed attributes
    for (key, new_value) in new_attrs {
        match old_attrs.get(key) {
            Some(old_value) if old_value != new_value => {
                patches.insert(key.clone(), Some(new_value.clone()));
            }
            None => {
                patches.insert(key.clone(), Some(new_value.clone()));
            }
            _ => {} // Unchanged
        }
    }

    patches
}

/// Diff properties between two nodes
fn diff_properties(old_props: &HashMap<String, serde_json::Value>, new_props: &HashMap<String, serde_json::Value>) -> HashMap<String, Option<serde_json::Value>> {
    let mut patches = HashMap::new();

    // Find removed properties
    for (key, _) in old_props {
        if !new_props.contains_key(key) {
            patches.insert(key.clone(), None);
        }
    }

    // Find added/changed properties
    for (key, new_value) in new_props {
        match old_props.get(key) {
            Some(old_value) if old_value != new_value => {
                patches.insert(key.clone(), Some(new_value.clone()));
            }
            None => {
                patches.insert(key.clone(), Some(new_value.clone()));
            }
            _ => {} // Unchanged
        }
    }

    patches
}

/// Diff event listeners between two nodes
fn diff_events(old_events: &HashMap<String, String>, new_events: &HashMap<String, String>) -> HashMap<String, Option<String>> {
    let mut patches = HashMap::new();

    // Find removed events
    for (key, _) in old_events {
        if !new_events.contains_key(key) {
            patches.insert(key.clone(), None);
        }
    }

    // Find added/changed events
    for (key, new_handler) in new_events {
        match old_events.get(key) {
            Some(old_handler) if old_handler != new_handler => {
                patches.insert(key.clone(), Some(new_handler.clone()));
            }
            None => {
                patches.insert(key.clone(), Some(new_handler.clone()));
            }
            _ => {} // Unchanged
        }
    }

    patches
}

/// Diff children using key-based reconciliation
fn diff_children(old_node: &VDomNode, new_node: &VDomNode, path: VDomPath, patches: &mut Vec<Patch>) {
    let old_children = &old_node.children;
    let new_children = &new_node.children;

    // Create key-to-index maps for efficient lookup
    let old_key_map = create_key_map(old_children);
    let new_key_map = create_key_map(new_children);

    let mut old_index = 0;
    let mut new_index = 0;
    let mut result_children = Vec::new();

    // Process new children
    while new_index < new_children.len() {
        let new_child = &new_children[new_index];

        if let Some(new_key) = &new_child.key {
            // Keyed child - try to find matching old child
            if let Some(&old_idx) = old_key_map.get(new_key) {
                // Found matching key
                let old_child = &old_children[old_idx];

                // Move old child to new position if needed
                if old_idx != result_children.len() {
                    // This would be a move operation in a full implementation
                    // For now, we'll just diff in place
                }

                // Diff the matched children
                let child_path = path.child(result_children.len());
                diff_nodes(old_child, new_child, child_path, patches);

                result_children.push(old_idx);
            } else {
                // New keyed child - insert
                patches.push(Patch::Insert {
                    parent_path: path.clone(),
                    index: result_children.len(),
                    node: new_child.clone(),
                });
                result_children.push(usize::MAX); // Placeholder
            }
        } else {
            // Non-keyed child - diff with corresponding old child if available
            if old_index < old_children.len() {
                let old_child = &old_children[old_index];

                // Only diff if old child is also non-keyed
                if old_child.key.is_none() {
                    let child_path = path.child(result_children.len());
                    diff_nodes(old_child, new_child, child_path, patches);
                    result_children.push(old_index);
                    old_index += 1;
                } else {
                    // Old child is keyed, new is not - insert new
                    patches.push(Patch::Insert {
                        parent_path: path.clone(),
                        index: result_children.len(),
                        node: new_child.clone(),
                    });
                    result_children.push(usize::MAX);
                }
            } else {
                // No corresponding old child - insert
                patches.push(Patch::Insert {
                    parent_path: path.clone(),
                    index: result_children.len(),
                    node: new_child.clone(),
                });
                result_children.push(usize::MAX);
            }
        }

        new_index += 1;
    }

    // Remove remaining old children
    while old_index < old_children.len() {
        let old_child = &old_children[old_index];
        if old_child.key.is_none() || !new_key_map.contains_key(&old_child.key.as_ref().unwrap()) {
            // Find the index in result_children
            if let Some(pos) = result_children.iter().position(|&idx| idx == old_index) {
                let remove_path = path.child(pos);
                patches.push(Patch::Remove { path: remove_path });
            }
        }
        old_index += 1;
    }
}

/// Create a map from keys to indices for efficient lookup
fn create_key_map(children: &[VDomNode]) -> HashMap<&Key, usize> {
    let mut map = HashMap::new();
    for (index, child) in children.iter().enumerate() {
        if let Some(key) = &child.key {
            map.insert(key, index);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vdom::VDomTree;

    #[test]
    fn test_diff_identical_nodes() {
        let node1 = VDomNode::text("Hello");
        let node2 = VDomNode::text("Hello");

        let mut patches = Vec::new();
        diff_nodes(&node1, &node2, VDomPath::root(), &mut patches);

        assert!(patches.is_empty());
    }

    #[test]
    fn test_diff_text_content() {
        let old_node = VDomNode::text("Hello");
        let new_node = VDomNode::text("World");

        let mut patches = Vec::new();
        diff_nodes(&old_node, &new_node, VDomPath::root(), &mut patches);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::UpdateText { new_text, .. } => assert_eq!(new_text, "World"),
            _ => panic!("Expected UpdateText patch"),
        }
    }

    #[test]
    fn test_diff_attributes() {
        let old_node = VDomNode::element("div").attr("class", "old");
        let new_node = VDomNode::element("div").attr("class", "new").attr("id", "test");

        let patches = diff_attributes(&old_node.attributes, &new_node.attributes);

        assert_eq!(patches.len(), 2);
        assert_eq!(patches["class"], Some("new".to_string()));
        assert_eq!(patches["id"], Some("test".to_string()));
    }

    #[test]
    fn test_diff_fragment_support() {
        let old_node = VDomNode::fragment()
            .child(VDomNode::text("old"));
        let new_node = VDomNode::fragment()
            .child(VDomNode::text("new"));

        let old_tree = VDomTree::new_from_node(old_node);
        let new_tree = VDomTree::new_from_node(new_node);

        let diff = diff_trees(&old_tree, &new_tree);
        assert!(!diff.is_empty());
        assert_eq!(diff.len(), 1);

        match &diff[0] {
            Patch::UpdateText { path, new_text } => {
                assert_eq!(path.0, vec![0]);
                assert_eq!(new_text, "new");
            }
            _ => panic!("Expected UpdateText patch"),
        }
    }

    #[test]
    fn test_diff_with_vdom_path() {
        let old_node = VDomNode::element("div")
            .child(VDomNode::text("old"));
        let new_node = VDomNode::element("div")
            .child(VDomNode::text("new"));

        let old_tree = VDomTree::new_from_node(old_node);
        let new_tree = VDomTree::new_from_node(new_node);

        let diff = diff_trees(&old_tree, &new_tree);
        assert!(!diff.is_empty());
        assert_eq!(diff.len(), 1);

        // Verify the patch uses VDomPath
        match &diff[0] {
            Patch::UpdateText { path, new_text } => {
                assert_eq!(path.0, vec![0]);
                assert_eq!(new_text, "new");
            }
            _ => panic!("Expected UpdateText patch"),
        }
    }
}