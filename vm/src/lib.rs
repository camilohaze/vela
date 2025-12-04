/*!
# Vela VM - Virtual Machine for Vela Language

Stack-based virtual machine with bytecode interpreter and garbage collection.

## Features

- **Stack-based execution**: Simple and debuggable architecture
- **Hybrid GC**: Reference counting with cycle detection
- **Value tagging**: Efficient NaN-boxing for numeric types
- **Call frames**: Full function call support with closures
- **Error handling**: Exception support with try/catch
- **Debugging**: Breakpoints and execution tracing

## Architecture

Based on ADR-801, the VM uses:
- Stack machine (like CPython, JVM)
- 256 opcodes with variable-length encoding
- Reference counting + mark-and-sweep for cycles
- Tagged pointers for efficient value representation

## Performance Targets

- 3-8x faster than CPython
- < 10ms startup time
- < 5% GC overhead

## Example

```rust,no_run
// TODO: VM implementation in progress
// Example will be available after Sprint 9 completion
use vela_vm::{Bytecode, Instruction, Value};

let mut bytecode = Bytecode::new();
bytecode.push(Instruction::Add);
bytecode.push(Instruction::Return);

let value = Value::int(42);
assert!(value.is_int());
```
*/

pub mod bytecode;
pub mod vm;
pub mod gc;
pub mod error;

/// Re-export main types
pub use bytecode::{Bytecode, Instruction, Value, CodeObject, Constant};
pub use vm::{VirtualMachine, CallFrame};
// TODO: Uncomment when gc.rs is implemented
// pub use gc::{GcHeap, GcObject};
pub use error::{Error, Result};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}