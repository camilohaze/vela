/*!
# Strings

Advanced string utilities for Vela.

## Features

- **Interpolation**: Template strings with `${}` syntax
- **Formatting**: Type-safe formatting utilities
- **Regex**: Pattern matching and replacement
- **Splitting**: Advanced string splitting

## Examples

```rust
use vela_stdlib::strings::{interpolate, format_string};

let name = "Vela";
let version = 1;
let result = interpolate("Hello, ${name} v${version}!");
```
*/

mod interpolation;
mod format;
mod regex;
mod split;

pub use interpolation::{interpolate, InterpolationError};
pub use format::{format_string, FormatError};
pub use regex::{Regex, RegexError};
pub use split::split_advanced;
