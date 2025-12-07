/*!
# Collections

Vela's collections module with List, Set, Dict, and Map.

## Features

- **List<T>**: Mutable dynamic array (primary collection type)
- **VelaList<T>**: Immutable dynamic array with functional API (map, filter, reduce)
- **Set<T>**: Mutable hash-based set (primary set type)
- **VelaSet<T>**: Immutable hash-based set with functional API
- **Dict<K,V>**: Mutable hash-based dictionary (primary key-value type)
- **VelaMap<K,V>**: Hash map with key-value storage

## Design Principles

- **List<T>**: Mutable by default, inspired by Rust's Vec<T>
- **Set<T>**: Mutable by default, inspired by Rust's HashSet<T>
- **Dict<K,V>**: Mutable by default, inspired by Rust's HashMap<K,V>
- **VelaList<T>**: Immutability by default (methods return new instances)
- **VelaSet<T>**: Immutability by default (methods return new instances)
- Functional API inspired by Rust/TypeScript/Python
- Zero-cost abstractions over Rust's standard collections
*/

mod dict;
mod list;
mod map;
mod set;

pub use dict::Dict;
pub use list::{List, VelaList};
pub use map::VelaMap;
pub use set::{Set, VelaSet};
