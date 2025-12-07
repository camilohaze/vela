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
// Mutable Set<T> - Main Vela Collection
// ============================================================================

/// Mutable hash-based set - Vela's primary set collection type
/// Inspired by Rust's HashSet<T>, Swift's Set<T>, and Kotlin's MutableSet<T>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set<T>
where
    T: Eq + Hash,
{
    items: HashSet<T>,
}

impl<T: Eq + Hash> Set<T> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create empty set
    pub fn new() -> Self {
        Set {
            items: HashSet::new(),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Set {
            items: HashSet::with_capacity(capacity),
        }
    }

    /// Create from iterator
    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Set {
            items: iter.into_iter().collect(),
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

    /// Insert element, returns true if it was not already present
    pub fn insert(&mut self, item: T) -> bool {
        self.items.insert(item)
    }

    /// Remove element, returns true if it was present
    pub fn remove(&mut self, item: &T) -> bool {
        self.items.remove(item)
    }

    /// Check if contains element
    pub fn contains(&self, item: &T) -> bool {
        self.items.contains(item)
    }

    /// Clear all elements
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
    // Set Operations (return new sets)
    // ============================================================================

    /// Union: elements in self or other
    pub fn union(&self, other: &Set<T>) -> Set<T>
    where
        T: Clone,
    {
        let mut result = self.clone();
        result.items.extend(other.items.iter().cloned());
        result
    }

    /// Intersection: elements in both self and other
    pub fn intersection(&self, other: &Set<T>) -> Set<T>
    where
        T: Clone,
    {
        let mut result = Set::new();
        for item in &self.items {
            if other.contains(item) {
                result.items.insert(item.clone());
            }
        }
        result
    }

    /// Difference: elements in self but not in other
    pub fn difference(&self, other: &Set<T>) -> Set<T>
    where
        T: Clone,
    {
        let mut result = Set::new();
        for item in &self.items {
            if !other.contains(item) {
                result.items.insert(item.clone());
            }
        }
        result
    }

    /// Symmetric difference: elements in either self or other, but not both
    pub fn symmetric_difference(&self, other: &Set<T>) -> Set<T>
    where
        T: Clone,
    {
        let mut result = Set::new();
        // Add elements in self but not in other
        for item in &self.items {
            if !other.contains(item) {
                result.items.insert(item.clone());
            }
        }
        // Add elements in other but not in self
        for item in &other.items {
            if !self.contains(item) {
                result.items.insert(item.clone());
            }
        }
        result
    }

    // ============================================================================
    // Set Predicates
    // ============================================================================

    /// Check if self is subset of other
    pub fn is_subset(&self, other: &Set<T>) -> bool {
        self.items.is_subset(&other.items)
    }

    /// Check if self is superset of other
    pub fn is_superset(&self, other: &Set<T>) -> bool {
        self.items.is_superset(&other.items)
    }

    /// Check if self and other have no elements in common
    pub fn is_disjoint(&self, other: &Set<T>) -> bool {
        self.items.is_disjoint(&other.items)
    }

    // ============================================================================
    // Functional Operations
    // ============================================================================

    /// Map elements to new set
    pub fn map<U, F>(&self, f: F) -> Set<U>
    where
        F: Fn(&T) -> U,
        U: Eq + Hash,
    {
        Set {
            items: self.items.iter().map(f).collect(),
        }
    }

    /// Filter elements
    pub fn filter<F>(&self, f: F) -> Set<T>
    where
        T: Clone,
        F: Fn(&T) -> bool,
    {
        Set {
            items: self.items.iter().filter(|x| f(x)).cloned().collect(),
        }
    }

    /// Execute function for each element
    pub fn for_each<F>(&self, f: F)
    where
        F: Fn(&T),
    {
        self.items.iter().for_each(f);
    }

    /// Find first element matching predicate
    pub fn find<F>(&self, f: F) -> Option<&T>
    where
        F: Fn(&T) -> bool,
    {
        self.items.iter().find(|x| f(x))
    }

    /// Check if any element matches predicate
    pub fn some<F>(&self, f: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        self.items.iter().any(f)
    }

    /// Check if all elements match predicate
    pub fn every<F>(&self, f: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        self.items.iter().all(f)
    }

    // ============================================================================
    // Conversion Operations
    // ============================================================================

    /// Convert to vector
    pub fn to_vec(self) -> Vec<T> {
        self.items.into_iter().collect()
    }

    /// Create from slice
    pub fn from_slice(slice: &[T]) -> Set<T>
    where
        T: Clone,
    {
        Set {
            items: slice.iter().cloned().collect(),
        }
    }
}

impl<T: Eq + Hash + fmt::Display> fmt::Display for Set<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for item in &self.items {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
            first = false;
        }
        write!(f, "}}")
    }
}

impl<T: Eq + Hash> From<HashSet<T>> for Set<T> {
    fn from(items: HashSet<T>) -> Self {
        Set { items }
    }
}

impl<T: Eq + Hash> From<Set<T>> for HashSet<T> {
    fn from(set: Set<T>) -> Self {
        set.items
    }
}

impl<T: Eq + Hash> Default for Set<T> {
    fn default() -> Self {
        Set::new()
    }
}

// ============================================================================
// Tests for Set<T>
// ============================================================================

#[cfg(test)]
mod set_tests {
    use super::*;

    #[test]
    fn test_set_new() {
        let set: Set<i32> = Set::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_set_with_capacity() {
        let set: Set<i32> = Set::with_capacity(10);
        assert!(set.is_empty());
        // Note: HashSet doesn't expose capacity, so we can't test it directly
    }

    #[test]
    fn test_set_insert_remove() {
        let mut set = Set::new();
        assert_eq!(set.insert(1), true);  // First insert succeeds
        assert_eq!(set.insert(1), false); // Duplicate insert fails
        assert_eq!(set.len(), 1);
        assert!(set.contains(&1));

        assert_eq!(set.remove(&1), true);  // Remove succeeds
        assert_eq!(set.remove(&1), false); // Remove again fails
        assert_eq!(set.len(), 0);
        assert!(!set.contains(&1));
    }

    #[test]
    fn test_set_from_iter() {
        let set = Set::from_iter(vec![1, 2, 2, 3, 3, 3]);
        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
    }

    #[test]
    fn test_set_union() {
        let mut set1 = Set::new();
        set1.insert(1);
        set1.insert(2);

        let mut set2 = Set::new();
        set2.insert(2);
        set2.insert(3);

        let union = set1.union(&set2);
        assert_eq!(union.len(), 3);
        assert!(union.contains(&1));
        assert!(union.contains(&2));
        assert!(union.contains(&3));
    }

    #[test]
    fn test_set_intersection() {
        let mut set1 = Set::new();
        set1.insert(1);
        set1.insert(2);

        let mut set2 = Set::new();
        set2.insert(2);
        set2.insert(3);

        let intersection = set1.intersection(&set2);
        assert_eq!(intersection.len(), 1);
        assert!(intersection.contains(&2));
        assert!(!intersection.contains(&1));
        assert!(!intersection.contains(&3));
    }

    #[test]
    fn test_set_difference() {
        let mut set1 = Set::new();
        set1.insert(1);
        set1.insert(2);

        let mut set2 = Set::new();
        set2.insert(2);
        set2.insert(3);

        let difference = set1.difference(&set2);
        assert_eq!(difference.len(), 1);
        assert!(difference.contains(&1));
        assert!(!difference.contains(&2));
        assert!(!difference.contains(&3));
    }

    #[test]
    fn test_set_symmetric_difference() {
        let mut set1 = Set::new();
        set1.insert(1);
        set1.insert(2);

        let mut set2 = Set::new();
        set2.insert(2);
        set2.insert(3);

        let sym_diff = set1.symmetric_difference(&set2);
        assert_eq!(sym_diff.len(), 2);
        assert!(sym_diff.contains(&1));
        assert!(!sym_diff.contains(&2));
        assert!(sym_diff.contains(&3));
    }

    #[test]
    fn test_set_predicates() {
        let mut set1 = Set::new();
        set1.insert(1);
        set1.insert(2);

        let mut set2 = Set::new();
        set2.insert(1);
        set2.insert(2);
        set2.insert(3);

        let mut set3 = Set::new();
        set3.insert(3);
        set3.insert(4);

        // set1 is subset of set2
        assert!(set1.is_subset(&set2));
        assert!(!set2.is_subset(&set1));

        // set2 is superset of set1
        assert!(set2.is_superset(&set1));
        assert!(!set1.is_superset(&set2));

        // set1 and set3 are disjoint
        assert!(set1.is_disjoint(&set3));
        assert!(!set1.is_disjoint(&set2));
    }

    #[test]
    fn test_set_map() {
        let mut set = Set::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let doubled = set.map(|x| x * 2);
        assert_eq!(doubled.len(), 3);
        assert!(doubled.contains(&2));
        assert!(doubled.contains(&4));
        assert!(doubled.contains(&6));
    }

    #[test]
    fn test_set_filter() {
        let mut set = Set::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);
        set.insert(4);

        let evens = set.filter(|x| x % 2 == 0);
        assert_eq!(evens.len(), 2);
        assert!(evens.contains(&2));
        assert!(evens.contains(&4));
    }

    #[test]
    fn test_set_find() {
        let mut set = Set::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let found = set.find(|x| *x > 2);
        assert_eq!(found, Some(&3));
    }

    #[test]
    fn test_set_some_every() {
        let mut set = Set::new();
        set.insert(2);
        set.insert(4);
        set.insert(6);

        assert!(set.every(|x| *x % 2 == 0));
        assert!(set.some(|x| *x > 5));

        let mut mixed_set = Set::new();
        mixed_set.insert(1);
        mixed_set.insert(2);
        mixed_set.insert(3);

        assert!(!mixed_set.every(|x| *x % 2 == 0));
        assert!(mixed_set.some(|x| *x % 2 == 0));
    }

    #[test]
    fn test_set_clear() {
        let mut set = Set::new();
        set.insert(1);
        set.insert(2);
        assert_eq!(set.len(), 2);

        set.clear();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_set_display() {
        let mut set = Set::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let display = format!("{}", set);
        // HashSet order is not guaranteed, so we just check it contains the elements
        assert!(display.contains("1"));
        assert!(display.contains("2"));
        assert!(display.contains("3"));
        assert!(display.starts_with("{"));
        assert!(display.ends_with("}"));
    }

    #[test]
    fn test_set_empty_display() {
        let set: Set<i32> = Set::new();
        assert_eq!(format!("{}", set), "{}");
    }

    #[test]
    fn test_set_to_vec() {
        let mut set = Set::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let vec = set.to_vec();
        assert_eq!(vec.len(), 3);
        assert!(vec.contains(&1));
        assert!(vec.contains(&2));
        assert!(vec.contains(&3));
    }

    #[test]
    fn test_set_from_slice() {
        let slice = [1, 2, 2, 3, 3, 3];
        let set = Set::from_slice(&slice);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
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
