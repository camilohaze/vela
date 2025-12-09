//! Build context for widget construction

use crate::key::Key;
use std::collections::HashMap;

/// Context passed during widget building
#[derive(Debug)]
pub struct BuildContext {
    /// Ancestor keys for efficient reconciliation
    pub ancestor_keys: Vec<Key>,
    /// Build configuration
    pub config: BuildConfig,
    /// Inherited properties
    pub inherited: HashMap<String, serde_json::Value>,
}

impl BuildContext {
    /// Create a new root build context
    pub fn new() -> Self {
        Self {
            ancestor_keys: Vec::new(),
            config: BuildConfig::default(),
            inherited: HashMap::new(),
        }
    }

    /// Create a child context with additional ancestor key
    pub fn with_key(mut self, key: Key) -> Self {
        self.ancestor_keys.push(key);
        self
    }

    /// Get inherited property
    pub fn get_inherited(&self, key: &str) -> Option<&serde_json::Value> {
        self.inherited.get(key)
    }

    /// Set inherited property
    pub fn set_inherited(&mut self, key: String, value: serde_json::Value) {
        self.inherited.insert(key, value);
    }

    /// Check if context has a specific ancestor key
    pub fn has_ancestor_key(&self, key: &Key) -> bool {
        self.ancestor_keys.contains(key)
    }

    /// Get the current depth in the widget tree
    pub fn depth(&self) -> usize {
        self.ancestor_keys.len()
    }
}

impl Default for BuildContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for widget building
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Enable debug mode
    pub debug: bool,
    /// Enable performance profiling
    pub profile: bool,
    /// Maximum widget tree depth
    pub max_depth: usize,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            debug: cfg!(debug_assertions),
            profile: false,
            max_depth: 1000,
        }
    }
}

/// Builder for BuildContext
pub struct BuildContextBuilder {
    ancestor_keys: Vec<Key>,
    config: BuildConfig,
    inherited: HashMap<String, serde_json::Value>,
}

impl BuildContextBuilder {
    pub fn new() -> Self {
        Self {
            ancestor_keys: Vec::new(),
            config: BuildConfig::default(),
            inherited: HashMap::new(),
        }
    }

    pub fn with_ancestor_key(mut self, key: Key) -> Self {
        self.ancestor_keys.push(key);
        self
    }

    pub fn with_config(mut self, config: BuildConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_inherited(
        mut self,
        key: String,
        value: serde_json::Value,
    ) -> Self {
        self.inherited.insert(key, value);
        self
    }

    pub fn build(self) -> BuildContext {
        BuildContext {
            ancestor_keys: self.ancestor_keys,
            config: self.config,
            inherited: self.inherited,
        }
    }
}

impl Default for BuildContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_context_creation() {
        let context = BuildContext::new();
        assert_eq!(context.depth(), 0);
        assert!(context.ancestor_keys.is_empty());
    }

    #[test]
    fn test_build_context_with_key() {
        let key = crate::key::Key::string("test");
        let context = BuildContext::new().with_key(key.clone());

        assert_eq!(context.depth(), 1);
        assert!(context.has_ancestor_key(&key));
    }

    #[test]
    fn test_inherited_properties() {
        let mut context = BuildContext::new();
        context.set_inherited("theme".to_string(), serde_json::json!("dark"));

        let theme = context.get_inherited("theme");
        assert_eq!(theme, Some(&serde_json::json!("dark")));

        let missing = context.get_inherited("missing");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_build_context_builder() {
        let key = crate::key::Key::int(42);
        let context = BuildContextBuilder::new()
            .with_ancestor_key(key.clone())
            .with_inherited("test".to_string(), serde_json::json!(123))
            .build();

        assert_eq!(context.depth(), 1);
        assert!(context.has_ancestor_key(&key));

        let value = context.get_inherited("test");
        assert_eq!(value, Some(&serde_json::json!(123)));
    }
}