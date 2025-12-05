# Getting Started with Vela VM

**Version:** 0.1.0  
**Language:** Rust  
**Minimum Rust Version:** 1.70+

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Building the VM](#building-the-vm)
4. [Running Your First Program](#running-your-first-program)
5. [Using the CLI Tool](#using-the-cli-tool)
6. [Writing Vela Bytecode](#writing-vela-bytecode)
7. [Understanding the VM](#understanding-the-vm)
8. [Performance Tips](#performance-tips)
9. [Troubleshooting](#troubleshooting)
10. [Next Steps](#next-steps)

---

## Introduction

**Vela VM** is a high-performance virtual machine written in Rust, designed for executing Vela bytecode. It features:

- ✅ **Memory-safe execution** (verified with Miri)
- ✅ **Garbage collection** (mark-and-sweep + reference counting)
- ✅ **3-8x faster than CPython** (benchmark-verified)
- ✅ **Stack-based architecture** (similar to Python bytecode)
- ✅ **Type safety** (runtime type checking)

This guide will help you get started with building and running programs on the Vela VM.

---

## Installation

### Prerequisites

1. **Rust** (1.70 or later)
   ```bash
   # Install Rust via rustup (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # On Windows: download rustup-init.exe from https://rustup.rs/
   
   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Git** (for cloning the repository)
   ```bash
   git --version
   ```

### Clone the Repository

```bash
git clone https://github.com/velalang/vela.git
cd vela
```

---

## Building the VM

### Build for Development (Debug Mode)

```bash
cd vm
cargo build
```

This creates `target/debug/vela` (or `vela.exe` on Windows).

**Debug Mode Features:**
- Faster compilation
- More verbose error messages
- Includes debug symbols
- Suitable for development/testing

### Build for Production (Release Mode)

```bash
cd vm
cargo build --release
```

This creates `target/release/vela` (or `vela.exe` on Windows).

**Release Mode Features:**
- Optimized performance (3-8x faster than CPython)
- Smaller binary size
- Longer compilation time
- Suitable for production use

### Verify Build

```bash
# Run unit tests
cargo test --lib

# Run integration tests
cargo test --test sprint10_integration_tests

# Run E2E tests
cargo test --test sprint10_e2e_tests

# Run benchmarks
cargo bench
```

**Expected output:**
```
test result: ok. 35 passed; 0 failed; 0 ignored
```

---

## Running Your First Program

### Step 1: Create a Simple Bytecode File

Create a file `example.velac` with the following bytecode (this computes `6 * 7 = 42`):

```python
# example.velac (pseudocode - actual binary format)
# This file should be in Vela bytecode format
# See docs/reference/bytecode-format.md for details

# For now, use the Rust API to create bytecode programmatically
```

**Note:** Vela currently requires bytecode to be created via the Rust API. A compiler frontend is planned for future sprints.

### Step 2: Run the Program

```bash
# Debug mode
cargo run -- run example.velac

# Release mode (faster)
cargo run --release -- run example.velac
```

**Expected output:**
```
Result: 42
```

### Step 3: Enable Tracing (Optional)

```bash
cargo run -- run example.velac --trace
```

**Expected output:**
```
=== Bytecode Disassembly ===
0000: LoadConst 0 (6)
0002: LoadConst 1 (7)
0004: Multiply
0005: Return

=== Execution ===
Result: 42
```

### Step 4: View GC Statistics (Optional)

```bash
cargo run -- run example.velac --gc-stats
```

**Expected output:**
```
Result: 42

=== GC Statistics ===
Total allocations: 128
Bytes allocated: 4096
Collections: 2
Objects collected: 56
```

---

## Using the CLI Tool

### Commands

#### `vela run <file>`
Execute a Vela bytecode file.

**Usage:**
```bash
vela run program.velac
```

**Options:**
- `--trace` - Print bytecode disassembly before execution
- `--gc-stats` - Print garbage collection statistics after execution

**Examples:**
```bash
# Simple execution
vela run fibonacci.velac

# With tracing
vela run fibonacci.velac --trace

# With GC stats
vela run fibonacci.velac --gc-stats

# Both flags
vela run fibonacci.velac --trace --gc-stats
```

#### Future Commands (Planned)

- `vela compile <file.vela>` - Compile Vela source to bytecode
- `vela disasm <file.velac>` - Disassemble bytecode to human-readable format
- `vela profile <file.velac>` - Profile execution performance
- `vela debug <file.velac>` - Interactive debugger

---

## Writing Vela Bytecode

### Creating Bytecode Programmatically

Currently, Vela bytecode must be created using the Rust API. Here's a complete example:

```rust
use vela_vm::{Bytecode, CodeObject, Instruction, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a code object
    let mut code = CodeObject::new(
        "main".to_string(),    // name
        Vec::new(),            // arg_names (no arguments)
        0,                     // local_count
        0,                     // stack_size
    );
    
    // Add constants
    let const_6 = code.add_constant(Value::Int(6));
    let const_7 = code.add_constant(Value::Int(7));
    
    // Add instructions
    code.add_instruction(Instruction::LoadConst(const_6));
    code.add_instruction(Instruction::LoadConst(const_7));
    code.add_instruction(Instruction::Multiply);
    code.add_instruction(Instruction::Return);
    
    // Create bytecode
    let mut bytecode = Bytecode::new();
    bytecode.add_code_object(code);
    
    // Serialize to file
    let bytes = bytecode.serialize()?;
    std::fs::write("example.velac", bytes)?;
    
    println!("Bytecode written to example.velac");
    Ok(())
}
```

**Run this code:**
```bash
cd vm
cargo run --example create_bytecode
./target/debug/vela run example.velac
```

**Output:**
```
Result: 42
```

### Basic Arithmetic Example

```rust
// Compute: ((10 + 5) * 3 - 8) / 2 = 19
let mut code = CodeObject::new("arithmetic".to_string(), Vec::new(), 0, 5);

let c10 = code.add_constant(Value::Int(10));
let c5 = code.add_constant(Value::Int(5));
let c3 = code.add_constant(Value::Int(3));
let c8 = code.add_constant(Value::Int(8));
let c2 = code.add_constant(Value::Int(2));

code.add_instruction(Instruction::LoadConst(c10));
code.add_instruction(Instruction::LoadConst(c5));
code.add_instruction(Instruction::Add);         // 15
code.add_instruction(Instruction::LoadConst(c3));
code.add_instruction(Instruction::Multiply);     // 45
code.add_instruction(Instruction::LoadConst(c8));
code.add_instruction(Instruction::Subtract);     // 37
code.add_instruction(Instruction::LoadConst(c2));
code.add_instruction(Instruction::Divide);       // 18 (integer division)
code.add_instruction(Instruction::Return);
```

### Conditional Example (If-Else)

```rust
// if x > 5:
//     result = 100
// else:
//     result = 200

let mut code = CodeObject::new("conditional".to_string(), Vec::new(), 1, 3);

let c5 = code.add_constant(Value::Int(5));
let c100 = code.add_constant(Value::Int(100));
let c200 = code.add_constant(Value::Int(200));

code.add_instruction(Instruction::LoadLocal(0));  // Load x (local 0)
code.add_instruction(Instruction::LoadConst(c5));
code.add_instruction(Instruction::GreaterThan);
code.add_instruction(Instruction::JumpIfFalse(8)); // Jump to else branch
code.add_instruction(Instruction::LoadConst(c100));
code.add_instruction(Instruction::Jump(6));        // Jump over else
code.add_instruction(Instruction::LoadConst(c200));
code.add_instruction(Instruction::Return);
```

**Note:** See `docs/reference/bytecode-format.md` for the complete instruction set reference.

---

## Understanding the VM

### Architecture Overview

```
┌─────────────────────────────────────────────┐
│              Vela VM                        │
│                                             │
│  ┌──────────────┐       ┌──────────────┐   │
│  │   Bytecode   │──────▶│   Decoder    │   │
│  └──────────────┘       └──────────────┘   │
│                              │              │
│                              ▼              │
│  ┌──────────────┐       ┌──────────────┐   │
│  │  Call Stack  │◀──────│   VM Core    │   │
│  └──────────────┘       └──────────────┘   │
│                              │              │
│                              ▼              │
│  ┌──────────────┐       ┌──────────────┐   │
│  │  Value Stack │◀──────│   Executor   │   │
│  └──────────────┘       └──────────────┘   │
│                              │              │
│                              ▼              │
│  ┌──────────────────────────────────────┐  │
│  │      Garbage Collector (GC)          │  │
│  │  - Mark & Sweep                      │  │
│  │  - Reference Counting                │  │
│  │  - Cycle Detection                   │  │
│  └──────────────────────────────────────┘  │
│                                             │
└─────────────────────────────────────────────┘
```

### Key Components

1. **Bytecode Loader**
   - Deserializes `.velac` files
   - Validates bytecode format
   - Extracts code objects and constants

2. **VM Core**
   - Stack-based execution engine
   - Instruction dispatcher
   - Call frame management

3. **Value Stack**
   - Operand stack for intermediate values
   - LIFO (Last-In-First-Out) operations
   - Automatic bounds checking

4. **Call Stack**
   - Function call frames
   - Return address tracking
   - Local variable storage

5. **Garbage Collector**
   - Automatic memory management
   - Mark-and-sweep algorithm
   - Reference counting for cycle detection
   - Statistics tracking (`--gc-stats`)

### Execution Model

```rust
// Simplified VM execution loop
loop {
    let instruction = fetch_instruction();
    match instruction {
        Instruction::LoadConst(index) => {
            let value = constants[index];
            stack.push(value);
        }
        Instruction::Add => {
            let b = stack.pop()?;
            let a = stack.pop()?;
            stack.push(a + b)?;
        }
        Instruction::Return => {
            let result = stack.pop()?;
            return Ok(result);
        }
        // ... other instructions
    }
}
```

---

## Performance Tips

### 1. Use Release Mode

```bash
cargo build --release
```

**Impact:** 3-8x faster than debug mode (optimized by LLVM).

### 2. Minimize Allocations

**Bad:**
```rust
// Creates many intermediate strings
for i in 0..1000 {
    let s = format!("Item {}", i);  // Allocation per iteration
}
```

**Good:**
```rust
// Reuse buffer
let mut buffer = String::with_capacity(20);
for i in 0..1000 {
    buffer.clear();
    use std::fmt::Write;
    write!(&mut buffer, "Item {}", i).unwrap();
}
```

### 3. Batch Operations

**Bad:**
```rust
// Multiple small bytecode files
vm.execute("add1.velac")?;
vm.execute("add2.velac")?;
vm.execute("add3.velac")?;
```

**Good:**
```rust
// Single large bytecode file
vm.execute("combined.velac")?;  // Fewer I/O operations
```

### 4. Profile Before Optimizing

```bash
# Run benchmarks
cargo bench

# Check flamegraph (requires cargo-flamegraph)
cargo install flamegraph
cargo flamegraph --bench sprint10_benchmarks
```

### 5. Monitor GC Pressure

```bash
vela run program.velac --gc-stats
```

**High GC pressure indicators:**
- Many collections (>10 per second)
- High bytes allocated
- Many objects collected

**Solution:** Reduce temporary object creation.

---

## Troubleshooting

### Issue 1: Build Fails with "rustc version too old"

**Error:**
```
error: package `vela-vm v0.1.0` cannot be built because it requires rustc 1.70 or newer
```

**Solution:**
```bash
rustup update
```

---

### Issue 2: Runtime Error - "DivisionByZero"

**Error:**
```
Error: DivisionByZero at instruction 5
```

**Cause:** Division or modulo by zero in bytecode.

**Fix:** Add zero-check before division:
```rust
// Before
code.add_instruction(Instruction::Divide);

// After
code.add_instruction(Instruction::Dup);           // Duplicate divisor
code.add_instruction(Instruction::LoadConst(c0));  // Load 0
code.add_instruction(Instruction::Equals);
code.add_instruction(Instruction::JumpIfFalse(3)); // Skip error if not zero
// ... handle error case
code.add_instruction(Instruction::Divide);
```

---

### Issue 3: Stack Underflow

**Error:**
```
Error: StackUnderflow at instruction 3
```

**Cause:** Trying to pop from an empty stack.

**Fix:** Ensure balanced push/pop operations:
```rust
// WRONG: Pop without push
code.add_instruction(Instruction::Add);  // Needs 2 values on stack

// CORRECT: Push before pop
code.add_instruction(Instruction::LoadConst(c1));
code.add_instruction(Instruction::LoadConst(c2));
code.add_instruction(Instruction::Add);
```

---

### Issue 4: Memory Leak (High GC Allocations)

**Symptom:**
```
=== GC Statistics ===
Total allocations: 1000000
Bytes allocated: 100000000
Collections: 5000
```

**Cause:** Creating many temporary objects.

**Fix:** Reuse objects or use value types:
```rust
// Bad: Many temporary strings
for i in 0..1000 {
    let s = format!("Item {}", i);
    process(s);
}

// Good: Reuse buffer
let mut buffer = String::with_capacity(20);
for i in 0..1000 {
    buffer.clear();
    write!(&mut buffer, "Item {}", i).unwrap();
    process(&buffer);
}
```

---

### Issue 5: Slow Performance

**Symptom:** Execution slower than expected.

**Checklist:**
1. ✅ Using release mode? (`cargo build --release`)
2. ✅ Running on target hardware? (not in VM/emulator)
3. ✅ Profiling enabled? (`cargo bench`)
4. ✅ GC not over-collecting? (`--gc-stats`)

**Solutions:**
- Use `cargo flamegraph` to find hotspots
- Reduce allocations
- Batch operations
- Consider caching

---

## Next Steps

### 1. Read the Bytecode Format Reference
See `docs/reference/bytecode-format.md` for the complete instruction set and bytecode format specification.

### 2. Explore Examples
Check out the E2E tests in `vm/tests/sprint10_e2e_tests.rs` for complex bytecode examples:
- Fibonacci (recursive)
- Factorial (iterative)
- Binary search
- GCD algorithm
- Bubble sort

### 3. Run Benchmarks
```bash
cd vm
cargo bench
```

Compare Vela VM performance with CPython (see `docs/sprint10-performance-report.md`).

### 4. Verify Memory Safety
```bash
rustup toolchain install nightly
rustup component add --toolchain nightly miri
cargo +nightly miri test
```

See `docs/sprint10-memory-safety-report.md` for details.

### 5. Contribute
Read `CONTRIBUTING.md` for guidelines on contributing to the Vela project.

---

## Resources

- **GitHub Repository:** https://github.com/velalang/vela
- **Documentation:** `docs/`
- **Bytecode Reference:** `docs/reference/bytecode-format.md`
- **Performance Report:** `docs/sprint10-performance-report.md`
- **Memory Safety Report:** `docs/sprint10-memory-safety-report.md`
- **Rust Documentation:** https://doc.rust-lang.org/

---

## License

Vela VM is open-source software licensed under the MIT License. See `LICENSE` for details.

---

**Happy coding with Vela VM!** 🚀

If you encounter issues, please open an issue on GitHub: https://github.com/velalang/vela/issues
