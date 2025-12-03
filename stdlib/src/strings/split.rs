/*!
# Advanced String Splitting

Advanced string splitting utilities.

## Examples

```rust
use vela_stdlib::strings::split_advanced;

let parts = split_advanced("a,b,c", ",");
assert_eq!(parts, vec!["a", "b", "c"]);
```
*/

/// Split string by delimiter
pub fn split_advanced(text: &str, delimiter: &str) -> Vec<String> {
    if delimiter.is_empty() {
        return text.chars().map(|c| c.to_string()).collect();
    }

    text.split(delimiter).map(|s| s.to_string()).collect()
}

/// Split by whitespace (any amount)
pub fn split_whitespace(text: &str) -> Vec<String> {
    text.split_whitespace().map(|s| s.to_string()).collect()
}

/// Split by multiple delimiters
pub fn split_by_any(text: &str, delimiters: &[char]) -> Vec<String> {
    text.split(|c| delimiters.contains(&c))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Split with limit (max number of splits)
pub fn split_n(text: &str, delimiter: &str, limit: usize) -> Vec<String> {
    text.splitn(limit, delimiter).map(|s| s.to_string()).collect()
}

/// Split from the end (reverse)
pub fn rsplit(text: &str, delimiter: &str) -> Vec<String> {
    text.rsplit(delimiter).map(|s| s.to_string()).collect()
}

/// Split and keep delimiter
pub fn split_inclusive(text: &str, delimiter: &str) -> Vec<String> {
    if delimiter.is_empty() {
        return vec![text.to_string()];
    }

    let mut parts = Vec::new();
    let mut start = 0;

    while let Some(pos) = text[start..].find(delimiter) {
        let abs_pos = start + pos;
        parts.push(text[start..abs_pos + delimiter.len()].to_string());
        start = abs_pos + delimiter.len();
    }

    // Add remaining
    if start < text.len() {
        parts.push(text[start..].to_string());
    }

    parts
}

/// Split into chunks of fixed size
pub fn chunk(text: &str, size: usize) -> Vec<String> {
    if size == 0 {
        return vec![text.to_string()];
    }

    text.chars()
        .collect::<Vec<_>>()
        .chunks(size)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

/// Split into lines (respecting different line endings)
pub fn split_lines(text: &str) -> Vec<String> {
    text.lines().map(|s| s.to_string()).collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_advanced() {
        let parts = split_advanced("a,b,c", ",");
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_empty_delimiter() {
        let parts = split_advanced("abc", "");
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_whitespace() {
        let parts = split_whitespace("hello   world  test");
        assert_eq!(parts, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_split_by_any() {
        let parts = split_by_any("a,b;c:d", &[',', ';', ':']);
        assert_eq!(parts, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_split_n() {
        let parts = split_n("a,b,c,d", ",", 3);
        assert_eq!(parts, vec!["a", "b", "c,d"]);
    }

    #[test]
    fn test_rsplit() {
        let parts = rsplit("a,b,c", ",");
        assert_eq!(parts, vec!["c", "b", "a"]);
    }

    #[test]
    fn test_split_inclusive() {
        let parts = split_inclusive("a,b,c", ",");
        assert_eq!(parts, vec!["a,", "b,", "c"]);
    }

    #[test]
    fn test_chunk() {
        let parts = chunk("abcdefg", 3);
        assert_eq!(parts, vec!["abc", "def", "g"]);
    }

    #[test]
    fn test_chunk_zero_size() {
        let parts = chunk("abc", 0);
        assert_eq!(parts, vec!["abc"]);
    }

    #[test]
    fn test_split_lines() {
        let parts = split_lines("line1\nline2\nline3\nline4");
        assert_eq!(parts, vec!["line1", "line2", "line3", "line4"]);
    }
}
