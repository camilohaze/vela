/*!
# Iterators

Iterator protocol for Vela with functional adapters.

## Design

VelaIterator provides a functional API inspired by Rust/TypeScript/Python:
- Lazy evaluation (adapters don't execute until consumed)
- Composable transformations (map, filter, reduce, etc.)
- Zero-cost abstractions

## Examples

```rust
use vela_stdlib::VelaIterator;

let nums = vec![1, 2, 3, 4, 5];
let sum: i32 = VelaIterator::from_vec(nums)
    .map(|x| x * 2)
    .filter(|x| x > 5)
    .sum();
```
*/

mod iter;

pub use iter::VelaIterator;
