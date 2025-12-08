# TASK-101: Implementar vela doctor

## 📋 Información General
- **Historia:** VELA-592
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar el comando `vela doctor` para diagnóstico de instalación y verificación del entorno de desarrollo de Vela.

## 🔨 Funcionalidades Implementadas
1. **Diagnóstico de instalación de Vela CLI**:
   - Verificación de versión instalada
   - Ubicación del ejecutable

2. **Verificación de estructura de proyecto**:
   - Detección de proyectos Vela (vela.yaml, Cargo.toml, package.json)
   - Mensajes informativos para proyectos no detectados

3. **Verificación de herramientas requeridas**:
   - Rust compiler (rustc)
   - Cargo package manager
   - Node.js (opcional para desarrollo web)

4. **Información del sistema**:
   - Sistema operativo
   - Arquitectura
   - Recursos básicos del sistema

5. **Modos de operación**:
   - Modo básico: diagnósticos esenciales
   - Modo verbose: información detallada
   - Modo fix: preparación para correcciones automáticas

### Archivos modificados
- `cli/src/main.rs` - Agregado comando Doctor y función handle_doctor
- `cli/src/test_cli_doctor.rs` - Tests unitarios para el comando doctor

### Código Principal

```rust
/// Diagnose Vela installation and environment
Doctor {
    /// Show detailed diagnostic information
    #[arg(long)]
    verbose: bool,

    /// Fix issues automatically if possible
    #[arg(long)]
    fix: bool,
},
```

```rust
fn handle_doctor(verbose: bool, fix: bool) -> Result<()> {
    println!("🔍 Running Vela Doctor - Installation Diagnostics");

    // Check Vela CLI version
    // Check project structure
    // Check required tools (Rust, Cargo, Node.js)
    // Check system resources
    // Provide summary and recommendations

    Ok(())
}
```

## ✅ Criterios de Aceptación
- [x] Comando `vela doctor` disponible en CLI
- [x] Verificación de instalación de Vela CLI
- [x] Detección de estructura de proyecto
- [x] Verificación de herramientas requeridas (Rust, Cargo)
- [x] Información básica del sistema
- [x] Soporte para modos verbose y fix
- [x] Tests unitarios completos (4 tests)
- [x] Mensajes informativos y sugerencias

## 🧪 Tests Implementados
1. `test_doctor_basic` - Verifica funcionamiento básico
2. `test_doctor_verbose` - Verifica modo verbose
3. `test_doctor_fix_mode` - Verifica modo fix
4. `test_doctor_verbose_and_fix` - Verifica combinación de modos

## 🔗 Referencias
- **Jira:** [TASK-101](https://velalang.atlassian.net/browse/TASK-101)
- **Historia:** [VELA-592](https://velalang.atlassian.net/browse/VELA-592)
- **Código:** `cli/src/main.rs` (función `handle_doctor`)
- **Tests:** `cli/src/test_cli_doctor.rs`