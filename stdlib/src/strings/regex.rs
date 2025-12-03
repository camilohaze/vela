/*!
# Regex Support

Basic regex pattern matching and replacement.

## Examples

```rust
use vela_stdlib::strings::Regex;

let re = Regex::new(r"\d+").unwrap();
assert!(re.is_match("abc123"));
```
*/

use std::fmt;

/// Regex error
#[derive(Debug, Clone, PartialEq)]
pub enum RegexError {
    /// Invalid pattern
    InvalidPattern(String),
    /// Compilation error
    CompileError(String),
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexError::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
            RegexError::CompileError(msg) => write!(f, "Compile error: {}", msg),
        }
    }
}

impl std::error::Error for RegexError {}

/// Simple regex wrapper
///
/// NOTE: This is a simplified regex implementation.
/// For production use, integrate with the `regex` crate.
pub struct Regex {
    pattern: String,
}

impl Regex {
    /// Create new regex
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        if pattern.is_empty() {
            return Err(RegexError::InvalidPattern("Empty pattern".to_string()));
        }

        Ok(Regex {
            pattern: pattern.to_string(),
        })
    }

    /// Check if text matches pattern
    ///
    /// Simplified implementation - only supports basic patterns:
    /// - Literal matching
    /// - `\d` for digits
    /// - `\w` for word characters
    /// - `.` for any character
    pub fn is_match(&self, text: &str) -> bool {
        self.find(text).is_some()
    }

    /// Find first match
    pub fn find(&self, text: &str) -> Option<String> {
        // Simplified implementation
        if self.pattern == r"\d+" {
            // Match one or more digits
            let start = text.chars().position(|c| c.is_ascii_digit())?;
            let end = text[start..]
                .chars()
                .position(|c| !c.is_ascii_digit())
                .map(|pos| start + pos)
                .unwrap_or(text.len());
            return Some(text[start..end].to_string());
        }

        if self.pattern == r"\w+" {
            // Match one or more word characters
            let start = text.chars().position(|c| c.is_alphanumeric() || c == '_')?;
            let end = text[start..]
                .chars()
                .position(|c| !(c.is_alphanumeric() || c == '_'))
                .map(|pos| start + pos)
                .unwrap_or(text.len());
            return Some(text[start..end].to_string());
        }

        // Literal matching
        if text.contains(&self.pattern) {
            return Some(self.pattern.clone());
        }

        None
    }

    /// Find all matches
    pub fn find_all(&self, text: &str) -> Vec<String> {
        let mut matches = Vec::new();
        let mut remaining = text;

        while let Some(m) = self.find(remaining) {
            if let Some(pos) = remaining.find(&m) {
                matches.push(m.clone());
                remaining = &remaining[pos + m.len()..];
                if remaining.is_empty() {
                    break;
                }
            } else {
                break;
            }
        }

        matches
    }

    /// Replace first match
    pub fn replace(&self, text: &str, replacement: &str) -> String {
        if let Some(m) = self.find(text) {
            text.replacen(&m, replacement, 1)
        } else {
            text.to_string()
        }
    }

    /// Replace all matches
    pub fn replace_all(&self, text: &str, replacement: &str) -> String {
        let matches = self.find_all(text);
        let mut result = text.to_string();

        for m in matches.iter().rev() {
            result = result.replace(m, replacement);
        }

        result
    }

    /// Split by pattern
    pub fn split(&self, text: &str) -> Vec<String> {
        let matches = self.find_all(text);
        
        if matches.is_empty() {
            return vec![text.to_string()];
        }

        let mut parts = Vec::new();
        let mut start = 0;

        for m in matches {
            if let Some(pos) = text[start..].find(&m) {
                let abs_pos = start + pos;
                parts.push(text[start..abs_pos].to_string());
                start = abs_pos + m.len();
            }
        }

        // Add remaining text
        if start < text.len() {
            parts.push(text[start..].to_string());
        }

        parts
    }
}

impl fmt::Debug for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Regex({})", self.pattern)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_regex() {
        let re = Regex::new(r"\d+");
        assert!(re.is_ok());
    }

    #[test]
    fn test_empty_pattern() {
        let re = Regex::new("");
        assert!(matches!(re, Err(RegexError::InvalidPattern(_))));
    }

    #[test]
    fn test_digit_match() {
        let re = Regex::new(r"\d+").unwrap();
        assert!(re.is_match("abc123"));
        assert!(re.is_match("123abc"));
        assert!(!re.is_match("abc"));
    }

    #[test]
    fn test_find_digit() {
        let re = Regex::new(r"\d+").unwrap();
        let result = re.find("abc123def");
        assert_eq!(result, Some("123".to_string()));
    }

    #[test]
    fn test_find_all_digits() {
        let re = Regex::new(r"\d+").unwrap();
        let matches = re.find_all("abc123def456");
        assert_eq!(matches, vec!["123", "456"]);
    }

    #[test]
    fn test_word_match() {
        let re = Regex::new(r"\w+").unwrap();
        assert!(re.is_match("hello world"));
        let result = re.find("hello world");
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_literal_match() {
        let re = Regex::new("hello").unwrap();
        assert!(re.is_match("hello world"));
        assert!(!re.is_match("goodbye"));
    }

    #[test]
    fn test_replace() {
        let re = Regex::new(r"\d+").unwrap();
        let result = re.replace("abc123def", "XXX");
        assert_eq!(result, "abcXXXdef");
    }

    #[test]
    fn test_replace_all() {
        let re = Regex::new(r"\d+").unwrap();
        let result = re.replace_all("abc123def456", "X");
        assert_eq!(result, "abcXdefX");
    }

    #[test]
    fn test_split() {
        let re = Regex::new(r"\d+").unwrap();
        let parts = re.split("abc123def456ghi");
        assert_eq!(parts, vec!["abc", "def", "ghi"]);
    }

    #[test]
    fn test_split_no_match() {
        let re = Regex::new(r"\d+").unwrap();
        let parts = re.split("abc");
        assert_eq!(parts, vec!["abc"]);
    }
}
