/*!
# Collections

Vela's collections module with List, Map, and Set.

## Features

- **List<T>**: Mutable dynamic array (primary collection type)
- **VelaList<T>**: Immutable dynamic array with functional API (map, filter, reduce)
- **Set<T>**: Mutable hash-based set (primary set type)
- **VelaSet<T>**: Immutable hash-based set with functional API
- **VelaMap<K,V>**: Hash map with key-value storage

## Design Principles

- **List<T>**: Mutable by default, inspired by Rust's Vec<T>
- **Set<T>**: Mutable by default, inspired by Rust's HashSet<T>
- **VelaList<T>**: Immutability by default (methods return new instances)
- **VelaSet<T>**: Immutability by default (methods return new instances)
- Functional API inspired by Rust/TypeScript/Python
- Zero-cost abstractions over Rust's standard collections
*/

mod list;
mod map;
mod set;

pub use list::{List, VelaList};
pub use map::VelaMap;
pub use set::{Set, VelaSet};
