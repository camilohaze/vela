/*!
# VelaMap

Hash map with key-value storage.

## Design

VelaMap wraps HashMap<K,V> and provides immutable operations.

## Examples

```rust
use vela_stdlib::VelaMap;

let map = VelaMap::new();
let map2 = map.insert("name", "Vela");
let value = map2.get(&"name");
```
*/

use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// Hash map
#[derive(Debug, Clone)]
pub struct VelaMap<K, V>
where
    K: Eq + Hash,
{
    items: HashMap<K, V>,
}

impl<K: Eq + Hash, V> VelaMap<K, V> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create empty map
    pub fn new() -> Self {
        VelaMap {
            items: HashMap::new(),
        }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        VelaMap {
            items: HashMap::with_capacity(capacity),
        }
    }

    /// Create from hash map
    pub fn from_hashmap(items: HashMap<K, V>) -> Self {
        VelaMap { items }
    }

    // ============================================================================
    // Inspections
    // ============================================================================

    /// Number of entries
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.items.get(key)
    }

    /// Check if contains key
    pub fn contains_key(&self, key: &K) -> bool {
        self.items.contains_key(key)
    }

    // ============================================================================
    // Modifications (create new map)
    // ============================================================================

    /// Insert key-value (returns new map)
    pub fn insert(&self, key: K, value: V) -> Self
    where
        K: Clone,
        V: Clone,
    {
        let mut new_items = self.items.clone();
        new_items.insert(key, value);
        VelaMap { items: new_items }
    }

    /// Remove key (returns new map)
    pub fn remove(&self, key: &K) -> Self
    where
        K: Clone,
        V: Clone,
    {
        let mut new_items = self.items.clone();
        new_items.remove(key);
        VelaMap { items: new_items }
    }

    // ============================================================================
    // Functional API
    // ============================================================================

    /// Map values
    pub fn map<U, F>(&self, mut f: F) -> VelaMap<K, U>
    where
        K: Clone,
        F: FnMut(&K, &V) -> U,
    {
        VelaMap {
            items: self
                .items
                .iter()
                .map(|(k, v)| (k.clone(), f(k, v)))
                .collect(),
        }
    }

    /// Filter entries
    pub fn filter<F>(&self, mut f: F) -> Self
    where
        K: Clone,
        V: Clone,
        F: FnMut(&K, &V) -> bool,
    {
        VelaMap {
            items: self
                .items
                .iter()
                .filter(|(k, v)| f(k, v))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    /// For each entry (side effects)
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&K, &V),
    {
        self.items.iter().for_each(|(k, v)| f(k, v));
    }

    // ============================================================================
    // Collections
    // ============================================================================

    /// Get all keys
    pub fn keys(&self) -> Vec<&K> {
        self.items.keys().collect()
    }

    /// Get all values
    pub fn values(&self) -> Vec<&V> {
        self.items.values().collect()
    }

    /// Get all entries as tuples
    pub fn entries(&self) -> Vec<(&K, &V)> {
        self.items.iter().collect()
    }

    // ============================================================================
    // Conversions
    // ============================================================================

    /// Convert to hash map
    pub fn to_hashmap(&self) -> HashMap<K, V>
    where
        K: Clone,
        V: Clone,
    {
        self.items.clone()
    }
}

impl<K: Eq + Hash, V> Default for VelaMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + fmt::Display, V: fmt::Display> fmt::Display for VelaMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, (key, value)) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", key, value)?;
        }
        write!(f, "}}")
    }
}

impl<K: Eq + Hash, V> From<HashMap<K, V>> for VelaMap<K, V> {
    fn from(items: HashMap<K, V>) -> Self {
        VelaMap { items }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> From<VelaMap<K, V>> for HashMap<K, V> {
    fn from(map: VelaMap<K, V>) -> Self {
        map.items
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
        let map: VelaMap<String, i32> = VelaMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_insert_get() {
        let map = VelaMap::new();
        let map2 = map.insert("age", 37);
        let map3 = map2.insert("score", 100);

        assert_eq!(map3.get(&"age"), Some(&37));
        assert_eq!(map3.get(&"score"), Some(&100));
        assert_eq!(map3.len(), 2);

        // Original unchanged
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_contains_key() {
        let map = VelaMap::new();
        let map2 = map.insert("name", "Vela");

        assert!(map2.contains_key(&"name"));
        assert!(!map2.contains_key(&"age"));
    }

    #[test]
    fn test_remove() {
        let map = VelaMap::new();
        let map2 = map.insert("a", 1).insert("b", 2);
        let map3 = map2.remove(&"a");

        assert!(!map3.contains_key(&"a"));
        assert!(map3.contains_key(&"b"));
        assert_eq!(map3.len(), 1);

        // Original unchanged
        assert_eq!(map2.len(), 2);
    }

    #[test]
    fn test_map_values() {
        let map = VelaMap::new();
        let map2 = map.insert("a", 1).insert("b", 2);
        let map3 = map2.map(|_, v| v * 2);

        assert_eq!(map3.get(&"a"), Some(&2));
        assert_eq!(map3.get(&"b"), Some(&4));
    }

    #[test]
    fn test_filter() {
        let map = VelaMap::new();
        let map2 = map.insert("a", 1).insert("b", 2).insert("c", 3);
        let map3 = map2.filter(|_, v| *v % 2 == 0);

        assert_eq!(map3.len(), 1);
        assert!(map3.contains_key(&"b"));
    }

    #[test]
    fn test_keys_values() {
        let map = VelaMap::new();
        let map2 = map.insert("a", 1).insert("b", 2);

        let mut keys = map2.keys();
        keys.sort();
        assert_eq!(keys, vec![&"a", &"b"]);

        let mut values = map2.values();
        values.sort();
        assert_eq!(values, vec![&1, &2]);
    }

    #[test]
    fn test_entries() {
        let map = VelaMap::new();
        let map2 = map.insert("a", 1);

        let entries = map2.entries();
        assert_eq!(entries.len(), 1);
        assert!(entries.contains(&(&"a", &1)));
    }

    #[test]
    fn test_for_each() {
        let map = VelaMap::new();
        let map2 = map.insert("a", 1).insert("b", 2);

        let mut sum = 0;
        map2.for_each(|_, v| sum += v);

        assert_eq!(sum, 3);
    }
}
