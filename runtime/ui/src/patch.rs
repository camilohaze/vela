//! DOM patching system

use crate::vdom::VDomNode;
use crate::diff::Patch;
use std::collections::HashMap;

/// Apply patches to the DOM
pub fn apply_patches(patches: Vec<Patch>) -> Result<(), Box<dyn std::error::Error>> {
    // In a real implementation, this would interact with the actual DOM
    // For now, we'll just validate the patches

    for patch in patches {
        match patch {
            Patch::Insert { .. } => {
                // Insert node into DOM
            }
            Patch::Remove { .. } => {
                // Remove node from DOM
            }
            Patch::Replace { .. } => {
                // Replace node in DOM
            }
            Patch::UpdateText { .. } => {
                // Update text content
            }
            Patch::UpdateAttributes { .. } => {
                // Update element attributes
            }
            Patch::UpdateProperties { .. } => {
                // Update element properties
            }
            Patch::UpdateEvents { .. } => {
                // Update event listeners
            }
        }
    }

    Ok(())
}

/// DOM node reference (in real implementation this would be web-sys Element)
#[derive(Debug)]
pub struct DomNode {
    pub node_type: String,
    pub attributes: HashMap<String, String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub event_listeners: HashMap<String, String>,
    pub children: Vec<DomNode>,
    pub text_content: Option<String>,
}

impl DomNode {
    /// Create from VDOM node
    pub fn from_vdom(vdom_node: &VDomNode) -> Self {
        Self {
            node_type: format!("{:?}", vdom_node.node_type),
            attributes: vdom_node.attributes.clone(),
            properties: vdom_node.properties.clone(),
            event_listeners: vdom_node.event_listeners.clone(),
            children: vdom_node.children.iter().map(Self::from_vdom).collect(),
            text_content: vdom_node.text_content.clone(),
        }
    }

    /// Apply a patch to this DOM node
    pub fn apply_patch(&mut self, patch: &Patch, path: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
        if path.is_empty() {
            // Apply patch to this node
            match patch {
                Patch::UpdateText { new_text, .. } => {
                    self.text_content = Some(new_text.clone());
                }
                Patch::UpdateAttributes { attributes, .. } => {
                    for (key, value) in attributes {
                        match value {
                            Some(val) => {
                                self.attributes.insert(key.clone(), val.clone());
                            }
                            None => {
                                self.attributes.remove(key);
                            }
                        }
                    }
                }
                Patch::UpdateProperties { properties, .. } => {
                    for (key, value) in properties {
                        match value {
                            Some(val) => {
                                self.properties.insert(key.clone(), val.clone());
                            }
                            None => {
                                self.properties.remove(key);
                            }
                        }
                    }
                }
                Patch::UpdateEvents { events, .. } => {
                    for (key, value) in events {
                        match value {
                            Some(val) => {
                                self.event_listeners.insert(key.clone(), val.clone());
                            }
                            None => {
                                self.event_listeners.remove(key);
                            }
                        }
                    }
                }
                _ => {
                    return Err(format!("Cannot apply patch {:?} to root node", patch).into());
                }
            }
        } else {
            // Navigate to child
            let child_index = path[0];
            if child_index >= self.children.len() {
                return Err(format!("Child index {} out of bounds", child_index).into());
            }

            self.children[child_index].apply_patch(patch, &path[1..])?;
        }

        Ok(())
    }

    /// Insert child at index
    pub fn insert_child(&mut self, index: usize, child: DomNode) -> Result<(), Box<dyn std::error::Error>> {
        if index > self.children.len() {
            return Err(format!("Insert index {} out of bounds", index).into());
        }

        self.children.insert(index, child);
        Ok(())
    }

    /// Remove child at index
    pub fn remove_child(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index >= self.children.len() {
            return Err(format!("Remove index {} out of bounds", index).into());
        }

        self.children.remove(index);
        Ok(())
    }

    /// Replace child at index
    pub fn replace_child(&mut self, index: usize, new_child: DomNode) -> Result<(), Box<dyn std::error::Error>> {
        if index >= self.children.len() {
            return Err(format!("Replace index {} out of bounds", index).into());
        }

        self.children[index] = new_child;
        Ok(())
    }
}

/// DOM tree representation
#[derive(Debug)]
pub struct DomTree {
    pub root: DomNode,
}

impl DomTree {
    /// Create from VDOM tree
    pub fn from_vdom(vdom_tree: &crate::vdom::VDomTree) -> Self {
        Self {
            root: DomNode::from_vdom(&vdom_tree.root),
        }
    }

    /// Apply patches to the DOM tree
    pub fn apply_patches(&mut self, patches: &[Patch]) -> Result<(), Box<dyn std::error::Error>> {
        for patch in patches {
            match patch {
                Patch::Insert { parent_path, index, node } => {
                    let dom_node = DomNode::from_vdom(node);
                    self.apply_patch_to_path(parent_path, |parent| {
                        parent.insert_child(*index, dom_node)
                    })?;
                }
                Patch::Remove { path } => {
                    if path.len() >= 2 {
                        let parent_path = &path[..path.len() - 1];
                        let child_index = *path.last().unwrap();
                        self.apply_patch_to_path(parent_path, |parent| {
                            parent.remove_child(child_index)
                        })?;
                    }
                }
                Patch::Replace { path, new_node } => {
                    if path.len() >= 2 {
                        let parent_path = &path[..path.len() - 1];
                        let child_index = *path.last().unwrap();
                        let dom_node = DomNode::from_vdom(new_node);
                        self.apply_patch_to_path(parent_path, |parent| {
                            parent.replace_child(child_index, dom_node)
                        })?;
                    }
                }
                Patch::UpdateText { path, new_text } => {
                    self.root.apply_patch(patch, path)?;
                }
                Patch::UpdateAttributes { path, .. } => {
                    self.root.apply_patch(patch, path)?;
                }
                Patch::UpdateProperties { path, .. } => {
                    self.root.apply_patch(patch, path)?;
                }
                Patch::UpdateEvents { path, .. } => {
                    self.root.apply_patch(patch, path)?;
                }
            }
        }

        Ok(())
    }

    /// Apply a patch function to a node at the given path
    fn apply_patch_to_path<F>(&mut self, path: &[usize], f: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut DomNode) -> Result<(), Box<dyn std::error::Error>>,
    {
        let mut current = &mut self.root;

        for &index in path {
            if index >= current.children.len() {
                return Err(format!("Path index {} out of bounds", index).into());
            }
            current = &mut current.children[index];
        }

        f(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vdom::VDomNode;
    use crate::diff::Patch;

    #[test]
    fn test_dom_node_from_vdom() {
        let vdom = VDomNode::element("div").attr("class", "test");
        let dom = DomNode::from_vdom(&vdom);

        assert_eq!(dom.node_type, "Element");
        assert_eq!(dom.attributes.get("class"), Some(&"test".to_string()));
    }

    #[test]
    fn test_dom_tree_from_vdom() {
        let vdom_tree = crate::vdom::VDomTree::new(crate::widget::Text::new("Test"));
        let dom_tree = DomTree::from_vdom(&vdom_tree);

        assert_eq!(dom_tree.root.node_type, "Text");
        assert_eq!(dom_tree.root.text_content, Some("Test".to_string()));
    }

    #[test]
    fn test_apply_text_patch() {
        let mut dom = DomNode::from_vdom(&VDomNode::text("Old"));
        let patch = Patch::UpdateText {
            path: vec![],
            new_text: "New".to_string(),
        };

        dom.apply_patch(&patch, &[]).unwrap();

        assert_eq!(dom.text_content, Some("New".to_string()));
    }

    #[test]
    fn test_apply_attribute_patch() {
        let mut dom = DomNode::from_vdom(&VDomNode::element("div"));
        let patch = Patch::UpdateAttributes {
            path: vec![],
            attributes: [("class".to_string(), Some("test".to_string()))].into(),
        };

        dom.apply_patch(&patch, &[]).unwrap();

        assert_eq!(dom.attributes.get("class"), Some(&"test".to_string()));
    }

    #[test]
    fn test_dom_tree_patches() {
        let vdom_tree = crate::vdom::VDomTree::new(crate::widget::Text::new("Old"));
        let mut dom_tree = DomTree::from_vdom(&vdom_tree);

        let patches = vec![Patch::UpdateText {
            path: vec![],
            new_text: "New".to_string(),
        }];

        dom_tree.apply_patches(&patches).unwrap();

        assert_eq!(dom_tree.root.text_content, Some("New".to_string()));
    }
}