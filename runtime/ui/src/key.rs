//! Keys for efficient widget reconciliation

use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

/// Key for widget identification during reconciliation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    /// String-based key
    String(String),
    /// Integer-based key
    Int(i64),
    /// UUID-based key
    Uuid(uuid::Uuid),
}

impl Key {
    /// Create a string key
    pub fn string<S: Into<String>>(s: S) -> Self {
        Key::String(s.into())
    }

    /// Create an integer key
    pub fn int(i: i64) -> Self {
        Key::Int(i)
    }

    /// Create a UUID key
    pub fn uuid() -> Self {
        Key::Uuid(uuid::Uuid::new_v4())
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::String(s) => write!(f, "{}", s),
            Key::Int(i) => write!(f, "{}", i),
            Key::Uuid(u) => write!(f, "{}", u),
        }
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Key::String(a), Key::String(b)) => a.cmp(b),
            (Key::Int(a), Key::Int(b)) => a.cmp(b),
            (Key::Uuid(a), Key::Uuid(b)) => a.cmp(b),
            // Cross-type comparison: String < Int < Uuid
            (Key::String(_), Key::Int(_)) => Ordering::Less,
            (Key::Int(_), Key::String(_)) => Ordering::Greater,
            (Key::String(_), Key::Uuid(_)) => Ordering::Less,
            (Key::Uuid(_), Key::String(_)) => Ordering::Greater,
            (Key::Int(_), Key::Uuid(_)) => Ordering::Less,
            (Key::Uuid(_), Key::Int(_)) => Ordering::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_string() {
        let key = Key::string("test");
        assert_eq!(key.to_string(), "test");
    }

    #[test]
    fn test_key_int() {
        let key = Key::int(42);
        assert_eq!(key.to_string(), "42");
    }

    #[test]
    fn test_key_uuid() {
        let key1 = Key::uuid();
        let key2 = Key::uuid();
        assert_ne!(key1, key2); // UUIDs should be unique
    }

    #[test]
    fn test_key_ordering() {
        let string_key = Key::string("a");
        let int_key = Key::int(1);
        let uuid_key = Key::uuid();

        assert!(string_key < int_key);
        assert!(int_key < uuid_key);
        assert!(string_key < uuid_key);
    }

    #[test]
    fn test_key_equality() {
        let key1 = Key::string("test");
        let key2 = Key::string("test");
        let key3 = Key::string("other");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}