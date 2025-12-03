/*!
# VelaList

Dynamic array with functional API.

## Design

VelaList wraps Vec<T> and provides immutable operations by default.

## Examples

```rust
use vela_stdlib::VelaList;

let list = VelaList::from(vec![1, 2, 3]);
let doubled = list.map(|x| x * 2);
let sum = list.reduce(|acc, x| acc + x, 0);
```
*/

use std::fmt;

/// Dynamic array
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VelaList<T> {
    items: Vec<T>,
}

impl<T> VelaList<T> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create empty list
    pub fn new() -> Self {
        VelaList { items: Vec::new() }
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        VelaList {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Create from vector
    pub fn from_vec(items: Vec<T>) -> Self {
        VelaList { items }
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

    /// Get element at index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// Get first element
    pub fn first(&self) -> Option<&T> {
        self.items.first()
    }

    /// Get last element
    pub fn last(&self) -> Option<&T> {
        self.items.last()
    }

    // ============================================================================
    // Modifications (create new list)
    // ============================================================================

    /// Push element (returns new list)
    pub fn push(&self, item: T) -> Self
    where
        T: Clone,
    {
        let mut new_items = self.items.clone();
        new_items.push(item);
        VelaList { items: new_items }
    }

    /// Pop last element (returns new list + element)
    pub fn pop(&self) -> Option<(Self, T)>
    where
        T: Clone,
    {
        if self.items.is_empty() {
            return None;
        }

        let mut new_items = self.items.clone();
        let popped = new_items.pop()?;
        Some((VelaList { items: new_items }, popped))
    }

    /// Insert at index (returns new list)
    pub fn insert(&self, index: usize, item: T) -> Self
    where
        T: Clone,
    {
        let mut new_items = self.items.clone();
        new_items.insert(index, item);
        VelaList { items: new_items }
    }

    /// Remove at index (returns new list)
    pub fn remove(&self, index: usize) -> Option<Self>
    where
        T: Clone,
    {
        if index >= self.items.len() {
            return None;
        }

        let mut new_items = self.items.clone();
        new_items.remove(index);
        Some(VelaList { items: new_items })
    }

    /// Reverse order (returns new list)
    pub fn reverse(&self) -> Self
    where
        T: Clone,
    {
        let mut new_items = self.items.clone();
        new_items.reverse();
        VelaList { items: new_items }
    }

    /// Sort (returns new list)
    pub fn sort(&self) -> Self
    where
        T: Clone + Ord,
    {
        let mut new_items = self.items.clone();
        new_items.sort();
        VelaList { items: new_items }
    }

    // ============================================================================
    // Functional API
    // ============================================================================

    /// Map elements
    pub fn map<U, F>(&self, f: F) -> VelaList<U>
    where
        F: FnMut(&T) -> U,
    {
        VelaList {
            items: self.items.iter().map(f).collect(),
        }
    }

    /// Filter elements
    pub fn filter<F>(&self, mut f: F) -> Self
    where
        T: Clone,
        F: FnMut(&T) -> bool,
    {
        VelaList {
            items: self.items.iter().filter(|x| f(x)).cloned().collect(),
        }
    }

    /// Reduce to single value
    pub fn reduce<U, F>(&self, f: F, init: U) -> U
    where
        F: FnMut(U, &T) -> U,
    {
        self.items.iter().fold(init, f)
    }

    /// For each element (side effects)
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        self.items.iter().for_each(|x| f(x));
    }

    /// Find first matching element
    pub fn find<F>(&self, mut f: F) -> Option<&T>
    where
        F: FnMut(&T) -> bool,
    {
        self.items.iter().find(|x| f(x))
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

    /// Take first N elements
    pub fn take(&self, n: usize) -> Self
    where
        T: Clone,
    {
        VelaList {
            items: self.items.iter().take(n).cloned().collect(),
        }
    }

    /// Skip first N elements
    pub fn skip(&self, n: usize) -> Self
    where
        T: Clone,
    {
        VelaList {
            items: self.items.iter().skip(n).cloned().collect(),
        }
    }

    // ============================================================================
    // Queries
    // ============================================================================

    /// Check if contains element
    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.items.contains(item)
    }

    /// Find index of element
    pub fn index_of(&self, item: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.items.iter().position(|x| x == item)
    }

    // ============================================================================
    // Combining
    // ============================================================================

    /// Concatenate two lists
    pub fn concat(&self, other: &VelaList<T>) -> Self
    where
        T: Clone,
    {
        let mut new_items = self.items.clone();
        new_items.extend(other.items.iter().cloned());
        VelaList { items: new_items }
    }

    /// Join elements to string
    pub fn join(&self, separator: &str) -> String
    where
        T: fmt::Display,
    {
        self.items
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(separator)
    }

    // ============================================================================
    // Conversions
    // ============================================================================

    /// Convert to vector
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.items.clone()
    }
}

impl<T> Default for VelaList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Display> fmt::Display for VelaList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

impl<T> From<Vec<T>> for VelaList<T> {
    fn from(items: Vec<T>) -> Self {
        VelaList { items }
    }
}

impl<T: Clone> From<VelaList<T>> for Vec<T> {
    fn from(list: VelaList<T>) -> Self {
        list.items
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
        let list: VelaList<i32> = VelaList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);

        let list = VelaList::from(vec![1, 2, 3]);
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_get() {
        let list = VelaList::from(vec![10, 20, 30]);
        assert_eq!(list.get(0), Some(&10));
        assert_eq!(list.get(1), Some(&20));
        assert_eq!(list.get(3), None);

        assert_eq!(list.first(), Some(&10));
        assert_eq!(list.last(), Some(&30));
    }

    #[test]
    fn test_push() {
        let list = VelaList::from(vec![1, 2]);
        let list2 = list.push(3);

        assert_eq!(list.len(), 2); // Original unchanged
        assert_eq!(list2.len(), 3);
        assert_eq!(list2.get(2), Some(&3));
    }

    #[test]
    fn test_pop() {
        let list = VelaList::from(vec![1, 2, 3]);
        let (list2, popped) = list.pop().unwrap();

        assert_eq!(popped, 3);
        assert_eq!(list2.len(), 2);
        assert_eq!(list.len(), 3); // Original unchanged
    }

    #[test]
    fn test_insert() {
        let list = VelaList::from(vec![1, 3]);
        let list2 = list.insert(1, 2);

        assert_eq!(list2.get(1), Some(&2));
        assert_eq!(list2.len(), 3);
    }

    #[test]
    fn test_remove() {
        let list = VelaList::from(vec![1, 2, 3]);
        let list2 = list.remove(1).unwrap();

        assert_eq!(list2.len(), 2);
        assert_eq!(list2.get(1), Some(&3));
    }

    #[test]
    fn test_map() {
        let list = VelaList::from(vec![1, 2, 3]);
        let doubled = list.map(|x| x * 2);

        assert_eq!(doubled.get(0), Some(&2));
        assert_eq!(doubled.get(1), Some(&4));
        assert_eq!(doubled.get(2), Some(&6));
    }

    #[test]
    fn test_filter() {
        let list = VelaList::from(vec![1, 2, 3, 4, 5]);
        let evens = list.filter(|x| *x % 2 == 0);

        assert_eq!(evens.len(), 2);
        assert_eq!(evens.get(0), Some(&2));
        assert_eq!(evens.get(1), Some(&4));
    }

    #[test]
    fn test_reduce() {
        let list = VelaList::from(vec![1, 2, 3, 4]);
        let sum = list.reduce(|acc, x| acc + x, 0);

        assert_eq!(sum, 10);
    }

    #[test]
    fn test_find() {
        let list = VelaList::from(vec![1, 2, 3, 4]);
        let found = list.find(|x| *x > 2);

        assert_eq!(found, Some(&3));
    }

    #[test]
    fn test_any_all() {
        let list = VelaList::from(vec![2, 4, 6]);

        assert!(list.all(|x| *x % 2 == 0));
        assert!(list.any(|x| *x > 4));
        assert!(!list.any(|x| *x > 10));
    }

    #[test]
    fn test_take_skip() {
        let list = VelaList::from(vec![1, 2, 3, 4, 5]);

        let first_two = list.take(2);
        assert_eq!(first_two.len(), 2);
        assert_eq!(first_two.get(0), Some(&1));

        let skip_two = list.skip(2);
        assert_eq!(skip_two.len(), 3);
        assert_eq!(skip_two.get(0), Some(&3));
    }

    #[test]
    fn test_contains() {
        let list = VelaList::from(vec![1, 2, 3]);

        assert!(list.contains(&2));
        assert!(!list.contains(&5));
    }

    #[test]
    fn test_concat() {
        let list1 = VelaList::from(vec![1, 2]);
        let list2 = VelaList::from(vec![3, 4]);
        let combined = list1.concat(&list2);

        assert_eq!(combined.len(), 4);
        assert_eq!(combined.get(2), Some(&3));
    }

    #[test]
    fn test_join() {
        let list = VelaList::from(vec![1, 2, 3]);
        let joined = list.join(", ");

        assert_eq!(joined, "1, 2, 3");
    }

    #[test]
    fn test_reverse() {
        let list = VelaList::from(vec![1, 2, 3]);
        let reversed = list.reverse();

        assert_eq!(reversed.get(0), Some(&3));
        assert_eq!(reversed.get(2), Some(&1));
    }

    #[test]
    fn test_sort() {
        let list = VelaList::from(vec![3, 1, 2]);
        let sorted = list.sort();

        assert_eq!(sorted.get(0), Some(&1));
        assert_eq!(sorted.get(1), Some(&2));
        assert_eq!(sorted.get(2), Some(&3));
    }
}
