# Extensión del Comando vela run (Implementado Incorrectamente como TASK-101)

## 📋 Información General
- **Historia:** VELA-592
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30
- **Nota:** Esta implementación corresponde a TASK-098 según roadmap, no TASK-101

## 🎯 Objetivo
Extender el comando `vela run` para que pueda ejecutar archivos fuente `.vela` directamente, compilando on-the-fly, además de los archivos bytecode `.velac` existentes.

## 🔨 Funcionalidades Implementadas
1. **Ejecución directa de archivos .vela**:
   - Compilación automática on-the-fly
   - Integración con el compilador existente
   - Mantiene compatibilidad con archivos .velac

2. **Detección automática de tipo de archivo**:
   - `.vela` → Compila y ejecuta
   - `.velac` → Carga bytecode directamente
   - Otros → Error con mensaje descriptivo

3. **Mejora en la documentación del comando**:
   - Actualización de help text
   - Clarificación de formatos soportados

### Archivos modificados
- `cli/src/main.rs` - Extensión de `handle_run` para compilar archivos .vela
- `cli/src/test_cli_run.rs` - Tests unitarios para la nueva funcionalidad

### Código Principal

```rust
fn handle_run(file: PathBuf, args: Vec<String>, trace: bool, gc_stats: bool) -> Result<()> {
    // Detect file type and process accordingly
    let ext = file.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let bytecode = if ext == "vela" {
        // Compile source file to bytecode
        println!("Compiling {}...", file.display());

        let source = fs::read_to_string(&file)?;
        let mut compiler = Compiler::new(Config::default());
        let bytecode_bytes = compiler.compile_string(&source, file.to_string_lossy().as_ref())?;

        Bytecode::deserialize(&bytecode_bytes)?
    } else if ext == "velac" {
        // Load existing bytecode file
        println!("Loading bytecode from {}...", file.display());
        let bytecode_bytes = fs::read(&file)?;
        Bytecode::deserialize(&bytecode_bytes)?
    } else {
        anyhow::bail!("Unsupported file type: .{}. Expected .vela (source) or .velac (bytecode)", ext);
    };

    // Execute bytecode...
}
```

## ✅ Criterios de Aceptación
- [x] Comando `vela run` acepta archivos `.vela`
- [x] Compilación automática on-the-fly
- [x] Compatibilidad mantenida con archivos `.velac`
- [x] Mensajes de error descriptivos
- [x] Tests unitarios completos (3 tests)
- [x] Documentación actualizada

## 🧪 Tests Implementados
1. `test_run_vela_source_file` - Verifica ejecución de archivos .vela
2. `test_run_file_not_found` - Manejo de errores de archivo inexistente
3. `test_run_unsupported_file_type` - Rechazo de tipos de archivo no soportados

## 🔗 Referencias
- **Historia:** VELA-592 (CLI Tooling)
- **Jira:** [TASK-098](https://velalang.atlassian.net/browse/TASK-098) - Según roadmap oficial
- **Código:** `cli/src/main.rs` (función `handle_run`)
- **Tests:** `cli/src/test_cli_run.rs`