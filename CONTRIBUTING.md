# Contributing to Vela

Thank you for your interest in contributing to **Vela**! This document provides guidelines and information to help you contribute effectively.

---

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)
- [Community](#community)

---

## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before contributing.

---

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust** 1.75+ (stable)
- **LLVM** 17+ (for native compilation)
- **Node.js** 18+ (for DevTools UI)
- **Git** 2.40+
- **PostgreSQL** 15+ (for package registry development)

### Installing Dependencies

```bash
# Clone the repository
git clone https://github.com/velalang/vela.git
cd vela

# Install Rust toolchain
rustup install stable
rustup default stable

# Install development tools
rustup component add rustfmt clippy

# Install Node.js dependencies (for DevTools)
cd devtools
npm install
cd ..

# Build the project
cargo build
```

---

## Development Setup

### IDE Configuration

We recommend using **Visual Studio Code** with the following extensions:

- **rust-analyzer**: Rust language server
- **CodeLLDB**: Debugger
- **Even Better TOML**: TOML syntax highlighting
- **Error Lens**: Inline error display

### Environment Variables

Create a `.env` file in the project root:

```bash
# Vela configuration
VELA_HOME=$HOME/.vela
VELA_CACHE=$HOME/.vela/cache
VELA_LOG=debug

# LLVM configuration
LLVM_SYS_170_PREFIX=/usr/lib/llvm-17  # Adjust for your system

# Database (for package registry development)
DATABASE_URL=postgres://vela:password@localhost/vela_registry
```

### Building Components

```bash
# Build all workspace members
cargo build --workspace

# Build specific component
cargo build --package vela-cli

# Build with optimizations
cargo build --release

# Build DevTools UI
cd devtools
npm run build
cd ..
```

---

## Project Structure

```
vela/
├── compiler/           # Vela compiler
│   ├── lexer/          # Lexical analyzer
│   ├── parser/         # Syntax parser
│   ├── semantic/       # Semantic analyzer
│   └── codegen/        # Code generation
├── vm/                 # VelaVM (bytecode interpreter)
├── stdlib/             # Standard library
├── cli/                # CLI tool
├── lsp/                # Language Server Protocol
├── devtools/           # DevTools (UI + Agent)
├── docs/               # Documentation
│   ├── architecture/   # ADRs
│   ├── specifications/ # Formal specs
│   └── tooling/        # Tooling docs
├── tests/              # Tests
│   ├── unit/           # Unit tests
│   ├── integration/    # Integration tests
│   └── e2e/            # End-to-end tests
├── .github/            # GitHub workflows
├── Cargo.toml          # Workspace config
└── README.md           # Project overview
```

---

## Development Workflow

### 1. Create an Issue

Before starting work, create or find an existing issue in [GitHub Issues](https://github.com/velalang/vela/issues) or [Jira](https://velalang.atlassian.net).

### 2. Create a Branch

```bash
# Feature branch
git checkout -b feature/VELA-XXX-short-description

# Bug fix branch
git checkout -b fix/VELA-XXX-short-description

# Documentation branch
git checkout -b docs/VELA-XXX-short-description
```

### 3. Make Changes

Follow the [Coding Standards](#coding-standards) and write tests for your changes.

### 4. Run Tests

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test --package vela-lexer

# Run with output
cargo test -- --nocapture

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings
```

### 5. Commit Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "feat(compiler): add support for async functions

- Implemented async/await syntax parsing
- Added async function code generation
- Updated semantic analyzer for async validation

Refs: VELA-123"
```

**Commit Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### 6. Push and Create PR

```bash
git push origin feature/VELA-XXX-short-description
```

Then create a Pull Request on GitHub using the PR template.

---

## Coding Standards

### Rust Style Guide

Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/):

- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Maximum line length: **100 characters**
- Use 4 spaces for indentation
- Use snake_case for functions and variables
- Use PascalCase for types and traits

### Code Quality

- **No `unsafe` code** without explicit approval
- **Error handling**: Use `Result<T, E>` and `?` operator
- **Avoid panics**: Use `expect()` with descriptive messages
- **Documentation**: All public items must have doc comments
- **Type safety**: Prefer strong types over primitives

### Example

```rust
/// Parses a Vela source file and returns an Abstract Syntax Tree.
///
/// # Arguments
///
/// * `source` - The source code to parse
/// * `filename` - The name of the file being parsed
///
/// # Returns
///
/// * `Ok(Ast)` - Successfully parsed AST
/// * `Err(ParseError)` - Syntax error with location
///
/// # Examples
///
/// ```
/// use vela_parser::parse;
///
/// let source = "fn main() { println(\"Hello\"); }";
/// let ast = parse(source, "main.vela")?;
/// ```
pub fn parse(source: &str, filename: &str) -> Result<Ast, ParseError> {
    let lexer = Lexer::new(source, filename);
    let mut parser = Parser::new(lexer);
    parser.parse()
}
```

---

## Testing Guidelines

### Test Coverage

- **Minimum coverage**: 80% for all code
- **Critical paths**: 100% coverage required
- Use `cargo tarpaulin` to measure coverage

### Test Types

#### Unit Tests

Located in `tests/unit/`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_tokenizes_integers() {
        let source = "42";
        let lexer = Lexer::new(source, "test.vela");
        let tokens: Vec<Token> = lexer.collect();
        
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Integer(42));
    }
}
```

#### Integration Tests

Located in `tests/integration/`:

```rust
#[test]
fn test_compile_and_run_hello_world() {
    let source = r#"
        fn main() {
            println("Hello, World!");
        }
    "#;
    
    let compiler = Compiler::new();
    let bytecode = compiler.compile(source, "test.vela").unwrap();
    
    let vm = VelaVM::new();
    let output = vm.run(bytecode).unwrap();
    
    assert_eq!(output, "Hello, World!\n");
}
```

#### End-to-End Tests

Located in `tests/e2e/`:

```bash
# Run end-to-end tests
cargo test --test e2e_cli
```

### Property-Based Testing

Use `proptest` for property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lexer_never_panics(s in ".*") {
        let lexer = Lexer::new(&s, "test.vela");
        let tokens: Vec<Token> = lexer.collect();
        // Should never panic
    }
}
```

### Benchmarks

Use `criterion` for benchmarking:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_lexer(c: &mut Criterion) {
    let source = include_str!("../fixtures/large_file.vela");
    
    c.bench_function("lexer::tokenize", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(source), "bench.vela");
            let tokens: Vec<Token> = lexer.collect();
            black_box(tokens);
        });
    });
}

criterion_group!(benches, benchmark_lexer);
criterion_main!(benches);
```

---

## Documentation

### Code Documentation

- Use `///` for public items
- Use `//!` for module-level documentation
- Include examples in doc comments
- Document panics, errors, and safety invariants

### Architecture Decision Records (ADRs)

For significant architectural decisions, create an ADR in `docs/architecture/`:

```markdown
# ADR-XXX: [Title]

## Status
✅ Accepted

## Date
YYYY-MM-DD

## Context
[Problem description]

## Decision
[Solution chosen]

## Consequences
### Positives
- [Benefit 1]

### Negatives
- [Trade-off 1]

## Alternatives Considered
1. [Alternative 1] - Rejected because [reason]

## References
- Jira: VELA-XXX
- Related ADRs: ADR-YYY
```

---

## Pull Request Process

### Before Creating PR

1. ✅ All tests pass: `cargo test --workspace`
2. ✅ Code is formatted: `cargo fmt --all`
3. ✅ No linter warnings: `cargo clippy --all-targets --all-features`
4. ✅ Documentation is updated
5. ✅ CHANGELOG.md is updated (if applicable)
6. ✅ Branch is up to date with `main`

### PR Template

Your PR should include:

- **Description**: What does this PR do?
- **Motivation**: Why is this change needed?
- **Testing**: How was this tested?
- **Checklist**: All items checked
- **Screenshots**: For UI changes
- **Related Issues**: Refs: VELA-XXX

### Review Process

1. **Automated checks**: CI/CD must pass
2. **Code review**: At least 1 approval required
3. **Maintainer review**: For significant changes
4. **Merge**: Squash and merge to `main`

### Merge Criteria

- ✅ CI/CD pipeline passes
- ✅ Code review approved
- ✅ No merge conflicts
- ✅ Documentation updated
- ✅ Tests added/updated

---

## Issue Guidelines

### Bug Reports

Use the Bug Report template:

- **Description**: What happened?
- **Expected behavior**: What should happen?
- **Steps to reproduce**: How to reproduce the bug?
- **Environment**: OS, Rust version, Vela version
- **Logs**: Error messages or stack traces

### Feature Requests

Use the Feature Request template:

- **Description**: What feature do you want?
- **Use case**: Why is this feature needed?
- **Proposal**: How should it work?
- **Alternatives**: Any alternatives considered?

---

## Community

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, ideas
- **Discord**: Real-time chat (coming soon)
- **Jira**: Project management ([velalang.atlassian.net](https://velalang.atlassian.net))

### Recognition

Contributors are recognized in:
- CHANGELOG.md
- Release notes
- GitHub Contributors page

---

## Questions?

If you have questions about contributing:

- **Email**: hello@velalang.org
- **GitHub Discussions**: [github.com/velalang/vela/discussions](https://github.com/velalang/vela/discussions)
- **Jira**: [velalang.atlassian.net](https://velalang.atlassian.net)

---

**Thank you for contributing to Vela! 🚀**

---

*Last updated: 2025-11-30*
