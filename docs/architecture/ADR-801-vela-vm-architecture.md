# ADR-801: Vela VM Architecture

## Estado
✅ Aceptado

## Fecha
2025-12-03

## Contexto

Sprint 9 de la migración de Vela a Rust requiere implementar una Virtual Machine (VM) completa para ejecutar bytecode de Vela. La VM es el componente central del runtime y debe ser:

- **Rápida**: Performance comparable o superior a CPython
- **Segura**: Memory safety garantizada por Rust
- **Eficiente**: Bajo overhead de memoria y CPU
- **Debuggeable**: Soporte para debugging y profiling
- **Extensible**: Fácil agregar nuevas instrucciones

### Decisiones Clave

1. **Stack Machine vs Register Machine**: ¿Qué arquitectura usar?
2. **Bytecode Format**: ¿Cómo codificar instrucciones?
3. **Garbage Collector**: ¿Qué estrategia de GC?
4. **Optimizaciones**: ¿Direct threading? ¿Inline caching?

## Decisión

### 1. VM Architecture: Stack Machine

**Decisión**: Implementar una **stack-based VM** (como CPython, JVM).

**Razones**:
- ✅ **Simplicidad**: Instrucciones más simples (no necesitan especificar registros)
- ✅ **Bytecode compacto**: Menos bytes por instrucción
- ✅ **Portabilidad**: Fácil de implementar en cualquier arquitectura
- ✅ **Debugging**: Stack traces más claros
- ✅ **Familiaridad**: Similar a CPython, JVM, .NET CLR

**Trade-offs**:
- ❌ Más push/pop operations que register machine
- ❌ Menos eficiente que register VM (Lua, V8)

**Alternativa rechazada**: Register Machine (Lua-style)
- Más rápida pero más compleja
- Bytecode más grande (necesita especificar registros)
- Menos familiaridad para desarrolladores

### 2. Bytecode Format

#### Instruction Encoding

```rust
// Opcode: u8 (256 opcodes posibles)
pub type Opcode = u8;

// Instruction: variable length
pub enum Instruction {
    // Stack operations (0x00 - 0x0F)
    LoadConst(u16),      // 0x00 index: Push constant from pool
    LoadLocal(u16),      // 0x01 index: Push local variable
    StoreLocal(u16),     // 0x02 index: Pop to local variable
    LoadGlobal(u16),     // 0x03 index: Push global variable
    StoreGlobal(u16),    // 0x04 index: Pop to global variable
    LoadAttr(u16),       // 0x05 name_idx: Load attribute (obj.attr)
    StoreAttr(u16),      // 0x06 name_idx: Store attribute
    Pop,                 // 0x07: Pop top of stack
    Dup,                 // 0x08: Duplicate top of stack
    
    // Arithmetic (0x10 - 0x1F)
    Add,                 // 0x10: a + b
    Sub,                 // 0x11: a - b
    Mul,                 // 0x12: a * b
    Div,                 // 0x13: a / b
    Mod,                 // 0x14: a % b
    Pow,                 // 0x15: a ** b
    Neg,                 // 0x16: -a
    
    // Comparison (0x20 - 0x2F)
    Eq,                  // 0x20: a == b
    Ne,                  // 0x21: a != b
    Lt,                  // 0x22: a < b
    Le,                  // 0x23: a <= b
    Gt,                  // 0x24: a > b
    Ge,                  // 0x25: a >= b
    
    // Logical (0x30 - 0x3F)
    And,                 // 0x30: a && b
    Or,                  // 0x31: a || b
    Not,                 // 0x32: !a
    
    // Control flow (0x40 - 0x4F)
    Jump(i32),           // 0x40 offset: Unconditional jump
    JumpIfFalse(i32),    // 0x41 offset: Jump if top is false
    JumpIfTrue(i32),     // 0x42 offset: Jump if top is true
    
    // Functions (0x50 - 0x5F)
    Call(u8),            // 0x50 argc: Call function with N args
    Return,              // 0x51: Return from function
    MakeFunction(u16),   // 0x52 code_idx: Create function object
    MakeClosure(u16, u8),// 0x53 code_idx, free_vars: Create closure
    
    // Collections (0x60 - 0x6F)
    BuildList(u16),      // 0x60 size: Build list from N stack items
    BuildDict(u16),      // 0x61 size: Build dict from N*2 items
    BuildSet(u16),       // 0x62 size: Build set from N items
    BuildTuple(u16),     // 0x63 size: Build tuple from N items
    
    // Subscript (0x70 - 0x7F)
    LoadSubscript,       // 0x70: obj[key]
    StoreSubscript,      // 0x71: obj[key] = value
    DeleteSubscript,     // 0x72: del obj[key]
    
    // Iteration (0x80 - 0x8F)
    GetIter,             // 0x80: Get iterator
    ForIter(i32),        // 0x81 offset: Iterate (jump if exhausted)
    
    // Exception handling (0x90 - 0x9F)
    SetupExcept(i32),    // 0x90 handler_offset: Setup exception handler
    PopExcept,           // 0x91: Pop exception handler
    Raise,               // 0x92: Raise exception
    
    // Imports (0xA0 - 0xAF)
    ImportName(u16),     // 0xA0 name_idx: Import module
    ImportFrom(u16),     // 0xA1 name_idx: Import from module
    
    // Debug (0xF0 - 0xFF)
    Nop,                 // 0xF0: No operation
    Breakpoint,          // 0xF1: Debugger breakpoint
}
```

#### Bytecode File Format

```
┌──────────────────────────────────────┐
│ Header (16 bytes)                    │
├──────────────────────────────────────┤
│ Magic Number: 0x56454C41 (VELA)      │ 4 bytes
│ Version: Major.Minor.Patch           │ 3 bytes
│ Flags: 0bxxxxxxxx                    │ 1 byte
│ Timestamp: u64                       │ 8 bytes
├──────────────────────────────────────┤
│ Constant Pool                        │
├──────────────────────────────────────┤
│ String Table                         │
├──────────────────────────────────────┤
│ Code Objects                         │
├──────────────────────────────────────┤
│ Metadata (optional)                  │
└──────────────────────────────────────┘
```

**Constant Pool**:
```rust
pub enum Constant {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(u16),       // Index to string table
    Code(u16),         // Index to code object
}
```

**Code Object**:
```rust
pub struct CodeObject {
    pub name: u16,              // String table index
    pub filename: u16,          // String table index
    pub arg_count: u16,
    pub local_count: u16,
    pub stack_size: u16,        // Max stack depth
    pub flags: u16,             // IS_GENERATOR, etc.
    pub bytecode: Vec<u8>,
    pub constants: Vec<Constant>,
    pub names: Vec<u16>,        // Variable names
    pub line_numbers: Vec<(u32, u32)>, // (bytecode_offset, line_number)
}
```

### 3. Garbage Collector

**Decisión**: Hybrid GC con **Reference Counting + Cycle Detection**

**Estrategia**:

1. **Reference Counting (RC)**:
   - Cada objeto tiene un contador de referencias (`Rc<RefCell<T>>`)
   - Inmediato: objetos se liberan cuando rc=0
   - Bajo overhead en runtime

2. **Cycle Detection**:
   - Tracing GC periódico para detectar ciclos
   - Mark-and-sweep sobre objetos sospechosos
   - Ejecuta cuando memoria > threshold

3. **Generational GC** (Phase 2):
   - Young generation: objetos nuevos (rc bajo)
   - Old generation: objetos longevos
   - Minor GC frecuente, Major GC ocasional

**Implementación**:

```rust
pub struct GcHeap {
    // Reference counted objects
    objects: Vec<Rc<RefCell<GcObject>>>,
    
    // Cycle detection
    cycle_buffer: Vec<*mut GcObject>,
    
    // Statistics
    allocated: usize,
    threshold: usize,
    collections: u64,
}

pub enum GcObject {
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Rc<RefCell<GcObject>>>),
    Dict(HashMap<String, Rc<RefCell<GcObject>>>),
    Function(FunctionObject),
    Closure(ClosureObject),
}
```

**Razones**:
- ✅ RC: Deterministic (objetos se liberan inmediatamente)
- ✅ RC: Compatible con Rust ownership (Rc/Arc)
- ✅ Cycle detection: Maneja referencias circulares
- ✅ Generational: Optimiza objetos short-lived

**Trade-offs**:
- ❌ RC overhead: incremento/decremento en cada asignación
- ❌ Ciclos: necesita tracing GC periódico

**Alternativas rechazadas**:
- **Solo RC**: No maneja ciclos (memory leaks)
- **Solo Tracing GC**: Stop-the-world pauses (menos determinístico)

### 4. VM Interpreter

#### Fetch-Decode-Execute Loop

```rust
pub struct VirtualMachine {
    // Execution state
    frames: Vec<CallFrame>,     // Call stack
    stack: Vec<Value>,          // Operand stack
    globals: HashMap<String, Value>,
    
    // Memory management
    heap: GcHeap,
    
    // Constants
    constants: Vec<Constant>,
    strings: Vec<String>,
    
    // Debugging
    breakpoints: HashSet<usize>,
    trace: bool,
}

impl VirtualMachine {
    pub fn execute(&mut self, code: &CodeObject) -> Result<Value> {
        let mut frame = CallFrame::new(code);
        self.frames.push(frame);
        
        loop {
            // Fetch
            let opcode = self.fetch_byte(&mut frame)?;
            
            // Decode
            let instruction = Instruction::decode(opcode, &mut frame)?;
            
            // Execute
            match instruction {
                Instruction::LoadConst(idx) => {
                    let value = self.constants[idx as usize].clone();
                    self.stack.push(value);
                }
                
                Instruction::Add => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(a + b);
                }
                
                Instruction::Call(argc) => {
                    let func = self.stack.pop()?;
                    let args = self.stack.drain(self.stack.len() - argc..);
                    self.call(func, args.collect())?;
                }
                
                Instruction::Return => {
                    let result = self.stack.pop()?;
                    self.frames.pop();
                    if self.frames.is_empty() {
                        return Ok(result);
                    }
                    self.stack.push(result);
                }
                
                // ... more instructions
            }
        }
    }
}
```

#### Call Frame

```rust
pub struct CallFrame {
    pub code: Rc<CodeObject>,
    pub ip: usize,              // Instruction pointer
    pub stack_base: usize,      // Base of stack for this frame
    pub locals: Vec<Value>,     // Local variables
}
```

### 5. Optimizations

#### Phase 1: Basic Optimizations

1. **Direct Threading** (computed goto):
   - Jump table de instrucciones
   - Elimina dispatch overhead
   - ~15% speedup

2. **Inline Caching**:
   - Cache de attribute lookups
   - Cache de method dispatch
   - ~30% speedup en código OOP

3. **Constant Folding**:
   - Pre-compute constant expressions
   - Compile-time optimization

#### Phase 2: Advanced Optimizations

4. **JIT Compilation** (usando cranelift):
   - Hot loop detection
   - JIT compile bytecode → native
   - ~10x speedup en loops

5. **Specialization**:
   - Specialized opcodes para tipos comunes
   - `AddInt`, `AddFloat` vs `Add`

6. **Register Allocation**:
   - Minimize stack operations
   - Inline small functions

### 6. Value Representation

```rust
// Tagged pointer: 64-bit value
// Layout:
// - Integers: [48 bits value][16 bits tag=0x0001]
// - Floats:   [64 bits] (NaN-boxing)
// - Pointers: [48 bits ptr][16 bits tag=0xFFFE]
// - Null:     0x0000000000000000
// - True:     0x0000000000000002
// - False:    0x0000000000000001

#[repr(transparent)]
pub struct Value(u64);

impl Value {
    pub fn from_int(n: i64) -> Self {
        // Tag with 0x0001
        Value((n as u64) << 16 | 0x0001)
    }
    
    pub fn from_float(f: f64) -> Self {
        // NaN-boxing
        Value(f.to_bits())
    }
    
    pub fn from_ptr(ptr: *mut GcObject) -> Self {
        // Tag with 0xFFFE
        Value((ptr as u64) | 0xFFFE)
    }
    
    pub fn is_int(&self) -> bool {
        self.0 & 0xFFFF == 0x0001
    }
    
    pub fn as_int(&self) -> i64 {
        (self.0 >> 16) as i64
    }
}
```

### 7. Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| **Startup time** | < 10ms | Cold start de VM |
| **Bytecode loading** | < 5ms/MB | Deserialización |
| **Function call** | < 50ns | overhead de call frame |
| **Attribute access** | < 20ns | con inline cache |
| **Arithmetic ops** | < 5ns | para ints/floats |
| **GC overhead** | < 5% | tiempo en GC vs ejecución |
| **Memory overhead** | < 2x | heap size vs Python |

**Benchmarks de referencia**:
- CPython 3.12: 1x (baseline)
- PyPy: 5-10x
- Lua 5.4: 3-5x
- Target: 3-8x vs CPython

## Consecuencias

### Positivas

1. **Stack Machine**: Bytecode compacto, debugging más fácil
2. **Hybrid GC**: Balance entre determinismo y ciclos
3. **Value tagging**: Memory efficient, fast type checks
4. **Direct threading**: Performance comparable a C interpreters

### Negativas

1. **Stack overhead**: Más push/pop que register VM
2. **RC overhead**: Incremento/decremento en cada operación
3. **Complexity**: Cycle detection agrega complejidad

## Alternativas Consideradas

### 1. Register Machine (Lua-style)
**Pros**: Más rápido (menos stack ops)  
**Cons**: Bytecode más grande, más complejo  
**Decisión**: Rechazado (simplicidad > velocidad inicial)

### 2. Tracing GC (solo mark-and-sweep)
**Pros**: Simple, maneja ciclos naturalmente  
**Cons**: Stop-the-world pauses, menos determinístico  
**Decisión**: Rechazado (queremos RC para determinismo)

### 3. Manual Memory Management
**Pros**: Máxima performance  
**Cons**: Unsafe, propenso a leaks  
**Decisión**: Rechazado (safety > raw speed)

### 4. JIT desde día 1
**Pros**: Máxima velocidad  
**Cons**: Muy complejo, tiempo de desarrollo  
**Decisión**: Postponed para Phase 2 (primero interpreter funcional)

## Referencias

### Papers & Specs

- **CPython VM**: https://github.com/python/cpython/blob/main/Python/ceval.c
- **JVM Spec**: https://docs.oracle.com/javase/specs/jvms/se17/html/
- **Lua 5.4**: https://www.lua.org/manual/5.4/
- **WebAssembly**: https://webassembly.github.io/spec/core/
- **V8 Ignition**: https://v8.dev/blog/ignition-interpreter

### Artículos

- "A Simple Stack-Based Virtual Machine" - Eli Bendersky
- "Efficient Implementation of the Smalltalk-80 System" - L. Peter Deutsch
- "Direct Threaded Code" - Bell (1973)
- "NaN Boxing or How to Make the World Dynamic" - Sean Barrett

### Implementaciones de Referencia

- **Crafting Interpreters**: https://craftinginterpreters.com/
- **WebAssembly VM**: https://github.com/bytecodealliance/wasmtime
- **Cranelift**: https://github.com/bytecodealliance/wasmtime/tree/main/cranelift

## Implementación

### Estructura del Crate

```
vm/
├── src/
│   ├── lib.rs              # Crate root
│   ├── bytecode/
│   │   ├── mod.rs          # Bytecode module
│   │   ├── instruction.rs  # Instruction definitions
│   │   ├── constant.rs     # Constant pool
│   │   ├── code_object.rs  # Code object
│   │   └── serialization.rs# Bytecode serialization
│   ├── vm/
│   │   ├── mod.rs          # VM module
│   │   ├── machine.rs      # VirtualMachine
│   │   ├── frame.rs        # CallFrame
│   │   ├── stack.rs        # Operand stack
│   │   └── value.rs        # Value representation
│   ├── gc/
│   │   ├── mod.rs          # GC module
│   │   ├── heap.rs         # GcHeap
│   │   ├── object.rs       # GcObject
│   │   ├── rc.rs           # Reference counting
│   │   └── cycle.rs        # Cycle detection
│   ├── error.rs            # Error types
│   └── utils.rs            # Utilities
├── tests/
│   ├── bytecode_tests.rs
│   ├── vm_tests.rs
│   └── gc_tests.rs
├── benches/
│   ├── vm_bench.rs
│   └── gc_bench.rs
├── Cargo.toml
└── README.md
```

### Dependencies

```toml
[dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"              # Bytecode serialization

# Collections
indexmap = "2.0"             # Ordered maps for constant pool
smallvec = "1.11"            # Stack-allocated small vectors

# Error handling
thiserror = "1.0"
miette = { version = "7.0", features = ["fancy"] }

# Hashing
ahash = "0.8"                # Fast non-cryptographic hashing

# Testing
criterion = "0.5"            # Benchmarking

[dev-dependencies]
proptest = "1.0"             # Property-based testing
```

### Roadmap

**Sprint 9 (Current)**:
- [x] ADR-801: Architecture design
- [ ] Bytecode format implementation
- [ ] Basic VM interpreter
- [ ] Reference counting GC
- [ ] Tests (unit + integration)
- [ ] Benchmarks vs CPython

**Future Work**:
- [ ] Inline caching
- [ ] Direct threading
- [ ] Cycle detection GC
- [ ] Generational GC
- [ ] JIT compilation (cranelift)
- [ ] Debugger support
- [ ] Profiler integration

## Notas

- Esta arquitectura prioriza **simplicidad y seguridad** sobre raw performance
- Stack machine facilita debugging y portabilidad
- Hybrid GC (RC + cycles) es un buen balance
- JIT compilation se postpone para Phase 2
- Performance target: 3-8x vs CPython (alcanzable con optimizaciones)

---

**Aprobado por**: VM Team  
**Revisado por**: Architecture Team  
**Fecha de implementación**: Sprint 9 (2025-12-03)
