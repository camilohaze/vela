# Vela Compiler Developer Guide

## 🛠️ Contributing to the Compiler

Esta guía está destinada a desarrolladores que quieren contribuir al compiler de Vela, ya sea extendiendo funcionalidades, corrigiendo bugs, o mejorando el rendimiento.

## 🏗️ Development Setup

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install stable
rustup component add rustfmt clippy

# Development tools
cargo install cargo-watch
cargo install cargo-tarpaulin  # Coverage
cargo install cargo-audit       # Security audit
```

### Clone and Setup

```bash
git clone https://github.com/velalang/vela.git
cd vela

# Setup pre-commit hooks
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace
```

### Development Workflow

```bash
# Development loop
cargo watch -x 'test --workspace'

# Format code
cargo fmt --all

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
cargo audit

# Generate documentation
cargo doc --workspace --open
```

## 📁 Project Structure

```
crates/
├── vela-compiler/          # Main compiler crate
│   ├── src/
│   │   ├── lib.rs         # Public API
│   │   ├── lexer.rs       # Lexical analysis
│   │   ├── parser.rs      # Syntax analysis
│   │   ├── semantic.rs    # Semantic analysis
│   │   ├── codegen.rs     # Code generation
│   │   ├── error.rs       # Error types
│   │   └── config.rs      # Configuration
│   ├── tests/
│   │   ├── lexer.rs       # Lexer tests
│   │   ├── parser.rs      # Parser tests
│   │   ├── semantic.rs    # Semantic tests
│   │   └── codegen.rs     # Codegen tests
│   └── Cargo.toml
│
├── vela-ast/              # AST definitions
├── vela-vm/               # Virtual machine
└── vela-cli/              # Command line interface
```

## 🔧 Adding New Language Features

### 1. Define the Feature

Antes de implementar, documenta la feature:

```markdown
# Feature: Optional Chaining

## Syntax
```vela
let value = obj?.property?.method()
```

## Semantics
- Returns `None` if any part of the chain is `None`
- Short-circuits evaluation
- Type: `Option<T>` where T is the final property type

## Implementation Plan
1. Add `?.` token to lexer
2. Extend parser for optional chaining expressions
3. Update semantic analyzer for type checking
4. Generate appropriate bytecode
```

### 2. Update the Lexer

```rust
// In lexer.rs
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens
    QuestionDot,  // ?
}

// In the scan_token method
match self.peek() {
    Some('?') => {
        self.advance();
        if self.peek() == Some('.') {
            self.advance();
            self.add_token(TokenKind::QuestionDot);
        } else {
            self.add_token(TokenKind::Question);
        }
    }
    // ... other cases
}
```

### 3. Extend the AST

```rust
// In vela-ast/src/lib.rs
#[derive(Debug, Clone)]
pub enum Expression {
    // ... existing expressions
    OptionalChain {
        object: Box<Expression>,
        property: String,
    },
}
```

### 4. Update the Parser

```rust
// In parser.rs
impl Parser {
    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let expr = self.parse_primary()?;

        if self.match_token(TokenKind::QuestionDot) {
            let property = self.parse_identifier()?;
            return Ok(Expression::OptionalChain {
                object: Box::new(expr),
                property,
            });
        }

        Ok(expr)
    }
}
```

### 5. Update Semantic Analysis

```rust
// In semantic.rs
impl SemanticAnalyzer {
    fn visit_optional_chain(&mut self, expr: &OptionalChain) -> Result<Type, SemanticError> {
        let object_type = self.visit_expression(&expr.object)?;

        match object_type {
            Type::Option(inner_type) => {
                // Check if property exists on inner_type
                if self.has_property(&inner_type, &expr.property) {
                    let property_type = self.get_property_type(&inner_type, &expr.property)?;
                    Ok(Type::Option(Box::new(property_type)))
                } else {
                    Err(SemanticError::PropertyNotFound {
                        property: expr.property.clone(),
                        on_type: inner_type,
                    })
                }
            }
            _ => Err(SemanticError::InvalidOptionalChain {
                on_type: object_type,
            })
        }
    }
}
```

### 6. Update Code Generation

```rust
// In codegen.rs
impl CodeGenerator {
    fn generate_optional_chain(&mut self, expr: &OptionalChain) -> Result<(), CodegenError> {
        // Generate code for object
        self.generate_expression(&expr.object)?;

        // Emit optional chain instruction
        self.emit(Instruction::OptionalChain {
            property: self.add_constant(expr.property.clone()),
        });

        Ok(())
    }
}
```

### 7. Add Tests

```rust
// In tests/codegen.rs
#[test]
fn test_optional_chaining() {
    let source = r#"
        let obj: Option<{value: Number}> = Some({value: 42})
        let result = obj?.value
    "#;

    let bytecode = compile(source).unwrap();

    // Verify bytecode contains optional chain instruction
    assert!(bytecode.instructions.contains(&Instruction::OptionalChain { property: 0 }));
}

// In tests/semantic.rs
#[test]
fn test_optional_chain_type_checking() {
    let source = r#"
        let obj: {value: Number} = {value: 42}
        let result = obj?.value  // Should fail: obj is not Option
    "#;

    let result = compile(source);
    assert!(matches!(result, Err(CompileError::Semantic(_))));
}
```

## 🐛 Debugging the Compiler

### Adding Debug Logging

```rust
use tracing::{info, debug, error};

pub fn compile(source: &str) -> Result<Bytecode, CompileError> {
    info!("Starting compilation");

    debug!("Tokenizing source");
    let tokens = lexer.tokenize(source)?;
    debug!("Found {} tokens", tokens.len());

    debug!("Parsing tokens");
    let ast = parser.parse(tokens)?;
    debug!("Parsed AST with {} declarations", ast.declarations.len());

    // ... continue with other phases

    info!("Compilation successful");
    Ok(bytecode)
}
```

### Using the Debug REPL

```bash
# Start compiler in debug mode
cargo run --bin vela-compiler -- --debug repl

# In the REPL, you can inspect internal state
> :tokens let x = 42
> :ast let x = 42
> :bytecode let x = 42
> :symbols let x = 42
```

### Debugging Parser Issues

```rust
// Add debug output to parser
impl Parser {
    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        debug!("Parsing expression at token: {:?}", self.peek());

        let expr = self.parse_primary()?;
        debug!("Parsed primary expression: {:?}", expr);

        // ... continue parsing
    }
}
```

## 🚀 Performance Optimization

### Profiling Compilation

```bash
# Profile with cargo flamegraph
cargo install flamegraph
cargo flamegraph --bin vela-compiler -- compile large_file.vela

# Profile memory usage
cargo build --release
valgrind --tool=massif target/release/vela-compiler compile large_file.vela
```

### Common Bottlenecks

#### 1. String Interning

```rust
// Use a string interner for identifiers
use lasso::ThreadedRodeo;

pub struct Compiler {
    strings: ThreadedRodeo,
}

impl Compiler {
    pub fn intern(&mut self, string: &str) -> StringId {
        self.strings.get_or_intern(string)
    }
}
```

#### 2. AST Reuse

```rust
// Reuse AST nodes when possible
pub struct AstNode<T> {
    data: T,
    span: SourceSpan,
    // Add cached analysis results
    cached_type: OnceCell<Type>,
}
```

#### 3. Incremental Compilation

```rust
// Track file dependencies
pub struct DependencyGraph {
    files: HashMap<PathBuf, FileInfo>,
    dependencies: HashMap<FileId, Vec<FileId>>,
}

impl DependencyGraph {
    pub fn needs_recompile(&self, file: &PathBuf) -> bool {
        // Check if file or dependencies changed
        // ...
    }
}
```

## 🧪 Testing Strategies

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = Lexer::new("fn if else");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::If);
        assert_eq!(tokens[2].kind, TokenKind::Else);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_compilation() {
    let source = r#"
        fn main() {
            print("Hello, World!")
        }
    "#;

    let result = compile(source);
    assert!(result.is_ok());

    let bytecode = result.unwrap();
    assert!(!bytecode.instructions.is_empty());
}
```

### Fuzz Testing

```rust
// Use cargo-fuzz for fuzzing
use cargo_fuzz::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        // Try to compile - we don't care if it fails,
        // we just want to make sure it doesn't crash
        let _ = compile(source);
    }
});
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_arithmetic_commutativity(a in 0..100, b in 0..100) {
        let source = format!("let result = {} + {}", a, b);
        let bytecode = compile(&source).unwrap();

        // Execute bytecode and verify result == a + b
        let result = execute_bytecode(&bytecode);
        prop_assert_eq!(result, a + b);
    }
}
```

## 🔒 Security Considerations

### Input Validation

```rust
// Always validate input sizes
pub fn compile(source: &str) -> Result<Bytecode, CompileError> {
    if source.len() > MAX_SOURCE_SIZE {
        return Err(CompileError::SourceTooLarge);
    }

    // Validate UTF-8
    if !source.is_char_boundary(source.len()) {
        return Err(CompileError::InvalidUtf8);
    }

    // Continue with compilation...
}
```

### Sandboxing

```rust
// Limit compiler resource usage
pub struct CompilerLimits {
    pub max_tokens: usize,
    pub max_ast_nodes: usize,
    pub max_bytecode_size: usize,
    pub timeout: Duration,
}

impl Compiler {
    pub fn compile_with_limits(
        &self,
        source: &str,
        limits: &CompilerLimits
    ) -> Result<Bytecode, CompileError> {
        // Implement resource limits...
    }
}
```

## 📊 Metrics and Monitoring

### Compilation Metrics

```rust
pub struct CompilationMetrics {
    pub source_size: usize,
    pub token_count: usize,
    pub ast_nodes: usize,
    pub bytecode_size: usize,
    pub compilation_time: Duration,
    pub error_count: usize,
}

impl Compiler {
    pub fn compile_with_metrics(&self, source: &str) -> (Result<Bytecode, CompileError>, CompilationMetrics) {
        let start_time = Instant::now();

        // ... compilation logic ...

        let metrics = CompilationMetrics {
            source_size: source.len(),
            // ... other metrics
            compilation_time: start_time.elapsed(),
        };

        (result, metrics)
    }
}
```

### Error Reporting

```rust
// Structured error reporting
#[derive(Debug)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: SourceLocation,
    pub suggestions: Vec<String>,
}

impl CompilerError {
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }
}
```

## 🔧 Tooling

### Custom Lints

```rust
// Add custom clippy lints
#[warn(clippy::pedantic)]
#[warn(clippy::nursery)]
#[warn(clippy::cargo)]

// Custom lint for compiler-specific patterns
declare_lint! {
    pub COMPILER_STYLE,
    Warn,
    "compiler style guide violation"
}
```

### Build Scripts

```rust
// build.rs for code generation
use std::process::Command;

fn main() {
    // Generate AST from grammar file
    Command::new("cargo")
        .args(&["run", "--bin", "ast_generator"])
        .status()
        .expect("Failed to generate AST");

    // Regenerate if grammar changes
    println!("cargo:rerun-if-changed=grammar.txt");
}
```

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - uses: actions-rs/cargo@v1
        with:
          command: test
          args: --workspace --all-features

  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run lexer -- -max_total_time=300
```

## 📚 Advanced Topics

### Implementing a New Backend

```rust
pub trait Backend {
    type Output;

    fn generate(&self, program: &SemanticProgram) -> Result<Self::Output, CodegenError>;
}

// LLVM Backend
pub struct LLVMBackend {
    context: LLVMContext,
}

impl Backend for LLVMBackend {
    type Output = LLVMModule;

    fn generate(&self, program: &SemanticProgram) -> Result<LLVMModule, CodegenError> {
        // Generate LLVM IR...
    }
}
```

### Parser Combinators

```rust
// Alternative parser implementation using nom
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::map_res,
    sequence::tuple,
};

fn parse_number(input: &str) -> IResult<&str, i64> {
    map_res(digit1, |s: &str| s.parse::<i64>())(input)
}

fn parse_addition(input: &str) -> IResult<&str, Expression> {
    map_res(
        tuple((parse_number, tag(" + "), parse_number)),
        |(a, _, b)| Ok(Expression::Add(a, b))
    )(input)
}
```

### JIT Compilation

```rust
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};

pub struct JITCompiler {
    module: JITModule,
    functions: HashMap<String, *const u8>,
}

impl JITCompiler {
    pub fn compile_function(&mut self, name: &str, source: &str) -> Result<*const u8, JITError> {
        // Parse and analyze function
        // Generate Cranelift IR
        // JIT compile to native code
        // Return function pointer
    }
}
```

## 🤝 Contributing Guidelines

### Code Style

```rust
// Use rustfmt
cargo fmt

// Follow clippy suggestions
cargo clippy

// Naming conventions
struct CompilerState    // PascalCase for types
fn compile_source()     // snake_case for functions
let max_tokens = 1000;  // snake_case for variables
const MAX_SIZE: usize = 1024;  // SCREAMING_SNAKE_CASE for constants
```

### Commit Messages

```bash
# Format: type(scope): description
feat(lexer): add support for raw strings
fix(parser): handle empty function bodies
docs(api): update error handling examples
refactor(codegen): simplify instruction emission
test(semantic): add type checking tests
```

### Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Commit** changes: `git commit -m 'feat: add amazing feature'`
4. **Push** to branch: `git push origin feature/amazing-feature`
5. **Open** a Pull Request
6. **Wait** for review and CI checks
7. **Merge** after approval

### Issue Tracking

- **Bug reports**: Use the bug report template
- **Feature requests**: Use the feature request template
- **Questions**: Check existing issues first, then ask in discussions

---

*Documentación generada automáticamente. Última actualización: 2025-12-03*