/*!
Integration tests for VelaVM

Tests complete bytecode programs execution end-to-end.
*/

use vela_vm::{Bytecode, CodeObject, Constant, Error, VirtualMachine};

/// Helper to create a simple program
fn create_program(bytecode_ops: Vec<u8>, constants: Vec<Constant>) -> Bytecode {
    let mut bytecode = Bytecode::new();

    // Add constants
    for constant in constants {
        bytecode.add_constant(constant);
    }

    // Create code object
    let mut code = CodeObject::new(0, 0);
    code.bytecode = bytecode_ops;
    bytecode.add_code_object(code);

    bytecode
}

#[test]
fn test_simple_arithmetic() {
    // Program: 5 + 3 = 8
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (3)
            0x10, // Add
            0x51, // Return
        ],
        vec![Constant::Int(5), Constant::Int(3)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(8));
}

#[test]
fn test_complex_arithmetic() {
    // Program: (10 + 5) * 2 - 3 = 27
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (10)
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x10, // Add (15)
            0x00, 0x02, 0x00, // LoadConst 2 (2)
            0x12, // Mul (30)
            0x00, 0x03, 0x00, // LoadConst 3 (3)
            0x11, // Sub (27)
            0x51, // Return
        ],
        vec![
            Constant::Int(10),
            Constant::Int(5),
            Constant::Int(2),
            Constant::Int(3),
        ],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(27));
}

#[test]
fn test_comparison_operations() {
    // Program: 5 > 3 = true
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (3)
            0x24, // Gt (greater than)
            0x51, // Return
        ],
        vec![Constant::Int(5), Constant::Int(3)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_bool(), Some(true));
}

#[test]
fn test_logical_operations() {
    // Program: true && false = false
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (true)
            0x00, 0x01, 0x00, // LoadConst 1 (false)
            0x30, // And
            0x51, // Return
        ],
        vec![Constant::Bool(true), Constant::Bool(false)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_bool(), Some(false));
}

#[test]
fn test_conditional_jump() {
    // Program: if false { return 10 } else { return 20 }
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (false) - Position 0-2
            0x41, 0x0C, 0x00, 0x00, 0x00, // JumpIfFalse to position 12 - Position 3-7
            0x00, 0x01, 0x00, // LoadConst 1 (10) - Position 8-10
            0x51, // Return - Position 11
            0x00, 0x02, 0x00, // LoadConst 2 (20) - Position 12-14 (jump target)
            0x51, // Return - Position 15
        ],
        vec![Constant::Bool(false), Constant::Int(10), Constant::Int(20)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(20));
}

#[test]
fn test_stack_operations() {
    // Program: DUP + POP
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x08, // Dup (stack: [5, 5])
            0x10, // Add (stack: [10])
            0x51, // Return
        ],
        vec![Constant::Int(5)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(10));
}

#[test]
fn test_local_variables() {
    // Program: local = 42; return local
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(42));

    let mut code = CodeObject::new(0, 0); // name=0, filename=0
    code.local_count = 1; // 1 local variable
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (42)
        0x02, 0x00, 0x00, // StoreLocal 0
        0x01, 0x00, 0x00, // LoadLocal 0
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_global_variables() {
    // Program: global = 100; return global
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (100)
            0x04, 0x00, 0x00, // StoreGlobal 0
            0x03, 0x00, 0x00, // LoadGlobal 0
            0x51, // Return
        ],
        vec![Constant::Int(100)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(100));
}

#[test]
fn test_division_by_zero() {
    // Program: 10 / 0
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (10)
            0x00, 0x01, 0x00, // LoadConst 1 (0)
            0x13, // Div
            0x51, // Return
        ],
        vec![Constant::Int(10), Constant::Int(0)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program);

    assert!(matches!(result, Err(Error::DivisionByZero)));
}

#[test]
fn test_stack_underflow() {
    // Program: Pop from empty stack
    let program = create_program(
        vec![
            0x07, // Pop (stack is empty!)
            0x51, // Return
        ],
        vec![],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program);

    assert!(matches!(result, Err(Error::StackUnderflow)));
}

#[test]
fn test_negation() {
    // Program: -(5 + 3) = -8
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (3)
            0x10, // Add (8)
            0x16, // Neg (-8)
            0x51, // Return
        ],
        vec![Constant::Int(5), Constant::Int(3)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(-8));
}

#[test]
fn test_power_operation() {
    // Program: 2 ** 3 = 8
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (2)
            0x00, 0x01, 0x00, // LoadConst 1 (3)
            0x15, // Pow
            0x51, // Return
        ],
        vec![Constant::Int(2), Constant::Int(3)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(8));
}

#[test]
fn test_modulo_operation() {
    // Program: 17 % 5 = 2
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (17)
            0x00, 0x01, 0x00, // LoadConst 1 (5)
            0x14, // Mod
            0x51, // Return
        ],
        vec![Constant::Int(17), Constant::Int(5)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(2));
}

#[test]
fn test_equality_operations() {
    // Program: 42 == 42 = true
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x00, 0x00, 0x00, // LoadConst 0 (42)
            0x20, // Eq
            0x51, // Return
        ],
        vec![Constant::Int(42)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_bool(), Some(true));
}

#[test]
fn test_not_equal() {
    // Program: 5 != 3 = true
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (5)
            0x00, 0x01, 0x00, // LoadConst 1 (3)
            0x21, // Ne
            0x51, // Return
        ],
        vec![Constant::Int(5), Constant::Int(3)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_bool(), Some(true));
}

#[test]
fn test_or_operation() {
    // Program: false || true = true
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (false)
            0x00, 0x01, 0x00, // LoadConst 1 (true)
            0x31, // Or
            0x51, // Return
        ],
        vec![Constant::Bool(false), Constant::Bool(true)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_bool(), Some(true));
}

#[test]
fn test_not_operation() {
    // Program: !false = true
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (false)
            0x32, // Not
            0x51, // Return
        ],
        vec![Constant::Bool(false)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_bool(), Some(true));
}

#[test]
fn test_unconditional_jump() {
    // Program: jump over LoadConst(10), return LoadConst(20)
    let program = create_program(
        vec![
            0x40, 0x08, 0x00, 0x00, 0x00, // Jump to position 8 (LoadConst 1)
            0x00, 0x00, 0x00, // LoadConst 0 (10) - SKIPPED
            0x00, 0x01, 0x00, // LoadConst 1 (20) - Position 8
            0x51, // Return
        ],
        vec![Constant::Int(10), Constant::Int(20)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    assert_eq!(result.as_int(), Some(20));
}

#[test]
fn test_empty_program() {
    // Program with only return
    let mut bytecode = Bytecode::new();
    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![0x51]; // Return
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();

    assert!(result.is_null());
}

#[test]
fn test_float_arithmetic() {
    // Program: 3.14 + 2.86 = 6.0
    let program = create_program(
        vec![
            0x00, 0x00, 0x00, // LoadConst 0 (3.14)
            0x00, 0x01, 0x00, // LoadConst 1 (2.86)
            0x10, // Add
            0x51, // Return
        ],
        vec![Constant::Float(3.14), Constant::Float(2.86)],
    );

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&program).unwrap();

    if let Some(f) = result.as_float() {
        assert!((f - 6.0).abs() < 0.001);
    } else {
        panic!("Expected float result");
    }
}
