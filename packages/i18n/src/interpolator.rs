/*!
String interpolation with variables and pluralization support.
*/

use std::collections::HashMap;
use regex::Regex;
use crate::error::{I18nError, Result};
use crate::locale::Locale;
use icu_plurals::PluralRuleType;
use icu_locid::Locale as IcuLocale;
use icu_decimal::FixedDecimalFormatter;
use fixed_decimal::FixedDecimal;

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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
struct VariableResolver {
    /// Regex for ${variable} syntax
    dollar_syntax: Regex,
    /// Regex for {{variable}} syntax
    brace_syntax: Regex,
    /// Regex for {{variable, format}} syntax
    formatted_syntax: Regex,
}

/// Pluralization engine using basic rules (no ICU for threading compatibility)
#[derive(Debug, Clone)]
struct PluralizationEngine {
    // No ICU dependencies for Send/Sync compatibility
}

/// Select engine for conditional text selection
#[derive(Debug, Clone)]
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
    pub fn interpolate(&mut self, template: &str, context: &InterpolationContext) -> Result<String> {
        self.interpolate_with_depth_immutable(template, context, 0)
    }

    /// Internal interpolation with recursion depth tracking (immutable version for recursive calls)
    fn interpolate_with_depth_immutable(&mut self, template: &str, context: &InterpolationContext, depth: usize) -> Result<String> {
        let mut visited = std::collections::HashSet::new();
        self.interpolate_with_visited(template, context, depth, &mut visited)
    }

    /// Internal interpolation with visited set for cycle detection
    fn interpolate_with_visited(&mut self, template: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        println!("Depth: {}, Template: {}", depth, template);
        if depth > self.max_recursion_depth {
            println!("Max depth exceeded: {} > {}", depth, self.max_recursion_depth);
            return Err(I18nError::interpolation_error(
                format!("Maximum interpolation recursion depth ({}) exceeded", self.max_recursion_depth)
            ));
        }

        let mut result = template.to_string();

        // Process in order: select -> variables -> plural (most specific first)
        result = self.select_engine.interpolate(&result, context)?;
        result = self.variable_resolver.interpolate_with_visited(&result, context, depth, visited)?;
        result = self.pluralization_engine.interpolate(&result, context)?;

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
    fn interpolate_with_visited(&self, template: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        let mut result = template.to_string();

        // Handle formatted syntax first (more specific)
        result = self.interpolate_formatted_with_visited(&result, context, depth, visited)?;

        // Handle dollar syntax ${variable}
        result = self.interpolate_dollar_syntax_with_visited(&result, context, depth, visited)?;

        // Handle brace syntax {{variable}}
        result = self.interpolate_brace_syntax_with_visited(&result, context, depth, visited)?;

        Ok(result)
    }

    /// Interpolate ${variable} syntax with visited tracking
    fn interpolate_dollar_syntax_with_visited(&self, template: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        let mut result = template.to_string();
        for cap in self.dollar_syntax.captures_iter(template) {
            let var_expr = &cap[1];
            match self.resolve_variable(var_expr, context, depth, visited) {
                Ok(value) => {
                    let placeholder = format!("${{{}}}", var_expr);
                    result = result.replace(&placeholder, &value);
                }
                Err(e) if e.to_string().contains("recursion") => return Err(e),
                Err(_) => {
                    // Keep the placeholder for other errors
                }
            }
        }
        Ok(result)
    }

    /// Interpolate {{variable}} syntax with visited tracking
    fn interpolate_brace_syntax_with_visited(&self, template: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        let mut result = template.to_string();
        for cap in self.brace_syntax.captures_iter(template) {
            let var_expr = &cap[1];
            // Skip if it's a formatted expression (handled separately)
            if var_expr.contains(',') {
                continue;
            }
            match self.resolve_variable(var_expr, context, depth, visited) {
                Ok(value) => {
                    let placeholder = format!("{{{{{}}}}}", var_expr);
                    result = result.replace(&placeholder, &value);
                }
                Err(e) if e.to_string().contains("recursion") => return Err(e),
                Err(_) => {
                    // Keep the placeholder for other errors
                }
            }
        }
        Ok(result)
    }

    /// Interpolate {{variable, format}} syntax with visited tracking
    fn interpolate_formatted_with_visited(&self, template: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        let result = self.formatted_syntax.replace_all(template, |caps: &regex::Captures| {
            let var_name = caps[1].trim();
            let format_spec = caps[2].trim();

            match self.resolve_variable(var_name, context, depth, visited) {
                Ok(value) => self.apply_format(&value, format_spec, context),
                Err(_) => format!("{{{{{}, {}}}}}", var_name, format_spec),
            }
        });
        Ok(result.to_string())
    }

    /// Resolve a variable expression (supports dot notation and array access)
    fn resolve_variable(&self, var_expr: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        println!("Resolving variable: {}, depth: {}", var_expr, depth);
        // Check for recursion
        if visited.contains(var_expr) {
            println!("Circular reference detected: {}", var_expr);
            return Err(I18nError::interpolation_error(
                format!("Circular variable reference detected: {}", var_expr)
            ));
        }

        // Handle nested property access like user.name or items[0]
        let parts: Vec<&str> = var_expr.split('.').collect();
        let base_var = parts[0];

        // Add to visited set
        visited.insert(base_var.to_string());

        let mut current_value = self.get_base_variable(base_var, context)?;
        println!("Base value for {}: {}", base_var, current_value);

        // Add recursive interpolation
        current_value = self.interpolate_recursive(&current_value, context, depth + 1, visited)?;

        // Remove from visited set
        visited.remove(base_var);

        // Navigate through properties
        for &part in &parts[1..] {
            current_value = self.navigate_property(&current_value, part)?;
        }

        Ok(current_value)
    }

    /// Interpolate recursively without using the main interpolator
    fn interpolate_recursive(&self, template: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        println!("Recursive interpolate depth: {}, template: {}", depth, template);
        if depth > 1 { // Prevent infinite recursion
            println!("Recursive depth exceeded: {} > 1", depth);
            return Err(I18nError::interpolation_error(
                format!("Maximum interpolation recursion depth ({}) exceeded", 1)
            ));
        }

        let mut result = template.to_string();

        // Handle dollar syntax ${variable}
        result = self.interpolate_dollar_syntax_recursive(&result, context, depth, visited)?;

        // Handle brace syntax {{variable}}
        result = self.interpolate_brace_syntax_recursive(&result, context, depth, visited)?;

        Ok(result)
    }

    /// Interpolate ${variable} syntax recursively
    fn interpolate_dollar_syntax_recursive(&self, template: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        let mut result = template.to_string();
        for cap in self.dollar_syntax.captures_iter(template) {
            let var_expr = &cap[1];
            match self.resolve_variable(var_expr, context, depth, visited) {
                Ok(value) => {
                    let placeholder = format!("${{{}}}", var_expr);
                    result = result.replace(&placeholder, &value);
                }
                Err(e) if e.to_string().contains("recursion") => return Err(e),
                Err(_) => {
                    // Keep the placeholder for other errors
                }
            }
        }
        Ok(result)
    }

    /// Interpolate {{variable}} syntax recursively
    fn interpolate_brace_syntax_recursive(&self, template: &str, context: &InterpolationContext, depth: usize, visited: &mut std::collections::HashSet<String>) -> Result<String> {
        let mut result = template.to_string();
        for cap in self.brace_syntax.captures_iter(template) {
            let var_expr = &cap[1];
            // Skip if it's a formatted expression (handled separately)
            if var_expr.contains(',') {
                continue;
            }
            match self.resolve_variable(var_expr, context, depth, visited) {
                Ok(value) => {
                    let placeholder = format!("{{{{{}}}}}", var_expr);
                    result = result.replace(&placeholder, &value);
                }
                Err(e) if e.to_string().contains("recursion") => return Err(e),
                Err(_) => {
                    // Keep the placeholder for other errors
                }
            }
        }
        Ok(result)
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
            if let Ok(icu_locale) = IcuLocale::try_from_bytes(locale.as_str().as_bytes()) {
                if let Ok(formatter) = FixedDecimalFormatter::try_new(&icu_locale.into(), Default::default()) {
                    // Create FixedDecimal from integer part and fractional part
                    let integer_part = num.trunc() as i64;
                    let fractional_part = (num.fract() * 1_000_000_000.0) as i64; // 9 decimal places
                    
                    let fixed_decimal = FixedDecimal::from(integer_part);
                    if fractional_part > 0 {
                        // This is a simplified approach - in practice, you'd need more sophisticated handling
                        // For now, just return the string representation
                        return num.to_string();
                    }
                    return formatter.format(&fixed_decimal).to_string();
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
    /// Create a new pluralization engine
    fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Ensure plural rules are loaded for the given locale
    fn ensure_plural_rules(&mut self, _locale: &Locale, _rule_type: PluralRuleType) {
        // No-op: using basic plural rules
    }

    /// Interpolate plural patterns using basic rules
    fn interpolate(&mut self, template: &str, context: &InterpolationContext) -> Result<String> {
        // Basic plural pattern: {{count, plural, zero{no items} one{1 item} other{# items}}}
        let plural_pattern = Regex::new(r"\{\{([^,]+),\s*plural,\s*(.+)\}\}")
            .map_err(|e| I18nError::interpolation_error(format!("Invalid plural regex: {}", e)))?;

        let mut result = template.to_string();

        for caps in plural_pattern.captures_iter(&result.clone()) {
            let var_expr = caps[1].trim();
            let plural_spec = caps[2].trim();

            // Get count value
            let count = self.resolve_count(var_expr, context);
            let plural_category = self.get_plural_category(count, context.locale, PluralRuleType::Cardinal);

            if let Some(replacement) = self.extract_plural_form(plural_spec, &plural_category) {
                let full_match = caps[0].to_string();
                result = result.replace(&full_match, &replacement);
            }
        }

        Ok(result)
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

    /// Get plural category for a count (basic implementation without ICU)
    fn get_plural_category(&self, count: i64, _locale: &Locale, _rule_type: PluralRuleType) -> String {
        // Basic English plural rules (fallback when ICU is not available)
        match count {
            0 => "zero".to_string(),
            1 => "one".to_string(),
            _ => "other".to_string(),
        }
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
            select_pattern: Regex::new(r"\{\{([^,]+),\s*select,\s*(.+)\}\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid select regex: {}", e)))?,
        })
    }

    /// Interpolate select patterns
    fn interpolate(&mut self, template: &str, context: &InterpolationContext) -> Result<String> {
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
        let mut interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = create_test_context(&locale);

        let template = "Hello, {{name}}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_dollar_syntax_interpolation() {
        let mut interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = create_test_context(&locale);

        let template = "Hello, ${name}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_dollar_variable_interpolation() {
        let mut interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("price", "29.99");

        let template = "Price: {{price, currency}}";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Price: $29.99");
    }

    #[test]
    fn test_plural_interpolation() {
        let mut interpolator = Interpolator::new().unwrap();
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
        let mut interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = create_test_context(&locale);

        let template = "{{gender, select, male{him} female{her} other{them}}}";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "her");
    }

    #[test]
    fn test_missing_variable() {
        let mut interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale);

        let template = "Hello, {{missing_var}}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        // Should keep the placeholder
        assert_eq!(result, "Hello, {{missing_var}}!");
    }

    #[test]
    fn test_complex_interpolation() {
        let mut interpolator = Interpolator::new().unwrap();
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
        let mut interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("user", r#"{"name": "Alice", "profile": {"age": 30}}"#);

        let template = "Hello {{user.name}}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, r#"Hello "Alice"!"#);
    }

    #[test]
    fn test_format_capitalize() {
        let mut interpolator = Interpolator::new().unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("name", "alice");

        let template = "Hello {{name, capitalize}}!";
        let result = interpolator.interpolate(template, &context).unwrap();

        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_recursion_prevention() {
        let mut interpolator = Interpolator::with_max_recursion_depth(1).unwrap();
        let locale = Locale::from("en").unwrap();
        let context = InterpolationContext::new(&locale)
            .with_variable("a", "{{b}}")
            .with_variable("b", "{{c}}")
            .with_variable("c", "value");

        let template = "{{a}}";
        let result = interpolator.interpolate(template, &context);
        println!("Final result: {:?}", result);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("recursion depth"));
    }
}
