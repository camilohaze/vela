# Vela Bytecode Format Reference

**Version:** 0.1.0  
**Format Version:** 1  
**Specification Date:** 2025-01-30

---

## Table of Contents

1. [Overview](#overview)
2. [File Format](#file-format)
3. [Data Types](#data-types)
4. [Value Encoding](#value-encoding)
5. [Code Objects](#code-objects)
6. [Instruction Set](#instruction-set)
7. [Examples](#examples)
8. [Appendix](#appendix)

---

## Overview

Vela bytecode (`.velac` files) is a compact binary format designed for efficient execution on the Vela VM. The format is:

- **Platform-independent** (little-endian encoding)
- **Version-controlled** (magic number + version)
- **Compact** (optimized for size and parsing speed)
- **Type-safe** (runtime type checking)

### Design Goals

1. **Fast loading** - Minimal parsing overhead
2. **Compact storage** - Efficient binary encoding
3. **Forward compatibility** - Version checking
4. **Security** - Bounds checking, validation

---

## File Format

### Structure

```
┌─────────────────────────────────────────────┐
│           VELA BYTECODE FILE                │
├─────────────────────────────────────────────┤
│  Magic Number (4 bytes)                     │  "VELA"
│  Version (4 bytes)                          │  u32 (currently 1)
│  Timestamp (8 bytes)                        │  u64 (Unix timestamp)
│  Code Object Count (4 bytes)               │  u32
├─────────────────────────────────────────────┤
│  Code Object 1                              │
│    - Name (length-prefixed string)          │
│    - Argument Names (array of strings)      │
│    - Local Count (u16)                      │
│    - Stack Size (u16)                       │
│    - Bytecode (length-prefixed bytes)       │
│    - Constants (array of values)            │
│    - Names (array of strings)               │
├─────────────────────────────────────────────┤
│  Code Object 2                              │
│  ...                                        │
├─────────────────────────────────────────────┤
│  Code Object N                              │
└─────────────────────────────────────────────┘
```

### Header

| Field | Type | Size | Description |
|-------|------|------|-------------|
| Magic | `[u8; 4]` | 4 bytes | ASCII "VELA" (0x56454C41) |
| Version | `u32` | 4 bytes | Format version (1) |
| Timestamp | `u64` | 8 bytes | Unix timestamp (seconds since epoch) |
| Code Count | `u32` | 4 bytes | Number of code objects |

**Total Header Size:** 20 bytes

### Validation Rules

1. Magic number MUST be `[0x56, 0x45, 0x4C, 0x41]` ("VELA")
2. Version MUST be supported by the VM (currently: 1)
3. Code count MUST be > 0
4. Timestamp MAY be zero (ignored by VM)

---

## Data Types

### Primitive Types

| Type | Description | Encoding |
|------|-------------|----------|
| `u8` | Unsigned 8-bit integer | 1 byte |
| `u16` | Unsigned 16-bit integer (little-endian) | 2 bytes |
| `u32` | Unsigned 32-bit integer (little-endian) | 4 bytes |
| `u64` | Unsigned 64-bit integer (little-endian) | 8 bytes |
| `i32` | Signed 32-bit integer (little-endian) | 4 bytes |
| `i64` | Signed 64-bit integer (little-endian) | 8 bytes |
| `f64` | 64-bit floating-point (IEEE 754) | 8 bytes |

### String Encoding

Strings are length-prefixed UTF-8:

```
┌──────────────┬─────────────────────┐
│ Length (u32) │ UTF-8 Bytes         │
└──────────────┴─────────────────────┘
```

**Example:** "Hello"
```
0x05 0x00 0x00 0x00   # Length = 5
0x48 0x65 0x6C 0x6C 0x6F  # "Hello"
```

### Array Encoding

Arrays are length-prefixed sequences:

```
┌──────────────┬─────────┬─────────┬─────────┐
│ Length (u32) │ Item 1  │ Item 2  │ ...     │
└──────────────┴─────────┴─────────┴─────────┘
```

---

## Value Encoding

### Value Types

Values are tagged unions (discriminant + data):

```rust
enum Value {
    Null,              // Tag: 0x00
    Bool(bool),        // Tag: 0x01
    Int(i64),          // Tag: 0x02
    Float(f64),        // Tag: 0x03
    Ptr(usize),        // Tag: 0x04 (GC-managed object pointer)
}
```

### Serialization Format

```
┌──────────────┬─────────────────────┐
│ Tag (u8)     │ Data (size varies)  │
└──────────────┴─────────────────────┘
```

#### 1. Null
```
0x00  # Tag only (no data)
```

#### 2. Bool
```
0x01  # Tag
0x00 or 0x01  # false (0) or true (1)
```

#### 3. Int
```
0x02  # Tag
[8 bytes]  # i64 little-endian
```

**Example:** `42`
```
0x02  # Tag
0x2A 0x00 0x00 0x00 0x00 0x00 0x00 0x00  # 42 as i64
```

#### 4. Float
```
0x03  # Tag
[8 bytes]  # f64 IEEE 754
```

**Example:** `3.14`
```
0x03  # Tag
0x1F 0x85 0xEB 0x51 0xB8 0x1E 0x09 0x40  # 3.14 as f64
```

#### 5. Ptr (GC Object)
```
0x04  # Tag
[8 bytes]  # usize (platform-dependent)
```

**Note:** Pointers are NOT serialized. They are reconstructed at runtime by the GC.

---

## Code Objects

### Structure

```rust
struct CodeObject {
    name: String,             // Function/module name
    arg_names: Vec<String>,   // Argument names (for debugging)
    local_count: u16,         // Number of local variables
    stack_size: u16,          // Maximum stack depth
    bytecode: Vec<u8>,        // Instruction bytes
    constants: Vec<Value>,    // Constant pool
    names: Vec<String>,       // Name table (globals, attributes)
}
```

### Serialization Format

```
┌─────────────────────────────────────────────┐
│  Name (length-prefixed string)              │
├─────────────────────────────────────────────┤
│  Argument Count (u32)                       │
│    - Arg Name 1 (length-prefixed string)    │
│    - Arg Name 2                             │
│    - ...                                    │
├─────────────────────────────────────────────┤
│  Local Count (u16)                          │
│  Stack Size (u16)                           │
├─────────────────────────────────────────────┤
│  Bytecode Length (u32)                      │
│    - Bytecode bytes                         │
├─────────────────────────────────────────────┤
│  Constant Count (u32)                       │
│    - Constant 1 (Value)                     │
│    - Constant 2 (Value)                     │
│    - ...                                    │
├─────────────────────────────────────────────┤
│  Name Count (u32)                           │
│    - Name 1 (length-prefixed string)        │
│    - Name 2                                 │
│    - ...                                    │
└─────────────────────────────────────────────┘
```

---

## Instruction Set

### Format

Instructions are variable-length:

```
┌──────────────┬─────────────────────┐
│ Opcode (u8)  │ Operands (varies)   │
└──────────────┴─────────────────────┘
```

### Opcode Table

| Opcode | Mnemonic | Operands | Stack Effect | Description |
|--------|----------|----------|--------------|-------------|
| 0x00 | `Nop` | - | - | No operation |
| 0x01 | `LoadConst` | u16 | +1 | Push constant[index] |
| 0x02 | `LoadNull` | - | +1 | Push null |
| 0x03 | `LoadTrue` | - | +1 | Push true |
| 0x04 | `LoadFalse` | - | +1 | Push false |
| 0x05 | `LoadLocal` | u16 | +1 | Push local[index] |
| 0x06 | `StoreLocal` | u16 | -1 | Pop → local[index] |
| 0x07 | `LoadGlobal` | u16 | +1 | Push global[index] |
| 0x08 | `StoreGlobal` | u16 | -1 | Pop → global[index] |
| 0x09 | `Pop` | - | -1 | Discard top value |
| 0x0A | `Dup` | - | +1 | Duplicate top value |
| 0x10 | `Add` | - | -1 | a + b → result |
| 0x11 | `Subtract` | - | -1 | a - b → result |
| 0x12 | `Multiply` | - | -1 | a * b → result |
| 0x13 | `Divide` | - | -1 | a / b → result (integer division) |
| 0x14 | `Modulo` | - | -1 | a % b → result |
| 0x15 | `Negate` | - | 0 | -a → result |
| 0x20 | `Equals` | - | -1 | a == b → bool |
| 0x21 | `NotEquals` | - | -1 | a != b → bool |
| 0x22 | `LessThan` | - | -1 | a < b → bool |
| 0x23 | `LessOrEqual` | - | -1 | a <= b → bool |
| 0x24 | `GreaterThan` | - | -1 | a > b → bool |
| 0x25 | `GreaterOrEqual` | - | -1 | a >= b → bool |
| 0x30 | `Jump` | i32 | 0 | Jump to offset (signed) |
| 0x31 | `JumpIfFalse` | i32 | -1 | Jump if top is falsy |
| 0x32 | `JumpIfTrue` | i32 | -1 | Jump if top is truthy |
| 0x40 | `Call` | u16 | varies | Call function (arg_count) |
| 0x41 | `Return` | - | -1 | Return top value |
| 0xFF | `Halt` | - | 0 | Stop execution |

### Stack Effect Notation

- `+1` - Pushes one value onto stack
- `-1` - Pops one value from stack
- `0` - No net change to stack depth
- `varies` - Depends on operand (e.g., `Call`)

---

## Detailed Instruction Reference

### 1. Load/Store Instructions

#### `LoadConst index:u16`
**Opcode:** 0x01  
**Operands:** Constant pool index (2 bytes)  
**Stack:** `[] → [value]`  
**Description:** Push constant from constant pool onto stack.

**Encoding:**
```
0x01  # Opcode
[u16] # Index (little-endian)
```

**Example:**
```
0x01 0x03 0x00  # LoadConst 3 (load constants[3])
```

---

#### `LoadLocal index:u16`
**Opcode:** 0x05  
**Operands:** Local variable index (2 bytes)  
**Stack:** `[] → [value]`  
**Description:** Push local variable onto stack.

**Example:**
```
0x05 0x00 0x00  # LoadLocal 0
```

---

#### `StoreLocal index:u16`
**Opcode:** 0x06  
**Operands:** Local variable index (2 bytes)  
**Stack:** `[value] → []`  
**Description:** Pop value from stack and store in local variable.

**Example:**
```
0x06 0x02 0x00  # StoreLocal 2 (store in locals[2])
```

---

#### `LoadGlobal index:u16`
**Opcode:** 0x07  
**Operands:** Global name index (2 bytes)  
**Stack:** `[] → [value]`  
**Description:** Push global variable onto stack.

**Example:**
```
0x07 0x01 0x00  # LoadGlobal 1 (names[1])
```

---

#### `StoreGlobal index:u16`
**Opcode:** 0x08  
**Operands:** Global name index (2 bytes)  
**Stack:** `[value] → []`  
**Description:** Pop value and store in global variable.

**Example:**
```
0x08 0x01 0x00  # StoreGlobal 1
```

---

### 2. Stack Manipulation

#### `Pop`
**Opcode:** 0x09  
**Operands:** None  
**Stack:** `[value] → []`  
**Description:** Discard top value from stack.

**Example:**
```
0x09  # Pop
```

---

#### `Dup`
**Opcode:** 0x0A  
**Operands:** None  
**Stack:** `[a] → [a, a]`  
**Description:** Duplicate top value on stack.

**Example:**
```
0x0A  # Dup
```

---

### 3. Arithmetic Instructions

#### `Add`
**Opcode:** 0x10  
**Operands:** None  
**Stack:** `[a, b] → [result]`  
**Description:** Add two values: `a + b`.  
**Types:** Int + Int → Int, Float + Float → Float

**Example:**
```
0x01 0x00 0x00  # LoadConst 0 (push 5)
0x01 0x01 0x00  # LoadConst 1 (push 3)
0x10            # Add (5 + 3 = 8)
```

---

#### `Subtract`
**Opcode:** 0x11  
**Operands:** None  
**Stack:** `[a, b] → [result]`  
**Description:** Subtract: `a - b`.

---

#### `Multiply`
**Opcode:** 0x12  
**Operands:** None  
**Stack:** `[a, b] → [result]`  
**Description:** Multiply: `a * b`.

---

#### `Divide`
**Opcode:** 0x13  
**Operands:** None  
**Stack:** `[a, b] → [result]`  
**Description:** Divide: `a / b` (integer division for ints).  
**Error:** `DivisionByZero` if b == 0.

---

#### `Modulo`
**Opcode:** 0x14  
**Operands:** None  
**Stack:** `[a, b] → [result]`  
**Description:** Modulo: `a % b`.  
**Error:** `DivisionByZero` if b == 0.

---

#### `Negate`
**Opcode:** 0x15  
**Operands:** None  
**Stack:** `[a] → [-a]`  
**Description:** Negate value: `-a`.

---

### 4. Comparison Instructions

#### `Equals`
**Opcode:** 0x20  
**Operands:** None  
**Stack:** `[a, b] → [bool]`  
**Description:** Compare equality: `a == b`.

---

#### `LessThan`
**Opcode:** 0x22  
**Operands:** None  
**Stack:** `[a, b] → [bool]`  
**Description:** Compare: `a < b`.

---

#### `GreaterThan`
**Opcode:** 0x24  
**Operands:** None  
**Stack:** `[a, b] → [bool]`  
**Description:** Compare: `a > b`.

---

### 5. Control Flow Instructions

#### `Jump offset:i32`
**Opcode:** 0x30  
**Operands:** Signed offset (4 bytes, little-endian)  
**Stack:** No change  
**Description:** Unconditional jump by offset (relative to current PC).

**Encoding:**
```
0x30  # Opcode
[i32] # Offset (little-endian, signed)
```

**Example:**
```
0x30 0x0A 0x00 0x00 0x00  # Jump forward 10 bytes
0x30 0xF6 0xFF 0xFF 0xFF  # Jump backward 10 bytes (-10)
```

**Calculation:**
```rust
new_pc = current_pc + offset
```

---

#### `JumpIfFalse offset:i32`
**Opcode:** 0x31  
**Operands:** Signed offset (4 bytes)  
**Stack:** `[condition] → []`  
**Description:** Jump if top value is falsy (false, 0, null).

**Example:**
```
0x05 0x00 0x00              # LoadLocal 0 (load condition)
0x31 0x0A 0x00 0x00 0x00    # JumpIfFalse +10 (skip if-block)
```

---

#### `JumpIfTrue offset:i32`
**Opcode:** 0x32  
**Operands:** Signed offset (4 bytes)  
**Stack:** `[condition] → []`  
**Description:** Jump if top value is truthy.

---

### 6. Function Calls

#### `Call arg_count:u16`
**Opcode:** 0x40  
**Operands:** Argument count (2 bytes)  
**Stack:** `[func, arg1, ..., argN] → [result]`  
**Description:** Call function with N arguments.

**Example:**
```
0x07 0x00 0x00    # LoadGlobal 0 (function)
0x01 0x00 0x00    # LoadConst 0 (arg1)
0x01 0x01 0x00    # LoadConst 1 (arg2)
0x40 0x02 0x00    # Call 2 (call with 2 args)
```

---

#### `Return`
**Opcode:** 0x41  
**Operands:** None  
**Stack:** `[return_value] → []`  
**Description:** Return from current function.

**Example:**
```
0x01 0x00 0x00  # LoadConst 0 (return value)
0x41            # Return
```

---

### 7. Special Instructions

#### `Nop`
**Opcode:** 0x00  
**Operands:** None  
**Description:** No operation (padding).

---

#### `Halt`
**Opcode:** 0xFF  
**Operands:** None  
**Description:** Stop VM execution immediately.

---

## Examples

### Example 1: Simple Arithmetic

**Source:** `6 * 7`

**Bytecode:**
```
# Header (20 bytes)
56 45 4C 41          # Magic "VELA"
01 00 00 00          # Version 1
[8 bytes timestamp]  # Timestamp
01 00 00 00          # 1 code object

# Code Object "main"
04 00 00 00          # Name length = 4
6D 61 69 6E          # "main"
00 00 00 00          # 0 arguments
00 00                # local_count = 0
02 00                # stack_size = 2

# Bytecode (7 bytes)
07 00 00 00          # Bytecode length = 7
01 00 00             # LoadConst 0
01 01 00             # LoadConst 1
12                   # Multiply
41                   # Return

# Constants (2)
02 00 00 00          # 2 constants
02                   # Tag: Int
06 00 00 00 00 00 00 00  # 6 (i64)
02                   # Tag: Int
07 00 00 00 00 00 00 00  # 7 (i64)

# Names (0)
00 00 00 00          # 0 names
```

**Disassembly:**
```
0000: LoadConst 0 (6)
0003: LoadConst 1 (7)
0006: Multiply
0007: Return
```

---

### Example 2: Conditional (If-Else)

**Source:**
```python
if x > 5:
    result = 100
else:
    result = 200
```

**Bytecode:**
```
0000: LoadLocal 0        # Load x
0003: LoadConst 0        # Load 5
0006: GreaterThan        # x > 5?
0007: JumpIfFalse +7     # Jump to else if false
0012: LoadConst 1        # Load 100
0015: StoreLocal 1       # Store in result
0018: Jump +5            # Jump over else
0023: LoadConst 2        # Load 200
0026: StoreLocal 1       # Store in result
0029: Return
```

---

### Example 3: Loop (Iterative Factorial)

**Source:**
```python
# fact(n) = n * (n-1) * ... * 1
result = 1
while n > 1:
    result = result * n
    n = n - 1
```

**Bytecode:**
```
0000: LoadConst 0        # Load 1
0003: StoreLocal 0       # result = 1

# Loop start
0006: LoadLocal 1        # Load n
0009: LoadConst 0        # Load 1
0012: GreaterThan        # n > 1?
0013: JumpIfFalse +20    # Exit loop if false

# Loop body
0018: LoadLocal 0        # Load result
0021: LoadLocal 1        # Load n
0024: Multiply           # result * n
0025: StoreLocal 0       # Store in result
0028: LoadLocal 1        # Load n
0031: LoadConst 0        # Load 1
0034: Subtract           # n - 1
0035: StoreLocal 1       # Store in n
0038: Jump -32           # Jump back to loop start

# After loop
0043: LoadLocal 0        # Load result
0046: Return
```

---

## Appendix

### A. Instruction Size Table

| Instruction | Size (bytes) |
|-------------|--------------|
| `Nop` | 1 |
| `LoadConst` | 3 (1 + u16) |
| `LoadNull` | 1 |
| `LoadLocal` | 3 (1 + u16) |
| `StoreLocal` | 3 (1 + u16) |
| `Pop` | 1 |
| `Dup` | 1 |
| `Add` | 1 |
| `Multiply` | 1 |
| `Jump` | 5 (1 + i32) |
| `JumpIfFalse` | 5 (1 + i32) |
| `Call` | 3 (1 + u16) |
| `Return` | 1 |

### B. Type Compatibility Matrix

| Operation | Int × Int | Float × Float | Int × Float | Bool × Bool |
|-----------|-----------|---------------|-------------|-------------|
| Add | ✅ Int | ✅ Float | ❌ TypeError | ❌ TypeError |
| Multiply | ✅ Int | ✅ Float | ❌ TypeError | ❌ TypeError |
| Equals | ✅ Bool | ✅ Bool | ❌ TypeError | ✅ Bool |
| LessThan | ✅ Bool | ✅ Bool | ❌ TypeError | ❌ TypeError |

### C. Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `DivisionByZero` | 0x01 | Division or modulo by zero |
| `StackUnderflow` | 0x02 | Pop from empty stack |
| `StackOverflow` | 0x03 | Stack exceeds max size |
| `TypeError` | 0x04 | Incompatible types for operation |
| `InvalidOpcode` | 0x05 | Unknown instruction opcode |
| `InvalidConstantIndex` | 0x06 | Out-of-bounds constant access |
| `InvalidLocalIndex` | 0x07 | Out-of-bounds local access |
| `InvalidGlobalIndex` | 0x08 | Out-of-bounds global access |
| `InvalidJumpTarget` | 0x09 | Jump to invalid address |
| `UndefinedVariable` | 0x0A | Global variable not defined |
| `CallFrameError` | 0x0B | Call stack corruption |

### D. Magic Numbers

| Constant | Value | Description |
|----------|-------|-------------|
| `MAGIC` | `[0x56, 0x45, 0x4C, 0x41]` | "VELA" in ASCII |
| `VERSION` | `1` | Current format version |
| `MAX_STACK_SIZE` | `1024` | Maximum stack depth |
| `MAX_CALL_DEPTH` | `100` | Maximum call nesting |

---

**Document Version:** 1.0  
**Last Updated:** 2025-01-30  
**Author:** Vela Development Team

For questions or suggestions, open an issue on GitHub: https://github.com/velalang/vela/issues
