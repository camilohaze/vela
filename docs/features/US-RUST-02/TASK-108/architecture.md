# Vela Compiler Architecture Guide

## 🏗️ Overview

El compiler de Vela está diseñado como un pipeline modular de análisis y transformación, siguiendo principios de diseño funcional y separación de responsabilidades. Esta arquitectura permite extensibilidad, testabilidad y mantenibilidad.

## 🏛️ Architectural Principles

### 1. **Pipeline Design**
El compiler sigue un patrón de pipeline lineal donde cada etapa transforma la salida de la anterior:

```
Source Code → Lexer → Parser → Semantic Analyzer → Code Generator → Bytecode
```

### 2. **Separation of Concerns**
Cada módulo tiene una responsabilidad única y bien definida:
- **Lexer**: Análisis léxico
- **Parser**: Análisis sintáctico
- **Semantic**: Análisis semántico
- **Codegen**: Generación de código

### 3. **Error Propagation**
Sistema unificado de errores que se propaga a través del pipeline con información contextual.

### 4. **Immutability**
Las estructuras de datos son inmutables donde es posible, siguiendo principios funcionales.

## 📦 Module Architecture

### Core Modules

```
vela-compiler/
├── src/
│   ├── lib.rs           # Punto de entrada principal
│   ├── lexer.rs         # Análisis léxico
│   ├── parser.rs        # Análisis sintáctico
│   ├── semantic.rs      # Análisis semántico
│   ├── codegen.rs       # Generación de código
│   ├── error.rs         # Sistema de errores
│   └── config.rs        # Configuración
├── Cargo.toml
└── tests/
    └── integration.rs   # Tests de integración
```

### Dependencies

```toml
[dependencies]
vela-ast = { path = "../ast" }          # Definiciones AST
vela-vm = { path = "../vm" }            # VM y bytecode
anyhow = "1.0"                          # Error handling
thiserror = "1.0"                       # Error definitions
serde = { version = "1.0", features = ["derive"] }  # Serialization
tracing = "0.1"                         # Logging
```

## 🔄 Pipeline Flow

### 1. Lexical Analysis (Lexer)

**Input:** `String` (código fuente)
**Output:** `Vec<Token>`
**Responsibility:** Tokenizar el código fuente en unidades léxicas

```rust
pub struct Lexer {
    source: String,
    source_path: PathBuf,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            let token = self.scan_token()?;
            tokens.push(token);
        }
        tokens.push(Token::new(TokenKind::EOF, self.current_range()));
        Ok(tokens)
    }
}
```

**Key Components:**
- **State Machine**: Maneja diferentes estados léxicos (normal, string, comment)
- **Error Recovery**: Continúa tokenizando después de errores
- **Source Location Tracking**: Mantiene posición precisa para errores

### 2. Syntax Analysis (Parser)

**Input:** `Vec<Token>`
**Output:** `Program` (AST)
**Responsibility:** Parsear tokens en estructura sintáctica

```rust
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            let declaration = self.parse_declaration()?;
            declarations.push(declaration);
        }

        Ok(Program::new(self.tokens[0].range.clone(), declarations))
    }
}
```

**Key Components:**
- **Recursive Descent**: Parser recursivo descendente con precedencia de operadores
- **Error Recovery**: Sincronización después de errores para continuar parsing
- **AST Construction**: Construye árbol sintáctico abstracto

### 3. Semantic Analysis

**Input:** `Program` (AST)
**Output:** `SemanticProgram` (AST anotado)
**Responsibility:** Validar semántica y resolver símbolos

```rust
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    scope_stack: Vec<Scope>,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn analyze(&mut self, program: &Program) -> Result<SemanticProgram, CompileError> {
        self.visit_program(program)?;
        if self.errors.is_empty() {
            Ok(SemanticProgram::from(program))
        } else {
            Err(CompileError::Semantic(self.errors.remove(0)))
        }
    }
}
```

**Key Components:**
- **Symbol Resolution**: Resuelve nombres a definiciones
- **Type Checking**: Verifica compatibilidad de tipos
- **Scope Management**: Maneja ámbitos léxicos
- **Error Collection**: Recolecta múltiples errores semánticos

### 4. Code Generation

**Input:** `SemanticProgram` (AST anotado)
**Output:** `Bytecode`
**Responsibility:** Generar bytecode ejecutable

```rust
pub struct CodeGenerator {
    bytecode: Bytecode,
    symbol_table: HashMap<String, usize>,
    functions: Vec<Function>,
}

impl CodeGenerator {
    pub fn generate_program(&mut self, program: &SemanticProgram) -> Result<Bytecode, CompileError> {
        for declaration in &program.declarations {
            self.generate_declaration(declaration)?;
        }
        Ok(self.bytecode.clone())
    }
}
```

**Key Components:**
- **Instruction Emission**: Emite instrucciones bytecode apropiadas
- **Symbol Table**: Mapea símbolos a índices de constantes
- **Control Flow**: Maneja saltos y bucles
- **Function Generation**: Crea definiciones de funciones

## 🔗 Integration Points

### With AST Module

```rust
// AST definitions shared between parser and semantic analyzer
pub mod ast {
    pub struct Program { /* ... */ }
    pub enum Declaration { /* ... */ }
    pub enum Expression { /* ... */ }
}
```

### With VM Module

```rust
// Bytecode format shared between codegen and VM
pub mod bytecode {
    pub enum Instruction { /* ... */ }
    pub struct Bytecode { /* ... */ }
}
```

## 🛡️ Error Handling Architecture

### Unified Error Types

```rust
pub enum CompileError {
    Lexer(LexerError),
    Parser(ParserError),
    Semantic(SemanticError),
    Codegen(CodegenError),
}

pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}
```

### Error Propagation

```rust
// Each pipeline stage returns Result<T, CompileError>
fn compile(source: &str) -> Result<Bytecode, CompileError> {
    let tokens = lexer.tokenize(source)?;
    let ast = parser.parse(tokens)?;
    let semantic_ast = semantic_analyzer.analyze(&ast)?;
    let bytecode = code_generator.generate(&semantic_ast)?;
    Ok(bytecode)
}
```

## ⚡ Performance Considerations

### Memory Management
- **AST Reuse**: El AST se reutiliza entre etapas cuando es posible
- **Streaming**: Procesamiento en streaming para archivos grandes
- **Lazy Evaluation**: Evaluación diferida de expresiones constantes

### Optimization Opportunities
- **Constant Folding**: Evaluación de expresiones constantes en compile-time
- **Dead Code Elimination**: Eliminación de código unreachable
- **Register Allocation**: Asignación óptima de registros en bytecode

## 🧪 Testing Architecture

### Unit Tests
Cada módulo tiene tests unitarios independientes:
- `lexer/tests.rs` - Tests del lexer
- `parser/tests.rs` - Tests del parser
- `semantic/tests.rs` - Tests del analizador semántico
- `codegen/tests.rs` - Tests del generador de código

### Integration Tests
Tests end-to-end que verifican el pipeline completo:
- `tests/integration.rs` - Tests de integración completa

### Fuzz Testing
Tests de fuzzing para entradas malformadas:
- `tests/fuzz_lexer.rs` - Fuzzing del lexer
- `tests/fuzz_parser.rs` - Fuzzing del parser

## 🔧 Configuration System

```rust
pub struct Config {
    pub optimization_level: OptimizationLevel,
    pub target_platform: TargetPlatform,
    pub debug_info: bool,
    pub warnings_as_errors: bool,
    pub error_format: ErrorFormat,
}

pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

pub enum TargetPlatform {
    Native,
    WebAssembly,
    CrossPlatform,
}
```

## 🚀 Extensibility Points

### Adding New Language Features

1. **Lexer**: Agregar nuevos tokens en `TokenKind`
2. **Parser**: Extender gramática en métodos de parsing
3. **Semantic**: Agregar reglas de validación
4. **Codegen**: Implementar emisión de bytecode

### Backend Targets

El diseño modular permite múltiples backends:
- **Bytecode VM** (actual)
- **Native Code** (futuro)
- **WebAssembly** (futuro)
- **LLVM IR** (futuro)

## 📊 Metrics & Monitoring

### Compilation Metrics
- **Token Count**: Número de tokens procesados
- **AST Node Count**: Tamaño del árbol sintáctico
- **Bytecode Size**: Tamaño del bytecode generado
- **Compilation Time**: Tiempo total de compilación

### Error Metrics
- **Error Rate**: Errores por línea de código
- **Error Types**: Distribución de tipos de error
- **Recovery Success**: Tasa de recuperación de errores

## 🔗 Related Documentation

- [API Reference](api-reference.md) - Referencia completa de APIs
- [User Guide](user-guide.md) - Guía para usuarios
- [Developer Guide](developer-guide.md) - Guía para desarrolladores
- [Troubleshooting](troubleshooting.md) - Solución de problemas

---

*Documentación generada automáticamente. Última actualización: 2025-12-03*