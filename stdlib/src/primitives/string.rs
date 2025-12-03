/*!
# VelaString

String type with rich API for text manipulation.

## Design

VelaString is an immutable wrapper over Rust's String, providing a rich API
inspired by JavaScript, TypeScript, and Swift.

## Examples

```rust
use vela_stdlib::VelaString;

let s = VelaString::new("Hello, Vela!");
assert_eq!(s.len(), 12);
assert!(s.contains("Vela"));

let upper = s.to_uppercase();
assert_eq!(upper.to_str(), "HELLO, VELA!");
```
*/

use std::fmt;
use crate::option_result::VelaOption;

/// String type with immutable API
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VelaString(String);

impl VelaString {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create a new VelaString
    pub fn new(s: impl Into<String>) -> Self {
        VelaString(s.into())
    }

    /// Create from characters
    pub fn from_chars(chars: &[char]) -> Self {
        VelaString(chars.iter().collect())
    }

    /// Empty string
    pub fn empty() -> Self {
        VelaString(String::new())
    }

    // ============================================================================
    // Inspections
    // ============================================================================

    /// Get length in bytes
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get characters as vector
    pub fn chars(&self) -> Vec<char> {
        self.0.chars().collect()
    }

    /// Get as Rust str
    pub fn to_str(&self) -> &str {
        &self.0
    }

    // ============================================================================
    // Transformations
    // ============================================================================

    /// Convert to uppercase
    pub fn to_uppercase(&self) -> VelaString {
        VelaString(self.0.to_uppercase())
    }

    /// Convert to lowercase
    pub fn to_lowercase(&self) -> VelaString {
        VelaString(self.0.to_lowercase())
    }

    /// Trim whitespace from both ends
    pub fn trim(&self) -> VelaString {
        VelaString(self.0.trim().to_string())
    }

    /// Trim whitespace from start
    pub fn trim_start(&self) -> VelaString {
        VelaString(self.0.trim_start().to_string())
    }

    /// Trim whitespace from end
    pub fn trim_end(&self) -> VelaString {
        VelaString(self.0.trim_end().to_string())
    }

    /// Replace all occurrences
    pub fn replace(&self, from: &str, to: &str) -> VelaString {
        VelaString(self.0.replace(from, to))
    }

    /// Repeat string n times
    pub fn repeat(&self, n: usize) -> VelaString {
        VelaString(self.0.repeat(n))
    }

    // ============================================================================
    // Searching
    // ============================================================================

    /// Check if contains substring
    pub fn contains(&self, substring: &str) -> bool {
        self.0.contains(substring)
    }

    /// Check if starts with prefix
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.0.starts_with(prefix)
    }

    /// Check if ends with suffix
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.0.ends_with(suffix)
    }

    /// Find first index of substring
    pub fn index_of(&self, substring: &str) -> VelaOption<usize> {
        self.0.find(substring).into()
    }

    /// Find last index of substring
    pub fn last_index_of(&self, substring: &str) -> VelaOption<usize> {
        self.0.rfind(substring).into()
    }

    // ============================================================================
    // Slicing
    // ============================================================================

    /// Get substring from start to end indices (bytes)
    pub fn substring(&self, start: usize, end: usize) -> VelaOption<VelaString> {
        if start <= end && end <= self.0.len() {
            VelaOption::some(VelaString(self.0[start..end].to_string()))
        } else {
            VelaOption::none()
        }
    }

    /// Get character at index
    pub fn char_at(&self, index: usize) -> VelaOption<char> {
        self.0.chars().nth(index).into()
    }

    // ============================================================================
    // Combining
    // ============================================================================

    /// Concatenate with another string
    pub fn concat(&self, other: &VelaString) -> VelaString {
        VelaString(format!("{}{}", self.0, other.0))
    }

    /// Join multiple strings with this string as separator
    pub fn join(&self, strings: &[VelaString]) -> VelaString {
        let parts: Vec<&str> = strings.iter().map(|s| s.to_str()).collect();
        VelaString(parts.join(self.to_str()))
    }
}

impl fmt::Display for VelaString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for VelaString {
    fn from(s: String) -> Self {
        VelaString(s)
    }
}

impl From<&str> for VelaString {
    fn from(s: &str) -> Self {
        VelaString(s.to_string())
    }
}

impl From<VelaString> for String {
    fn from(s: VelaString) -> Self {
        s.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        let s1 = VelaString::new("hello");
        assert_eq!(s1.to_str(), "hello");

        let s2 = VelaString::from_chars(&['h', 'i']);
        assert_eq!(s2.to_str(), "hi");

        let s3 = VelaString::empty();
        assert!(s3.is_empty());
    }

    #[test]
    fn test_len_and_empty() {
        let s = VelaString::new("hello");
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());

        let empty = VelaString::empty();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_case_conversion() {
        let s = VelaString::new("Hello World");
        assert_eq!(s.to_uppercase().to_str(), "HELLO WORLD");
        assert_eq!(s.to_lowercase().to_str(), "hello world");
    }

    #[test]
    fn test_trim() {
        let s = VelaString::new("  hello  ");
        assert_eq!(s.trim().to_str(), "hello");
        assert_eq!(s.trim_start().to_str(), "hello  ");
        assert_eq!(s.trim_end().to_str(), "  hello");
    }

    #[test]
    fn test_replace() {
        let s = VelaString::new("hello world");
        assert_eq!(s.replace("world", "Vela").to_str(), "hello Vela");
    }

    #[test]
    fn test_repeat() {
        let s = VelaString::new("ab");
        assert_eq!(s.repeat(3).to_str(), "ababab");
    }

    #[test]
    fn test_contains() {
        let s = VelaString::new("hello world");
        assert!(s.contains("world"));
        assert!(!s.contains("bye"));
    }

    #[test]
    fn test_starts_with_ends_with() {
        let s = VelaString::new("hello world");
        assert!(s.starts_with("hello"));
        assert!(s.ends_with("world"));
        assert!(!s.starts_with("world"));
    }

    #[test]
    fn test_index_of() {
        let s = VelaString::new("hello world");
        assert_eq!(s.index_of("world").unwrap(), 6);
        assert!(s.index_of("xyz").is_none());
    }

    #[test]
    fn test_concat() {
        let s1 = VelaString::new("hello");
        let s2 = VelaString::new(" world");
        assert_eq!(s1.concat(&s2).to_str(), "hello world");
    }

    #[test]
    fn test_join() {
        let sep = VelaString::new(", ");
        let parts = vec![
            VelaString::new("a"),
            VelaString::new("b"),
            VelaString::new("c"),
        ];
        assert_eq!(sep.join(&parts).to_str(), "a, b, c");
    }
}
