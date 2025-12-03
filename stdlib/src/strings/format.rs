/*!
# String Formatting

Type-safe string formatting utilities.

## Examples

```rust
use vela_stdlib::strings::format_string;

let result = format_string("Hello, {}!", &["World"]).unwrap();
assert_eq!(result, "Hello, World!");
```
*/

use std::fmt;

/// Format error
#[derive(Debug, Clone, PartialEq)]
pub enum FormatError {
    /// Mismatched argument count
    MismatchedArguments { expected: usize, got: usize },
    /// Invalid format specifier
    InvalidSpecifier(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::MismatchedArguments { expected, got } => {
                write!(f, "Expected {} arguments, got {}", expected, got)
            }
            FormatError::InvalidSpecifier(spec) => {
                write!(f, "Invalid format specifier: {}", spec)
            }
        }
    }
}

impl std::error::Error for FormatError {}

/// Format string with positional arguments
///
/// # Examples
///
/// ```
/// use vela_stdlib::strings::format_string;
///
/// let result = format_string("Hello, {}!", &["World"]).unwrap();
/// assert_eq!(result, "Hello, World!");
/// ```
pub fn format_string(template: &str, args: &[&str]) -> Result<String, FormatError> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    let mut arg_index = 0;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Check for {{
            if chars.peek() == Some(&'{') {
                chars.next();
                result.push('{');
                continue;
            }

            // Check for }
            if chars.peek() == Some(&'}') {
                chars.next();

                // Get argument
                if arg_index >= args.len() {
                    return Err(FormatError::MismatchedArguments {
                        expected: count_placeholders(template),
                        got: args.len(),
                    });
                }

                result.push_str(args[arg_index]);
                arg_index += 1;
            } else {
                // Invalid format
                return Err(FormatError::InvalidSpecifier("Unclosed placeholder".to_string()));
            }
        } else if ch == '}' {
            // Check for }}
            if chars.peek() == Some(&'}') {
                chars.next();
                result.push('}');
            } else {
                return Err(FormatError::InvalidSpecifier("Unmatched }".to_string()));
            }
        } else {
            result.push(ch);
        }
    }

    // Check if all arguments were used
    if arg_index < args.len() {
        return Err(FormatError::MismatchedArguments {
            expected: count_placeholders(template),
            got: args.len(),
        });
    }

    Ok(result)
}

/// Count placeholders in template
fn count_placeholders(template: &str) -> usize {
    let mut count = 0;
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() != Some(&'{') {
            if chars.peek() == Some(&'}') {
                count += 1;
            }
        }
    }

    count
}

/// Format with named arguments
pub fn format_named(template: &str, args: &[(&str, &str)]) -> Result<String, FormatError> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                result.push('{');
                continue;
            }

            // Extract name
            let mut name = String::new();
            while let Some(ch) = chars.next() {
                if ch == '}' {
                    break;
                }
                name.push(ch);
            }

            // Find argument
            let value = args
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| *v)
                .ok_or_else(|| FormatError::InvalidSpecifier(format!("Argument '{}' not found", name)))?;

            result.push_str(value);
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
                result.push('}');
            } else {
                return Err(FormatError::InvalidSpecifier("Unmatched }".to_string()));
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_format() {
        let result = format_string("Hello, {}!", &["World"]).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_multiple_arguments() {
        let result = format_string("{} v{} by {}", &["Vela", "1.0", "Team"]).unwrap();
        assert_eq!(result, "Vela v1.0 by Team");
    }

    #[test]
    fn test_no_placeholders() {
        let result = format_string("Hello, World!", &[]).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_escaped_braces() {
        let result = format_string("{{}}{}!", &["test"]).unwrap();
        assert_eq!(result, "{}test!");
    }

    #[test]
    fn test_too_few_arguments() {
        let result = format_string("Hello, {}!", &[]);
        assert!(matches!(result, Err(FormatError::MismatchedArguments { .. })));
    }

    #[test]
    fn test_too_many_arguments() {
        let result = format_string("Hello!", &["extra"]);
        assert!(matches!(result, Err(FormatError::MismatchedArguments { .. })));
    }

    #[test]
    fn test_unclosed_placeholder() {
        let result = format_string("Hello, {", &["World"]);
        assert!(matches!(result, Err(FormatError::InvalidSpecifier(_))));
    }

    #[test]
    fn test_unmatched_close_brace() {
        let result = format_string("Hello}", &[]);
        assert!(matches!(result, Err(FormatError::InvalidSpecifier(_))));
    }

    #[test]
    fn test_format_named() {
        let args = [("name", "Vela"), ("version", "1.0")];
        let result = format_named("{name} v{version}", &args).unwrap();
        assert_eq!(result, "Vela v1.0");
    }

    #[test]
    fn test_format_named_not_found() {
        let args = [("name", "Vela")];
        let result = format_named("{name} v{version}", &args);
        assert!(matches!(result, Err(FormatError::InvalidSpecifier(_))));
    }
}
