/*!
# VelaResult<T, E>

Type-safe error handling.

## Design

VelaResult represents the result of an operation that can succeed with a value T
or fail with an error E. This forces explicit error handling without exceptions.

## Examples

```rust
use vela_stdlib::VelaResult;

fn divide(a: i32, b: i32) -> VelaResult<i32, String> {
    if b == 0 {
        VelaResult::err("Division by zero".to_string())
    } else {
        VelaResult::ok(a / b)
    }
}

let result = divide(10, 2);
assert!(result.is_ok());
assert_eq!(result.unwrap(), 5);

let error = divide(10, 0);
assert!(error.is_err());
```
*/

use std::fmt;

/// Result type for operations that can succeed or fail
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VelaResult<T, E> {
    /// Success with value T
    Ok(T),
    /// Failure with error E
    Err(E),
}

impl<T, E> VelaResult<T, E> {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create an Ok variant
    pub fn ok(value: T) -> Self {
        VelaResult::Ok(value)
    }

    /// Create an Err variant
    pub fn err(error: E) -> Self {
        VelaResult::Err(error)
    }

    // ============================================================================
    // Query
    // ============================================================================

    /// Returns true if the result is Ok
    pub fn is_ok(&self) -> bool {
        matches!(self, VelaResult::Ok(_))
    }

    /// Returns true if the result is Err
    pub fn is_err(&self) -> bool {
        matches!(self, VelaResult::Err(_))
    }

    // ============================================================================
    // Extracting values
    // ============================================================================

    /// Unwrap the Ok value, panicking if Err
    pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            VelaResult::Ok(val) => val,
            VelaResult::Err(err) => panic!("Called unwrap on Err: {:?}", err),
        }
    }

    /// Unwrap the Err value, panicking if Ok
    pub fn unwrap_err(self) -> E
    where
        T: fmt::Debug,
    {
        match self {
            VelaResult::Ok(val) => panic!("Called unwrap_err on Ok: {:?}", val),
            VelaResult::Err(err) => err,
        }
    }

    /// Unwrap the Ok value or return a default
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            VelaResult::Ok(val) => val,
            VelaResult::Err(_) => default,
        }
    }

    /// Unwrap the Ok value or compute it from the error
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        match self {
            VelaResult::Ok(val) => val,
            VelaResult::Err(err) => f(err),
        }
    }

    // ============================================================================
    // Transforming Ok values
    // ============================================================================

    /// Maps a Result<T, E> to Result<U, E> by applying a function to Ok value
    pub fn map<U, F>(self, f: F) -> VelaResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            VelaResult::Ok(val) => VelaResult::Ok(f(val)),
            VelaResult::Err(err) => VelaResult::Err(err),
        }
    }

    /// Maps a Result<T, E> to Result<U, E> by applying a function that returns Result<U, E>
    pub fn and_then<U, F>(self, f: F) -> VelaResult<U, E>
    where
        F: FnOnce(T) -> VelaResult<U, E>,
    {
        match self {
            VelaResult::Ok(val) => f(val),
            VelaResult::Err(err) => VelaResult::Err(err),
        }
    }

    // ============================================================================
    // Transforming Err values
    // ============================================================================

    /// Maps a Result<T, E> to Result<T, F> by applying a function to Err value
    pub fn map_err<F, O>(self, f: O) -> VelaResult<T, F>
    where
        O: FnOnce(E) -> F,
    {
        match self {
            VelaResult::Ok(val) => VelaResult::Ok(val),
            VelaResult::Err(err) => VelaResult::Err(f(err)),
        }
    }

    /// Calls f if the result is Err, otherwise returns self
    pub fn or_else<F, O>(self, f: O) -> VelaResult<T, F>
    where
        O: FnOnce(E) -> VelaResult<T, F>,
    {
        match self {
            VelaResult::Ok(val) => VelaResult::Ok(val),
            VelaResult::Err(err) => f(err),
        }
    }

    // ============================================================================
    // Combining results
    // ============================================================================

    /// Returns self if it's Ok, otherwise returns other
    pub fn or(self, other: VelaResult<T, E>) -> VelaResult<T, E> {
        match self {
            VelaResult::Ok(_) => self,
            VelaResult::Err(_) => other,
        }
    }

    /// Returns self if it's Err, otherwise returns other
    pub fn and<U>(self, other: VelaResult<U, E>) -> VelaResult<U, E> {
        match self {
            VelaResult::Ok(_) => other,
            VelaResult::Err(err) => VelaResult::Err(err),
        }
    }

    // ============================================================================
    // References
    // ============================================================================

    /// Converts from &Result<T, E> to Result<&T, &E>
    pub fn as_ref(&self) -> VelaResult<&T, &E> {
        match self {
            VelaResult::Ok(ref val) => VelaResult::Ok(val),
            VelaResult::Err(ref err) => VelaResult::Err(err),
        }
    }

    /// Converts from &mut Result<T, E> to Result<&mut T, &mut E>
    pub fn as_mut(&mut self) -> VelaResult<&mut T, &mut E> {
        match self {
            VelaResult::Ok(ref mut val) => VelaResult::Ok(val),
            VelaResult::Err(ref mut err) => VelaResult::Err(err),
        }
    }
}

impl<T: fmt::Display, E: fmt::Display> fmt::Display for VelaResult<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VelaResult::Ok(val) => write!(f, "Ok({})", val),
            VelaResult::Err(err) => write!(f, "Err({})", err),
        }
    }
}

// ============================================================================
// Conversion from Rust's Result
// ============================================================================

impl<T, E> From<Result<T, E>> for VelaResult<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(val) => VelaResult::Ok(val),
            Err(err) => VelaResult::Err(err),
        }
    }
}

impl<T, E> From<VelaResult<T, E>> for Result<T, E> {
    fn from(result: VelaResult<T, E>) -> Self {
        match result {
            VelaResult::Ok(val) => Ok(val),
            VelaResult::Err(err) => Err(err),
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
        let ok: VelaResult<i32, String> = VelaResult::ok(42);
        assert_eq!(ok, VelaResult::Ok(42));

        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        assert_eq!(err, VelaResult::Err("error".to_string()));
    }

    #[test]
    fn test_is_ok_is_err() {
        let ok: VelaResult<i32, String> = VelaResult::ok(42);
        assert!(ok.is_ok());
        assert!(!ok.is_err());

        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        assert!(!err.is_ok());
        assert!(err.is_err());
    }

    #[test]
    fn test_unwrap() {
        let ok: VelaResult<i32, String> = VelaResult::ok(42);
        assert_eq!(ok.unwrap(), 42);
    }

    #[test]
    #[should_panic]
    fn test_unwrap_panics() {
        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        err.unwrap();
    }

    #[test]
    fn test_unwrap_err() {
        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        assert_eq!(err.unwrap_err(), "error");
    }

    #[test]
    fn test_unwrap_or() {
        let ok: VelaResult<i32, String> = VelaResult::ok(42);
        assert_eq!(ok.unwrap_or(0), 42);

        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        assert_eq!(err.unwrap_or(0), 0);
    }

    #[test]
    fn test_map() {
        let ok: VelaResult<i32, String> = VelaResult::ok(2);
        let mapped = ok.map(|x| x * 2);
        assert_eq!(mapped, VelaResult::Ok(4));

        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        let mapped = err.map(|x| x * 2);
        assert_eq!(mapped, VelaResult::Err("error".to_string()));
    }

    #[test]
    fn test_map_err() {
        let ok: VelaResult<i32, String> = VelaResult::ok(42);
        let mapped = ok.map_err(|e| format!("Error: {}", e));
        assert_eq!(mapped, VelaResult::Ok(42));

        let err: VelaResult<i32, String> = VelaResult::err("fail".to_string());
        let mapped = err.map_err(|e| format!("Error: {}", e));
        assert_eq!(mapped, VelaResult::Err("Error: fail".to_string()));
    }

    #[test]
    fn test_and_then() {
        let ok: VelaResult<i32, String> = VelaResult::ok(2);
        let result = ok.and_then(|x| VelaResult::ok(x * 2));
        assert_eq!(result, VelaResult::Ok(4));

        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        let result = err.and_then(|x| VelaResult::ok(x * 2));
        assert_eq!(result, VelaResult::Err("error".to_string()));
    }

    #[test]
    fn test_or() {
        let ok: VelaResult<i32, String> = VelaResult::ok(42);
        let other: VelaResult<i32, String> = VelaResult::ok(100);
        assert_eq!(ok.or(other), VelaResult::Ok(42));

        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        let other: VelaResult<i32, String> = VelaResult::ok(100);
        assert_eq!(err.or(other), VelaResult::Ok(100));
    }

    #[test]
    fn test_and() {
        let ok1: VelaResult<i32, String> = VelaResult::ok(42);
        let ok2: VelaResult<i32, String> = VelaResult::ok(100);
        assert_eq!(ok1.and(ok2), VelaResult::Ok(100));

        let ok: VelaResult<i32, String> = VelaResult::ok(42);
        let err: VelaResult<i32, String> = VelaResult::err("error".to_string());
        assert_eq!(ok.and(err), VelaResult::Err("error".to_string()));
    }

    #[test]
    fn test_conversion_from_rust_result() {
        let rust_ok: Result<i32, String> = Ok(42);
        let vela_ok: VelaResult<i32, String> = rust_ok.into();
        assert_eq!(vela_ok, VelaResult::Ok(42));

        let rust_err: Result<i32, String> = Err("error".to_string());
        let vela_err: VelaResult<i32, String> = rust_err.into();
        assert_eq!(vela_err, VelaResult::Err("error".to_string()));
    }
}
