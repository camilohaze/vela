/*!
# Finders

Widget finders for locating widgets in the test tree.

## Example

```rust,no_run
use vela_testing::finders::*;

// Find by key
let button = find::by_key("submit-button");

// Find by text
let title = find::by_text("Welcome");

// Find by type
let buttons = find::by_type::<Button>();
```

*/

use crate::widget_testing::TestableWidget;
use std::any::TypeId;

/// Trait for finding widgets in the test tree
#[async_trait::async_trait]
pub trait Finder: Send + Sync {
    async fn find(&self, widgets: &std::collections::HashMap<String, Box<dyn TestableWidget>>) -> Result<Vec<Box<dyn TestableWidget>>, String>;
}

/// Find widget by key/ID
pub struct ByKey {
    pub key: String,
}

impl ByKey {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Finder for ByKey {
    async fn find(&self, widgets: &std::collections::HashMap<String, Box<dyn TestableWidget>>) -> Result<Vec<Box<dyn TestableWidget>>, String> {
        if let Some(widget) = widgets.get(&self.key) {
            Ok(vec![widget.clone_box().await])
        } else {
            Ok(vec![])
        }
    }
}

/// Find widget by text content
pub struct ByText {
    pub text: String,
}

impl ByText {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Finder for ByText {
    async fn find(&self, widgets: &std::collections::HashMap<String, Box<dyn TestableWidget>>) -> Result<Vec<Box<dyn TestableWidget>>, String> {
        let mut results = Vec::new();

        for widget in widgets.values() {
            let properties = widget.get_properties().await;
            if let Some(text) = properties.get("text") {
                if let Some(text_str) = text.as_str() {
                    if text_str.contains(&self.text) {
                        results.push(widget.clone_box().await);
                    }
                }
            }
        }

        Ok(results)
    }
}

/// Find widget by type (simplified - would need runtime type info)
pub struct ByType {
    pub type_name: String,
}

impl ByType {
    pub fn new(type_name: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Finder for ByType {
    async fn find(&self, widgets: &std::collections::HashMap<String, Box<dyn TestableWidget>>) -> Result<Vec<Box<dyn TestableWidget>>, String> {
        let mut results = Vec::new();

        for widget in widgets.values() {
            // Simplified type checking - in real implementation would use TypeId
            let properties = widget.get_properties().await;
            if properties.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == self.type_name)
                .unwrap_or(false) {
                results.push(widget.clone_box().await);
            }
        }

        Ok(results)
    }
}

/// Find descendant widgets
pub struct Descendant {
    pub ancestor: Box<dyn Finder>,
    pub descendant: Box<dyn Finder>,
}

impl Descendant {
    pub fn new(ancestor: Box<dyn Finder>, descendant: Box<dyn Finder>) -> Self {
        Self {
            ancestor,
            descendant,
        }
    }
}

#[async_trait::async_trait]
impl Finder for Descendant {
    async fn find(&self, widgets: &std::collections::HashMap<String, Box<dyn TestableWidget>>) -> Result<Vec<Box<dyn TestableWidget>>, String> {
        let ancestors = self.ancestor.find(widgets).await?;
        let mut results = Vec::new();

        for ancestor in ancestors {
            let descendant_results = self.descendant.find(widgets).await?;
            let ancestor_children = ancestor.get_children().await;
            for descendant in descendant_results {
                // Check if descendant is actually a child of ancestor
                if ancestor_children.iter().any(|child| child.get_id() == descendant.get_id()) {
                    results.push(descendant);
                }
            }
        }

        Ok(results)
    }
}

/// Convenience functions for creating finders
pub mod find {
    use super::*;

    pub fn by_key(key: &str) -> ByKey {
        ByKey::new(key)
    }

    pub fn by_text(text: &str) -> ByText {
        ByText::new(text)
    }

    pub fn by_type(type_name: &str) -> ByType {
        ByType::new(type_name)
    }

    pub fn descendant(ancestor: Box<dyn Finder>, descendant: Box<dyn Finder>) -> Descendant {
        Descendant::new(ancestor, descendant)
    }
}