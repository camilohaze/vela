# TASK-000J: Diseñar Arquitectura del Vela CLI

## 📋 Información General
- **Historia:** VELA-562 (Tooling Design - Phase 0)
- **Epic:** EPIC-00C: Tooling Design
- **Sprint:** 2
- **Estado:** Completado ✅
- **Prioridad:** P0 (Crítica)
- **Estimación:** 48 horas
- **Dependencias:** VELA-561 (especificaciones formales)

---

## 🎯 Objetivo

Diseñar la arquitectura completa del **Vela CLI**, la herramienta de línea de comandos principal para el desarrollo en Vela, incluyendo:

- **Comandos principales** (build, run, test, fmt, lint, doc, publish)
- **Sistema de configuración** (`vela.yaml` jerárquico)
- **Sistema de plugins** (extensibilidad)
- **Performance targets** (startup time, parallel compilation)
- **User experience** (mensajes de error, progress bars)

---

## 🏗️ Arquitectura del CLI

### 1. Comandos Principales

#### 1.1 Project Management

##### `vela new <name> [--template=<template>]`
**Propósito:** Crear nuevo proyecto Vela

**Opciones:**
- `--template`: Plantilla a usar (app, lib, cli, web, actor-system)
- `--no-git`: No inicializar repositorio git
- `--path`: Ruta donde crear proyecto

**Estructura generada:**
```
my-project/
├── vela.yaml               # Configuración del proyecto
├── src/
│   └── main.vela          # Punto de entrada
├── tests/
│   └── main_test.vela     # Tests
├── README.md
├── .gitignore
└── .vela/                 # Cache local
    └── build/
```

**Ejemplo:**
```bash
$ vela new my-app --template=web
✔ Created project 'my-app' (web template)
✔ Initialized git repository
✔ Created src/main.vela
✔ Created vela.yaml

Next steps:
  cd my-app
  vela run
```

---

##### `vela init`
**Propósito:** Inicializar proyecto Vela en directorio existente

**Comportamiento:**
- Crea `vela.yaml` interactivamente
- Detecta estructura existente
- No sobrescribe archivos

**Ejemplo:**
```bash
$ cd existing-project
$ vela init
? Project name: my-project
? Version: 0.1.0
? Author: Alice <alice@example.com>
? License: MIT
✔ Created vela.yaml
```

---

#### 1.2 Build & Run

##### `vela build [--release] [--target=<target>]`
**Propósito:** Compilar proyecto

**Opciones:**
- `--release`: Build optimizado (O3, sin debug info)
- `--debug`: Build con debug info (default)
- `--target`: Target platform (x86_64-linux, aarch64-macos, wasm32, etc.)
- `--jobs=<N>`: Número de jobs paralelos (default: num_cpus)
- `--verbose`: Output detallado

**Output:**
```bash
$ vela build --release
   Compiling vela-stdlib v1.0.0
   Compiling my-project v0.1.0
    Finished release [optimized] target(s) in 2.34s
```

**Artifacts:**
- Debug: `target/debug/my-project`
- Release: `target/release/my-project`

---

##### `vela run [file.vela] [-- <args>]`
**Propósito:** Ejecutar programa Vela

**Comportamiento:**
- Sin argumentos: ejecuta `src/main.vela`
- Con archivo: ejecuta ese archivo específico
- Compila automáticamente si hay cambios
- Pasa argumentos después de `--` al programa

**Ejemplo:**
```bash
$ vela run
   Compiling my-project v0.1.0
    Finished dev [unoptimized] target(s) in 0.45s
     Running target/debug/my-project
Hello, Vela!

$ vela run src/server.vela -- --port 8080
Server listening on :8080
```

---

##### `vela watch [command]`
**Propósito:** Ejecutar comando cuando archivos cambien

**Comportamiento:**
- Observa archivos `src/`, `tests/`
- Re-ejecuta comando al detectar cambios
- Debouncing de 500ms

**Ejemplo:**
```bash
$ vela watch run
Watching src/ for changes...
   Compiling my-project v0.1.0
    Finished dev target(s) in 0.45s
     Running target/debug/my-project
Hello, Vela!

[File changed: src/main.vela]
   Compiling my-project v0.1.0 (1 file changed)
    Finished dev target(s) in 0.12s (incremental)
     Running target/debug/my-project
Hello, World!  # Output actualizado
```

---

#### 1.3 Testing

##### `vela test [pattern] [--watch]`
**Propósito:** Ejecutar tests

**Opciones:**
- `pattern`: Filtrar tests por nombre (glob pattern)
- `--watch`: Re-ejecutar al cambiar archivos
- `--coverage`: Generar reporte de cobertura
- `--verbose`: Mostrar output de cada test
- `--parallel=<N>`: Ejecutar N tests en paralelo

**Output:**
```bash
$ vela test
   Compiling my-project v0.1.0
    Finished test target(s) in 0.34s
     Running 12 tests

test math::test_addition ... ok (0.001s)
test math::test_division ... ok (0.002s)
test math::test_division_by_zero ... ok (0.001s)
test string::test_concat ... ok (0.003s)
...

test result: ok. 12 passed; 0 failed; 0 ignored

$ vela test --coverage
Test coverage: 87.5% (350/400 lines)
Coverage report: target/coverage/index.html
```

---

##### `vela bench [pattern]`
**Propósito:** Ejecutar benchmarks

**Opciones:**
- `pattern`: Filtrar benchmarks
- `--baseline=<name>`: Comparar con baseline guardado
- `--save-baseline=<name>`: Guardar resultados como baseline

**Output:**
```bash
$ vela bench
   Compiling my-project v0.1.0 (bench mode)
    Finished bench target(s) in 1.23s
     Running 5 benchmarks

bench sort_1000_items     ... 1.234 ms ± 0.045 ms
bench hash_map_insert     ... 0.234 μs ± 0.012 μs
bench json_parse_large    ... 5.678 ms ± 0.123 ms

$ vela bench --save-baseline=v1.0
Saved baseline 'v1.0'

$ vela bench --baseline=v1.0
bench sort_1000_items     ... 1.156 ms (-6.3% faster) ✅
bench hash_map_insert     ... 0.245 μs (+4.7% slower) ⚠️
```

---

#### 1.4 Code Quality

##### `vela fmt [--check] [--write]`
**Propósito:** Formatear código

**Opciones:**
- `--check`: Solo verificar, no modificar (exit code 1 si hay diffs)
- `--write`: Aplicar cambios (default)
- `--config=<file>`: Usar archivo de config custom

**Ejemplo:**
```bash
$ vela fmt --check
Checking formatting...
✗ src/main.vela (not formatted)
✗ src/utils.vela (not formatted)
2 files need formatting

$ vela fmt
Formatting 2 files...
✔ src/main.vela
✔ src/utils.vela
```

**Estilo de formato:**
- 2 espacios de indentación (no tabs)
- 100 caracteres por línea (configurable)
- Trailing commas en listas multi-línea
- Espacios alrededor de operadores
- Estilo Rust-like (no JavaScript-like)

---

##### `vela lint [--fix]`
**Propósito:** Ejecutar linter (análisis estático)

**Opciones:**
- `--fix`: Auto-fix problemas simples
- `--deny=<lint>`: Tratar lint como error
- `--allow=<lint>`: Silenciar lint específico

**Output:**
```bash
$ vela lint
Checking 15 files...

warning: unused variable 'x'
  --> src/main.vela:10:5
   |
10 |     x: Number = 42
   |     ^ help: prefix with '_' if intentional: `_x`

warning: function 'oldApi' is deprecated
  --> src/utils.vela:25:3
   |
25 |   oldApi()
   |   ^^^^^^ use 'newApi' instead

2 warnings emitted

$ vela lint --fix
Fixed 1 issue automatically
1 warning remains
```

**Lints disponibles:**
- `unused-vars`: Variables no usadas
- `dead-code`: Código inalcanzable
- `deprecated`: APIs deprecadas
- `unsafe`: Uso de código unsafe
- `complexity`: Funciones muy complejas (cyclomatic complexity)
- `performance`: Sugerencias de performance
- `style`: Estilo no idiomático

---

#### 1.5 Documentation

##### `vela doc [--open] [--private]`
**Propósito:** Generar documentación HTML

**Opciones:**
- `--open`: Abrir en navegador después de generar
- `--private`: Incluir items privados
- `--no-deps`: No documentar dependencias

**Output:**
```bash
$ vela doc --open
Documenting my-project v0.1.0
 Documenting http v2.0.0
 Documenting json v1.5.0
    Finished documentation in 1.23s
   Generated target/doc/my-project/index.html
    Opening http://localhost:8000/my-project
```

**Formato de docs:**
```vela
/// Suma dos números
///
/// # Ejemplos
/// ```
/// result = add(2, 3)
/// assert(result == 5)
/// ```
///
/// # Panics
/// Esta función nunca hace panic.
fn add(a: Number, b: Number) -> Number {
  return a + b
}
```

---

#### 1.6 Package Management

##### `vela add <package>[@version]`
**Propósito:** Agregar dependencia

**Ejemplo:**
```bash
$ vela add http
   Updating vela-registry.io index
   Resolving dependencies...
      Added http v2.0.0
    Updated vela.yaml
    Updated vela.lock

$ vela add json@^1.5.0
      Added json v1.5.3

$ vela add --dev test-framework
      Added test-framework v3.0.0 (dev-dependency)
```

---

##### `vela remove <package>`
**Propósito:** Eliminar dependencia

**Ejemplo:**
```bash
$ vela remove http
    Removed http v2.0.0
    Updated vela.yaml
    Updated vela.lock
```

---

##### `vela update [package]`
**Propósito:** Actualizar dependencias

**Ejemplo:**
```bash
$ vela update
   Updating vela-registry.io index
   Resolving dependencies...
    Updating http v2.0.0 -> v2.1.0
    Updating json v1.5.3 -> v1.5.4
    Updated vela.lock

$ vela update http
    Updating http v2.0.0 -> v2.1.0
```

---

##### `vela publish [--dry-run]`
**Propósito:** Publicar package a registry

**Precondiciones:**
- Limpio working directory (git)
- Tests pasando
- Documentación generada
- Version tag en git

**Ejemplo:**
```bash
$ vela publish --dry-run
Validating package...
✔ Package name available: my-package
✔ Version 1.0.0 not published
✔ Git working directory clean
✔ All tests passing
✔ Documentation generated
✔ License file present

Would upload:
  my-package-1.0.0.tar.gz (245 KB)

$ vela publish
Uploading my-package v1.0.0 to vela-registry.io...
✔ Uploaded (245 KB in 0.8s)
✔ Published my-package v1.0.0

View at: https://vela-registry.io/packages/my-package
```

---

### 2. Sistema de Configuración

#### 2.1 Jerarquía de Configs

```
1. Global:    ~/.vela/config.yaml
2. Workspace: ./vela.yaml (workspace root)
3. Project:   ./vela.yaml (current directory)
4. CLI args:  --flag=value (máxima prioridad)
```

**Regla de merge:** Configs más específicos sobrescriben globales.

---

#### 2.2 Formato `vela.yaml`

```yaml
# Project metadata
name: my-project
version: 0.1.0
authors:
  - "Alice <alice@example.com>"
license: MIT
description: "A cool Vela project"
repository: "https://github.com/alice/my-project"
homepage: "https://my-project.io"
keywords: ["web", "api", "rest"]

# Build configuration
build:
  target: "x86_64-linux"        # Default target
  optimization: "debug"          # debug | release
  incremental: true              # Incremental compilation
  parallel: true                 # Parallel compilation
  jobs: 8                        # Number of parallel jobs

# Dependencies
dependencies:
  http: "^2.0.0"
  json: "^1.5.0"
  actors: "^1.0.0"

dev-dependencies:
  test-framework: "^3.0.0"

# Linter configuration
lint:
  deny:
    - unused-vars
    - dead-code
  allow:
    - complexity              # Allow complex functions
  max-complexity: 15

# Formatter configuration
format:
  indent: 2                   # Spaces per indent
  max-width: 100              # Max line width
  trailing-commas: true

# Test configuration
test:
  parallel: true
  timeout: 60                 # Seconds per test
  coverage-threshold: 80      # Fail if coverage < 80%

# Documentation configuration
doc:
  include-private: false
  include-dependencies: true

# Custom scripts
scripts:
  dev: "vela watch run"
  prod: "vela build --release && ./target/release/my-project"
  deploy: "vela publish && ./scripts/deploy.sh"
```

---

#### 2.3 Global Config (`~/.vela/config.yaml`)

```yaml
# Registry configuration
registry:
  url: "https://vela-registry.io"
  token: "${VELA_REGISTRY_TOKEN}"  # Environment variable

# Build defaults
build:
  jobs: 8
  incremental: true

# Editor integration
editor:
  vscode:
    extensions:
      - vela-lang.vela-vscode
  neovim:
    lsp-enabled: true

# Telemetry (opt-out)
telemetry:
  enabled: false
```

---

### 3. Sistema de Plugins

#### 3.1 Arquitectura de Plugins

**Ubicación:** `~/.vela/plugins/<plugin-name>/`

**Estructura:**
```
~/.vela/plugins/
├── my-plugin/
│   ├── plugin.yaml          # Metadata
│   ├── bin/
│   │   └── my-plugin        # Executable
│   └── lib/
│       └── plugin.so        # Shared library (opcional)
```

**Instalación:**
```bash
$ vela plugin install my-plugin
Downloading my-plugin v1.0.0...
✔ Installed to ~/.vela/plugins/my-plugin/
✔ Added command: vela my-plugin

$ vela my-plugin --help
my-plugin 1.0.0
A custom Vela plugin

USAGE:
    vela my-plugin [OPTIONS]
```

---

#### 3.2 Plugin API

**`plugin.yaml`:**
```yaml
name: my-plugin
version: 1.0.0
author: "Alice <alice@example.com>"
description: "Custom Vela plugin"

# Comandos que agrega
commands:
  - name: my-plugin
    description: "Run my custom plugin"
    args:
      - name: input
        type: string
        required: true

# Hooks que implementa
hooks:
  - pre-build     # Ejecutado antes de build
  - post-test     # Ejecutado después de tests
```

**Plugin executable contract:**
```bash
# Plugin recibe argumentos vía stdin (JSON)
$ echo '{"input": "value"}' | ~/.vela/plugins/my-plugin/bin/my-plugin

# Plugin retorna resultado vía stdout (JSON)
{
  "success": true,
  "output": "Plugin executed successfully"
}
```

---

#### 3.3 Built-in Plugins (Ideas)

- `vela-deploy`: Deploy a cloud providers (AWS, GCP, Azure)
- `vela-migrate`: Database migrations
- `vela-bundle`: Bundle web apps (webpack-style)
- `vela-docker`: Generar Dockerfiles optimizados

---

### 4. Performance Targets

#### 4.1 Startup Time

**Target:** < 50ms (cold), < 10ms (warm)

**Estrategia:**
- CLI binario en Rust (startup rápido)
- Lazy loading de subcommands
- Minimal dependencies (no LLVM en CLI)

**Benchmark:**
```bash
$ hyperfine "vela --version"
Benchmark: vela --version
  Time (mean ± σ):      12.3 ms ±   1.2 ms    [User: 8.1 ms, System: 4.2 ms]
  Range (min … max):    10.5 ms …  18.7 ms    100 runs
```

---

#### 4.2 Parallel Compilation

**Target:** Utilizar 100% de cores disponibles

**Estrategia:**
- Dependency graph parallelizable
- Per-module compilation (no monolithic)
- Rayon para paralelismo (Rust)

**Benchmark:**
```bash
# Compilar 100 módulos
$ time vela build --jobs=1
real    2m 30s

$ time vela build --jobs=8
real    0m 25s    # 6x speedup
```

---

#### 4.3 Incremental Compilation

**Target:** Solo recompilar módulos modificados

**Estrategia:**
- Hash de source files
- Dependency tracking
- Salsa-style caching

**Ejemplo:**
```bash
# Primera compilación
$ time vela build
real    10.5s

# Modificar 1 archivo
$ echo "// comment" >> src/utils.vela

# Recompilación incremental
$ time vela build
   Compiling my-project v0.1.0 (1 file changed)
    Finished dev target(s) in 0.45s  # 23x faster
```

---

### 5. User Experience

#### 5.1 Mensajes de Error

**Principios:**
- Específicos (no genéricos)
- Accionables (sugerir fix)
- Colorized (rojo para errores, amarillo para warnings)
- Con snippets de código

**Ejemplo:**
```bash
$ vela build
error: type mismatch
  --> src/main.vela:15:10
   |
15 |   result = "hello" + 42
   |            ^^^^^^^^^^^^ expected Number, found String
   |
   = help: consider using string interpolation: "${42}"
   = note: cannot add String and Number directly

error: could not compile `my-project`
```

---

#### 5.2 Progress Bars

**Para operaciones largas:**
```bash
$ vela build
   Compiling 15 dependencies...
  [=========>                    ] 35% (5/15) http v2.0.0
```

**Para downloads:**
```bash
$ vela add large-package
  Downloading large-package v1.0.0...
  [====================] 100% 245 MB / 245 MB (5.2 MB/s)
```

---

#### 5.3 Interactive Mode

**Para comandos que requieren input:**
```bash
$ vela publish
? Package name: my-package
? Version (current: 0.1.0): 1.0.0
? Confirm publish to vela-registry.io? (y/N) y
✔ Published my-package v1.0.0
```

---

### 6. CLI Framework Choice

**Decisión:** **Clap (Rust)**

**Razones:**
- ✅ Performance excelente (startup < 10ms)
- ✅ Argument parsing robusto
- ✅ Completions automáticos (bash, zsh, fish)
- ✅ Help messages generados automáticamente
- ✅ Validación de argumentos
- ✅ Usado por Cargo, ripgrep, fd

**Alternativas consideradas:**

| Framework | Lenguaje | Startup Time | Razón de rechazo |
|-----------|----------|--------------|------------------|
| Commander.js | Node.js | ~200ms | Demasiado lento |
| Click | Python | ~150ms | Demasiado lento |
| Cobra | Go | ~30ms | Menos ecosystem que Clap |

---

### 7. Implementación (Rust)

**Estructura del código:**
```rust
// src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "vela", version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create new Vela project
    New {
        /// Project name
        name: String,
        
        /// Template to use
        #[clap(long, default_value = "app")]
        template: String,
    },
    
    /// Build the project
    Build {
        /// Build in release mode
        #[clap(long)]
        release: bool,
        
        /// Number of parallel jobs
        #[clap(long, short = 'j')]
        jobs: Option<usize>,
    },
    
    /// Run the project
    Run {
        /// File to run
        file: Option<String>,
        
        /// Arguments to pass to program
        #[clap(last = true)]
        args: Vec<String>,
    },
    
    // ... otros comandos
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New { name, template } => {
            commands::new::execute(&name, &template);
        }
        Commands::Build { release, jobs } => {
            commands::build::execute(release, jobs);
        }
        Commands::Run { file, args } => {
            commands::run::execute(file, args);
        }
        // ...
    }
}
```

---

## ✅ Criterios de Aceptación

- [x] Comandos principales definidos (new, build, run, test, fmt, lint, doc, publish)
- [x] Sistema de configuración diseñado (vela.yaml jerárquico)
- [x] Sistema de plugins especificado
- [x] Performance targets establecidos (< 50ms startup, parallel compilation)
- [x] UX guidelines definidos (mensajes de error, progress bars)
- [x] CLI framework seleccionado (Clap)
- [x] Comparación con CLIs modernos (Cargo, npm, go)

---

## 🔗 Referencias

### CLIs de Referencia
- [Cargo (Rust)](https://doc.rust-lang.org/cargo/commands/index.html)
- [Go CLI](https://go.dev/ref/mod)
- [npm CLI](https://docs.npmjs.com/cli/v10)
- [Bun CLI](https://bun.sh/docs/cli)

### Frameworks
- [Clap (Rust)](https://docs.rs/clap/latest/clap/)
- [Commander.js](https://github.com/tj/commander.js)
- [Cobra (Go)](https://github.com/spf13/cobra)

### Performance Benchmarking
- [hyperfine](https://github.com/sharkdp/hyperfine)

---

**Estado:** ✅ Diseño completo  
**Prioridad:** P0 - Crítico para DX (Developer Experience)  
**Siguiente paso:** TASK-000K (Package Manager Architecture)
