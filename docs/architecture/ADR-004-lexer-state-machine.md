# ADR-004: Diseño del Lexer con State Machine

## Estado
✅ Aceptado

## Fecha
2025-11-30

## Contexto

Necesitamos implementar un lexer de producción para tokenizar código fuente Vela. El lexer debe:
- Reconocer ~100 keywords (funcional puro + domain-specific + reactive)
- Tokenizar 40+ operadores con distintos niveles
- Manejar string interpolation `${}`
- Tracking preciso de posiciones (line, column, offset)
- Performance óptima (O(n) en caracteres)
- Error recovery para seguir tokenizando después de errores

### Opciones Evaluadas

**Opción 1: Hand-written State Machine**
- Control total sobre comportamiento
- Performance óptima
- Debugging directo
- Mantenimiento manual

**Opción 2: Lexer Generator (Lex, Flex, LALRPOP)**
- Generación automática desde reglas
- Menos código manual
- Menos flexible
- Debugging indirecto

**Opción 3: Parser Combinator (nom, chumsky)**
- Composición elegante
- Funcional
- Performance variable
- Stack usage alto

## Decisión

**Elegimos Opción 1: Hand-written State Machine** implementado en Rust.

### Arquitectura del Lexer

```rust
pub struct Lexer {
    source: Vec<char>,      // Código fuente como chars
    position: Position,      // Posición actual (line, column, offset)
    current: usize,          // Índice en source
    start: usize,            // Inicio del token actual
    keywords: HashMap<String, TokenKind>, // Tabla de keywords
}

pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub position: Position,
}

pub enum TokenKind {
    // Keywords funcionales
    If, Else, Match, Return, Yield,
    
    // Declarations
    State, Fn, Struct, Enum, Trait, Impl, Type, Interface,
    
    // Domain-specific
    Widget, Component, Service, Repository, Controller,
    Dto, Entity, ValueObject, Model,
    Factory, Builder, Strategy, Observer, Singleton,
    
    // Reactive
    Signal, Computed, Effect, Watch, Store,
    
    // Types
    Number, Float, String, Bool, Option, Result,
    
    // Operators
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, BangEqual,
    Less, LessEqual, Greater, GreaterEqual,
    AmpersandAmpersand, PipePipe,
    QuestionQuestion,  // Option<T> coalescing
    QuestionDot,       // Optional chaining
    
    // Literals
    NumberLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    None,
    
    // Delimiters
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftBracket, RightBracket,
    Comma, Semicolon, Colon, Dot,
    Arrow, FatArrow,
    
    // Special
    Eof,
    Error(String),
}
```

### State Machine Principal

```rust
impl Lexer {
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        
        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }
        
        let c = self.advance();
        
        match c {
            // Identifiers y Keywords
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            
            // Numbers
            '0'..='9' => self.number(),
            
            // Strings
            '"' => self.string(),
            
            // Operators (dos caracteres)
            '=' => {
                if self.matches('=') {
                    self.make_token(TokenKind::EqualEqual)
                } else {
                    self.make_token(TokenKind::Equal)
                }
            }
            '!' => {
                if self.matches('=') {
                    self.make_token(TokenKind::BangEqual)
                } else {
                    self.make_token(TokenKind::Bang)
                }
            }
            '<' => {
                if self.matches('=') {
                    self.make_token(TokenKind::LessEqual)
                } else {
                    self.make_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.matches('=') {
                    self.make_token(TokenKind::GreaterEqual)
                } else {
                    self.make_token(TokenKind::Greater)
                }
            }
            '&' => {
                if self.matches('&') {
                    self.make_token(TokenKind::AmpersandAmpersand)
                } else {
                    self.make_token(TokenKind::Ampersand)
                }
            }
            '|' => {
                if self.matches('|') {
                    self.make_token(TokenKind::PipePipe)
                } else {
                    self.make_token(TokenKind::Pipe)
                }
            }
            '?' => {
                if self.matches('?') {
                    self.make_token(TokenKind::QuestionQuestion)
                } else if self.matches('.') {
                    self.make_token(TokenKind::QuestionDot)
                } else {
                    self.make_token(TokenKind::Question)
                }
            }
            
            // Single-character tokens
            '+' => self.make_token(TokenKind::Plus),
            '-' => {
                if self.matches('>') {
                    self.make_token(TokenKind::Arrow)
                } else {
                    self.make_token(TokenKind::Minus)
                }
            }
            '*' => {
                if self.matches('*') {
                    self.make_token(TokenKind::StarStar)  // Exponenciación
                } else {
                    self.make_token(TokenKind::Star)
                }
            }
            '/' => {
                if self.matches('/') {
                    self.comment_line()
                } else if self.matches('*') {
                    self.comment_block()
                } else {
                    self.make_token(TokenKind::Slash)
                }
            }
            '%' => self.make_token(TokenKind::Percent),
            
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '{' => self.make_token(TokenKind::LeftBrace),
            '}' => self.make_token(TokenKind::RightBrace),
            '[' => self.make_token(TokenKind::LeftBracket),
            ']' => self.make_token(TokenKind::RightBracket),
            
            ',' => self.make_token(TokenKind::Comma),
            ';' => self.make_token(TokenKind::Semicolon),
            ':' => self.make_token(TokenKind::Colon),
            '.' => self.make_token(TokenKind::Dot),
            
            _ => self.error_token(&format!("Unexpected character: '{}'", c)),
        }
    }
    
    fn identifier(&mut self) -> Token {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        
        let text = self.current_lexeme();
        let kind = self.keyword_or_identifier(&text);
        self.make_token(kind)
    }
    
    fn keyword_or_identifier(&self, text: &str) -> TokenKind {
        self.keywords.get(text).cloned().unwrap_or(TokenKind::Identifier)
    }
    
    fn number(&mut self) -> Token {
        while self.peek().is_numeric() {
            self.advance();
        }
        
        // Check for float
        if self.peek() == '.' && self.peek_next().is_numeric() {
            self.advance(); // consume '.'
            while self.peek().is_numeric() {
                self.advance();
            }
            
            let value: f64 = self.current_lexeme().parse().unwrap();
            self.make_token(TokenKind::FloatLiteral(value))
        } else {
            let value: i64 = self.current_lexeme().parse().unwrap();
            self.make_token(TokenKind::NumberLiteral(value))
        }
    }
    
    fn string(&mut self) -> Token {
        // Ver ADR-005 para implementación de interpolation
        // Por ahora: string simple
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.position.line += 1;
                self.position.column = 1;
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return self.error_token("Unterminated string");
        }
        
        self.advance(); // closing "
        
        let value = self.current_lexeme();
        let value = &value[1..value.len()-1]; // Remove quotes
        self.make_token(TokenKind::StringLiteral(value.to_string()))
    }
}
```

### Tabla de Keywords

Inicialización en `new()`:

```rust
impl Lexer {
    pub fn new(source: String) -> Self {
        let mut keywords = HashMap::new();
        
        // Control Flow (funcional)
        keywords.insert("if".to_string(), TokenKind::If);
        keywords.insert("else".to_string(), TokenKind::Else);
        keywords.insert("match".to_string(), TokenKind::Match);
        keywords.insert("return".to_string(), TokenKind::Return);
        keywords.insert("yield".to_string(), TokenKind::Yield);
        
        // NO loops imperativos (for, while, loop)
        
        // Declarations
        keywords.insert("state".to_string(), TokenKind::State);
        keywords.insert("fn".to_string(), TokenKind::Fn);
        keywords.insert("struct".to_string(), TokenKind::Struct);
        keywords.insert("enum".to_string(), TokenKind::Enum);
        keywords.insert("trait".to_string(), TokenKind::Trait);
        keywords.insert("impl".to_string(), TokenKind::Impl);
        keywords.insert("type".to_string(), TokenKind::Type);
        keywords.insert("interface".to_string(), TokenKind::Interface);
        
        // NO let, const, var, mut
        
        // Visibility
        keywords.insert("public".to_string(), TokenKind::Public);
        keywords.insert("private".to_string(), TokenKind::Private);
        keywords.insert("protected".to_string(), TokenKind::Protected);
        
        // Domain-specific (25 keywords)
        keywords.insert("widget".to_string(), TokenKind::Widget);
        keywords.insert("component".to_string(), TokenKind::Component);
        keywords.insert("service".to_string(), TokenKind::Service);
        keywords.insert("repository".to_string(), TokenKind::Repository);
        keywords.insert("controller".to_string(), TokenKind::Controller);
        keywords.insert("usecase".to_string(), TokenKind::Usecase);
        keywords.insert("dto".to_string(), TokenKind::Dto);
        keywords.insert("entity".to_string(), TokenKind::Entity);
        keywords.insert("valueObject".to_string(), TokenKind::ValueObject);
        keywords.insert("model".to_string(), TokenKind::Model);
        keywords.insert("factory".to_string(), TokenKind::Factory);
        keywords.insert("builder".to_string(), TokenKind::Builder);
        keywords.insert("strategy".to_string(), TokenKind::Strategy);
        keywords.insert("observer".to_string(), TokenKind::Observer);
        keywords.insert("singleton".to_string(), TokenKind::Singleton);
        keywords.insert("adapter".to_string(), TokenKind::Adapter);
        keywords.insert("decorator".to_string(), TokenKind::Decorator);
        keywords.insert("guard".to_string(), TokenKind::Guard);
        keywords.insert("middleware".to_string(), TokenKind::Middleware);
        keywords.insert("interceptor".to_string(), TokenKind::Interceptor);
        keywords.insert("validator".to_string(), TokenKind::Validator);
        keywords.insert("pipe".to_string(), TokenKind::Pipe);
        keywords.insert("task".to_string(), TokenKind::Task);
        keywords.insert("helper".to_string(), TokenKind::Helper);
        keywords.insert("mapper".to_string(), TokenKind::Mapper);
        keywords.insert("serializer".to_string(), TokenKind::Serializer);
        
        // Reactive (8 keywords)
        keywords.insert("Signal".to_string(), TokenKind::Signal);
        keywords.insert("Computed".to_string(), TokenKind::Computed);
        keywords.insert("Effect".to_string(), TokenKind::Effect);
        keywords.insert("Watch".to_string(), TokenKind::Watch);
        keywords.insert("store".to_string(), TokenKind::Store);
        keywords.insert("dispatch".to_string(), TokenKind::Dispatch);
        keywords.insert("provide".to_string(), TokenKind::Provide);
        keywords.insert("inject".to_string(), TokenKind::Inject);
        
        // Types
        keywords.insert("Number".to_string(), TokenKind::Number);
        keywords.insert("Float".to_string(), TokenKind::Float);
        keywords.insert("String".to_string(), TokenKind::String);
        keywords.insert("Bool".to_string(), TokenKind::Bool);
        keywords.insert("Option".to_string(), TokenKind::Option);
        keywords.insert("Result".to_string(), TokenKind::Result);
        
        // Valores
        keywords.insert("true".to_string(), TokenKind::BoolLiteral(true));
        keywords.insert("false".to_string(), TokenKind::BoolLiteral(false));
        keywords.insert("None".to_string(), TokenKind::None);
        keywords.insert("Some".to_string(), TokenKind::Some);
        
        // NO null, undefined, nil
        
        // Error handling
        keywords.insert("try".to_string(), TokenKind::Try);
        keywords.insert("catch".to_string(), TokenKind::Catch);
        keywords.insert("throw".to_string(), TokenKind::Throw);
        keywords.insert("finally".to_string(), TokenKind::Finally);
        
        // Async
        keywords.insert("async".to_string(), TokenKind::Async);
        keywords.insert("await".to_string(), TokenKind::Await);
        
        // Module system
        keywords.insert("import".to_string(), TokenKind::Import);
        keywords.insert("from".to_string(), TokenKind::From);
        keywords.insert("as".to_string(), TokenKind::As);
        keywords.insert("show".to_string(), TokenKind::Show);
        keywords.insert("hide".to_string(), TokenKind::Hide);
        
        // NO export keyword
        
        // ... (más keywords según necesidad)
        
        Self {
            source: source.chars().collect(),
            position: Position { line: 1, column: 1, offset: 0 },
            current: 0,
            start: 0,
            keywords,
        }
    }
}
```

## Justificación

### Por qué Hand-written State Machine:

1. **Performance Óptima**: O(n) garantizado, single pass
2. **Error Recovery Control**: Podemos decidir exactamente cómo recuperar
3. **Debugging Simple**: Stack traces directos, no código generado
4. **Flexibilidad Total**: String interpolation requiere lógica custom
5. **Mantenimiento Claro**: Código explícito, fácil de leer
6. **Zero Dependencies**: No parser generators externos

### Por qué Rust:

1. **Memory Safety**: Perfecto para manipulación de strings y chars
2. **Performance**: Comparable a C++ sin riesgos
3. **Pattern Matching**: Excelente para state machine
4. **Zero-cost Abstractions**: HashMap lookup es óptimo
5. **Strong Type System**: TokenKind como enum es type-safe

## Consecuencias

### Positivas

- ✅ Performance óptima (O(n) single pass)
- ✅ Control total sobre tokenización
- ✅ Error messages precisos
- ✅ Debugging directo
- ✅ Fácil agregar nuevos tokens
- ✅ Sin dependencias externas
- ✅ Position tracking preciso

### Negativas

- ⚠️ Más código manual (~1,000 líneas vs ~200 con generator)
- ⚠️ Mantenimiento manual de keywords
- ⚠️ Tests manuales exhaustivos necesarios

### Mitigaciones

- **Código manual**: Compensado por claridad y control
- **Mantenimiento keywords**: HashMap inicializado en un solo lugar
- **Tests exhaustivos**: Automatizables con macros de Rust

## Alternativas Consideradas

### 1. LALRPOP (Lexer + Parser Generator)
**Rechazada porque:**
- Debugging indirecto (código generado)
- String interpolation requiere lógica custom de todos modos
- Overkill para lexer simple

### 2. nom (Parser Combinator)
**Rechazada porque:**
- Stack usage alto en expresiones complejas
- Performance variable
- Error messages menos claros

### 3. Logos (Lexer Generator macro-based)
**Considerada pero rechazada porque:**
- Menos control sobre error recovery
- Documentación limitada
- Preferimos explícito sobre mágico

## Referencias

- **Jira**: VELA-567 (Sprint 5)
- **Subtask**: TASK-004
- **Roadmap**: vela-roadmap-scrum.csv línea 34
- **Especificación**: docs/language-design/vela-grammar-ebnf.md
- **Keywords**: docs/language-design/reserved-keywords.md

## Implementación

- **Archivo principal**: `src/lexer/mod.rs`
- **Position tracking**: `src/lexer/position.rs`
- **Token types**: `src/lexer/token.rs`
- **Tests**: `tests/unit/lexer/test_lexer.rs`

## Próximos ADRs

- **ADR-005**: String Interpolation Design (TASK-005)
- **ADR-006**: Error Recovery Strategy
- **ADR-007**: Performance Benchmarks
