# TASK-099: Implementar comando vela test

## 📋 Información General
- **Historia:** VELA-XXX
- **Estado:** Completada ✅
- **Fecha:** 2025-12-10

## 🎯 Objetivo
Implementar el comando `vela test` como runner de tests unitarios para Vela, permitiendo ejecutar archivos .vela que contengan funciones marcadas con el decorador `@test`.

## 🔨 Implementación

### Arquitectura del Comando Test
El comando `vela test` sigue el patrón establecido por `vela run`:

1. **Discovery de Tests**: Busca recursivamente archivos `.vela` que contengan `@test`
2. **Compilación Individual**: Compila cada archivo de test usando `vela_compiler::Compiler`
3. **Ejecución**: Ejecuta el bytecode compilado en la VM
4. **Reporting**: Reporta resultados (tests pasados/fallidos)

### Código Implementado

#### Función `execute_test` en `tooling/src/cli/commands.rs`
```rust
pub fn execute_test(filter: Option<&str>, release: bool) -> Result<()> {
    // 1. Configuración y logging
    // 2. Búsqueda recursiva de archivos .vela con @test
    // 3. Compilación individual usando vela_compiler
    // 4. Ejecución en VM y captura de resultados
    // 5. Reporte final con estadísticas
}
```

#### Discovery de Tests
```rust
fn find_test_files(dir: &std::path::Path, test_files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    // Busca archivos .vela que contengan "@test"
    // Excluye directorios comunes (target, .git, etc.)
}
```

#### Compilación y Ejecución
```rust
// Compilar usando vela-compiler
let mut compiler = vela_compiler::Compiler::default();
let bytecode_bytes = compiler.compile_file(test_file)?;

// Deserializar y ejecutar en VM
let bytecode: vela_vm::Bytecode = bincode::deserialize(&bytecode_bytes)?;
let mut vm = vela_vm::VirtualMachine::new();
let result = vm.execute(&bytecode);
```

### Framework de Testing Vela

El comando aprovecha el framework de testing existente:

- **Decoradores**: `@test`, `@beforeEach`, `@afterEach`
- **Assertions**: `assert()`, `assertEquals()`, `assertThrows()`
- **Setup/Teardown**: Configuración automática por archivo

### Ejemplo de Archivo de Test
```vela
import 'system:test' show { test, assert, assertEquals }

@test
fn testAddition() -> void {
  result = add(2, 3)
  assertEquals(result, 5, "2 + 3 should equal 5")
}

@test
fn testSubtraction() -> void {
  result = subtract(5, 3)
  assert(result == 2, "5 - 3 should equal 2")
}
```

## ✅ Criterios de Aceptación
- [x] Comando `vela test` implementado
- [x] Discovery automático de archivos con `@test`
- [x] Compilación individual de tests
- [x] Ejecución en VM con captura de errores
- [x] Reporte detallado de resultados
- [x] Integración con framework de testing existente
- [x] Manejo de errores de compilación
- [x] Tests pasan cuando no hay assertions fallidas
- [x] Tests fallan cuando hay assertions fallidas

## 🔗 Referencias
- **Jira:** [TASK-099](https://velalang.atlassian.net/browse/TASK-099)
- **Historia:** [VELA-XXX](https://velalang.atlassian.net/browse/VELA-XXX)
- **Código relacionado:**
  - `tooling/src/cli/commands.rs` - Implementación del comando
  - `tests/unit/vm/test_heap.vela` - Ejemplo de tests existentes
  - `tooling/src/cli/mod.rs` - Integración con CLI</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-XXX\TASK-099.md