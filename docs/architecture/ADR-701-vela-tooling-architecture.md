# ADR-701: Vela Tooling Architecture

## Estado
✅ Aceptado

## Fecha
2025-01-15

## Contexto

Vela necesita un ecosistema completo de herramientas para el desarrollo, compilación, testing y gestión de paquetes. Este ADR define la arquitectura del crate `vela-tooling` que incluye:

1. **CLI Tools**: Comandos de línea de comandos (`vela new`, `vela build`, `vela run`, etc.)
2. **Build System**: Sistema de compilación incremental con dependency graph
3. **Package Manager**: Gestión de dependencias, versionado y registry client

El diseño debe ser modular, extensible y fácil de usar, inspirándose en las mejores prácticas de Cargo (Rust), npm (Node.js), pip (Python) y rustc CLI.

### Problemas a Resolver

1. **CLI Ergonomics**: Los comandos deben ser intuitivos, con autocompletado y mensajes de error claros
2. **Build Performance**: Compilación incremental rápida con cache inteligente
3. **Dependency Resolution**: Resolver dependencias con versionado semántico, evitando conflictos
4. **Cross-platform**: Funcionar en Windows, Linux y macOS
5. **Extensibility**: Permitir plugins y extensiones del sistema de build

### Referencias

- **Cargo (Rust)**: CLI ergonómico, build incremental, Cargo.toml/Cargo.lock
- **npm (Node.js)**: package.json, semantic versioning, lock files
- **pip (Python)**: requirements.txt, virtual environments
- **rustc**: Compilador CLI con flags detallados
- **clap**: Rust crate para argument parsing (usado por Cargo)
- **miette**: Pretty error reporting (usado por modern Rust tools)

---

## Decisión

### Arquitectura de Alto Nivel

```
vela-tooling/
├── cli/           # CLI commands y argument parsing
│   ├── commands/  # Comandos individuales (new, build, run, etc.)
│   ├── parser.rs  # Argument parsing con clap
│   └── output.rs  # Formatted output y error reporting
│
├── build/         # Build system
│   ├── graph.rs   # Dependency graph
│   ├── cache.rs   # Build cache y incremental compilation
│   ├── executor.rs # Parallel build execution
│   └── config.rs  # Build configuration
│
├── package/       # Package manager
│   ├── manifest.rs # Vela.toml parser
│   ├── lock.rs    # Vela.lock file
│   ├── resolver.rs # Dependency resolution
│   ├── registry.rs # Registry client (HTTP)
│   └── version.rs # Semantic versioning
│
└── common/        # Shared utilities
    ├── error.rs   # Error types
    ├── fs.rs      # File system utilities
    └── project.rs # Project structure detection
```

---

## 1. CLI Tools

### Comandos Principales

```bash
# Crear nuevo proyecto
vela new my-project --template web

# Compilar proyecto
vela build [--release] [--target <platform>]

# Ejecutar proyecto
vela run [--release] [-- <args>]

# Testing
vela test [--filter <pattern>]

# Formateo de código
vela fmt [--check]

# Linting
vela lint [--fix]

# Gestión de dependencias
vela add <package>[@<version>]
vela remove <package>
vela update [<package>]

# Información del proyecto
vela version
vela info
```

### Argument Parsing con `clap`

Usar `clap` v4 con derive API para type-safe argument parsing:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vela")]
#[command(about = "Vela language toolchain", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Vela project
    New {
        /// Project name
        name: String,
        
        /// Project template
        #[arg(short, long, default_value = "bin")]
        template: String,
    },
    
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
        
        /// Target platform
        #[arg(short, long)]
        target: Option<String>,
    },
    
    /// Run the project
    Run {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
        
        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },
    
    // ... otros comandos
}
```

### Error Reporting

Usar `miette` para pretty error messages:

```rust
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Failed to parse Vela.toml")]
#[diagnostic(
    code(vela::manifest::parse_error),
    help("Check your Vela.toml syntax")
)]
struct ManifestParseError {
    #[source_code]
    src: String,
    
    #[label("Invalid TOML here")]
    span: SourceSpan,
}
```

---

## 2. Build System

### Build Process

```
1. Parse Vela.toml (manifest)
2. Resolve dependencies
3. Build dependency graph
4. Incremental compilation check
5. Parallel execution of build tasks
6. Cache artifacts
```

### Dependency Graph

```rust
pub struct BuildGraph {
    nodes: HashMap<ModuleId, ModuleNode>,
    edges: HashMap<ModuleId, HashSet<ModuleId>>,
}

pub struct ModuleNode {
    id: ModuleId,
    path: PathBuf,
    last_modified: SystemTime,
    artifact: Option<Artifact>,
}

impl BuildGraph {
    /// Build topological order for parallel execution
    pub fn topological_sort(&self) -> Result<Vec<Vec<ModuleId>>> {
        // Return levels of independent modules that can be built in parallel
    }
    
    /// Check if module needs rebuild
    pub fn needs_rebuild(&self, id: ModuleId) -> bool {
        // Compare timestamps, check if dependencies changed
    }
}
```

### Incremental Compilation

- **Cache Strategy**: Hash de contenido (SHA-256) para detectar cambios
- **Granularity**: Por archivo `.vela`
- **Cache Location**: `target/cache/` (similar a Cargo)
- **Invalidation**: Cambios en archivo o dependencias

```rust
pub struct BuildCache {
    entries: HashMap<PathBuf, CacheEntry>,
}

pub struct CacheEntry {
    source_hash: Hash,
    artifact: Artifact,
    dependencies: HashSet<PathBuf>,
    timestamp: SystemTime,
}

impl BuildCache {
    /// Check if cache is valid
    pub fn is_valid(&self, path: &Path) -> bool {
        // Compare hash, check dependencies
    }
    
    /// Store artifact in cache
    pub fn store(&mut self, path: &Path, artifact: Artifact) {
        // Compute hash, save artifact
    }
}
```

### Parallel Execution

Usar `rayon` para parallel builds:

```rust
use rayon::prelude::*;

pub struct BuildExecutor {
    graph: BuildGraph,
    cache: BuildCache,
}

impl BuildExecutor {
    /// Execute build in parallel
    pub fn execute(&self) -> Result<BuildResult> {
        let levels = self.graph.topological_sort()?;
        
        for level in levels {
            // Compile all modules in this level in parallel
            level.par_iter()
                .map(|&id| self.compile_module(id))
                .collect::<Result<Vec<_>>>()?;
        }
        
        Ok(BuildResult::success())
    }
}
```

---

## 3. Package Manager

### Manifest Format (`Vela.toml`)

Similar a `Cargo.toml`:

```toml
[package]
name = "my-project"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2025"
license = "MIT OR Apache-2.0"

[dependencies]
http = "^2.1.0"
json = "1.0"
reactive = { version = "0.6", features = ["async"] }

[dev-dependencies]
test-utils = "0.1"

[build-dependencies]
codegen = "1.0"

[features]
default = ["async"]
async = []
experimental = []

[[bin]]
name = "my-app"
path = "src/main.vela"

[profile.release]
opt-level = 3
```

### Lock File (`Vela.lock`)

```toml
# This file is automatically generated by Vela.
# Do not edit manually.

[[package]]
name = "http"
version = "2.1.0"
source = "registry+https://registry.velalang.org"
checksum = "sha256:..."

dependencies = [
    "json 1.0.0",
]

[[package]]
name = "json"
version = "1.0.0"
source = "registry+https://registry.velalang.org"
checksum = "sha256:..."
```

### Dependency Resolution

Algoritmo de resolución basado en **Semantic Versioning** (semver):

```rust
pub struct DependencyResolver {
    registry: Registry,
    cache: PackageCache,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    name: String,
    version_req: VersionReq, // "^2.1.0", ">=1.0, <2.0"
    features: Vec<String>,
}

impl DependencyResolver {
    /// Resolve dependencies to concrete versions
    pub fn resolve(&self, root: &Manifest) -> Result<DependencyGraph> {
        // PubGrub algorithm (usado por Cargo)
        // https://github.com/pubgrub-rs/pubgrub
    }
    
    /// Check for conflicts
    pub fn check_conflicts(&self, graph: &DependencyGraph) -> Result<()> {
        // Detect version conflicts, circular dependencies
    }
}
```

### Semantic Versioning

```rust
use semver::{Version, VersionReq};

pub struct VersionManager;

impl VersionManager {
    /// Check if version satisfies requirement
    pub fn matches(version: &Version, req: &VersionReq) -> bool {
        req.matches(version)
    }
    
    /// Find best matching version
    pub fn find_best_match(
        versions: &[Version],
        req: &VersionReq
    ) -> Option<Version> {
        versions.iter()
            .filter(|v| req.matches(v))
            .max()
            .cloned()
    }
}
```

### Registry Client

```rust
use reqwest::blocking::Client;

pub struct Registry {
    url: String,
    client: Client,
}

impl Registry {
    /// Fetch package metadata
    pub fn fetch_metadata(&self, name: &str) -> Result<PackageMetadata> {
        let url = format!("{}/api/v1/crates/{}", self.url, name);
        let resp = self.client.get(&url).send()?;
        Ok(resp.json()?)
    }
    
    /// Download package tarball
    pub fn download_package(
        &self,
        name: &str,
        version: &Version
    ) -> Result<Vec<u8>> {
        let url = format!(
            "{}/api/v1/crates/{}/{}/download",
            self.url, name, version
        );
        let resp = self.client.get(&url).send()?;
        Ok(resp.bytes()?.to_vec())
    }
}
```

---

## Performance Characteristics

### CLI Startup Time

- **Target**: < 50ms from invocation to first output
- **Strategy**: 
  - Lazy loading de subcommands
  - Minimize dependencies in main binary
  - Use `cargo-bloat` para detectar binaries grandes

### Build Time

| Operation | Target | Strategy |
|-----------|--------|----------|
| Clean build (1K LOC) | < 500ms | Parallel compilation |
| Incremental rebuild (1 file changed) | < 100ms | Smart caching |
| Dependency resolution | < 200ms | Local cache + HTTP/2 |
| Lock file generation | < 50ms | Efficient serialization |

### Memory Usage

- **CLI commands**: < 50 MB RAM
- **Build process**: O(n) where n = number of modules
- **Cache size**: Configurable, default 1 GB

---

## Dependencies

### Core Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "color"] }
miette = { version = "7.0", features = ["fancy"] }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
semver = "1.0"
reqwest = { version = "0.12", features = ["blocking", "json"] }
rayon = "1.10"
sha2 = "0.10"
walkdir = "2.5"
tempfile = "3.10"
```

### Rationale

- **clap**: Industry-standard CLI parsing (usado por Cargo, ripgrep)
- **miette**: Pretty error messages con source code context
- **serde + toml**: Parse Vela.toml/Vela.lock
- **semver**: Semantic versioning (spec-compliant)
- **reqwest**: HTTP client para registry
- **rayon**: Data parallelism para builds
- **sha2**: Hashing para cache validation

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dependency_resolution() {
        let resolver = DependencyResolver::new(mock_registry());
        let manifest = mock_manifest();
        
        let graph = resolver.resolve(&manifest).unwrap();
        assert_eq!(graph.packages.len(), 5);
    }
    
    #[test]
    fn test_build_cache_hit() {
        let cache = BuildCache::new();
        let path = Path::new("test.vela");
        
        cache.store(path, mock_artifact());
        assert!(cache.is_valid(path));
    }
}
```

### Integration Tests

```rust
#[test]
fn test_vela_new_command() {
    let temp = TempDir::new().unwrap();
    
    let output = Command::new("vela")
        .args(&["new", "my-project"])
        .current_dir(&temp)
        .output()
        .unwrap();
    
    assert!(output.status.success());
    assert!(temp.path().join("my-project/Vela.toml").exists());
}

#[test]
fn test_full_build_workflow() {
    // Create project, add dependencies, build, run
    let temp = TempDir::new().unwrap();
    
    // 1. vela new
    run_command(&["new", "test-project"], &temp);
    
    let project_dir = temp.path().join("test-project");
    
    // 2. vela add
    run_command(&["add", "json"], &project_dir);
    
    // 3. vela build
    let output = run_command(&["build"], &project_dir);
    assert!(output.status.success());
    
    // 4. vela run
    let output = run_command(&["run"], &project_dir);
    assert!(output.status.success());
}
```

### Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_dependency_resolution(c: &mut Criterion) {
    let resolver = DependencyResolver::new(mock_registry());
    let manifest = large_manifest(); // 100 dependencies
    
    c.bench_function("resolve 100 deps", |b| {
        b.iter(|| resolver.resolve(black_box(&manifest)))
    });
}

fn bench_incremental_build(c: &mut Criterion) {
    let executor = setup_build_executor();
    
    c.bench_function("incremental rebuild 1 file", |b| {
        b.iter(|| executor.execute_incremental(black_box("main.vela")))
    });
}

criterion_group!(benches, bench_dependency_resolution, bench_incremental_build);
criterion_main!(benches);
```

---

## File Structure

```
tooling/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                 # Crate root
│   │
│   ├── cli/                   # CLI commands
│   │   ├── mod.rs
│   │   ├── parser.rs          # clap argument parsing
│   │   ├── output.rs          # Formatted output
│   │   └── commands/
│   │       ├── mod.rs
│   │       ├── new.rs         # vela new
│   │       ├── build.rs       # vela build
│   │       ├── run.rs         # vela run
│   │       ├── test.rs        # vela test
│   │       ├── fmt.rs         # vela fmt
│   │       ├── lint.rs        # vela lint
│   │       └── package.rs     # vela add/remove/update
│   │
│   ├── build/                 # Build system
│   │   ├── mod.rs
│   │   ├── graph.rs           # Dependency graph
│   │   ├── cache.rs           # Build cache
│   │   ├── executor.rs        # Build executor
│   │   └── config.rs          # Build configuration
│   │
│   ├── package/               # Package manager
│   │   ├── mod.rs
│   │   ├── manifest.rs        # Vela.toml parser
│   │   ├── lock.rs            # Vela.lock
│   │   ├── resolver.rs        # Dependency resolution
│   │   ├── registry.rs        # Registry client
│   │   └── version.rs         # Semver utilities
│   │
│   └── common/                # Shared utilities
│       ├── mod.rs
│       ├── error.rs           # Error types
│       ├── fs.rs              # File system utilities
│       └── project.rs         # Project structure
│
├── tests/                     # Integration tests
│   ├── cli_tests.rs
│   ├── build_tests.rs
│   └── package_tests.rs
│
└── benches/                   # Benchmarks
    ├── cli_bench.rs
    ├── build_bench.rs
    └── resolver_bench.rs
```

---

## Consecuencias

### Positivas

1. **CLI Ergonomics**: clap + miette → excellent UX
2. **Performance**: Incremental builds + parallel execution → fast
3. **Reliability**: Semantic versioning + lock files → reproducible builds
4. **Extensibility**: Modular design → plugins fáciles de agregar
5. **Cross-platform**: Pure Rust → Windows/Linux/macOS sin cambios
6. **Industry Standards**: Usa best practices de Cargo, npm, pip

### Negativas

1. **Complexity**: Dependency resolution es NP-hard (PubGrub ayuda)
2. **Registry Dependency**: Requiere registry server (similar a crates.io)
3. **Binary Size**: CLI binary puede ser grande (~10MB) con todas las deps
4. **Learning Curve**: Los usuarios deben aprender nuevo toolchain

### Trade-offs

| Aspecto | Decisión | Alternativa | Razón |
|---------|----------|-------------|-------|
| Argument Parsing | clap | structopt, argh | clap v4 tiene mejor derive API |
| Error Reporting | miette | anyhow, eyre | miette tiene fancy formatting |
| HTTP Client | reqwest | ureq, curl | reqwest es async-ready y popular |
| Parallelism | rayon | tokio tasks | rayon es más simple para CPU-bound |
| Dependency Resolution | PubGrub | SAT solver | PubGrub es usado por Cargo |

---

## Alternativas Consideradas

### 1. Python-based CLI (Rechazada)

**Pros**:
- Fácil de prototipar
- Ecosistema rico (click, typer)

**Cons**:
- Slower startup time (Python interpreter)
- Requires Python installation
- Harder to distribute

**Decisión**: Rechazada. Rust CLI es más performante y fácil de distribuir.

### 2. Custom Dependency Resolver (Rechazada)

**Pros**:
- Control total del algoritmo

**Cons**:
- Complejo, bug-prone
- PubGrub es probado (usado por Cargo)

**Decisión**: Rechazada. Usar PubGrub es mejor opción.

### 3. Binary Package Format (Postponed)

**Pros**:
- Faster installation (no need to compile)

**Cons**:
- Platform-specific binaries
- Storage overhead

**Decisión**: Postponed para futura iteración. Por ahora, solo source packages.

---

## Referencias

### Especificaciones

- [Semantic Versioning 2.0.0](https://semver.org/)
- [TOML v1.0.0](https://toml.io/en/v1.0.0)
- [PubGrub Algorithm](https://github.com/pubgrub-rs/pubgrub)

### Implementaciones de Referencia

- [Cargo Source Code](https://github.com/rust-lang/cargo)
- [npm CLI](https://github.com/npm/cli)
- [pip](https://github.com/pypa/pip)
- [clap Examples](https://github.com/clap-rs/clap/tree/master/examples)

### Documentación

- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [clap Documentation](https://docs.rs/clap/)
- [miette Documentation](https://docs.rs/miette/)
- [semver Specification](https://docs.rs/semver/)

---

## Implementación

Ver código en:
- `tooling/src/cli/` - CLI commands
- `tooling/src/build/` - Build system
- `tooling/src/package/` - Package manager
- `tooling/tests/` - Integration tests
- `tooling/benches/` - Benchmarks

## Líneas de Código Estimadas

- **CLI**: ~800 LOC
- **Build System**: ~1,200 LOC
- **Package Manager**: ~1,500 LOC
- **Common Utilities**: ~500 LOC
- **Tests**: ~1,000 LOC
- **Total**: ~5,000 LOC

## Métricas de Calidad

- **Test Coverage**: >= 80%
- **Doc Coverage**: 100% (public API)
- **Clippy Warnings**: 0
- **MSRV**: Rust 1.75+
