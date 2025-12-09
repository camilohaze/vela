//! TextStyle and styling APIs for Vela UI Framework
//!
//! This module provides a comprehensive styling system for text and UI components,
//! including typography, colors, spacing, and decorations with support for
//! composition, theming, and performance optimizations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Re-export style system components
pub use crate::style::{
    types::{FontSize, FontWeight, FontStyle, TextDecoration, TextAlign, TextTransform, Color, LetterSpacing, WordSpacing, LineHeight, ResolvedStyle},
    composition::{StyleComposable, ComposedStyle, StyleCascade},
    registry::{StyleRegistry, StyleRef},
    theme::{Theme, ThemeRegistry},
    resolver::{StyleResolver, ContextualStyleResolver},
    widget_integration::{Stylable, Themed, StyleAwareWidget, StyleContext}
};

/// Main TextStyle struct with comprehensive typography and styling properties
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    // Typography
    pub font_family: Option<String>,
    pub font_size: Option<FontSize>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,

    // Colors
    pub color: Option<Color>,
    pub background_color: Option<Color>,

    // Spacing
    pub letter_spacing: Option<LetterSpacing>,
    pub word_spacing: Option<WordSpacing>,
    pub line_height: Option<LineHeight>,

    // Decorations
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_transform: Option<TextTransform>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: None,
            font_size: None,
            font_weight: None,
            font_style: None,
            color: None,
            background_color: None,
            letter_spacing: None,
            word_spacing: None,
            line_height: None,
            text_decoration: None,
            text_align: None,
            text_transform: None,
        }
    }
}

impl TextStyle {
    /// Create a new empty TextStyle
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method for font_family
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    /// Builder method for font_size
    pub fn font_size(mut self, size: FontSize) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Builder method for font_weight
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    /// Builder method for font_style
    pub fn font_style(mut self, style: FontStyle) -> Self {
        self.font_style = Some(style);
        self
    }

    /// Builder method for color
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Builder method for background_color
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Builder method for letter_spacing
    pub fn letter_spacing(mut self, spacing: LetterSpacing) -> Self {
        self.letter_spacing = Some(spacing);
        self
    }

    /// Builder method for word_spacing
    pub fn word_spacing(mut self, spacing: WordSpacing) -> Self {
        self.word_spacing = Some(spacing);
        self
    }

    /// Builder method for line_height
    pub fn line_height(mut self, height: LineHeight) -> Self {
        self.line_height = Some(height);
        self
    }

    /// Builder method for text_decoration
    pub fn text_decoration(mut self, decoration: TextDecoration) -> Self {
        self.text_decoration = Some(decoration);
        self
    }

    /// Builder method for text_align
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    /// Builder method for text_transform
    pub fn text_transform(mut self, transform: TextTransform) -> Self {
        self.text_transform = Some(transform);
        self
    }

    /// Merge another TextStyle into this one (other takes precedence)
    pub fn merge(&mut self, other: &TextStyle) {
        if let Some(font_family) = &other.font_family {
            self.font_family = Some(font_family.clone());
        }
        if let Some(font_size) = &other.font_size {
            self.font_size = Some(font_size.clone());
        }
        if let Some(font_weight) = &other.font_weight {
            self.font_weight = Some(font_weight.clone());
        }
        if let Some(font_style) = &other.font_style {
            self.font_style = Some(font_style.clone());
        }
        if let Some(color) = &other.color {
            self.color = Some(color.clone());
        }
        if let Some(background_color) = &other.background_color {
            self.background_color = Some(background_color.clone());
        }
        if let Some(letter_spacing) = &other.letter_spacing {
            self.letter_spacing = Some(letter_spacing.clone());
        }
        if let Some(word_spacing) = &other.word_spacing {
            self.word_spacing = Some(word_spacing.clone());
        }
        if let Some(line_height) = &other.line_height {
            self.line_height = Some(line_height.clone());
        }
        if let Some(text_decoration) = &other.text_decoration {
            self.text_decoration = Some(text_decoration.clone());
        }
        if let Some(text_align) = &other.text_align {
            self.text_align = Some(text_align.clone());
        }
        if let Some(text_transform) = &other.text_transform {
            self.text_transform = Some(text_transform.clone());
        }
    }

    /// Create a merged copy of this style with another
    pub fn merged_with(&self, other: &TextStyle) -> Self {
        let mut result = self.clone();
        result.merge(other);
        result
    }

    /// Check if this style has any properties set
    pub fn is_empty(&self) -> bool {
        self.font_family.is_none() &&
        self.font_size.is_none() &&
        self.font_weight.is_none() &&
        self.font_style.is_none() &&
        self.color.is_none() &&
        self.background_color.is_none() &&
        self.letter_spacing.is_none() &&
        self.word_spacing.is_none() &&
        self.line_height.is_none() &&
        self.text_decoration.is_none() &&
        self.text_align.is_none() &&
        self.text_transform.is_none()
    }

    /// Get a CSS representation of this style
    pub fn to_css(&self) -> String {
        let mut css = Vec::new();

        if let Some(ref family) = self.font_family {
            css.push(format!("font-family: {};", family));
        }
        if let Some(ref size) = self.font_size {
            css.push(format!("font-size: {};", size.to_css()));
        }
        if let Some(ref weight) = self.font_weight {
            css.push(format!("font-weight: {};", weight.to_css()));
        }
        if let Some(ref style) = self.font_style {
            css.push(format!("font-style: {};", style.to_css()));
        }
        if let Some(ref color) = self.color {
            css.push(format!("color: {};", color.to_css()));
        }
        if let Some(ref bg_color) = self.background_color {
            css.push(format!("background-color: {};", bg_color.to_css()));
        }
        if let Some(ref spacing) = self.letter_spacing {
            css.push(format!("letter-spacing: {};", spacing.to_css()));
        }
        if let Some(ref spacing) = self.word_spacing {
            css.push(format!("word-spacing: {};", spacing.to_css()));
        }
        if let Some(ref height) = self.line_height {
            css.push(format!("line-height: {};", height.to_css()));
        }
        if let Some(ref decoration) = self.text_decoration {
            css.push(format!("text-decoration: {};", decoration.to_css()));
        }
        if let Some(ref align) = self.text_align {
            css.push(format!("text-align: {};", align.to_css()));
        }
        if let Some(ref transform) = self.text_transform {
            css.push(format!("text-transform: {};", transform.to_css()));
        }

        css.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textstyle_builder_pattern() {
        let style = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(16.0))
            .font_weight(FontWeight::Bold)
            .color(Color::hex("#333333"));

        assert_eq!(style.font_family, Some("Inter".to_string()));
        assert_eq!(style.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(style.font_weight, Some(FontWeight::Bold));
        assert_eq!(style.color, Some(Color::hex("#333333")));
    }

    #[test]
    fn test_textstyle_merge() {
        let mut base = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(14.0));

        let overlay = TextStyle::new()
            .font_size(FontSize::Px(16.0))
            .color(Color::hex("#333333"));

        base.merge(&overlay);

        assert_eq!(base.font_family, Some("Inter".to_string()));
        assert_eq!(base.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(base.color, Some(Color::hex("#333333")));
    }

    #[test]
    fn test_textstyle_merged_with() {
        let base = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(14.0));

        let overlay = TextStyle::new()
            .font_size(FontSize::Px(16.0))
            .color(Color::hex("#333333"));

        let merged = base.merged_with(&overlay);

        assert_eq!(merged.font_family, Some("Inter".to_string()));
        assert_eq!(merged.font_size, Some(FontSize::Px(16.0)));
        assert_eq!(merged.color, Some(Color::hex("#333333")));
    }

    #[test]
    fn test_textstyle_is_empty() {
        let empty = TextStyle::new();
        assert!(empty.is_empty());

        let not_empty = TextStyle::new().font_family("Inter");
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_textstyle_to_css() {
        let style = TextStyle::new()
            .font_family("Inter")
            .font_size(FontSize::Px(16.0))
            .color(Color::hex("#333333"));

        let css = style.to_css();
        assert!(css.contains("font-family: Inter;"));
        assert!(css.contains("font-size: 16px;"));
        assert!(css.contains("color: #333333;"));
    }
}