/*!
# Vela Standard Library

The Vela standard library provides essential functionality for Vela programs,
including primitives, collections, option/result types, iterators, and utilities.

## Modules

- `primitives` - Basic types (Number, String, Bool)
- `collections` - Data structures (List, Map, Set)
- `option_result` - Optional and result types
- `iterators` - Iterator protocol and adapters
- `strings` - String utilities and interpolation

## Architecture

This stdlib is designed with:
- **Type-safety**: Generic types and trait bounds
- **Zero-cost abstractions**: Thin wrappers over Rust types
- **Immutability by default**: Functional programming paradigm
- **Rich APIs**: Inspired by Rust, TypeScript, Python, Swift
*/

// Core modules
pub mod primitives;
pub mod collections;
pub mod option_result;
pub mod iterators;
pub mod strings;
pub mod io;
pub mod http;
pub mod websocket;

// Re-export commonly used types
pub use primitives::{VelaNumber, VelaString, VelaBool};
pub use collections::{VelaList, VelaMap, VelaSet};
pub use option_result::{VelaOption, VelaResult};
pub use iterators::VelaIterator;

/// Initialize the standard library
pub fn init() {
    // Standard library initialization
    // Currently a no-op, but can be used for future initialization needs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_init() {
        init(); // Should not panic
    }
}