/*!
# VelaOption<T>

Type-safe optional values (replacement for null/undefined/nil).

## Design

VelaOption represents a value that may or may not exist. This eliminates null pointer
exceptions and forces explicit handling of missing values.

## Examples

```rust
use vela_stdlib::VelaOption;

let some_value = VelaOption::some(42);
assert!(some_value.is_some());

let none_value: VelaOption<i32> = VelaOption::none();
assert!(none_value.is_none());

// Safe unwrapping
let value = some_value.unwrap_or(0);
assert_eq!(value, 42);
```
*/

use std::fmt;

/// Optional value that may be Some(T) or None
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VelaOption<T> {
    /// Some value of type T
    Some(T),
    /// No value
    None,
}

impl<T> VelaOption<T> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create a Some variant
    pub fn some(value: T) -> Self {
        VelaOption::Some(value)
    }

    /// Create a None variant
    pub fn none() -> Self {
        VelaOption::None
    }

    // ============================================================================
    // Query
    // ============================================================================

    /// Returns true if the option is a Some value
    pub fn is_some(&self) -> bool {
        matches!(self, VelaOption::Some(_))
    }

    /// Returns true if the option is None
    pub fn is_none(&self) -> bool {
        matches!(self, VelaOption::None)
    }

    // ============================================================================
    // Extracting values
    // ============================================================================

    /// Unwrap the value, panicking if None
    pub fn unwrap(self) -> T {
        match self {
            VelaOption::Some(val) => val,
            VelaOption::None => panic!("Called unwrap on None"),
        }
    }

    /// Unwrap the value or return a default
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            VelaOption::Some(val) => val,
            VelaOption::None => default,
        }
    }

    /// Unwrap the value or compute it from a closure
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            VelaOption::Some(val) => val,
            VelaOption::None => f(),
        }
    }

    // ============================================================================
    // Transforming
    // ============================================================================

    /// Maps an Option<T> to Option<U> by applying a function
    pub fn map<U, F>(self, f: F) -> VelaOption<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            VelaOption::Some(val) => VelaOption::Some(f(val)),
            VelaOption::None => VelaOption::None,
        }
    }

    /// Maps an Option<T> to Option<U> by applying a function that returns Option<U>
    pub fn and_then<U, F>(self, f: F) -> VelaOption<U>
    where
        F: FnOnce(T) -> VelaOption<U>,
    {
        match self {
            VelaOption::Some(val) => f(val),
            VelaOption::None => VelaOption::None,
        }
    }

    /// Returns the option if it contains a value, otherwise calls f
    pub fn or_else<F>(self, f: F) -> VelaOption<T>
    where
        F: FnOnce() -> VelaOption<T>,
    {
        match self {
            VelaOption::Some(_) => self,
            VelaOption::None => f(),
        }
    }

    /// Returns Some(t) if the option is Some and the predicate returns true
    pub fn filter<F>(self, predicate: F) -> VelaOption<T>
    where
        F: FnOnce(&T) -> bool,
    {
        match self {
            VelaOption::Some(ref val) if predicate(val) => self,
            _ => VelaOption::None,
        }
    }

    // ============================================================================
    // Combining
    // ============================================================================

    /// Returns Some if both self and other are Some, otherwise None
    pub fn zip<U>(self, other: VelaOption<U>) -> VelaOption<(T, U)> {
        match (self, other) {
            (VelaOption::Some(a), VelaOption::Some(b)) => VelaOption::Some((a, b)),
            _ => VelaOption::None,
        }
    }

    /// Returns self if it's Some, otherwise returns other
    pub fn or(self, other: VelaOption<T>) -> VelaOption<T> {
        match self {
            VelaOption::Some(_) => self,
            VelaOption::None => other,
        }
    }

    // ============================================================================
    // References
    // ============================================================================

    /// Converts from &Option<T> to Option<&T>
    pub fn as_ref(&self) -> VelaOption<&T> {
        match self {
            VelaOption::Some(ref val) => VelaOption::Some(val),
            VelaOption::None => VelaOption::None,
        }
    }

    /// Converts from &mut Option<T> to Option<&mut T>
    pub fn as_mut(&mut self) -> VelaOption<&mut T> {
        match self {
            VelaOption::Some(ref mut val) => VelaOption::Some(val),
            VelaOption::None => VelaOption::None,
        }
    }
}

impl<T: fmt::Display> fmt::Display for VelaOption<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VelaOption::Some(val) => write!(f, "Some({})", val),
            VelaOption::None => write!(f, "None"),
        }
    }
}

// ============================================================================
// Conversion from Rust's Option
// ============================================================================

impl<T> From<Option<T>> for VelaOption<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(val) => VelaOption::Some(val),
            None => VelaOption::None,
        }
    }
}

impl<T> From<VelaOption<T>> for Option<T> {
    fn from(opt: VelaOption<T>) -> Self {
        match opt {
            VelaOption::Some(val) => Some(val),
            VelaOption::None => None,
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
    fn test_constructors() {
        let some: VelaOption<i32> = VelaOption::some(42);
        assert_eq!(some, VelaOption::Some(42));

        let none: VelaOption<i32> = VelaOption::none();
        assert_eq!(none, VelaOption::None);
    }

    #[test]
    fn test_is_some_is_none() {
        let some = VelaOption::some(42);
        assert!(some.is_some());
        assert!(!some.is_none());

        let none: VelaOption<i32> = VelaOption::none();
        assert!(!none.is_some());
        assert!(none.is_none());
    }

    #[test]
    fn test_unwrap() {
        let some = VelaOption::some(42);
        assert_eq!(some.unwrap(), 42);
    }

    #[test]
    #[should_panic(expected = "Called unwrap on None")]
    fn test_unwrap_panics() {
        let none: VelaOption<i32> = VelaOption::none();
        none.unwrap();
    }

    #[test]
    fn test_unwrap_or() {
        let some = VelaOption::some(42);
        assert_eq!(some.unwrap_or(0), 42);

        let none: VelaOption<i32> = VelaOption::none();
        assert_eq!(none.unwrap_or(0), 0);
    }

    #[test]
    fn test_unwrap_or_else() {
        let some = VelaOption::some(42);
        assert_eq!(some.unwrap_or_else(|| 0), 42);

        let none: VelaOption<i32> = VelaOption::none();
        assert_eq!(none.unwrap_or_else(|| 100), 100);
    }

    #[test]
    fn test_map() {
        let some = VelaOption::some(2);
        let mapped = some.map(|x| x * 2);
        assert_eq!(mapped, VelaOption::Some(4));

        let none: VelaOption<i32> = VelaOption::none();
        let mapped = none.map(|x| x * 2);
        assert_eq!(mapped, VelaOption::None);
    }

    #[test]
    fn test_and_then() {
        let some = VelaOption::some(2);
        let result = some.and_then(|x| VelaOption::some(x * 2));
        assert_eq!(result, VelaOption::Some(4));

        let none: VelaOption<i32> = VelaOption::none();
        let result = none.and_then(|x| VelaOption::some(x * 2));
        assert_eq!(result, VelaOption::None);
    }

    #[test]
    fn test_or_else() {
        let some = VelaOption::some(42);
        let result = some.or_else(|| VelaOption::some(100));
        assert_eq!(result, VelaOption::Some(42));

        let none: VelaOption<i32> = VelaOption::none();
        let result = none.or_else(|| VelaOption::some(100));
        assert_eq!(result, VelaOption::Some(100));
    }

    #[test]
    fn test_filter() {
        let some = VelaOption::some(42);
        let filtered = some.filter(|x| *x > 40);
        assert_eq!(filtered, VelaOption::Some(42));

        let some = VelaOption::some(10);
        let filtered = some.filter(|x| *x > 40);
        assert_eq!(filtered, VelaOption::None);
    }

    #[test]
    fn test_zip() {
        let a = VelaOption::some(1);
        let b = VelaOption::some(2);
        assert_eq!(a.zip(b), VelaOption::Some((1, 2)));

        let a = VelaOption::some(1);
        let b: VelaOption<i32> = VelaOption::none();
        assert_eq!(a.zip(b), VelaOption::None);
    }

    #[test]
    fn test_or() {
        let some = VelaOption::some(42);
        let other = VelaOption::some(100);
        assert_eq!(some.or(other), VelaOption::Some(42));

        let none: VelaOption<i32> = VelaOption::none();
        let other = VelaOption::some(100);
        assert_eq!(none.or(other), VelaOption::Some(100));
    }

    #[test]
    fn test_conversion_from_rust_option() {
        let rust_some = Some(42);
        let vela_some: VelaOption<i32> = rust_some.into();
        assert_eq!(vela_some, VelaOption::Some(42));

        let rust_none: Option<i32> = None;
        let vela_none: VelaOption<i32> = rust_none.into();
        assert_eq!(vela_none, VelaOption::None);
    }
}
