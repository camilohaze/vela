//! WebAssembly Code Generator for Vela
//!
//! This module generates WebAssembly bytecode from Vela's Intermediate Representation (IR).
//! It provides high-performance compilation target for web environments.
//!
//! Jira: VELA-1119
//! Task: TASK-118
//! Date: 2025-12-14

use crate::ir::{IRModule, IRFunction, IRInstruction, IRType, IRExpression};
use std::collections::HashMap;

/// WebAssembly code generator
pub struct WasmGenerator {
    module: IRModule,
    function_indices: HashMap<String, u32>,
    global_indices: HashMap<String, u32>,
    type_indices: HashMap<String, u32>,
    next_function_index: u32,
    next_global_index: u32,
    next_type_index: u32,
}

impl WasmGenerator {
    /// Create a new WASM generator for the given IR module
    pub fn new(module: IRModule) -> Self {
        Self {
            module,
            function_indices: HashMap::new(),
            global_indices: HashMap::new(),
            type_indices: HashMap::new(),
            next_function_index: 0,
            next_global_index: 0,
            next_type_index: 0,
        }
    }

    /// Generate WebAssembly module
    pub fn generate(&mut self) -> Result<Vec<u8>, WasmError> {
        let mut wasm_bytes = Vec::new();

        // WASM magic number and version
        wasm_bytes.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // \0asm
        wasm_bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // version 1

        // Generate sections
        self.generate_type_section(&mut wasm_bytes)?;
        self.generate_import_section(&mut wasm_bytes)?;
        self.generate_function_section(&mut wasm_bytes)?;
        self.generate_global_section(&mut wasm_bytes)?;
        self.generate_export_section(&mut wasm_bytes)?;
        self.generate_code_section(&mut wasm_bytes)?;

        Ok(wasm_bytes)
    }

    /// Generate type section
    fn generate_type_section(&mut self, wasm: &mut Vec<u8>) -> Result<(), WasmError> {
        let mut section = Vec::new();

        // Count function types
        let type_count = self.module.functions.len() as u32;
        section.extend_from_slice(&leb128_encode(type_count));

        for function in &self.module.functions {
            // Function type
            section.push(0x60); // func type

            // Parameter types
            let param_count = function.parameters.len() as u32;
            section.extend_from_slice(&leb128_encode(param_count));
            for param in &function.parameters {
                section.push(self.map_type_to_wasm(&param.param_type));
            }

            // Return types
            if let Some(return_type) = &function.return_type {
                section.push(0x01); // one return type
                section.push(self.map_type_to_wasm(return_type));
            } else {
                section.push(0x00); // no return type
            }

            // Register type index
            let type_index = self.next_type_index;
            self.type_indices.insert(function.name.clone(), type_index);
            self.next_type_index += 1;
        }

        // Add section to WASM
        self.add_section(wasm, 0x01, &section); // Type section
        Ok(())
    }

    /// Generate import section
    fn generate_import_section(&mut self, wasm: &mut Vec<u8>) -> Result<(), WasmError> {
        // For now, minimal imports (could add WASI or JS imports later)
        let section = vec![0x00]; // 0 imports
        self.add_section(wasm, 0x02, &section); // Import section
        Ok(())
    }

    /// Generate function section
    fn generate_function_section(&mut self, wasm: &mut Vec<u8>) -> Result<(), WasmError> {
        let mut section = Vec::new();

        let function_count = self.module.functions.len() as u32;
        section.extend_from_slice(&leb128_encode(function_count));

        for function in &self.module.functions {
            let type_index = self.type_indices[&function.name];
            section.extend_from_slice(&leb128_encode(type_index));

            // Register function index
            let func_index = self.next_function_index;
            self.function_indices.insert(function.name.clone(), func_index);
            self.next_function_index += 1;
        }

        self.add_section(wasm, 0x03, &section); // Function section
        Ok(())
    }

    /// Generate global section
    fn generate_global_section(&mut self, wasm: &mut Vec<u8>) -> Result<(), WasmError> {
        let mut section = Vec::new();

        // Count globals
        let global_count = self.module.globals.len() as u32;
        section.extend_from_slice(&leb128_encode(global_count));

        for global in &self.module.globals {
            // Global type
            section.push(self.map_type_to_wasm(&global.global_type));
            section.push(0x00); // mutable = false (for now)

            // Initialize with zero
            section.push(0x41); // i32.const
            section.push(0x00); // 0
            section.push(0x0B); // end

            // Register global index
            let global_index = self.next_global_index;
            self.global_indices.insert(global.name.clone(), global_index);
            self.next_global_index += 1;
        }

        self.add_section(wasm, 0x06, &section); // Global section
        Ok(())
    }

    /// Generate export section
    fn generate_export_section(&mut self, wasm: &mut Vec<u8>) -> Result<(), WasmError> {
        let mut section = Vec::new();

        // Export all public functions
        let export_count = self.module.functions.iter()
            .filter(|f| f.is_public)
            .count() as u32;
        section.extend_from_slice(&leb128_encode(export_count));

        for function in &self.module.functions {
            if function.is_public {
                // Export name
                let name_bytes = function.name.as_bytes();
                section.extend_from_slice(&leb128_encode(name_bytes.len() as u32));
                section.extend_from_slice(name_bytes);

                // Export type (function)
                section.push(0x00); // function export

                // Function index
                let func_index = self.function_indices[&function.name];
                section.extend_from_slice(&leb128_encode(func_index));
            }
        }

        self.add_section(wasm, 0x07, &section); // Export section
        Ok(())
    }

    /// Generate code section
    fn generate_code_section(&mut self, wasm: &mut Vec<u8>) -> Result<(), WasmError> {
        let mut section = Vec::new();

        let function_count = self.module.functions.len() as u32;
        section.extend_from_slice(&leb128_encode(function_count));

        for function in &self.module.functions {
            let mut function_body = Vec::new();

            // Generate function body
            self.generate_function_body(function, &mut function_body)?;

            // Function size
            section.extend_from_slice(&leb128_encode(function_body.len() as u32));
            section.extend_from_slice(&function_body);
        }

        self.add_section(wasm, 0x0A, &section); // Code section
        Ok(())
    }

    /// Generate function body
    fn generate_function_body(&self, function: &IRFunction, body: &mut Vec<u8>) -> Result<(), WasmError> {
        // Local variables (none for now)
        body.extend_from_slice(&leb128_encode(0)); // 0 locals

        // Generate instructions
        for instruction in &function.body {
            self.generate_instruction(instruction, body)?;
        }

        // End function
        body.push(0x0B); // end
        Ok(())
    }

    /// Generate single instruction
    fn generate_instruction(&self, instruction: &IRInstruction, body: &mut Vec<u8>) -> Result<(), WasmError> {
        match instruction {
            IRInstruction::Const(value) => {
                match value {
                    IRExpression::Int32(val) => {
                        body.push(0x41); // i32.const
                        body.extend_from_slice(&leb128_encode(*val as u32));
                    }
                    IRExpression::Int64(val) => {
                        body.push(0x42); // i64.const
                        body.extend_from_slice(&leb128_encode(*val));
                    }
                    IRExpression::Float32(val) => {
                        body.push(0x43); // f32.const
                        body.extend_from_slice(&val.to_le_bytes());
                    }
                    IRExpression::Float64(val) => {
                        body.push(0x44); // f64.const
                        body.extend_from_slice(&val.to_le_bytes());
                    }
                }
            }
            IRInstruction::Add => body.push(0x6A), // i32.add
            IRInstruction::Sub => body.push(0x6B), // i32.sub
            IRInstruction::Mul => body.push(0x6C), // i32.mul
            IRInstruction::Div => body.push(0x6D), // i32.div_s
            IRInstruction::Return => body.push(0x0F), // return
            IRInstruction::Call(function_name) => {
                body.push(0x10); // call
                if let Some(&func_index) = self.function_indices.get(function_name) {
                    body.extend_from_slice(&leb128_encode(func_index));
                } else {
                    return Err(WasmError::UndefinedFunction(function_name.clone()));
                }
            }
            IRInstruction::LocalGet(index) => {
                body.push(0x20); // local.get
                body.extend_from_slice(&leb128_encode(*index));
            }
            IRInstruction::LocalSet(index) => {
                body.push(0x21); // local.set
                body.extend_from_slice(&leb128_encode(*index));
            }
            IRInstruction::GlobalGet(name) => {
                body.push(0x23); // global.get
                if let Some(&global_index) = self.global_indices.get(name) {
                    body.extend_from_slice(&leb128_encode(global_index));
                } else {
                    return Err(WasmError::UndefinedGlobal(name.clone()));
                }
            }
            IRInstruction::GlobalSet(name) => {
                body.push(0x24); // global.set
                if let Some(&global_index) = self.global_indices.get(name) {
                    body.extend_from_slice(&leb128_encode(global_index));
                } else {
                    return Err(WasmError::UndefinedGlobal(name.clone()));
                }
            }
        }
        Ok(())
    }

    /// Map Vela IR type to WASM type
    fn map_type_to_wasm(&self, ir_type: &IRType) -> u8 {
        match ir_type {
            IRType::I32 => 0x7F, // i32
            IRType::I64 => 0x7E, // i64
            IRType::F32 => 0x7D, // f32
            IRType::F64 => 0x7C, // f64
            IRType::Void => panic!("Void type in WASM mapping"),
        }
    }

    /// Add a section to WASM module
    fn add_section(&self, wasm: &mut Vec<u8>, section_id: u8, section_data: &[u8]) {
        wasm.push(section_id);
        wasm.extend_from_slice(&leb128_encode(section_data.len() as u32));
        wasm.extend_from_slice(section_data);
    }
}

/// LEB128 encoding for WASM
fn leb128_encode(mut value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
    bytes
}

/// Errors that can occur during WASM generation
#[derive(Debug, Clone)]
pub enum WasmError {
    UndefinedFunction(String),
    UndefinedGlobal(String),
    InvalidInstruction(String),
}

impl std::fmt::Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WasmError::UndefinedFunction(name) => write!(f, "Undefined function: {}", name),
            WasmError::UndefinedGlobal(name) => write!(f, "Undefined global: {}", name),
            WasmError::InvalidInstruction(inst) => write!(f, "Invalid instruction: {}", inst),
        }
    }
}

impl std::error::Error for WasmError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IRModule, IRFunction, IRParameter, IRInstruction, IRType, IRExpression};

    #[test]
    fn test_wasm_generator_creation() {
        let module = IRModule {
            name: "test".to_string(),
            functions: vec![],
            globals: vec![],
        };
        let generator = WasmGenerator::new(module);
        assert_eq!(generator.next_function_index, 0);
    }

    #[test]
    fn test_simple_function_generation() {
        let function = IRFunction {
            name: "add".to_string(),
            parameters: vec![
                IRParameter { name: "a".to_string(), param_type: IRType::I32 },
                IRParameter { name: "b".to_string(), param_type: IRType::I32 },
            ],
            return_type: Some(IRType::I32),
            body: vec![
                IRInstruction::LocalGet(0), // a
                IRInstruction::LocalGet(1), // b
                IRInstruction::Add,
                IRInstruction::Return,
            ],
            is_public: true,
        };

        let module = IRModule {
            name: "test".to_string(),
            functions: vec![function],
            globals: vec![],
        };

        let mut generator = WasmGenerator::new(module);
        let result = generator.generate();
        assert!(result.is_ok());

        let wasm_bytes = result.unwrap();
        // Check WASM magic number
        assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        // Check version
        assert_eq!(&wasm_bytes[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_leb128_encoding() {
        assert_eq!(leb128_encode(0), vec![0x00]);
        assert_eq!(leb128_encode(1), vec![0x01]);
        assert_eq!(leb128_encode(127), vec![0x7F]);
        assert_eq!(leb128_encode(128), vec![0x80, 0x01]);
    }
}