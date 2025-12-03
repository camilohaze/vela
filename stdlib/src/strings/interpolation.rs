/*!
# String Interpolation

Template string interpolation with `${}` syntax.

## Design

Parses template strings and replaces `${variable}` with actual values.

## Examples

```rust
use vela_stdlib::strings::interpolate;
use std::collections::HashMap;

let mut vars = HashMap::new();
vars.insert("name".to_string(), "Vela".to_string());
vars.insert("version".to_string(), "1.0".to_string());

let result = interpolate("Hello, ${name} v${version}!", &vars).unwrap();
assert_eq!(result, "Hello, Vela v1.0!");
```
*/

use std::collections::HashMap;
use std::fmt;

/// Interpolation error
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationError {
    /// Variable not found
    VariableNotFound(String),
    /// Invalid syntax
    InvalidSyntax(String),
}

impl fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpolationError::VariableNotFound(var) => {
                write!(f, "Variable not found: {}", var)
            }
            InterpolationError::InvalidSyntax(msg) => {
                write!(f, "Invalid syntax: {}", msg)
            }
        }
    }
}

impl std::error::Error for InterpolationError {}

/// Interpolate template string with variables
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use vela_stdlib::strings::interpolate;
///
/// let mut vars = HashMap::new();
/// vars.insert("name".to_string(), "World".to_string());
/// let result = interpolate("Hello, ${name}!", &vars).unwrap();
/// assert_eq!(result, "Hello, World!");
/// ```
pub fn interpolate(
    template: &str,
    variables: &HashMap<String, String>,
) -> Result<String, InterpolationError> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Check for ${
            if chars.peek() == Some(&'{') {
                chars.next(); // Consume '{'
                
                // Extract variable name
                let mut var_name = String::new();
                let mut found_close = false;
                
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        found_close = true;
                        break;
                    }
                    var_name.push(ch);
                }
                
                if !found_close {
                    return Err(InterpolationError::InvalidSyntax(
                        "Unclosed interpolation".to_string(),
                    ));
                }
                
                // Lookup variable
                let value = variables
                    .get(&var_name)
                    .ok_or_else(|| InterpolationError::VariableNotFound(var_name.clone()))?;
                
                result.push_str(value);
            } else {
                // Just a dollar sign
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

/// Interpolate with fallback for missing variables
pub fn interpolate_with_fallback(
    template: &str,
    variables: &HashMap<String, String>,
    fallback: &str,
) -> String {
    interpolate(template, variables).unwrap_or_else(|_| fallback.to_string())
}

/// Extract variable names from template
pub fn extract_variables(template: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next(); // Consume '{'
            
            let mut var_name = String::new();
            while let Some(ch) = chars.next() {
                if ch == '}' {
                    break;
                }
                var_name.push(ch);
            }
            
            if !var_name.is_empty() {
                variables.push(var_name);
            }
        }
    }

    variables
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vars() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Vela".to_string());
        vars.insert("version".to_string(), "1.0".to_string());
        vars.insert("author".to_string(), "Team".to_string());
        vars
    }

    #[test]
    fn test_simple_interpolation() {
        let vars = make_vars();
        let result = interpolate("Hello, ${name}!", &vars).unwrap();
        assert_eq!(result, "Hello, Vela!");
    }

    #[test]
    fn test_multiple_interpolations() {
        let vars = make_vars();
        let result = interpolate("${name} v${version} by ${author}", &vars).unwrap();
        assert_eq!(result, "Vela v1.0 by Team");
    }

    #[test]
    fn test_no_interpolation() {
        let vars = HashMap::new();
        let result = interpolate("Hello, World!", &vars).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_variable_not_found() {
        let vars = HashMap::new();
        let result = interpolate("Hello, ${name}!", &vars);
        assert!(matches!(result, Err(InterpolationError::VariableNotFound(_))));
    }

    #[test]
    fn test_unclosed_interpolation() {
        let vars = make_vars();
        let result = interpolate("Hello, ${name", &vars);
        assert!(matches!(result, Err(InterpolationError::InvalidSyntax(_))));
    }

    #[test]
    fn test_dollar_sign_without_brace() {
        let vars = HashMap::new();
        let result = interpolate("Price: $100", &vars).unwrap();
        assert_eq!(result, "Price: $100");
    }

    #[test]
    fn test_empty_variable_name() {
        let vars = HashMap::new();
        let result = interpolate("Test ${}", &vars);
        assert!(matches!(result, Err(InterpolationError::VariableNotFound(_))));
    }

    #[test]
    fn test_extract_variables() {
        let template = "${name} v${version} by ${author}";
        let vars = extract_variables(template);
        assert_eq!(vars, vec!["name", "version", "author"]);
    }

    #[test]
    fn test_extract_variables_no_vars() {
        let template = "Hello, World!";
        let vars = extract_variables(template);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_interpolate_with_fallback() {
        let vars = HashMap::new();
        let result = interpolate_with_fallback("Hello, ${name}!", &vars, "Fallback");
        assert_eq!(result, "Fallback");
    }
}
