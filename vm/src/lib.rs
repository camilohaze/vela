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
pub mod loader;
pub mod module_resolver;
pub mod jit;

/// Re-export main types
pub use bytecode::{Bytecode, Instruction, Value, CodeObject, Constant};
pub use vm::{VirtualMachine, CallFrame};
pub use gc::{GcHeap, GcObject, GcPtr, GcStats, FunctionObject, ClosureObject};
pub use error::{Error, Result};
pub use loader::{BytecodeLoader, LoadedModule};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_module_loading() {
        let mut vm = VirtualMachine::new();

        // Test loading a module
        let mut module_globals = std::collections::HashMap::new();
        module_globals.insert("test_var".to_string(), Value::int(42));

        vm.load_module("test_module".to_string(), module_globals);

        // Verify module is loaded
        assert!(vm.is_module_loaded("test_module"));
        assert!(!vm.is_module_loaded("nonexistent"));

        // Get loaded modules
        let modules = vm.get_loaded_modules();
        assert!(modules.contains(&"test_module".to_string()));

        // Get module globals
        let globals = vm.get_module("test_module").unwrap();
        assert_eq!(globals.get("test_var"), Some(&Value::int(42)));
    }
}