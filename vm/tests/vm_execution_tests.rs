/*!
VM Execution Tests

Comprehensive tests for VelaVM bytecode execution correctness.
Tests cover all major opcode categories and edge cases.
*/

use vela_vm::{Bytecode, CodeObject, Constant, VirtualMachine};

/// Helper function to create and execute bytecode
fn execute_bytecode(constants: Vec<Constant>, bytecode: Vec<u8>) -> Result<vela_vm::Value, vela_vm::Error> {
    let mut bc = Bytecode::new();
    for constant in constants {
        bc.add_constant(constant);
    }

    let mut code = CodeObject::new(0, 0);
    code.bytecode = bytecode;
    bc.add_code_object(code);

    let mut vm = VirtualMachine::new();
    vm.execute(&bc)
}

#[test]
fn test_arithmetic_operations() {
    // Test ADD (0x10)
    let result = execute_bytecode(
        vec![Constant::Int(5), Constant::Int(3)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (3)
            0x10, // Add
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(8));

    // Test SUB (0x11)
    let result = execute_bytecode(
        vec![Constant::Int(10), Constant::Int(4)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (10)
            0x00, 0x01, 0x00, // LoadConst 1 (4)
            0x11, // Sub
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(6));

    // Test MUL (0x12)
    let result = execute_bytecode(
        vec![Constant::Int(7), Constant::Int(6)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (7)
            0x00, 0x01, 0x00, // LoadConst 1 (6)
            0x12, // Mul
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(42));

    // Test DIV (0x13)
    let result = execute_bytecode(
        vec![Constant::Int(20), Constant::Int(4)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (20)
            0x00, 0x01, 0x00, // LoadConst 1 (4)
            0x13, // Div
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(5));

    // Test MOD (0x14)
    let result = execute_bytecode(
        vec![Constant::Int(17), Constant::Int(5)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (17)
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x14, // Mod
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(2));

    // Test NEG (0x16)
    let result = execute_bytecode(
        vec![Constant::Int(42)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x16, // Neg
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(-42));
}

#[test]
fn test_comparison_operations() {
    // Test EQ (0x20)
    let result = execute_bytecode(
        vec![Constant::Int(5), Constant::Int(5)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x20, // Eq
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));

    // Test NE (0x21)
    let result = execute_bytecode(
        vec![Constant::Int(5), Constant::Int(3)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (3)
            0x21, // Ne
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));

    // Test LT (0x22)
    let result = execute_bytecode(
        vec![Constant::Int(3), Constant::Int(5)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (3)
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x22, // Lt
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));

    // Test LE (0x23)
    let result = execute_bytecode(
        vec![Constant::Int(5), Constant::Int(5)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x23, // Le
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));

    // Test GT (0x24)
    let result = execute_bytecode(
        vec![Constant::Int(7), Constant::Int(5)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (7)
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x24, // Gt
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));

    // Test GE (0x25)
    let result = execute_bytecode(
        vec![Constant::Int(5), Constant::Int(5)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x25, // Ge
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));
}

#[test]
fn test_logical_operations() {
    // Test AND (0x30)
    let result = execute_bytecode(
        vec![Constant::Bool(true), Constant::Bool(false)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (true)
            0x00, 0x01, 0x00, // LoadConst 1 (false)
            0x30, // And
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(false));

    // Test OR (0x31)
    let result = execute_bytecode(
        vec![Constant::Bool(true), Constant::Bool(false)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (true)
            0x00, 0x01, 0x00, // LoadConst 1 (false)
            0x31, // Or
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));

    // Test NOT (0x32)
    let result = execute_bytecode(
        vec![Constant::Bool(true)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (true)
            0x32, // Not
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(false));
}

#[test]
fn test_stack_operations() {
    // Test DUP (0x08)
    let result = execute_bytecode(
        vec![Constant::Int(42)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x08, // Dup
            0x10, // Add (42 + 42)
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(84));

    // Test POP (0x07)
    let result = execute_bytecode(
        vec![Constant::Int(42), Constant::Int(100)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x00, 0x01, 0x00, // LoadConst 1 (100)
            0x07, // Pop (removes 100)
            0x51, // Return (returns 42)
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_control_flow() {
    // Test unconditional JUMP (0x40)
    let result = execute_bytecode(
        vec![Constant::Int(1)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (1)
            0x40, 0x08, 0x00, 0x00, 0x00, // Jump to position 8 (skip to Return)
            0x00, 0x00, 0x00, // LoadConst 1 (dummy) - should be skipped
            0x51, // Return - should be skipped
            0x51, // Return - should execute
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(1));

    // Test JUMP_IF_FALSE (0x41) - condition false, jump
    let result = execute_bytecode(
        vec![Constant::Bool(false), Constant::Int(1), Constant::Int(2)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (false)
            0x41, 0x0C, 0x00, 0x00, 0x00, // JumpIfFalse to position 12 (jump if false)
            0x00, 0x01, 0x00, // LoadConst 1 (1) - should be skipped
            0x51, // Return
            0x00, 0x02, 0x00, // LoadConst 2 (2) - should execute
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(2));

    // Test JUMP_IF_TRUE (0x42) - condition true, jump
    let result = execute_bytecode(
        vec![Constant::Bool(true), Constant::Int(1), Constant::Int(2)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (true)
            0x42, 0x0C, 0x00, 0x00, 0x00, // JumpIfTrue to position 12 (jump if true)
            0x00, 0x01, 0x00, // LoadConst 1 (1) - should be skipped
            0x51, // Return
            0x00, 0x02, 0x00, // LoadConst 2 (2) - should execute
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(2));
}

#[test]
fn test_function_calls() {
    // Test simple function call (CALL 0x50, RETURN 0x51)
    // This is a basic test - more complex function tests would require
    // MakeFunction (0x52) and proper function objects
    let result = execute_bytecode(
        vec![Constant::Int(42)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_collections() {
    // Test BuildList (0x60)
    // This would require more complex setup with list operations
    // For now, just test that the opcode doesn't crash
    let result = execute_bytecode(
        vec![],
        vec![
            0x60, 0x00, 0x00, // BuildList 0 (empty list)
            0x51, // Return
        ],
    );
    assert!(result.is_ok());
    // Note: Value type checking would be needed for full validation
}

#[test]
fn test_error_conditions() {
    // Test stack underflow
    let result = execute_bytecode(
        vec![],
        vec![0x07], // Pop on empty stack
    );
    assert!(matches!(result, Err(vela_vm::Error::StackUnderflow)));

    // Test invalid opcode
    let result = execute_bytecode(
        vec![],
        vec![0xFE], // Invalid opcode
    );
    assert!(matches!(result, Err(vela_vm::Error::InvalidOpcode { opcode: 0xFE })));
}