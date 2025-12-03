/*!
# Primitives Module

Basic types for Vela programs.

## Types

- `VelaNumber` - Numeric type (Int or Float)
- `VelaString` - String type with rich API
- `VelaBool` - Boolean type
*/

mod number;
mod string;
mod bool;

pub use number::VelaNumber;
pub use string::VelaString;
pub use bool::VelaBool;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives_exist() {
        // Ensure all primitives are exported
        let _n: Option<VelaNumber> = None;
        let _s: Option<VelaString> = None;
        let _b: Option<VelaBool> = None;
    }
}
