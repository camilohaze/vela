# Vela Operator Precedence and Associativity

**Version:** 0.1.0 (Phase 0 - Sprint 4)  
**Status:** Draft  
**Date:** 2025-11-30

## Overview

This document defines the complete operator precedence and associativity rules for the Vela programming language. Precedence determines the order of evaluation when multiple operators appear in an expression without explicit parentheses.

## Precedence Table

The following table lists all operators from **lowest** to **highest** precedence. Operators at the same level have equal precedence and are evaluated according to their associativity.

| Level | Operators | Category | Associativity | Example |
|-------|-----------|----------|---------------|---------|
| 1 | `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `**=`, `&=`, `\|=`, `^=`, `<<=`, `>>=` | Assignment | **Right** | `a = b = c` → `a = (b = c)` |
| 2 | `\|\|` | Logical OR | **Left** | `a \|\| b \|\| c` → `(a \|\| b) \|\| c` |
| 3 | `&&` | Logical AND | **Left** | `a && b && c` → `(a && b) && c` |
| 4 | `??` | Null Coalescing | **Left** | `a ?? b ?? c` → `(a ?? b) ?? c` |
| 5 | `==`, `!=` | Equality | **Left** | `a == b != c` → `(a == b) != c` |
| 6 | `<`, `>`, `<=`, `>=` | Comparison | **Left** | `a < b < c` → `(a < b) < c` |
| 7 | `\|` | Bitwise OR | **Left** | `a \| b \| c` → `(a \| b) \| c` |
| 8 | `^` | Bitwise XOR | **Left** | `a ^ b ^ c` → `(a ^ b) ^ c` |
| 9 | `&` | Bitwise AND | **Left** | `a & b & c` → `(a & b) & c` |
| 10 | `<<`, `>>` | Bitwise Shift | **Left** | `a << b << c` → `(a << b) << c` |
| 11 | `+`, `-` | Addition, Subtraction | **Left** | `a + b - c` → `(a + b) - c` |
| 12 | `*`, `/`, `%` | Multiplication, Division, Modulo | **Left** | `a * b / c` → `(a * b) / c` |
| 13 | `**` | Exponentiation | **Right** | `a ** b ** c` → `a ** (b ** c)` |
| 14 | `-`, `!`, `~`, `*`, `&`, `&mut` | Unary | **Right** | `-!x` → `-(!(x))` |
| 15 | `()`, `[]`, `.`, `?.`, `?` | Postfix | **Left** | `a.b().c[0]` → `((a.b()).c)[0]` |

## Detailed Operator Groups

### 1. Assignment Operators (Precedence: 1, Right Associative)

**Operators:**
- `=` - Simple assignment
- `+=`, `-=`, `*=`, `/=`, `%=` - Arithmetic compound assignment
- `**=` - Exponentiation compound assignment
- `&=`, `|=`, `^=` - Bitwise compound assignment
- `<<=`, `>>=` - Shift compound assignment

**Associativity:** Right (right-to-left)

**Examples:**
```vela
// Right associativity
let x = y = z = 10;
// Parsed as: let x = (y = (z = 10));

// Compound assignment
x += 5;  // x = x + 5
y *= 2;  // y = y * 2
```

**Note:** Assignment is an expression that returns the assigned value.

---

### 2. Logical OR (Precedence: 2, Left Associative)

**Operator:** `||`

**Associativity:** Left (left-to-right)

**Short-circuit:** Yes (if left is `true`, right is not evaluated)

**Examples:**
```vela
a || b || c
// Parsed as: (a || b) || c

// Short-circuit behavior
is_valid() || throw Error("Invalid");
```

---

### 3. Logical AND (Precedence: 3, Left Associative)

**Operator:** `&&`

**Associativity:** Left (left-to-right)

**Short-circuit:** Yes (if left is `false`, right is not evaluated)

**Examples:**
```vela
a && b && c
// Parsed as: (a && b) && c

// Short-circuit behavior
user != null && user.is_active
```

---

### 4. Null Coalescing (Precedence: 4, Left Associative)

**Operator:** `??`

**Associativity:** Left (left-to-right)

**Semantics:** Returns right operand if left is `null`, otherwise returns left

**Examples:**
```vela
a ?? b ?? c
// Parsed as: (a ?? b) ?? c

// Usage
let name = user?.name ?? "Anonymous";
let port = config.port ?? 8080;
```

---

### 5. Equality (Precedence: 5, Left Associative)

**Operators:**
- `==` - Equal to
- `!=` - Not equal to

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a == b == c
// Parsed as: (a == b) == c
// Warning: This is rarely what you want!

// Typical usage
if x == 5 {
    // ...
}

if y != null {
    // ...
}
```

---

### 6. Comparison (Precedence: 6, Left Associative)

**Operators:**
- `<` - Less than
- `>` - Greater than
- `<=` - Less than or equal
- `>=` - Greater than or equal

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a < b < c
// Parsed as: (a < b) < c
// Warning: This is NOT chained comparison!

// Typical usage
if x < 10 {
    // ...
}

if y >= 0 && y <= 100 {
    // Range check (explicit)
}

// Use range patterns instead:
match y {
    0..=100 => { /* in range */ }
    _ => { /* out of range */ }
}
```

---

### 7. Bitwise OR (Precedence: 7, Left Associative)

**Operator:** `|`

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a | b | c
// Parsed as: (a | b) | c

// Usage
let flags = READ | WRITE | EXECUTE;
```

---

### 8. Bitwise XOR (Precedence: 8, Left Associative)

**Operator:** `^`

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a ^ b ^ c
// Parsed as: (a ^ b) ^ c

// Usage (swap without temp)
x = x ^ y;
y = x ^ y;
x = x ^ y;
```

---

### 9. Bitwise AND (Precedence: 9, Left Associative)

**Operator:** `&`

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a & b & c
// Parsed as: (a & b) & c

// Usage (mask)
let masked = value & 0xFF;
```

---

### 10. Bitwise Shift (Precedence: 10, Left Associative)

**Operators:**
- `<<` - Left shift
- `>>` - Right shift (arithmetic)

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a << b << c
// Parsed as: (a << b) << c

// Usage
let doubled = x << 1;  // Multiply by 2
let halved = x >> 1;   // Divide by 2
```

---

### 11. Addition / Subtraction (Precedence: 11, Left Associative)

**Operators:**
- `+` - Addition (also string concatenation)
- `-` - Subtraction

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a + b - c + d
// Parsed as: ((a + b) - c) + d

// String concatenation
let greeting = "Hello, " + name + "!";
```

---

### 12. Multiplication / Division / Modulo (Precedence: 12, Left Associative)

**Operators:**
- `*` - Multiplication
- `/` - Division
- `%` - Modulo (remainder)

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a * b / c % d
// Parsed as: ((a * b) / c) % d

// Typical usage
let area = width * height;
let half = total / 2;
let remainder = x % 10;
```

---

### 13. Exponentiation (Precedence: 13, Right Associative)

**Operator:** `**`

**Associativity:** Right (right-to-left)

**Examples:**
```vela
a ** b ** c
// Parsed as: a ** (b ** c)

// Typical usage
let squared = x ** 2;
let cubed = x ** 3;
let power_tower = 2 ** 3 ** 2;  // 2 ** (3 ** 2) = 2 ** 9 = 512
```

**Note:** Right associativity matches mathematical convention: $a^{b^c} = a^{(b^c)}$

---

### 14. Unary Operators (Precedence: 14, Right Associative)

**Operators:**
- `-` - Negation
- `!` - Logical NOT
- `~` - Bitwise NOT
- `*` - Dereference
- `&` - Borrow (immutable reference)
- `&mut` - Mutable borrow

**Associativity:** Right (right-to-left)

**Examples:**
```vela
-!x
// Parsed as: -(!(x))

~-x
// Parsed as: ~(-x)

*&x
// Parsed as: *(&x)  (identity)

// Typical usage
let negative = -value;
let inverted = !is_valid;
let bitwise_inv = ~flags;
let deref = *pointer;
let reference = &variable;
let mut_ref = &mut data;
```

---

### 15. Postfix Operators (Precedence: 15, Left Associative)

**Operators:**
- `()` - Function call
- `[]` - Index/Subscript
- `.` - Member access
- `?.` - Safe navigation
- `?` - Unwrap (error propagation)

**Associativity:** Left (left-to-right)

**Examples:**
```vela
a.b().c[0]?.d
// Parsed as: ((((a.b()).c)[0])?.d)

// Function call
result = calculate(a, b);

// Indexing
element = array[5];

// Member access
name = user.name;

// Safe navigation
email = user?.profile?.email;  // Returns null if any is null

// Error propagation
let file = open_file("data.txt")?;  // Returns early if error
```

---

## Interaction Examples

### Example 1: Mixed Arithmetic

```vela
let result = 2 + 3 * 4 ** 2 - 10 / 2;
```

**Parsing:**
1. `4 ** 2` → `16` (Exponentiation, highest precedence)
2. `3 * 16` → `48` (Multiplication)
3. `10 / 2` → `5` (Division, same level as multiplication)
4. `2 + 48` → `50` (Addition)
5. `50 - 5` → `45` (Subtraction, same level as addition)

**Result:** `45`

---

### Example 2: Logical Operators

```vela
let valid = a || b && c || d;
```

**Parsing:**
1. `b && c` (AND has higher precedence than OR)
2. `a || (b && c)` (Left OR)
3. `(a || (b && c)) || d` (Right OR)

**Result:** `(a || (b && c)) || d`

---

### Example 3: Assignment Chain

```vela
let x = y = z = 10;
```

**Parsing (right-to-left):**
1. `z = 10`
2. `y = (z = 10)` → `y = 10`
3. `x = (y = 10)` → `x = 10`

**Result:** All variables are `10`

---

### Example 4: Safe Navigation with Null Coalescing

```vela
let display_name = user?.profile?.name ?? "Anonymous";
```

**Parsing:**
1. `user?.profile` (Safe navigation)
2. `(user?.profile)?.name` (Safe navigation)
3. `((user?.profile)?.name) ?? "Anonymous"` (Null coalescing)

**Result:** Returns user's profile name, or "Anonymous" if any step is null

---

### Example 5: Complex Expression

```vela
let result = -a * b + c ** 2 / d && e || f;
```

**Parsing:**
1. `-a` (Unary negation)
2. `c ** 2` (Exponentiation)
3. `(-a) * b` (Multiplication)
4. `(c ** 2) / d` (Division)
5. `((-a) * b) + ((c ** 2) / d)` (Addition)
6. `(((-a) * b) + ((c ** 2) / d)) && e` (Logical AND)
7. `((((-a) * b) + ((c ** 2) / d)) && e) || f` (Logical OR)

**Result:** `((-a * b + c ** 2 / d) && e) || f`

---

## Special Cases

### 1. Chained Comparisons (NOT Supported)

**Invalid:**
```vela
a < b < c  // Does NOT mean "a < b AND b < c"
```

**Parsed as:**
```vela
(a < b) < c  // Compares boolean result with c!
```

**Correct way:**
```vela
a < b && b < c
```

---

### 2. Ternary Operator (Not in Vela)

Vela does NOT have `? :` ternary operator. Use `if` expressions instead:

```vela
// Other languages:
// result = condition ? value1 : value2;

// Vela:
let result = if condition { value1 } else { value2 };
```

---

### 3. Postfix `?` vs `??`

- `?` (postfix): Error propagation / unwrap
- `??` (infix): Null coalescing

```vela
// Error propagation
let file = open_file("data.txt")?;  // Returns early if error

// Null coalescing
let port = config.port ?? 8080;  // Use default if null
```

---

## Design Rationale

### Why Right Associative Exponentiation?

Matches mathematical convention: $2^{3^2} = 2^9 = 512$, not $(2^3)^2 = 8^2 = 64$

### Why Separate `??` from `||`?

- `||` is for boolean logic with short-circuiting
- `??` is specifically for null/undefined handling
- Different precedence levels prevent confusion

### Why No Chained Comparisons?

Explicit is better than implicit. `a < b && b < c` is clearer than `a < b < c`.

---

## Comparison with Other Languages

| Language | Precedence Levels | Notes |
|----------|-------------------|-------|
| Vela | 15 | Clean, predictable |
| C/C++ | 17 | Complex, error-prone |
| Rust | 14 | Similar to Vela |
| Python | 16 | Has chained comparisons |
| JavaScript | 20 | Very complex |
| Java | 16 | Similar to C |

---

**TASK:** TASK-002  
**Historia:** VELA-566 (US-01)  
**Sprint:** Sprint 4 (Phase 0)  
**Status:** Completed ✅  
**Date:** 2025-11-30
