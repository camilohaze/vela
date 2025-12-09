//! Widget integration for styling system
//!
//! This module provides traits and utilities for integrating the styling
//! system with UI widgets, including automatic style resolution and application.

use super::types::*;
use super::registry::*;
use super::resolver::*;
use crate::TextStyle;
use crate::style::composition::StyleComposable;

/// Trait for widgets that support styling
pub trait Stylable {
    /// Get the current style
    fn style(&self) -> Option<&TextStyle>;

    /// Set a new style
    fn set_style(&mut self, style: TextStyle);

    /// Merge a style with the current one
    fn merge_style(&mut self, style: &TextStyle);

    /// Apply resolved styles to the widget
    fn apply_resolved_style(&mut self, resolved: &ResolvedStyle);

    /// Get style references for resolution
    fn style_refs(&self) -> Vec<StyleRef> {
        Vec::new()
    }

    /// Resolve current styles
    fn resolve_styles(&self) -> ResolvedStyle {
        let refs = self.style_refs();
        StyleResolver::new().resolve_uncached(&refs)
    }
}

/// Trait for widgets that support theming
pub trait Themed {
    /// Get the current theme name
    fn theme(&self) -> Option<&str>;

    /// Set the theme
    fn set_theme(&mut self, theme: &str);

    /// Apply theme styles
    fn apply_theme(&mut self);
}

/// Style-aware widget base implementation
#[derive(Clone, Debug)]
pub struct StyleAwareWidget {
    inline_style: Option<TextStyle>,
    named_styles: Vec<String>,
    theme_styles: Vec<String>,
}

impl StyleAwareWidget {
    /// Create a new style-aware widget
    pub fn new() -> Self {
        Self {
            inline_style: None,
            named_styles: Vec::new(),
            theme_styles: Vec::new(),
        }
    }

    /// Add a named style
    pub fn with_named_style(mut self, name: impl Into<String>) -> Self {
        self.named_styles.push(name.into());
        self
    }

    /// Add a theme style
    pub fn with_theme_style(mut self, name: impl Into<String>) -> Self {
        self.theme_styles.push(name.into());
        self
    }

    /// Set inline style
    pub fn with_inline_style(mut self, style: TextStyle) -> Self {
        self.inline_style = Some(style);
        self
    }

    /// Get all style references
    pub fn style_refs(&self) -> Vec<StyleRef> {
        let mut refs = Vec::new();

        // Add theme styles first (lowest precedence)
        for theme_style in &self.theme_styles {
            refs.push(StyleRef::Named(theme_style.clone()));
        }

        // Add named styles
        for named_style in &self.named_styles {
            refs.push(StyleRef::Named(named_style.clone()));
        }

        // Add inline style (highest precedence)
        if let Some(ref inline) = self.inline_style {
            refs.push(StyleRef::Inline(inline.clone()));
        }

        refs
    }

    /// Resolve styles for this widget
    pub fn resolve_styles(&self) -> ResolvedStyle {
        let refs = self.style_refs();
        StyleResolver::new().resolve_uncached(&refs)
    }
}

impl Default for StyleAwareWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Stylable for StyleAwareWidget {
    fn style(&self) -> Option<&TextStyle> {
        self.inline_style.as_ref()
    }

    fn set_style(&mut self, style: TextStyle) {
        self.inline_style = Some(style);
    }

    fn merge_style(&mut self, style: &TextStyle) {
        if let Some(ref mut existing) = self.inline_style {
            existing.merge_style(style);
        } else {
            self.inline_style = Some(style.clone());
        }
    }

    fn apply_resolved_style(&mut self, resolved: &ResolvedStyle) {
        // This would typically update the widget's visual properties
        // For now, we just store the resolved style as inline style
        let mut style = TextStyle::new();

        if let Some(ref font_family) = resolved.font_family {
            style.font_family = Some(font_family.clone());
        }
        if let Some(ref font_size) = resolved.font_size {
            style.font_size = Some(font_size.clone());
        }
        if let Some(ref font_weight) = resolved.font_weight {
            style.font_weight = Some(font_weight.clone());
        }
        if let Some(ref font_style) = resolved.font_style {
            style.font_style = Some(font_style.clone());
        }
        if let Some(ref color) = resolved.color {
            style.color = Some(color.clone());
        }
        if let Some(ref background_color) = resolved.background_color {
            style.background_color = Some(background_color.clone());
        }
        if let Some(ref letter_spacing) = resolved.letter_spacing {
            style.letter_spacing = Some(letter_spacing.clone());
        }
        if let Some(ref word_spacing) = resolved.word_spacing {
            style.word_spacing = Some(word_spacing.clone());
        }
        if let Some(ref line_height) = resolved.line_height {
            style.line_height = Some(line_height.clone());
        }
        if let Some(ref text_decoration) = resolved.text_decoration {
            style.text_decoration = Some(text_decoration.clone());
        }
        if let Some(ref text_align) = resolved.text_align {
            style.text_align = Some(text_align.clone());
        }
        if let Some(ref text_transform) = resolved.text_transform {
            style.text_transform = Some(text_transform.clone());
        }

        self.inline_style = Some(style);
    }

    fn style_refs(&self) -> Vec<StyleRef> {
        self.style_refs()
    }
}

/// Style cascade context for widget trees
#[derive(Clone, Debug)]
pub struct StyleContext {
    inherited_styles: Vec<TextStyle>,
    current_theme: Option<String>,
}

impl StyleContext {
    /// Create a new style context
    pub fn new() -> Self {
        Self {
            inherited_styles: Vec::new(),
            current_theme: None,
        }
    }

    /// Create with a theme
    pub fn with_theme(theme: impl Into<String>) -> Self {
        Self {
            inherited_styles: Vec::new(),
            current_theme: Some(theme.into()),
        }
    }

    /// Push an inherited style
    pub fn push_inherited(&mut self, style: TextStyle) {
        self.inherited_styles.push(style);
    }

    /// Pop the last inherited style
    pub fn pop_inherited(&mut self) {
        self.inherited_styles.pop();
    }

    /// Get the current inherited style
    pub fn inherited_style(&self) -> TextStyle {
        let mut result = TextStyle::new();
        for style in &self.inherited_styles {
            result.inherits_from(style);
        }
        result
    }

    /// Set the current theme
    pub fn set_theme(&mut self, theme: impl Into<String>) {
        self.current_theme = Some(theme.into());
    }

    /// Get the current theme
    pub fn theme(&self) -> Option<&str> {
        self.current_theme.as_deref()
    }

    /// Resolve styles within this context
    pub fn resolve_styles(&self, style_refs: &[StyleRef]) -> ResolvedStyle {
        let mut contextual_refs = Vec::new();

        // Add inherited styles
        for inherited in &self.inherited_styles {
            contextual_refs.push(StyleRef::Inline(inherited.clone()));
        }

        // Add requested styles
        contextual_refs.extend_from_slice(style_refs);

        StyleResolver::new().resolve_uncached(&contextual_refs)
    }
}

impl Default for StyleContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for widget styling
pub mod helpers {
    use super::*;

    /// Create a styled widget builder
    pub fn styled_widget() -> StyleAwareWidget {
        StyleAwareWidget::new()
    }

    /// Create a style context
    pub fn style_context() -> StyleContext {
        StyleContext::new()
    }

    /// Apply a named style to a stylable widget
    pub fn apply_named_style<W: Stylable>(widget: &mut W, name: &str) {
        if let Some(style) = global_registry().get(name) {
            widget.merge_style(style);
        }
    }

    /// Apply theme styles to a stylable widget
    pub fn apply_theme_styles<W: Stylable>(widget: &mut W, theme_style_names: &[&str]) {
        for name in theme_style_names {
            if let Some(style) = super::super::theme::helpers::theme_text_style(name) {
                widget.merge_style(&style);
            }
        }
    }

    /// Create a cascading style context
    pub fn cascading_context() -> StyleContext {
        let mut context = StyleContext::new();

        // Set current theme if available
        if let Some(theme) = super::super::theme::helpers::current_theme() {
            context.set_theme(theme.name.clone());
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_aware_widget_creation() {
        let widget = StyleAwareWidget::new();
        assert!(widget.inline_style.is_none());
        assert!(widget.named_styles.is_empty());
        assert!(widget.theme_styles.is_empty());
    }

    #[test]
    fn test_style_aware_widget_with_styles() {
        let widget = StyleAwareWidget::new()
            .with_named_style("body")
            .with_theme_style("heading1")
            .with_inline_style(TextStyle::new().color(Color::hex("#333")));

        assert_eq!(widget.named_styles, vec!["body"]);
        assert_eq!(widget.theme_styles, vec!["heading1"]);
        assert!(widget.inline_style.is_some());
    }

    #[test]
    fn test_style_aware_widget_style_refs() {
        let widget = StyleAwareWidget::new()
            .with_theme_style("theme1")
            .with_named_style("named1")
            .with_inline_style(TextStyle::new().font_family("Inline"));

        let refs = widget.style_refs();

        assert_eq!(refs.len(), 3);
        match &refs[0] {
            StyleRef::Named(name) => assert_eq!(name, "theme1"),
            _ => panic!("Expected named style"),
        }
        match &refs[1] {
            StyleRef::Named(name) => assert_eq!(name, "named1"),
            _ => panic!("Expected named style"),
        }
        match &refs[2] {
            StyleRef::Inline(style) => assert_eq!(style.font_family, Some("Inline".to_string())),
            _ => panic!("Expected inline style"),
        }
    }

    #[test]
    fn test_stylable_trait_on_style_aware_widget() {
        let mut widget = StyleAwareWidget::new();

        // Test set_style
        let style = TextStyle::new().font_family("Test");
        widget.set_style(style.clone());
        assert_eq!(widget.style(), Some(&style));

        // Test merge_style
        let overlay = TextStyle::new().color(Color::hex("#333"));
        widget.merge_style(&overlay);

        let current_style = widget.style().unwrap();
        assert_eq!(current_style.font_family, Some("Test".to_string()));
        assert_eq!(current_style.color, Some(Color::hex("#333")));
    }

    #[test]
    fn test_style_context_basic() {
        let context = StyleContext::new();
        assert!(context.inherited_styles.is_empty());
        assert!(context.current_theme.is_none());
    }

    #[test]
    fn test_style_context_with_theme() {
        let context = StyleContext::with_theme("dark");
        assert_eq!(context.theme(), Some("dark"));
    }

    #[test]
    fn test_style_context_inheritance() {
        let mut context = StyleContext::new();

        let parent_style = TextStyle::new().font_family("Parent");
        context.push_inherited(parent_style);

        let child_style = TextStyle::new().color(Color::hex("#333"));
        context.push_inherited(child_style);

        let inherited = context.inherited_style();
        assert_eq!(inherited.font_family, Some("Parent".to_string()));
        assert_eq!(inherited.color, Some(Color::hex("#333")));

        context.pop_inherited();
        let inherited_after_pop = context.inherited_style();
        assert_eq!(inherited_after_pop.font_family, Some("Parent".to_string()));
        assert_eq!(inherited_after_pop.color, None);
    }

    #[test]
    fn test_style_context_resolve_styles() {
        let mut context = StyleContext::new();

        // Add inherited style
        let inherited = TextStyle::new().font_family("Inherited");
        context.push_inherited(inherited);

        // Resolve with additional styles
        let style_refs = vec![
            StyleRef::Inline(TextStyle::new().font_size(FontSize::Px(16.0))),
        ];

        let resolved = context.resolve_styles(&style_refs);

        assert_eq!(resolved.font_family, Some("Inherited".to_string()));
        assert_eq!(resolved.font_size, Some(FontSize::Px(16.0)));
    }

    #[test]
    fn test_widget_helpers_styled_widget() {
        let widget = helpers::styled_widget();
        assert!(widget.inline_style.is_none());
    }

    #[test]
    fn test_widget_helpers_style_context() {
        let context = helpers::style_context();
        assert!(context.inherited_styles.is_empty());
    }

    #[test]
    fn test_apply_resolved_style() {
        let mut widget = StyleAwareWidget::new();

        let resolved = ResolvedStyle {
            font_family: Some("Resolved".to_string()),
            font_size: Some(FontSize::Px(18.0)),
            color: Some(Color::hex("#666")),
            ..Default::default()
        };

        widget.apply_resolved_style(&resolved);

        let applied_style = widget.style().unwrap();
        assert_eq!(applied_style.font_family, Some("Resolved".to_string()));
        assert_eq!(applied_style.font_size, Some(FontSize::Px(18.0)));
        assert_eq!(applied_style.color, Some(Color::hex("#666")));
    }
}