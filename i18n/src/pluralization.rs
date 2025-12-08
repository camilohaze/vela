/*!
Pluralization rules for different locales.
*/

use std::collections::HashMap;
use crate::locale::Locale;

/// Plural category for a given count
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralCategory {
    /// Zero items (0)
    Zero,
    /// One item (1)
    One,
    /// Two items (2)
    Two,
    /// Few items (e.g., 3-10 in some languages)
    Few,
    /// Many items (e.g., 11-99 in some languages)
    Many,
    /// Other/unknown (default)
    Other,
}

impl PluralCategory {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }
}

/// Pluralization rules engine
#[derive(Debug)]
pub struct PluralizationRules {
    /// Rules for different locales
    rules: HashMap<String, Box<dyn PluralRule + Send + Sync>>,
}

impl PluralizationRules {
    /// Create a new pluralization rules engine with default rules
    pub fn new() -> Self {
        let mut rules = HashMap::new();

        // English rules
        rules.insert("en".to_string(), Box::new(EnglishPluralRule) as Box<dyn PluralRule + Send + Sync>);
        rules.insert("en-US".to_string(), Box::new(EnglishPluralRule));
        rules.insert("en-GB".to_string(), Box::new(EnglishPluralRule));

        // Spanish rules
        rules.insert("es".to_string(), Box::new(SpanishPluralRule));
        rules.insert("es-ES".to_string(), Box::new(SpanishPluralRule));
        rules.insert("es-MX".to_string(), Box::new(SpanishPluralRule));

        // French rules
        rules.insert("fr".to_string(), Box::new(FrenchPluralRule));
        rules.insert("fr-FR".to_string(), Box::new(FrenchPluralRule));

        // German rules
        rules.insert("de".to_string(), Box::new(GermanPluralRule));
        rules.insert("de-DE".to_string(), Box::new(GermanPluralRule));

        // Portuguese rules
        rules.insert("pt".to_string(), Box::new(PortuguesePluralRule));
        rules.insert("pt-BR".to_string(), Box::new(PortuguesePluralRule));
        rules.insert("pt-PT".to_string(), Box::new(PortuguesePluralRule));

        // Arabic rules (more complex)
        rules.insert("ar".to_string(), Box::new(ArabicPluralRule));

        // Russian rules
        rules.insert("ru".to_string(), Box::new(RussianPluralRule));

        // Japanese rules (simple - no plurals)
        rules.insert("ja".to_string(), Box::new(JapanesePluralRule));

        // Chinese rules (simple - no plurals)
        rules.insert("zh".to_string(), Box::new(ChinesePluralRule));
        rules.insert("zh-CN".to_string(), Box::new(ChinesePluralRule));
        rules.insert("zh-TW".to_string(), Box::new(ChinesePluralRule));

        Self { rules }
    }

    /// Get the plural category for a count in the given locale
    pub fn get_category(&self, count: i64, locale: &Locale) -> PluralCategory {
        // Try exact locale match first
        if let Some(rule) = self.rules.get(&locale.to_string()) {
            return rule.category(count);
        }

        // Try language-only match
        if let Some(rule) = self.rules.get(locale.language()) {
            return rule.category(count);
        }

        // Fallback to English rules
        if let Some(rule) = self.rules.get("en") {
            return rule.category(count);
        }

        // Ultimate fallback
        PluralCategory::Other
    }

    /// Add custom plural rules for a locale
    pub fn add_rule(&mut self, locale: &str, rule: Box<dyn PluralRule + Send + Sync>) {
        self.rules.insert(locale.to_string(), rule);
    }

    /// Get all supported locales
    pub fn supported_locales(&self) -> Vec<String> {
        self.rules.keys().cloned().collect()
    }
}

impl Default for PluralizationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for plural rules
pub trait PluralRule {
    /// Get the plural category for a given count
    fn category(&self, count: i64) -> PluralCategory;
}

/// English plural rules (simple: singular vs plural)
pub struct EnglishPluralRule;

impl PluralRule for EnglishPluralRule {
    fn category(&self, count: i64) -> PluralCategory {
        match count {
            0 => PluralCategory::Zero,
            1 => PluralCategory::One,
            _ => PluralCategory::Other,
        }
    }
}

/// Spanish plural rules
pub struct SpanishPluralRule;

impl PluralRule for SpanishPluralRule {
    fn category(&self, count: i64) -> PluralCategory {
        match count {
            0 => PluralCategory::Zero,
            1 => PluralCategory::One,
            _ => PluralCategory::Other,
        }
    }
}

/// French plural rules
pub struct FrenchPluralRule;

impl PluralRule for FrenchPluralRule {
    fn category(&self, count: i64) -> PluralCategory {
    match count {
        0 => PluralCategory::Zero,
        1 => PluralCategory::One,
        _ => PluralCategory::Other,
    }
}
}

/// German plural rules (more complex)
pub struct GermanPluralRule;

impl PluralRule for GermanPluralRule {
    fn category(&self, count: i64) -> PluralCategory {
        match count {
            0 => PluralCategory::Zero,
            1 => PluralCategory::One,
            _ => PluralCategory::Other,
        }
    }
}

/// Portuguese plural rules
pub struct PortuguesePluralRule;

impl PluralRule for PortuguesePluralRule {
    fn category(&self, count: i64) -> PluralCategory {
        match count {
            0 => PluralCategory::Zero,
            1 => PluralCategory::One,
            _ => PluralCategory::Other,
        }
    }
}

/// Arabic plural rules (complex)
pub struct ArabicPluralRule;

impl PluralRule for ArabicPluralRule {
    fn category(&self, count: i64) -> PluralCategory {
        match count {
            0 => PluralCategory::Zero,
            1 => PluralCategory::One,
            2 => PluralCategory::Two,
            n if (3..=10).contains(&n) => PluralCategory::Few,
            n if (11..=99).contains(&n) => PluralCategory::Many,
            _ => PluralCategory::Other,
        }
    }
}

/// Russian plural rules
pub struct RussianPluralRule;

impl PluralRule for RussianPluralRule {
    fn category(&self, count: i64) -> PluralCategory {
        let n = count.abs();

        // Russian plural rules are based on the last two digits
        let last_two = n % 100;
        let last_digit = n % 10;

        match () {
            _ if last_two >= 11 && last_two <= 19 => PluralCategory::Many,
            _ if last_digit == 1 => PluralCategory::One,
            _ if (2..=4).contains(&last_digit) => PluralCategory::Few,
            _ if n == 0 => PluralCategory::Zero,
            _ => PluralCategory::Many,
        }
    }
}

/// Japanese plural rules (no distinction)
pub struct JapanesePluralRule;

impl PluralRule for JapanesePluralRule {
    fn category(&self, _count: i64) -> PluralCategory {
        // Japanese doesn't have plural forms
        PluralCategory::Other
    }
}

/// Chinese plural rules (no distinction)
pub struct ChinesePluralRule;

impl PluralRule for ChinesePluralRule {
    fn category(&self, _count: i64) -> PluralCategory {
        // Chinese doesn't have plural forms
        PluralCategory::Other
    }
}

/// Utility functions for pluralization
pub mod utils {
    use super::*;

    /// Get plural form from ICU-like syntax
    pub fn parse_plural_spec(spec: &str, count: i64, locale: &Locale, rules: &PluralizationRules) -> Option<String> {
        let category = rules.get_category(count, locale);
        let category_str = category.as_str();

        // Parse patterns like: one{# item} other{# items}
        let option_pattern = regex::Regex::new(r"(\w+)\{([^}]*)\}").ok()?;

        for cap in option_pattern.captures_iter(spec) {
            let form = &cap[1];
            let text = &cap[2];

            if form == category_str {
                return Some(text.to_string());
            }
        }

        // Fallback to "other" if available
        if category_str != "other" {
            for cap in option_pattern.captures_iter(spec) {
                let form = &cap[1];
                let text = &cap[2];

                if form == "other" {
                    return Some(text.to_string());
                }
            }
        }

        None
    }

    /// Format a count with plural suffix
    pub fn format_plural(count: i64, singular: &str, plural: &str, locale: &Locale, rules: &PluralizationRules) -> String {
        let category = rules.get_category(count, locale);

        match category {
            PluralCategory::One => format!("{} {}", count, singular),
            _ => format!("{} {}", count, plural),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_plural_rules() {
        let rules = PluralizationRules::new();
        let locale = Locale::from("en").unwrap();

        assert_eq!(rules.get_category(0, &locale), PluralCategory::Zero);
        assert_eq!(rules.get_category(1, &locale), PluralCategory::One);
        assert_eq!(rules.get_category(2, &locale), PluralCategory::Other);
        assert_eq!(rules.get_category(10, &locale), PluralCategory::Other);
    }

    #[test]
    fn test_arabic_plural_rules() {
        let rules = PluralizationRules::new();
        let locale = Locale::from("ar").unwrap();

        assert_eq!(rules.get_category(0, &locale), PluralCategory::Zero);
        assert_eq!(rules.get_category(1, &locale), PluralCategory::One);
        assert_eq!(rules.get_category(2, &locale), PluralCategory::Two);
        assert_eq!(rules.get_category(3, &locale), PluralCategory::Few);
        assert_eq!(rules.get_category(11, &locale), PluralCategory::Many);
        assert_eq!(rules.get_category(100, &locale), PluralCategory::Other);
    }

    #[test]
    fn test_russian_plural_rules() {
        let rules = PluralizationRules::new();
        let locale = Locale::from("ru").unwrap();

        assert_eq!(rules.get_category(1, &locale), PluralCategory::One);
        assert_eq!(rules.get_category(2, &locale), PluralCategory::Few);
        assert_eq!(rules.get_category(5, &locale), PluralCategory::Many);
        assert_eq!(rules.get_category(11, &locale), PluralCategory::Many);
        assert_eq!(rules.get_category(21, &locale), PluralCategory::One);
    }

    #[test]
    fn test_japanese_no_plurals() {
        let rules = PluralizationRules::new();
        let locale = Locale::from("ja").unwrap();

        // Japanese should always return Other (no plural distinction)
        assert_eq!(rules.get_category(0, &locale), PluralCategory::Other);
        assert_eq!(rules.get_category(1, &locale), PluralCategory::Other);
        assert_eq!(rules.get_category(100, &locale), PluralCategory::Other);
    }

    #[test]
    fn test_fallback_to_language() {
        let rules = PluralizationRules::new();
        let locale = Locale::from("en-US").unwrap();

        // Should use English rules for en-US
        assert_eq!(rules.get_category(1, &locale), PluralCategory::One);
    }

    #[test]
    fn test_fallback_to_english() {
        let rules = PluralizationRules::new();
        let locale = Locale::from("unknown-lang").unwrap();

        // Should fallback to English rules
        assert_eq!(rules.get_category(1, &locale), PluralCategory::One);
    }

    #[test]
    fn test_plural_category_as_str() {
        assert_eq!(PluralCategory::Zero.as_str(), "zero");
        assert_eq!(PluralCategory::One.as_str(), "one");
        assert_eq!(PluralCategory::Two.as_str(), "two");
        assert_eq!(PluralCategory::Few.as_str(), "few");
        assert_eq!(PluralCategory::Many.as_str(), "many");
        assert_eq!(PluralCategory::Other.as_str(), "other");
    }

    #[test]
    fn test_parse_plural_spec() {
        let rules = PluralizationRules::new();
        let locale = Locale::from("en").unwrap();

        let spec = "one{# item} other{# items}";
        assert_eq!(utils::parse_plural_spec(spec, 1, &locale, &rules), Some("# item".to_string()));
        assert_eq!(utils::parse_plural_spec(spec, 2, &locale, &rules), Some("# items".to_string()));
    }

    #[test]
    fn test_format_plural() {
        let rules = PluralizationRules::new();
        let locale = Locale::from("en").unwrap();

        assert_eq!(utils::format_plural(1, "item", "items", &locale, &rules), "1 item");
        assert_eq!(utils::format_plural(2, "item", "items", &locale, &rules), "2 items");
    }
}