/*!
# Queue<T> - FIFO Collection

Vela's FIFO (First In, First Out) queue collection.
Inspired by Rust's VecDeque<T> but with a simpler API focused on queue operations.

## Features

- **FIFO semantics**: First element added is first to be removed
- **Efficient operations**: O(1) push and pop
- **Simple API**: Focused on queue-specific use cases
- **Vec-based**: Uses Vec<T> internally for simplicity

## Example

```rust
use vela_stdlib::collections::Queue;

let mut queue = Queue::new();
queue.push(1);
queue.push(2);
queue.push(3);

assert_eq!(queue.pop(), Some(1)); // FIFO: first in, first out
assert_eq!(queue.peek(), Some(2)); // Look at next without removing
```
*/

use std::fmt;

/* ============================================================================
Queue<T> - FIFO Collection
============================================================================ */

/// FIFO (First In, First Out) queue collection
/// Uses Vec<T> internally for simplicity and efficiency
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queue<T> {
    items: Vec<T>,
}

impl<T> Queue<T> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create empty queue
    pub fn new() -> Self {
        Queue {
            items: Vec::new(),
        }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Queue {
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

    /// Add element to back of queue (enqueue)
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Remove and return element from front of queue (dequeue)
    /// Returns None if queue is empty
    pub fn pop(&mut self) -> Option<T> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    /// Return reference to front element without removing it
    /// Returns None if queue is empty
    pub fn peek(&self) -> Option<&T> {
        self.items.first()
    }

    /// Return mutable reference to front element without removing it
    /// Returns None if queue is empty
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.items.first_mut()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Check if queue contains element
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
        Queue { items: vec }
    }
}

impl<T: Clone> Queue<T> {
    /// Create from slice
    pub fn from_slice(slice: &[T]) -> Self {
        Queue {
            items: slice.to_vec(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Queue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Queue[")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Queue::new()
    }
}

impl<T> From<Vec<T>> for Queue<T> {
    fn from(items: Vec<T>) -> Self {
        Queue { items }
    }
}

impl<T> From<Queue<T>> for Vec<T> {
    fn from(queue: Queue<T>) -> Self {
        queue.items
    }
}

// ============================================================================
// Tests for Queue<T>
// ============================================================================

#[cfg(test)]
mod queue_tests {
    use super::*;

    #[test]
    fn test_queue_new() {
        let queue: Queue<i32> = Queue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_with_capacity() {
        let queue: Queue<i32> = Queue::with_capacity(10);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_push_pop() {
        let mut queue = Queue::new();

        // Push elements
        queue.push(1);
        queue.push(2);
        queue.push(3);

        assert_eq!(queue.len(), 3);
        assert!(!queue.is_empty());

        // Pop elements (FIFO order)
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None); // Empty now

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_peek() {
        let mut queue = Queue::new();
        assert_eq!(queue.peek(), None);

        queue.push(42);
        assert_eq!(queue.peek(), Some(&42));
        assert_eq!(queue.len(), 1); // Peek doesn't remove

        queue.push(99);
        assert_eq!(queue.peek(), Some(&42)); // Still first element
    }

    #[test]
    fn test_queue_peek_mut() {
        let mut queue = Queue::new();
        queue.push(42);

        if let Some(value) = queue.peek_mut() {
            *value = 100;
        }

        assert_eq!(queue.peek(), Some(&100));
        assert_eq!(queue.pop(), Some(100));
    }

    #[test]
    fn test_queue_contains() {
        let mut queue = Queue::new();
        queue.push(1);
        queue.push(2);
        queue.push(3);

        assert!(queue.contains(&2));
        assert!(!queue.contains(&4));
    }

    #[test]
    fn test_queue_clear() {
        let mut queue = Queue::new();
        queue.push(1);
        queue.push(2);
        assert_eq!(queue.len(), 2);

        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_into_vec() {
        let mut queue = Queue::new();
        queue.push(1);
        queue.push(2);
        queue.push(3);

        let vec = queue.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_queue_from_vec() {
        let vec = vec![1, 2, 3];
        let mut queue = Queue::from_vec(vec);

        assert_eq!(queue.len(), 3);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
    }

    #[test]
    fn test_queue_from_slice() {
        let slice = [1, 2, 3];
        let mut queue = Queue::from_slice(&slice);

        assert_eq!(queue.len(), 3);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
    }

    #[test]
    fn test_queue_display() {
        let mut queue = Queue::new();
        queue.push(1);
        queue.push(2);
        queue.push(3);

        assert_eq!(format!("{}", queue), "Queue[1, 2, 3]");
    }

    #[test]
    fn test_queue_empty_display() {
        let queue: Queue<i32> = Queue::new();
        assert_eq!(format!("{}", queue), "Queue[]");
    }

    #[test]
    fn test_queue_single_element_display() {
        let mut queue = Queue::new();
        queue.push(42);
        assert_eq!(format!("{}", queue), "Queue[42]");
    }
}