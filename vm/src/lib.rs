/*!
# VelaVM - Virtual Machine

The Vela Virtual Machine executes Vela bytecode with high performance
and memory safety. Features include:

- Stack-based execution
- Garbage collection
- Hot reloading support
- Performance profiling
- Memory safety guarantees
*/

pub mod bytecode;
pub mod vm;
pub mod gc;
pub mod profiler;

/// Re-export main types
pub use vm::VM;
pub use bytecode::Bytecode;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}