# TASK-RUST-103: Implementación del Lexer en Rust

## 📋 Información General
- **Historia:** US-RUST-02 (Compiler Foundation)
- **Estado:** En curso 🟡
- **Fecha:** Diciembre 2025
- **Estimación:** 32 horas
- **Tiempo Real:** Pendiente

## 🎯 Objetivo

Implementar un **lexer completo para el lenguaje Vela** que convierta código fuente en una secuencia de tokens. El lexer debe manejar todas las características sintácticas de Vela incluyendo keywords, operadores, literales y comentarios.

## 🔨 Implementación

### Arquitectura del Lexer

#### 1. **Token Types**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn, Let, Var, If, Else, Match, Return, Async, Await,
    Struct, Enum, Interface, Impl, Type, Import, Export,
    Public, Private, Static, Const, Mut, State,

    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BoolLiteral(bool),

    // Operators
    Plus, Minus, Star, Slash, Percent,         // + - * / %
    Equal, NotEqual, Less, LessEqual,          // == != < <=
    Greater, GreaterEqual,                     // > >=
    And, Or, Not,                              // && || !
    Assign, PlusAssign, MinusAssign,           // = += -=
    StarAssign, SlashAssign,                   // *= /=
    Arrow, DoubleArrow,                        // -> =>
    Dot, DoubleDot, TripleDot,                 // . .. ...
    Question, Colon, DoubleColon,              // ? : ::
    Semicolon, Comma,                          // ; ,

    // Delimiters
    LeftParen, RightParen,                     // ( )
    LeftBracket, RightBracket,                 // [ ]
    LeftBrace, RightBrace,                     // { }
    LeftAngle, RightAngle,                     // < >

    // Special
    At, Hash, Dollar, Backtick,                // @ # $ `
    EOF,
}
```

#### 2. **Token Structure**
```rust
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub range: Range,
}
```

#### 3. **Lexer State**
```rust
pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}
```

### Keywords del Lenguaje Vela

#### Core Keywords
- `fn` - Function declaration
- `async` - Async function
- `await` - Await expression
- `return` - Return statement
- `if` - Conditional
- `else` - Alternative conditional
- `match` - Pattern matching
- `state` - Mutable reactive variable
- `const` - Constant (inmutable)

#### Type System Keywords
- `struct` - Struct declaration
- `enum` - Enum declaration
- `interface` - Interface declaration
- `impl` - Implementation
- `type` - Type alias
- `public` - Public modifier
- `private` - Private modifier

#### Module System Keywords
- `import` - Import statement
- `export` - Export (mapped to `public`)
- `package` - Package declaration
- `module` - Module declaration

#### Pattern Keywords
- `let` - Pattern binding (prohibited - use direct assignment)
- `var` - Variable declaration (prohibited - use `state` or direct)
- `mut` - Mutable modifier (prohibited - use `state`)

### Literals Support

#### String Literals
```vela
"hello world"        // Simple string
"hello ${name}"      // Interpolated string
'raw string'         // Raw string (no interpolation)
```

#### Number Literals
```vela
42                   // Integer
3.14                 // Float
0xFF                 // Hexadecimal
0b1010               // Binary
0o777                // Octal
```

#### Boolean Literals
```vela
true
false
```

### Operators Hierarchy

#### Arithmetic Operators (Precedence 1-2)
- `*`, `/`, `%` (highest precedence)
- `+`, `-`

#### Comparison Operators (Precedence 3)
- `==`, `!=`, `<`, `<=`, `>`, `>=`

#### Logical Operators (Precedence 4-5)
- `&&`, `||` (short-circuit)
- `!` (unary)

#### Assignment Operators (Precedence 6)
- `=`, `+=`, `-=`, `*=`, `/=`, `%=`

### Comments Support

#### Single-line Comments
```vela
// This is a comment
fn add(a: Number, b: Number) -> Number { // inline comment
    return a + b
}
```

#### Multi-line Comments
```vela
/*
 * Multi-line comment
 * with multiple lines
 */
fn complex() -> void {
    /* inline multi-line */
}
```

### Error Handling

#### Lexical Errors
- **Unexpected character:** Invalid character in source
- **Unterminated string:** String literal without closing quote
- **Invalid number:** Malformed number literal
- **Invalid escape:** Invalid escape sequence in string

#### Error Recovery
- Skip invalid characters and continue lexing
- Report errors but don't stop compilation
- Provide meaningful error messages with position

### Implementation Steps

#### 1. **Token Recognition**
```rust
impl Lexer {
    fn scan_token(&mut self) -> Result<(), LexError> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '+' => self.add_token(TokenKind::Plus),
            // ... more single character tokens
            '"' => self.string()?,
            '0'..='9' => self.number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.identifier()?,
            ' ' | '\r' | '\t' => {}, // ignore whitespace
            '\n' => self.newline(),
            '/' => self.slash_or_comment()?,
            _ => return Err(LexError::UnexpectedCharacter(c, self.current_pos())),
        }
        Ok(())
    }
}
```

#### 2. **String Lexing**
```rust
fn string(&mut self) -> Result<(), LexError> {
    while self.peek() != '"' && !self.is_at_end() {
        if self.peek() == '\n' {
            return Err(LexError::UnterminatedString);
        }
        if self.peek() == '$' && self.peek_next() == '{' {
            // Handle interpolation
            self.interpolation()?;
        } else {
            self.advance();
        }
    }

    if self.is_at_end() {
        return Err(LexError::UnterminatedString);
    }

    self.advance(); // closing quote
    let value = self.source[self.start+1..self.current-1].to_string();
    self.add_token(TokenKind::StringLiteral(value));
    Ok(())
}
```

#### 3. **Number Lexing**
```rust
fn number(&mut self) -> Result<(), LexError> {
    while self.peek().is_ascii_digit() {
        self.advance();
    }

    // Look for fractional part
    if self.peek() == '.' && self.peek_next().is_ascii_digit() {
        self.advance(); // consume '.'
        while self.peek().is_ascii_digit() {
            self.advance();
        }
    }

    let value = &self.source[self.start..self.current];
    self.add_token(TokenKind::NumberLiteral(value.to_string()));
    Ok(())
}
```

#### 4. **Identifier/Keyword Lexing**
```rust
fn identifier(&mut self) -> Result<(), LexError> {
    while self.peek().is_alphanumeric() || self.peek() == '_' {
        self.advance();
    }

    let text = &self.source[self.start..self.current];
    let kind = match text {
        "fn" => TokenKind::Fn,
        "async" => TokenKind::Async,
        "await" => TokenKind::Await,
        "return" => TokenKind::Return,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "match" => TokenKind::Match,
        "state" => TokenKind::State,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "interface" => TokenKind::Interface,
        "impl" => TokenKind::Impl,
        "type" => TokenKind::Type,
        "import" => TokenKind::Import,
        "export" => TokenKind::Export,
        "public" => TokenKind::Public,
        "private" => TokenKind::Private,
        "true" => TokenKind::BoolLiteral(true),
        "false" => TokenKind::BoolLiteral(false),
        _ => TokenKind::Identifier(text.to_string()),
    };

    self.add_token(kind);
    Ok(())
}
```

### Testing Strategy

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let tokens = lex("fn async return").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Async);
        assert_eq!(tokens[2].kind, TokenKind::Return);
    }

    #[test]
    fn test_string_literals() {
        let tokens = lex(r#""hello world""#).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello world".to_string()));
    }

    #[test]
    fn test_operators() {
        let tokens = lex("a + b == c").unwrap();
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[3].kind, TokenKind::Equal);
    }
}
```

#### Integration Tests
- Lex complete Vela programs
- Verify token sequences match expected output
- Test error recovery scenarios

### Performance Considerations

#### Memory Efficiency
- Use `&str` slices instead of `String` where possible
- Pre-allocate token vector with estimated capacity
- Avoid unnecessary allocations in hot paths

#### Speed Optimizations
- Single-pass lexing
- Minimal bounds checking in inner loops
- Fast character classification functions

### Error Types

```rust
#[derive(Debug, Clone)]
pub enum LexError {
    UnexpectedCharacter(char, Position),
    UnterminatedString,
    InvalidEscapeSequence(String),
    InvalidNumberLiteral(String),
}

#[derive(Debug, Clone)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
}
```

## ✅ Criterios de Aceptación

- [ ] **Keywords completos:** Todos los keywords de Vela reconocidos
- [ ] **Literals support:** Strings, numbers, booleans con escape sequences
- [ ] **Operators hierarchy:** Precedencia correcta de operadores
- [ ] **Comments handling:** Single-line y multi-line comments
- [ ] **Error recovery:** Continúa lexing después de errores
- [ ] **Position tracking:** Rango preciso para cada token
- [ ] **Performance:** < 1ms para archivos típicos de 1000 líneas
- [ ] **Tests:** Cobertura > 90% con tests unitarios e integración

## 🔗 Referencias

- **Historia:** [US-RUST-02](https://velalang.atlassian.net/browse/US-RUST-02)
- **TASK anterior:** [TASK-RUST-102](TASK-RUST-102.md)
- **Especificación Vela:** `docs/specification/`
- **Python lexer original:** `src/lexer/` (legacy)

## 📊 Métricas Esperadas

- **Complejidad ciclomática:** < 15 por función
- **Coverage:** > 90%
- **Performance:** O(n) time complexity
- **Memory:** < 2x input size
- **Error rate:** < 1% false positives</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\features\US-RUST-02\TASK-RUST-103.md