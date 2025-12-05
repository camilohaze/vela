/*!
Performance Benchmarks for Sprint 10

Benchmarks comparing Vela VM performance against CPython baseline.
Target: 3-5x faster than CPython 3.12 for core operations.
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vela_vm::{Bytecode, CodeObject, Constant, Instruction, VirtualMachine};

/// Benchmark: Arithmetic operations (1M operations)
/// Target: 5-8x faster than CPython
fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");
    
    // Test with different operation counts
    for count in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut bytecode = Bytecode::new();
            
            // Create constants
            bytecode.add_constant(Constant::Int(10));
            bytecode.add_constant(Constant::Int(20));
            bytecode.add_constant(Constant::Int(2));
            
            let mut code = CodeObject::new(0, 0);
            code.local_count = 1;
            bytecode.add_code_object(code);
            
            // Initialize counter
            bytecode.add_constant(Constant::Int(count as i64));
            bytecode.push(Instruction::LoadConst(3));
            bytecode.push(Instruction::StoreLocal(0));
            
            // Loop: counter times do arithmetic
            let loop_start = bytecode.current_position();
            
            bytecode.push(Instruction::LoadLocal(0));
            bytecode.add_constant(Constant::Int(0));
            bytecode.push(Instruction::LoadConst(4));
            bytecode.push(Instruction::Gt);
            
            let jump_offset = bytecode.current_position() as usize;
            bytecode.push(Instruction::JumpIfFalse(0));
            
            // Arithmetic: (10 + 20) * 2 = 60
            bytecode.push(Instruction::LoadConst(0));
            bytecode.push(Instruction::LoadConst(1));
            bytecode.push(Instruction::Add);
            bytecode.push(Instruction::LoadConst(2));
            bytecode.push(Instruction::Mul);
            bytecode.push(Instruction::Pop);
            
            // Decrement counter
            bytecode.push(Instruction::LoadLocal(0));
            bytecode.add_constant(Constant::Int(1));
            bytecode.push(Instruction::LoadConst(5));
            bytecode.push(Instruction::Sub);
            bytecode.push(Instruction::StoreLocal(0));
            
            bytecode.push(Instruction::Jump(loop_start as i32));
            
            let loop_end = bytecode.current_position();
            bytecode.patch_jump(jump_offset, loop_end);
            
            bytecode.add_constant(Constant::Int(42));
            bytecode.push(Instruction::LoadConst(6));
            bytecode.push(Instruction::Return);
            
            b.iter(|| {
                let mut vm = VirtualMachine::new();
                black_box(vm.execute(&bytecode).unwrap());
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Fibonacci calculation
/// Target: 3-5x faster than CPython
fn bench_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");
    
    for n in [10, 20, 30].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(n), n, |b, &n| {
            let mut bytecode = Bytecode::new();
            
            bytecode.add_constant(Constant::Int(0));
            bytecode.add_constant(Constant::Int(1));
            bytecode.add_constant(Constant::Int(n as i64));
            
            let mut code = CodeObject::new(0, 0);
            code.local_count = 3;
            bytecode.add_code_object(code);
            
            // Initialize
            bytecode.push(Instruction::LoadConst(0));
            bytecode.push(Instruction::StoreLocal(0));
            bytecode.push(Instruction::LoadConst(1));
            bytecode.push(Instruction::StoreLocal(1));
            bytecode.push(Instruction::LoadConst(2));
            bytecode.push(Instruction::StoreLocal(2));
            
            // Loop
            let loop_start = bytecode.current_position();
            
            bytecode.push(Instruction::LoadLocal(2));
            bytecode.push(Instruction::LoadConst(0));
            bytecode.push(Instruction::Gt);
            
            let jump_offset = bytecode.current_position() as usize;
            bytecode.push(Instruction::JumpIfFalse(0));
            
            bytecode.push(Instruction::LoadLocal(0));
            bytecode.push(Instruction::LoadLocal(1));
            bytecode.push(Instruction::Add);
            
            bytecode.push(Instruction::LoadLocal(0));
            bytecode.push(Instruction::StoreLocal(1));
            
            bytecode.push(Instruction::StoreLocal(0));
            
            bytecode.push(Instruction::LoadLocal(2));
            bytecode.add_constant(Constant::Int(1));
            bytecode.push(Instruction::LoadConst(3));
            bytecode.push(Instruction::Sub);
            bytecode.push(Instruction::StoreLocal(2));
            
            bytecode.push(Instruction::Jump(loop_start as i32));
            
            let loop_end = bytecode.current_position();
            bytecode.patch_jump(jump_offset, loop_end);
            
            bytecode.push(Instruction::LoadLocal(0));
            bytecode.push(Instruction::Return);
            
            b.iter(|| {
                let mut vm = VirtualMachine::new();
                black_box(vm.execute(&bytecode).unwrap());
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Local variable operations
/// Target: 4-6x faster than CPython
fn bench_local_variables(c: &mut Criterion) {
    let mut group = c.benchmark_group("local_variables");
    
    for locals in [5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(locals), locals, |b, &locals| {
            let mut bytecode = Bytecode::new();
            
            for i in 0..locals {
                bytecode.add_constant(Constant::Int(i as i64));
            }
            
            let mut code = CodeObject::new(0, 0);
            code.local_count = locals as u16;
            bytecode.add_code_object(code);
            
            // Store all locals
            for i in 0..locals {
                bytecode.push(Instruction::LoadConst(i as u16));
                bytecode.push(Instruction::StoreLocal(i as u16));
            }
            
            // Load and sum all locals
            bytecode.push(Instruction::LoadLocal(0));
            for i in 1..locals {
                bytecode.push(Instruction::LoadLocal(i as u16));
                bytecode.push(Instruction::Add);
            }
            
            bytecode.push(Instruction::Return);
            
            b.iter(|| {
                let mut vm = VirtualMachine::new();
                black_box(vm.execute(&bytecode).unwrap());
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Global variable operations
/// Target: 3-4x faster than CPython
fn bench_global_variables(c: &mut Criterion) {
    let mut group = c.benchmark_group("global_variables");
    
    for globals in [5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(globals), globals, |b, &globals| {
            let mut bytecode = Bytecode::new();
            
            for i in 0..globals {
                bytecode.add_constant(Constant::Int(i as i64));
            }
            
            bytecode.add_code_object(CodeObject::new(0, 0));
            
            // Store all globals
            for i in 0..globals {
                bytecode.push(Instruction::LoadConst(i as u16));
                bytecode.push(Instruction::StoreGlobal(i as u16));
            }
            
            // Load and sum all globals
            bytecode.push(Instruction::LoadGlobal(0));
            for i in 1..globals {
                bytecode.push(Instruction::LoadGlobal(i as u16));
                bytecode.push(Instruction::Add);
            }
            
            bytecode.push(Instruction::Return);
            
            b.iter(|| {
                let mut vm = VirtualMachine::new();
                black_box(vm.execute(&bytecode).unwrap());
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Stack operations (push/pop)
/// Target: 6-10x faster than CPython
fn bench_stack_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack_operations");
    
    for depth in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            let mut bytecode = Bytecode::new();
            
            for i in 0..depth {
                bytecode.add_constant(Constant::Int(i as i64));
            }
            
            bytecode.add_code_object(CodeObject::new(0, 0));
            
            // Push all values
            for i in 0..depth {
                bytecode.push(Instruction::LoadConst(i as u16));
            }
            
            // Pop all but one
            for _ in 0..depth-1 {
                bytecode.push(Instruction::Pop);
            }
            
            bytecode.push(Instruction::Return);
            
            b.iter(|| {
                let mut vm = VirtualMachine::new();
                black_box(vm.execute(&bytecode).unwrap());
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Control flow (jumps and conditionals)
/// Target: 4-7x faster than CPython
fn bench_control_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("control_flow");
    
    for iterations in [100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(iterations), iterations, |b, &iterations| {
            let mut bytecode = Bytecode::new();
            
            bytecode.add_constant(Constant::Int(iterations as i64));
            bytecode.add_constant(Constant::Int(0));
            bytecode.add_constant(Constant::Int(1));
            bytecode.add_constant(Constant::Int(10));
            
            let mut code = CodeObject::new(0, 0);
            code.local_count = 2;
            bytecode.add_code_object(code);
            
            // counter = iterations
            bytecode.push(Instruction::LoadConst(0));
            bytecode.push(Instruction::StoreLocal(0));
            
            // sum = 0
            bytecode.push(Instruction::LoadConst(1));
            bytecode.push(Instruction::StoreLocal(1));
            
            // Loop
            let loop_start = bytecode.current_position();
            
            bytecode.push(Instruction::LoadLocal(0));
            bytecode.push(Instruction::LoadConst(1));
            bytecode.push(Instruction::Gt);
            
            let jump_offset = bytecode.current_position() as usize;
            bytecode.push(Instruction::JumpIfFalse(0));
            
            // if counter % 2 == 0: sum += 10
            bytecode.push(Instruction::LoadLocal(0));
            bytecode.push(Instruction::LoadConst(2));
            bytecode.push(Instruction::Mod);
            bytecode.push(Instruction::LoadConst(1));
            bytecode.push(Instruction::Eq);
            
            let skip_offset = bytecode.current_position() as usize;
            bytecode.push(Instruction::JumpIfFalse(0));
            
            bytecode.push(Instruction::LoadLocal(1));
            bytecode.push(Instruction::LoadConst(3));
            bytecode.push(Instruction::Add);
            bytecode.push(Instruction::StoreLocal(1));
            
            let skip_target = bytecode.current_position();
            bytecode.patch_jump(skip_offset, skip_target);
            
            // counter -= 1
            bytecode.push(Instruction::LoadLocal(0));
            bytecode.push(Instruction::LoadConst(2));
            bytecode.push(Instruction::Sub);
            bytecode.push(Instruction::StoreLocal(0));
            
            bytecode.push(Instruction::Jump(loop_start as i32));
            
            let loop_end = bytecode.current_position();
            bytecode.patch_jump(jump_offset, loop_end);
            
            bytecode.push(Instruction::LoadLocal(1));
            bytecode.push(Instruction::Return);
            
            b.iter(|| {
                let mut vm = VirtualMachine::new();
                black_box(vm.execute(&bytecode).unwrap());
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_arithmetic,
    bench_fibonacci,
    bench_local_variables,
    bench_global_variables,
    bench_stack_operations,
    bench_control_flow
);

criterion_main!(benches);
