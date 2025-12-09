//! Theme system for contextual styling
//!
//! This module provides a theming system that allows switching between
//! different visual themes (light, dark, custom) with automatic style resolution.

use super::types::*;
use crate::TextStyle;
use std::collections::HashMap;

/// Theme definition with named styles and values
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub text_styles: HashMap<String, TextStyle>,
    pub colors: HashMap<String, Color>,
    pub spacing: HashMap<String, f32>,
    pub breakpoints: HashMap<String, f32>,
}

impl Theme {
    /// Create a new theme with a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text_styles: HashMap::new(),
            colors: HashMap::new(),
            spacing: HashMap::new(),
            breakpoints: HashMap::new(),
        }
    }

    /// Create a light theme with sensible defaults
    pub fn light() -> Self {
        let mut theme = Self::new("light");

        // Text styles
        theme.text_styles.insert("heading1".to_string(), TextStyle::new()
            .font_size(FontSize::Px(32.0))
            .font_weight(FontWeight::Bold)
            .color(Color::hex("#1a1a1a"))
            .line_height(LineHeight::Percent(120.0)));

        theme.text_styles.insert("heading2".to_string(), TextStyle::new()
            .font_size(FontSize::Px(24.0))
            .font_weight(FontWeight::Bold)
            .color(Color::hex("#1a1a1a"))
            .line_height(LineHeight::Percent(120.0)));

        theme.text_styles.insert("heading3".to_string(), TextStyle::new()
            .font_size(FontSize::Px(20.0))
            .font_weight(FontWeight::SemiBold)
            .color(Color::hex("#1a1a1a"))
            .line_height(LineHeight::Percent(120.0)));

        theme.text_styles.insert("body".to_string(), TextStyle::new()
            .font_size(FontSize::Px(16.0))
            .color(Color::hex("#333333"))
            .line_height(LineHeight::Percent(150.0)));

        theme.text_styles.insert("caption".to_string(), TextStyle::new()
            .font_size(FontSize::Px(14.0))
            .color(Color::hex("#666666"))
            .line_height(LineHeight::Percent(140.0)));

        theme.text_styles.insert("small".to_string(), TextStyle::new()
            .font_size(FontSize::Px(12.0))
            .color(Color::hex("#666666"))
            .line_height(LineHeight::Percent(130.0)));

        // Colors
        theme.colors.insert("primary".to_string(), Color::hex("#0066cc"));
        theme.colors.insert("secondary".to_string(), Color::hex("#666666"));
        theme.colors.insert("success".to_string(), Color::hex("#28a745"));
        theme.colors.insert("warning".to_string(), Color::hex("#ffc107"));
        theme.colors.insert("error".to_string(), Color::hex("#dc3545"));
        theme.colors.insert("info".to_string(), Color::hex("#17a2b8"));

        theme.colors.insert("background".to_string(), Color::hex("#ffffff"));
        theme.colors.insert("surface".to_string(), Color::hex("#f8f9fa"));
        theme.colors.insert("text".to_string(), Color::hex("#333333"));
        theme.colors.insert("text-secondary".to_string(), Color::hex("#666666"));
        theme.colors.insert("border".to_string(), Color::hex("#dee2e6"));

        // Spacing
        theme.spacing.insert("xs".to_string(), 4.0);
        theme.spacing.insert("sm".to_string(), 8.0);
        theme.spacing.insert("md".to_string(), 16.0);
        theme.spacing.insert("lg".to_string(), 24.0);
        theme.spacing.insert("xl".to_string(), 32.0);
        theme.spacing.insert("xxl".to_string(), 48.0);

        // Breakpoints
        theme.breakpoints.insert("sm".to_string(), 576.0);
        theme.breakpoints.insert("md".to_string(), 768.0);
        theme.breakpoints.insert("lg".to_string(), 992.0);
        theme.breakpoints.insert("xl".to_string(), 1200.0);

        theme
    }

    /// Create a dark theme with sensible defaults
    pub fn dark() -> Self {
        let mut theme = Self::new("dark");

        // Text styles
        theme.text_styles.insert("heading1".to_string(), TextStyle::new()
            .font_size(FontSize::Px(32.0))
            .font_weight(FontWeight::Bold)
            .color(Color::hex("#ffffff"))
            .line_height(LineHeight::Percent(120.0)));

        theme.text_styles.insert("heading2".to_string(), TextStyle::new()
            .font_size(FontSize::Px(24.0))
            .font_weight(FontWeight::Bold)
            .color(Color::hex("#ffffff"))
            .line_height(LineHeight::Percent(120.0)));

        theme.text_styles.insert("heading3".to_string(), TextStyle::new()
            .font_size(FontSize::Px(20.0))
            .font_weight(FontWeight::SemiBold)
            .color(Color::hex("#ffffff"))
            .line_height(LineHeight::Percent(120.0)));

        theme.text_styles.insert("body".to_string(), TextStyle::new()
            .font_size(FontSize::Px(16.0))
            .color(Color::hex("#e9ecef"))
            .line_height(LineHeight::Percent(150.0)));

        theme.text_styles.insert("caption".to_string(), TextStyle::new()
            .font_size(FontSize::Px(14.0))
            .color(Color::hex("#adb5bd"))
            .line_height(LineHeight::Percent(140.0)));

        theme.text_styles.insert("small".to_string(), TextStyle::new()
            .font_size(FontSize::Px(12.0))
            .color(Color::hex("#adb5bd"))
            .line_height(LineHeight::Percent(130.0)));

        // Colors
        theme.colors.insert("primary".to_string(), Color::hex("#4dabf7"));
        theme.colors.insert("secondary".to_string(), Color::hex("#adb5bd"));
        theme.colors.insert("success".to_string(), Color::hex("#51cf66"));
        theme.colors.insert("warning".to_string(), Color::hex("#ffd43b"));
        theme.colors.insert("error".to_string(), Color::hex("#ff6b6b"));
        theme.colors.insert("info".to_string(), Color::hex("#74c0fc"));

        theme.colors.insert("background".to_string(), Color::hex("#212529"));
        theme.colors.insert("surface".to_string(), Color::hex("#343a40"));
        theme.colors.insert("text".to_string(), Color::hex("#e9ecef"));
        theme.colors.insert("text-secondary".to_string(), Color::hex("#adb5bd"));
        theme.colors.insert("border".to_string(), Color::hex("#495057"));

        // Spacing (same as light theme)
        theme.spacing.insert("xs".to_string(), 4.0);
        theme.spacing.insert("sm".to_string(), 8.0);
        theme.spacing.insert("md".to_string(), 16.0);
        theme.spacing.insert("lg".to_string(), 24.0);
        theme.spacing.insert("xl".to_string(), 32.0);
        theme.spacing.insert("xxl".to_string(), 48.0);

        // Breakpoints (same as light theme)
        theme.breakpoints.insert("sm".to_string(), 576.0);
        theme.breakpoints.insert("md".to_string(), 768.0);
        theme.breakpoints.insert("lg".to_string(), 992.0);
        theme.breakpoints.insert("xl".to_string(), 1200.0);

        theme
    }

    /// Add a text style to the theme
    pub fn add_text_style(&mut self, name: impl Into<String>, style: TextStyle) {
        self.text_styles.insert(name.into(), style);
    }

    /// Get a text style from the theme
    pub fn get_text_style(&self, name: &str) -> Option<&TextStyle> {
        self.text_styles.get(name)
    }

    /// Add a color to the theme
    pub fn add_color(&mut self, name: impl Into<String>, color: Color) {
        self.colors.insert(name.into(), color);
    }

    /// Get a color from the theme
    pub fn get_color(&self, name: &str) -> Option<&Color> {
        self.colors.get(name)
    }

    /// Add spacing to the theme
    pub fn add_spacing(&mut self, name: impl Into<String>, value: f32) {
        self.spacing.insert(name.into(), value);
    }

    /// Get spacing from the theme
    pub fn get_spacing(&self, name: &str) -> Option<f32> {
        self.spacing.get(name).copied()
    }

    /// Add a breakpoint to the theme
    pub fn add_breakpoint(&mut self, name: impl Into<String>, value: f32) {
        self.breakpoints.insert(name.into(), value);
    }

    /// Get a breakpoint from the theme
    pub fn get_breakpoint(&self, name: &str) -> Option<f32> {
        self.breakpoints.get(name).copied()
    }

    /// Merge another theme into this one (other takes precedence)
    pub fn merge(&mut self, other: &Theme) {
        // Merge text styles
        for (name, style) in &other.text_styles {
            self.text_styles.insert(name.clone(), style.clone());
        }

        // Merge colors
        for (name, color) in &other.colors {
            self.colors.insert(name.clone(), color.clone());
        }

        // Merge spacing
        for (name, value) in &other.spacing {
            self.spacing.insert(name.clone(), *value);
        }

        // Merge breakpoints
        for (name, value) in &other.breakpoints {
            self.breakpoints.insert(name.clone(), *value);
        }
    }

    /// Create a derived theme with modifications
    pub fn derive(&self, name: impl Into<String>) -> Theme {
        let mut derived = self.clone();
        derived.name = name.into();
        derived
    }
}

/// Theme registry for managing multiple themes
#[derive(Clone, Debug)]
pub struct ThemeRegistry {
    themes: HashMap<String, Theme>,
    current_theme: Option<String>,
}

impl ThemeRegistry {
    /// Create a new theme registry
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
            current_theme: None,
        }
    }

    /// Register a theme
    pub fn register(&mut self, theme: Theme) {
        let name = theme.name.clone();
        self.themes.insert(name, theme);
    }

    /// Set the current theme
    pub fn set_current(&mut self, name: &str) -> Result<(), String> {
        if self.themes.contains_key(name) {
            self.current_theme = Some(name.to_string());
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }

    /// Get the current theme
    pub fn current(&self) -> Option<&Theme> {
        self.current_theme.as_ref()
            .and_then(|name| self.themes.get(name))
    }

    /// Get a theme by name
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Get all theme names
    pub fn theme_names(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// Initialize with default themes
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Theme::light());
        registry.register(Theme::dark());
        registry.set_current("light").unwrap();
        registry
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global theme registry instance
static mut GLOBAL_THEME_REGISTRY: Option<ThemeRegistry> = None;

/// Get the global theme registry
pub fn global_theme_registry() -> &'static mut ThemeRegistry {
    unsafe {
        if GLOBAL_THEME_REGISTRY.is_none() {
            GLOBAL_THEME_REGISTRY = Some(ThemeRegistry::with_defaults());
        }
        GLOBAL_THEME_REGISTRY.as_mut().unwrap()
    }
}

/// Set the global theme registry
pub fn set_global_theme_registry(registry: ThemeRegistry) {
    unsafe {
        GLOBAL_THEME_REGISTRY = Some(registry);
    }
}

/// Helper functions for theme operations
pub mod helpers {
    use super::*;

    /// Get the current theme
    pub fn current_theme() -> Option<&'static Theme> {
        global_theme_registry().current()
    }

    /// Set the current theme
    pub fn set_theme(name: &str) -> Result<(), String> {
        global_theme_registry().set_current(name)
    }

    /// Get a text style from the current theme
    pub fn theme_text_style(name: &str) -> Option<TextStyle> {
        current_theme()
            .and_then(|theme| theme.get_text_style(name))
            .map(|style| style.clone())
    }

    /// Get a color from the current theme
    pub fn theme_color(name: &str) -> Option<Color> {
        current_theme()
            .and_then(|theme| theme.get_color(name))
            .map(|color| color.clone())
    }

    /// Get spacing from the current theme
    pub fn theme_spacing(name: &str) -> Option<f32> {
        current_theme()
            .and_then(|theme| theme.get_spacing(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = Theme::new("custom");
        assert_eq!(theme.name, "custom");
        assert!(theme.text_styles.is_empty());
        assert!(theme.colors.is_empty());
    }

    #[test]
    fn test_light_theme_defaults() {
        let theme = Theme::light();
        assert_eq!(theme.name, "light");

        // Check text styles
        assert!(theme.text_styles.contains_key("heading1"));
        assert!(theme.text_styles.contains_key("body"));

        // Check colors
        assert!(theme.colors.contains_key("primary"));
        assert!(theme.colors.contains_key("background"));

        // Check spacing
        assert_eq!(theme.get_spacing("md"), Some(16.0));
    }

    #[test]
    fn test_dark_theme_defaults() {
        let theme = Theme::dark();
        assert_eq!(theme.name, "dark");

        // Check that dark theme has different colors
        let light_theme = Theme::light();
        let light_bg = light_theme.get_color("background").unwrap();
        let dark_bg = theme.get_color("background").unwrap();
        assert_ne!(light_bg, dark_bg);
    }

    #[test]
    fn test_theme_add_and_get() {
        let mut theme = Theme::new("test");

        let style = TextStyle::new().font_family("Test");
        theme.add_text_style("custom", style.clone());

        let color = Color::hex("#123456");
        theme.add_color("custom-color", color.clone());

        theme.add_spacing("custom-space", 42.0);

        assert_eq!(theme.get_text_style("custom"), Some(&style));
        assert_eq!(theme.get_color("custom-color"), Some(&color));
        assert_eq!(theme.get_spacing("custom-space"), Some(42.0));
    }

    #[test]
    fn test_theme_merge() {
        let mut theme1 = Theme::new("base");
        theme1.add_color("color1", Color::hex("#111"));

        let mut theme2 = Theme::new("overlay");
        theme2.add_color("color2", Color::hex("#222"));
        theme2.add_color("color1", Color::hex("#333")); // Override

        theme1.merge(&theme2);

        assert_eq!(theme1.get_color("color1"), Some(&Color::hex("#333")));
        assert_eq!(theme1.get_color("color2"), Some(&Color::hex("#222")));
    }

    #[test]
    fn test_theme_derive() {
        let base = Theme::light();
        let derived = base.derive("derived");

        assert_eq!(derived.name, "derived");
        assert_eq!(derived.colors, base.colors); // Should be copied
    }

    #[test]
    fn test_theme_registry() {
        let mut registry = ThemeRegistry::new();

        let theme = Theme::light();
        registry.register(theme);

        assert!(registry.get("light").is_some());
        assert!(registry.theme_names().contains(&"light".to_string()));
    }

    #[test]
    fn test_theme_registry_current() {
        let mut registry = ThemeRegistry::new();
        registry.register(Theme::light());
        registry.register(Theme::dark());

        registry.set_current("dark").unwrap();
        assert_eq!(registry.current().unwrap().name, "dark");

        assert!(registry.set_current("nonexistent").is_err());
    }

    #[test]
    fn test_theme_registry_with_defaults() {
        let registry = ThemeRegistry::with_defaults();

        assert!(registry.get("light").is_some());
        assert!(registry.get("dark").is_some());
        assert_eq!(registry.current().unwrap().name, "light");
    }

    #[test]
    fn test_global_theme_registry() {
        // Clear any existing registry
        unsafe { GLOBAL_THEME_REGISTRY = None };

        let registry = global_theme_registry();
        assert!(registry.get("light").is_some());
        assert!(registry.get("dark").is_some());
    }

    #[test]
    fn test_theme_helpers() {
        // Clear any existing registry
        unsafe { GLOBAL_THEME_REGISTRY = None };

        // Set current theme to light
        helpers::set_theme("light").unwrap();

        // Test getting theme values
        let primary_color = helpers::theme_color("primary");
        assert!(primary_color.is_some());

        let spacing = helpers::theme_spacing("md");
        assert_eq!(spacing, Some(16.0));
    }
}