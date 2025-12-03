/*!
# VelaIterator

Iterator with functional adapters.

## Design

VelaIterator wraps Rust's Iterator trait and provides:
- Lazy evaluation
- Method chaining
- Functional transformations

## Examples

```rust
use vela_stdlib::VelaIterator;

let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 5]);
let doubled: Vec<i32> = iter.map(|x| x * 2).collect();
```
*/

/// Iterator with functional adapters
pub struct VelaIterator<T> {
    iter: Box<dyn Iterator<Item = T>>,
}

impl<T: 'static> VelaIterator<T> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create from vector
    pub fn from_vec(vec: Vec<T>) -> Self {
        VelaIterator {
            iter: Box::new(vec.into_iter()),
        }
    }

    /// Create from iterator
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: Iterator<Item = T> + 'static,
    {
        VelaIterator {
            iter: Box::new(iter),
        }
    }

    /// Create range
    pub fn range(start: T, end: T) -> Self
    where
        T: std::ops::Add<Output = T> + PartialOrd + Clone + From<u8> + 'static,
    {
        let mut current = start.clone();
        let step = T::from(1);
        let iter = std::iter::from_fn(move || {
            if current < end.clone() {
                let value = current.clone();
                current = current.clone() + step.clone();
                Some(value)
            } else {
                None
            }
        });
        VelaIterator {
            iter: Box::new(iter),
        }
    }

    // ============================================================================
    // Transformations
    // ============================================================================

    /// Map elements
    pub fn map<U, F>(self, f: F) -> VelaIterator<U>
    where
        U: 'static,
        F: FnMut(T) -> U + 'static,
    {
        VelaIterator {
            iter: Box::new(self.iter.map(f)),
        }
    }

    /// Filter elements
    pub fn filter<F>(self, f: F) -> VelaIterator<T>
    where
        F: FnMut(&T) -> bool + 'static,
    {
        VelaIterator {
            iter: Box::new(self.iter.filter(f)),
        }
    }

    /// Filter and map (flat_map)
    pub fn flat_map<U, F>(self, f: F) -> VelaIterator<U>
    where
        U: 'static,
        F: FnMut(T) -> Vec<U> + 'static,
    {
        VelaIterator {
            iter: Box::new(self.iter.flat_map(f)),
        }
    }

    /// Take first N elements
    pub fn take(self, n: usize) -> VelaIterator<T> {
        VelaIterator {
            iter: Box::new(self.iter.take(n)),
        }
    }

    /// Skip first N elements
    pub fn skip(self, n: usize) -> VelaIterator<T> {
        VelaIterator {
            iter: Box::new(self.iter.skip(n)),
        }
    }

    /// Take while predicate
    pub fn take_while<F>(self, f: F) -> VelaIterator<T>
    where
        F: FnMut(&T) -> bool + 'static,
    {
        VelaIterator {
            iter: Box::new(self.iter.take_while(f)),
        }
    }

    /// Skip while predicate
    pub fn skip_while<F>(self, f: F) -> VelaIterator<T>
    where
        F: FnMut(&T) -> bool + 'static,
    {
        VelaIterator {
            iter: Box::new(self.iter.skip_while(f)),
        }
    }

    /// Enumerate (add index)
    pub fn enumerate(self) -> VelaIterator<(usize, T)> {
        VelaIterator {
            iter: Box::new(self.iter.enumerate()),
        }
    }

    /// Chain with another iterator
    pub fn chain(self, other: VelaIterator<T>) -> VelaIterator<T> {
        VelaIterator {
            iter: Box::new(self.iter.chain(other.iter)),
        }
    }

    /// Zip with another iterator
    pub fn zip<U>(self, other: VelaIterator<U>) -> VelaIterator<(T, U)>
    where
        U: 'static,
    {
        VelaIterator {
            iter: Box::new(self.iter.zip(other.iter)),
        }
    }

    // ============================================================================
    // Reductions
    // ============================================================================

    /// Reduce to single value
    pub fn reduce<F>(mut self, f: F) -> Option<T>
    where
        F: FnMut(T, T) -> T,
    {
        self.iter.reduce(f)
    }

    /// Fold with initial value
    pub fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, T) -> B,
    {
        self.iter.fold(init, f)
    }

    /// Sum elements
    pub fn sum(self) -> T
    where
        T: std::iter::Sum,
    {
        self.iter.sum()
    }

    /// Product of elements
    pub fn product(self) -> T
    where
        T: std::iter::Product,
    {
        self.iter.product()
    }

    /// Count elements
    pub fn count(self) -> usize {
        self.iter.count()
    }

    // ============================================================================
    // Queries
    // ============================================================================

    /// Check if any element matches
    pub fn any<F>(mut self, mut f: F) -> bool
    where
        F: FnMut(T) -> bool,
    {
        self.iter.any(|x| f(x))
    }

    /// Check if all elements match
    pub fn all<F>(mut self, mut f: F) -> bool
    where
        F: FnMut(T) -> bool,
    {
        self.iter.all(|x| f(x))
    }

    /// Find first matching element
    pub fn find<F>(mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&T) -> bool,
    {
        self.iter.find(|x| f(x))
    }

    /// Find position of first match
    pub fn position<F>(mut self, mut f: F) -> Option<usize>
    where
        F: FnMut(T) -> bool,
    {
        self.iter.position(|x| f(x))
    }

    /// Get min element
    pub fn min(mut self) -> Option<T>
    where
        T: Ord,
    {
        self.iter.min()
    }

    /// Get max element
    pub fn max(mut self) -> Option<T>
    where
        T: Ord,
    {
        self.iter.max()
    }

    // ============================================================================
    // Collections
    // ============================================================================

    /// Collect to vector
    pub fn collect(self) -> Vec<T> {
        self.iter.collect()
    }

    /// Partition into two vectors
    pub fn partition<F>(self, mut f: F) -> (Vec<T>, Vec<T>)
    where
        F: FnMut(&T) -> bool,
    {
        self.iter.partition(|x| f(x))
    }

    // ============================================================================
    // Side Effects
    // ============================================================================

    /// For each element
    pub fn for_each<F>(self, mut f: F)
    where
        F: FnMut(T),
    {
        self.iter.for_each(|x| f(x));
    }

    /// Inspect elements (side effects without consuming)
    pub fn inspect<F>(self, f: F) -> VelaIterator<T>
    where
        F: FnMut(&T) + 'static,
    {
        VelaIterator {
            iter: Box::new(self.iter.inspect(f)),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_vec() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3]);
        let result: Vec<i32> = iter.collect();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_map() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3]);
        let doubled: Vec<i32> = iter.map(|x| x * 2).collect();
        assert_eq!(doubled, vec![2, 4, 6]);
    }

    #[test]
    fn test_filter() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 5]);
        let evens: Vec<i32> = iter.filter(|x| x % 2 == 0).collect();
        assert_eq!(evens, vec![2, 4]);
    }

    #[test]
    fn test_map_filter_chain() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 5]);
        let result: Vec<i32> = iter.map(|x| x * 2).filter(|x| *x > 5).collect();
        assert_eq!(result, vec![6, 8, 10]);
    }

    #[test]
    fn test_take() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 5]);
        let first_two: Vec<i32> = iter.take(2).collect();
        assert_eq!(first_two, vec![1, 2]);
    }

    #[test]
    fn test_skip() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 5]);
        let skip_two: Vec<i32> = iter.skip(2).collect();
        assert_eq!(skip_two, vec![3, 4, 5]);
    }

    #[test]
    fn test_reduce() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4]);
        let sum = iter.reduce(|acc, x| acc + x);
        assert_eq!(sum, Some(10));
    }

    #[test]
    fn test_fold() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4]);
        let sum = iter.fold(0, |acc, x| acc + x);
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_sum() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4]);
        let sum: i32 = iter.sum();
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_product() {
        let iter = VelaIterator::from_vec(vec![2, 3, 4]);
        let prod: i32 = iter.product();
        assert_eq!(prod, 24);
    }

    #[test]
    fn test_count() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 5]);
        let count = iter.filter(|x| x % 2 == 0).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_any() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3]);
        assert!(iter.any(|x| x > 2));

        let iter2 = VelaIterator::from_vec(vec![1, 2, 3]);
        assert!(!iter2.any(|x| x > 5));
    }

    #[test]
    fn test_all() {
        let iter = VelaIterator::from_vec(vec![2, 4, 6]);
        assert!(iter.all(|x| x % 2 == 0));

        let iter2 = VelaIterator::from_vec(vec![1, 2, 3]);
        assert!(!iter2.all(|x| x % 2 == 0));
    }

    #[test]
    fn test_find() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4]);
        let found = iter.find(|x| x > &2);
        assert_eq!(found, Some(3));
    }

    #[test]
    fn test_min_max() {
        let iter = VelaIterator::from_vec(vec![3, 1, 4, 1, 5]);
        assert_eq!(iter.min(), Some(1));

        let iter2 = VelaIterator::from_vec(vec![3, 1, 4, 1, 5]);
        assert_eq!(iter2.max(), Some(5));
    }

    #[test]
    fn test_enumerate() {
        let iter = VelaIterator::from_vec(vec![10, 20, 30]);
        let indexed: Vec<(usize, i32)> = iter.enumerate().collect();
        assert_eq!(indexed, vec![(0, 10), (1, 20), (2, 30)]);
    }

    #[test]
    fn test_chain() {
        let iter1 = VelaIterator::from_vec(vec![1, 2]);
        let iter2 = VelaIterator::from_vec(vec![3, 4]);
        let combined: Vec<i32> = iter1.chain(iter2).collect();
        assert_eq!(combined, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_zip() {
        let iter1 = VelaIterator::from_vec(vec![1, 2, 3]);
        let iter2 = VelaIterator::from_vec(vec!["a", "b", "c"]);
        let zipped: Vec<(i32, &str)> = iter1.zip(iter2).collect();
        assert_eq!(zipped, vec![(1, "a"), (2, "b"), (3, "c")]);
    }

    #[test]
    fn test_partition() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 5]);
        let (evens, odds) = iter.partition(|x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4]);
        assert_eq!(odds, vec![1, 3, 5]);
    }

    #[test]
    fn test_flat_map() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3]);
        let expanded: Vec<i32> = iter.flat_map(|x| vec![x, x * 10]).collect();
        assert_eq!(expanded, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn test_take_while() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 1]);
        let result: Vec<i32> = iter.take_while(|x| x < &4).collect();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_skip_while() {
        let iter = VelaIterator::from_vec(vec![1, 2, 3, 4, 5]);
        let result: Vec<i32> = iter.skip_while(|x| x < &3).collect();
        assert_eq!(result, vec![3, 4, 5]);
    }
}
