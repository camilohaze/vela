# Vela Compiler API Reference

## 📚 Overview

Esta documentación describe la API pública del compiler de Vela en Rust. El compiler está organizado en varios módulos que forman un pipeline de compilación completo.

## 🏗️ Architecture

```rust
Source Code → Lexer → Parser → Semantic Analyzer → Code Generator → Bytecode
```

## 📦 Modules

### `vela_compiler::lexer`

Módulo responsable del análisis léxico - convierte código fuente en tokens.

#### `pub struct Lexer`

Analizador léxico principal.

**Methods:**
```rust
pub fn new(source: String, source_path: PathBuf) -> Self
pub fn tokenize(&mut self) -> Result<Vec<Token>, CompileError>
```

**Example:**
```rust
use vela_compiler::lexer::Lexer;

let mut lexer = Lexer::new("let x = 42".to_string(), PathBuf::from("example.vela"));
let tokens = lexer.tokenize()?;
```

#### `pub enum TokenKind`

Tipos de tokens reconocidos por el lexer.

**Variants:**
- `Identifier(String)` - Identificadores (variables, funciones)
- `Number(f64)` - Números literales
- `String(String)` - Cadenas de texto
- `Bool(bool)` - Booleanos
- `Plus`, `Minus`, `Star`, `Slash` - Operadores aritméticos
- `Equal`, `NotEqual`, `Less`, `Greater` - Operadores de comparación
- `And`, `Or` - Operadores lógicos
- `LeftParen`, `RightParen` - Paréntesis
- `LeftBrace`, `RightBrace` - Llaves
- `Semicolon` - Punto y coma
- `Let`, `Fn`, `If`, `Else`, `Return` - Keywords
- `EOF` - Fin de archivo

### `vela_compiler::parser`

Módulo responsable del análisis sintáctico - convierte tokens en AST.

#### `pub struct Parser`

Analizador sintáctico principal.

**Methods:**
```rust
pub fn new(tokens: Vec<Token>) -> Self
pub fn parse(&mut self) -> Result<Program, CompileError>
```

**Example:**
```rust
use vela_compiler::{lexer::Lexer, parser::Parser};

let mut lexer = Lexer::new("fn main() { return 42; }".to_string(), PathBuf::new());
let tokens = lexer.tokenize()?;
let mut parser = Parser::new(tokens);
let ast = parser.parse()?;
```

#### AST Node Types

##### `pub struct Program`
Nodo raíz del AST.
```rust
pub struct Program {
    pub range: SourceRange,
    pub imports: Vec<ImportDeclaration>,
    pub declarations: Vec<Declaration>,
}
```

##### `pub enum Declaration`
Declaraciones de nivel superior.
```rust
pub enum Declaration {
    Function(FunctionDeclaration),
    Variable(VariableDeclaration),
    Import(ImportDeclaration),
}
```

##### `pub enum Statement`
Statements ejecutables.
```rust
pub enum Statement {
    Block(BlockStatement),
    Expression(ExpressionStatement),
    Variable(VariableDeclaration),
    Assignment(AssignmentStatement),
    Return(ReturnStatement),
    If(IfStatement),
}
```

##### `pub enum Expression`
Expresiones evaluables.
```rust
pub enum Expression {
    Literal(Literal),
    Identifier(Identifier),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Call(CallExpression),
    ArrayLiteral(ArrayLiteral),
    StructLiteral(StructLiteral),
}
```

### `vela_compiler::semantic`

Módulo responsable del análisis semántico - validación de tipos y resolución de símbolos.

#### `pub struct SemanticAnalyzer`

Analizador semántico principal.

**Methods:**
```rust
pub fn new() -> Self
pub fn analyze(&mut self, program: &Program) -> Result<SemanticProgram, CompileError>
```

**Example:**
```rust
use vela_compiler::{parser::Parser, semantic::SemanticAnalyzer};

let mut parser = Parser::new(tokens);
let ast = parser.parse()?;
let mut analyzer = SemanticAnalyzer::new();
let semantic_program = analyzer.analyze(&ast)?;
```

### `vela_compiler::codegen`

Módulo responsable de la generación de bytecode - convierte AST en bytecode ejecutable.

#### `pub struct CodeGenerator`

Generador de código principal.

**Methods:**
```rust
pub fn new() -> Self
pub fn generate_program(&mut self, program: &Program) -> Result<Bytecode, CompileError>
```

**Example:**
```rust
use vela_compiler::codegen::CodeGenerator;

let mut generator = CodeGenerator::new();
let bytecode = generator.generate_program(&ast)?;
```

### `vela_compiler::error`

Sistema de manejo de errores unificado.

#### `pub enum CompileError`

Tipos de errores de compilación.

**Variants:**
```rust
CompileError::Lexer(LexerError)
CompileError::Parser(ParserError)
CompileError::Semantic(SemanticError)
CompileError::Codegen(CodegenError)
```

#### Error Types

##### `LexerError`
```rust
pub struct LexerError {
    pub message: String,
    pub location: Option<SourceLocation>,
}
```

##### `ParserError`
```rust
pub struct ParserError {
    pub message: String,
    pub location: Option<SourceLocation>,
    pub expected: Vec<String>,
}
```

##### `SemanticError`
```rust
pub struct SemanticError {
    pub message: String,
    pub location: Option<SourceLocation>,
    pub error_type: SemanticErrorType,
}
```

##### `CodegenError`
```rust
pub struct CodegenError {
    pub message: String,
    pub location: Option<SourceLocation>,
}
```

### `vela_compiler::config`

Configuración del compiler.

#### `pub struct Config`

Configuración principal del compiler.

```rust
pub struct Config {
    pub optimization_level: OptimizationLevel,
    pub target_platform: TargetPlatform,
    pub debug_info: bool,
    pub warnings_as_errors: bool,
}
```

## 🔧 Usage Examples

### Compilación Básica

```rust
use vela_compiler::{Compiler, Config};
use std::path::Path;

// Crear configuración
let config = Config {
    optimization_level: OptimizationLevel::Release,
    target_platform: TargetPlatform::Native,
    debug_info: true,
    warnings_as_errors: false,
};

// Crear compiler
let mut compiler = Compiler::new(config);

// Compilar archivo
let source_path = Path::new("program.vela");
let bytecode = compiler.compile_file(source_path)?;

// Ejecutar bytecode
let result = compiler.execute_bytecode(&bytecode)?;
println!("Result: {:?}", result);
```

### Compilación desde String

```rust
use vela_compiler::Compiler;

// Código fuente
let source = r#"
fn factorial(n: Number) -> Number {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() {
    let result = factorial(5);
    return result;
}
"#;

// Compilar
let bytecode = compiler.compile_string(source)?;

// El bytecode puede ser serializado y guardado
let bytecode_bytes = bytecode.into_bytes();
std::fs::write("program.bytecode", &bytecode_bytes)?;
```

### Manejo de Errores

```rust
use vela_compiler::{Compiler, CompileError};

match compiler.compile_string(source) {
    Ok(bytecode) => {
        println!("Compilation successful!");
        // Ejecutar bytecode...
    }
    Err(CompileError::Lexer(error)) => {
        eprintln!("Lexer error: {} at {:?}", error.message, error.location);
    }
    Err(CompileError::Parser(error)) => {
        eprintln!("Parser error: {} at {:?}", error.message, error.location);
        eprintln!("Expected: {:?}", error.expected);
    }
    Err(CompileError::Semantic(error)) => {
        eprintln!("Semantic error: {} at {:?}", error.message, error.location);
    }
    Err(CompileError::Codegen(error)) => {
        eprintln!("Code generation error: {} at {:?}", error.message, error.location);
    }
}
```

## 📊 Performance Characteristics

- **Lexer**: O(n) donde n es la longitud del código fuente
- **Parser**: O(n) con recuperación de errores
- **Semantic Analysis**: O(n) con resolución de símbolos
- **Code Generation**: O(n) con optimizaciones locales
- **Memory Usage**: Proporcional al tamaño del AST

## 🔗 Related Documentation

- [Architecture Guide](architecture.md) - Diseño interno del compiler
- [User Guide](user-guide.md) - Cómo usar el compiler
- [Developer Guide](developer-guide.md) - Cómo extender el compiler
- [Troubleshooting](troubleshooting.md) - Problemas comunes

---

*Esta documentación se genera automáticamente del código fuente. Última actualización: 2025-12-03*