/*!
Error types for Vela VM
*/

use miette::Diagnostic;
use thiserror::Error;

/// Result type for VM operations
pub type Result<T> = std::result::Result<T, Error>;

/// VM error types
#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("Stack underflow")]
    #[diagnostic(
        code(vm::stack_underflow),
        help("The operand stack is empty. This usually indicates a compiler bug or corrupted bytecode.")
    )]
    StackUnderflow,

    #[error("Stack overflow")]
    #[diagnostic(
        code(vm::stack_overflow),
        help("The operand stack exceeded its maximum size. Consider reducing recursion depth.")
    )]
    StackOverflow,

    #[error("Invalid opcode: {opcode:#04x}")]
    #[diagnostic(
        code(vm::invalid_opcode),
        help("Encountered an unknown bytecode instruction. This indicates corrupted bytecode.")
    )]
    InvalidOpcode { opcode: u8 },

    #[error("Invalid constant index: {index}")]
    #[diagnostic(
        code(vm::invalid_constant),
        help("Constant pool index is out of bounds.")
    )]
    InvalidConstant { index: usize },

    #[error("Invalid local variable index: {index}")]
    #[diagnostic(
        code(vm::invalid_local),
        help("Local variable index is out of bounds.")
    )]
    InvalidLocal { index: usize },

    #[error("Type error: expected {expected}, got {actual}")]
    #[diagnostic(
        code(vm::type_error),
        help("Operation requires a specific type. Check your bytecode generation.")
    )]
    TypeError {
        expected: String,
        actual: String,
    },

    #[error("Division by zero")]
    #[diagnostic(
        code(vm::division_by_zero),
        help("Attempted to divide by zero.")
    )]
    DivisionByZero,

    #[error("Call stack overflow")]
    #[diagnostic(
        code(vm::call_overflow),
        help("Maximum call depth exceeded. Check for infinite recursion.")
    )]
    CallStackOverflow,

    #[error("Undefined variable: {name}")]
    #[diagnostic(
        code(vm::undefined_variable),
        help("Variable was not defined in the current scope.")
    )]
    UndefinedVariable { name: String },

    #[error("Invalid jump target: {target}")]
    #[diagnostic(
        code(vm::invalid_jump),
        help("Jump instruction points to an invalid bytecode address.")
    )]
    InvalidJump { target: usize },

    #[error("GC error: {message}")]
    #[diagnostic(
        code(vm::gc_error),
        help("Garbage collector encountered an error.")
    )]
    GcError { message: String },

    #[error("IO error: {0}")]
    #[diagnostic(code(vm::io_error))]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    #[diagnostic(code(vm::serialization_error))]
    Serialization(#[from] bincode::Error),

    #[error("Runtime exception: {message}")]
    #[diagnostic(
        code(vm::runtime_exception),
        help("An exception was raised during execution.")
    )]
    RuntimeException { message: String },
}

impl Error {
    /// Create a type error
    pub fn type_error(expected: &str, actual: &str) -> Self {
        Self::TypeError {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Create an undefined variable error
    pub fn undefined_variable(name: &str) -> Self {
        Self::UndefinedVariable {
            name: name.to_string(),
        }
    }

    /// Create a GC error
    pub fn gc_error(message: &str) -> Self {
        Self::GcError {
            message: message.to_string(),
        }
    }

    /// Create a runtime exception
    pub fn runtime_exception(message: &str) -> Self {
        Self::RuntimeException {
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::StackUnderflow;
        assert_eq!(err.to_string(), "Stack underflow");
    }

    #[test]
    fn test_type_error() {
        let err = Error::type_error("Int", "String");
        assert!(err.to_string().contains("expected Int"));
        assert!(err.to_string().contains("got String"));
    }

    #[test]
    fn test_undefined_variable() {
        let err = Error::undefined_variable("x");
        assert!(err.to_string().contains("x"));
    }

    #[test]
    fn test_invalid_opcode() {
        let err = Error::InvalidOpcode { opcode: 0xFF };
        assert!(err.to_string().contains("0xff"));
    }

    #[test]
    fn test_division_by_zero() {
        let err = Error::DivisionByZero;
        assert_eq!(err.to_string(), "Division by zero");
    }
}
