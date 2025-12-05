/*!
Bytecode format for Vela VM

This module implements the bytecode instruction set and data structures
as defined in ADR-801.

## Instruction Set

The VM supports 256 opcodes organized into categories:
- Stack operations (0x00-0x0F)
- Arithmetic (0x10-0x1F)
- Comparison (0x20-0x2F)
- Logical (0x30-0x3F)
- Control flow (0x40-0x4F)
- Functions (0x50-0x5F)
- Collections (0x60-0x6F)
- Subscript (0x70-0x7F)
- Iteration (0x80-0x8F)
- Exception handling (0x90-0x9F)
- Imports (0xA0-0xAF)
- Debug (0xF0-0xFF)
*/

use serde::{Deserialize, Serialize};
use std::fmt;
use indexmap::IndexMap;

/// Bytecode instruction set (ADR-801)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Instruction {
    // Stack operations (0x00 - 0x0F)
    LoadConst(u16) = 0x00,      // Push constant from pool
    LoadLocal(u16) = 0x01,      // Push local variable
    StoreLocal(u16) = 0x02,     // Pop to local variable
    LoadGlobal(u16) = 0x03,     // Push global variable
    StoreGlobal(u16) = 0x04,    // Pop to global variable
    LoadAttr(u16) = 0x05,       // Load attribute (obj.attr)
    StoreAttr(u16) = 0x06,      // Store attribute
    Pop = 0x07,                 // Pop top of stack
    Dup = 0x08,                 // Duplicate top of stack

    // Arithmetic (0x10 - 0x1F)
    Add = 0x10,                 // a + b
    Sub = 0x11,                 // a - b
    Mul = 0x12,                 // a * b
    Div = 0x13,                 // a / b
    Mod = 0x14,                 // a % b
    Pow = 0x15,                 // a ** b
    Neg = 0x16,                 // -a

    // Comparison (0x20 - 0x2F)
    Eq = 0x20,                  // a == b
    Ne = 0x21,                  // a != b
    Lt = 0x22,                  // a < b
    Le = 0x23,                  // a <= b
    Gt = 0x24,                  // a > b
    Ge = 0x25,                  // a >= b

    // Logical (0x30 - 0x3F)
    And = 0x30,                 // a && b
    Or = 0x31,                  // a || b
    Not = 0x32,                 // !a

    // Control flow (0x40 - 0x4F)
    Jump(i32) = 0x40,           // Unconditional jump
    JumpIfFalse(i32) = 0x41,    // Jump if top is false
    JumpIfTrue(i32) = 0x42,     // Jump if top is true

    // Functions (0x50 - 0x5F)
    Call(u8) = 0x50,            // Call function with N args
    Return = 0x51,              // Return from function
    MakeFunction(u16) = 0x52,   // Create function object
    MakeClosure(u16, u8) = 0x53,// Create closure

    // Collections (0x60 - 0x6F)
    BuildList(u16) = 0x60,      // Build list from N items
    BuildDict(u16) = 0x61,      // Build dict from N*2 items
    BuildSet(u16) = 0x62,       // Build set from N items
    BuildTuple(u16) = 0x63,     // Build tuple from N items

    // Subscript (0x70 - 0x7F)
    LoadSubscript = 0x70,       // obj[key]
    StoreSubscript = 0x71,      // obj[key] = value
    DeleteSubscript = 0x72,     // del obj[key]

    // Iteration (0x80 - 0x8F)
    GetIter = 0x80,             // Get iterator
    ForIter(i32) = 0x81,        // Iterate (jump if exhausted)

    // Exception handling (0x90 - 0x9F)
    SetupExcept(i32) = 0x90,    // Setup exception handler
    PopExcept = 0x91,           // Pop exception handler
    Raise = 0x92,               // Raise exception

    // Imports (0xA0 - 0xAF)
    ImportName(u16) = 0xA0,     // Import module
    ImportFrom(u16) = 0xA1,     // Import from module

    // Debug (0xF0 - 0xFF)
    Nop = 0xF0,                 // No operation
    Breakpoint = 0xF1,          // Debugger breakpoint
}

/// Constant in constant pool
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constant {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(u16),       // Index to string table
    Code(u16),         // Index to code object
}

/// Code object (function/module bytecode)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeObject {
    pub name: u16,              // String table index
    pub filename: u16,          // String table index
    pub arg_count: u16,
    pub local_count: u16,
    pub stack_size: u16,        // Max stack depth
    pub flags: u16,             // IS_GENERATOR, etc.
    pub bytecode: Vec<u8>,
    pub constants: Vec<Constant>,
    pub names: Vec<u16>,        // Variable names
    pub line_numbers: Vec<(u32, u32)>, // (bytecode_offset, line_number)
}

impl CodeObject {
    pub fn new(name: u16, filename: u16) -> Self {
        Self {
            name,
            filename,
            arg_count: 0,
            local_count: 0,
            stack_size: 0,
            flags: 0,
            bytecode: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
            line_numbers: Vec::new(),
        }
    }
}

/// Complete bytecode file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bytecode {
    pub magic: u32,             // 0x56454C41 ("VELA")
    pub version: (u8, u8, u8),  // Major.Minor.Patch
    pub timestamp: u64,
    pub constants: Vec<Constant>,
    pub strings: Vec<String>,   // String table
    pub code_objects: Vec<CodeObject>,
    pub metadata: IndexMap<String, Vec<u8>>,
}

impl Bytecode {
    pub const MAGIC: u32 = 0x56454C41; // "VELA"

    pub fn new() -> Self {
        Self {
            magic: Self::MAGIC,
            version: (0, 1, 0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            constants: Vec::new(),
            strings: Vec::new(),
            code_objects: Vec::new(),
            metadata: IndexMap::new(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        if let Some(code) = self.code_objects.last_mut() {
            Self::encode_instruction_static(&instruction, &mut code.bytecode);
        }
    }

    /// Encode instruction to bytecode (static method to avoid borrow issues)
    fn encode_instruction_static(instr: &Instruction, bytecode: &mut Vec<u8>) {
        use Instruction::*;
        match instr {
            // Stack operations (0x00-0x0F)
            LoadConst(idx) => {
                bytecode.push(0x00);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            LoadLocal(idx) => {
                bytecode.push(0x01);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            StoreLocal(idx) => {
                bytecode.push(0x02);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            LoadGlobal(idx) => {
                bytecode.push(0x03);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            StoreGlobal(idx) => {
                bytecode.push(0x04);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            LoadAttr(idx) => {
                bytecode.push(0x05);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            StoreAttr(idx) => {
                bytecode.push(0x06);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            Pop => bytecode.push(0x07),
            Dup => bytecode.push(0x08),
            
            // Arithmetic (0x10-0x1F)
            Add => bytecode.push(0x10),
            Sub => bytecode.push(0x11),
            Mul => bytecode.push(0x12),
            Div => bytecode.push(0x13),
            Mod => bytecode.push(0x14),
            Pow => bytecode.push(0x15),
            Neg => bytecode.push(0x16),
            
            // Comparison (0x20-0x2F)
            Eq => bytecode.push(0x20),
            Ne => bytecode.push(0x21),
            Lt => bytecode.push(0x22),
            Le => bytecode.push(0x23),
            Gt => bytecode.push(0x24),
            Ge => bytecode.push(0x25),
            
            // Logical (0x30-0x3F)
            And => bytecode.push(0x30),
            Or => bytecode.push(0x31),
            Not => bytecode.push(0x32),
            
            // Control flow (0x40-0x4F)
            Jump(offset) => {
                bytecode.push(0x40);
                bytecode.extend_from_slice(&offset.to_le_bytes());
            }
            JumpIfFalse(offset) => {
                bytecode.push(0x41);
                bytecode.extend_from_slice(&offset.to_le_bytes());
            }
            JumpIfTrue(offset) => {
                bytecode.push(0x42);
                bytecode.extend_from_slice(&offset.to_le_bytes());
            }
            
            // Functions (0x50-0x5F)
            Call(argc) => {
                bytecode.push(0x50);
                bytecode.push(*argc);
            }
            Return => bytecode.push(0x51),
            MakeFunction(idx) => {
                bytecode.push(0x52);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            MakeClosure(idx, free_vars) => {
                bytecode.push(0x53);
                bytecode.extend_from_slice(&idx.to_le_bytes());
                bytecode.push(*free_vars);
            }
            
            // Collections (0x60-0x6F)
            BuildList(count) => {
                bytecode.push(0x60);
                bytecode.extend_from_slice(&count.to_le_bytes());
            }
            BuildDict(count) => {
                bytecode.push(0x61);
                bytecode.extend_from_slice(&count.to_le_bytes());
            }
            BuildSet(count) => {
                bytecode.push(0x62);
                bytecode.extend_from_slice(&count.to_le_bytes());
            }
            BuildTuple(count) => {
                bytecode.push(0x63);
                bytecode.extend_from_slice(&count.to_le_bytes());
            }
            
            // Subscript (0x70-0x7F)
            LoadSubscript => bytecode.push(0x70),
            StoreSubscript => bytecode.push(0x71),
            DeleteSubscript => bytecode.push(0x72),
            
            // Iteration (0x80-0x8F)
            GetIter => bytecode.push(0x80),
            ForIter(offset) => {
                bytecode.push(0x81);
                bytecode.extend_from_slice(&offset.to_le_bytes());
            }
            
            // Exception handling (0x90-0x9F)
            SetupExcept(offset) => {
                bytecode.push(0x90);
                bytecode.extend_from_slice(&offset.to_le_bytes());
            }
            PopExcept => bytecode.push(0x91),
            Raise => bytecode.push(0x92),
            
            // Imports (0xA0-0xAF)
            ImportName(idx) => {
                bytecode.push(0xA0);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            ImportFrom(idx) => {
                bytecode.push(0xA1);
                bytecode.extend_from_slice(&idx.to_le_bytes());
            }
            
            // Debug (0xF0-0xFF)
            Nop => bytecode.push(0xF0),
            Breakpoint => bytecode.push(0xFF),
        }
    }

    pub fn add_constant(&mut self, constant: Constant) -> u16 {
        self.constants.push(constant);
        (self.constants.len() - 1) as u16
    }

    pub fn add_string(&mut self, string: String) -> u16 {
        self.strings.push(string);
        (self.strings.len() - 1) as u16
    }

    pub fn add_code_object(&mut self, code: CodeObject) -> u16 {
        self.code_objects.push(code);
        (self.code_objects.len() - 1) as u16
    }

    pub fn len(&self) -> usize {
        self.code_objects.iter().map(|c| c.bytecode.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.code_objects.is_empty()
    }

    /// Get current bytecode position in the last code object
    pub fn current_position(&self) -> u16 {
        self.code_objects
            .last()
            .map(|code| code.bytecode.len() as u16)
            .unwrap_or(0)
    }

    /// Patch a jump instruction at the given position
    /// Jump instructions use i32 offset encoded as 4 bytes (little-endian)
    pub fn patch_jump(&mut self, position: usize, target: u16) {
        if let Some(code) = self.code_objects.last_mut() {
            if position + 4 < code.bytecode.len() {
                let target_i32 = target as i32;
                let bytes = target_i32.to_le_bytes();
                code.bytecode[position + 1] = bytes[0];
                code.bytecode[position + 2] = bytes[1];
                code.bytecode[position + 3] = bytes[2];
                code.bytecode[position + 4] = bytes[3];
            }
        }
    }

    /// Serialize bytecode to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize bytecode from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
    
    /// Serialize bytecode to bytes (alias for CLI compatibility)
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        self.to_bytes()
    }
    
    /// Deserialize bytecode from bytes (alias for CLI compatibility)
    pub fn deserialize(bytes: &[u8]) -> Result<Self, bincode::Error> {
        Self::from_bytes(bytes)
    }
    
    /// Disassemble bytecode for debugging (prints to stdout)
    pub fn disassemble(&self) {
        println!("=== Bytecode Disassembly ===");
        println!("Magic: 0x{:08X} ({})", self.magic, 
            String::from_utf8_lossy(&self.magic.to_be_bytes()));
        println!("Version: {}.{}.{}", self.version.0, self.version.1, self.version.2);
        println!("Timestamp: {}", self.timestamp);
        println!();
        
        // Print constants
        if !self.constants.is_empty() {
            println!("Constants ({}):", self.constants.len());
            for (i, c) in self.constants.iter().enumerate() {
                println!("  [{}] {:?}", i, c);
            }
            println!();
        }
        
        // Print strings
        if !self.strings.is_empty() {
            println!("Strings ({}):", self.strings.len());
            for (i, s) in self.strings.iter().enumerate() {
                println!("  [{}] {:?}", i, s);
            }
            println!();
        }
        
        // Print code objects
        println!("Code Objects ({}):", self.code_objects.len());
        for (i, code) in self.code_objects.iter().enumerate() {
            println!("\nCode Object [{}]:", i);
            println!("  Arguments: {}", code.arg_count);
            println!("  Locals: {}", code.local_count);
            println!("  Bytecode ({} bytes):", code.bytecode.len());
            
            // Simple bytecode disassembly
            let mut offset = 0;
            while offset < code.bytecode.len() {
                print!("    {:04}: ", offset);
                
                let opcode = code.bytecode[offset];
                offset += 1;
                
                // Decode instruction (simplified)
                match opcode {
                    0x00 => {
                        if offset + 1 < code.bytecode.len() {
                            let idx = u16::from_le_bytes([
                                code.bytecode[offset],
                                code.bytecode[offset + 1],
                            ]);
                            println!("LOAD_CONST {}", idx);
                            offset += 2;
                        }
                    }
                    0x01 => {
                        if offset + 1 < code.bytecode.len() {
                            let idx = u16::from_le_bytes([
                                code.bytecode[offset],
                                code.bytecode[offset + 1],
                            ]);
                            println!("LOAD_LOCAL {}", idx);
                            offset += 2;
                        }
                    }
                    0x10 => println!("ADD"),
                    0x11 => println!("SUB"),
                    0x12 => println!("MUL"),
                    0x13 => println!("DIV"),
                    0x20 => println!("EQ"),
                    0x21 => println!("NE"),
                    0x22 => println!("LT"),
                    0x40 => {
                        if offset + 1 < code.bytecode.len() {
                            let target = u16::from_le_bytes([
                                code.bytecode[offset],
                                code.bytecode[offset + 1],
                            ]);
                            println!("JUMP {}", target);
                            offset += 2;
                        }
                    }
                    0x50 => {
                        if offset < code.bytecode.len() {
                            let argc = code.bytecode[offset];
                            println!("CALL {}", argc);
                            offset += 1;
                        }
                    }
                    0x51 => println!("RETURN"),
                    0xFF => println!("HALT"),
                    _ => println!("UNKNOWN(0x{:02X})", opcode),
                }
            }
        }
        println!("\n============================");
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;
        match self {
            // Stack operations (0x00-0x0F)
            LoadConst(idx) => write!(f, "LOAD_CONST {}", idx),
            LoadLocal(idx) => write!(f, "LOAD_LOCAL {}", idx),
            StoreLocal(idx) => write!(f, "STORE_LOCAL {}", idx),
            LoadGlobal(idx) => write!(f, "LOAD_GLOBAL {}", idx),
            StoreGlobal(idx) => write!(f, "STORE_GLOBAL {}", idx),
            LoadAttr(idx) => write!(f, "LOAD_ATTR {}", idx),
            StoreAttr(idx) => write!(f, "STORE_ATTR {}", idx),
            Pop => write!(f, "POP"),
            Dup => write!(f, "DUP"),
            
            // Arithmetic (0x10-0x1F)
            Add => write!(f, "ADD"),
            Sub => write!(f, "SUB"),
            Mul => write!(f, "MUL"),
            Div => write!(f, "DIV"),
            Mod => write!(f, "MOD"),
            Pow => write!(f, "POW"),
            Neg => write!(f, "NEG"),
            
            // Comparison (0x20-0x2F)
            Eq => write!(f, "EQ"),
            Ne => write!(f, "NE"),
            Lt => write!(f, "LT"),
            Le => write!(f, "LE"),
            Gt => write!(f, "GT"),
            Ge => write!(f, "GE"),
            
            // Logical (0x30-0x3F)
            And => write!(f, "AND"),
            Or => write!(f, "OR"),
            Not => write!(f, "NOT"),
            
            // Control flow (0x40-0x4F)
            Jump(offset) => write!(f, "JUMP {}", offset),
            JumpIfFalse(offset) => write!(f, "JUMP_IF_FALSE {}", offset),
            JumpIfTrue(offset) => write!(f, "JUMP_IF_TRUE {}", offset),
            
            // Functions (0x50-0x5F)
            Call(argc) => write!(f, "CALL {}", argc),
            Return => write!(f, "RETURN"),
            MakeFunction(idx) => write!(f, "MAKE_FUNCTION {}", idx),
            MakeClosure(idx, free_vars) => write!(f, "MAKE_CLOSURE {} {}", idx, free_vars),
            
            // Collections (0x60-0x6F)
            BuildList(count) => write!(f, "BUILD_LIST {}", count),
            BuildDict(count) => write!(f, "BUILD_DICT {}", count),
            BuildSet(count) => write!(f, "BUILD_SET {}", count),
            BuildTuple(count) => write!(f, "BUILD_TUPLE {}", count),
            
            // Subscript (0x70-0x7F)
            LoadSubscript => write!(f, "LOAD_SUBSCRIPT"),
            StoreSubscript => write!(f, "STORE_SUBSCRIPT"),
            DeleteSubscript => write!(f, "DELETE_SUBSCRIPT"),
            
            // Iteration (0x80-0x8F)
            GetIter => write!(f, "GET_ITER"),
            ForIter(offset) => write!(f, "FOR_ITER {}", offset),
            
            // Exception handling (0x90-0x9F)
            SetupExcept(offset) => write!(f, "SETUP_EXCEPT {}", offset),
            PopExcept => write!(f, "POP_EXCEPT"),
            Raise => write!(f, "RAISE"),
            
            // Imports (0xA0-0xAF)
            ImportName(idx) => write!(f, "IMPORT_NAME {}", idx),
            ImportFrom(idx) => write!(f, "IMPORT_FROM {}", idx),
            
            // Debug (0xF0-0xFF)
            Nop => write!(f, "NOP"),
            Breakpoint => write!(f, "BREAKPOINT"),
        }
    }
}

/// Runtime value with tagged pointers (NaN-boxing)
/// 
/// Values are represented as 64-bit integers with tags:
/// - `0x0001_XXXX_XXXX_XXXX`: Int (lower 48 bits)
/// - `0xFFFE_XXXX_XXXX_XXXX`: Heap pointer (lower 48 bits)
/// - `0x0000_0000_0000_0000`: Null
/// - `0x0000_0000_0000_0002`: True
/// - `0x0000_0000_0000_0001`: False
/// - NaN-boxed floats: quiet NaNs with payload
///
/// This representation enables:
/// - Efficient 64-bit immediate values
/// - Fast type checks (single bitwise operation)
/// - Pointer compression (48-bit addresses)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Value(u64);

impl Value {
    // Tag constants
    const TAG_INT: u64 = 0x0001_0000_0000_0000;
    const TAG_PTR: u64 = 0xFFFE_0000_0000_0000;
    const MASK_PAYLOAD: u64 = 0x0000_FFFF_FFFF_FFFF;
    
    // Special values
    pub const NULL: Value = Value(0);
    pub const TRUE: Value = Value(2);
    pub const FALSE: Value = Value(1);
    
    /// Create integer value
    pub fn int(n: i64) -> Self {
        // Mask to 48 bits and preserve as signed
        let payload = (n as u64) & Self::MASK_PAYLOAD;
        Value(Self::TAG_INT | payload)
    }
    
    /// Create float value (NaN-boxing)
    pub fn float(f: f64) -> Self {
        Value(f.to_bits())
    }
    
    /// Create heap pointer value
    pub fn ptr(addr: usize) -> Self {
        Value(Self::TAG_PTR | (addr as u64 & Self::MASK_PAYLOAD))
    }
    
    /// Create boolean value
    pub fn bool(b: bool) -> Self {
        if b { Self::TRUE } else { Self::FALSE }
    }
    
    /// Check if value is null
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
    
    /// Check if value is integer
    pub fn is_int(&self) -> bool {
        (self.0 & !Self::MASK_PAYLOAD) == Self::TAG_INT
    }
    
    /// Check if value is float
    pub fn is_float(&self) -> bool {
        // Float is anything that's not tagged as int/ptr and not a special value
        let has_tag = (self.0 & !Self::MASK_PAYLOAD) == Self::TAG_INT
            || (self.0 & !Self::MASK_PAYLOAD) == Self::TAG_PTR;
        let is_special = self.0 == 0 || self.0 == 1 || self.0 == 2;
        !has_tag && !is_special
    }
    
    /// Check if value is heap pointer
    pub fn is_ptr(&self) -> bool {
        (self.0 & !Self::MASK_PAYLOAD) == Self::TAG_PTR
    }
    
    /// Check if value is boolean
    pub fn is_bool(&self) -> bool {
        self.0 == 1 || self.0 == 2
    }
    
    /// Extract integer value
    pub fn as_int(&self) -> Option<i64> {
        if self.is_int() {
            let payload = self.0 & Self::MASK_PAYLOAD;
            // Sign extend from 48 bits to 64 bits
            let sign_bit = payload & 0x0000_8000_0000_0000;
            let extended = if sign_bit != 0 {
                payload | 0xFFFF_0000_0000_0000
            } else {
                payload
            };
            Some(extended as i64)
        } else {
            None
        }
    }
    
    /// Extract float value
    pub fn as_float(&self) -> Option<f64> {
        if self.is_float() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }
    
    /// Extract pointer value
    pub fn as_ptr(&self) -> Option<usize> {
        if self.is_ptr() {
            Some((self.0 & Self::MASK_PAYLOAD) as usize)
        } else {
            None
        }
    }
    
    /// Extract boolean value
    pub fn as_bool(&self) -> Option<bool> {
        if self.is_bool() {
            Some(self.0 == 2)
        } else {
            None
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            write!(f, "null")
        } else if let Some(n) = self.as_int() {
            write!(f, "{}", n)
        } else if let Some(fl) = self.as_float() {
            write!(f, "{}", fl)
        } else if let Some(b) = self.as_bool() {
            write!(f, "{}", b)
        } else if let Some(ptr) = self.as_ptr() {
            write!(f, "ptr(0x{:x})", ptr)
        } else {
            write!(f, "unknown")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_int() {
        let v = Value::int(42);
        assert!(v.is_int());
        assert_eq!(v.as_int(), Some(42));
        assert!(!v.is_float());
        assert!(!v.is_null());
    }

    #[test]
    fn test_value_float() {
        let v = Value::float(3.14);
        assert!(v.is_float());
        assert_eq!(v.as_float(), Some(3.14));
        assert!(!v.is_int());
    }

    #[test]
    fn test_value_bool() {
        let v_true = Value::bool(true);
        let v_false = Value::bool(false);
        assert!(v_true.is_bool());
        assert!(v_false.is_bool());
        assert_eq!(v_true.as_bool(), Some(true));
        assert_eq!(v_false.as_bool(), Some(false));
    }

    #[test]
    fn test_value_null() {
        let v = Value::NULL;
        assert!(v.is_null());
        assert!(!v.is_int());
        assert!(!v.is_float());
    }

    #[test]
    fn test_value_ptr() {
        let v = Value::ptr(0x1234_5678);
        assert!(v.is_ptr());
        assert_eq!(v.as_ptr(), Some(0x1234_5678));
        assert!(!v.is_int());
    }

    #[test]
    fn test_bytecode_creation() {
        let bytecode = Bytecode::new();
        assert_eq!(bytecode.magic, Bytecode::MAGIC);
        assert_eq!(bytecode.version, (0, 1, 0));
        assert!(bytecode.is_empty());
    }

    #[test]
    fn test_code_object_creation() {
        let code = CodeObject::new(0, 1);
        assert_eq!(code.name, 0);
        assert_eq!(code.filename, 1);
        assert_eq!(code.arg_count, 0);
        assert!(code.bytecode.is_empty());
    }

    #[test]
    fn test_constant_types() {
        let c_int = Constant::Int(42);
        let c_float = Constant::Float(3.14);
        let c_bool = Constant::Bool(true);
        let c_null = Constant::Null;
        
        assert!(matches!(c_int, Constant::Int(42)));
        assert!(matches!(c_float, Constant::Float(_)));
        assert!(matches!(c_bool, Constant::Bool(true)));
        assert!(matches!(c_null, Constant::Null));
    }

    #[test]
    fn test_instruction_display() {
        assert_eq!(format!("{}", Instruction::Add), "ADD");
        assert_eq!(format!("{}", Instruction::LoadConst(0)), "LOAD_CONST 0");
        assert_eq!(format!("{}", Instruction::Jump(100)), "JUMP 100");
        assert_eq!(format!("{}", Instruction::Call(3)), "CALL 3");
    }

    #[test]
    fn test_bytecode_serialization() {
        let mut bytecode = Bytecode::new();
        bytecode.add_string("test".to_string());
        bytecode.add_constant(Constant::Int(42));
        
        let bytes = bytecode.to_bytes().unwrap();
        let deserialized = Bytecode::from_bytes(&bytes).unwrap();
        
        assert_eq!(bytecode.magic, deserialized.magic);
        assert_eq!(bytecode.strings, deserialized.strings);
        assert_eq!(bytecode.constants, deserialized.constants);
    }
}