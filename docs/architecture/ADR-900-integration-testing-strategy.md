# ADR-900: Integration & Testing Strategy

## Estado
✅ Aceptado

## Fecha
2025-12-04

## Contexto

**EPIC-RUST-10: Integration & Testing** marca la culminación de la migración Rust Phase 1. Después de completar Sprint 9 (VM & Bytecode), necesitamos una estrategia integral para:

1. **Integrar todos los crates** en un sistema coherente
2. **Verificar correctitud** con tests end-to-end
3. **Medir performance** vs baseline Python
4. **Garantizar memory safety** (zero leaks, zero unsafe)
5. **Documentar** la migración completa

### Estado Actual (Post Sprint 9)

**Crates Completados:**
- ✅ `vm` (VirtualMachine, Bytecode, GC) - 94% tests passing
- ✅ `tooling` (CLI, package manager) - básico
- ⚠️ `compiler` - stub, necesita Parser → Bytecode
- ⚠️ `stdlib` - stub, necesita implementación
- ⚠️ `runtime` - stub
- ⚠️ Other crates - stubs

**Limitaciones Actuales:**
- No hay parser Vela → Bytecode compiler
- No hay stdlib mínimo funcional
- No hay CLI principal que orqueste todo
- No hay tests end-to-end (source → bytecode → execution)

### Objetivos de Sprint 10

**Primarios:**
1. ✅ Integrar crates existentes en flujo coherente
2. ✅ Tests end-to-end (aunque sea con bytecode manual)
3. ✅ Performance benchmarks (VM vs CPython baseline)
4. ✅ Memory safety verification
5. ✅ Documentación de integración

**Secundarios (futuro):**
- Parser Vela → Bytecode (Sprint 11)
- Stdlib completo (Sprints 12-15)
- Python compatibility layer (Sprint 16+)

---

## Decisión

### 1. Arquitectura de Integración

#### 1.1 Workspace Structure

```
vela/
├── Cargo.toml              # Workspace root
├── vm/                     # ✅ DONE Sprint 9
│   ├── src/
│   │   ├── bytecode.rs     # Bytecode format + NaN-boxing
│   │   ├── vm.rs           # VirtualMachine execution
│   │   ├── gc.rs           # Hybrid GC
│   │   └── error.rs        # Error types
│   └── tests/              # 120 tests (94% passing)
│
├── compiler/               # 🚧 SPRINT 11 (Parser → Bytecode)
│   ├── src/
│   │   ├── lexer.rs        # Tokenizer (futuro)
│   │   ├── parser.rs       # AST builder (futuro)
│   │   └── codegen.rs      # AST → Bytecode (futuro)
│   └── tests/
│
├── stdlib/                 # 🚧 SPRINTS 12-15 (Standard Library)
│   ├── src/
│   │   ├── io.rs           # File I/O
│   │   ├── collections.rs  # List, Dict, Set
│   │   └── string.rs       # String operations
│   └── tests/
│
├── cli/                    # 🚧 SPRINT 10 (Main CLI)
│   ├── src/
│   │   ├── main.rs         # Entry point: vela run, vela build
│   │   ├── repl.rs         # REPL (futuro)
│   │   └── commands.rs     # CLI subcommands
│   └── Cargo.toml
│
├── runtime/                # 🚧 SPRINT 10 (Integration glue)
│   ├── src/
│   │   ├── lib.rs          # Re-exports públicos
│   │   └── integration.rs  # Glue: compiler + vm + stdlib
│   └── Cargo.toml
│
└── tests/                  # 🚧 SPRINT 10 (End-to-end tests)
    ├── integration/        # Source → Execution tests
    ├── benchmarks/         # Performance tests vs Python
    └── memory/             # Memory safety tests
```

#### 1.2 Flujo de Ejecución (Sprint 10 MVP)

**Actualmente (Sprint 9):**
```
Manual Bytecode → VirtualMachine.execute() → Result<Value>
```

**Sprint 10 MVP (sin parser):**
```
Bytecode file (.velac) → VirtualMachine.execute() → Result<Value>
                ↑
         (manual creation)
```

**Futuro (Sprint 11+):**
```
Source (.vela) → Compiler → Bytecode (.velac) → VM → Result<Value>
```

#### 1.3 Integration Points

**Sprint 10 se enfoca en:**
1. ✅ **VM standalone execution** (ya funciona)
2. ✅ **CLI para ejecutar bytecode** (`vela run program.velac`)
3. ✅ **Tests end-to-end con bytecode manual**
4. ✅ **Benchmarks VM vs CPython**
5. ✅ **Memory profiling**

**No incluye (Sprint 11+):**
- ❌ Parser Vela source
- ❌ Stdlib completo
- ❌ Python FFI

---

### 2. Testing Strategy

#### 2.1 Test Pyramid

```
         /\
        /  \  E2E Tests (10)       - Full programs bytecode → result
       /____\
      /      \  Integration (30)   - VM + GC + Error handling
     /________\
    /          \  Unit Tests (120) - Individual modules
   /____________\
```

**Distribución Target Sprint 10:**
- Unit tests: 120 (✅ done Sprint 9)
- Integration tests: 30 new
- End-to-end tests: 10 new
- **Total: 160 tests**

#### 2.2 Test Categories

**A. Unit Tests (120) - ✅ Done Sprint 9**
- Bytecode serialization: 18 tests
- GC allocation: 24 tests
- VM execution: 23 tests
- Integration: 20 tests
- Embedded: 35 tests

**B. Integration Tests (30) - 🚧 Sprint 10**
1. **VM + GC Integration (10 tests)**
   - Large programs with heavy allocation
   - GC triggering during execution
   - Reference cycle cleanup
   - Memory growth patterns

2. **Error Propagation (10 tests)**
   - Stack traces across call frames
   - Exception handling
   - Error recovery
   - Panic safety

3. **Multi-module Programs (10 tests)**
   - Code objects with multiple functions
   - Global variable sharing
   - Cross-module calls (futuro)

**C. End-to-End Tests (10) - 🚧 Sprint 10**
1. **Fibonacci (recursive)**
2. **Factorial (iterative)**
3. **Prime sieve**
4. **Sorting algorithms** (bubble, quick)
5. **Tree traversal** (DFS, BFS)
6. **String manipulation**
7. **Arithmetic expressions**
8. **Control flow** (if/else, loops)
9. **Function calls** (simple)
10. **Error cases** (division by zero, stack overflow)

#### 2.3 Performance Benchmarks

**Benchmark Suite (vs CPython 3.12):**

| Benchmark | Description | Target Speedup |
|-----------|-------------|----------------|
| `arithmetic` | 1M arithmetic ops | 5-8x |
| `fibonacci_recursive` | fib(30) | 3-5x |
| `list_operations` | 100K append/pop | 4-6x |
| `dict_operations` | 100K insert/lookup | 3-4x |
| `function_calls` | 1M function calls | 6-10x |
| `gc_stress` | Heavy allocation | 2-3x |

**Criterion.rs Configuration:**
```rust
criterion_group! {
    name = vm_benchmarks;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10));
    targets = 
        bench_arithmetic,
        bench_fibonacci,
        bench_list_ops,
        bench_dict_ops,
        bench_function_calls,
        bench_gc_stress
}
```

#### 2.4 Memory Safety Verification

**Tools & Techniques:**

1. **MIRI (Rust Interpreter)**
   - Detects undefined behavior
   - Verifies unsafe code (none expected)
   - Checks aliasing violations
   ```bash
   cargo +nightly miri test
   ```

2. **Valgrind (Linux/Mac)**
   - Memory leak detection
   - Invalid memory access
   - Use-after-free
   ```bash
   valgrind --leak-check=full ./target/release/vela
   ```

3. **AddressSanitizer (ASAN)**
   - Runtime memory error detection
   ```bash
   RUSTFLAGS="-Z sanitizer=address" cargo test
   ```

4. **Static Analysis**
   - Clippy lints (pedantic + nursery)
   - Cargo-deny (dependency audit)
   - Cargo-audit (security vulnerabilities)

**Memory Safety Checklist:**
- [ ] Zero memory leaks (Valgrind clean)
- [ ] Zero unsafe blocks (audit if any)
- [ ] All lifetimes correct (MIRI clean)
- [ ] No data races (thread safety)
- [ ] Panic safety (no resource leaks on panic)
- [ ] Drop implementations correct

---

### 3. CLI Integration

#### 3.1 `vela` CLI Tool

**Commands (Sprint 10 MVP):**

```bash
# Execute bytecode file
vela run program.velac

# Show version
vela --version

# Show help
vela --help
```

**Futuro (Sprint 11+):**
```bash
# Compile source to bytecode
vela build program.vela -o program.velac

# Run source directly (compile + execute)
vela run program.vela

# REPL
vela repl

# Disassemble bytecode
vela dis program.velac

# Run tests
vela test

# Package manager
vela install package-name
```

#### 3.2 CLI Implementation

**cli/src/main.rs:**
```rust
use clap::{Parser, Subcommand};
use vela_vm::{Bytecode, VirtualMachine};

#[derive(Parser)]
#[command(name = "vela")]
#[command(about = "Vela Language Runtime", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a bytecode file
    Run {
        /// Path to .velac bytecode file
        file: String,
    },
    /// Show version
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { file } => {
            let bytecode = Bytecode::from_file(&file)?;
            let mut vm = VirtualMachine::new();
            let result = vm.execute(&bytecode)?;
            println!("{}", result);
            Ok(())
        }
        Commands::Version => {
            println!("vela {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
```

---

### 4. Performance Targets

#### 4.1 Baseline: CPython 3.12

**Mediciones actuales (Python):**
- Arithmetic ops: ~50ns/op
- Function calls: ~200ns/call
- List append: ~100ns/op
- Dict insert: ~150ns/op
- GC overhead: ~10-15%

**Target Vela VM (Sprint 10):**
- Arithmetic ops: <10ns/op (5x faster)
- Function calls: <40ns/call (5x faster)
- List append: <20ns/op (5x faster)
- Dict insert: <50ns/op (3x faster)
- GC overhead: <5%

#### 4.2 Optimization Strategy

**Phase 1 (Sprint 10): Baseline Performance**
- ✅ Stack-based bytecode interpreter
- ✅ NaN-boxing value representation
- ✅ Hybrid GC (RC + cycle detection)
- Target: 3-5x faster than CPython

**Phase 2 (Sprint 16+): Optimizations**
- 🚧 Inline caching (method dispatch)
- 🚧 Type specialization (monomorphization)
- 🚧 Constant folding
- 🚧 Dead code elimination
- Target: 5-10x faster

**Phase 3 (Sprint 20+): JIT Compilation**
- 🚧 Cranelift JIT backend
- 🚧 Hot path detection
- 🚧 Tiered compilation
- Target: 10-20x faster

---

### 5. Documentation Strategy

#### 5.1 Documentation Structure

```
docs/
├── architecture/
│   ├── ADR-801-vm-architecture.md        # ✅ Done Sprint 9
│   ├── ADR-900-integration-strategy.md   # ✅ This document
│   └── ADR-XXX-future-decisions.md
│
├── guides/
│   ├── getting-started.md                # Quick start guide
│   ├── migration-guide.md                # Python → Vela
│   ├── performance-guide.md              # Optimization tips
│   └── contributor-guide.md              # How to contribute
│
├── reference/
│   ├── bytecode-format.md                # Bytecode specification
│   ├── vm-internals.md                   # VM implementation details
│   ├── gc-design.md                      # GC algorithm explanation
│   └── api-reference.md                  # Public API docs
│
├── tutorials/
│   ├── 01-hello-world.md
│   ├── 02-control-flow.md
│   ├── 03-functions.md
│   └── 04-advanced.md
│
└── releases/
    ├── sprint-9.md                       # ✅ Done
    └── sprint-10.md                      # 🚧 This sprint
```

#### 5.2 Documentation Priorities Sprint 10

**Must Have:**
1. ✅ Integration architecture (this ADR)
2. 🚧 Getting started guide (how to build & run)
3. 🚧 Bytecode format reference
4. 🚧 Performance benchmark results
5. 🚧 Memory safety verification report

**Nice to Have:**
- Migration guide Python → Vela
- VM internals deep dive
- GC design explanation
- Tutorial series

---

### 6. Compatibility Layer (Futuro - Sprint 16+)

**Python Interop via PyO3:**

```rust
use pyo3::prelude::*;

#[pyclass]
struct VelaVM {
    vm: VirtualMachine,
}

#[pymethods]
impl VelaVM {
    #[new]
    fn new() -> Self {
        VelaVM { vm: VirtualMachine::new() }
    }
    
    fn execute(&mut self, bytecode_path: &str) -> PyResult<PyObject> {
        let bytecode = Bytecode::from_file(bytecode_path)?;
        let result = self.vm.execute(&bytecode)?;
        // Convert Vela Value → Python object
        Ok(value_to_pyobject(result))
    }
}

#[pymodule]
fn vela(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<VelaVM>()?;
    Ok(())
}
```

**Uso desde Python:**
```python
import vela

vm = vela.VelaVM()
result = vm.execute("program.velac")
print(result)
```

---

## Consecuencias

### Positivas

1. **Integración clara**: Workspace coherente con responsabilidades bien definidas
2. **Testing robusto**: 160 tests cubriendo unit, integration, end-to-end
3. **Performance medible**: Benchmarks sistemáticos vs CPython
4. **Memory safety garantizada**: Múltiples herramientas de verificación
5. **Documentación completa**: Facilita onboarding y contribuciones
6. **Escalabilidad**: Arquitectura preparada para Parser (Sprint 11) y Stdlib (12-15)

### Negativas

1. **No hay parser aún**: Sprint 10 solo ejecuta bytecode manual
2. **Stdlib mínimo**: Sin I/O, collections avanzadas, etc.
3. **Single-threaded**: Concurrency en sprints futuros
4. **No Python FFI**: Compatibility layer pospuesto

### Trade-offs

**Decidimos priorizar:**
- ✅ VM robusto y testeado (base sólida)
- ✅ Performance measurement desde inicio
- ✅ Memory safety verification exhaustiva

**En lugar de:**
- ❌ Parser completo (Sprint 11)
- ❌ Stdlib rica (Sprints 12-15)
- ❌ Python compatibility (Sprint 16+)

**Razón:** Es mejor tener un VM pequeño y correcto que un sistema grande y bugueado.

---

## Alternativas Consideradas

### Alternativa 1: "Big Bang" Integration

**Descripción:** Implementar todo en Sprint 10 (Parser + Stdlib + VM + CLI).

**Pros:**
- Sistema completo más rápido
- Demo end-to-end desde source

**Cons:**
- Alto riesgo de bugs
- Testing insuficiente
- Deuda técnica acumulada

**Decisión:** ❌ Rechazada - Preferimos iteración incremental.

### Alternativa 2: Parser primero, Testing después

**Descripción:** Sprint 10 para Parser, Sprint 11 para Testing.

**Pros:**
- Demo más atractivo (source → execution)

**Cons:**
- VM sin validar adecuadamente
- Performance desconocida
- Memory issues no detectados

**Decisión:** ❌ Rechazada - Testing es crítico para calidad.

### Alternativa 3: Solo Testing, sin integración CLI

**Descripción:** Sprint 10 solo para tests, sin CLI.

**Pros:**
- Máxima cobertura de tests
- VM extremadamente robusto

**Cons:**
- No hay forma de usar Vela desde CLI
- Difícil demo para stakeholders

**Decisión:** ❌ Rechazada - CLI mínimo es necesario para usabilidad.

---

## Referencias

- **ADR-801**: VM Architecture (Sprint 9)
- **EPIC-RUST-10**: Integration & Testing (Roadmap)
- **Criterion.rs**: https://github.com/bheisler/criterion.rs
- **MIRI**: https://github.com/rust-lang/miri
- **PyO3**: https://pyo3.rs (futuro)

---

## Implementación

### Sprint 10 Roadmap

**Week 1:**
- ✅ ADR-900 (this document)
- 🚧 CLI tool (`vela run`)
- 🚧 Integration tests (30 tests)

**Week 2:**
- 🚧 End-to-end tests (10 tests)
- 🚧 Performance benchmarks (6 benchmarks)
- 🚧 Memory safety verification (MIRI + Valgrind)

**Week 3:**
- 🚧 Documentation (guides + reference)
- 🚧 CI/CD pipeline (GitHub Actions)
- 🚧 Release sprint-10 tag

---

**ÚLTIMA ACTUALIZACIÓN:** 2025-12-04  
**VERSIÓN:** 1.0.0  
**STATUS:** ✅ Aceptado
