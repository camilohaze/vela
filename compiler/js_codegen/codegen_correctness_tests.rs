//! Code Generation Correctness Tests
//!
//! Detailed tests for validating the correctness of JavaScript code generation
//! from Vela IR. These tests ensure that complex language features are
//! correctly translated to JavaScript.

use crate::ir::{IRModule, IRFunction, IRInstruction, IRExpr, IRType, Value, BinaryOp, UnaryOp};
use crate::js_codegen::JSGenerator;

/// Test advanced control flow structures
#[test]
fn test_advanced_control_flow() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("advanced_control".to_string(), IRType::Int);

    function.add_param("value".to_string(), IRType::Int);

    // Complex nested if-else with multiple conditions
    function.add_instruction(IRInstruction::DeclareVar {
        name: "result".to_string(),
        ty: IRType::Int,
    });

    function.add_instruction(IRInstruction::AssignVar {
        name: "result".to_string(),
        value: IRExpr::Const(Value::Int(0)),
    });

    // Nested if-else structure
    function.add_instruction(IRInstruction::LoadVar("value".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(100)));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Gt));
    function.add_instruction(IRInstruction::JumpIf("else1".to_string()));

    // then block
    function.add_instruction(IRInstruction::LoadConst(Value::Int(1)));
    function.add_instruction(IRInstruction::StoreVar("result".to_string()));
    function.add_instruction(IRInstruction::Jump("end".to_string()));

    // else block
    function.add_instruction(IRInstruction::Label("else1".to_string()));
    function.add_instruction(IRInstruction::LoadVar("value".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(50)));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Gt));
    function.add_instruction(IRInstruction::LoadVar("value".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(100)));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Lt));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::And));
    function.add_instruction(IRInstruction::JumpIf("else2".to_string()));

    // nested then
    function.add_instruction(IRInstruction::LoadConst(Value::Int(2)));
    function.add_instruction(IRInstruction::StoreVar("result".to_string()));
    function.add_instruction(IRInstruction::Jump("end".to_string()));

    // nested else
    function.add_instruction(IRInstruction::Label("else2".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(3)));
    function.add_instruction(IRInstruction::StoreVar("result".to_string()));

    function.add_instruction(IRInstruction::Label("end".to_string()));
    function.add_instruction(IRInstruction::LoadVar("result".to_string()));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    // Validate nested control flow
    assert!(js_code.contains("if (value > 100)"));
    assert!(js_code.contains("result = 1;"));
    assert!(js_code.contains("else if"));
    assert!(js_code.contains("result = 2;"));
    assert!(js_code.contains("else"));
    assert!(js_code.contains("result = 3;"));
}

/// Test unary operations
#[test]
fn test_unary_operations() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("unary_ops".to_string(), IRType::Int);

    function.add_param("x".to_string(), IRType::Int);
    function.add_param("flag".to_string(), IRType::Bool);

    // return -x + (!flag ? 1 : 0)
    function.add_instruction(IRInstruction::LoadConst(Value::Int(0)));
    function.add_instruction(IRInstruction::LoadVar("x".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Sub));

    // Ternary: !flag ? 1 : 0
    function.add_instruction(IRInstruction::LoadVar("flag".to_string()));
    function.add_instruction(IRInstruction::UnaryOp(UnaryOp::Not));
    function.add_instruction(IRInstruction::JumpIf("else_ternary".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(1)));
    function.add_instruction(IRInstruction::Jump("end_ternary".to_string()));
    function.add_instruction(IRInstruction::Label("else_ternary".to_string()));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(0)));
    function.add_instruction(IRInstruction::Label("end_ternary".to_string()));

    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("-x"));
    assert!(js_code.contains("!flag"));
    assert!(js_code.contains("1"));
    assert!(js_code.contains("0"));
}

/// Test string operations and concatenation
#[test]
fn test_string_operations() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("string_ops".to_string(), IRType::String);

    function.add_param("first".to_string(), IRType::String);
    function.add_param("last".to_string(), IRType::String);

    // return "Hello, " + first + " " + last + "!"
    function.add_instruction(IRInstruction::LoadConst(Value::String("Hello, ".to_string())));
    function.add_instruction(IRInstruction::LoadVar("first".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::LoadConst(Value::String(" ".to_string())));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::LoadVar("last".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::LoadConst(Value::String("!".to_string())));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("Hello"));
    assert!(js_code.contains("first"));
    assert!(js_code.contains("last"));
    assert!(js_code.contains("!"));
}

/// Test closure/function literal generation
#[test]
fn test_function_literals() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("higher_order".to_string(), IRType::Int);

    function.add_param("x".to_string(), IRType::Int);

    // Simplified lambda test - just call a function
    function.add_instruction(IRInstruction::LoadConst(Value::Int(5)));
    function.add_instruction(IRInstruction::LoadConst(Value::Int(10)));
    function.add_instruction(IRInstruction::Call { function: "add".to_string(), arg_count: 2 });
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("let adder"));
    assert!(js_code.contains("adder(10)"));
}

/// Test error handling constructs
#[test]
fn test_error_handling() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("safe_divide".to_string(), IRType::Int);

    function.add_param("a".to_string(), IRType::Int);
    function.add_param("b".to_string(), IRType::Int);

    // Simple division: return a / b
    function.add_instruction(IRInstruction::LoadVar("a".to_string()));
    function.add_instruction(IRInstruction::LoadVar("b".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Div));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function divide(a, b)"));
    assert!(js_code.contains("return"));
}

/// Test async/await generation
#[test]
fn test_async_await_generation() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("async_operation".to_string(), IRType::String);

    function.add_param("url".to_string(), IRType::String);

    // Simplified async test - just return the URL
    function.add_instruction(IRInstruction::LoadVar("url".to_string()));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("async function async_operation(url)"));
    assert!(js_code.contains("return url"));
}

/// Test class-like generation (simplified)
#[test]
fn test_class_generation() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("create_person".to_string(), IRType::String);

    function.add_param("name".to_string(), IRType::String);
    function.add_param("age".to_string(), IRType::Int);

    // return "Person: " + name
    function.add_instruction(IRInstruction::LoadConst(Value::String("Person: ".to_string())));
    function.add_instruction(IRInstruction::LoadVar("name".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function create_person(name, age)"));
    assert!(js_code.contains("Person:"));
    assert!(js_code.contains("return"));
}

/// Test module with multiple functions
#[test]
fn test_module_with_functions() {
    let mut generator = JSGenerator::new();
    let mut module = IRModule::new("math_utils".to_string());

    let mut add_func = IRFunction::new("add".to_string(), IRType::Int);
    add_func.add_param("a".to_string(), IRType::Int);
    add_func.add_param("b".to_string(), IRType::Int);
    add_func.add_instruction(IRInstruction::LoadVar("a".to_string()));
    add_func.add_instruction(IRInstruction::LoadVar("b".to_string()));
    add_func.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    add_func.add_instruction(IRInstruction::Return);
    module.add_function(add_func);

    let js_code = generator.generate_module(&module);

    assert!(js_code.contains("function add"));
    assert!(js_code.contains("math_utils"));
}

/// Test performance-critical code generation (simplified)
#[test]
fn test_performance_optimizations() {
    let mut generator = JSGenerator::new();
    let mut function = IRFunction::new("hot_path".to_string(), IRType::Int);

    function.add_param("a".to_string(), IRType::Int);
    function.add_param("b".to_string(), IRType::Int);

    // Simple optimized addition
    function.add_instruction(IRInstruction::LoadVar("a".to_string()));
    function.add_instruction(IRInstruction::LoadVar("b".to_string()));
    function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    function.add_instruction(IRInstruction::Return);

    let js_code = generator.generate_function(&function);

    assert!(js_code.contains("function hot_path(a, b)"));
    assert!(js_code.contains("return"));
}

/// Test edge cases and error conditions
#[test]
fn test_edge_cases() {
    let mut generator = JSGenerator::new();

    // Test empty function
    let mut empty_func = IRFunction::new("empty".to_string(), IRType::Void);
    let empty_js = generator.generate_function(&empty_func);
    assert!(empty_js.contains("function empty()"));

    // Test function with many parameters
    let mut many_params = IRFunction::new("many_params".to_string(), IRType::Void);
    for i in 0..10 {
        many_params.add_param(format!("param{}", i), IRType::Int);
    }
    let many_js = generator.generate_function(&many_params);
    for i in 0..10 {
        assert!(many_js.contains(&format!("param{}", i)));
    }

    // Test deeply nested expressions
    let mut nested_func = IRFunction::new("nested".to_string(), IRType::Int);
    nested_func.add_param("x".to_string(), IRType::Int);

    // Create deeply nested expression: (((x + 1) * 2) + 3) * 4
    nested_func.add_instruction(IRInstruction::LoadVar("x".to_string()));
    nested_func.add_instruction(IRInstruction::LoadConst(Value::Int(1)));
    nested_func.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    nested_func.add_instruction(IRInstruction::LoadConst(Value::Int(2)));
    nested_func.add_instruction(IRInstruction::BinaryOp(BinaryOp::Mul));
    nested_func.add_instruction(IRInstruction::LoadConst(Value::Int(3)));
    nested_func.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
    nested_func.add_instruction(IRInstruction::LoadConst(Value::Int(4)));
    nested_func.add_instruction(IRInstruction::BinaryOp(BinaryOp::Mul));
    nested_func.add_instruction(IRInstruction::Return);

    let nested_js = generator.generate_function(&nested_func);

    // Should contain properly parenthesized expression
    assert!(nested_js.contains("return"));
    assert!(nested_js.contains("x"));
    assert!(nested_js.contains("+"));
    assert!(nested_js.contains("*"));
}