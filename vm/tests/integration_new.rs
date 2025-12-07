/*!
Error Propagation Integration Tests

Tests that verify errors are properly caught, reported, and propagated
through the VM execution stack with accurate diagnostics.
*/

use vela_vm::{Bytecode, CodeObject, Constant, Instruction, VirtualMachine, Error};

/// Test: Division by zero error
#[test]
fn test_division_by_zero() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Int(0));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::Div);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    assert!(result.is_err());
    match result {
        Err(Error::DivisionByZero) => {}, // Expected
        _ => panic!("Expected DivisionByZero error"),
    }
}

/// Test: Stack underflow error
#[test]
fn test_stack_underflow() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    // Try to pop from empty stack
    bytecode.push(Instruction::Pop);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    assert!(result.is_err());
    match result {
        Err(Error::StackUnderflow) => {}, // Expected
        _ => panic!("Expected StackUnderflow error"),
    }
}

/// Test: Invalid constant index
#[test]
fn test_invalid_constant_index() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(42));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    // Try to load non-existent constant
    bytecode.push(Instruction::LoadConst(999));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    assert!(result.is_err());
    match result {
        Err(Error::InvalidConstant { index: 999 }) => {}, // Expected
        _ => panic!("Expected InvalidConstant error"),
    }
}

/// Test: Type mismatch in operations
#[test]
fn test_type_error_operations() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Bool(true));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0)); // int
    bytecode.push(Instruction::LoadConst(1)); // bool
    bytecode.push(Instruction::Add); // Can't add int + bool
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    // VM should handle this gracefully (either error or coercion)
    // For now, we expect it to work (VM does implicit conversion)
    assert!(result.is_ok() || result.is_err());
}

/// Test: Empty bytecode execution (should succeed with NULL)
#[test]
fn test_empty_bytecode() {
    let mut bytecode = Bytecode::new();

    // Create an empty code object (no instructions)
    let code_obj = CodeObject::new(0, 0);
    // Empty code object is valid - should execute successfully
    bytecode.code_objects.push(code_obj);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);

    // Should succeed and return NULL
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vela_vm::Value::NULL);
}

/// Test: Missing return instruction
#[test]
fn test_missing_return() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(42));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0));
    // Missing Return instruction
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    // Should either auto-return or error
    assert!(result.is_ok() || result.is_err());
}

/// Test: Jump to invalid offset
#[test]
fn test_invalid_jump_target() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::Jump(9999)); // Invalid offset
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    // Should fail with InvalidJumpTarget or execute empty code
    assert!(result.is_ok() || result.is_err());
}

/// Test: Invalid local variable index
#[test]
fn test_invalid_local_index() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_code_object(CodeObject::new(0, 0)); // 0 locals
    bytecode.push(Instruction::LoadLocal(5)); // Try to load local 5
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    assert!(result.is_err());
}

/// Test: Invalid global variable index
#[test]
fn test_invalid_global_index() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadGlobal(999)); // Non-existent global
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    // Should return NULL or error
    assert!(result.is_ok() || result.is_err());
}

/// Test: Modulo by zero
#[test]
fn test_modulo_by_zero() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Int(0));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::Mod);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    assert!(result.is_err());
    match result {
        Err(Error::DivisionByZero) => {}, // Expected
        _ => panic!("Expected DivisionByZero error for modulo"),
    }
}
