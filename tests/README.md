# Test Organization Guide

This directory contains all test suites for the Vela project.

## Directory Structure

```
tests/
├── unit/               # Unit tests (isolated component testing)
│   ├── lexer/          # Lexer tests
│   ├── parser/         # Parser tests
│   ├── semantic/       # Semantic analyzer tests
│   ├── codegen/        # Code generation tests
│   ├── vm/             # VM tests
│   └── stdlib/         # Standard library tests
├── integration/        # Integration tests (multi-component)
│   ├── compiler-e2e/   # Compiler end-to-end tests
│   ├── lsp-e2e/        # LSP end-to-end tests
│   └── devtools-e2e/   # DevTools end-to-end tests
└── e2e/                # End-to-end tests (full application)
    ├── fixtures/       # Test fixtures (sample Vela projects)
    │   ├── hello-world/
    │   ├── todo-app/
    │   └── benchmarks/
    └── snapshots/      # Snapshot test outputs
```

## Test Types

### Unit Tests (`tests/unit/`)

**Purpose:** Test individual components in isolation.

**Location:** Each test file should match the source file it tests.

**Example:**
```rust
// tests/unit/lexer/test_tokenizer.rs
use vela_lexer::Tokenizer;

#[test]
fn test_tokenize_integer() {
    let source = "42";
    let mut tokenizer = Tokenizer::new(source, "test.vela");
    let tokens: Vec<_> = tokenizer.collect();
    
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
}
```

**Run:**
```bash
cargo test --lib
cargo test --package vela-lexer
```

### Integration Tests (`tests/integration/`)

**Purpose:** Test interactions between multiple components.

**Example:**
```rust
// tests/integration/compiler-e2e/test_compile_and_run.rs
use vela_compiler::Compiler;
use vela_vm::VelaVM;

#[test]
fn test_hello_world_compile_and_run() {
    let source = r#"
        fn main() {
            println("Hello, World!");
        }
    "#;
    
    let compiler = Compiler::new();
    let bytecode = compiler.compile(source, "test.vela").unwrap();
    
    let mut vm = VelaVM::new();
    let output = vm.run(bytecode).unwrap();
    
    assert_eq!(output.stdout, "Hello, World!\n");
}
```

**Run:**
```bash
cargo test --test '*'
cargo test --test compiler-e2e
```

### End-to-End Tests (`tests/e2e/`)

**Purpose:** Test complete workflows from CLI to output.

**Example:**
```rust
// tests/e2e/test_cli.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_vela_run_hello_world() {
    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("run")
       .arg("tests/e2e/fixtures/hello-world/main.vela");
    
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("Hello, World!"));
}
```

**Run:**
```bash
cargo test --test e2e_cli
```

## Test Frameworks

### Core Testing: `cargo test`

Standard Rust test framework.

### Snapshot Testing: `insta`

For testing outputs that are too large to manually verify.

```rust
use insta::assert_snapshot;

#[test]
fn test_parser_output() {
    let source = "fn main() { println(\"test\"); }";
    let ast = parse(source).unwrap();
    assert_snapshot!(format!("{:#?}", ast));
}
```

**Update snapshots:**
```bash
cargo insta test
cargo insta review
```

### Property-Based Testing: `proptest`

For testing invariants with random inputs.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lexer_never_panics(s in ".*") {
        let lexer = Lexer::new(&s, "test.vela");
        let _tokens: Vec<Token> = lexer.collect();
        // Should never panic
    }
}
```

### Benchmarking: `criterion`

For performance testing.

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parser(c: &mut Criterion) {
    let source = include_str!("../fixtures/large_file.vela");
    
    c.bench_function("parser::parse", |b| {
        b.iter(|| {
            let ast = parse(black_box(source));
            black_box(ast);
        });
    });
}

criterion_group!(benches, benchmark_parser);
criterion_main!(benches);
```

**Run:**
```bash
cargo bench
```

## Code Coverage

### Using `cargo-tarpaulin`

```bash
# Install
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --all-features --timeout 300 --out Html

# Output: target/tarpaulin/tarpaulin-report.html
```

### Using `cargo-llvm-cov`

```bash
# Install
cargo install cargo-llvm-cov

# Run coverage
cargo llvm-cov --html

# Output: target/llvm-cov/html/index.html
```

### Coverage Goals

- **Overall:** >= 80%
- **Critical paths:** >= 95%
- **New code:** >= 90%

## Test Fixtures

### Location: `tests/e2e/fixtures/`

Sample Vela projects for end-to-end testing.

**Structure:**
```
tests/e2e/fixtures/
├── hello-world/
│   ├── main.vela
│   └── vela.yaml
├── todo-app/
│   ├── main.vela
│   ├── models.vela
│   ├── ui.vela
│   └── vela.yaml
└── benchmarks/
    ├── fibonacci.vela
    ├── quicksort.vela
    └── binary-tree.vela
```

## Running Tests

### All Tests
```bash
cargo test --workspace
```

### Specific Package
```bash
cargo test --package vela-lexer
```

### Specific Test
```bash
cargo test test_tokenize_integer
```

### With Output
```bash
cargo test -- --nocapture
```

### Ignored Tests
```bash
cargo test -- --ignored
```

### Single-threaded
```bash
cargo test -- --test-threads=1
```

### Watch Mode
```bash
cargo install cargo-watch
cargo watch -x test
```

## CI/CD Integration

Tests run automatically on GitHub Actions:

- **Check job:** Formatting and linting
- **Test job:** All tests on 3 OS × 2 Rust versions
- **Coverage job:** Code coverage with tarpaulin
- **Benchmark job:** Performance benchmarks

See `.github/workflows/ci.yml` for configuration.

## Writing Good Tests

### Test Naming

```rust
#[test]
fn test_<component>_<scenario>_<expected_outcome>() {
    // ...
}
```

**Example:**
```rust
#[test]
fn test_lexer_tokenizes_integers_correctly() {
    // ...
}
```

### Test Structure (AAA Pattern)

```rust
#[test]
fn test_example() {
    // Arrange
    let input = "test input";
    let expected = "expected output";
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

### Assertions

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, not_expected);

// Boolean
assert!(condition);
assert!(condition, "Custom error message");

// Pretty assertions (use pretty_assertions crate)
use pretty_assertions::assert_eq;

// Result assertions
assert!(result.is_ok());
assert!(result.is_err());
assert_eq!(result.unwrap(), expected);
```

### Test Data

```rust
// Use constants for reusable test data
const SIMPLE_PROGRAM: &str = r#"
    fn main() {
        println("Hello");
    }
"#;

#[test]
fn test_with_constant() {
    let ast = parse(SIMPLE_PROGRAM).unwrap();
    assert!(ast.is_valid());
}
```

## Debugging Tests

### Print Debugging
```bash
cargo test test_name -- --nocapture
```

### Debugger (VS Code)
```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug test",
    "cargo": {
        "args": ["test", "--no-run", "--lib"],
        "filter": {
            "name": "test_name",
            "kind": "test"
        }
    }
}
```

## Troubleshooting

### Flaky Tests
- Use `#[ignore]` to skip temporarily
- Investigate timing issues
- Check for shared state

### Slow Tests
- Use `#[ignore]` for expensive tests
- Run only in CI with `--ignored`
- Consider mocking expensive operations

### Test Isolation
- Each test should be independent
- Use `setup()` and `teardown()` for common logic
- Avoid global state

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [The Little Book of Rust Macros](https://danielkeep.github.io/tlborm/book/)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Insta Documentation](https://insta.rs/)
- [Proptest Documentation](https://proptest-rs.github.io/proptest/)

---

*Last updated: 2025-11-30*
