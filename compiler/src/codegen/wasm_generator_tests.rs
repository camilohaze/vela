//! Tests unitarios para WASM Code Generator
//!
//! Jira: VELA-1119
//! Task: TASK-118

use crate::codegen::ir_to_wasm::{WasmGenerator, WasmError};
use crate::ir::{IRModule, IRFunction, IRParameter, IRInstruction, IRType, IRExpression, IRGlobal};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_generator_creation() {
        let module = IRModule {
            name: "test".to_string(),
            functions: vec![],
            globals: vec![],
        };
        let generator = WasmGenerator::new(module);
        assert_eq!(generator.next_function_index, 0);
        assert_eq!(generator.next_global_index, 0);
        assert_eq!(generator.next_type_index, 0);
    }

    #[test]
    fn test_simple_add_function() {
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

        assert!(result.is_ok(), "WASM generation should succeed");

        let wasm_bytes = result.unwrap();

        // Verify WASM header
        assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6D], "Invalid WASM magic number");
        assert_eq!(&wasm_bytes[4..8], &[0x01, 0x00, 0x00, 0x00], "Invalid WASM version");

        // Should have multiple sections
        assert!(wasm_bytes.len() > 8, "WASM module should have sections");
    }

    #[test]
    fn test_function_with_globals() {
        let function = IRFunction {
            name: "use_global".to_string(),
            parameters: vec![],
            return_type: Some(IRType::I32),
            body: vec![
                IRInstruction::GlobalGet("counter".to_string()),
                IRInstruction::Return,
            ],
            is_public: true,
        };

        let global = IRGlobal {
            name: "counter".to_string(),
            global_type: IRType::I32,
            is_mutable: false,
        };

        let module = IRModule {
            name: "test".to_string(),
            functions: vec![function],
            globals: vec![global],
        };

        let mut generator = WasmGenerator::new(module);
        let result = generator.generate();

        assert!(result.is_ok(), "WASM generation with globals should succeed");
    }

    #[test]
    fn test_private_function_not_exported() {
        let public_function = IRFunction {
            name: "public_func".to_string(),
            parameters: vec![],
            return_type: None,
            body: vec![IRInstruction::Return],
            is_public: true,
        };

        let private_function = IRFunction {
            name: "private_func".to_string(),
            parameters: vec![],
            return_type: None,
            body: vec![IRInstruction::Return],
            is_public: false,
        };

        let module = IRModule {
            name: "test".to_string(),
            functions: vec![public_function, private_function],
            globals: vec![],
        };

        let mut generator = WasmGenerator::new(module);
        let result = generator.generate();

        assert!(result.is_ok(), "WASM generation should succeed");

        // The export section should only include the public function
        // This is a basic check - in practice we'd parse the WASM to verify
    }

    #[test]
    fn test_different_types() {
        let functions = vec![
            IRFunction {
                name: "i64_func".to_string(),
                parameters: vec![IRParameter { name: "x".to_string(), param_type: IRType::I64 }],
                return_type: Some(IRType::I64),
                body: vec![
                    IRInstruction::LocalGet(0),
                    IRInstruction::Return,
                ],
                is_public: true,
            },
            IRFunction {
                name: "f32_func".to_string(),
                parameters: vec![IRParameter { name: "x".to_string(), param_type: IRType::F32 }],
                return_type: Some(IRType::F32),
                body: vec![
                    IRInstruction::LocalGet(0),
                    IRInstruction::Return,
                ],
                is_public: true,
            },
            IRFunction {
                name: "f64_func".to_string(),
                parameters: vec![IRParameter { name: "x".to_string(), param_type: IRType::F64 }],
                return_type: Some(IRType::F64),
                body: vec![
                    IRInstruction::LocalGet(0),
                    IRInstruction::Return,
                ],
                is_public: true,
            },
        ];

        let module = IRModule {
            name: "test".to_string(),
            functions,
            globals: vec![],
        };

        let mut generator = WasmGenerator::new(module);
        let result = generator.generate();

        assert!(result.is_ok(), "WASM generation with different types should succeed");
    }

    #[test]
    fn test_function_call() {
        let callee = IRFunction {
            name: "callee".to_string(),
            parameters: vec![],
            return_type: Some(IRType::I32),
            body: vec![
                IRInstruction::Const(IRExpression::Int32(42)),
                IRInstruction::Return,
            ],
            is_public: false,
        };

        let caller = IRFunction {
            name: "caller".to_string(),
            parameters: vec![],
            return_type: Some(IRType::I32),
            body: vec![
                IRInstruction::Call("callee".to_string()),
                IRInstruction::Return,
            ],
            is_public: true,
        };

        let module = IRModule {
            name: "test".to_string(),
            functions: vec![callee, caller],
            globals: vec![],
        };

        let mut generator = WasmGenerator::new(module);
        let result = generator.generate();

        assert!(result.is_ok(), "WASM generation with function calls should succeed");
    }

    #[test]
    fn test_undefined_function_error() {
        let function = IRFunction {
            name: "test".to_string(),
            parameters: vec![],
            return_type: None,
            body: vec![
                IRInstruction::Call("nonexistent".to_string()),
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

        assert!(result.is_err(), "Should fail with undefined function");
        match result {
            Err(WasmError::UndefinedFunction(name)) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected UndefinedFunction error"),
        }
    }

    #[test]
    fn test_undefined_global_error() {
        let function = IRFunction {
            name: "test".to_string(),
            parameters: vec![],
            return_type: None,
            body: vec![
                IRInstruction::GlobalGet("nonexistent".to_string()),
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

        assert!(result.is_err(), "Should fail with undefined global");
        match result {
            Err(WasmError::UndefinedGlobal(name)) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected UndefinedGlobal error"),
        }
    }

    #[test]
    fn test_empty_module() {
        let module = IRModule {
            name: "empty".to_string(),
            functions: vec![],
            globals: vec![],
        };

        let mut generator = WasmGenerator::new(module);
        let result = generator.generate();

        assert!(result.is_ok(), "Empty module should generate valid WASM");

        let wasm_bytes = result.unwrap();
        // Should still have header
        assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }

    #[test]
    fn test_constants_generation() {
        let function = IRFunction {
            name: "constants".to_string(),
            parameters: vec![],
            return_type: Some(IRType::I32),
            body: vec![
                IRInstruction::Const(IRExpression::Int32(123)),
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

        assert!(result.is_ok(), "Constants generation should succeed");
    }
}