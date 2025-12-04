/*!
# Vela Tooling

Complete toolchain for the Vela programming language, including:

- **CLI Tools**: Command-line interface for project management
- **Build System**: Incremental compilation with dependency tracking
- **Package Manager**: Dependency resolution and version management

## Quick Start

### CLI Usage

```bash
# Create new project
vela new my-project

# Build project
vela build

# Run project
vela run

# Add dependency
vela add http@^2.0
```

### Library Usage

```rust
use vela_tooling::build::{BuildExecutor, BuildConfig};
use vela_tooling::package::Manifest;

// Load project manifest
let manifest = Manifest::from_file("Vela.toml")?;

// Execute build
let config = BuildConfig::default();
let executor = BuildExecutor::new(config);
let result = executor.execute(&manifest)?;

println!("Build successful: {} modules compiled", result.modules_count);
```

## Architecture

### CLI Module

Command-line interface built with `clap` for argument parsing and `miette`
for pretty error messages.

### Build Module

Incremental build system with:
- Dependency graph analysis
- Parallel compilation using `rayon`
- Smart caching with SHA-256 hashing

### Package Module

Package management with:
- Semantic versioning support
- Dependency resolution (PubGrub algorithm)
- Registry client for package downloads

## Performance

| Operation | Target | Actual |
|-----------|--------|--------|
| CLI startup | < 50ms | TBD |
| Clean build (1K LOC) | < 500ms | TBD |
| Incremental rebuild | < 100ms | TBD |
| Dependency resolution | < 200ms | TBD |

## Examples

See `examples/` directory for complete examples:

- `simple_build.rs` - Basic build workflow
- `dependency_resolution.rs` - Package management
- `incremental_build.rs` - Incremental compilation

## Testing

Run tests:
```bash
cargo test -p vela-tooling
```

Run benchmarks:
```bash
cargo bench -p vela-tooling
```
*/

// Re-export main modules
pub mod cli;
pub mod build;
pub mod package;
pub mod common;

// Re-export commonly used types
pub use cli::{Cli, Commands};
pub use build::{BuildExecutor, BuildConfig, BuildResult};
pub use package::{Manifest, DependencyResolver, Registry};
pub use common::{Error, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_version() {
        let version = env!("CARGO_PKG_VERSION");
        assert_eq!(version, "0.1.0");
    }
}
