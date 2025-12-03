/*!
# VelaBool

Boolean type with logical operations.

## Design

VelaBool is a simple wrapper over Rust's bool with explicit logical operations.

## Examples

```rust
use vela_stdlib::VelaBool;

let t = VelaBool::new(true);
let f = VelaBool::new(false);

assert!(t.as_bool());
assert!(t.and(&f).as_bool() == false);
assert!(t.or(&f).as_bool() == true);
assert!(t.not().as_bool() == false);
```
*/

use std::fmt;

/// Boolean type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VelaBool(bool);

impl VelaBool {
    // ============================================================================
    // Constructors
    // ============================================================================

    /// Create a new VelaBool
    pub fn new(value: bool) -> Self {
        VelaBool(value)
    }

    /// True value
    pub fn true_val() -> Self {
        VelaBool(true)
    }

    /// False value
    pub fn false_val() -> Self {
        VelaBool(false)
    }

    // ============================================================================
    // Conversions
    // ============================================================================

    /// Get as Rust bool
    pub fn as_bool(&self) -> bool {
        self.0
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    // ============================================================================
    // Logical operations
    // ============================================================================

    /// Logical AND
    pub fn and(&self, other: &VelaBool) -> VelaBool {
        VelaBool(self.0 && other.0)
    }

    /// Logical OR
    pub fn or(&self, other: &VelaBool) -> VelaBool {
        VelaBool(self.0 || other.0)
    }

    /// Logical NOT
    pub fn not(&self) -> VelaBool {
        VelaBool(!self.0)
    }

    /// Logical XOR
    pub fn xor(&self, other: &VelaBool) -> VelaBool {
        VelaBool(self.0 ^ other.0)
    }

    /// Logical NAND
    pub fn nand(&self, other: &VelaBool) -> VelaBool {
        VelaBool(!(self.0 && other.0))
    }

    /// Logical NOR
    pub fn nor(&self, other: &VelaBool) -> VelaBool {
        VelaBool(!(self.0 || other.0))
    }

    /// Logical XNOR (equivalence)
    pub fn xnor(&self, other: &VelaBool) -> VelaBool {
        VelaBool(!(self.0 ^ other.0))
    }

    /// Implication (if self then other)
    pub fn implies(&self, other: &VelaBool) -> VelaBool {
        VelaBool(!self.0 || other.0)
    }
}

impl fmt::Display for VelaBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for VelaBool {
    fn from(b: bool) -> Self {
        VelaBool(b)
    }
}

impl From<VelaBool> for bool {
    fn from(b: VelaBool) -> Self {
        b.0
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
        let t = VelaBool::new(true);
        assert_eq!(t.as_bool(), true);

        let f = VelaBool::new(false);
        assert_eq!(f.as_bool(), false);

        assert_eq!(VelaBool::true_val().as_bool(), true);
        assert_eq!(VelaBool::false_val().as_bool(), false);
    }

    #[test]
    fn test_and() {
        let t = VelaBool::new(true);
        let f = VelaBool::new(false);

        assert_eq!(t.and(&t).as_bool(), true);
        assert_eq!(t.and(&f).as_bool(), false);
        assert_eq!(f.and(&t).as_bool(), false);
        assert_eq!(f.and(&f).as_bool(), false);
    }

    #[test]
    fn test_or() {
        let t = VelaBool::new(true);
        let f = VelaBool::new(false);

        assert_eq!(t.or(&t).as_bool(), true);
        assert_eq!(t.or(&f).as_bool(), true);
        assert_eq!(f.or(&t).as_bool(), true);
        assert_eq!(f.or(&f).as_bool(), false);
    }

    #[test]
    fn test_not() {
        let t = VelaBool::new(true);
        let f = VelaBool::new(false);

        assert_eq!(t.not().as_bool(), false);
        assert_eq!(f.not().as_bool(), true);
    }

    #[test]
    fn test_xor() {
        let t = VelaBool::new(true);
        let f = VelaBool::new(false);

        assert_eq!(t.xor(&t).as_bool(), false);
        assert_eq!(t.xor(&f).as_bool(), true);
        assert_eq!(f.xor(&t).as_bool(), true);
        assert_eq!(f.xor(&f).as_bool(), false);
    }

    #[test]
    fn test_nand() {
        let t = VelaBool::new(true);
        let f = VelaBool::new(false);

        assert_eq!(t.nand(&t).as_bool(), false);
        assert_eq!(t.nand(&f).as_bool(), true);
        assert_eq!(f.nand(&t).as_bool(), true);
        assert_eq!(f.nand(&f).as_bool(), true);
    }

    #[test]
    fn test_nor() {
        let t = VelaBool::new(true);
        let f = VelaBool::new(false);

        assert_eq!(t.nor(&t).as_bool(), false);
        assert_eq!(t.nor(&f).as_bool(), false);
        assert_eq!(f.nor(&t).as_bool(), false);
        assert_eq!(f.nor(&f).as_bool(), true);
    }

    #[test]
    fn test_implies() {
        let t = VelaBool::new(true);
        let f = VelaBool::new(false);

        assert_eq!(t.implies(&t).as_bool(), true);
        assert_eq!(t.implies(&f).as_bool(), false);
        assert_eq!(f.implies(&t).as_bool(), true);
        assert_eq!(f.implies(&f).as_bool(), true);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(VelaBool::new(true).to_string(), "true");
        assert_eq!(VelaBool::new(false).to_string(), "false");
    }

    #[test]
    fn test_conversion_from_bool() {
        let b: VelaBool = true.into();
        assert_eq!(b.as_bool(), true);

        let b: VelaBool = false.into();
        assert_eq!(b.as_bool(), false);
    }
}
