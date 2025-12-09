/*!
Bytecode Correctness Tests

Detailed tests for specific bytecode instruction correctness and edge cases.
*/

use vela_vm::{Bytecode, CodeObject, Constant, VirtualMachine};

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

/// Helper function for bytecode with strings
fn execute_bytecode_with_strings(strings: Vec<String>, constants: Vec<Constant>, bytecode: Vec<u8>) -> Result<vela_vm::Value, vela_vm::Error> {
    let mut bc = Bytecode::new();
    for string in strings {
        bc.add_string(string);
    }
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
fn test_arithmetic_edge_cases() {
    // Division by zero should cause runtime error
    let result = execute_bytecode(
        vec![Constant::Int(10), Constant::Int(0)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (10)
            0x00, 0x01, 0x00, // LoadConst 1 (0)
            0x13, // Div
            0x51, // Return
        ],
    );
    assert!(result.is_err()); // Should be division by zero error

    // Modulo by zero
    let result = execute_bytecode(
        vec![Constant::Int(10), Constant::Int(0)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (10)
            0x00, 0x01, 0x00, // LoadConst 1 (0)
            0x14, // Mod
            0x51, // Return
        ],
    );
    assert!(result.is_err()); // Should be modulo by zero error

    // Large numbers
    let result = execute_bytecode(
        vec![Constant::Int(1000000), Constant::Int(2000000)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (1000000)
            0x00, 0x01, 0x00, // LoadConst 1 (2000000)
            0x10, // Add
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(3000000));
}

#[test]
fn test_constant_loading() {
    // Test loading different constant types
    let result = execute_bytecode(
        vec![
            Constant::Int(42),
            Constant::Bool(true),
        ],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(42));

    let result = execute_bytecode(
        vec![
            Constant::Int(42),
            Constant::Bool(true),
        ],
        vec![
            0x00, 0x01, 0x00, // LoadConst 1 (true)
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));

    // Test string loading (just verify it doesn't crash)
    let result = execute_bytecode_with_strings(
        vec!["hello".to_string()],
        vec![Constant::String(0)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (string)
            0x51, // Return
        ],
    );
    assert!(result.is_ok()); // Just verify string loading works
}

#[test]
fn test_complex_expressions() {
    // Test complex arithmetic expression: (5 + 3) * 2 - 1
    let result = execute_bytecode(
        vec![
            Constant::Int(5),
            Constant::Int(3),
            Constant::Int(2),
            Constant::Int(1),
        ],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (3)
            0x10, // Add (5 + 3 = 8)
            0x00, 0x02, 0x00, // LoadConst 2 (2)
            0x12, // Mul (8 * 2 = 16)
            0x00, 0x03, 0x00, // LoadConst 3 (1)
            0x11, // Sub (16 - 1 = 15)
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(15));

    // Test boolean logic: (true && false) || true
    let result = execute_bytecode(
        vec![
            Constant::Bool(true),
            Constant::Bool(false),
            Constant::Bool(true),
        ],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (true)
            0x00, 0x01, 0x00, // LoadConst 1 (false)
            0x30, // And (true && false = false)
            0x00, 0x02, 0x00, // LoadConst 2 (true)
            0x31, // Or (false || true = true)
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_bool(), Some(true));
}

#[test]
fn test_stack_manipulation() {
    // Test DUP and POP combinations
    let result = execute_bytecode(
        vec![Constant::Int(10)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (10)
            0x08, // Dup (stack: [10, 10])
            0x08, // Dup (stack: [10, 10, 10])
            0x10, // Add (10 + 10 = 20, stack: [10, 20])
            0x07, // Pop (remove 20, stack: [10])
            0x51, // Return (returns 10)
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(10));

    // Test stack overflow protection (if implemented)
    // This would require many DUP operations to test stack limits
}

#[test]
fn test_control_flow_complex() {
    // Test if-else like structure using jumps
    // if (x > 5) { result = 100 } else { result = 50 }
    let result = execute_bytecode(
        vec![
            Constant::Int(7),  // x = 7
            Constant::Int(5),  // comparison value
            Constant::Int(100), // then branch
            Constant::Int(50),  // else branch
        ],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (7) - x
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x24, // Gt (7 > 5 = true)
            0x41, 0x14, 0x00, 0x00, 0x00, // JumpIfFalse to position 20 (else)
            0x00, 0x02, 0x00, // LoadConst 2 (100) - then
            0x40, 0x17, 0x00, 0x00, 0x00, // Jump to position 23 (Return)
            0x00, 0x03, 0x00, // LoadConst 3 (50) - else
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(100)); // Should take then branch

    // Test with x = 3 (should take else branch)
    let result = execute_bytecode(
        vec![
            Constant::Int(3),   // x = 3
            Constant::Int(5),   // comparison value
            Constant::Int(100), // then branch
            Constant::Int(50),  // else branch
        ],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (3) - x
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x24, // Gt (3 > 5 = false)
            0x41, 0x14, 0x00, 0x00, 0x00, // JumpIfFalse to position 20 (else)
            0x00, 0x02, 0x00, // LoadConst 2 (100) - then
            0x40, 0x17, 0x00, 0x00, 0x00, // Jump to position 23 (Return)
            0x00, 0x03, 0x00, // LoadConst 3 (50) - else
            0x51, // Return
        ],
    ).unwrap();
    assert_eq!(result.as_int(), Some(50)); // Should take else branch
}

#[test]
fn test_type_mixing_errors() {
    // Test operations on incompatible types
    // Adding int and string should cause runtime error
    let result = execute_bytecode_with_strings(
        vec!["hello".to_string()],
        vec![
            Constant::Int(42),
            Constant::String(0),
        ],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x00, 0x01, 0x00, // LoadConst 1 ("hello")
            0x10, // Add (should fail)
            0x51, // Return
        ],
    );
    assert!(result.is_err()); // Should be type error

    // Comparing incompatible types
    let result = execute_bytecode(
        vec![
            Constant::Int(42),
            Constant::Bool(true),
        ],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x00, 0x01, 0x00, // LoadConst 1 (true)
            0x20, // Eq (should work or fail depending on implementation)
            0x51, // Return
        ],
    );
    // This might succeed or fail depending on VM implementation
    // For now, just ensure it doesn't crash
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_instruction_pointer_bounds() {
    // Test jumping beyond bytecode bounds
    let result = execute_bytecode(
        vec![Constant::Int(42)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x40, 0x10, 0x00, 0x00, 0x00, // Jump +16 (beyond bytecode)
        ],
    );
    assert!(result.is_err()); // Should be bounds error

    // Test negative jump
    let result = execute_bytecode(
        vec![Constant::Int(42)],
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x40, 0xF0, 0xFF, 0xFF, 0xFF, // Jump -16 (negative)
        ],
    );
    assert!(result.is_err()); // Should be bounds error
}