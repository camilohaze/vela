//! Context-based theming system
//!
//! This module provides contextual theming capabilities, allowing different
//! parts of the UI to have different themes through ThemeProvider widgets.

use crate::widget::{Widget, BaseWidget};
use crate::vdom::VDomNode;
use crate::context::BuildContext;
use crate::style::theme::{Theme, global_theme_registry};
use std::rc::Rc;

/// Theme provider widget that provides theme to its subtree
#[derive(Debug)]
pub struct ThemeProvider {
    base: BaseWidget,
    theme: Rc<Theme>,
    child: Box<dyn Widget>,
}

impl ThemeProvider {
    /// Create a new theme provider
    pub fn new(theme: Theme, child: Box<dyn Widget>) -> Self {
        Self {
            base: BaseWidget::new(),
            theme: Rc::new(theme),
            child,
        }
    }

    /// Create with a named theme from registry
    pub fn named(theme_name: &str, child: Box<dyn Widget>) -> Option<Self> {
        let registry = global_theme_registry();
        registry.get(theme_name).map(|theme| {
            Self::new(theme.clone(), child)
        })
    }

    /// Set key for reconciliation
    pub fn with_key(mut self, key: crate::key::Key) -> Self {
        self.base = BaseWidget::with_key(key);
        self
    }
}

impl Widget for ThemeProvider {
    fn build(&self, context: &BuildContext) -> VDomNode {
        // Create child context with theme name inherited
        let mut child_context = context.clone();
        child_context.set_inherited(
            "theme_name".to_string(),
            serde_json::Value::String(self.theme.name.clone())
        );

        // Build child with themed context
        let child_node = self.child.build(&child_context);

        // Wrap in a div with theme data attribute for CSS theming
        let mut node = VDomNode::element("div");
        node.attributes.insert("class".to_string(), "vela-theme-provider".to_string());
        node.attributes.insert("data-theme".to_string(), self.theme.name.clone());
        node.children.push(child_node);

        node
    }

    fn key(&self) -> Option<crate::key::Key> {
        self.base.key()
    }
}

/// Theme consumer widget that builds based on current theme
pub struct ThemeConsumer<F> {
    base: BaseWidget,
    builder: F,
}

impl<F> std::fmt::Debug for ThemeConsumer<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeConsumer")
            .field("base", &self.base)
            .field("builder", &"<function>")
            .finish()
    }
}

impl<F> ThemeConsumer<F>
where
    F: Fn(&Theme) -> Box<dyn Widget> + 'static,
{
    /// Create a new theme consumer
    pub fn new(builder: F) -> Self {
        Self {
            base: BaseWidget::new(),
            builder,
        }
    }

    /// Set key for reconciliation
    pub fn with_key(mut self, key: crate::key::Key) -> Self {
        self.base = BaseWidget::with_key(key);
        self
    }
}

impl<F> Widget for ThemeConsumer<F>
where
    F: Fn(&Theme) -> Box<dyn Widget> + 'static,
{
    fn build(&self, context: &BuildContext) -> VDomNode {
        // Get current theme from context or global
        let theme = get_current_theme(context);

        // Build child widget using theme
        let child_widget = (self.builder)(&theme);
        child_widget.build(context)
    }

    fn key(&self) -> Option<crate::key::Key> {
        self.base.key()
    }
}

/// Get the current theme from context or global fallback
pub fn get_current_theme(context: &BuildContext) -> Rc<Theme> {
    // Check if theme_name is inherited in context
    if let Some(serde_json::Value::String(theme_name)) = context.get_inherited("theme_name") {
        // Try to get theme from global registry
        let registry = global_theme_registry();
        if let Some(theme) = registry.get(&theme_name) {
            return Rc::new(theme.clone());
        }
    }

    // Fallback to global theme
    let registry = global_theme_registry();
    if let Some(current_theme) = registry.current() {
        Rc::new(current_theme.clone())
    } else {
        // Ultimate fallback: light theme
        Rc::new(Theme::light())
    }
}

/// Hook-style function to get current theme (for use in widget build methods)
pub fn use_theme(context: &BuildContext) -> Rc<Theme> {
    get_current_theme(context)
}

/// Extension trait for widgets to easily access theme
pub trait ThemedWidget: Widget {
    /// Get current theme in build context
    fn current_theme(&self, context: &BuildContext) -> Rc<Theme> {
        get_current_theme(context)
    }
}

// Auto-implement for all widgets
impl<W: Widget> ThemedWidget for W {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::Text;
    use crate::style::theme::Theme;

    #[test]
    fn test_theme_provider_creation() {
        let theme = Theme::light();
        let child = Box::new(Text::new("Hello"));
        let provider = ThemeProvider::new(theme, child);

        assert_eq!(provider.theme.name, "light");
    }

    #[test]
    fn test_theme_provider_named() {
        let child = Box::new(Text::new("Hello"));
        let provider = ThemeProvider::named("light", child);

        assert!(provider.is_some());
        assert_eq!(provider.unwrap().theme.name, "light");
    }

    #[test]
    fn test_theme_consumer() {
        let consumer = ThemeConsumer::new(|theme: &Theme| {
            Box::new(crate::Text::new(&format!("Theme: {}", theme.name)))
        });

        let context = BuildContext::new();
        let node = consumer.build(&context);

        // Should build successfully
        assert_eq!(node.tag_name.as_ref().unwrap(), "span"); // Text widget wraps in span
    }

    #[test]
    fn test_get_current_theme_fallback() {
        let context = BuildContext::new();
        let theme = get_current_theme(&context);

        // Should fallback to light theme
        assert_eq!(theme.name, "light");
    }

    #[test]
    fn test_use_theme() {
        let context = BuildContext::new();
        let theme = use_theme(&context);

        assert_eq!(theme.name, "light");
    }
}