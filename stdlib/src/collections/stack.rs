/*!
# Stack<T> - LIFO Collection

Vela's LIFO (Last In, First Out) stack collection.
Inspired by traditional stack data structures with a simple, efficient API.

## Features

- **LIFO semantics**: Last element added is first to be removed
- **Efficient operations**: O(1) push and pop
- **Simple API**: Focused on stack-specific use cases
- **Vec-based**: Uses Vec<T> internally for simplicity

## Example

```rust
use vela_stdlib::collections::Stack;

let mut stack = Stack::new();
stack.push(1);
stack.push(2);
stack.push(3);

assert_eq!(stack.pop(), Some(3)); // LIFO: last in, first out
assert_eq!(stack.peek(), Some(2)); // Look at top without removing
```
*/

use std::fmt;

/* ============================================================================
Stack<T> - LIFO Collection
============================================================================ */

/// LIFO (Last In, First Out) stack collection
/// Uses Vec<T> internally for simplicity and efficiency
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create empty stack
    pub fn new() -> Self {
        Stack {
            items: Vec::new(),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Stack {
            items: Vec::with_capacity(capacity),
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

    /// Add element to top of stack (push)
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Remove and return element from top of stack (pop)
    /// Returns None if stack is empty
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Return reference to top element without removing it
    /// Returns None if stack is empty
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    /// Return mutable reference to top element without removing it
    /// Returns None if stack is empty
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.items.last_mut()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Check if stack contains element
    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.items.contains(item)
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
    // Conversion Operations
    // ============================================================================

    /// Convert into vector
    pub fn into_vec(self) -> Vec<T> {
        self.items
    }

    /// Create from vector
    pub fn from_vec(vec: Vec<T>) -> Self {
        Stack { items: vec }
    }
}

impl<T: Clone> Stack<T> {
    /// Create from slice
    pub fn from_slice(slice: &[T]) -> Self {
        Stack {
            items: slice.to_vec(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stack[")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Stack::new()
    }
}

impl<T> From<Vec<T>> for Stack<T> {
    fn from(items: Vec<T>) -> Self {
        Stack { items }
    }
}

impl<T> From<Stack<T>> for Vec<T> {
    fn from(stack: Stack<T>) -> Self {
        stack.items
    }
}

// ============================================================================
// Tests for Stack<T>
// ============================================================================

#[cfg(test)]
mod stack_tests {
    use super::*;

    #[test]
    fn test_stack_new() {
        let stack: Stack<i32> = Stack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_stack_with_capacity() {
        let stack: Stack<i32> = Stack::with_capacity(10);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_push_pop() {
        let mut stack = Stack::new();

        // Push elements
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.len(), 3);
        assert!(!stack.is_empty());

        // Pop elements (LIFO order)
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None); // Empty now

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_stack_peek() {
        let mut stack = Stack::new();
        assert_eq!(stack.peek(), None);

        stack.push(42);
        assert_eq!(stack.peek(), Some(&42));
        assert_eq!(stack.len(), 1); // Peek doesn't remove

        stack.push(99);
        assert_eq!(stack.peek(), Some(&99)); // Top element changed
    }

    #[test]
    fn test_stack_peek_mut() {
        let mut stack = Stack::new();
        stack.push(42);

        if let Some(value) = stack.peek_mut() {
            *value = 100;
        }

        assert_eq!(stack.peek(), Some(&100));
        assert_eq!(stack.pop(), Some(100));
    }

    #[test]
    fn test_stack_contains() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert!(stack.contains(&2));
        assert!(!stack.contains(&4));
    }

    #[test]
    fn test_stack_clear() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        assert_eq!(stack.len(), 2);

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_stack_into_vec() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        let vec = stack.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_stack_from_vec() {
        let vec = vec![1, 2, 3];
        let mut stack = Stack::from_vec(vec);

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
    }

    #[test]
    fn test_stack_from_slice() {
        let slice = [1, 2, 3];
        let mut stack = Stack::from_slice(&slice);

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
    }

    #[test]
    fn test_stack_display() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(format!("{}", stack), "Stack[1, 2, 3]");
    }

    #[test]
    fn test_stack_empty_display() {
        let stack: Stack<i32> = Stack::new();
        assert_eq!(format!("{}", stack), "Stack[]");
    }

    #[test]
    fn test_stack_single_element_display() {
        let mut stack = Stack::new();
        stack.push(42);
        assert_eq!(format!("{}", stack), "Stack[42]");
    }
}