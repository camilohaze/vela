# Compiler Pipeline Diagram

```
Source Code (.vela)
        │
        ▼
    ┌─────────────┐
    │   LEXER     │  Tokenization
    │             │  "fn main() {" → [Fn, Identifier("main"), LeftParen, ...]
    └─────────────┘
        │
        ▼
    ┌─────────────┐
    │   PARSER    │  Syntax Analysis
    │             │  Tokens → AST (Abstract Syntax Tree)
    └─────────────┘
        │
        ▼
    ┌─────────────┐
    │ SEMANTIC    │  Semantic Analysis
    │ ANALYZER    │  Type checking, symbol resolution
    └─────────────┘
        │
        ▼
    ┌─────────────┐
    │ CODE        │  Code Generation
    │ GENERATOR   │  AST → Bytecode
    └─────────────┘
        │
        ▼
   Bytecode (.bytecode)
        │
        ▼
    ┌─────────────┐
    │  VELA VM    │  Execution
    │             │  Bytecode interpretation
    └─────────────┘
```

## Detailed Component View

```
┌─────────────────────────────────────────────────────────────┐
│                    VELA COMPILER                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │    LEXER    │ │   PARSER    │ │   SEMANTIC  │           │
│  │             │ │             │ │  ANALYZER   │           │
│  │ • Scanner   │ │ • Recursive │ │ • Type      │           │
│  │ • Tokenizer │ │   Descent   │ │   Checker   │           │
│  │ • Error     │ │ • AST       │ │ • Symbol    │           │
│  │   Recovery  │ │   Builder   │ │   Resolver  │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
│           │               │               │                 │
├───────────┼───────────────┼───────────────┼─────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │ ERROR       │ │ SYMBOL      │ │ CONFIG      │           │
│  │ HANDLING    │ │ TABLE       │ │             │           │
│  │ • Compile   │ │ • Scopes    │ │ • Options   │           │
│  │   Errors    │ │ • Types     │ │ • Target    │           │
│  │ • Source    │ │ • Functions │ │   Platform  │           │
│  │   Location  │ │ • Variables │ │ • Debug     │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │                 CODE GENERATOR                      │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │ • Instruction Emission • Register Allocation       │   │
│  │ • Control Flow • Function Calls • Constants        │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

```
Input: String (source code)
       ↓
Lexer: String → Vec<Token>
       ↓
Parser: Vec<Token> → Program (AST)
       ↓
Semantic: Program → SemanticProgram (annotated AST)
       ↓
CodeGen: SemanticProgram → Bytecode
       ↓
Output: Bytecode (executable)
```

## Error Propagation

```
Any Stage → CompileError
    │
    ├── LexerError (tokenization failed)
    ├── ParserError (syntax error)
    ├── SemanticError (type error, undefined symbol)
    └── CodegenError (code generation failed)
```

## Module Dependencies

```
vela-compiler
├── vela-ast      # AST definitions
├── vela-vm       # Bytecode format, VM interface
└── common        # Shared utilities, error types
```

---

*Diagram generated automatically. Last updated: 2025-12-03*