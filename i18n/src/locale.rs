/*!
Locale management for internationalization.
*/

use std::fmt;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;
use crate::error::{I18nError, Result};

/// Represents a locale identifier (e.g., "en-US", "es-ES", "pt-BR")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    /// The underlying language identifier
    langid: LanguageIdentifier,
    /// String representation for caching
    string_repr: String,
}

impl Locale {
    /// Create a new locale from a string
    pub fn from<S: AsRef<str>>(locale_str: S) -> Result<Self> {
        let langid = LanguageIdentifier::from_str(locale_str.as_ref())
            .map_err(|_| I18nError::unsupported_locale(locale_str.as_ref()))?;

        let string_repr = langid.to_string();

        Ok(Self { langid, string_repr })
    }

    /// Create a locale from language and region
    pub fn from_parts(language: &str, region: Option<&str>) -> Result<Self> {
        let mut langid = LanguageIdentifier::from_str(language)
            .map_err(|_| I18nError::unsupported_locale(language))?;

        if let Some(region) = region {
            langid.region = Some(region.parse().map_err(|_| {
                I18nError::unsupported_locale(format!("{}-{}", language, region))
            })?);
        }

        let string_repr = langid.to_string();

        Ok(Self { langid, string_repr })
    }

    /// Get the language part (e.g., "en" from "en-US")
    pub fn language(&self) -> &str {
        self.langid.language.as_str()
    }

    /// Get the region part if present (e.g., "US" from "en-US")
    pub fn region(&self) -> Option<&str> {
        self.langid.region.as_ref().map(|r| r.as_str())
    }

    /// Get the base locale (language only, without region)
    pub fn base_locale(&self) -> Self {
        let mut base_langid = self.langid.clone();
        base_langid.region = None;
        let string_repr = base_langid.to_string();

        Self {
            langid: base_langid,
            string_repr,
        }
    }

    /// Check if this locale matches another (considering fallbacks)
    pub fn matches(&self, other: &Locale) -> bool {
        // Exact match
        if self == other {
            return true;
        }

        // Language match (ignoring region)
        if self.language() == other.language() {
            return true;
        }

        false
    }

    /// Get fallback locales in order of preference
    pub fn fallback_chain(&self) -> Vec<Locale> {
        let mut chain = vec![self.clone()];

        // Add base locale if different
        let base = self.base_locale();
        if base != *self {
            chain.push(base);
        }

        // Add English as ultimate fallback if not already included
        if self.language() != "en" {
            if let Ok(en) = Locale::from("en") {
                chain.push(en);
            }
        }

        chain
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string_repr)
    }
}

impl FromStr for Locale {
    type Err = I18nError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from(s)
    }
}

impl AsRef<str> for Locale {
    fn as_ref(&self) -> &str {
        &self.string_repr
    }
}

/// Configuration for locale-specific formatting
#[derive(Debug, Clone)]
pub struct LocaleConfig {
    /// Date format patterns
    pub date_formats: DateFormats,
    /// Number format patterns
    pub number_formats: NumberFormats,
    /// Currency format patterns
    pub currency_formats: CurrencyFormats,
    /// List separator patterns
    pub list_formats: ListFormats,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            date_formats: DateFormats::default(),
            number_formats: NumberFormats::default(),
            currency_formats: CurrencyFormats::default(),
            list_formats: ListFormats::default(),
        }
    }
}

/// Date format configurations
#[derive(Debug, Clone)]
pub struct DateFormats {
    pub short: String,
    pub medium: String,
    pub long: String,
    pub full: String,
}

impl Default for DateFormats {
    fn default() -> Self {
        Self {
            short: "%Y-%m-%d".to_string(),
            medium: "%b %e, %Y".to_string(),
            long: "%B %e, %Y".to_string(),
            full: "%A, %B %e, %Y".to_string(),
        }
    }
}

/// Number format configurations
#[derive(Debug, Clone)]
pub struct NumberFormats {
    pub decimal_separator: String,
    pub group_separator: String,
    pub group_size: usize,
}

impl Default for NumberFormats {
    fn default() -> Self {
        Self {
            decimal_separator: ".".to_string(),
            group_separator: ",".to_string(),
            group_size: 3,
        }
    }
}

/// Currency format configurations
#[derive(Debug, Clone)]
pub struct CurrencyFormats {
    pub symbol: String,
    pub symbol_position: CurrencyPosition,
    pub decimal_digits: usize,
}

impl Default for CurrencyFormats {
    fn default() -> Self {
        Self {
            symbol: "$".to_string(),
            symbol_position: CurrencyPosition::Before,
            decimal_digits: 2,
        }
    }
}

/// Currency symbol position
#[derive(Debug, Clone)]
pub enum CurrencyPosition {
    Before,
    After,
}

/// List format configurations
#[derive(Debug, Clone)]
pub struct ListFormats {
    pub and_separator: String,
    pub or_separator: String,
}

impl Default for ListFormats {
    fn default() -> Self {
        Self {
            and_separator: ", ".to_string(),
            or_separator: " or ".to_string(),
        }
    }
}

/// Manager for locale configurations
#[derive(Debug)]
pub struct LocaleManager {
    configs: std::collections::HashMap<Locale, LocaleConfig>,
}

impl LocaleManager {
    /// Create a new locale manager
    pub fn new() -> Self {
        Self {
            configs: std::collections::HashMap::new(),
        }
    }

    /// Add or update locale configuration
    pub fn set_config(&mut self, locale: Locale, config: LocaleConfig) {
        self.configs.insert(locale, config);
    }

    /// Get locale configuration, falling back to default if not found
    pub fn get_config(&self, locale: &Locale) -> &LocaleConfig {
        self.configs.get(locale).unwrap_or(&LocaleConfig::default())
    }

    /// Get configuration for locale with fallback chain
    pub fn get_config_with_fallback(&self, locale: &Locale) -> &LocaleConfig {
        for fallback_locale in locale.fallback_chain() {
            if let Some(config) = self.configs.get(&fallback_locale) {
                return config;
            }
        }
        &LocaleConfig::default()
    }
}

impl Default for LocaleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_creation() {
        let locale = Locale::from("en-US").unwrap();
        assert_eq!(locale.language(), "en");
        assert_eq!(locale.region(), Some("US"));
        assert_eq!(locale.to_string(), "en-US");
    }

    #[test]
    fn test_locale_base() {
        let locale = Locale::from("en-US").unwrap();
        let base = locale.base_locale();
        assert_eq!(base.language(), "en");
        assert_eq!(base.region(), None);
        assert_eq!(base.to_string(), "en");
    }

    #[test]
    fn test_locale_fallback_chain() {
        let locale = Locale::from("en-US").unwrap();
        let chain = locale.fallback_chain();
        assert_eq!(chain.len(), 2); // en-US, en
        assert_eq!(chain[0].to_string(), "en-US");
        assert_eq!(chain[1].to_string(), "en");
    }

    #[test]
    fn test_locale_matches() {
        let en_us = Locale::from("en-US").unwrap();
        let en_gb = Locale::from("en-GB").unwrap();
        let es = Locale::from("es").unwrap();

        assert!(en_us.matches(&en_gb)); // Same language
        assert!(!en_us.matches(&es));   // Different language
    }
}