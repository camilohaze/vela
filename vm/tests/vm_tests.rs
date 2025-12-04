/*!
VirtualMachine tests

Tests for VM execution, call frames, and error handling.
*/

use vela_vm::{Bytecode, CallFrame, CodeObject, Constant, Error, VirtualMachine};

#[test]
fn test_vm_initialization() {
    let _vm = VirtualMachine::new();
    // VM initialized successfully (no panic)
    assert!(true);
}

#[test]
fn test_vm_empty_execution() {
    let mut bytecode = Bytecode::new();
    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![0x51]; // Return
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert!(result.is_null());
}

#[test]
fn test_vm_stack_operations_via_bytecode() {
    // Test stack via bytecode execution (push/pop are private)
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(42));
    bytecode.add_constant(Constant::Int(100));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (42) - pushes to stack
        0x00, 0x01, 0x00, // LoadConst 1 (100) - pushes to stack
        0x07, // Pop (removes 100)
        0x51, // Return (returns 42)
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_vm_dup_instruction_peeks() {
    // Test peek behavior via Dup instruction
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(42));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (42)
        0x08, // Dup (peeks and duplicates top)
        0x10, // Add (42 + 42 = 84)
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(84));
}

#[test]
fn test_vm_stack_underflow_error() {
    // Test stack underflow via bytecode (Pop on empty stack)
    let mut bytecode = Bytecode::new();
    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![0x07]; // Pop on empty stack
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    assert!(matches!(result, Err(Error::StackUnderflow)));
}



#[test]
fn test_vm_execute_load_const() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(42));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_vm_execute_arithmetic() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Int(5));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (10)
        0x00, 0x01, 0x00, // LoadConst 1 (5)
        0x10, // Add
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(15));
}

#[test]
fn test_vm_execute_comparison() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Int(5));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (10)
        0x00, 0x01, 0x00, // LoadConst 1 (5)
        0x24, // Gt (greater than)
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_bool(), Some(true));
}

#[test]
fn test_vm_execute_jump() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(42));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x40, 0x03, 0x00, 0x00, 0x00, // Jump +3 bytes
        0x51, // Return (skipped)
        0x00, 0x00, 0x00, // LoadConst 0 (42)
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(42));
}

#[test]
fn test_vm_execute_conditional_jump() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Bool(false));
    bytecode.add_constant(Constant::Int(100));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (false)
        0x41, 0x04, 0x00, 0x00, 0x00, // JumpIfFalse +4
        0x51, // Return (skipped)
        0x00, 0x01, 0x00, // LoadConst 1 (100)
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(100));
}

#[test]
fn test_vm_local_variables() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(42));

    let mut code = CodeObject::new(0, 2); // 2 locals
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
fn test_vm_global_variables() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(999));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (999)
        0x04, 0x00, 0x00, // StoreGlobal 0
        0x03, 0x00, 0x00, // LoadGlobal 0
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(999));
}

#[test]
fn test_vm_pop_instruction() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(1));
    bytecode.add_constant(Constant::Int(2));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (1)
        0x00, 0x01, 0x00, // LoadConst 1 (2)
        0x07, // Pop (removes 2)
        0x51, // Return (returns 1)
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(1));
}

#[test]
fn test_vm_dup_instruction() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(5));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (5)
        0x08, // Dup (stack: [5, 5])
        0x10, // Add (5 + 5 = 10)
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(10));
}

#[test]
fn test_vm_division_by_zero() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(10));
    bytecode.add_constant(Constant::Int(0));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (10)
        0x00, 0x01, 0x00, // LoadConst 1 (0)
        0x13, // Div
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    assert!(matches!(result, Err(Error::DivisionByZero)));
}

#[test]
fn test_vm_invalid_opcode() {
    let mut bytecode = Bytecode::new();
    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![0xFF]; // Invalid opcode
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode);
    assert!(matches!(result, Err(Error::InvalidOpcode { opcode: _ })));
}

#[test]
fn test_call_frame_initialization() {
    use std::rc::Rc;

    let code = CodeObject::new(2, 3);
    let frame = CallFrame::new(Rc::new(code), 0);

    assert_eq!(frame.ip, 0);
    assert_eq!(frame.stack_base, 0);
    assert_eq!(frame.locals.len(), 3);
}

#[test]
fn test_call_frame_fetch() {
    use std::rc::Rc;

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![0x10, 0x11, 0x12];

    let mut frame = CallFrame::new(Rc::new(code), 0);

    // fetch() returns Result<Instruction>, not Option<u8>
    // Test by checking IP advances
    assert_eq!(frame.ip, 0);
    let _ = frame.fetch(); // Fetch first byte
    assert_eq!(frame.ip, 1);
    let _ = frame.fetch(); // Fetch second byte
    assert_eq!(frame.ip, 2);
}

#[test]
fn test_call_frame_ip_advancement() {
    use std::rc::Rc;

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![0x00, 0x12, 0x34, 0x56];

    let mut frame = CallFrame::new(Rc::new(code), 0);
    
    // Simulate IP advancement (read_u16/read_i32 are private)
    assert_eq!(frame.ip, 0);
    let _ = frame.fetch(); // Advances IP
    assert_eq!(frame.ip, 1);
}
#[test]
fn test_call_frame_locals() {
    use std::rc::Rc;

    let code = CodeObject::new(2, 5); // 2 args, 5 locals
    let frame = CallFrame::new(Rc::new(code), 0);

    // Verify locals initialized
    assert_eq!(frame.locals.len(), 5);
    // All locals should be NULL initially
    for local in &frame.locals {
        assert!(local.is_null());
    }
}

#[test]
fn test_vm_negation_instruction() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Int(42));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (42)
        0x16, // Neg
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_int(), Some(-42));
}

#[test]
fn test_vm_logical_not() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Bool(true));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (true)
        0x32, // Not
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();
    assert_eq!(result.as_bool(), Some(false));
}

#[test]
fn test_vm_float_operations() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Constant::Float(3.5));
    bytecode.add_constant(Constant::Float(2.5));

    let mut code = CodeObject::new(0, 0);
    code.bytecode = vec![
        0x00, 0x00, 0x00, // LoadConst 0 (3.5)
        0x00, 0x01, 0x00, // LoadConst 1 (2.5)
        0x10, // Add
        0x51, // Return
    ];
    bytecode.add_code_object(code);

    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).unwrap();

    if let Some(f) = result.as_float() {
        assert!((f - 6.0).abs() < 0.001);
    } else {
        panic!("Expected float result");
    }
}
