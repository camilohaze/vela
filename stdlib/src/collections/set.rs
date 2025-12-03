/*!
# VelaSet

Unique element storage.

## Design

VelaSet wraps HashSet<T> and provides immutable operations.

## Examples

```rust
use vela_stdlib::VelaSet;

let set = VelaSet::new();
let set2 = set.insert(1).insert(2);
let has_one = set2.contains(&1);
```
*/

use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;

/// Unique element storage
#[derive(Debug, Clone)]
pub struct VelaSet<T>
where
    T: Eq + Hash,
{
    items: HashSet<T>,
}

impl<T: Eq + Hash> VelaSet<T> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create empty set
    pub fn new() -> Self {
        VelaSet {
            items: HashSet::new(),
        }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        VelaSet {
            items: HashSet::with_capacity(capacity),
        }
    }

    /// Create from hash set
    pub fn from_hashset(items: HashSet<T>) -> Self {
        VelaSet { items }
    }

    // ============================================================================
    // Inspections
    // ============================================================================

    /// Number of elements
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Check if contains element
    pub fn contains(&self, item: &T) -> bool {
        self.items.contains(item)
    }

    // ============================================================================
    // Modifications (create new set)
    // ============================================================================

    /// Insert element (returns new set)
    pub fn insert(&self, item: T) -> Self
    where
        T: Clone,
    {
        let mut new_items = self.items.clone();
        new_items.insert(item);
        VelaSet { items: new_items }
    }

    /// Remove element (returns new set)
    pub fn remove(&self, item: &T) -> Self
    where
        T: Clone,
    {
        let mut new_items = self.items.clone();
        new_items.remove(item);
        VelaSet { items: new_items }
    }

    // ============================================================================
    // Set operations
    // ============================================================================

    /// Union with another set
    pub fn union(&self, other: &VelaSet<T>) -> Self
    where
        T: Clone,
    {
        VelaSet {
            items: self.items.union(&other.items).cloned().collect(),
        }
    }

    /// Intersection with another set
    pub fn intersection(&self, other: &VelaSet<T>) -> Self
    where
        T: Clone,
    {
        VelaSet {
            items: self.items.intersection(&other.items).cloned().collect(),
        }
    }

    /// Difference with another set
    pub fn difference(&self, other: &VelaSet<T>) -> Self
    where
        T: Clone,
    {
        VelaSet {
            items: self.items.difference(&other.items).cloned().collect(),
        }
    }

    /// Symmetric difference
    pub fn symmetric_difference(&self, other: &VelaSet<T>) -> Self
    where
        T: Clone,
    {
        VelaSet {
            items: self
                .items
                .symmetric_difference(&other.items)
                .cloned()
                .collect(),
        }
    }

    /// Check if subset
    pub fn is_subset(&self, other: &VelaSet<T>) -> bool {
        self.items.is_subset(&other.items)
    }

    /// Check if superset
    pub fn is_superset(&self, other: &VelaSet<T>) -> bool {
        self.items.is_superset(&other.items)
    }

    /// Check if disjoint
    pub fn is_disjoint(&self, other: &VelaSet<T>) -> bool {
        self.items.is_disjoint(&other.items)
    }

    // ============================================================================
    // Functional API
    // ============================================================================

    /// Map elements
    pub fn map<U, F>(&self, f: F) -> VelaSet<U>
    where
        U: Eq + Hash,
        F: FnMut(&T) -> U,
    {
        VelaSet {
            items: self.items.iter().map(f).collect(),
        }
    }

    /// Filter elements
    pub fn filter<F>(&self, mut f: F) -> Self
    where
        T: Clone,
        F: FnMut(&T) -> bool,
    {
        VelaSet {
            items: self.items.iter().filter(|x| f(x)).cloned().collect(),
        }
    }

    /// For each element (side effects)
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        self.items.iter().for_each(|x| f(x));
    }

    /// Check if any element matches
    pub fn any<F>(&self, mut f: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        self.items.iter().any(|x| f(x))
    }

    /// Check if all elements match
    pub fn all<F>(&self, mut f: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        self.items.iter().all(|x| f(x))
    }

    // ============================================================================
    // Collections
    // ============================================================================

    /// Convert to vector
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.items.iter().cloned().collect()
    }

    // ============================================================================
    // Conversions
    // ============================================================================

    /// Convert to hash set
    pub fn to_hashset(&self) -> HashSet<T>
    where
        T: Clone,
    {
        self.items.clone()
    }
}

impl<T: Eq + Hash> Default for VelaSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + fmt::Display> fmt::Display for VelaSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "}}")
    }
}

impl<T: Eq + Hash> From<HashSet<T>> for VelaSet<T> {
    fn from(items: HashSet<T>) -> Self {
        VelaSet { items }
    }
}

impl<T: Eq + Hash + Clone> From<VelaSet<T>> for HashSet<T> {
    fn from(set: VelaSet<T>) -> Self {
        set.items
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
        let set: VelaSet<i32> = VelaSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_insert() {
        let set = VelaSet::new();
        let set2 = set.insert(1);
        let set3 = set2.insert(2);
        let set4 = set3.insert(1); // Duplicate

        assert_eq!(set4.len(), 2); // Still 2 elements
        assert!(set4.contains(&1));
        assert!(set4.contains(&2));

        // Original unchanged
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_remove() {
        let set = VelaSet::new();
        let set2 = set.insert(1).insert(2).insert(3);
        let set3 = set2.remove(&2);

        assert_eq!(set3.len(), 2);
        assert!(!set3.contains(&2));

        // Original unchanged
        assert_eq!(set2.len(), 3);
    }

    #[test]
    fn test_union() {
        let set1 = VelaSet::new().insert(1).insert(2);
        let set2 = VelaSet::new().insert(2).insert(3);
        let union = set1.union(&set2);

        assert_eq!(union.len(), 3);
        assert!(union.contains(&1));
        assert!(union.contains(&2));
        assert!(union.contains(&3));
    }

    #[test]
    fn test_intersection() {
        let set1 = VelaSet::new().insert(1).insert(2);
        let set2 = VelaSet::new().insert(2).insert(3);
        let intersection = set1.intersection(&set2);

        assert_eq!(intersection.len(), 1);
        assert!(intersection.contains(&2));
    }

    #[test]
    fn test_difference() {
        let set1 = VelaSet::new().insert(1).insert(2);
        let set2 = VelaSet::new().insert(2).insert(3);
        let diff = set1.difference(&set2);

        assert_eq!(diff.len(), 1);
        assert!(diff.contains(&1));
    }

    #[test]
    fn test_symmetric_difference() {
        let set1 = VelaSet::new().insert(1).insert(2);
        let set2 = VelaSet::new().insert(2).insert(3);
        let sym_diff = set1.symmetric_difference(&set2);

        assert_eq!(sym_diff.len(), 2);
        assert!(sym_diff.contains(&1));
        assert!(sym_diff.contains(&3));
    }

    #[test]
    fn test_subset_superset() {
        let set1 = VelaSet::new().insert(1).insert(2);
        let set2 = VelaSet::new().insert(1);

        assert!(set2.is_subset(&set1));
        assert!(set1.is_superset(&set2));
        assert!(!set1.is_subset(&set2));
    }

    #[test]
    fn test_disjoint() {
        let set1 = VelaSet::new().insert(1).insert(2);
        let set2 = VelaSet::new().insert(3).insert(4);
        let set3 = VelaSet::new().insert(2).insert(3);

        assert!(set1.is_disjoint(&set2));
        assert!(!set1.is_disjoint(&set3));
    }

    #[test]
    fn test_filter() {
        let set = VelaSet::new().insert(1).insert(2).insert(3).insert(4);
        let evens = set.filter(|x| *x % 2 == 0);

        assert_eq!(evens.len(), 2);
        assert!(evens.contains(&2));
        assert!(evens.contains(&4));
    }

    #[test]
    fn test_any_all() {
        let set = VelaSet::new().insert(2).insert(4).insert(6);

        assert!(set.all(|x| *x % 2 == 0));
        assert!(set.any(|x| *x > 4));
        assert!(!set.any(|x| *x > 10));
    }

    #[test]
    fn test_to_vec() {
        let set = VelaSet::new().insert(1).insert(2).insert(3);
        let mut vec = set.to_vec();
        vec.sort();

        assert_eq!(vec, vec![1, 2, 3]);
    }
}
