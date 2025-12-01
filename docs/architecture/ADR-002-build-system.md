# ADR-002: Definir Arquitectura del Build System

## Estado
✅ Aceptado

## Fecha
2025-11-30

## Contexto

El compilador de Vela necesita un build system robusto que:

- Gestione dependencias de manera eficiente
- Soporte builds incrementales
- Sea multiplataforma
- Integre con CI/CD fácilmente
- Proporcione herramientas de desarrollo (testing, linting, formatting)

Dado que decidimos implementar en Rust (ADR-001), necesitamos definir la arquitectura del build system.

## Decisión

**Se decide utilizar Cargo como build system principal del proyecto Vela.**

### Estructura del proyecto:

```
vela/
├── Cargo.toml          # Workspace root
├── Cargo.lock          # Lockfile
├── crates/
│   ├── vela-compiler/  # Compilador principal
│   ├── vela-parser/    # Parser
│   ├── vela-ast/       # Abstract Syntax Tree
│   ├── vela-codegen/   # Generación de código
│   ├── vela-runtime/   # Runtime library
│   └── vela-cli/       # Command-line interface
└── examples/           # Ejemplos de código Vela
```

### Comandos principales:

```bash
cargo build          # Compilar proyecto
cargo test           # Ejecutar tests
cargo run            # Ejecutar compilador
cargo fmt            # Formatear código
cargo clippy         # Linter
cargo doc            # Generar documentación
```

## Consecuencias

### Positivas

- ✅ Build system oficial de Rust (estándar de facto)
- ✅ Gestión automática de dependencias
- ✅ Builds incrementales out-of-the-box
- ✅ Integración nativa con crates.io
- ✅ Tooling integrado (fmt, clippy, doc, test)
- ✅ Workspaces para organizar múltiples crates
- ✅ Excelente soporte en IDEs y CI/CD

### Negativas

- ⚠️ Tiempos de compilación inicial pueden ser largos
- ⚠️ Lockfile debe mantenerse en Git (pero es buena práctica)

## Alternativas Consideradas

### 1. Make/CMake
**Rechazada porque:**
- No gestiona dependencias automáticamente
- Menos integración con ecosistema Rust
- Mayor complejidad de configuración
- No proporciona tooling integrado

### 2. Bazel
**Rechazada porque:**
- Overkill para un proyecto de este tamaño inicial
- Mayor complejidad de configuración
- Menos adopción en comunidad Rust

### 3. Custom build scripts
**Rechazada porque:**
- Reinventar la rueda
- Mayor mantenimiento
- Peor integración con herramientas

## Referencias

- **Jira**: VELA-1196 (TASK-000B)
- **Historia**: VELA-560 (US-00A)
- **Documentación**: https://doc.rust-lang.org/cargo/

## Implementación

```toml
# Cargo.toml - Configuración del workspace

[workspace]
members = [
    "crates/vela-compiler",
    "crates/vela-parser",
    "crates/vela-ast",
    "crates/vela-codegen",
    "crates/vela-runtime",
    "crates/vela-cli"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Vela Team"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# Dependencias comunes
clap = "4.0"
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
```

Ver implementación en: `Cargo.toml` (a crear en futuras Historias)

---

*ADR creado: 2025-11-30*  
*Sprint: 0*
