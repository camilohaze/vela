//! Tests for JavaScript-WebAssembly Glue Code Generator
//!
//! This module tests the JS glue code generation for WASM interop.
//!
//! Jira: VELA-1119
//! Task: TASK-119
//! Date: 2025-12-14

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IRModule, IRFunction, IRParam, IRType, IRGlobal, Value};

    #[test]
    fn test_simple_function_glue_generation() {
        let mut function = IRFunction::new("add".to_string(), IRType::Int);
        function.add_param("a".to_string(), IRType::Int);
        function.add_param("b".to_string(), IRType::Int);

        let mut module = IRModule::new("test".to_string());
        module.add_function(function);

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        // Check class structure
        assert!(glue_code.contains("export class TestModule"));
        assert!(glue_code.contains("constructor(wasmModule)"));
        assert!(glue_code.contains("async init()"));

        // Check function wrapper
        assert!(glue_code.contains("add(arg0 /* number */, arg1 /* number */)"));
        assert!(glue_code.contains("this.exports.add(arg0, arg1)"));
        assert!(glue_code.contains("return this.i32ToJS(result)"));

        // Check helper functions
        assert!(glue_code.contains("export async function loadWasmModule"));
        assert!(glue_code.contains("export async function createWasmInstance"));
    }

    #[test]
    fn test_void_function_glue_generation() {
        let mut function = IRFunction::new("print".to_string(), IRType::Void);
        function.add_param("message".to_string(), IRType::Int); // pointer

        let mut module = IRModule::new("test".to_string());
        module.add_function(function);

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        assert!(glue_code.contains("print(arg0 /* number */)"));
        assert!(glue_code.contains("this.exports.print(arg0)"));
        assert!(!glue_code.contains("return this."));
    }

    #[test]
    fn test_multiple_types_glue_generation() {
        let mut add_int = IRFunction::new("add_int".to_string(), IRType::Int);
        add_int.add_param("a".to_string(), IRType::Int);
        add_int.add_param("b".to_string(), IRType::Int);

        let mut add_float = IRFunction::new("add_float".to_string(), IRType::Float);
        add_float.add_param("a".to_string(), IRType::Float);
        add_float.add_param("b".to_string(), IRType::Float);

        let mut big_calc = IRFunction::new("big_calc".to_string(), IRType::Int);
        big_calc.add_param("a".to_string(), IRType::Int);

        let mut module = IRModule::new("math".to_string());
        module.add_function(add_int);
        module.add_function(add_float);
        module.add_function(big_calc);

        let generator = JSGlueGenerator::new(module, "MathModule".to_string());
        let glue_code = generator.generate_glue_code();

        // Check all functions are generated
        assert!(glue_code.contains("add_int"));
        assert!(glue_code.contains("add_float"));
        assert!(glue_code.contains("big_calc"));

        // Check type conversions
        assert!(glue_code.contains("this.i32ToJS"));
        assert!(glue_code.contains("this.f64ToJS"));
    }

    #[test]
    fn test_globals_initialization() {
        let globals = vec![
            IRGlobal {
                name: "counter".to_string(),
                ty: IRType::Int,
                initial_value: Some(Value::Int(0)),
            },
            IRGlobal {
                name: "pi".to_string(),
                ty: IRType::Float,
                initial_value: Some(Value::Float(3.14159)),
            },
        ];

        let mut module = IRModule::new("test".to_string());
        for global in globals {
            module.add_global(global);
        }

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        assert!(glue_code.contains("initGlobals()"));
        assert!(glue_code.contains("this.counter = 0"));
        assert!(glue_code.contains("this.pi = 0.0"));
    }

    #[test]
    fn test_typescript_definitions_generation() {
        let mut calculate = IRFunction::new("calculate".to_string(), IRType::Float);
        calculate.add_param("x".to_string(), IRType::Float);
        calculate.add_param("y".to_string(), IRType::Float);

        let mut process = IRFunction::new("process".to_string(), IRType::Void);
        process.add_param("data".to_string(), IRType::Int);

        let mut module = IRModule::new("processor".to_string());
        module.add_function(calculate);
        module.add_function(process);

        let generator = TypeScriptGenerator::new(module, "ProcessorModule".to_string());
        let ts_code = generator.generate_types();

        assert!(ts_code.contains("export declare class ProcessorModule"));
        assert!(ts_code.contains("calculate(arg0: number, arg1: number): number"));
        assert!(ts_code.contains("process(arg0: number): void"));
        assert!(ts_code.contains("init(): Promise<boolean>"));
    }

    #[test]
    fn test_memory_helpers_presence() {
        let module = IRModule::new("test".to_string());

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        // Check memory management functions
        assert!(glue_code.contains("readString(ptr, len)"));
        assert!(glue_code.contains("writeString(str)"));
        assert!(glue_code.contains("allocate(size)"));
        assert!(glue_code.contains("deallocate(ptr)"));

        // Check type conversion helpers
        assert!(glue_code.contains("i32ToJS(value)"));
        assert!(glue_code.contains("i64ToJS(value)"));
        assert!(glue_code.contains("f32ToJS(value)"));
        assert!(glue_code.contains("f64ToJS(value)"));
    }

    #[test]
    fn test_helper_functions_exported() {
        let module = IRModule::new("test".to_string());

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        // Check exported helper functions
        assert!(glue_code.contains("export async function loadWasmModule"));
        assert!(glue_code.contains("export async function createWasmInstance"));
        assert!(glue_code.contains("export async function loadAndInstantiate"));
    }

    #[test]
    fn test_error_handling_in_functions() {
        let mut function = IRFunction::new("risky_operation".to_string(), IRType::Int);
        function.add_param("input".to_string(), IRType::Int);

        let mut module = IRModule::new("test".to_string());
        module.add_function(function);

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        assert!(glue_code.contains("try {"));
        assert!(glue_code.contains("} catch (error) {"));
        assert!(glue_code.contains("console.error(`Error calling"));
        assert!(glue_code.contains("throw error;"));
    }

    #[test]
    fn test_class_structure_complete() {
        let module = IRModule::new("test".to_string());

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        // Check class structure
        assert!(glue_code.contains("export class TestModule {"));
        assert!(glue_code.contains("constructor(wasmModule) {"));
        assert!(glue_code.contains("this.wasmModule = wasmModule;"));
        assert!(glue_code.contains("this.instance = null;"));
        assert!(glue_code.contains("async init()"));
        assert!(glue_code.contains("} // end of class"));
    }

    #[test]
    fn test_string_type_handling() {
        let mut function = IRFunction::new("process_string".to_string(), IRType::String);
        function.add_param("input".to_string(), IRType::String);

        let mut module = IRModule::new("test".to_string());
        module.add_function(function);

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        assert!(glue_code.contains("process_string(arg0 /* string */)"));
        assert!(glue_code.contains("this.exports.process_string(arg0)"));
    }

    #[test]
    fn test_array_type_handling() {
        let mut function = IRFunction::new("process_array".to_string(), IRType::Array(Box::new(IRType::Int)));
        function.add_param("arr".to_string(), IRType::Array(Box::new(IRType::Int)));

        let mut module = IRModule::new("test".to_string());
        module.add_function(function);

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        assert!(glue_code.contains("process_array(arg0 /* array */)"));
    }
}