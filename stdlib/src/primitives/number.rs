/*!
# VelaNumber

Numeric type that can represent both integers and floating-point numbers.

## Design

VelaNumber is a union type similar to JavaScript's Number or Python's numeric types.
It can hold either an i64 or f64, with automatic conversions during operations.

## Examples

```rust
use vela_stdlib::VelaNumber;

// Create integers
let int_num = VelaNumber::int(42);

// Create floats
let float_num = VelaNumber::float(3.14);

// Arithmetic operations
let sum = int_num.add(&float_num); // Automatic type promotion
assert_eq!(sum.as_float(), 45.14);

// Comparisons
assert!(float_num.gt(&int_num) == false);
```
*/

use std::fmt;
use std::ops::{Add, Sub, Mul, Neg};

use crate::option_result::{VelaOption, VelaResult};

/// Numeric type that can represent both integers and floating-point numbers
#[derive(Debug, Clone, PartialEq)]
pub enum VelaNumber {
    /// Integer value (i64)
    Int(i64),
    /// Floating-point value (f64)
    Float(f64),
}

impl VelaNumber {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create a new integer number
    pub fn int(value: i64) -> Self {
        VelaNumber::Int(value)
    }

    /// Create a new floating-point number
    pub fn float(value: f64) -> Self {
        VelaNumber::Float(value)
    }

    /// Create a number from a string
    pub fn from_str(s: &str) -> VelaResult<Self, String> {
        // Try parsing as integer first
        if let Ok(int_val) = s.parse::<i64>() {
            return VelaResult::ok(VelaNumber::Int(int_val));
        }

        // Try parsing as float
        if let Ok(float_val) = s.parse::<f64>() {
            return VelaResult::ok(VelaNumber::Float(float_val));
        }

        VelaResult::err(format!("Cannot parse '{}' as number", s))
    }

    // ============================================================================
    // Type inspection
    // ============================================================================

    /// Check if this number is an integer
    pub fn is_int(&self) -> bool {
        matches!(self, VelaNumber::Int(_))
    }

    /// Check if this number is a float
    pub fn is_float(&self) -> bool {
        matches!(self, VelaNumber::Float(_))
    }

    // ============================================================================
    // Conversions
    // ============================================================================

    /// Get as integer (returns None if this is a non-integer float)
    pub fn as_int(&self) -> VelaOption<i64> {
        match self {
            VelaNumber::Int(i) => VelaOption::some(*i),
            VelaNumber::Float(f) => {
                if f.fract() == 0.0 && f.is_finite() {
                    VelaOption::some(*f as i64)
                } else {
                    VelaOption::none()
                }
            }
        }
    }

    /// Get as float (always succeeds)
    pub fn as_float(&self) -> f64 {
        match self {
            VelaNumber::Int(i) => *i as f64,
            VelaNumber::Float(f) => *f,
        }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        match self {
            VelaNumber::Int(i) => i.to_string(),
            VelaNumber::Float(f) => {
                // Remove trailing .0 for whole numbers
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            }
        }
    }

    // ============================================================================
    // Arithmetic operations
    // ============================================================================

    /// Add two numbers
    pub fn add(&self, other: &VelaNumber) -> VelaNumber {
        match (self, other) {
            (VelaNumber::Int(a), VelaNumber::Int(b)) => VelaNumber::Int(a + b),
            _ => VelaNumber::Float(self.as_float() + other.as_float()),
        }
    }

    /// Subtract two numbers
    pub fn sub(&self, other: &VelaNumber) -> VelaNumber {
        match (self, other) {
            (VelaNumber::Int(a), VelaNumber::Int(b)) => VelaNumber::Int(a - b),
            _ => VelaNumber::Float(self.as_float() - other.as_float()),
        }
    }

    /// Multiply two numbers
    pub fn mul(&self, other: &VelaNumber) -> VelaNumber {
        match (self, other) {
            (VelaNumber::Int(a), VelaNumber::Int(b)) => VelaNumber::Int(a * b),
            _ => VelaNumber::Float(self.as_float() * other.as_float()),
        }
    }

    /// Divide two numbers (returns error on division by zero)
    pub fn div(&self, other: &VelaNumber) -> VelaResult<VelaNumber, String> {
        if other.as_float() == 0.0 {
            return VelaResult::err("Division by zero".to_string());
        }

        let result = self.as_float() / other.as_float();
        VelaResult::ok(VelaNumber::Float(result))
    }

    /// Integer division (floor division)
    pub fn div_floor(&self, other: &VelaNumber) -> VelaResult<VelaNumber, String> {
        if other.as_float() == 0.0 {
            return VelaResult::err("Division by zero".to_string());
        }

        let result = (self.as_float() / other.as_float()).floor();
        VelaResult::ok(VelaNumber::Float(result))
    }

    /// Modulo operation
    pub fn modulo(&self, other: &VelaNumber) -> VelaResult<VelaNumber, String> {
        if other.as_float() == 0.0 {
            return VelaResult::err("Modulo by zero".to_string());
        }

        match (self, other) {
            (VelaNumber::Int(a), VelaNumber::Int(b)) => {
                VelaResult::ok(VelaNumber::Int(a % b))
            }
            _ => {
                let result = self.as_float() % other.as_float();
                VelaResult::ok(VelaNumber::Float(result))
            }
        }
    }

    /// Power operation
    pub fn pow(&self, exponent: &VelaNumber) -> VelaNumber {
        let result = self.as_float().powf(exponent.as_float());
        VelaNumber::Float(result)
    }

    /// Square root
    pub fn sqrt(&self) -> VelaResult<VelaNumber, String> {
        let value = self.as_float();
        if value < 0.0 {
            return VelaResult::err("Cannot take square root of negative number".to_string());
        }

        VelaResult::ok(VelaNumber::Float(value.sqrt()))
    }

    /// Absolute value
    pub fn abs(&self) -> VelaNumber {
        match self {
            VelaNumber::Int(i) => VelaNumber::Int(i.abs()),
            VelaNumber::Float(f) => VelaNumber::Float(f.abs()),
        }
    }

    /// Negation
    pub fn neg(&self) -> VelaNumber {
        match self {
            VelaNumber::Int(i) => VelaNumber::Int(-i),
            VelaNumber::Float(f) => VelaNumber::Float(-f),
        }
    }

    // ============================================================================
    // Comparison operations
    // ============================================================================

    /// Equal to
    pub fn eq(&self, other: &VelaNumber) -> bool {
        match (self, other) {
            (VelaNumber::Int(a), VelaNumber::Int(b)) => a == b,
            _ => (self.as_float() - other.as_float()).abs() < f64::EPSILON,
        }
    }

    /// Not equal to
    pub fn ne(&self, other: &VelaNumber) -> bool {
        !self.eq(other)
    }

    /// Greater than
    pub fn gt(&self, other: &VelaNumber) -> bool {
        self.as_float() > other.as_float()
    }

    /// Greater than or equal to
    pub fn ge(&self, other: &VelaNumber) -> bool {
        self.as_float() >= other.as_float()
    }

    /// Less than
    pub fn lt(&self, other: &VelaNumber) -> bool {
        self.as_float() < other.as_float()
    }

    /// Less than or equal to
    pub fn le(&self, other: &VelaNumber) -> bool {
        self.as_float() <= other.as_float()
    }

    // ============================================================================
    // Math utilities
    // ============================================================================

    /// Round to nearest integer
    pub fn round(&self) -> VelaNumber {
        VelaNumber::Float(self.as_float().round())
    }

    /// Floor (round down)
    pub fn floor(&self) -> VelaNumber {
        VelaNumber::Float(self.as_float().floor())
    }

    /// Ceil (round up)
    pub fn ceil(&self) -> VelaNumber {
        VelaNumber::Float(self.as_float().ceil())
    }

    /// Check if number is NaN
    pub fn is_nan(&self) -> bool {
        match self {
            VelaNumber::Float(f) => f.is_nan(),
            VelaNumber::Int(_) => false,
        }
    }

    /// Check if number is infinite
    pub fn is_infinite(&self) -> bool {
        match self {
            VelaNumber::Float(f) => f.is_infinite(),
            VelaNumber::Int(_) => false,
        }
    }

    /// Check if number is finite
    pub fn is_finite(&self) -> bool {
        match self {
            VelaNumber::Float(f) => f.is_finite(),
            VelaNumber::Int(_) => true,
        }
    }

    /// Minimum of two numbers
    pub fn min(&self, other: &VelaNumber) -> VelaNumber {
        if self.lt(other) {
            self.clone()
        } else {
            other.clone()
        }
    }

    /// Maximum of two numbers
    pub fn max(&self, other: &VelaNumber) -> VelaNumber {
        if self.gt(other) {
            self.clone()
        } else {
            other.clone()
        }
    }

    /// Clamp number between min and max
    pub fn clamp(&self, min: &VelaNumber, max: &VelaNumber) -> VelaNumber {
        self.max(min).min(max)
    }
}

// ============================================================================
// Trait implementations for ergonomic usage
// ============================================================================

impl fmt::Display for VelaNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Add for VelaNumber {
    type Output = VelaNumber;

    fn add(self, other: VelaNumber) -> VelaNumber {
        VelaNumber::add(&self, &other)
    }
}

impl Sub for VelaNumber {
    type Output = VelaNumber;

    fn sub(self, other: VelaNumber) -> VelaNumber {
        VelaNumber::sub(&self, &other)
    }
}

impl Mul for VelaNumber {
    type Output = VelaNumber;

    fn mul(self, other: VelaNumber) -> VelaNumber {
        VelaNumber::mul(&self, &other)
    }
}

impl Neg for VelaNumber {
    type Output = VelaNumber;

    fn neg(self) -> VelaNumber {
        VelaNumber::neg(&self)
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
        let int_num = VelaNumber::int(42);
        assert_eq!(int_num, VelaNumber::Int(42));

        let float_num = VelaNumber::float(3.14);
        assert_eq!(float_num, VelaNumber::Float(3.14));
    }

    #[test]
    fn test_from_str() {
        let int_num = VelaNumber::from_str("42").unwrap();
        assert_eq!(int_num, VelaNumber::Int(42));

        let float_num = VelaNumber::from_str("3.14").unwrap();
        assert_eq!(float_num, VelaNumber::Float(3.14));

        let err = VelaNumber::from_str("not_a_number");
        assert!(err.is_err());
    }

    #[test]
    fn test_type_checks() {
        let int_num = VelaNumber::int(42);
        assert!(int_num.is_int());
        assert!(!int_num.is_float());

        let float_num = VelaNumber::float(3.14);
        assert!(!float_num.is_int());
        assert!(float_num.is_float());
    }

    #[test]
    fn test_conversions() {
        let int_num = VelaNumber::int(42);
        assert_eq!(int_num.as_int().unwrap(), 42);
        assert_eq!(int_num.as_float(), 42.0);

        let float_num = VelaNumber::float(3.14);
        assert!(float_num.as_int().is_none());
        assert_eq!(float_num.as_float(), 3.14);

        let whole_float = VelaNumber::float(42.0);
        assert_eq!(whole_float.as_int().unwrap(), 42);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(VelaNumber::int(42).to_string(), "42");
        assert_eq!(VelaNumber::float(3.14).to_string(), "3.14");
        assert_eq!(VelaNumber::float(42.0).to_string(), "42");
    }

    #[test]
    fn test_addition() {
        let a = VelaNumber::int(10);
        let b = VelaNumber::int(20);
        assert_eq!(VelaNumber::add(&a, &b), VelaNumber::Int(30));

        let c = VelaNumber::float(3.5);
        let d = VelaNumber::int(2);
        assert_eq!(VelaNumber::add(&c, &d), VelaNumber::Float(5.5));
    }

    #[test]
    fn test_subtraction() {
        let a = VelaNumber::int(20);
        let b = VelaNumber::int(10);
        assert_eq!(VelaNumber::sub(&a, &b), VelaNumber::Int(10));
    }

    #[test]
    fn test_multiplication() {
        let a = VelaNumber::int(5);
        let b = VelaNumber::int(6);
        assert_eq!(VelaNumber::mul(&a, &b), VelaNumber::Int(30));
    }

    #[test]
    fn test_division() {
        let a = VelaNumber::int(10);
        let b = VelaNumber::int(2);
        assert_eq!(a.div(&b).unwrap(), VelaNumber::Float(5.0));

        let c = VelaNumber::int(10);
        let d = VelaNumber::int(0);
        assert!(c.div(&d).is_err());
    }

    #[test]
    fn test_modulo() {
        let a = VelaNumber::int(10);
        let b = VelaNumber::int(3);
        assert_eq!(a.modulo(&b).unwrap(), VelaNumber::Int(1));
    }

    #[test]
    fn test_pow() {
        let a = VelaNumber::int(2);
        let b = VelaNumber::int(3);
        assert_eq!(a.pow(&b), VelaNumber::Float(8.0));
    }

    #[test]
    fn test_sqrt() {
        let a = VelaNumber::int(16);
        assert_eq!(a.sqrt().unwrap(), VelaNumber::Float(4.0));

        let b = VelaNumber::int(-1);
        assert!(b.sqrt().is_err());
    }

    #[test]
    fn test_abs() {
        assert_eq!(VelaNumber::int(-5).abs(), VelaNumber::Int(5));
        assert_eq!(VelaNumber::float(-3.14).abs(), VelaNumber::Float(3.14));
    }

    #[test]
    fn test_neg() {
        assert_eq!(VelaNumber::int(5).neg(), VelaNumber::Int(-5));
        assert_eq!(VelaNumber::float(3.14).neg(), VelaNumber::Float(-3.14));
    }

    #[test]
    fn test_comparisons() {
        let a = VelaNumber::int(10);
        let b = VelaNumber::int(20);

        assert!(a.lt(&b));
        assert!(a.le(&b));
        assert!(b.gt(&a));
        assert!(b.ge(&a));
        assert!(a.eq(&a));
        assert!(a.ne(&b));
    }

    #[test]
    fn test_rounding() {
        let a = VelaNumber::float(3.7);
        assert_eq!(a.round(), VelaNumber::Float(4.0));
        assert_eq!(a.floor(), VelaNumber::Float(3.0));
        assert_eq!(a.ceil(), VelaNumber::Float(4.0));
    }

    #[test]
    fn test_min_max() {
        let a = VelaNumber::int(10);
        let b = VelaNumber::int(20);

        assert_eq!(a.min(&b), VelaNumber::Int(10));
        assert_eq!(a.max(&b), VelaNumber::Int(20));
    }

    #[test]
    fn test_clamp() {
        let a = VelaNumber::int(15);
        let min = VelaNumber::int(10);
        let max = VelaNumber::int(20);

        assert_eq!(a.clamp(&min, &max), VelaNumber::Int(15));

        let b = VelaNumber::int(5);
        assert_eq!(b.clamp(&min, &max), VelaNumber::Int(10));

        let c = VelaNumber::int(25);
        assert_eq!(c.clamp(&min, &max), VelaNumber::Int(20));
    }

    #[test]
    fn test_operator_overloads() {
        let a = VelaNumber::int(10);
        let b = VelaNumber::int(20);

        assert_eq!(a.clone() + b.clone(), VelaNumber::Int(30));
        assert_eq!(b.clone() - a.clone(), VelaNumber::Int(10));
        assert_eq!(a.clone() * VelaNumber::int(2), VelaNumber::Int(20));
        assert_eq!(-a.clone(), VelaNumber::Int(-10));
    }
}
