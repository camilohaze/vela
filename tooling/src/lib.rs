/*!
# Vela Tooling

Complete toolchain for the Vela programming language, including:

- **CLI Tools**: Command-line interface for project management
- **Build System**: Incremental compilation with dependency tracking
- **LSP Server**: Language server for IDE integration

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

// Execute build
let config = BuildConfig::default();
let executor = BuildExecutor::new(config);
let result = executor.execute()?;

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

## Performance

| Operation | Target | Actual |
|-----------|--------|--------|
| CLI startup | < 50ms | TBD |
| Clean build (1K LOC) | < 500ms | TBD |
| Incremental rebuild | < 100ms | TBD |
| LSP response | < 50ms | TBD |

## Examples

See `examples/` directory for complete examples:

- `simple_build.rs` - Basic build workflow
- `incremental_build.rs` - Incremental compilation
- `lsp_server.rs` - Language server integration

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
pub mod lsp;
pub mod common;
pub mod package;

// Re-export commonly used types
pub use cli::{Cli, Commands};
pub use build::{BuildExecutor, BuildConfig, BuildResult};
pub use lsp::LanguageServer;
pub use common::{Error, Result};
pub use package::PackageManager;

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_version() {
        let version = env!("CARGO_PKG_VERSION");
        assert_eq!(version, "0.1.0");
    }
}
