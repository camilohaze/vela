# Vela Tooling

Complete toolchain for the Vela programming language, providing command-line tools, build system, and package management.

[![Tests](https://img.shields.io/badge/tests-83%20passing-brightgreen)](.)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](.)

## Overview

`vela-tooling` provides the complete development toolchain for Vela:

- **CLI Tools**: Command-line interface for project management (`vela new`, `vela build`, `vela run`, etc.)
- **Build System**: Incremental compilation with dependency graph analysis and parallel execution
- **Package Manager**: Dependency resolution with semantic versioning

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vela-tooling = "0.1.0"
```

## Quick Start

### CLI Usage

```bash
# Create a new project
vela new my-project

# Navigate to project
cd my-project

# Build the project
vela build

# Run the project
vela run

# Run tests
vela test

# Format code
vela fmt

# Lint code
vela lint

# Add a dependency
vela add http@^2.0

# Update dependencies
vela update
```

### Library Usage

```rust
use vela_tooling::build::{BuildExecutor, BuildConfig};
use vela_tooling::package::Manifest;

// Load project manifest
let manifest = Manifest::from_file("Vela.toml")?;
println!("Project: {} v{}", manifest.package.name, manifest.package.version);

// Configure and execute build
let config = BuildConfig::new()
    .with_release(true)
    .with_jobs(4);

let mut executor = BuildExecutor::new(config);

// Add modules to build graph
let graph = executor.graph_mut();
let main_id = graph.add_module(PathBuf::from("src/main.vela"));

// Execute build
let result = executor.execute()?;
println!("Build completed: {} modules compiled in {}ms", 
    result.modules_compiled, result.duration_ms);
```

## Architecture

### CLI Module

Command-line interface built with:
- **clap**: Type-safe argument parsing with derive API
- **miette**: Pretty error messages with source context

Available commands:
- `new` - Create new project
- `build` - Compile project
- `run` - Execute project
- `test` - Run tests
- `fmt` - Format code
- `lint` - Lint code
- `add` - Add dependency
- `remove` - Remove dependency
- `update` - Update dependencies
- `version` - Show version
- `info` - Show project info

### Build Module

Incremental build system with:
- **Dependency Graph**: Topological sort for correct build order
- **Parallel Compilation**: Using `rayon` for multi-threaded builds
- **Smart Caching**: SHA-256 hashing for change detection
- **Incremental Builds**: Only recompile changed modules

Components:
- `BuildGraph` - Dependency graph with cycle detection
- `BuildCache` - File-based cache with timestamp validation
- `BuildExecutor` - Parallel build execution engine
- `BuildConfig` - Build configuration (release mode, target, jobs)

### Package Module

Package management with:
- **Manifest Parser**: Parse `Vela.toml` configuration
- **Semantic Versioning**: Using `semver` crate (^, ~, >= operators)
- **Dependency Resolution**: Stub for PubGrub algorithm
- **Registry Client**: HTTP client for package downloads

Components:
- `Manifest` - Vela.toml parser/serializer
- `DependencyResolver` - Dependency resolution engine (stub)
- `Registry` - Package registry client
- `Version` - Semantic versioning utilities

## Vela.toml Format

```toml
[package]
name = "my-project"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2025"
license = "MIT OR Apache-2.0"
description = "My Vela project"

[dependencies]
http = "^2.1.0"
json = "1.0"
reactive = { version = "0.6", features = ["async"] }

[dev-dependencies]
test-utils = "0.1"

[features]
default = ["async"]
async = []
```

## Performance

| Operation | Target | Status |
|-----------|--------|--------|
| CLI startup | < 50ms | TODO |
| Clean build (1K LOC) | < 500ms | TODO |
| Incremental rebuild | < 100ms | TODO |
| Dependency resolution | < 200ms | TODO |

## Testing

Run all tests:

```bash
cargo test -p vela-tooling
```

Run specific test module:

```bash
cargo test -p vela-tooling build::cache
```

Current test coverage: **83 passing tests**

### Test Breakdown

| Module | Tests | Coverage |
|--------|-------|----------|
| `build::cache` | 8 | Cache validation, storage, invalidation |
| `build::config` | 4 | Configuration builder pattern |
| `build::executor` | 8 | Build execution, parallel compilation |
| `build::graph` | 7 | Dependency graph, topological sort |
| `cli::parser` | 6 | Argument parsing, CLI commands |
| `cli::commands` | 4 | Command execution |
| `common::error` | 10 | Error types and conversions |
| `common::fs` | 10 | File system operations |
| `common::project` | 8 | Project structure detection |
| `package::manifest` | 6 | Manifest parsing, serialization |
| `package::registry` | 4 | Registry client (stub) |
| `package::resolver` | 3 | Dependency resolution (stub) |
| `package::version` | 5 | Semantic versioning |

## Examples

### Creating a New Project

```rust
use vela_tooling::cli::commands;

// Create new project
commands::execute_new("my-app", "bin", Some("/path/to/projects"))?;
```

### Building with Custom Config

```rust
use vela_tooling::build::{BuildConfig, BuildExecutor};

let config = BuildConfig::new()
    .with_release(true)
    .with_target("linux-x64")
    .with_jobs(8)
    .with_incremental(true);

let executor = BuildExecutor::new(config);
let result = executor.execute()?;

if result.success {
    println!("✅ Build successful!");
} else {
    eprintln!("❌ Build failed!");
}
```

### Managing Dependencies

```rust
use vela_tooling::package::Manifest;

// Load manifest
let mut manifest = Manifest::from_file("Vela.toml")?;

// Add dependency
manifest.add_dependency("http", "^2.0");

// Remove dependency
manifest.remove_dependency("old-lib");

// Save changes
manifest.to_file("Vela.toml")?;
```

### Working with Build Cache

```rust
use vela_tooling::build::BuildCache;
use std::path::PathBuf;

let mut cache = BuildCache::new();

// Store compiled module
let path = PathBuf::from("src/main.vela");
cache.store(path.clone(), vec![])?;

// Check if cache is valid
if cache.is_valid(&path)? {
    println!("✅ Cache hit - skipping compilation");
} else {
    println!("❌ Cache miss - recompiling");
}
```

### Dependency Graph Analysis

```rust
use vela_tooling::build::BuildGraph;
use std::path::PathBuf;

let mut graph = BuildGraph::new();

// Add modules
let main = graph.add_module(PathBuf::from("src/main.vela"));
let lib = graph.add_module(PathBuf::from("src/lib.vela"));
let utils = graph.add_module(PathBuf::from("src/utils.vela"));

// Add dependencies (main depends on lib and utils)
graph.add_dependency(main, lib);
graph.add_dependency(main, utils);

// Get build order
let levels = graph.topological_sort()?;
println!("Build order: {} levels", levels.len());

// Level 0: lib and utils (can build in parallel)
// Level 1: main (depends on level 0)
```

## Dependencies

### Core Dependencies

- `clap` 4.5 - CLI argument parsing
- `miette` 7.0 - Pretty error messages
- `thiserror` 1.0 - Error derive macros
- `serde` 1.0 - Serialization framework
- `toml` 0.8 - TOML parser for Vela.toml
- `semver` 1.0 - Semantic versioning
- `reqwest` 0.12 - HTTP client for registry
- `rayon` 1.10 - Data parallelism
- `sha2` 0.10 - SHA-256 hashing
- `walkdir` 2.5 - Directory traversal
- `tempfile` 3.10 - Temporary files for testing

### Dev Dependencies

- `criterion` 0.5 - Benchmarking framework

## Future Work

### Phase 1: Complete Implementation
- [ ] Implement actual compilation in `BuildExecutor`
- [ ] Implement PubGrub dependency resolution
- [ ] Implement lock file generation (Vela.lock)
- [ ] Add benchmarks for performance validation

### Phase 2: Advanced Features
- [ ] Watch mode for automatic rebuilds
- [ ] Distributed caching
- [ ] Plugin system
- [ ] Build profiles (dev, release, test)

### Phase 3: Tooling Integration
- [ ] LSP integration for build diagnostics
- [ ] IDE extensions
- [ ] CI/CD templates
- [ ] Docker support

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

Licensed under MIT OR Apache-2.0. See [LICENSE-MIT](../LICENSE-MIT) and [LICENSE-APACHE](../LICENSE-APACHE).

## References

### Architecture
- **ADR-701**: [Vela Tooling Architecture](../docs/architecture/ADR-701-vela-tooling-architecture.md)

### Inspiration
- [Cargo](https://doc.rust-lang.org/cargo/) - Rust package manager
- [npm](https://www.npmjs.com/) - Node.js package manager
- [pip](https://pip.pypa.io/) - Python package installer

### Algorithms
- [PubGrub](https://github.com/pubgrub-rs/pubgrub) - Dependency resolution algorithm
- [Topological Sort](https://en.wikipedia.org/wiki/Topological_sorting) - Build order determination

## Metrics

- **Lines of Code**: ~2,500
- **Test Coverage**: 83 tests passing
- **Modules**: 15 (cli, build, package, common)
- **Dependencies**: 11 production + 1 dev
- **MSRV**: Rust 1.75+
