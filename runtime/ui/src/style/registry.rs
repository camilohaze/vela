//! Style registry for named styles
//!
//! This module provides a registry system for storing and retrieving
//! named styles that can be reused across the application.

use super::types::*;
use crate::TextStyle;
use std::collections::HashMap;

/// Registry for named styles
#[derive(Clone, Debug)]
pub struct StyleRegistry {
    styles: HashMap<String, TextStyle>,
}

impl StyleRegistry {
    /// Create a new empty style registry
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
        }
    }

    /// Register a named style
    pub fn register(&mut self, name: impl Into<String>, style: TextStyle) {
        self.styles.insert(name.into(), style);
    }

    /// Get a style by name
    pub fn get(&self, name: &str) -> Option<&TextStyle> {
        self.styles.get(name)
    }

    /// Check if a style is registered
    pub fn contains(&self, name: &str) -> bool {
        self.styles.contains_key(name)
    }

    /// Remove a style from the registry
    pub fn remove(&mut self, name: &str) -> bool {
        self.styles.remove(name).is_some()
    }

    /// Get all registered style names
    pub fn names(&self) -> Vec<String> {
        self.styles.keys().cloned().collect()
    }

    /// Get the number of registered styles
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    /// Clear all styles from the registry
    pub fn clear(&mut self) {
        self.styles.clear();
    }

    /// Merge another registry into this one (other takes precedence)
    pub fn merge(&mut self, other: &StyleRegistry) {
        for (name, style) in &other.styles {
            self.styles.insert(name.clone(), style.clone());
        }
    }

    /// Create a registry with common default styles
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Typography styles
        registry.register("heading1", TextStyle::new()
            .font_size(FontSize::Px(32.0))
            .font_weight(FontWeight::Bold)
            .line_height(LineHeight::Percent(120.0)));

        registry.register("heading2", TextStyle::new()
            .font_size(FontSize::Px(24.0))
            .font_weight(FontWeight::Bold)
            .line_height(LineHeight::Percent(120.0)));

        registry.register("heading3", TextStyle::new()
            .font_size(FontSize::Px(20.0))
            .font_weight(FontWeight::SemiBold)
            .line_height(LineHeight::Percent(120.0)));

        registry.register("body", TextStyle::new()
            .font_size(FontSize::Px(16.0))
            .line_height(LineHeight::Percent(150.0)));

        registry.register("caption", TextStyle::new()
            .font_size(FontSize::Px(14.0))
            .color(Color::named("gray"))
            .line_height(LineHeight::Percent(140.0)));

        registry.register("small", TextStyle::new()
            .font_size(FontSize::Px(12.0))
            .line_height(LineHeight::Percent(130.0)));

        // Interactive styles
        registry.register("link", TextStyle::new()
            .color(Color::hex("#0066cc"))
            .text_decoration(TextDecoration::Underline));

        registry.register("link-hover", TextStyle::new()
            .color(Color::hex("#004499"))
            .text_decoration(TextDecoration::Underline));

        registry.register("button", TextStyle::new()
            .font_weight(FontWeight::Medium)
            .text_align(TextAlign::Center));

        registry.register("button-primary", TextStyle::new()
            .color(Color::hex("#ffffff"))
            .background_color(Color::hex("#0066cc")));

        registry.register("button-secondary", TextStyle::new()
            .color(Color::hex("#0066cc"))
            .background_color(Color::hex("#ffffff")));

        registry
    }
}

impl Default for StyleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global style registry instance
static mut GLOBAL_REGISTRY: Option<StyleRegistry> = None;

/// Get the global style registry (creates one if it doesn't exist)
pub fn global_registry() -> &'static mut StyleRegistry {
    unsafe {
        if GLOBAL_REGISTRY.is_none() {
            GLOBAL_REGISTRY = Some(StyleRegistry::with_defaults());
        }
        GLOBAL_REGISTRY.as_mut().unwrap()
    }
}

/// Set the global style registry
pub fn set_global_registry(registry: StyleRegistry) {
    unsafe {
        GLOBAL_REGISTRY = Some(registry);
    }
}

/// Helper functions for common style operations
pub mod helpers {
    use super::*;

    /// Get a style from the global registry
    pub fn get_style(name: &str) -> Option<TextStyle> {
        global_registry().get(name).cloned()
    }

    /// Register a style in the global registry
    pub fn register_style(name: impl Into<String>, style: TextStyle) {
        global_registry().register(name, style);
    }

    /// Create a style reference for use in widgets
    pub fn style_ref(name: impl Into<String>) -> StyleRef {
        StyleRef::Named(name.into())
    }
}

/// Style reference for composition
#[derive(Clone, Debug, PartialEq)]
pub enum StyleRef {
    /// Reference to a named style
    Named(String),
    /// Inline style definition
    Inline(TextStyle),
}

impl StyleRef {
    pub fn named(name: impl Into<String>) -> Self {
        StyleRef::Named(name.into())
    }

    pub fn inline(style: TextStyle) -> Self {
        StyleRef::Inline(style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_registry_basic_operations() {
        let mut registry = StyleRegistry::new();

        let style = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(16.0));

        // Register style
        registry.register("body", style.clone());
        assert!(registry.contains("body"));
        assert_eq!(registry.len(), 1);

        // Get style
        let retrieved = registry.get("body");
        assert_eq!(retrieved, Some(&style));

        // Remove style
        assert!(registry.remove("body"));
        assert!(!registry.contains("body"));
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_style_registry_with_defaults() {
        let registry = StyleRegistry::with_defaults();

        assert!(registry.contains("heading1"));
        assert!(registry.contains("body"));
        assert!(registry.contains("button"));
        assert!(registry.contains("link"));

        // Check some default styles
        let heading1 = registry.get("heading1").unwrap();
        assert_eq!(heading1.font_size, Some(FontSize::Px(32.0)));
        assert_eq!(heading1.font_weight, Some(FontWeight::Bold));

        let body = registry.get("body").unwrap();
        assert_eq!(body.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(body.line_height, Some(LineHeight::Percent(150.0)));
    }

    #[test]
    fn test_style_registry_merge() {
        let mut registry1 = StyleRegistry::new();
        registry1.register("style1", TextStyle::new().font_family("Inter"));

        let mut registry2 = StyleRegistry::new();
        registry2.register("style2", TextStyle::new().font_size(FontSize::Px(16.0)));
        registry2.register("style1", TextStyle::new().color(Color::hex("#333"))); // Override

        registry1.merge(&registry2);

        assert_eq!(registry1.len(), 2);
        assert!(registry1.contains("style1"));
        assert!(registry1.contains("style2"));

        // Check that style1 was overridden
        let style1 = registry1.get("style1").unwrap();
        assert_eq!(style1.color, Some(Color::hex("#333")));
        assert_eq!(style1.font_family, None); // Was overridden
    }

    #[test]
    fn test_global_registry() {
        // Clear any existing global registry
        unsafe { GLOBAL_REGISTRY = None };

        let registry = global_registry();
        assert!(!registry.is_empty()); // Should have defaults

        // Register a custom style
        registry.register("custom", TextStyle::new().font_family("Custom"));

        // Get it back
        let custom = registry.get("custom");
        assert_eq!(custom.unwrap().font_family, Some("Custom".to_string()));
    }

    #[test]
    fn test_helpers() {
        // Clear any existing global registry
        unsafe { GLOBAL_REGISTRY = None };

        // Register a style using helper
        helpers::register_style("test-style", TextStyle::new().font_family("Test"));

        // Get it back using helper
        let style = helpers::get_style("test-style");
        assert_eq!(style.unwrap().font_family, Some("Test".to_string()));

        // Create style ref
        let style_ref = helpers::style_ref("test-style");
        assert_eq!(style_ref, StyleRef::Named("test-style".to_string()));
    }

    #[test]
    fn test_style_ref() {
        let named_ref = StyleRef::named("body");
        assert_eq!(named_ref, StyleRef::Named("body".to_string()));

        let inline_style = TextStyle::new().color(Color::hex("#333"));
        let inline_ref = StyleRef::inline(inline_style.clone());
        assert_eq!(inline_ref, StyleRef::Inline(inline_style));
    }
}