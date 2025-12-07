/*!
# Dict<K,V> - Mutable Hash Map Collection

Vela's primary mutable dictionary/hash map collection for key-value storage.
Inspired by Rust's HashMap<K,V>, Swift's Dictionary<Key,Value>, and Kotlin's MutableMap<K,V>.

## Features

- **Mutable by default**: Primary collection type for key-value storage
- **Generic constraints**: K: Eq + Hash, V: any type
- **Functional API**: map_values, filter, for_each, find, some, every
- **Zero-cost abstraction**: Over Rust's HashMap

## Example

```rust
use vela_stdlib::collections::Dict;

let mut dict = Dict::new();
dict.insert("name", "Vela");
dict.insert("version", "1.0");

let name = dict.get("name");  // Some("Vela")
let keys = dict.keys();       // ["name", "version"]
let doubled = dict.map_values(|v| format!("{}-{}", v, v));
```
*/

use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/* ============================================================================
Mutable Dict<K,V> - Main Vela Collection
============================================================================ */

/// Mutable hash-based dictionary - Vela's primary key-value collection type
/// Inspired by Rust's HashMap<K,V>, Swift's Dictionary<Key,Value>, and Kotlin's MutableMap<K,V>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dict<K, V>
where
    K: Eq + Hash,
{
    items: HashMap<K, V>,
}

impl<K: Eq + Hash, V> Dict<K, V> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create empty dictionary
    pub fn new() -> Self {
        Dict {
            items: HashMap::new(),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Dict {
            items: HashMap::with_capacity(capacity),
        }
    }

    /// Create from iterator of key-value pairs
    pub fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Dict {
            items: iter.into_iter().collect(),
        }
    }

    /// Create from vector of key-value pairs
    pub fn from_pairs(pairs: Vec<(K, V)>) -> Self {
        Dict {
            items: pairs.into_iter().collect(),
        }
    }

    // ============================================================================
    // Basic Operations
    // ============================================================================

    /// Get length
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Insert key-value pair, returns previous value if key existed
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.items.insert(key, value)
    }

    /// Get value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.items.get(key)
    }

    /// Get mutable value by key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.items.get_mut(key)
    }

    /// Remove entry by key, returns removed value if key existed
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.items.remove(key)
    }

    /// Check if contains key
    pub fn contains_key(&self, key: &K) -> bool {
        self.items.contains_key(key)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
    }

    /// Shrink capacity to fit current length
    pub fn shrink_to_fit(&mut self) {
        self.items.shrink_to_fit();
    }

    // ============================================================================
    // Advanced Operations
    // ============================================================================

    /// Get value or return default
    pub fn get_or_default(&self, key: &K, default: V) -> V
    where
        V: Clone,
    {
        self.get(key).cloned().unwrap_or(default)
    }

    /// Insert only if key doesn't exist, returns true if inserted
    pub fn insert_if_absent(&mut self, key: K, value: V) -> bool {
        if self.contains_key(&key) {
            false
        } else {
            self.insert(key, value);
            true
        }
    }

    /// Update existing value, returns true if key existed and was updated
    pub fn update<F>(&mut self, key: &K, f: F) -> bool
    where
        F: FnOnce(V) -> V,
        K: Clone,
    {
        if let Some(value) = self.items.remove(key) {
            let new_value = f(value);
            self.items.insert(key.clone(), new_value);
            true
        } else {
            false
        }
    }

    /// Merge with another dictionary, overwriting existing keys
    pub fn merge(&mut self, other: Dict<K, V>) {
        self.items.extend(other.items);
    }

    /// Merge with another dictionary, only inserting new keys
    pub fn merge_new(&mut self, other: Dict<K, V>) {
        for (key, value) in other.items {
            self.items.entry(key).or_insert(value);
        }
    }

    // ============================================================================
    // Iteration and Access
    // ============================================================================

    /// Get all keys as vector
    pub fn keys(&self) -> Vec<&K> {
        self.items.keys().collect()
    }

    /// Get all values as vector
    pub fn values(&self) -> Vec<&V> {
        self.items.values().collect()
    }

    /// Get all entries as vector of (key, value) tuples
    pub fn entries(&self) -> Vec<(&K, &V)> {
        self.items.iter().collect()
    }

    /// Get all key-value pairs as owned vector
    pub fn into_pairs(self) -> Vec<(K, V)> {
        self.items.into_iter().collect()
    }

    // ============================================================================
    // Functional Operations
    // ============================================================================

    /// Map values to new dictionary
    pub fn map_values<U, F>(&self, f: F) -> Dict<K, U>
    where
        F: Fn(&V) -> U,
        K: Clone,
    {
        Dict {
            items: self.items.iter().map(|(k, v)| (k.clone(), f(v))).collect(),
        }
    }

    /// Filter entries
    pub fn filter<F>(&self, f: F) -> Dict<K, V>
    where
        F: Fn(&K, &V) -> bool,
        K: Clone,
        V: Clone,
    {
        Dict {
            items: self.items.iter()
                .filter(|(k, v)| f(k, v))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    /// Execute function for each entry
    pub fn for_each<F>(&self, f: F)
    where
        F: Fn(&K, &V),
    {
        self.items.iter().for_each(|(k, v)| f(k, v));
    }

    /// Find first entry matching predicate
    pub fn find<F>(&self, f: F) -> Option<(&K, &V)>
    where
        F: Fn(&K, &V) -> bool,
    {
        self.items.iter().find(|(k, v)| f(k, v))
    }

    /// Check if any entry matches predicate
    pub fn some<F>(&self, f: F) -> bool
    where
        F: Fn(&K, &V) -> bool,
    {
        self.items.iter().any(|(k, v)| f(k, v))
    }

    /// Check if all entries match predicate
    pub fn every<F>(&self, f: F) -> bool
    where
        F: Fn(&K, &V) -> bool,
    {
        self.items.iter().all(|(k, v)| f(k, v))
    }
}

impl<K: Eq + Hash + fmt::Display, V: fmt::Display> fmt::Display for Dict<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (key, value) in &self.items {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", key, value)?;
            first = false;
        }
        write!(f, "}}")
    }
}

impl<K: Eq + Hash, V> From<HashMap<K, V>> for Dict<K, V> {
    fn from(items: HashMap<K, V>) -> Self {
        Dict { items }
    }
}

impl<K: Eq + Hash, V> From<Dict<K, V>> for HashMap<K, V> {
    fn from(dict: Dict<K, V>) -> Self {
        dict.items
    }
}

impl<K: Eq + Hash, V> Default for Dict<K, V> {
    fn default() -> Self {
        Dict::new()
    }
}

// ============================================================================
// Tests for Dict<K,V>
// ============================================================================

#[cfg(test)]
mod dict_tests {
    use super::*;

    #[test]
    fn test_dict_new() {
        let dict: Dict<String, i32> = Dict::new();
        assert!(dict.is_empty());
        assert_eq!(dict.len(), 0);
    }

    #[test]
    fn test_dict_with_capacity() {
        let dict: Dict<String, i32> = Dict::with_capacity(10);
        assert!(dict.is_empty());
    }

    #[test]
    fn test_dict_insert_get_remove() {
        let mut dict = Dict::new();
        assert_eq!(dict.insert("key1", 100), None);  // First insert succeeds
        assert_eq!(dict.insert("key1", 200), Some(100)); // Update returns old value

        assert_eq!(dict.get(&"key1"), Some(&200));
        assert_eq!(dict.get(&"nonexistent"), None);

        assert_eq!(dict.remove(&"key1"), Some(200));  // Remove succeeds
        assert_eq!(dict.remove(&"key1"), None);  // Remove again fails
        assert_eq!(dict.get(&"key1"), None);
    }

    #[test]
    fn test_dict_contains_key() {
        let mut dict = Dict::new();
        dict.insert("exists", 42);

        assert!(dict.contains_key(&"exists"));
        assert!(!dict.contains_key(&"nonexistent"));
    }

    #[test]
    fn test_dict_from_iter() {
        let dict = Dict::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
        assert_eq!(dict.len(), 3);
        assert_eq!(dict.get(&"a"), Some(&1));
        assert_eq!(dict.get(&"b"), Some(&2));
        assert_eq!(dict.get(&"c"), Some(&3));
    }

    #[test]
    fn test_dict_from_pairs() {
        let pairs = vec![("x", 10), ("y", 20)];
        let dict = Dict::from_pairs(pairs);

        assert_eq!(dict.len(), 2);
        assert_eq!(dict.get(&"x"), Some(&10));
        assert_eq!(dict.get(&"y"), Some(&20));
    }

    #[test]
    fn test_dict_keys_values_entries() {
        let mut dict = Dict::new();
        dict.insert("a", 1);
        dict.insert("b", 2);

        let keys = dict.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&&"a"));
        assert!(keys.contains(&&"b"));

        let values = dict.values();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&&1));
        assert!(values.contains(&&2));

        let entries = dict.entries();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&(&"a", &1)));
        assert!(entries.contains(&(&"b", &2)));
    }

    #[test]
    fn test_dict_get_or_default() {
        let mut dict = Dict::new();
        dict.insert("exists", 42);

        assert_eq!(dict.get_or_default(&"exists", 99), 42);
        assert_eq!(dict.get_or_default(&"missing", 99), 99);
    }

    #[test]
    fn test_dict_insert_if_absent() {
        let mut dict = Dict::new();
        dict.insert("existing", 100);

        assert_eq!(dict.insert_if_absent("existing", 200), false); // Should not insert
        assert_eq!(dict.get(&"existing"), Some(&100)); // Value unchanged

        assert_eq!(dict.insert_if_absent("new", 300), true); // Should insert
        assert_eq!(dict.get(&"new"), Some(&300));
    }

    #[test]
    fn test_dict_update() {
        let mut dict = Dict::new();
        dict.insert("counter", 5);

        assert_eq!(dict.update(&"counter", |x| x * 2), true); // Should update
        assert_eq!(dict.get(&"counter"), Some(&10));

        assert_eq!(dict.update(&"missing", |x| x + 1), false); // Should not update
    }

    #[test]
    fn test_dict_merge() {
        let mut dict1 = Dict::new();
        dict1.insert("a", 1);
        dict1.insert("b", 2);

        let mut dict2 = Dict::new();
        dict2.insert("b", 20); // Overwrite
        dict2.insert("c", 30); // New

        dict1.merge(dict2);

        assert_eq!(dict1.len(), 3);
        assert_eq!(dict1.get(&"a"), Some(&1));
        assert_eq!(dict1.get(&"b"), Some(&20)); // Overwritten
        assert_eq!(dict1.get(&"c"), Some(&30));
    }

    #[test]
    fn test_dict_merge_new() {
        let mut dict1 = Dict::new();
        dict1.insert("a", 1);
        dict1.insert("b", 2);

        let mut dict2 = Dict::new();
        dict2.insert("b", 20); // Should not overwrite
        dict2.insert("c", 30); // Should add

        dict1.merge_new(dict2);

        assert_eq!(dict1.len(), 3);
        assert_eq!(dict1.get(&"a"), Some(&1));
        assert_eq!(dict1.get(&"b"), Some(&2)); // Not overwritten
        assert_eq!(dict1.get(&"c"), Some(&30));
    }

    #[test]
    fn test_dict_map_values() {
        let mut dict = Dict::new();
        dict.insert("a", 1);
        dict.insert("b", 2);
        dict.insert("c", 3);

        let doubled = dict.map_values(|x| x * 2);
        assert_eq!(doubled.len(), 3);
        assert_eq!(doubled.get(&"a"), Some(&2));
        assert_eq!(doubled.get(&"b"), Some(&4));
        assert_eq!(doubled.get(&"c"), Some(&6));
    }

    #[test]
    fn test_dict_filter() {
        let mut dict = Dict::new();
        dict.insert("a", 1);
        dict.insert("b", 2);
        dict.insert("c", 3);
        dict.insert("d", 4);

        let evens = dict.filter(|_k, v| v % 2 == 0);
        assert_eq!(evens.len(), 2);
        assert_eq!(evens.get(&"b"), Some(&2));
        assert_eq!(evens.get(&"d"), Some(&4));
    }

    #[test]
    fn test_dict_find() {
        let mut dict = Dict::new();
        dict.insert("a", 1);
        dict.insert("b", 2);
        dict.insert("c", 3);

        let found = dict.find(|_k, v| *v > 2);
        assert_eq!(found, Some((&"c", &3)));
    }

    #[test]
    fn test_dict_some_every() {
        let mut dict = Dict::new();
        dict.insert("a", 2);
        dict.insert("b", 4);
        dict.insert("c", 6);

        assert!(dict.every(|_k, v| *v % 2 == 0));
        assert!(dict.some(|_k, v| *v > 5));

        let mut mixed_dict = Dict::new();
        mixed_dict.insert("a", 1);
        mixed_dict.insert("b", 2);
        mixed_dict.insert("c", 3);

        assert!(!mixed_dict.every(|_k, v| *v % 2 == 0));
        assert!(mixed_dict.some(|_k, v| *v % 2 == 0));
    }

    #[test]
    fn test_dict_clear() {
        let mut dict = Dict::new();
        dict.insert("a", 1);
        dict.insert("b", 2);
        assert_eq!(dict.len(), 2);

        dict.clear();
        assert!(dict.is_empty());
        assert_eq!(dict.len(), 0);
    }

    #[test]
    fn test_dict_display() {
        let mut dict = Dict::new();
        dict.insert("a", 1);
        dict.insert("b", 2);

        let display = format!("{}", dict);
        // HashMap order is not guaranteed, so we just check it contains the elements
        assert!(display.contains("a: 1"));
        assert!(display.contains("b: 2"));
        assert!(display.starts_with("{"));
        assert!(display.ends_with("}"));
    }

    #[test]
    fn test_dict_empty_display() {
        let dict: Dict<String, i32> = Dict::new();
        assert_eq!(format!("{}", dict), "{}");
    }

    #[test]
    fn test_dict_into_pairs() {
        let mut dict = Dict::new();
        dict.insert("a", 1);
        dict.insert("b", 2);

        let pairs = dict.into_pairs();
        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&("a", 1)));
        assert!(pairs.contains(&("b", 2)));
    }

    #[test]
    fn test_dict_get_mut() {
        let mut dict = Dict::new();
        dict.insert("counter", 5);

        if let Some(value) = dict.get_mut(&"counter") {
            *value += 1;
        }

        assert_eq!(dict.get(&"counter"), Some(&6));
    }
}