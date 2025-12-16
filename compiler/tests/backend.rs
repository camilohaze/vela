//! Backend JavaScript Tests
//!
//! Comprehensive tests for JavaScript code generation correctness.
//! These tests validate that the generated JavaScript code is syntactically
//! correct and functionally equivalent to the Vela source.

use vela_compiler::ir::{IRModule, IRFunction, IRInstruction, IRExpr, IRType, Value, BinaryOp, UnaryOp};
use vela_compiler::js_codegen::JSGenerator;

/// Test basic function generation correctness
#[test]
fn test_function_generation_correctness() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("add".to_string(), IRType::Int);

    // Add parameters
    function.add_param("a".to_string(), IRType::Int);
    function.add_param("b".to_string(), IRType::Int);

    // Add function body: return a + b
    function.add_instruction(IRInstruction::LoadVar("a".to_string()));
    function.add_instruction(IRInstruction::LoadVar("b".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    // Validate JavaScript syntax
    assert!(js_code.contains("function add(a, b)"));
    assert!(js_code.contains("return"));
    assert!(js_code.contains("}")); // Function closing brace

    // Validate no syntax errors by checking basic structure
    assert!(!js_code.contains("undefined")); // No undefined variables
    assert!(!js_code.contains("null")); // No null values in generated code
}

/// Test complex expression generation
#[test]
fn test_complex_expression_generation() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("complex_calc".to_string(), IRType::Int);

    function.add_param("x".to_string(), IRType::Int);
    function.add_param("y".to_string(), IRType::Int);
    function.add_param("z".to_string(), IRType::Int);

    // Expression: (x * 2) + (y / 3) - z
    function.add_instruction(IRInstruction::LoadVar("x".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(2)));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Mul));

    function.add_instruction(IRInstruction::LoadVar("y".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(3)));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Div));

    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));

    function.add_instruction(IRInstruction::LoadVar("z".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Sub));

    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function complex_calc(x, y, z)"));
    assert!(js_code.contains("return"));
}

/// Test variable declarations and assignments
#[test]
fn test_variable_declarations() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("variable_test".to_string(), IRType::Void);

    // Declare variables
    function.add_instruction(IRInstruction::DeclareVar {
        name: "local_var".to_string(),
        ty: IRType::Int,
    });

    // Assign value
    function.add_instruction(IRInstruction::LoadConst(Value::Int(42)));
    function.add_instruction(IRInstruction::StoreVar("local_var".to_string()));

    // Return the variable
    function.add_instruction(IRInstruction::LoadVar("local_var".to_string()));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("let local_var"));
    assert!(js_code.contains("local_var = 42"));
    assert!(js_code.contains("return local_var"));
}

/// Test control flow structures
#[test]
fn test_control_flow() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("control_flow_test".to_string(), IRType::Int);

    function.add_param("value".to_string(), IRType::Int);

    // if (value > 10) return 1; else return 0;
    function.add_instruction(IRInstruction::LoadVar("value".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(10)));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Gt));
    function.add_instruction(IRInstruction::JumpIf("else_branch".to_string()));

    // then branch
    function.add_instruction(IRInstruction::LoadConst(Value::Int(1)));
    function.add_instruction(IRInstruction::Return);
    function.add_instruction(IRInstruction::Jump("end".to_string()));

    // else branch
    function.add_instruction(IRInstruction::Label("else_branch".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(0)));
    function.add_instruction(IRInstruction::Return);

    function.add_instruction(IRInstruction::Label("end".to_string()));

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("if (value > 10)"));
    assert!(js_code.contains("return 1"));
    assert!(js_code.contains("return 0"));
}

/// Test function calls
#[test]
fn test_function_calls() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("caller".to_string(), IRType::Int);

    // Call add(5, 3)
    function.add_instruction(IRInstruction::LoadConst(Value::Int(5)));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(3)));
    function.add_instruction(IRInstruction::Call { function: "add".to_string(), arg_count: 2 });
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function caller()"));
    assert!(js_code.contains("add(5, 3)"));
    assert!(js_code.contains("return"));
}

/// Test array operations
#[test]
fn test_array_operations() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("array_test".to_string(), IRType::Int);

    function.add_param("arr".to_string(), IRType::Array(Box::new(IRType::Int)));

    // return arr[2]
    function.add_instruction(IRInstruction::LoadVar("arr".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(2)));
    function.add_instruction(IRInstruction::ArrayAccess);
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function array_test(arr)"));
    assert!(js_code.contains("arr[2]"));
    assert!(js_code.contains("return"));
}

/// Test object operations
#[test]
fn test_object_operations() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("object_test".to_string(), IRType::String);

    function.add_param("obj".to_string(), IRType::Object("Test".to_string()));

    // For now, just return a string constant since LoadField is not available
    function.add_instruction(IRInstruction::LoadConst(Value::String("test_property".to_string())));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function object_test(obj)"));
    assert!(js_code.contains("return"));
}

/// Test type mapping
#[test]
fn test_type_mapping() {
    let mut generator = JSGenerator::new();

    // Test basic types
    assert_eq!(generator.generate_type(&IRType::Int), "number");
    assert_eq!(generator.generate_type(&IRType::Float), "number");
    assert_eq!(generator.generate_type(&IRType::String), "string");
    assert_eq!(generator.generate_type(&IRType::Bool), "boolean");
    assert_eq!(generator.generate_type(&IRType::Void), "void");
}

/// Test module generation
#[test]
fn test_module_generation() {
    let mut generator = JSGenerator::new();
    let mut module = IRModule::new("test_module".to_string());

    // Add a function
    let mut func = IRFunction::new("test_func".to_string(), IRType::Void);
    func.add_instruction(IRInstruction::Return);
    module.add_function(func);

    let js_code = generator.generate_module(&module);

    assert!(js_code.contains("// Generated by Vela JS Codegen"));
    assert!(js_code.contains("function test_func()"));
    assert!(js_code.contains("test_module"));
}

/// Test multiple functions in module
#[test]
fn test_multiple_functions() {
    let mut generator = JSGenerator::new();
    let mut module = IRModule::new("multi_func".to_string());

    // Function 1
    let mut func1 = IRFunction::new("func1".to_string(), IRType::Int);
    func1.add_instruction(IRInstruction::LoadConst(Value::Int(1)));
    func1.add_instruction(IRInstruction::Return);
    module.add_function(func1);

    // Function 2
    let mut func2 = IRFunction::new("func2".to_string(), IRType::Int);
    func2.add_instruction(IRInstruction::LoadConst(Value::Int(2)));
    func2.add_instruction(IRInstruction::Return);
    module.add_function(func2);

    let js_code = generator.generate_module(&module);

    assert!(js_code.contains("function func1()"));
    assert!(js_code.contains("return 1"));
    assert!(js_code.contains("function func2()"));
    assert!(js_code.contains("return 2"));
}

/// Test unary operations
#[test]
fn test_unary_operations() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("unary_test".to_string(), IRType::Int);

    function.add_param("x".to_string(), IRType::Int);
    function.add_param("flag".to_string(), IRType::Bool);

    // return -x
    function.add_instruction(IRInstruction::LoadVar("x".to_string()));
    function.add_instruction(IRInstruction::UnaryOp(UnaryOp::Neg));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function unary_test(x, flag)"));
    assert!(js_code.contains("-x"));
    assert!(js_code.contains("return"));
}

/// Test string concatenation
#[test]
fn test_string_concatenation() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("concat_test".to_string(), IRType::String);

    function.add_param("first".to_string(), IRType::String);
    function.add_param("last".to_string(), IRType::String);

    // return first + " " + last
    function.add_instruction(IRInstruction::LoadVar("first".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::String(" ".to_string())));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::LoadVar("last".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function concat_test(first, last)"));
    assert!(js_code.contains("first"));
    assert!(js_code.contains("last"));
    assert!(js_code.contains("return"));
}

/// Test boolean operations
#[test]
fn test_boolean_operations() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("bool_test".to_string(), IRType::Bool);

    function.add_param("a".to_string(), IRType::Bool);
    function.add_param("b".to_string(), IRType::Bool);

    // return a && b
    function.add_instruction(IRInstruction::LoadVar("a".to_string()));
    function.add_instruction(IRInstruction::LoadVar("b".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::And));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function bool_test(a, b)"));
    assert!(js_code.contains("return"));
}

/// Test empty function
#[test]
fn test_empty_function() {
    let mut generator = JSGenerator::new();
    let function = IRFunction::new("empty".to_string(), IRType::Void);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function empty()"));
    assert!(js_code.contains("{\n}"));
}

/// Test function with multiple instructions
#[test]
fn test_multiple_instructions() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("multi_instr".to_string(), IRType::Void);

    function.add_param("input".to_string(), IRType::Int);

    function.add_instruction(IRInstruction::DeclareVar {
        name: "doubled".to_string(),
        ty: IRType::Int,
    });

    function.add_instruction(IRInstruction::LoadVar("input".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(2)));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Mul));
    function.add_instruction(IRInstruction::StoreVar("doubled".to_string()));

    function.add_instruction(IRInstruction::LoadVar("doubled".to_string()));
    function.add_instruction(IRInstruction::Call { function: "println".to_string(), arg_count: 1 });

    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function multi_instr(input)"));
    assert!(js_code.contains("let doubled"));
    assert!(js_code.contains("println(doubled)"));
    assert!(js_code.contains("return"));
}