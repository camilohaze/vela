//! Style resolution engine
//!
//! This module provides the style resolution engine that combines
//! theme, named, and inline styles with proper precedence and caching.

use super::types::*;
use super::registry::*;
use super::theme::*;
use super::composition::*;
use std::collections::HashMap;
use crate::TextStyle;
use std::hash::{Hash, Hasher};

/// Style resolution engine with caching
#[derive(Clone, Debug)]
pub struct StyleResolver {
    cache: HashMap<u64, ResolvedStyle>,
}

impl StyleResolver {
    /// Create a new style resolver
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Resolve styles from multiple sources with caching
    pub fn resolve(&mut self, style_refs: &[StyleRef]) -> ResolvedStyle {
        let cache_key = self.compute_cache_key(style_refs);

        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        let resolved = self.do_resolve(style_refs);
        self.cache.insert(cache_key, resolved.clone());
        resolved
    }

    /// Force resolve without caching (for testing)
    pub fn resolve_uncached(&self, style_refs: &[StyleRef]) -> ResolvedStyle {
        self.do_resolve(style_refs)
    }

    /// Clear the resolution cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Internal resolution logic
    fn do_resolve(&self, style_refs: &[StyleRef]) -> ResolvedStyle {
        let mut resolved = ResolvedStyle::default();

        // Collect styles by precedence
        let mut theme_styles = Vec::new();
        let mut named_styles = Vec::new();
        let mut inline_styles = Vec::new();

        for style_ref in style_refs {
            match style_ref {
                StyleRef::Named(name) => {
                    if let Some(style) = global_registry().get(name) {
                        named_styles.push(ComposedStyle::named(style.clone(), name));
                    }
                }
                StyleRef::Inline(style) => {
                    inline_styles.push(ComposedStyle::inline(style.clone(), "inline"));
                }
            }
        }

        // Add current theme styles if available
        if let Some(theme) = global_theme_registry().current() {
            // Add theme text styles (these would be referenced by name)
            for (name, style) in &theme.text_styles {
                theme_styles.push(ComposedStyle::theme(style.clone(), format!("{}.{}", theme.name, name)));
            }
        }

        // Combine all styles with precedence: theme -> named -> inline
        let mut all_styles = Vec::new();
        all_styles.extend(theme_styles);
        all_styles.extend(named_styles);
        all_styles.extend(inline_styles);

        // Compose with precedence
        let final_style = compose_styles(&all_styles);

        // Convert to resolved style
        ResolvedStyle {
            font_family: final_style.font_family,
            font_size: final_style.font_size,
            font_weight: final_style.font_weight,
            font_style: final_style.font_style,
            color: final_style.color,
            background_color: final_style.background_color,
            letter_spacing: final_style.letter_spacing,
            word_spacing: final_style.word_spacing,
            line_height: final_style.line_height,
            text_decoration: final_style.text_decoration,
            text_align: final_style.text_align,
            text_transform: final_style.text_transform,
        }
    }

    /// Compute cache key for style references
    fn compute_cache_key(&self, style_refs: &[StyleRef]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        // Sort style refs for consistent hashing
        let mut sorted_refs = style_refs.to_vec();
        sorted_refs.sort_by(|a, b| match (a, b) {
            (StyleRef::Named(name_a), StyleRef::Named(name_b)) => name_a.cmp(name_b),
            (StyleRef::Inline(_), StyleRef::Named(_)) => std::cmp::Ordering::Greater,
            (StyleRef::Named(_), StyleRef::Inline(_)) => std::cmp::Ordering::Less,
            (StyleRef::Inline(style_a), StyleRef::Inline(style_b)) => {
                // Simple comparison - in practice you'd want a more sophisticated hash
                format!("{:?}", style_a).cmp(&format!("{:?}", style_b))
            }
        });

        for style_ref in &sorted_refs {
            match style_ref {
                StyleRef::Named(name) => {
                    0u8.hash(&mut hasher);
                    name.hash(&mut hasher);
                }
                StyleRef::Inline(style) => {
                    1u8.hash(&mut hasher);
                    // Hash the style properties using string representation
                    format!("{:?}", style.font_family).hash(&mut hasher);
                    format!("{:?}", style.font_size).hash(&mut hasher);
                    format!("{:?}", style.font_weight).hash(&mut hasher);
                    format!("{:?}", style.font_style).hash(&mut hasher);
                    format!("{:?}", style.color).hash(&mut hasher);
                    format!("{:?}", style.background_color).hash(&mut hasher);
                    format!("{:?}", style.letter_spacing).hash(&mut hasher);
                    format!("{:?}", style.word_spacing).hash(&mut hasher);
                    format!("{:?}", style.line_height).hash(&mut hasher);
                    format!("{:?}", style.text_decoration).hash(&mut hasher);
                    format!("{:?}", style.text_align).hash(&mut hasher);
                    format!("{:?}", style.text_transform).hash(&mut hasher);
                }
            }
        }

        // Include current theme in cache key
        if let Some(theme) = global_theme_registry().current() {
            theme.name.hash(&mut hasher);
        }

        hasher.finish()
    }
}

impl Default for StyleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Contextual style resolver with inheritance
#[derive(Clone, Debug)]
pub struct ContextualStyleResolver {
    resolver: StyleResolver,
    inherited_styles: Vec<TextStyle>,
}

impl ContextualStyleResolver {
    /// Create a new contextual resolver
    pub fn new() -> Self {
        Self {
            resolver: StyleResolver::new(),
            inherited_styles: Vec::new(),
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

    /// Resolve styles with inheritance context
    pub fn resolve_with_context(&mut self, style_refs: &[StyleRef]) -> ResolvedStyle {
        let mut contextual_refs = Vec::new();

        // Add inherited styles as inline styles (lower precedence)
        for inherited in &self.inherited_styles {
            contextual_refs.push(StyleRef::Inline(inherited.clone()));
        }

        // Add the requested styles
        contextual_refs.extend_from_slice(style_refs);

        self.resolver.resolve(&contextual_refs)
    }

    /// Get the current inheritance chain
    pub fn inheritance_chain(&self) -> &[TextStyle] {
        &self.inherited_styles
    }

    /// Clear the inheritance chain
    pub fn clear_inheritance(&mut self) {
        self.inherited_styles.clear();
    }
}

impl Default for ContextualStyleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Global style resolver instance
static mut GLOBAL_RESOLVER: Option<StyleResolver> = None;

/// Get the global style resolver
pub fn global_resolver() -> &'static mut StyleResolver {
    unsafe {
        if GLOBAL_RESOLVER.is_none() {
            GLOBAL_RESOLVER = Some(StyleResolver::new());
        }
        GLOBAL_RESOLVER.as_mut().unwrap()
    }
}

/// Helper functions for style resolution
pub mod helpers {
    use super::*;

    /// Resolve styles using the global resolver
    pub fn resolve_styles(style_refs: &[StyleRef]) -> ResolvedStyle {
        global_resolver().resolve(style_refs)
    }

    /// Resolve a single named style
    pub fn resolve_named_style(name: &str) -> Option<ResolvedStyle> {
        let style_ref = StyleRef::Named(name.to_string());
        Some(global_resolver().resolve(&[style_ref]))
    }

    /// Resolve an inline style
    pub fn resolve_inline_style(style: TextStyle) -> ResolvedStyle {
        let style_ref = StyleRef::Inline(style);
        global_resolver().resolve(&[style_ref])
    }

    /// Create a contextual resolver
    pub fn contextual_resolver() -> ContextualStyleResolver {
        ContextualStyleResolver::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_resolver_basic() {
        let mut resolver = StyleResolver::new();

        let style_refs = vec![
            StyleRef::Inline(TextStyle::new().font_family("Inter")),
            StyleRef::Inline(TextStyle::new().font_size(FontSize::Px(16.0))),
        ];

        let resolved = resolver.resolve(&style_refs);

        assert_eq!(resolved.font_family, Some("Inter".to_string()));
        assert_eq!(resolved.font_size, Some(FontSize::Px(16.0)));
    }

    #[test]
    fn test_style_resolver_caching() {
        let mut resolver = StyleResolver::new();

        let style_refs = vec![
            StyleRef::Inline(TextStyle::new().color(Color::hex("#333"))),
        ];

        // First resolution
        let resolved1 = resolver.resolve(&style_refs);
        assert_eq!(resolver.cache_size(), 1);

        // Second resolution (should use cache)
        let resolved2 = resolver.resolve(&style_refs);
        assert_eq!(resolved1, resolved2);
        assert_eq!(resolver.cache_size(), 1); // Still 1 entry
    }

    #[test]
    fn test_style_resolver_clear_cache() {
        let mut resolver = StyleResolver::new();

        let style_refs = vec![
            StyleRef::Inline(TextStyle::new().font_family("Test")),
        ];

        resolver.resolve(&style_refs);
        assert_eq!(resolver.cache_size(), 1);

        resolver.clear_cache();
        assert_eq!(resolver.cache_size(), 0);
    }

    #[test]
    fn test_contextual_resolver_inheritance() {
        let mut resolver = ContextualStyleResolver::new();

        // Push inherited style
        let inherited = TextStyle::new().font_family("Inherited");
        resolver.push_inherited(inherited);

        // Resolve with additional styles
        let style_refs = vec![
            StyleRef::Inline(TextStyle::new().font_size(FontSize::Px(16.0))),
        ];

        let resolved = resolver.resolve_with_context(&style_refs);

        assert_eq!(resolved.font_family, Some("Inherited".to_string()));
        assert_eq!(resolved.font_size, Some(FontSize::Px(16.0)));
    }

    #[test]
    fn test_contextual_resolver_multiple_inheritance() {
        let mut resolver = ContextualStyleResolver::new();

        // Push multiple inherited styles
        resolver.push_inherited(TextStyle::new().font_family("Parent"));
        resolver.push_inherited(TextStyle::new().color(Color::hex("#333")));

        let resolved = resolver.resolve_with_context(&[]);

        assert_eq!(resolved.font_family, Some("Parent".to_string()));
        assert_eq!(resolved.color, Some(Color::hex("#333")));
    }

    #[test]
    fn test_contextual_resolver_pop_inherited() {
        let mut resolver = ContextualStyleResolver::new();

        resolver.push_inherited(TextStyle::new().font_family("First"));
        resolver.push_inherited(TextStyle::new().font_family("Second"));

        assert_eq!(resolver.inheritance_chain().len(), 2);

        resolver.pop_inherited();
        assert_eq!(resolver.inheritance_chain().len(), 1);

        let resolved = resolver.resolve_with_context(&[]);
        assert_eq!(resolved.font_family, Some("First".to_string()));
    }

    #[test]
    fn test_helpers_resolve_styles() {
        let style_refs = vec![
            StyleRef::Inline(TextStyle::new().font_family("HelperTest")),
        ];

        let resolved = helpers::resolve_styles(&style_refs);
        assert_eq!(resolved.font_family, Some("HelperTest".to_string()));
    }

    #[test]
    fn test_helpers_resolve_inline_style() {
        let style = TextStyle::new().color(Color::hex("#123"));
        let resolved = helpers::resolve_inline_style(style);

        assert_eq!(resolved.color, Some(Color::hex("#123")));
    }

    #[test]
    fn test_cache_key_consistency() {
        let resolver = StyleResolver::new();

        let refs1 = vec![
            StyleRef::Inline(TextStyle::new().font_family("Test")),
            StyleRef::Inline(TextStyle::new().font_size(FontSize::Px(16.0))),
        ];

        let refs2 = vec![
            StyleRef::Inline(TextStyle::new().font_size(FontSize::Px(16.0))),
            StyleRef::Inline(TextStyle::new().font_family("Test")),
        ];

        let key1 = resolver.compute_cache_key(&refs1);
        let key2 = resolver.compute_cache_key(&refs2);

        // Should be the same due to sorting
        assert_eq!(key1, key2);
    }
}