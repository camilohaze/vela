/*!
# Tests for LLVM Backend

Tests for the LLVM code generation backend, ensuring correct translation
from Vela IR to LLVM IR with proper type handling and optimization.
*/

#[cfg(feature = "llvm_backend")]
use super::ir_to_llvm::LLVMGenerator;
#[cfg(feature = "llvm_backend")]
use crate::ir::{IRModule, IRFunction, IRParam, IRType, IRInstruction, Value};
#[cfg(feature = "llvm_backend")]
use inkwell::context::Context;
#[cfg(feature = "llvm_backend")]
use inkwell::OptimizationLevel;

#[cfg(feature = "llvm_backend")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function_generation() {
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "test_module");

        // Create a simple add function: fn add(a: Int, b: Int) -> Int { return a + b }
        let function = IRFunction {
            name: "add".to_string(),
            params: vec![
                IRParam { name: "a".to_string(), ty: IRType::Int },
                IRParam { name: "b".to_string(), ty: IRType::Int },
            ],
            return_type: IRType::Int,
            locals: vec![],
            body: vec![
                IRInstruction::BinaryOp {
                    dest: "result".to_string(),
                    op: "+".to_string(),
                    left: "a".to_string(),
                    right: "b".to_string(),
                },
                IRInstruction::Return {
                    value: Some("result".to_string()),
                },
            ],
            position: None,
        };

        let module = IRModule {
            name: "test".to_string(),
            functions: vec![function],
            globals: vec![],
            position: None,
        };

        // Generate LLVM IR
        let result = generator.generate(&module);
        assert!(result.is_ok(), "Failed to generate LLVM IR: {:?}", result.err());

        // Verify the generated IR contains expected elements
        let ir_string = generator.to_string();
        println!("Generated LLVM IR:\n{}", ir_string);

        assert!(ir_string.contains("define i64 @add("));
        assert!(ir_string.contains("i64 %a, i64 %b)"));
        assert!(ir_string.contains("add i64"));
        assert!(ir_string.contains("ret i64"));
    }

    #[test]
    fn test_float_operations() {
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "float_test");

        let function = IRFunction {
            name: "multiply_float".to_string(),
            params: vec![
                IRParam { name: "x".to_string(), ty: IRType::Float },
                IRParam { name: "y".to_string(), ty: IRType::Float },
            ],
            return_type: IRType::Float,
            locals: vec![],
            body: vec![
                IRInstruction::BinaryOp {
                    dest: "result".to_string(),
                    op: "*".to_string(),
                    left: "x".to_string(),
                    right: "y".to_string(),
                },
                IRInstruction::Return {
                    value: Some("result".to_string()),
                },
            ],
            position: None,
        };

        let module = IRModule {
            name: "float_test".to_string(),
            functions: vec![function],
            globals: vec![],
            position: None,
        };

        let result = generator.generate(&module);
        assert!(result.is_ok());

        let ir_string = generator.to_string();
        assert!(ir_string.contains("define double @multiply_float("));
        assert!(ir_string.contains("fmul double"));
        assert!(ir_string.contains("ret double"));
    }

    #[test]
    fn test_constant_loading() {
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "const_test");

        let function = IRFunction {
            name: "return_constant".to_string(),
            params: vec![],
            return_type: IRType::Int,
            locals: vec![],
            body: vec![
                IRInstruction::LoadConst {
                    dest: "value".to_string(),
                    value: Value::Int(42),
                },
                IRInstruction::Return {
                    value: Some("value".to_string()),
                },
            ],
            position: None,
        };

        let module = IRModule {
            name: "const_test".to_string(),
            functions: vec![function],
            globals: vec![],
            position: None,
        };

        let result = generator.generate(&module);
        assert!(result.is_ok());

        let ir_string = generator.to_string();
        assert!(ir_string.contains("define i64 @return_constant()"));
        assert!(ir_string.contains("ret i64 42"));
    }

    #[test]
    fn test_function_call() {
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "call_test");

        // Create add function
        let add_function = IRFunction {
            name: "add".to_string(),
            params: vec![
                IRParam { name: "a".to_string(), ty: IRType::Int },
                IRParam { name: "b".to_string(), ty: IRType::Int },
            ],
            return_type: IRType::Int,
            locals: vec![],
            body: vec![
                IRInstruction::BinaryOp {
                    dest: "result".to_string(),
                    op: "+".to_string(),
                    left: "a".to_string(),
                    right: "b".to_string(),
                },
                IRInstruction::Return {
                    value: Some("result".to_string()),
                },
            ],
            position: None,
        };

        // Create main function that calls add
        let main_function = IRFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: IRType::Int,
            locals: vec![],
            body: vec![
                IRInstruction::LoadConst {
                    dest: "x".to_string(),
                    value: Value::Int(10),
                },
                IRInstruction::LoadConst {
                    dest: "y".to_string(),
                    value: Value::Int(20),
                },
                IRInstruction::Call {
                    dest: Some("result".to_string()),
                    function: "add".to_string(),
                    args: vec!["x".to_string(), "y".to_string()],
                },
                IRInstruction::Return {
                    value: Some("result".to_string()),
                },
            ],
            position: None,
        };

        let module = IRModule {
            name: "call_test".to_string(),
            functions: vec![add_function, main_function],
            globals: vec![],
            position: None,
        };

        let result = generator.generate(&module);
        assert!(result.is_ok());

        let ir_string = generator.to_string();
        assert!(ir_string.contains("define i64 @add("));
        assert!(ir_string.contains("define i64 @main()"));
        assert!(ir_string.contains("call i64 @add("));
    }

    #[test]
    fn test_void_function() {
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "void_test");

        let function = IRFunction {
            name: "print_hello".to_string(),
            params: vec![],
            return_type: IRType::Void,
            locals: vec![],
            body: vec![
                // Just return void
                IRInstruction::Return { value: None },
            ],
            position: None,
        };

        let module = IRModule {
            name: "void_test".to_string(),
            functions: vec![function],
            globals: vec![],
            position: None,
        };

        let result = generator.generate(&module);
        assert!(result.is_ok());

        let ir_string = generator.to_string();
        assert!(ir_string.contains("define void @print_hello()"));
        assert!(ir_string.contains("ret void"));
    }

    #[test]
    fn test_string_handling() {
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "string_test");

        let function = IRFunction {
            name: "return_string".to_string(),
            params: vec![],
            return_type: IRType::String,
            locals: vec![],
            body: vec![
                IRInstruction::LoadConst {
                    dest: "message".to_string(),
                    value: Value::String("Hello, LLVM!".to_string()),
                },
                IRInstruction::Return {
                    value: Some("message".to_string()),
                },
            ],
            position: None,
        };

        let module = IRModule {
            name: "string_test".to_string(),
            functions: vec![function],
            globals: vec![],
            position: None,
        };

        let result = generator.generate(&module);
        assert!(result.is_ok());

        let ir_string = generator.to_string();
        assert!(ir_string.contains("define { i32, i8* } @return_string()"));
        assert!(ir_string.contains("@.str = private unnamed_addr constant"));
    }

    #[test]
    fn test_bitcode_generation() {
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "bitcode_test");

        let function = IRFunction {
            name: "simple".to_string(),
            params: vec![],
            return_type: IRType::Int,
            locals: vec![],
            body: vec![
                IRInstruction::LoadConst {
                    dest: "value".to_string(),
                    value: Value::Int(123),
                },
                IRInstruction::Return {
                    value: Some("value".to_string()),
                },
            ],
            position: None,
        };

        let module = IRModule {
            name: "bitcode_test".to_string(),
            functions: vec![function],
            globals: vec![],
            position: None,
        };

        let result = generator.generate(&module);
        assert!(result.is_ok());

        // Test bitcode writing (this will create a temporary file)
        let temp_file = "test_output.bc";
        let bitcode_result = generator.write_bitcode_to_file(temp_file);
        assert!(bitcode_result.is_ok());

        // Clean up
        std::fs::remove_file(temp_file).ok();
    }
}