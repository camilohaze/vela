# ADR-069: Bytecode Instruction Set Architecture (ISA)

## Estado
✅ Aceptado

## Fecha
2025-12-07

## Contexto

VelaVM necesita un conjunto de instrucciones bytecode completo para ejecutar programas Vela compilados. Este ISA debe:

1. **Ser stack-based** (como JVM, Python VM) en lugar de register-based (como Lua, BEAM)
2. **Soportar todas las operaciones del lenguaje** (aritméticas, lógicas, control de flujo, funciones, objetos)
3. **Ser compacto** para minimizar tamaño de archivos compilados
4. **Ser rápido de interpretar** con un ciclo fetch-decode-execute eficiente
5. **Prepararse para JIT futuro** con información de tipos inline
6. **Soportar reactividad** (signals, computed, effects)
7. **Soportar concurrencia** (actors, async/await)

### Decisiones de diseño previas

- **Memoria**: ARC (Automatic Reference Counting) para gestión de memoria
- **Concurrencia**: Actor model con message passing
- **Reactividad**: Sistema de signals integrado
- **Tipos**: Type system híbrido (estático + inferencia)

## Decisión

Diseñamos un **ISA stack-based de 256 instrucciones máximo** (1 byte por opcode) con las siguientes categorías:

### 1. Stack Operations (0x00 - 0x0F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x00 | `NOP` | - | No operation (padding/debugging) |
| 0x01 | `POP` | - | Pop top of stack |
| 0x02 | `DUP` | - | Duplicate top of stack |
| 0x03 | `DUP2` | - | Duplicate top 2 values |
| 0x04 | `SWAP` | - | Swap top 2 values |
| 0x05 | `ROT3` | - | Rotate top 3 values (a, b, c → c, a, b) |

### 2. Constants (0x10 - 0x1F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x10 | `CONST_NONE` | - | Push None |
| 0x11 | `CONST_TRUE` | - | Push true |
| 0x12 | `CONST_FALSE` | - | Push false |
| 0x13 | `CONST_0` | - | Push 0 |
| 0x14 | `CONST_1` | - | Push 1 |
| 0x15 | `CONST_NEG1` | - | Push -1 |
| 0x16 | `LOAD_CONST` | u16 idx | Load constant from constant pool |
| 0x17 | `LOAD_CONST_SMALL` | u8 value | Load small integer (-128..127) |

### 3. Arithmetic Operations (0x20 - 0x2F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x20 | `ADD` | - | a + b |
| 0x21 | `SUB` | - | a - b |
| 0x22 | `MUL` | - | a * b |
| 0x23 | `DIV` | - | a / b (float division) |
| 0x24 | `IDIV` | - | a // b (integer division) |
| 0x25 | `MOD` | - | a % b |
| 0x26 | `POW` | - | a ** b |
| 0x27 | `NEG` | - | -a (unary minus) |
| 0x28 | `ABS` | - | abs(a) |
| 0x29 | `INC` | - | a + 1 (optimized) |
| 0x2A | `DEC` | - | a - 1 (optimized) |

### 4. Bitwise Operations (0x30 - 0x3F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x30 | `BIT_AND` | - | a & b |
| 0x31 | `BIT_OR` | - | a \| b |
| 0x32 | `BIT_XOR` | - | a ^ b |
| 0x33 | `BIT_NOT` | - | ~a |
| 0x34 | `SHL` | - | a << b |
| 0x35 | `SHR` | - | a >> b (arithmetic right shift) |
| 0x36 | `USHR` | - | a >>> b (logical right shift) |

### 5. Comparison Operations (0x40 - 0x4F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x40 | `EQ` | - | a == b |
| 0x41 | `NE` | - | a != b |
| 0x42 | `LT` | - | a < b |
| 0x43 | `LE` | - | a <= b |
| 0x44 | `GT` | - | a > b |
| 0x45 | `GE` | - | a >= b |
| 0x46 | `IS_NONE` | - | a is None |
| 0x47 | `IS_NOT_NONE` | - | a is not None |
| 0x48 | `IS_TYPE` | u8 type | Check runtime type |

### 6. Logical Operations (0x50 - 0x5F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x50 | `LOGIC_AND` | - | a && b (short-circuit via jumps) |
| 0x51 | `LOGIC_OR` | - | a \|\| b (short-circuit via jumps) |
| 0x52 | `LOGIC_NOT` | - | !a |

### 7. Control Flow (0x60 - 0x6F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x60 | `JUMP` | i16 offset | Unconditional jump |
| 0x61 | `JUMP_IF_TRUE` | i16 offset | Jump if top is truthy |
| 0x62 | `JUMP_IF_FALSE` | i16 offset | Jump if top is falsy |
| 0x63 | `JUMP_IF_NONE` | i16 offset | Jump if top is None |
| 0x64 | `JUMP_FORWARD` | u8 offset | Short forward jump (optimization) |
| 0x65 | `LOOP` | i16 offset | Loop back (negative jump) |
| 0x66 | `MATCH_JUMP_TABLE` | u16 table_idx | Pattern matching jump table |

### 8. Variables (0x70 - 0x7F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x70 | `LOAD_LOCAL` | u8 idx | Load local variable |
| 0x71 | `STORE_LOCAL` | u8 idx | Store to local variable |
| 0x72 | `LOAD_GLOBAL` | u16 name_idx | Load global variable |
| 0x73 | `STORE_GLOBAL` | u16 name_idx | Store to global variable |
| 0x74 | `LOAD_UPVALUE` | u8 idx | Load captured variable (closures) |
| 0x75 | `STORE_UPVALUE` | u8 idx | Store to captured variable |
| 0x76 | `LOAD_FAST` | u8 idx | Optimized local load (hot path) |
| 0x77 | `STORE_FAST` | u8 idx | Optimized local store (hot path) |

### 9. Function Calls (0x80 - 0x8F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x80 | `CALL` | u8 argc | Call function with argc arguments |
| 0x81 | `CALL_METHOD` | u8 argc, u16 name_idx | Call method on object |
| 0x82 | `RETURN` | - | Return from function |
| 0x83 | `RETURN_NONE` | - | Return None (optimization) |
| 0x84 | `TAIL_CALL` | u8 argc | Tail-call optimization |
| 0x85 | `CALL_BUILTIN` | u8 builtin_id, u8 argc | Call built-in function |
| 0x86 | `CALL_ASYNC` | u8 argc | Async function call |
| 0x87 | `AWAIT` | - | Await promise/future |

### 10. Object Operations (0x90 - 0x9F)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0x90 | `NEW_OBJECT` | u16 class_idx | Create new object instance |
| 0x91 | `GET_ATTR` | u16 name_idx | Get object attribute |
| 0x92 | `SET_ATTR` | u16 name_idx | Set object attribute |
| 0x93 | `GET_METHOD` | u16 name_idx | Get method (bound) |
| 0x94 | `LOAD_THIS` | - | Load `this` reference |
| 0x95 | `LOAD_SUPER` | - | Load `super` reference |
| 0x96 | `DELETE_ATTR` | u16 name_idx | Delete attribute |
| 0x97 | `HAS_ATTR` | u16 name_idx | Check if attribute exists |

### 11. Collections (0xA0 - 0xAF)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0xA0 | `BUILD_LIST` | u8 size | Build list from top N stack values |
| 0xA1 | `BUILD_MAP` | u8 size | Build map from top 2N stack values |
| 0xA2 | `BUILD_SET` | u8 size | Build set from top N stack values |
| 0xA3 | `BUILD_TUPLE` | u8 size | Build immutable tuple |
| 0xA4 | `LIST_APPEND` | - | Append to list |
| 0xA5 | `MAP_INSERT` | - | Insert key-value to map |
| 0xA6 | `GET_ITEM` | - | collection[index] |
| 0xA7 | `SET_ITEM` | - | collection[index] = value |
| 0xA8 | `DELETE_ITEM` | - | delete collection[index] |
| 0xA9 | `GET_SLICE` | - | collection[start:end] |
| 0xAA | `CONTAINS` | - | key in collection |
| 0xAB | `LIST_EXTEND` | - | list1.extend(list2) |

### 12. String Operations (0xB0 - 0xBF)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0xB0 | `BUILD_STRING` | u8 count | Concatenate top N strings |
| 0xB1 | `FORMAT_STRING` | u8 parts | String interpolation ${} |
| 0xB2 | `STRING_CONCAT` | - | a + b (string concat) |
| 0xB3 | `STRING_REPEAT` | - | string * n |
| 0xB4 | `STRING_SLICE` | - | string[start:end] |

### 13. Type Operations (0xC0 - 0xCF)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0xC0 | `CAST` | u8 type | Type cast |
| 0xC1 | `INSTANCEOF` | u16 class_idx | Check instanceof |
| 0xC2 | `TYPEOF` | - | Get type of value |
| 0xC3 | `AS_NUMBER` | - | Convert to Number |
| 0xC4 | `AS_STRING` | - | Convert to String |
| 0xC5 | `AS_BOOL` | - | Convert to Bool |

### 14. Closures & Functions (0xD0 - 0xDF)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0xD0 | `MAKE_CLOSURE` | u16 fn_idx, u8 upvalue_count | Create closure |
| 0xD1 | `LOAD_CLOSURE` | u8 idx | Load closure from locals |
| 0xD2 | `CLOSE_UPVALUE` | u8 idx | Close over variable |
| 0xD3 | `MAKE_GENERATOR` | u16 fn_idx | Create generator function |
| 0xD4 | `YIELD` | - | Yield value (generator) |
| 0xD5 | `YIELD_FROM` | - | Yield from another generator |

### 15. Exception Handling (0xE0 - 0xEF)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0xE0 | `THROW` | - | Throw exception |
| 0xE1 | `TRY_BEGIN` | u16 catch_offset | Begin try block |
| 0xE2 | `TRY_END` | - | End try block |
| 0xE3 | `CATCH` | u16 name_idx | Catch exception |
| 0xE4 | `FINALLY` | u16 offset | Finally block |
| 0xE5 | `RERAISE` | - | Re-throw caught exception |

### 16. Reactive System (0xF0 - 0xF7)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0xF0 | `SIGNAL_CREATE` | - | Create reactive signal |
| 0xF1 | `SIGNAL_READ` | - | Read signal value (subscribe) |
| 0xF2 | `SIGNAL_WRITE` | - | Write signal value (notify) |
| 0xF3 | `COMPUTED_CREATE` | u16 fn_idx | Create computed signal |
| 0xF4 | `EFFECT_CREATE` | u16 fn_idx | Create effect |
| 0xF5 | `WATCH_CREATE` | u16 fn_idx | Create watcher |
| 0xF6 | `BATCH_UPDATE_BEGIN` | - | Begin batch update |
| 0xF7 | `BATCH_UPDATE_END` | - | End batch update |

### 17. Actor System (0xF8 - 0xFD)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0xF8 | `ACTOR_SPAWN` | u16 actor_idx | Spawn new actor |
| 0xF9 | `ACTOR_SEND` | - | Send message to actor |
| 0xFA | `ACTOR_RECEIVE` | - | Receive message (blocking) |
| 0xFB | `ACTOR_SELF` | - | Get current actor reference |
| 0xFC | `ACTOR_LINK` | - | Link two actors |
| 0xFD | `ACTOR_MONITOR` | - | Monitor actor |

### 18. Debug & Metadata (0xFE - 0xFF)

| Opcode | Mnemonic | Operands | Descripción |
|--------|----------|----------|-------------|
| 0xFE | `DEBUG_LINE` | u16 line_num | Source line number (debugging) |
| 0xFF | `HALT` | - | Stop execution (end of program) |

## Bytecode Format

### Instruction Encoding

```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│  Opcode     │  Operand 1  │  Operand 2  │  Operand 3  │
│  (1 byte)   │  (0-2 bytes)│  (0-2 bytes)│  (0-2 bytes)│
└─────────────┴─────────────┴─────────────┴─────────────┘
```

**Tamaños de instrucción**:
- 1 byte: Opcode solo (ej: `ADD`, `POP`, `RETURN`)
- 2 bytes: Opcode + u8 operand (ej: `LOAD_LOCAL 5`)
- 3 bytes: Opcode + u16 operand (ej: `LOAD_CONST 1000`)
- 4 bytes: Opcode + 2 operands (ej: `CALL_METHOD argc, name_idx`)

### Constant Pool

El bytecode incluye un **constant pool** al inicio del archivo:

```
┌──────────────────────────────────────┐
│        MAGIC NUMBER (4 bytes)         │  0x56454C41 ("VELA")
├──────────────────────────────────────┤
│        VERSION (2 bytes)              │  Major.Minor
├──────────────────────────────────────┤
│   CONSTANT POOL SIZE (2 bytes)       │  Number of constants
├──────────────────────────────────────┤
│   CONSTANT POOL ENTRIES              │
│   ┌──────────────────────────────┐   │
│   │ Type Tag (1 byte)            │   │
│   │ Data (variable length)       │   │
│   └──────────────────────────────┘   │
│   ... (repeated)                     │
├──────────────────────────────────────┤
│   CODE SIZE (4 bytes)                │  Size of bytecode
├──────────────────────────────────────┤
│   BYTECODE INSTRUCTIONS              │  Actual instructions
└──────────────────────────────────────┘
```

**Tipos de constantes**:
- `0x01`: None
- `0x02`: Bool
- `0x03`: Number (64-bit integer)
- `0x04`: Float (64-bit double)
- `0x05`: String (UTF-8)
- `0x06`: Function (bytecode offset + metadata)
- `0x07`: Class (metadata)
- `0x08`: Type (type descriptor)

## Ejemplo de Compilación

### Código Vela:
```vela
fn factorial(n: Number) -> Number {
  if n <= 1 {
    return 1
  }
  return n * factorial(n - 1)
}

result = factorial(5)
```

### Bytecode generado:
```
# Constant Pool:
# [0] = Function "factorial" (offset: 10)
# [1] = Number 1
# [2] = String "factorial"

# Code Section:
# factorial function (offset 10):
0000: LOAD_LOCAL 0        # Load n
0002: CONST_1             # Push 1
0003: LE                  # n <= 1
0004: JUMP_IF_FALSE 8     # If false, jump to else
0006: CONST_1             # Return 1
0007: RETURN
0008: LOAD_LOCAL 0        # Load n
0010: LOAD_LOCAL 0        # Load n again
0012: CONST_1             # Push 1
0013: SUB                 # n - 1
0014: LOAD_GLOBAL 2       # Load "factorial"
0017: CALL 1              # Call factorial(n-1)
0019: MUL                 # n * result
0020: RETURN

# Main code:
0021: LOAD_CONST 0        # Load factorial function
0024: CONST_5             # Push 5
0025: CALL 1              # Call factorial(5)
0027: STORE_GLOBAL 3      # Store to "result"
0030: HALT
```

## Consecuencias

### Positivas

1. **Stack-based simplicity**: Más fácil de implementar que register-based
2. **Compacto**: Instrucciones de 1-4 bytes
3. **Completo**: Soporta todas las features de Vela
4. **Extensible**: Espacio para 256 opcodes (actualmente ~120 usados)
5. **Optimizable**: Instrucciones especiales (CONST_0, LOAD_FAST, TAIL_CALL)
6. **Preparado para JIT**: Información de tipos inline
7. **Reactive-first**: Instrucciones dedicadas para signals
8. **Actor-ready**: Instrucciones para concurrencia

### Negativas

1. **Overhead de stack**: Más push/pop que register-based
2. **Sin SIMD todavía**: Requiere extensiones futuras
3. **Tamaño de bytecode**: Más grande que WASM (pero más pequeño que JVM)

### Trade-offs

- **Stack-based vs Register-based**: Elegimos stack por simplicidad de implementación inicial
- **1 byte opcodes vs 2 bytes**: Elegimos 1 byte (256 opcodes suficientes para Vela 1.0)
- **Inline types vs Type table**: Elegimos constant pool (más compacto)

## Alternativas Consideradas

### 1. Register-based VM (como Lua, BEAM)
- **PRO**: ~30% más rápido en interpreters
- **CONTRA**: Más complejo de implementar, más difícil de generar código correcto
- **Decisión**: RECHAZADA - Complejidad no vale la pena para v1.0

### 2. Threaded code (como FORTH)
- **PRO**: Muy rápido en interpreters puros
- **CONTRA**: No portable, dificulta JIT futuro
- **Decisión**: RECHAZADA - No compatible con goals de portabilidad

### 3. WASM como target
- **PRO**: Ecosistema existente, optimizaciones gratis
- **CONTRA**: No soporta signals/actors nativamente, requiere polyfills pesados
- **Decisión**: RECHAZADA para VM nativo - Mantenemos WASM como target alternativo

### 4. JVM bytecode directo
- **PRO**: Ecosistema Java gigante
- **CONTRA**: Impedance mismatch con features de Vela (signals, actors)
- **Decisión**: RECHAZADA - Mantenemos JVM como target alternativo

## Referencias

- **Jira**: US-16, TASK-069
- **Epic**: EPIC-06: Compiler Backend (VelaVM)
- **Sprint**: 23
- **Inspiraciones**:
  - Python bytecode (CPython 3.11+)
  - JVM bytecode (Java Virtual Machine)
  - Lua bytecode (Lua 5.4)
  - BEAM bytecode (Erlang VM)
  - V8 Ignition bytecode

## Implementación

- **ADR**: `docs/architecture/ADR-069-bytecode-instruction-set.md` (este archivo)
- **Código fuente**: `vm/opcodes.vela` (definiciones de constantes)
- **Tests**: `tests/unit/vm/test_opcodes.vela`
- **Documentación**: `docs/features/US-16/TASK-069.md`

## Notas de Implementación

### Optimizaciones Futuras (Post-1.0)

1. **Superinstructions**: Fusionar instrucciones comunes (ej: `LOAD_LOCAL + ADD` → `LOCAL_ADD`)
2. **Quickening**: Reemplazar instrucciones genéricas por especializadas en hot paths
3. **Inline caching**: Cache de method lookups
4. **JIT compilation**: Compilar hot paths a código nativo
5. **SIMD instructions**: Vectorización para operaciones numéricas

### Consideraciones de Performance

- **Hot instructions** (TOP 10 más frecuentes):
  1. `LOAD_LOCAL` / `STORE_LOCAL`
  2. `CALL`
  3. `RETURN`
  4. `ADD` / `SUB`
  5. `GET_ATTR` / `SET_ATTR`
  6. `JUMP_IF_FALSE`
  7. `LOAD_CONST`
  8. `EQ` / `LT`
  9. `SIGNAL_READ` / `SIGNAL_WRITE`
  10. `BUILD_LIST`

Estas instrucciones deben ser optimizadas al máximo en el interpreter loop.

### Memory Layout

**Stack frame structure**:
```
┌──────────────────────┐  ← Frame Pointer (FP)
│  Return Address      │
├──────────────────────┤
│  Previous FP         │
├──────────────────────┤
│  Function Object     │
├──────────────────────┤
│  Argument Count      │
├──────────────────────┤
│  Arguments (N)       │
├──────────────────────┤
│  Local Variables (M) │
├──────────────────────┤
│  Temp Stack (grows→) │
└──────────────────────┘  ← Stack Pointer (SP)
```

---

**Este ADR define el foundation completo de VelaVM. Todas las decisiones posteriores de codegen y runtime se basan en este ISA.**
