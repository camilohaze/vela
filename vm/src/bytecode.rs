/*
Bytecode format for VelaVM

This module defines the bytecode instructions and format used by the Vela Virtual Machine.
The bytecode is stack-based and designed for functional programming with reactivity.
*/

use serde::{Deserialize, Serialize};
use std::fmt;

/// Bytecode instruction set
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    // Stack operations
    Push(i64),           // Push integer literal
    PushFloat(f64),      // Push float literal
    PushString(String),  // Push string literal
    PushBool(bool),      // Push boolean literal
    Pop,                 // Pop top of stack

    // Arithmetic operations
    Add,                 // Add top two values
    Sub,                 // Subtract top two values
    Mul,                 // Multiply top two values
    Div,                 // Divide top two values

    // Comparison operations
    Eq,                  // Equal
    Ne,                  // Not equal
    Lt,                  // Less than
    Le,                  // Less or equal
    Gt,                  // Greater than
    Ge,                  // Greater or equal

    // Control flow
    Jump(usize),         // Unconditional jump to address
    JumpIf(usize),       // Jump if top of stack is true
    Call(usize),         // Call function at address
    Return,              // Return from function

    // Variable operations
    Load(usize),         // Load variable from environment
    Store(usize),        // Store to variable in environment

    // Reactive operations
    SignalNew,           // Create new signal
    SignalGet,           // Get signal value
    SignalSet,           // Set signal value
    ComputedNew(usize),  // Create computed with body at address
    EffectNew(usize),    // Create effect with body at address

    // Function operations
    FnNew(usize),        // Create function with body at address
    ClosureNew(usize),   // Create closure

    // Data structure operations
    ListNew,             // Create new list
    ListPush,            // Push to list
    DictNew,             // Create new dict
    DictSet,             // Set dict key-value

    // Pattern matching
    Match,               // Start pattern match
    Case(usize),         // Case branch
    Wildcard,            // Wildcard pattern

    // Error handling
    Try(usize),          // Try block
    Catch(usize),        // Catch block
    Throw,               // Throw exception

    // Built-in functions
    Print,               // Print top of stack
    Len,                 // Length of collection
    TypeOf,              // Get type of value

    // Module operations
    Import(String),      // Import module
    Export(String),      // Export symbol

    // Debug
    Nop,                 // No operation
}

/// Compiled bytecode with metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,  // Constant pool
    pub functions: Vec<Function>, // Function definitions
    pub symbols: Vec<String>,   // Symbol table
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
            symbols: Vec::new(),
        }
    }

    pub fn push_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn add_symbol(&mut self, symbol: String) -> usize {
        self.symbols.push(symbol);
        self.symbols.len() - 1
    }

    /// Serializar bytecode a bytes
    pub fn into_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Deserializar bytecode desde bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(bytes)?)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Push(val) => write!(f, "PUSH {}", val),
            Instruction::PushFloat(val) => write!(f, "PUSH_FLOAT {}", val),
            Instruction::PushString(val) => write!(f, "PUSH_STRING \"{}\"", val),
            Instruction::PushBool(val) => write!(f, "PUSH_BOOL {}", val),
            Instruction::Pop => write!(f, "POP"),
            Instruction::Add => write!(f, "ADD"),
            Instruction::Sub => write!(f, "SUB"),
            Instruction::Mul => write!(f, "MUL"),
            Instruction::Div => write!(f, "DIV"),
            Instruction::Eq => write!(f, "EQ"),
            Instruction::Ne => write!(f, "NE"),
            Instruction::Lt => write!(f, "LT"),
            Instruction::Le => write!(f, "LE"),
            Instruction::Gt => write!(f, "GT"),
            Instruction::Ge => write!(f, "GE"),
            Instruction::Jump(addr) => write!(f, "JUMP {}", addr),
            Instruction::JumpIf(addr) => write!(f, "JUMP_IF {}", addr),
            Instruction::Call(addr) => write!(f, "CALL {}", addr),
            Instruction::Return => write!(f, "RETURN"),
            Instruction::Load(idx) => write!(f, "LOAD {}", idx),
            Instruction::Store(idx) => write!(f, "STORE {}", idx),
            Instruction::SignalNew => write!(f, "SIGNAL_NEW"),
            Instruction::SignalGet => write!(f, "SIGNAL_GET"),
            Instruction::SignalSet => write!(f, "SIGNAL_SET"),
            Instruction::ComputedNew(addr) => write!(f, "COMPUTED_NEW {}", addr),
            Instruction::EffectNew(addr) => write!(f, "EFFECT_NEW {}", addr),
            Instruction::FnNew(addr) => write!(f, "FN_NEW {}", addr),
            Instruction::ClosureNew(addr) => write!(f, "CLOSURE_NEW {}", addr),
            Instruction::ListNew => write!(f, "LIST_NEW"),
            Instruction::ListPush => write!(f, "LIST_PUSH"),
            Instruction::DictNew => write!(f, "DICT_NEW"),
            Instruction::DictSet => write!(f, "DICT_SET"),
            Instruction::Match => write!(f, "MATCH"),
            Instruction::Case(addr) => write!(f, "CASE {}", addr),
            Instruction::Wildcard => write!(f, "WILDCARD"),
            Instruction::Try(addr) => write!(f, "TRY {}", addr),
            Instruction::Catch(addr) => write!(f, "CATCH {}", addr),
            Instruction::Throw => write!(f, "THROW"),
            Instruction::Print => write!(f, "PRINT"),
            Instruction::Len => write!(f, "LEN"),
            Instruction::TypeOf => write!(f, "TYPEOF"),
            Instruction::Import(module) => write!(f, "IMPORT {}", module),
            Instruction::Export(symbol) => write!(f, "EXPORT {}", symbol),
            Instruction::Nop => write!(f, "NOP"),
        }
    }
}

/// Runtime values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    Dict(std::collections::HashMap<String, Value>),
    Function(Function),
    Signal(Box<Value>),
    None,
}

/// Function definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body_start: usize,  // Address where function body starts
    pub body_end: usize,    // Address where function body ends
}

impl Function {
    pub fn new(name: String, params: Vec<String>, body_start: usize, body_end: usize) -> Self {
        Self {
            name,
            params,
            body_start,
            body_end,
        }
    }
}