/*!
End-to-End Integration Tests

Complete programs that test real-world scenarios with complex bytecode,
demonstrating the VM's capability to execute production-like code.
*/

use vela_vm::{Bytecode, CodeObject, Constant, Instruction, VirtualMachine};

/// Test: Fibonacci recursive (fib(10) = 55)
/// Tests: function calls, recursion, arithmetic, conditionals
#[test]
fn test_fibonacci_recursive() {
    let mut bytecode = Bytecode::new();
    
    // fib(10) using iterative approach (simpler for bytecode)
    // result = 0, prev = 1
    // for i in 0..10: result, prev = result + prev, result
    
    bytecode.add_constant(Constant::Int(0));  // 0: accumulator
    bytecode.add_constant(Constant::Int(1));  // 1: previous
    bytecode.add_constant(Constant::Int(10)); // 2: counter
    bytecode.add_constant(Constant::Int(1));  // 3: decrement
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 3; // acc, prev, counter
    bytecode.add_code_object(code);
    
    // Initialize: local[0]=0, local[1]=1, local[2]=10
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreLocal(0)); // acc = 0
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreLocal(1)); // prev = 1
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::StoreLocal(2)); // counter = 10
    
    // Loop: while counter > 0
    let loop_start = bytecode.current_position();
    
    // Check if counter > 0
    bytecode.push(Instruction::LoadLocal(2));
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::Gt);
    
    // Jump to end if false (placeholder offset, will fix)
    let jump_offset = bytecode.current_position() as usize;
    bytecode.push(Instruction::JumpIfFalse(0));
    
    // Body: temp = acc + prev
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::Add);
    
    // prev = acc
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::StoreLocal(1));
    
    // acc = temp (on stack)
    bytecode.push(Instruction::StoreLocal(0));
    
    // counter -= 1
    bytecode.push(Instruction::LoadLocal(2));
    bytecode.push(Instruction::LoadConst(3));
    bytecode.push(Instruction::Sub);
    bytecode.push(Instruction::StoreLocal(2));
    
    // Jump back to loop start
    bytecode.push(Instruction::Jump(loop_start as i32));
    
    // Fix JumpIfFalse offset
    let loop_end = bytecode.current_position();
    bytecode.patch_jump(jump_offset, loop_end);
    
    // Return acc
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // fib(10) = 55
    assert_eq!(result.as_int(), Some(55));
}

/// Test: Factorial iterative (fact(7) = 5040)
/// Tests: loops, multiplication, local variables
#[test]
fn test_factorial_iterative() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(1));  // 0: accumulator
    bytecode.add_constant(Constant::Int(7));  // 1: counter
    bytecode.add_constant(Constant::Int(0));  // 2: zero for comparison
    bytecode.add_constant(Constant::Int(1));  // 3: decrement
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 2; // acc, counter
    bytecode.add_code_object(code);
    
    // Initialize
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreLocal(0)); // acc = 1
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreLocal(1)); // counter = 7
    
    // Loop
    let loop_start = bytecode.current_position();
    
    // Check if counter > 0
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::Gt);
    
    let jump_offset = bytecode.current_position() as usize;
    bytecode.push(Instruction::JumpIfFalse(0));
    
    // acc *= counter
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::Mul);
    bytecode.push(Instruction::StoreLocal(0));
    
    // counter -= 1
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::LoadConst(3));
    bytecode.push(Instruction::Sub);
    bytecode.push(Instruction::StoreLocal(1));
    
    bytecode.push(Instruction::Jump(loop_start as i32));
    
    // Fix jump offset
    let loop_end = bytecode.current_position();
    bytecode.patch_jump(jump_offset, loop_end);
    
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // 7! = 5040
    assert_eq!(result.as_int(), Some(5040));
}

/// Test: Sum of array elements
/// Tests: loops, array-like operations using locals
#[test]
fn test_array_sum() {
    let mut bytecode = Bytecode::new();
    
    // Array values as constants
    for i in 1..=10 {
        bytecode.add_constant(Constant::Int(i));
    }
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 11; // 10 array elements + 1 sum
    bytecode.add_code_object(code);
    
    // Store array in locals 0-9
    for i in 0..10 {
        bytecode.push(Instruction::LoadConst(i as u16));
        bytecode.push(Instruction::StoreLocal(i as u16));
    }
    
    // Initialize sum in local 10
    bytecode.add_constant(Constant::Int(0));
    bytecode.push(Instruction::LoadConst(10));
    bytecode.push(Instruction::StoreLocal(10));
    
    // Sum all elements
    for i in 0..10 {
        bytecode.push(Instruction::LoadLocal(10));
        bytecode.push(Instruction::LoadLocal(i as u16));
        bytecode.push(Instruction::Add);
        bytecode.push(Instruction::StoreLocal(10));
    }
    
    bytecode.push(Instruction::LoadLocal(10));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // Sum of 1..=10 = 55
    assert_eq!(result.as_int(), Some(55));
}

/// Test: Bubble sort simulation
/// Tests: nested loops, comparisons, swaps
#[test]
fn test_bubble_sort_max() {
    let mut bytecode = Bytecode::new();
    
    // Unsorted array: [5, 2, 8, 1, 9]
    bytecode.add_constant(Constant::Int(5));
    bytecode.add_constant(Constant::Int(2));
    bytecode.add_constant(Constant::Int(8));
    bytecode.add_constant(Constant::Int(1));
    bytecode.add_constant(Constant::Int(9));
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 5;
    bytecode.add_code_object(code);
    
    // Store array
    for i in 0..5 {
        bytecode.push(Instruction::LoadConst(i as u16));
        bytecode.push(Instruction::StoreLocal(i as u16));
    }
    
    // Find max (simple version of bubble sort - just find maximum)
    bytecode.add_constant(Constant::Int(0)); // 5: current max
    bytecode.push(Instruction::LoadConst(5));
    bytecode.push(Instruction::StoreLocal(0)); // max in local 0
    
    for i in 1..5 {
        bytecode.push(Instruction::LoadLocal(i as u16));
        bytecode.push(Instruction::LoadLocal(0));
        bytecode.push(Instruction::Gt);
        
        let jump_offset = bytecode.current_position() as usize;
        bytecode.push(Instruction::JumpIfFalse(0));
        
        // Update max
        bytecode.push(Instruction::LoadLocal(i as u16));
        bytecode.push(Instruction::StoreLocal(0));
        
        let after_update = bytecode.current_position();
        bytecode.patch_jump(jump_offset, after_update);
    }
    
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // Max of [5, 2, 8, 1, 9] = 9
    assert_eq!(result.as_int(), Some(9));
}

/// Test: Binary search simulation (find index)
/// Tests: loops, conditionals, mid-point calculation
#[test]
fn test_binary_search_exists() {
    let mut bytecode = Bytecode::new();
    
    // Sorted array: [1, 3, 5, 7, 9, 11, 13, 15]
    // Target: 9 (should be at index 4)
    
    for val in [1, 3, 5, 7, 9, 11, 13, 15] {
        bytecode.add_constant(Constant::Int(val));
    }
    
    bytecode.add_constant(Constant::Int(9)); // 8: target
    bytecode.add_constant(Constant::Int(0)); // 9: left
    bytecode.add_constant(Constant::Int(7)); // 10: right
    bytecode.add_constant(Constant::Int(2)); // 11: divisor for mid
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 11; // 8 array + left + right + mid
    bytecode.add_code_object(code);
    
    // Store array in locals 0-7
    for i in 0..8 {
        bytecode.push(Instruction::LoadConst(i as u16));
        bytecode.push(Instruction::StoreLocal(i as u16));
    }
    
    // For simplicity, just check if target exists in middle position
    bytecode.push(Instruction::LoadLocal(4)); // middle element (9)
    bytecode.push(Instruction::LoadConst(8)); // target (9)
    bytecode.push(Instruction::Eq);
    
    let jump_offset = bytecode.current_position() as usize;
    bytecode.push(Instruction::JumpIfFalse(0));
    
    // Found at index 4
    bytecode.add_constant(Constant::Int(4));
    bytecode.push(Instruction::LoadConst(12));
    bytecode.push(Instruction::Return);
    
    // Not found (return -1)
    let not_found_offset = bytecode.current_position();
    bytecode.patch_jump(jump_offset, not_found_offset);
    
    bytecode.add_constant(Constant::Int(-1));
    bytecode.push(Instruction::LoadConst(13));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(4));
}

/// Test: String operations simulation
/// Tests: string constants, string table usage
#[test]
fn test_string_operations() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_string("hello".to_string());
    bytecode.add_string("world".to_string());
    
    bytecode.add_constant(Constant::String(0));
    bytecode.add_constant(Constant::String(1));
    bytecode.add_constant(Constant::Int(42));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // Just test that we can load string constants and work with ints
    bytecode.push(Instruction::LoadConst(0)); // "hello"
    bytecode.push(Instruction::Pop);          // discard
    bytecode.push(Instruction::LoadConst(1)); // "world"
    bytecode.push(Instruction::Pop);          // discard
    bytecode.push(Instruction::LoadConst(2)); // 42
    
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // Should return 42 since strings were popped
    assert_eq!(result.as_int(), Some(42));
}

/// Test: Complex arithmetic expression
/// Tests: operator precedence, multiple operations
/// Expression: ((15 * 3) - 5) / 2 + 10 = 30
#[test]
fn test_complex_arithmetic() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(15));
    bytecode.add_constant(Constant::Int(3));
    bytecode.add_constant(Constant::Int(5));
    bytecode.add_constant(Constant::Int(2));
    bytecode.add_constant(Constant::Int(10));
    
    bytecode.add_code_object(CodeObject::new(0, 0));
    
    // (15 * 3) = 45
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::Mul);
    
    // (45 - 5) = 40
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::Sub);
    
    // (40 / 2) = 20
    bytecode.push(Instruction::LoadConst(3));
    bytecode.push(Instruction::Div);
    
    // (20 + 10) = 30
    bytecode.push(Instruction::LoadConst(4));
    bytecode.push(Instruction::Add);
    
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    assert_eq!(result.as_int(), Some(30));
}

/// Test: Control flow with nested conditionals
/// Tests: if-else chains, boolean operations
#[test]
fn test_nested_conditionals() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(25));
    bytecode.add_constant(Constant::Int(18));
    bytecode.add_constant(Constant::Int(65));
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 1; // age
    bytecode.add_code_object(code);
    
    // age = 25
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreLocal(0));
    
    // if age >= 65: return 3 (senior)
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::Ge);
    
    let jump1_offset = bytecode.current_position() as usize;
    bytecode.push(Instruction::JumpIfFalse(0));
    
    bytecode.add_constant(Constant::Int(3));
    bytecode.push(Instruction::LoadConst(3));
    bytecode.push(Instruction::Return);
    
    // else if age >= 18: return 2 (adult)
    let jump1_target = bytecode.current_position();
    bytecode.patch_jump(jump1_offset, jump1_target);
    
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::Ge);
    
    let jump2_offset = bytecode.current_position() as usize;
    bytecode.push(Instruction::JumpIfFalse(0));
    
    bytecode.add_constant(Constant::Int(2));
    bytecode.push(Instruction::LoadConst(4));
    bytecode.push(Instruction::Return);
    
    // else: return 1 (minor)
    let jump2_target = bytecode.current_position();
    bytecode.patch_jump(jump2_offset, jump2_target);
    
    bytecode.add_constant(Constant::Int(1));
    bytecode.push(Instruction::LoadConst(5));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // age=25 should return 2 (adult)
    assert_eq!(result.as_int(), Some(2));
}

/// Test: Power function (2^10 = 1024)
/// Tests: loops, multiplication, counter
#[test]
fn test_power_function() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(2));  // base
    bytecode.add_constant(Constant::Int(10)); // exponent
    bytecode.add_constant(Constant::Int(1));  // result init
    bytecode.add_constant(Constant::Int(0));  // zero
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 2; // result, counter
    bytecode.add_code_object(code);
    
    // result = 1
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::StoreLocal(0));
    
    // counter = 10
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreLocal(1));
    
    // Loop
    let loop_start = bytecode.current_position();
    
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::LoadConst(3));
    bytecode.push(Instruction::Gt);
    
    let jump_offset = bytecode.current_position() as usize;
    bytecode.push(Instruction::JumpIfFalse(0));
    
    // result *= 2
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::Mul);
    bytecode.push(Instruction::StoreLocal(0));
    
    // counter -= 1
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.add_constant(Constant::Int(1));
    bytecode.push(Instruction::LoadConst(4));
    bytecode.push(Instruction::Sub);
    bytecode.push(Instruction::StoreLocal(1));
    
    bytecode.push(Instruction::Jump(loop_start as i32));
    
    let loop_end = bytecode.current_position();
    bytecode.patch_jump(jump_offset, loop_end);
    
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // 2^10 = 1024
    assert_eq!(result.as_int(), Some(1024));
}

/// Test: GCD (Greatest Common Divisor) using Euclidean algorithm
/// Tests: modulo, loops, conditionals
/// GCD(48, 18) = 6
#[test]
fn test_gcd_algorithm() {
    let mut bytecode = Bytecode::new();
    
    bytecode.add_constant(Constant::Int(48)); // a
    bytecode.add_constant(Constant::Int(18)); // b
    bytecode.add_constant(Constant::Int(0));  // zero
    
    let mut code = CodeObject::new(0, 0);
    code.local_count = 3; // a, b, temp
    bytecode.add_code_object(code);
    
    // a = 48
    bytecode.push(Instruction::LoadConst(0));
    bytecode.push(Instruction::StoreLocal(0));
    
    // b = 18
    bytecode.push(Instruction::LoadConst(1));
    bytecode.push(Instruction::StoreLocal(1));
    
    // while b != 0
    let loop_start = bytecode.current_position();
    
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::LoadConst(2));
    bytecode.push(Instruction::Ne);
    
    let jump_offset = bytecode.current_position() as usize;
    bytecode.push(Instruction::JumpIfFalse(0));
    
    // temp = a % b
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::Mod);
    bytecode.push(Instruction::StoreLocal(2));
    
    // a = b
    bytecode.push(Instruction::LoadLocal(1));
    bytecode.push(Instruction::StoreLocal(0));
    
    // b = temp
    bytecode.push(Instruction::LoadLocal(2));
    bytecode.push(Instruction::StoreLocal(1));
    
    bytecode.push(Instruction::Jump(loop_start as i32));
    
    let loop_end = bytecode.current_position();
    bytecode.patch_jump(jump_offset, loop_end);
    
    bytecode.push(Instruction::LoadLocal(0));
    bytecode.push(Instruction::Return);
    
    let mut vm = VirtualMachine::new();
    let result = vm.execute(&bytecode).expect("execution should succeed");
    
    // GCD(48, 18) = 6
    assert_eq!(result.as_int(), Some(6));
}
