//! Core types for the styling system
//!
//! This module defines all the fundamental types used in the styling system,
//! including FontSize, FontWeight, Color, and other style-related enums.

use serde::{Deserialize, Serialize};
use std::fmt;

// Forward declaration to avoid circular imports
// TextStyle is defined in the parent text_style module
use crate::TextStyle;

/// Font size with different units
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FontSize {
    Px(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
}

impl FontSize {
    pub fn to_css(&self) -> String {
        match self {
            FontSize::Px(value) => format!("{}px", value),
            FontSize::Em(value) => format!("{}em", value),
            FontSize::Rem(value) => format!("{}rem", value),
            FontSize::Percent(value) => format!("{}%", value),
        }
    }
}

/// Font weight values
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Regular = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl FontWeight {
    pub fn to_css(&self) -> String {
        match self {
            FontWeight::Thin => "100".to_string(),
            FontWeight::ExtraLight => "200".to_string(),
            FontWeight::Light => "300".to_string(),
            FontWeight::Regular => "400".to_string(),
            FontWeight::Medium => "500".to_string(),
            FontWeight::SemiBold => "600".to_string(),
            FontWeight::Bold => "700".to_string(),
            FontWeight::ExtraBold => "800".to_string(),
            FontWeight::Black => "900".to_string(),
        }
    }
}

/// Font style variants
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl FontStyle {
    pub fn to_css(&self) -> String {
        match self {
            FontStyle::Normal => "normal".to_string(),
            FontStyle::Italic => "italic".to_string(),
            FontStyle::Oblique => "oblique".to_string(),
        }
    }
}

/// Color representation with multiple formats
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: f32 },
    Hsl { h: f32, s: f32, l: f32 },
    Hsla { h: f32, s: f32, l: f32, a: f32 },
    Hex(String),
    Named(String),
}

impl Color {
    /// Create a color from hex string (with or without #)
    pub fn hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        Color::Hex(format!("#{}", hex))
    }

    /// Create RGB color
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb { r, g, b }
    }

    /// Create RGBA color
    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Color::Rgba { r, g, b, a }
    }

    /// Create HSL color
    pub fn hsl(h: f32, s: f32, l: f32) -> Self {
        Color::Hsl { h, s, l }
    }

    /// Create HSLA color
    pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        Color::Hsla { h, s, l, a }
    }

    /// Create named color
    pub fn named(name: &str) -> Self {
        Color::Named(name.to_string())
    }

    pub fn to_css(&self) -> String {
        match self {
            Color::Rgb { r, g, b } => format!("rgb({}, {}, {})", r, g, b),
            Color::Rgba { r, g, b, a } => format!("rgba({}, {}, {}, {})", r, g, b, a),
            Color::Hsl { h, s, l } => format!("hsl({}, {}%, {}%)", h, s * 100.0, l * 100.0),
            Color::Hsla { h, s, l, a } => format!("hsla({}, {}%, {}%, {})", h, s * 100.0, l * 100.0, a),
            Color::Hex(hex) => hex.clone(),
            Color::Named(name) => name.clone(),
        }
    }
}

/// Letter spacing values
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LetterSpacing {
    Px(f32),
    Em(f32),
    Normal,
}

impl LetterSpacing {
    pub fn to_css(&self) -> String {
        match self {
            LetterSpacing::Px(value) => format!("{}px", value),
            LetterSpacing::Em(value) => format!("{}em", value),
            LetterSpacing::Normal => "normal".to_string(),
        }
    }
}

/// Word spacing values
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WordSpacing {
    Px(f32),
    Em(f32),
    Normal,
}

impl WordSpacing {
    pub fn to_css(&self) -> String {
        match self {
            WordSpacing::Px(value) => format!("{}px", value),
            WordSpacing::Em(value) => format!("{}em", value),
            WordSpacing::Normal => "normal".to_string(),
        }
    }
}

/// Line height values
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LineHeight {
    Px(f32),
    Em(f32),
    Percent(f32),
    Number(f32),
    Normal,
}

impl LineHeight {
    pub fn to_css(&self) -> String {
        match self {
            LineHeight::Px(value) => format!("{}px", value),
            LineHeight::Em(value) => format!("{}em", value),
            LineHeight::Percent(value) => format!("{}%", value),
            LineHeight::Number(value) => value.to_string(),
            LineHeight::Normal => "normal".to_string(),
        }
    }
}

/// Text decoration options
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
}

impl TextDecoration {
    pub fn to_css(&self) -> String {
        match self {
            TextDecoration::None => "none".to_string(),
            TextDecoration::Underline => "underline".to_string(),
            TextDecoration::Overline => "overline".to_string(),
            TextDecoration::LineThrough => "line-through".to_string(),
        }
    }
}

/// Text alignment options
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
    Start,
    End,
}

impl TextAlign {
    pub fn to_css(&self) -> String {
        match self {
            TextAlign::Left => "left".to_string(),
            TextAlign::Right => "right".to_string(),
            TextAlign::Center => "center".to_string(),
            TextAlign::Justify => "justify".to_string(),
            TextAlign::Start => "start".to_string(),
            TextAlign::End => "end".to_string(),
        }
    }
}

/// Text transformation options
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub enum TextTransform {
    None,
    Capitalize,
    Uppercase,
    Lowercase,
}

impl TextTransform {
    pub fn to_css(&self) -> String {
        match self {
            TextTransform::None => "none".to_string(),
            TextTransform::Capitalize => "capitalize".to_string(),
            TextTransform::Uppercase => "uppercase".to_string(),
            TextTransform::Lowercase => "lowercase".to_string(),
        }
    }
}

/// Resolved style after composition and inheritance
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedStyle {
    pub font_family: Option<String>,
    pub font_size: Option<FontSize>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub letter_spacing: Option<LetterSpacing>,
    pub word_spacing: Option<WordSpacing>,
    pub line_height: Option<LineHeight>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_transform: Option<TextTransform>,
}

impl Default for ResolvedStyle {
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

impl ResolvedStyle {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_size_to_css() {
        assert_eq!(FontSize::Px(16.0).to_css(), "16px");
        assert_eq!(FontSize::Em(1.5).to_css(), "1.5em");
        assert_eq!(FontSize::Rem(2.0).to_css(), "2rem");
        assert_eq!(FontSize::Percent(150.0).to_css(), "150%");
    }

    #[test]
    fn test_font_weight_to_css() {
        assert_eq!(FontWeight::Regular.to_css(), "400");
        assert_eq!(FontWeight::Bold.to_css(), "700");
        assert_eq!(FontWeight::Black.to_css(), "900");
    }

    #[test]
    fn test_color_hex() {
        let color = Color::hex("#333333");
        assert_eq!(color.to_css(), "#333333");

        let color2 = Color::hex("666666");
        assert_eq!(color2.to_css(), "#666666");
    }

    #[test]
    fn test_color_rgb() {
        let color = Color::rgb(255, 0, 0);
        assert_eq!(color.to_css(), "rgb(255, 0, 0)");
    }

    #[test]
    fn test_color_rgba() {
        let color = Color::rgba(255, 0, 0, 0.5);
        assert_eq!(color.to_css(), "rgba(255, 0, 0, 0.5)");
    }

    #[test]
    fn test_letter_spacing_to_css() {
        assert_eq!(LetterSpacing::Px(1.0).to_css(), "1px");
        assert_eq!(LetterSpacing::Em(0.1).to_css(), "0.1em");
        assert_eq!(LetterSpacing::Normal.to_css(), "normal");
    }

    #[test]
    fn test_text_align_to_css() {
        assert_eq!(TextAlign::Left.to_css(), "left");
        assert_eq!(TextAlign::Center.to_css(), "center");
        assert_eq!(TextAlign::Justify.to_css(), "justify");
    }

    #[test]
    fn test_resolved_style_merge() {
        let mut resolved = ResolvedStyle::default();
        let text_style = TextStyle::new()
            .font_family("Inter")
            .color(Color::hex("#333"));

        resolved.merge(&text_style);

        assert_eq!(resolved.font_family, Some("Inter".to_string()));
        assert_eq!(resolved.color, Some(Color::hex("#333")));
    }
}