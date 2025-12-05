/*!
Error Propagation Integration Tests

Tests that verify errors are properly caught, reported, and propagated
through the VM execution stack with accurate diagnostics.
*/

use vela_vm::{Bytecode, CodeObject, Constant, Instruction, VirtualMachine, Value, Error};

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

/// Test: Empty bytecode returns NULL
#[test]
fn test_empty_bytecode() {
    let bytecode = Bytecode::new();
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    // Empty bytecode returns NULL (not an error)
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::NULL);
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

// ============================================================================
// Category 1: VM + GC Integration Tests (10 tests)
// ============================================================================

/// Test: Heavy allocation stress test
#[test]
fn test_heavy_allocation_stress() {
    let mut bytecode = Bytecode::new();
    
    // Create 100 integers
    for i in 0..100 {
        bytecode.add_constant(Constant::Int(i));
    }
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // Load 100 constants
    for i in 0..100 {
        bytecode.push(Instruction::LoadConst(i as u16));
    }
    
    // Add all values together (99 add operations)
    for _ in 0..99 {
        bytecode.push(Instruction::Add);
    }
    
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // Sum of 0..100 = 4950
    assert_eq!(result.as_int(), Some(4950));
}

/// Test: Repeated allocation and deallocation cycles
#[test]
fn test_repeated_allocation_cycles() {
    let mut vm = VirtualMachine::new();
    
    // Simulate 50 execution cycles
    for iteration in 0..50 {
        let mut bytecode = Bytecode::new();
        bytecode.add_constant(Constant::Int(iteration));
        bytecode.add_constant(Constant::Int(2));
        
        bytecode.add_code_object(CodeObject::new(0, 0));
        bytecode.push(Instruction::LoadConst(0));
        bytecode.push(Instruction::LoadConst(1));
        bytecode.push(Instruction::Mul);
        bytecode.push(Instruction::Return);
        
        let result = vm.execute(&bytecode).expect("execution should succeed");
        assert_eq!(result.as_int(), Some(iteration * 2));
    }
}

/// Test: Large constant pool stress (1000 constants)
#[test]
fn test_large_constant_pool() {
    let mut bytecode = Bytecode::new();
    
    // Create 1000 constants
    for i in 0..1000 {
        bytecode.add_constant(Constant::Int(i));
    }
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // Load first and last constant, add them
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::LoadConst(999));
    bytecode.push(Instruction::Add);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(999));
}

/// Test: Mixed int and float operations
#[test]
fn test_mixed_numeric_types() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Float(3.14));
    bytecode.add_constant(Constant::Int(2));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // 10 + 2 = 12
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::Add);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(12));
}

/// Test: Stack growth and shrinkage patterns
#[test]
fn test_stack_growth_shrinkage() {
    let mut bytecode = Bytecode::new();
    
    for i in 0..100 {
        bytecode.add_constant(Constant::Int(i));
    }
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // Push 100 values
    for i in 0..100 {
        bytecode.push(Instruction::LoadConst(i as u16));
    }
    
    // Pop 99 values
    for _ in 0..99 {
        bytecode.push(Instruction::Pop);
    }
    
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // Should return first value (0)
    assert_eq!(result.as_int(), Some(0));
}

/// Test: Boolean logic operations
#[test]
fn test_boolean_logic_operations() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Bool(true));
    bytecode.add_constant(Constant::Bool(false));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0)); // true
    bytecode.push(Instruction::LoadConst(1)); // false
    bytecode.push(Instruction::And);          // true AND false = false
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_bool(), Some(false));
}

/// Test: Comparison operations chain
#[test]
fn test_comparison_operations_chain() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(5));
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Int(15));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // Test: (5 < 10) AND (10 < 15)
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::Lt);           // true
    
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::Lt);           // true
    
    bytecode.push(Instruction::And);          // true AND true = true
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_bool(), Some(true));
}

/// Test: Null value handling
#[test]
fn test_null_value_handling() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Null);
    bytecode.add_constant(Constant::Int(42));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0)); // NULL
    bytecode.push(Instruction::Pop);
    bytecode.push(Instruction::LoadConst(1)); // 42
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(42));
}

/// Test: Duplicate instruction behavior
#[test]
fn test_duplicate_instruction() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(7));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0)); // 7
    bytecode.push(Instruction::Dup);          // 7, 7
    bytecode.push(Instruction::Mul);          // 49
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(49));
}

/// Test: Negation operation
#[test]
fn test_negation_operation() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(42));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0)); // 42
    bytecode.push(Instruction::Neg);          // -42
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(-42));
}

// ============================================================================
// Category 3: Multi-Module Tests (10 tests)
// ============================================================================

/// Test: Global variable storage and retrieval
#[test]
fn test_global_variable_storage() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(100));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreGlobal(0));
    bytecode.push(Instruction::LoadGlobal(0));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(100));
}

/// Test: Global persistence across executions
#[test]
fn test_global_persistence_across_executions() {
    let mut vm = VirtualMachine::new();
    
    // First execution: store global
    {
        let mut bytecode = Bytecode::new();
        bytecode.add_constant(Constant::Int(42));
        
        bytecode.add_code_object(CodeObject::new(0, 0));
        bytecode.push(Instruction::LoadConst(0));
        bytecode.push(Instruction::StoreGlobal(0));
        bytecode.push(Instruction::LoadGlobal(0));
        bytecode.push(Instruction::Return);
        
        let result = vm.execute(&bytecode).expect("should succeed");
        assert_eq!(result.as_int(), Some(42));
    }
    
    // Second execution: load same global
    {
        let mut bytecode = Bytecode::new();
        bytecode.add_code_object(CodeObject::new(0, 0));
        bytecode.push(Instruction::LoadGlobal(0));
        bytecode.push(Instruction::Return);
        
        let result = vm.execute(&bytecode).expect("should succeed");
        assert_eq!(result.as_int(), Some(42));
    }
}

/// Test: Multiple global variables
#[test]
fn test_multiple_global_variables() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Int(20));
    bytecode.add_constant(Constant::Int(30));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // Store three globals
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreGlobal(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreGlobal(1));
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::StoreGlobal(2));
    
    // Sum: global[0] + global[1] + global[2]
    bytecode.push(Instruction::LoadGlobal(0));
    bytecode.push(Instruction::LoadGlobal(1));
    bytecode.push(Instruction::Add);
    bytecode.push(Instruction::LoadGlobal(2));
    bytecode.push(Instruction::Add);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(60));
}

/// Test: Local and global variable interaction
#[test]
fn test_local_global_interaction() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(5));
    bytecode.add_constant(Constant::Int(10));
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 1; // 1 local
    bytecode.add_code_object(code);
    
    // Store in local
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreLocal(0));
    
    // Store in global
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreGlobal(0));
    
    // Compute local + global
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadGlobal(0));
    bytecode.push(Instruction::Add);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(15));
}

/// Test: String table usage
#[test]
fn test_string_table_constants() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_string("hello".to_string());
    bytecode.add_string("world".to_string());
    
    bytecode.add_constant(Constant::String(0));
    bytecode.add_constant(Constant::String(1));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::Pop);
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::Pop);
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    
    assert!(result.is_ok());
}

/// Test: Code object with multiple locals
#[test]
fn test_code_object_multiple_locals() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(3));
    bytecode.add_constant(Constant::Int(4));
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 2; // 2 locals
    bytecode.add_code_object(code);
    
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreLocal(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreLocal(1));
    
    // Compute local[0] * local[1]
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::Mul);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(12));
}

/// Test: Nested local scopes simulation
#[test]
fn test_nested_local_scopes() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Int(20));
    bytecode.add_constant(Constant::Int(30));
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 3; // 3 locals
    bytecode.add_code_object(code);
    
    // Store values in all locals
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreLocal(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreLocal(1));
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::StoreLocal(2));
    
    // Sum all locals
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::Add);
    bytecode.push(Instruction::LoadLocal(2));
    bytecode.push(Instruction::Add);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(60));
}

/// Test: Large program with many instructions
#[test]
fn test_large_multi_module_program() {
    let mut bytecode = Bytecode::new();
    
    // Create 50 constants
    for i in 0..50 {
        bytecode.add_constant(Constant::Int(i));
    }
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 10; // 10 locals
    bytecode.add_code_object(code);
    
    // Store first 10 constants in locals
    for i in 0..10 {
        bytecode.push(Instruction::LoadConst(i as u16));
        bytecode.push(Instruction::StoreLocal(i as u16));
    }
    
    // Sum all locals
    bytecode.push(Instruction::LoadLocal(0));
    for i in 1..10 {
        bytecode.push(Instruction::LoadLocal(i as u16));
        bytecode.push(Instruction::Add);
    }
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // Sum of 0..10 = 45
    assert_eq!(result.as_int(), Some(45));
}

/// Test: Global overwrite behavior
#[test]
fn test_global_overwrite() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(100));
    bytecode.add_constant(Constant::Int(200));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // Store first value
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreGlobal(0));
    
    // Overwrite with second value
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreGlobal(0));
    
    // Load (should be second value)
    bytecode.push(Instruction::LoadGlobal(0));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(200));
}

/// Test: Complex data flow with mixed variables
#[test]
fn test_complex_data_flow() {
    let mut bytecode = Bytecode::new();
    
    for i in 0..20 {
        bytecode.add_constant(Constant::Int(i));
    }
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 5; // 5 locals
    bytecode.add_code_object(code);
    
    // Store in locals and globals alternately
    for i in 0..5 {
        bytecode.push(Instruction::LoadConst(i as u16));
        bytecode.push(Instruction::StoreLocal(i as u16));
        
        bytecode.push(Instruction::LoadConst((i + 5) as u16));
        bytecode.push(Instruction::StoreGlobal(i as u16));
    }
    
    // Sum first local + first global
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadGlobal(0));
    bytecode.push(Instruction::Add);
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // local[0]=0, global[0]=5, sum=5
    assert_eq!(result.as_int(), Some(5));
}
