//! Style composition system
//!
//! This module provides utilities for composing and merging styles,
//! including inheritance logic and style precedence rules.

use super::types::*;
use crate::TextStyle;

/// Style composition utilities
pub trait StyleComposable {
    /// Merge another style into this one with proper precedence
    fn merge_style(&mut self, other: &TextStyle);

    /// Create a new style by composing this with another
    fn compose_with(&self, other: &TextStyle) -> TextStyle;

    /// Check if this style inherits from a parent style
    fn inherits_from(&mut self, parent: &TextStyle);
}

impl StyleComposable for TextStyle {
    fn merge_style(&mut self, other: &TextStyle) {
        // Merge strategy: other takes precedence over self
        if other.font_family.is_some() {
            self.font_family = other.font_family.clone();
        }
        if other.font_size.is_some() {
            self.font_size = other.font_size.clone();
        }
        if other.font_weight.is_some() {
            self.font_weight = other.font_weight.clone();
        }
        if other.font_style.is_some() {
            self.font_style = other.font_style.clone();
        }
        if other.color.is_some() {
            self.color = other.color.clone();
        }
        if other.background_color.is_some() {
            self.background_color = other.background_color.clone();
        }
        if other.letter_spacing.is_some() {
            self.letter_spacing = other.letter_spacing.clone();
        }
        if other.word_spacing.is_some() {
            self.word_spacing = other.word_spacing.clone();
        }
        if other.line_height.is_some() {
            self.line_height = other.line_height.clone();
        }
        if other.text_decoration.is_some() {
            self.text_decoration = other.text_decoration.clone();
        }
        if other.text_align.is_some() {
            self.text_align = other.text_align.clone();
        }
        if other.text_transform.is_some() {
            self.text_transform = other.text_transform.clone();
        }
    }

    fn compose_with(&self, other: &TextStyle) -> TextStyle {
        let mut result = self.clone();
        result.merge_style(other);
        result
    }

    fn inherits_from(&mut self, parent: &TextStyle) {
        // Inheritance logic: only inherit if not already set
        if self.font_family.is_none() {
            self.font_family = parent.font_family.clone();
        }
        if self.font_size.is_none() {
            self.font_size = parent.font_size.clone();
        }
        if self.font_weight.is_none() {
            self.font_weight = parent.font_weight.clone();
        }
        if self.font_style.is_none() {
            self.font_style = parent.font_style.clone();
        }
        if self.color.is_none() {
            self.color = parent.color.clone();
        }
        if self.background_color.is_none() {
            self.background_color = parent.background_color.clone();
        }
        if self.letter_spacing.is_none() {
            self.letter_spacing = parent.letter_spacing.clone();
        }
        if self.word_spacing.is_none() {
            self.word_spacing = parent.word_spacing.clone();
        }
        if self.line_height.is_none() {
            self.line_height = parent.line_height.clone();
        }
        if self.text_decoration.is_none() {
            self.text_decoration = parent.text_decoration.clone();
        }
        if self.text_align.is_none() {
            self.text_align = parent.text_align.clone();
        }
        if self.text_transform.is_none() {
            self.text_transform = parent.text_transform.clone();
        }
    }
}

/// Style precedence levels for composition
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum StylePrecedence {
    /// Default theme styles (lowest priority)
    Theme = 0,
    /// Named/global styles
    Named = 1,
    /// Inline styles (highest priority)
    Inline = 2,
}

/// Composed style with precedence information
#[derive(Clone, Debug)]
pub struct ComposedStyle {
    pub style: TextStyle,
    pub precedence: StylePrecedence,
    pub source: String, // For debugging: "theme:light", "named:heading1", "inline"
}

impl ComposedStyle {
    pub fn new(style: TextStyle, precedence: StylePrecedence, source: impl Into<String>) -> Self {
        Self {
            style,
            precedence,
            source: source.into(),
        }
    }

    pub fn theme(style: TextStyle, name: impl Into<String>) -> Self {
        Self::new(style, StylePrecedence::Theme, format!("theme:{}", name.into()))
    }

    pub fn named(style: TextStyle, name: impl Into<String>) -> Self {
        Self::new(style, StylePrecedence::Named, format!("named:{}", name.into()))
    }

    pub fn inline(style: TextStyle, source: impl Into<String>) -> Self {
        Self::new(style, StylePrecedence::Inline, format!("inline:{}", source.into()))
    }
}

/// Compose multiple styles with precedence rules
pub fn compose_styles(styles: &[ComposedStyle]) -> TextStyle {
    let mut result = TextStyle::new();

    // Sort by precedence (higher precedence wins)
    let mut sorted_styles = styles.to_vec();
    sorted_styles.sort_by(|a, b| a.precedence.cmp(&b.precedence));

    // Apply styles in order (lower to higher precedence)
    for composed in sorted_styles {
        result.merge_style(&composed.style);
    }

    result
}

/// Style cascade utilities for widget trees
pub struct StyleCascade {
    inherited_styles: Vec<TextStyle>,
}

impl StyleCascade {
    pub fn new() -> Self {
        Self {
            inherited_styles: Vec::new(),
        }
    }

    pub fn push_inherited(&mut self, style: TextStyle) {
        self.inherited_styles.push(style);
    }

    pub fn pop_inherited(&mut self) {
        self.inherited_styles.pop();
    }

    pub fn get_inherited_style(&self) -> TextStyle {
        let mut result = TextStyle::new();
        for style in &self.inherited_styles {
            result.inherits_from(style);
        }
        result
    }

    pub fn cascade_for_widget(&self, widget_style: Option<&TextStyle>) -> TextStyle {
        let mut result = self.get_inherited_style();

        if let Some(widget_style) = widget_style {
            result.merge_style(widget_style);
        }

        result
    }
}

impl Default for StyleCascade {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_composition_merge() {
        let mut base = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(14.0));

        let overlay = TextStyle::new()
            .font_size(FontSize::Px(16.0))
            .color(Color::hex("#333"));

        base.merge_style(&overlay);

        assert_eq!(base.font_family, Some("Inter".to_string()));
        assert_eq!(base.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(base.color, Some(Color::hex("#333")));
    }

    #[test]
    fn test_style_composition_compose_with() {
        let base = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(14.0));

        let overlay = TextStyle::new()
            .font_size(FontSize::Px(16.0))
            .color(Color::hex("#333"));

        let composed = base.compose_with(&overlay);

        assert_eq!(composed.font_family, Some("Inter".to_string()));
        assert_eq!(composed.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(composed.color, Some(Color::hex("#333")));
    }

    #[test]
    fn test_style_inheritance() {
        let mut child = TextStyle::new()
            .color(Color::hex("#333"));

        let parent = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(16.0))
            .color(Color::hex("#666")); // This should not override child's color

        child.inherits_from(&parent);

        assert_eq!(child.font_family, Some("Inter".to_string()));
        assert_eq!(child.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(child.color, Some(Color::hex("#333"))); // Child's color preserved
    }

    #[test]
    fn test_composed_style_creation() {
        let style = TextStyle::new().font_family("Inter");
        let composed = ComposedStyle::named(style.clone(), "heading");

        assert_eq!(composed.style, style);
        assert_eq!(composed.precedence, StylePrecedence::Named);
        assert_eq!(composed.source, "named:heading");
    }

    #[test]
    fn test_compose_styles_with_precedence() {
        let theme_style = ComposedStyle::theme(
            TextStyle::new().font_family("Inter"),
            "light"
        );

        let named_style = ComposedStyle::named(
            TextStyle::new().font_size(FontSize::Px(16.0)),
            "body"
        );

        let inline_style = ComposedStyle::inline(
            TextStyle::new().color(Color::hex("#333")),
            "widget"
        );

        let composed = compose_styles(&[theme_style, named_style, inline_style]);

        assert_eq!(composed.font_family, Some("Inter".to_string()));
        assert_eq!(composed.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(composed.color, Some(Color::hex("#333")));
    }

    #[test]
    fn test_style_cascade() {
        let mut cascade = StyleCascade::new();

        let parent_style = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(16.0));

        cascade.push_inherited(parent_style);

        let child_style = TextStyle::new()
            .color(Color::hex("#333"));

        let cascaded = cascade.cascade_for_widget(Some(&child_style));

        assert_eq!(cascaded.font_family, Some("Inter".to_string()));
        assert_eq!(cascaded.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(cascaded.color, Some(Color::hex("#333")));
    }

    #[test]
    fn test_style_cascade_no_widget_style() {
        let mut cascade = StyleCascade::new();

        let inherited_style = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(16.0));

        cascade.push_inherited(inherited_style);

        let cascaded = cascade.cascade_for_widget(None);

        assert_eq!(cascaded.font_family, Some("Inter".to_string()));
        assert_eq!(cascaded.font_size, Some(FontSize::Px(16.0)));
    }
}