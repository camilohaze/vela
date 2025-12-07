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
// Mutable List<T> - Main Vela Collection
// ============================================================================

/// Mutable dynamic array - Vela's primary collection type
/// Inspired by Rust's Vec<T>, Swift's Array<T>, and Kotlin's MutableList<T>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List<T> {
    items: Vec<T>,
}

impl<T> List<T> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create empty list
    pub fn new() -> Self {
        List { items: Vec::new() }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        List {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Create from vector
    pub fn from_vec(items: Vec<T>) -> Self {
        List { items }
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

    /// Get current capacity
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    // ============================================================================
    // Access Operations
    // ============================================================================

    /// Get element at index (panics if out of bounds)
    pub fn get(&self, index: usize) -> &T {
        &self.items[index]
    }

    /// Get element at index safely
    pub fn get_option(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// Get mutable element at index
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        &mut self.items[index]
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
    // Modification Operations
    // ============================================================================

    /// Add element to end
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Remove and return last element
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Insert element at index
    pub fn insert(&mut self, index: usize, item: T) {
        self.items.insert(index, item);
    }

    /// Remove and return element at index
    pub fn remove(&mut self, index: usize) -> T {
        self.items.remove(index)
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Extend with iterator
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.items.extend(iter);
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
    // Functional Operations (Immutable)
    // ============================================================================

    /// Map elements to new list
    pub fn map<U, F>(&self, f: F) -> List<U>
    where
        F: Fn(&T) -> U,
    {
        List {
            items: self.items.iter().map(f).collect(),
        }
    }

    /// Filter elements
    pub fn filter<F>(&self, f: F) -> List<T>
    where
        T: Clone,
        F: Fn(&T) -> bool,
    {
        List {
            items: self.items.iter().filter(|x| f(x)).cloned().collect(),
        }
    }

    /// Reduce to single value
    pub fn reduce<U, F>(&self, initial: U, f: F) -> U
    where
        F: Fn(U, &T) -> U,
    {
        self.items.iter().fold(initial, f)
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

    /// Take first n elements
    pub fn take(&self, n: usize) -> List<T>
    where
        T: Clone,
    {
        List {
            items: self.items.iter().take(n).cloned().collect(),
        }
    }

    /// Skip first n elements
    pub fn drop(&self, n: usize) -> List<T>
    where
        T: Clone,
    {
        List {
            items: self.items.iter().skip(n).cloned().collect(),
        }
    }

    /// Reverse elements
    pub fn reversed(&self) -> List<T>
    where
        T: Clone,
    {
        let mut items = self.items.clone();
        items.reverse();
        List { items }
    }

    /// Sort elements
    pub fn sorted(&self) -> List<T>
    where
        T: Clone + Ord,
    {
        let mut items = self.items.clone();
        items.sort();
        List { items }
    }

    // ============================================================================
    // Search Operations
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

    /// Find last index of element
    pub fn last_index_of(&self, item: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.items.iter().rposition(|x| x == item)
    }

    // ============================================================================
    // Conversion Operations
    // ============================================================================

    /// Convert to vector
    pub fn to_vec(self) -> Vec<T> {
        self.items
    }

    /// Create from slice
    pub fn from_slice(slice: &[T]) -> List<T>
    where
        T: Clone,
    {
        List {
            items: slice.to_vec(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for List<T> {
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

impl<T> From<Vec<T>> for List<T> {
    fn from(items: Vec<T>) -> Self {
        List { items }
    }
}

impl<T> From<List<T>> for Vec<T> {
    fn from(list: List<T>) -> Self {
        list.items
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        List::new()
    }
}

// ============================================================================
// Tests for List<T>
// ============================================================================

#[cfg(test)]
mod list_tests {
    use super::*;

    #[test]
    fn test_list_new() {
        let list: List<i32> = List::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_list_with_capacity() {
        let list: List<i32> = List::with_capacity(10);
        assert!(list.is_empty());
        assert_eq!(list.capacity(), 10);
    }

    #[test]
    fn test_list_push_pop() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), &1);
        assert_eq!(list.get(1), &2);
        assert_eq!(list.get(2), &3);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop(), Some(2));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn test_list_insert_remove() {
        let mut list = List::from(vec![1, 2, 4]);
        list.insert(2, 3);

        assert_eq!(list.len(), 4);
        assert_eq!(list.get(0), &1);
        assert_eq!(list.get(1), &2);
        assert_eq!(list.get(2), &3);
        assert_eq!(list.get(3), &4);

        let removed = list.remove(2);
        assert_eq!(removed, 3);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(2), &4);
    }

    #[test]
    fn test_list_get_bounds() {
        let list = List::from(vec![10, 20, 30]);

        assert_eq!(list.get_option(0), Some(&10));
        assert_eq!(list.get_option(1), Some(&20));
        assert_eq!(list.get_option(2), Some(&30));
        assert_eq!(list.get_option(3), None);
    }

    #[test]
    fn test_list_map() {
        let list = List::from(vec![1, 2, 3]);
        let doubled = list.map(|x| x * 2);

        assert_eq!(doubled.len(), 3);
        assert_eq!(doubled.get(0), &2);
        assert_eq!(doubled.get(1), &4);
        assert_eq!(doubled.get(2), &6);
    }

    #[test]
    fn test_list_filter() {
        let list = List::from(vec![1, 2, 3, 4, 5, 6]);
        let evens = list.filter(|x| x % 2 == 0);

        assert_eq!(evens.len(), 3);
        assert_eq!(evens.get(0), &2);
        assert_eq!(evens.get(1), &4);
        assert_eq!(evens.get(2), &6);
    }

    #[test]
    fn test_list_reduce() {
        let list = List::from(vec![1, 2, 3, 4]);
        let sum = list.reduce(0, |acc, x| acc + x);

        assert_eq!(sum, 10);
    }

    #[test]
    fn test_list_find() {
        let list = List::from(vec![1, 2, 3, 4, 5]);
        let found = list.find(|x| *x > 3);

        assert_eq!(found, Some(&4));
    }

    #[test]
    fn test_list_some_every() {
        let list = List::from(vec![2, 4, 6, 8]);
        assert!(list.every(|x| *x % 2 == 0));
        assert!(list.some(|x| *x > 5));

        let mixed_list = List::from(vec![1, 2, 3, 4]);
        assert!(!mixed_list.every(|x| *x % 2 == 0));
        assert!(mixed_list.some(|x| *x % 2 == 0));
    }

    #[test]
    fn test_list_contains() {
        let list = List::from(vec![1, 2, 3]);
        assert!(list.contains(&2));
        assert!(!list.contains(&5));
    }

    #[test]
    fn test_list_take_drop() {
        let list = List::from(vec![1, 2, 3, 4, 5]);
        let first_two = list.take(2);
        let last_three = list.drop(2);

        assert_eq!(first_two.len(), 2);
        assert_eq!(first_two.get(0), &1);
        assert_eq!(first_two.get(1), &2);

        assert_eq!(last_three.len(), 3);
        assert_eq!(last_three.get(0), &3);
        assert_eq!(last_three.get(1), &4);
        assert_eq!(last_three.get(2), &5);
    }

    #[test]
    fn test_list_extend() {
        let mut list = List::from(vec![1, 2]);
        list.extend(vec![3, 4, 5]);

        assert_eq!(list.len(), 5);
        assert_eq!(list.get(2), &3);
        assert_eq!(list.get(3), &4);
        assert_eq!(list.get(4), &5);
    }

    #[test]
    fn test_list_display() {
        let list = List::from(vec![1, 2, 3]);
        assert_eq!(format!("{}", list), "[1, 2, 3]");

        let empty_list: List<i32> = List::new();
        assert_eq!(format!("{}", empty_list), "[]");
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
