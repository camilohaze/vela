/*!
String interpolation with variables and pluralization support.
*/

use std::collections::HashMap;
use regex::Regex;
use crate::error::{I18nError, Result};
use crate::locale::Locale;

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

/// String interpolator with advanced features
#[derive(Debug)]
pub struct Interpolator {
    /// Regex for finding interpolation patterns
    variable_pattern: Regex,
    /// Regex for pluralization patterns
    plural_pattern: Regex,
    /// Regex for select patterns
    select_pattern: Regex,
}

impl Interpolator {
    /// Create a new interpolator
    pub fn new() -> Result<Self> {
        Ok(Self {
            variable_pattern: Regex::new(r"\{\{([^}]+)\}\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid regex: {}", e)))?,
            plural_pattern: Regex::new(r"\{\{([^,]+),\s*plural,\s*([^}]+)\}\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid regex: {}", e)))?,
            select_pattern: Regex::new(r"\{\{([^,]+),\s*select,\s*([^}]+)\}\}")
                .map_err(|e| I18nError::interpolation_error(format!("Invalid regex: {}", e)))?,
        })
    }

    /// Interpolate a string with the given context
    pub fn interpolate(&self, template: &str, context: &InterpolationContext) -> Result<String> {
        let mut result = template.to_string();

        // Handle pluralization first (more complex)
        result = self.interpolate_plurals(&result, context)?;

        // Handle select patterns
        result = self.interpolate_selects(&result, context)?;

        // Handle simple variables
        result = self.interpolate_variables(&result, context)?;

        Ok(result)
    }

    /// Interpolate simple variables like {{name}}
    fn interpolate_variables(&self, template: &str, context: &InterpolationContext) -> Result<String> {
        let result = self.variable_pattern.replace_all(template, |caps: &regex::Captures| {
            let var_name = &caps[1];

            // Handle formatting directives like {{count, number}}
            if var_name.contains(',') {
                let parts: Vec<&str> = var_name.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    let var = parts[0];
                    let format_type = parts[1];

                    if let Some(value) = context.variables.get(var) {
                        return match format_type {
                            "number" => self.format_number(value),
                            "currency" => self.format_currency(value),
                            "date" => self.format_date(value),
                            _ => value.clone(),
                        };
                    }
                }
            } else {
                // Simple variable
                if let Some(value) = context.variables.get(var_name) {
                    return value.clone();
                }
            }

            // Variable not found, keep the placeholder
            format!("{{{{{}}}}}", var_name)
        });

        Ok(result.to_string())
    }

    /// Interpolate pluralization patterns like {{count, plural, one{# item} other{# items}}}
    fn interpolate_plurals(&self, template: &str, context: &InterpolationContext) -> Result<String> {
        let result = self.plural_pattern.replace_all(template, |caps: &regex::Captures| {
            let var_name = caps[1].trim();
            let plural_spec = caps[2].trim();

            // Get the count value
            let count = match context.variables.get(var_name) {
                Some(value) => value.parse::<i64>().unwrap_or(0),
                None => 0,
            };

            // Parse plural options
            let plural_form = self.get_plural_form(count, context.locale);
            self.extract_plural_option(plural_spec, &plural_form)
                .unwrap_or_else(|| format!("{{{{{}}}}}", &caps[0]))
        });

        Ok(result.to_string())
    }

    /// Interpolate select patterns like {{gender, select, male{él} female{ella} other{elle}}}
    fn interpolate_selects(&self, template: &str, context: &InterpolationContext) -> Result<String> {
        let result = self.select_pattern.replace_all(template, |caps: &regex::Captures| {
            let var_name = caps[1].trim();
            let select_spec = caps[2].trim();

            // Get the selector value
            let selector = context.variables.get(var_name)
                .map(|s| s.as_str())
                .unwrap_or("other");

            self.extract_select_option(select_spec, selector)
                .unwrap_or_else(|| format!("{{{{{}}}}}", &caps[0]))
        });

        Ok(result.to_string())
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

    /// Format a value as a number
    fn format_number(&self, value: &str) -> String {
        // Basic number formatting - in a real implementation, this would use
        // locale-specific formatting
        value.to_string()
    }

    /// Format a value as currency
    fn format_currency(&self, value: &str) -> String {
        // Basic currency formatting
        format!("${}", value)
    }

    /// Format a value as a date
    fn format_date(&self, value: &str) -> String {
        // Basic date formatting - in a real implementation, this would parse
        // and format dates properly
        value.to_string()
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
}