/*!
# Collections

Vela's collections module with List, Map, and Set.

## Features

- **VelaList<T>**: Dynamic array with functional API (map, filter, reduce)
- **VelaMap<K,V>**: Hash map with key-value storage
- **VelaSet<T>**: Unique element storage

## Design Principles

- Immutability by default (methods return new instances)
- Functional API inspired by Rust/TypeScript/Python
- Zero-cost abstractions over Rust's standard collections
*/

mod list;
mod map;
mod set;

pub use list::VelaList;
pub use map::VelaMap;
pub use set::VelaSet;
