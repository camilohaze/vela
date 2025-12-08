/*!
String interpolation with variables and pluralization support.
*/

use std::collections::HashMap;
use regex::Regex;
use crate::error::{I18nError, Result};
use crate::locale::Locale;
use icu_plurals::{PluralRules, PluralRuleType};
use icu_locid::Locale as IcuLocale;
use icu_decimal::FixedDecimalFormatter;
use icu_datetime::DateTimeFormatter;
use std::sync::Arc;

/// Context for interpolation operations
#[derive(Debug, Clone)]
pub struct InterpolationContext<'a> {
    /// Variables available for interpolation
    pub variables: HashMap<String, String>,
    /// Current locale for pluralization
    pub locale: &'a Locale,
}

impl<'a> InterpolationContext<'a> {
    /// Create a new interpolation context
    pub fn new(locale: &'a Locale) -> Self {
        Self {
            variables: HashMap::new(),
            locale,
        }
    }

    /// Add a variable to the context
    pub fn with_variable<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Add multiple variables
    pub fn with_variables(mut self, variables: HashMap<String, String>) -> Self {
        self.variables.extend(variables);
        self
    }
}

/// Advanced string interpolator with ICU support
#[derive(Debug)]
pub struct Interpolator {
    /// Variable resolver for ${variable} and {{variable}} syntax
    variable_resolver: VariableResolver,
    /// Pluralization engine with ICU support
    pluralization_engine: PluralizationEngine,
    /// Select engine for conditional text
    select_engine: SelectEngine,
    /// Maximum recursion depth for nested interpolation
    max_recursion_depth: usize,
}

/// Variable resolver with recursive interpolation support
#[derive(Debug)]
struct VariableResolver {
    /// Regex for ${variable} syntax
    dollar_syntax: Regex,
    /// Regex for {{variable}} syntax
    brace_syntax: Regex,
    /// Regex for {{variable, format}} syntax
    formatted_syntax: Regex,
}

/// Pluralization engine using ICU plural rules
#[derive(Debug)]
struct PluralizationEngine {
    /// ICU plural rules for cardinal numbers
    cardinal_rules: HashMap<String, Arc<PluralRules>>,
    /// ICU plural rules for ordinal numbers
    ordinal_rules: HashMap<String, Arc<PluralRules>>,
    /// Regex for parsing plural patterns
    plural_pattern: Regex,
}

/// Select engine for conditional text selection
#[derive(Debug)]
struct SelectEngine {
    /// Regex for parsing select patterns
    select_pattern: Regex,
}

impl Interpolator {
    /// Create a new interpolator with ICU support
    pub fn new() -> Result<Self> {
        Ok(Self {
            variable_resolver: VariableResolver::new()?,
            pluralization_engine: PluralizationEngine::new()?,
            select_engine: SelectEngine::new()?,
            max_recursion_depth: 10, // Prevent infinite recursion
        })
    }

    /// Create interpolator with custom recursion depth
    pub fn with_max_recursion_depth(max_depth: usize) -> Result<Self> {
        let mut interpolator = Self::new()?;
        interpolator.max_recursion_depth = max_depth;
        Ok(interpolator)
    }

    /// Interpolate a string with the given context
    pub fn interpolate(&self, template: &str, context: &InterpolationContext) -> Result<String> {
        self.interpolate_with_depth(template, context, 0)
    }

    /// Internal interpolation with recursion depth tracking
    fn interpolate_with_depth(&self, template: &str, context: &InterpolationContext, depth: usize) -> Result<String> {
        if depth > self.max_recursion_depth {
            return Err(I18nError::interpolation_error(
                format!("Maximum interpolation recursion depth ({}) exceeded", self.max_recursion_depth)
            ));
        }

        let mut result = template.to_string();

        // Process in order: select -> plural -> variables (most specific first)
        result = self.select_engine.interpolate(&result, context)?;
        result = self.pluralization_engine.interpolate(&result, context)?;
        result = self.variable_resolver.interpolate(&result, context, depth, self)?;

        Ok(result)
    }
}

impl VariableResolver {
    /// Create a new variable resolver
    fn new() -> Result<Self> {
        Ok(Self {
            dollar_syntax: Regex::new(r"\$\{([^}]+)\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid dollar syntax regex: {}", e)))?,
            brace_syntax: Regex::new(r"\{\{([^}]+)\}\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid brace syntax regex: {}", e)))?,
            formatted_syntax: Regex::new(r"\{\{([^,]+),\s*([^}]+)\}\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid formatted syntax regex: {}", e)))?,
        })
    }

    /// Interpolate variables in the template
    fn interpolate(&self, template: &str, context: &InterpolationContext, depth: usize, interpolator: &Interpolator) -> Result<String> {
        let mut result = template.to_string();

        // Handle formatted syntax first (more specific)
        result = self.interpolate_formatted(&result, context, depth, interpolator)?;

        // Handle dollar syntax ${variable}
        result = self.interpolate_dollar_syntax(&result, context, depth, interpolator)?;

        // Handle brace syntax {{variable}}
        result = self.interpolate_brace_syntax(&result, context, depth, interpolator)?;

        Ok(result)
    }

    /// Interpolate ${variable} syntax
    fn interpolate_dollar_syntax(&self, template: &str, context: &InterpolationContext, depth: usize, interpolator: &Interpolator) -> Result<String> {
        let result = self.dollar_syntax.replace_all(template, |caps: &regex::Captures| {
            let var_expr = &caps[1];
            self.resolve_variable(var_expr, context, depth, interpolator)
                .unwrap_or_else(|_| format!("${{{}}}", var_expr))
        });
        Ok(result.to_string())
    }

    /// Interpolate {{variable}} syntax
    fn interpolate_brace_syntax(&self, template: &str, context: &InterpolationContext, depth: usize, interpolator: &Interpolator) -> Result<String> {
        let result = self.brace_syntax.replace_all(template, |caps: &regex::Captures| {
            let var_expr = &caps[1];
            // Skip if it's a formatted expression (handled separately)
            if var_expr.contains(',') {
                return format!("{{{{{}}}}}", var_expr);
            }
            self.resolve_variable(var_expr, context, depth, interpolator)
                .unwrap_or_else(|_| format!("{{{{{}}}}}", var_expr))
        });
        Ok(result.to_string())
    }

    /// Interpolate {{variable, format}} syntax
    fn interpolate_formatted(&self, template: &str, context: &InterpolationContext, depth: usize, interpolator: &Interpolator) -> Result<String> {
        let result = self.formatted_syntax.replace_all(template, |caps: &regex::Captures| {
            let var_name = caps[1].trim();
            let format_spec = caps[2].trim();

            match self.resolve_variable(var_name, context, depth, interpolator) {
                Ok(value) => self.apply_format(&value, format_spec, context),
                Err(_) => format!("{{{{{}, {}}}}}", var_name, format_spec),
            }
        });
        Ok(result.to_string())
    }

    /// Resolve a variable expression (supports dot notation and array access)
    fn resolve_variable(&self, var_expr: &str, context: &InterpolationContext, depth: usize, interpolator: &Interpolator) -> Result<String> {
        // Handle nested property access like user.name or items[0]
        let parts: Vec<&str> = var_expr.split('.').collect();
        let mut current_value = self.get_base_variable(parts[0], context)?;

        // Recursively interpolate the base value first
        current_value = interpolator.interpolate_with_depth(&current_value, context, depth + 1)?;

        // Navigate through properties
        for &part in &parts[1..] {
            current_value = self.navigate_property(&current_value, part)?;
        }

        Ok(current_value)
    }

    /// Get the base variable value
    fn get_base_variable(&self, var_name: &str, context: &InterpolationContext) -> Result<String> {
        context.variables.get(var_name)
            .map(|s| s.clone())
            .ok_or_else(|| I18nError::interpolation_error(format!("Variable '{}' not found", var_name)))
    }

    /// Navigate through object properties or array indices
    fn navigate_property(&self, value: &str, property: &str) -> Result<String> {
        // Try to parse as JSON for object/array access
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(value) {
            return self.navigate_json_property(&json_value, property);
        }

        // If not JSON, treat as simple string (no navigation possible)
        Err(I18nError::interpolation_error(format!("Cannot navigate property '{}' on non-object value", property)))
    }

    /// Navigate JSON properties
    fn navigate_json_property(&self, value: &serde_json::Value, property: &str) -> Result<String> {
        if let Some(index) = self.parse_array_index(property) {
            // Array access like items[0]
            if let Some(array) = value.as_array() {
                if let Some(item) = array.get(index) {
                    return Ok(item.to_string());
                }
            }
        } else {
            // Object property access like user.name
            if let Some(obj) = value.as_object() {
                if let Some(prop_value) = obj.get(property) {
                    return Ok(prop_value.to_string());
                }
            }
        }

        Err(I18nError::interpolation_error(format!("Property '{}' not found", property)))
    }

    /// Parse array index from property like "items[0]"
    fn parse_array_index(&self, property: &str) -> Option<usize> {
        if property.ends_with(']') {
            if let Some(start) = property.find('[') {
                let index_str = &property[start + 1..property.len() - 1];
                index_str.parse::<usize>().ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Apply formatting to a value
    fn apply_format(&self, value: &str, format_spec: &str, context: &InterpolationContext) -> String {
        match format_spec {
            "number" => self.format_number(value, context.locale),
            "currency" => self.format_currency(value, context.locale),
            "date" => self.format_date(value, context.locale),
            "uppercase" => value.to_uppercase(),
            "lowercase" => value.to_lowercase(),
            "capitalize" => self.capitalize(value),
            _ => value.to_string(), // Unknown format, return as-is
        }
    }

    /// Format as number using ICU
    fn format_number(&self, value: &str, locale: &Locale) -> String {
        if let Ok(num) = value.parse::<f64>() {
            // Use ICU decimal formatter
            if let Ok(icu_locale) = IcuLocale::try_from_bytes(locale.id.as_bytes()) {
                if let Ok(formatter) = FixedDecimalFormatter::try_new(&icu_locale.into(), Default::default()) {
                    if let Ok(decimal) = rust_decimal::Decimal::try_from(num) {
                        if let Ok(fixed_decimal) = icu_decimal::FixedDecimal::try_from(decimal) {
                            return formatter.format(&fixed_decimal).to_string();
                        }
                    }
                }
            }
        }
        value.to_string() // Fallback
    }

    /// Format as currency
    fn format_currency(&self, value: &str, locale: &Locale) -> String {
        // Basic currency formatting - could be enhanced with ICU currency formatter
        format!("${}", value)
    }

    /// Format as date using ICU
    fn format_date(&self, value: &str, locale: &Locale) -> String {
        // Basic date formatting - could be enhanced with ICU date formatter
        value.to_string()
    }

    /// Capitalize first letter
    fn capitalize(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

impl PluralizationEngine {
    /// Create a new pluralization engine with ICU support
    fn new() -> Result<Self> {
        Ok(Self {
            cardinal_rules: HashMap::new(),
            ordinal_rules: HashMap::new(),
            plural_pattern: Regex::new(r"\{\{([^,]+),\s*plural,\s*([^}]+)\}\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid plural regex: {}", e)))?,
        })
    }

    /// Interpolate plural patterns using ICU rules
    fn interpolate(&self, template: &str, context: &InterpolationContext) -> Result<String> {
        let result = self.plural_pattern.replace_all(template, |caps: &regex::Captures| {
            let var_expr = caps[1].trim();
            let plural_spec = caps[2].trim();

            // Get the count value (supporting expressions)
            let count = self.resolve_count(var_expr, context);

            // Get plural category using ICU
            let plural_category = self.get_plural_category(count, context.locale, PluralRuleType::Cardinal);

            // Extract the appropriate plural form
            self.extract_plural_form(plural_spec, &plural_category)
                .unwrap_or_else(|| format!("{{{{{}}}}}", &caps[0]))
        });

        Ok(result.to_string())
    }

    /// Resolve count value from variable expression
    fn resolve_count(&self, var_expr: &str, context: &InterpolationContext) -> i64 {
        // Try to get as direct variable first
        if let Some(value) = context.variables.get(var_expr) {
            if let Ok(num) = value.parse::<i64>() {
                return num;
            }
        }

        // Try to evaluate as expression (basic support for now)
        // Could be extended to support more complex expressions
        0 // Default fallback
    }

    /// Get plural category using ICU plural rules
    fn get_plural_category(&self, count: i64, locale: &Locale, rule_type: PluralRuleType) -> String {
        let locale_key = locale.id.clone();

        let rules = match rule_type {
            PluralRuleType::Cardinal => {
                if !self.cardinal_rules.contains_key(&locale_key) {
                    if let Ok(icu_locale) = IcuLocale::try_from_bytes(locale.id.as_bytes()) {
                        if let Ok(rules) = PluralRules::try_new(&icu_locale.into(), rule_type) {
                            self.cardinal_rules.insert(locale_key.clone(), Arc::new(rules));
                        }
                    }
                }
                self.cardinal_rules.get(&locale_key)
            }
            PluralRuleType::Ordinal => {
                if !self.ordinal_rules.contains_key(&locale_key) {
                    if let Ok(icu_locale) = IcuLocale::try_from_bytes(locale.id.as_bytes()) {
                        if let Ok(rules) = PluralRules::try_new(&icu_locale.into(), rule_type) {
                            self.ordinal_rules.insert(locale_key.clone(), Arc::new(rules));
                        }
                    }
                }
                self.ordinal_rules.get(&locale_key)
            }
        };

        if let Some(rules) = rules {
            if let Ok(category) = rules.category_for(count as u64) {
                return format!("{:?}", category).to_lowercase();
            }
        }

        // Fallback to basic English rules
        match count {
            0 => "zero",
            1 => "one",
            2 => "two",
            _ => "other",
        }.to_string()
    }

    /// Extract the appropriate plural form from specification
    fn extract_plural_form(&self, plural_spec: &str, category: &str) -> Option<String> {
        // Parse patterns like: zero{no items} one{# item} few{# items} many{# items} other{# items}
        let option_pattern = Regex::new(r"(\w+)\{([^}]*)\}").ok()?;

        // First try exact category match
        for cap in option_pattern.captures_iter(plural_spec) {
            let form = &cap[1];
            let text = &cap[2];

            if form == category {
                return Some(text.to_string());
            }
        }

        // Fallback to "other" if available
        if category != "other" {
            for cap in option_pattern.captures_iter(plural_spec) {
                let form = &cap[1];
                let text = &cap[2];

                if form == "other" {
                    return Some(text.to_string());
                }
            }
        }

        None
    }
}

impl SelectEngine {
    /// Create a new select engine
    fn new() -> Result<Self> {
        Ok(Self {
            select_pattern: Regex::new(r"\{\{([^,]+),\s*select,\s*([^}]+)\}\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid select regex: {}", e)))?,
        })
    }

    /// Interpolate select patterns
    fn interpolate(&self, template: &str, context: &InterpolationContext) -> Result<String> {
        let result = self.select_pattern.replace_all(template, |caps: &regex::Captures| {
            let var_expr = caps[1].trim();
            let select_spec = caps[2].trim();

            // Get the selector value
            let selector = self.resolve_selector(var_expr, context);

            // Extract the appropriate select option
            self.extract_select_option(select_spec, &selector)
                .unwrap_or_else(|| format!("{{{{{}}}}}", &caps[0]))
        });

        Ok(result.to_string())
    }

    /// Resolve selector value from variable expression
    fn resolve_selector(&self, var_expr: &str, context: &InterpolationContext) -> String {
        // Try to get as direct variable first
        if let Some(value) = context.variables.get(var_expr) {
            return value.clone();
        }

        // Could be extended to support expressions
        "other".to_string() // Default fallback
    }

    /// Extract the appropriate option from select specification
    fn extract_select_option(&self, select_spec: &str, selector: &str) -> Option<String> {
        // Parse patterns like: male{him} female{her} other{them}
        let option_pattern = Regex::new(r"(\w+)\{([^}]*)\}").ok()?;

        // First try exact match
        for cap in option_pattern.captures_iter(select_spec) {
            let key = &cap[1];
            let text = &cap[2];

            if key == selector {
                return Some(text.to_string());
            }
        }

        // Fallback to "other" if available
        if selector != "other" {
            for cap in option_pattern.captures_iter(select_spec) {
                let key = &cap[1];
                let text = &cap[2];

                if key == "other" {
                    return Some(text.to_string());
                }
            }
        }

        None
    }
}

    /// Get the plural form for a count in the given locale
    fn get_plural_form(&self, count: i64, _locale: &Locale) -> String {
        // Simplified plural rules - in a real implementation, this would use
        // the intl_pluralrules crate for proper locale-specific rules
        match count {
            0 => "zero".to_string(),
            1 => "one".to_string(),
            2 => "two".to_string(),
            _ => "other".to_string(),
        }
    }

    /// Extract the appropriate option from a plural specification
    fn extract_plural_option(&self, plural_spec: &str, plural_form: &str) -> Option<String> {
        // Parse patterns like: one{# item} other{# items}
        let option_pattern = Regex::new(r"(\w+)\{([^}]*)\}").ok()?;

        for cap in option_pattern.captures_iter(plural_spec) {
            let form = &cap[1];
            let text = &cap[2];

            if form == plural_form {
                return Some(text.to_string());
            }
        }

        // Fallback to "other" if available
        if plural_form != "other" {
            for cap in option_pattern.captures_iter(plural_spec) {
                let form = &cap[1];
                let text = &cap[2];

                if form == "other" {
                    return Some(text.to_string());
                }
            }
        }

        None
    }

    /// Extract the appropriate option from a select specification
    fn extract_select_option(&self, select_spec: &str, selector: &str) -> Option<String> {
        // Parse patterns like: male{él} female{ella} other{elle}
        let option_pattern = Regex::new(r"(\w+)\{([^}]*)\}").ok()?;

        for cap in option_pattern.captures_iter(select_spec) {
            let key = &cap[1];
            let text = &cap[2];

            if key == selector {
                return Some(text.to_string());
            }
        }

        // Fallback to "other" if available
        if selector != "other" {
            for cap in option_pattern.captures_iter(select_spec) {
                let key = &cap[1];
                let text = &cap[2];

                if key == "other" {
                    return Some(text.to_string());
                }
            }
        }

        None
    }

impl Default for Interpolator {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(locale: &Locale) -> InterpolationContext {
        InterpolationContext::new(locale)
            .with_variable("name", "Alice")
            .with_variable("count", "5")
            .with_variable("gender", "female")
            .with_variable("price", "29.99")
    }

    #[test]
    fn test_simple_variable_interpolation() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = create_test_context(&locale);

        let template = "Hello, {{name}}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_dollar_syntax_interpolation() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = create_test_context(&locale);

        let template = "Hello, ${name}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_formatted_variable_interpolation() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("price", "29.99");

        let template = "Price: {{price, currency}}";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Price: $29.99");
    }

    #[test]
    fn test_plural_interpolation() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();

        // Test singular
        let context_one = InterpolationContext::new(&locale)
            .with_variable("count", "1");

        let template = "You have {{count, plural, one{# item} other{# items}}}";
        let result = interpolator.interpolate(template, &context_one).unwrap();
        assert_eq!(result, "You have # item");

        // Test plural
        let context_many = InterpolationContext::new(&locale)
            .with_variable("count", "5");

        let result = interpolator.interpolate(template, &context_many).unwrap();
        assert_eq!(result, "You have # items");
    }

    #[test]
    fn test_select_interpolation() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = create_test_context(&locale);

        let template = "{{gender, select, male{him} female{her} other{them}}}";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "her");
    }

    #[test]
    fn test_missing_variable() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale);

        let template = "Hello, {{missing_var}}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        // Should keep the placeholder
        assert_eq!(result, "Hello, {{missing_var}}!");
    }

    #[test]
    fn test_complex_interpolation() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("name", "Alice")
            .with_variable("count", "3")
            .with_variable("gender", "female");

        let template = "{{name}} has {{count, plural, one{# friend} other{# friends}}} and {{gender, select, male{he} female{she} other{they}}} likes programming.";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Alice has # friends and she likes programming.");
    }

    #[test]
    fn test_nested_property_access() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("user", r#"{"name": "Alice", "profile": {"age": 30}}"#);

        let template = "Hello {{user.name}}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, r#"Hello "Alice"!"#);
    }

    #[test]
    fn test_format_capitalize() {
        let interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("name", "alice");

        let template = "Hello {{name, capitalize}}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_recursion_prevention() {
        let interpolator = Interpolator::with_max_recursion_depth(2).unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("a", "{{b}}")
            .with_variable("b", "{{c}}")
            .with_variable("c", "value");

        let template = "{{a}}";
        let result = interpolator.interpolate(template, &context);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("recursion depth"));
    }
}
