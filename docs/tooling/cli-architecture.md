# Arquitectura del Vela CLI

**Historia:** VELA-562 (US-00C)  
**Subtask:** TASK-000J  
**Fecha:** 2025-11-30  
**Estado:** ✅ Completado

---

## 📋 Resumen Ejecutivo

Este documento define la arquitectura completa del **Vela CLI** (`vela`), la herramienta principal para desarrollar aplicaciones en Vela. El CLI será el punto de entrada unificado para todas las operaciones de desarrollo: compilación, ejecución, testing, formateo, linting, gestión de paquetes y DevTools.

---

## 1. Framework CLI Elegido

### **Opción seleccionada: `clap` (Rust)**

**Razones:**
- ✅ **Ecosystem fit**: Rust es el lenguaje del compilador → integración nativa
- ✅ **Performance**: Parsing de argumentos instantáneo
- ✅ **Type safety**: Validación en compile-time de subcomandos
- ✅ **Auto-completion**: Genera completions para Bash/Zsh/Fish/PowerShell
- ✅ **Error messages**: Mensajes de error con sugerencias ("did you mean?")
- ✅ **Derive macros**: Menos boilerplate con `#[derive(Parser)]`

**Alternativas descartadas:**
- ❌ **cobra (Go)**: Requiere reescribir CLI en Go, mala integración con Rust compiler
- ❌ **Commander.js (Node)**: Startup lento, requiere Node runtime
- ❌ **Manual parsing**: Reinventar rueda, sin auto-completion

**Implementación:**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "vela", version, about = "The Vela programming language CLI")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    
    /// Verbose output
    #[clap(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project
    Build(BuildArgs),
    /// Run the project
    Run(RunArgs),
    /// Run tests
    Test(TestArgs),
    /// Format code
    Fmt(FmtArgs),
    /// Lint code
    Lint(LintArgs),
    /// Manage packages
    #[clap(subcommand)]
    Pkg(PkgCommands),
    /// Open DevTools
    Devtools(DevtoolsArgs),
}
```

---

## 2. Estructura de Comandos

### **2.1. Comando `vela build`**

**Descripción:** Compila el proyecto a targets específicos.

**Sintaxis:**
```bash
vela build [OPTIONS] [TARGET]
```

**Opciones:**
- `--target <TARGET>`: Target de compilación
  - `vm` (default): VelaVM bytecode
  - `native`: LLVM native binary
  - `web`: JavaScript/WASM
  - `mobile-ios`: iOS binary
  - `mobile-android`: Android APK
  - `desktop`: Desktop app (Tauri-like)
- `--release`: Build optimizado (sin debug info)
- `--debug`: Build con símbolos de depuración (default)
- `--output <PATH>`: Carpeta de salida (default: `build/`)
- `--watch`: Watch mode (recompila automáticamente)

**Ejemplo:**
```bash
vela build --target native --release
vela build --target web --output dist/
vela build --watch  # Hot reload
```

**Salida:**
```
📦 Building Vela project...
   Compiling vela_core v1.0.0 (src/main.vela)
   Compiling my_app v0.1.0 (src/)
    Finished release [optimized] target(s) in 2.34s
```

---

### **2.2. Comando `vela run`**

**Descripción:** Ejecuta el proyecto (compila + ejecuta en un solo paso).

**Sintaxis:**
```bash
vela run [OPTIONS] [ARGS]
```

**Opciones:**
- `--target <TARGET>`: Target de ejecución (default: `vm`)
- `--release`: Ejecutar build optimizado
- `--watch`: Watch mode con hot reload
- `-- [ARGS]`: Argumentos para la aplicación

**Ejemplo:**
```bash
vela run
vela run --release -- --config prod.yaml
vela run --watch  # Dev server con hot reload
```

---

### **2.3. Comando `vela test`**

**Descripción:** Ejecuta tests unitarios e integración.

**Sintaxis:**
```bash
vela test [OPTIONS] [PATTERN]
```

**Opciones:**
- `--unit`: Solo tests unitarios
- `--integration`: Solo tests de integración
- `--e2e`: Solo tests end-to-end
- `--watch`: Watch mode (re-run on change)
- `--coverage`: Generar reporte de cobertura
- `--filter <PATTERN>`: Filtrar por nombre de test
- `--parallel`: Ejecutar tests en paralelo (default: true)
- `--sequential`: Ejecutar tests secuencialmente

**Ejemplo:**
```bash
vela test                    # Todos los tests
vela test --unit             # Solo unit tests
vela test --coverage         # Con coverage
vela test --filter "parser"  # Tests con "parser" en nombre
```

**Salida:**
```
🧪 Running 127 tests...
   test lexer::test_tokenize ... ok (0.03s)
   test parser::test_expression ... ok (0.12s)
   ...
   test result: ok. 127 passed; 0 failed; 0 ignored

📊 Coverage: 87.3%
```

---

### **2.4. Comando `vela fmt`**

**Descripción:** Formatea código según estilo oficial.

**Sintaxis:**
```bash
vela fmt [OPTIONS] [FILES]
```

**Opciones:**
- `--check`: Verifica sin modificar archivos
- `--emit files`: Solo formatea archivos con cambios
- `--config <FILE>`: Archivo de configuración custom

**Ejemplo:**
```bash
vela fmt                  # Formatea todos los archivos
vela fmt src/main.vela    # Formatea archivo específico
vela fmt --check          # CI: verificar sin modificar
```

**Configuración** (`vela.yaml`):
```yaml
fmt:
  max_width: 100
  indent_spaces: 2
  trailing_comma: true
```

---

### **2.5. Comando `vela lint`**

**Descripción:** Ejecuta linter para detectar code smells.

**Sintaxis:**
```bash
vela lint [OPTIONS] [FILES]
```

**Opciones:**
- `--fix`: Aplicar fixes automáticos
- `--deny <LINT>`: Tratar warning como error
- `--allow <LINT>`: Ignorar lint específico
- `--explain <CODE>`: Explicar error/warning

**Ejemplo:**
```bash
vela lint
vela lint --fix
vela lint --deny unused_variable
vela lint --explain E0308
```

**Salida:**
```
⚠️  Warning: Unused variable `x` in function `foo`
   --> src/main.vela:42:9
    |
 42 |     let x = 10;
    |         ^ help: consider using `_x` if unused
    |
```

---

### **2.6. Comando `vela pkg` (Package Manager)**

**Descripción:** Gestión de paquetes.

**Subcomandos:**
- `vela pkg install [PACKAGE]`: Instalar paquete
- `vela pkg update [PACKAGE]`: Actualizar paquete
- `vela pkg remove [PACKAGE]`: Remover paquete
- `vela pkg list`: Listar paquetes instalados
- `vela pkg search <QUERY>`: Buscar en registry
- `vela pkg publish`: Publicar paquete
- `vela pkg init`: Crear nuevo proyecto

**Ejemplo:**
```bash
vela pkg install http@^1.0.0
vela pkg update
vela pkg search "json parser"
vela pkg publish
```

**Gestión de `vela.yaml`** (auto-actualizado):
```yaml
name: "my_app"
version: "0.1.0"
dependencies:
  http: "^1.0.0"
  json: "2.3.1"
```

**Gestión de `vela.lock`** (lockfile auto-generado):
```yaml
packages:
  - name: http
    version: 1.2.3
    checksum: sha256:abcd1234...
    dependencies: []
```

---

### **2.7. Comando `vela devtools`**

**Descripción:** Abrir DevTools UI (navegador embebido).

**Sintaxis:**
```bash
vela devtools [OPTIONS]
```

**Opciones:**
- `--port <PORT>`: Puerto del servidor DevTools (default: 9229)
- `--open`: Abrir navegador automáticamente

**Ejemplo:**
```bash
vela devtools
vela devtools --port 8080 --open
```

**Funcionalidades:**
- 🔍 **UI Inspector**: Tree view de widgets, live editing
- 📊 **Signal Graph**: Visualizar dependency graph reactivo
- 🚀 **Performance Profiler**: CPU, memoria, network
- 📜 **Console**: Logs de aplicación

---

## 3. Sistema de Configuración

### **3.1. Archivo `vela.yaml`**

**Ubicación:** Raíz del proyecto.

**Schema completo:**
```yaml
# Metadata del proyecto
name: "my_app"
version: "0.1.0"
description: "My Vela application"
authors: ["John Doe <john@example.com>"]
license: "Apache-2.0"

# Configuración de build
build:
  target: "vm"  # vm | native | web | mobile-ios | mobile-android | desktop
  output: "build/"
  optimization: "release"  # debug | release

# Dependencias
dependencies:
  http: "^1.0.0"
  json: "2.3.1"

# Dev dependencies (solo para desarrollo)
dev_dependencies:
  testing: "^0.5.0"

# Configuración de formato
fmt:
  max_width: 100
  indent_spaces: 2
  trailing_comma: true

# Configuración de linter
lint:
  deny:
    - unused_variable
  allow:
    - dead_code

# Scripts custom
scripts:
  start: "vela run --watch"
  prod: "vela build --release && vela run --release"

# Targets específicos
targets:
  web:
    output: "dist/"
    minify: true
  native:
    optimization_level: 3
```

**Validación:** El CLI valida el schema usando JSON Schema interno.

---

### **3.2. Variables de Entorno**

**Soportadas:**
- `VELA_HOME`: Directorio de instalación (default: `~/.vela`)
- `VELA_CACHE`: Directorio de caché (default: `$VELA_HOME/cache`)
- `VELA_LOG`: Nivel de logging (`debug`, `info`, `warn`, `error`)
- `VELA_REGISTRY`: URL del registry (default: `https://registry.velalang.org`)

**Ejemplo:**
```bash
export VELA_LOG=debug
vela build  # Build con logging verbose
```

---

### **3.3. Flags CLI**

**Prioridad** (mayor a menor):
1. **Flags CLI** (ej: `--target native`)
2. **Variables de entorno** (`VELA_LOG=debug`)
3. **Archivo `vela.yaml`**
4. **Defaults del CLI**

**Ejemplo:**
```bash
# vela.yaml: target = "vm"
vela build --target native  # Flag gana: compila a native
```

---

## 4. Sistema de Plugins

### **4.1. Arquitectura de Plugins**

**Requisito:** Soporte para plugins de terceros (linters custom, formatters, targets custom).

**Enfoque:** Plugins como **dynamic libraries** (`.so`, `.dylib`, `.dll`).

**API de Plugin:**
```rust
// Plugin interface
pub trait VelaPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self, context: &PluginContext) -> Result<()>;
    fn execute(&self, args: &PluginArgs) -> Result<PluginOutput>;
}

// Macro para exportar plugin
#[vela_plugin]
pub struct MyPlugin;

impl VelaPlugin for MyPlugin {
    // ...
}
```

**Instalación de Plugins:**
```bash
vela pkg install @plugins/custom-linter
```

**Configuración** (`vela.yaml`):
```yaml
plugins:
  - name: custom-linter
    enabled: true
    config:
      rules: ["no-console", "no-any"]
```

---

### **4.2. Hooks de Plugins**

**Hooks soportados:**
- `pre_build`: Antes de compilar
- `post_build`: Después de compilar
- `pre_test`: Antes de tests
- `post_test`: Después de tests
- `lint`: Linter custom
- `fmt`: Formatter custom

**Ejemplo de Plugin:**
```rust
#[vela_plugin]
pub struct CustomLinter;

impl VelaPlugin for CustomLinter {
    fn name(&self) -> &str { "custom-linter" }
    fn version(&self) -> &str { "1.0.0" }
    
    fn execute(&self, args: &PluginArgs) -> Result<PluginOutput> {
        let ast = args.context.get_ast()?;
        // Linting logic
        Ok(PluginOutput::Diagnostics(diagnostics))
    }
}
```

---

## 5. Integración con Toolchain

### **5.1. Interacción CLI ↔ Compiler**

```
┌─────────────┐
│   vela CLI  │
└──────┬──────┘
       │ (FFI/API calls)
       │
       v
┌─────────────────────┐
│  vela_compiler_api  │  (Rust crate)
└──────┬──────────────┘
       │
       v
┌─────────────────────┐
│  Vela Compiler      │
│  - Lexer            │
│  - Parser           │
│  - Type Checker     │
│  - Codegen          │
└─────────────────────┘
```

**API Pública:**
```rust
// vela_compiler_api crate
pub fn compile(
    source: &Path,
    target: Target,
    options: CompilerOptions,
) -> Result<CompilationOutput>;

pub fn format(source: &str, config: FmtConfig) -> Result<String>;

pub fn lint(source: &str, config: LintConfig) -> Result<Vec<Diagnostic>>;
```

---

### **5.2. Modos de Ejecución**

| Comando | Target | Ejecución |
|---------|--------|-----------|
| `vela run` | `vm` | Compila a bytecode → VelaVM interpreta |
| `vela run --target native` | `native` | Compila a LLVM IR → Binary nativo |
| `vela run --target web` | `web` | Compila a JS/WASM → Abre navegador |
| `vela devtools` | - | Inicia DevTools server (WebSocket) |

---

## 6. Performance y UX

### **6.1. Caching**

**Estrategia:**
- ✅ **Incremental compilation**: Solo recompilar archivos modificados
- ✅ **Dependency caching**: Cache de paquetes descargados
- ✅ **Build artifacts**: Reutilizar outputs previos

**Ubicación de caché:** `~/.vela/cache/`

---

### **6.2. Error Messages**

**Principios:**
- ✅ **Contexto claro**: Mostrar línea de código con error
- ✅ **Sugerencias**: "did you mean?" para typos
- ✅ **Links**: `--explain E0308` para documentación detallada

**Ejemplo:**
```
error[E0308]: mismatched types
  --> src/main.vela:10:5
   |
10 |     return "hello";
   |     ^^^^^^^^^^^^^^ expected `Int`, found `String`
   |
   = note: expected type `Int`
              found type `String`
   = help: try casting with `as Int` or change return type to `String`

For more information about this error, try `vela lint --explain E0308`
```

---

### **6.3. Progress Indicators**

**Feedback visual:**
- ✅ **Spinner**: Durante compilación larga
- ✅ **Progress bar**: Para descarga de paquetes
- ✅ **Emoji**: Estado visual (✅ success, ❌ error, ⚠️ warning)

---

## 7. Cross-Platform Support

### **7.1. Soporte de Plataformas**

| Plataforma | Arquitecturas | Status |
|------------|---------------|--------|
| **Linux** | x86_64, aarch64 | ✅ Full support |
| **macOS** | x86_64 (Intel), aarch64 (M1/M2) | ✅ Full support |
| **Windows** | x86_64 | ✅ Full support |

---

### **7.2. Instalación**

**Método recomendado: Shell installer**
```bash
# Unix/macOS
curl --proto '=https' --tlsv1.2 -sSf https://velalang.org/install.sh | sh

# Windows (PowerShell)
iwr https://velalang.org/install.ps1 -useb | iex
```

**Instaladores alternativos:**
- **Homebrew** (macOS): `brew install vela`
- **Chocolatey** (Windows): `choco install vela`
- **Cargo** (Rust): `cargo install vela-cli`

---

## 8. Testing del CLI

### **8.1. Tests Unitarios**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_parsing() {
        let cli = Cli::parse_from(&["vela", "build", "--release"]);
        assert!(matches!(cli.command, Commands::Build(_)));
    }
}
```

---

### **8.2. Tests de Integración**

**Framework:** `assert_cmd` + `predicates` (Rust).

```rust
#[test]
fn test_vela_run_hello_world() {
    Command::cargo_bin("vela")
        .unwrap()
        .args(&["run", "tests/fixtures/hello_world.vela"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}
```

---

## 9. Roadmap de Implementación

### **Fase 1: MVP (Sprint 2-3)**
- ✅ Comandos básicos: `build`, `run`, `test`
- ✅ Parsing de argumentos con `clap`
- ✅ Integración con compiler API

### **Fase 2: Tooling (Sprint 4-5)**
- ✅ Comandos: `fmt`, `lint`
- ✅ Package manager: `pkg install/update/remove`
- ✅ Sistema de configuración (`vela.yaml`)

### **Fase 3: DevTools (Sprint 6-7)**
- ✅ Comando `devtools`
- ✅ UI Inspector, Signal Graph, Profiler

### **Fase 4: Plugins (Sprint 8-9)**
- ✅ Sistema de plugins con dynamic libraries
- ✅ API pública para plugins

---

## 10. Referencias

- **clap**: https://docs.rs/clap/
- **Cargo CLI**: https://doc.rust-lang.org/cargo/
- **Deno CLI**: https://github.com/denoland/deno
- **Go CLI**: https://github.com/spf13/cobra

---

**Autor:** Vela Core Team  
**Revisión:** 2025-11-30  
**Versión:** 1.0
