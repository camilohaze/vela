# VELA-XXX: Implementar comando vela test

## 📋 Información General
- **Epic:** EPIC-06: Compiler Backend
- **Sprint:** Sprint Toolchain
- **Estado:** Completada ✅
- **Fecha:** 2025-12-10

## 🎯 Descripción
Implementación completa del comando `vela test` como runner de tests unitarios para el lenguaje Vela. El comando permite ejecutar automáticamente todos los archivos .vela que contengan funciones marcadas con el decorador `@test`, compilándolos y ejecutándolos en la VM con reporting detallado de resultados.

## 📦 Subtasks Completadas

### TASK-099: Implementar comando vela test
- ✅ Comando `vela test` funcional
- ✅ Discovery automático de archivos con `@test`
- ✅ Compilación individual usando `vela_compiler`
- ✅ Ejecución en VM con captura de resultados
- ✅ Reporte detallado: tests pasados/fallidos
- ✅ Integración con framework de testing existente
- ✅ Manejo de errores de compilación y ejecución

## 🔨 Implementación Completa

### Arquitectura del Comando Test

#### 1. Discovery de Tests
```rust
fn find_test_files(dir: &std::path::Path, test_files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    // Búsqueda recursiva de archivos .vela
    // Filtrado por contenido: archivos que contienen "@test"
    // Exclusión de directorios: target/, .git/, node_modules/, etc.
}
```

#### 2. Compilación Individual
```rust
// Cada archivo de test se compila individualmente
let mut compiler = vela_compiler::Compiler::default();
let bytecode_bytes = compiler.compile_file(test_file)?;

// Bytecode se guarda en target/ para ejecución
let bytecode_path = output_dir.join(format!("{}.velac", file_stem));
std::fs::write(&bytecode_path, &bytecode_bytes)?;
```

#### 3. Ejecución y Reporting
```rust
// Deserialización y ejecución en VM
let bytecode: vela_vm::Bytecode = bincode::deserialize(&bytecode_bytes)?;
let mut vm = vela_vm::VirtualMachine::new();
let result = vm.execute(&bytecode);

// Tests pasan si ejecución exitosa (sin excepciones)
// Tests fallan si hay excepciones de assertions
```

### Framework de Testing Integrado

El comando `vela test` aprovecha el framework de testing completo de Vela:

#### Decoradores de Testing
- `@test`: Marca función como test unitario
- `@beforeEach`: Setup que se ejecuta antes de cada test
- `@afterEach`: Cleanup que se ejecuta después de cada test

#### Assertions Disponibles
- `assert(condition, message)`: Assertion básica
- `assertEquals(actual, expected, message)`: Verificación de igualdad
- `assertThrows(block, expectedError)`: Verificación de excepciones

#### Ejemplo de Archivo de Test
```vela
import 'system:test' show { test, assert, assertEquals, beforeEach, afterEach }

state calculator: Option<Calculator> = None

@beforeEach
fn setup() -> void {
  calculator = Some(Calculator())
}

@afterEach
fn teardown() -> void {
  calculator = None
}

@test
fn testAddition() -> void {
  calc = calculator.unwrap()
  result = calc.add(2, 3)
  assertEquals(result, 5, "2 + 3 should equal 5")
}

@test
fn testDivisionByZero() -> void {
  calc = calculator.unwrap()
  assertThrows(|| calc.divide(10, 0), "DivisionByZeroError")
}
```

### Output del Comando

#### Ejecución Exitosa
```
🧪 Running Vela tests...
📋 Configuration:
   Release mode: false

📂 Found 3 test files:
   tests/unit/vm/test_heap.vela
   tests/unit/vm/test_gc.vela
   tests/unit/vm/test_vm.vela

▶️  Running tests in: tests/unit/vm/test_heap.vela
✅ Tests passed in 45 ms

▶️  Running tests in: tests/unit/vm/test_gc.vela
✅ Tests passed in 32 ms

▶️  Running tests in: tests/unit/vm/test_vm.vela
✅ Tests passed in 28 ms

📊 Test Results:
   Files: 3
   Passed: 3 ✅
   Failed: 0 ❌

✅ All tests passed!
```

#### Con Tests Fallidos
```
🧪 Running Vela tests...

▶️  Running tests in: tests/unit/vm/test_heap.vela
❌ Tests failed after 67 ms: AssertionError: Expected 5, got 4

📊 Test Results:
   Files: 1
   Passed: 0 ✅
   Failed: 1 ❌

❌ Some tests failed!
```

## 📊 Métricas
- **Subtasks completadas**: 1/1
- **Archivos modificados**: 1 (`tooling/src/cli/commands.rs`)
- **Líneas de código agregadas**: ~120 líneas
- **Funcionalidad**: Discovery, compilación, ejecución, reporting
- **Integración**: Framework de testing Vela completo

## ✅ Definición de Hecho
- [x] Comando `vela test` implementado y funcional
- [x] Discovery automático de archivos de test
- [x] Compilación individual de tests usando `vela_compiler`
- [x] Ejecución en VM con captura de resultados
- [x] Reporte detallado de tests pasados/fallidos
- [x] Integración completa con framework `@test`
- [x] Manejo robusto de errores de compilación
- [x] Tests unitarios para validar implementación
- [x] Documentación completa generada

## 🔗 Referencias
- **Jira:** [VELA-XXX](https://velalang.atlassian.net/browse/VELA-XXX)
- **Epic:** [EPIC-06: Compiler Backend](https://velalang.atlassian.net/browse/EPIC-06)

## 📁 Estructura de Archivos
```
tooling/src/cli/commands.rs          # Implementación execute_test()
tests/unit/vm/                        # Archivos de test existentes
├── test_heap.vela                    # Tests de heap allocation
├── test_gc.vela                      # Tests de garbage collector
└── test_vm.vela                      # Tests de VM core

docs/features/VELA-XXX/
├── README.md                         # Esta documentación
└── TASK-099.md                       # Documentación detallada
```