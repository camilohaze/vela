# TASK-RUST-104: Parser Implementation en Rust

## 📋 Información General
- **Historia:** US-RUST-02 (Compiler Foundation)
- **Estado:** En curso 🟡
- **Fecha:** Diciembre 2025
- **Estimación:** 80 horas
- **Tiempo Real:** Pendiente

## 🎯 Objetivo

Implementar un **parser recursivo descendente completo** para el lenguaje Vela que convierta tokens del lexer en un AST (Abstract Syntax Tree). El parser debe manejar toda la gramática de Vela con precedence climbing para expresiones y error recovery avanzado.

## 🔨 Implementación

### Arquitectura del Parser

#### 1. **Parser State**
```rust
pub struct Parser {
    tokens: Vec<crate::lexer::Token>,
    current: usize,
    source_path: std::path::PathBuf,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<crate::lexer::Token>, source_path: &std::path::Path) -> Self {
        Self {
            tokens,
            current: 0,
            source_path: source_path.to_path_buf(),
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> CompileResult<crate::ast::Program> {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(decl) => declarations.push(decl),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }

        Ok(crate::ast::Program {
            imports: vec![], // TODO: parse imports
            declarations,
        })
    }
}
```

#### 2. **Precedence Climbing para Expressions**

```rust
impl Parser {
    fn expression(&mut self) -> ParseResult<Expression> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> ParseResult<Expression> {
        // Parse left operand
        let mut left = self.unary()?;

        while self.can_continue_parsing(precedence) {
            let operator = self.current_token_kind();
            let op_precedence = self.get_precedence(operator);

            if op_precedence <= precedence {
                break;
            }

            self.advance();
            let right = self.parse_precedence(op_precedence)?;
            left = self.create_binary_expr(left, operator, right);
        }

        Ok(left)
    }

    fn unary(&mut self) -> ParseResult<Expression> {
        if self.match_token(TokenKind::Not) || self.match_token(TokenKind::Minus) {
            let operator = self.previous_token_kind();
            let operand = self.unary()?;
            return Ok(Expression::Unary(UnaryExpr {
                operator,
                operand: Box::new(operand),
                range: self.create_range(),
            }));
        }

        self.primary()
    }
}
```

#### 3. **Parsing de Statements**

```rust
impl Parser {
    fn statement(&mut self) -> ParseResult<Statement> {
        match self.current_token_kind() {
            TokenKind::If => self.if_statement(),
            TokenKind::Match => self.match_statement(),
            TokenKind::State => self.state_declaration(),
            TokenKind::Return => self.return_statement(),
            TokenKind::LeftBrace => self.block_statement(),
            _ => {
                if self.is_declaration_start() {
                    self.declaration_statement()
                } else {
                    self.expression_statement()
                }
            }
        }
    }

    fn if_statement(&mut self) -> ParseResult<Statement> {
        self.consume(TokenKind::If)?;
        let condition = self.expression()?;
        let then_branch = self.statement()?;
        let else_branch = if self.match_token(TokenKind::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
            range: self.create_range(),
        }))
    }
}
```

#### 4. **Parsing de Declarations**

```rust
impl Parser {
    fn declaration(&mut self) -> ParseResult<Declaration> {
        match self.current_token_kind() {
            TokenKind::Fn => self.function_declaration(),
            TokenKind::Struct => self.struct_declaration(),
            TokenKind::Enum => self.enum_declaration(),
            TokenKind::Type => self.type_declaration(),
            TokenKind::Const => self.const_declaration(),
            TokenKind::Public => {
                self.advance(); // consume public
                self.public_declaration()
            }
            _ => Err(self.error("Expected declaration")),
        }
    }

    fn function_declaration(&mut self) -> ParseResult<Declaration> {
        self.consume(TokenKind::Fn)?;
        let name = self.consume_identifier()?;
        self.consume(TokenKind::LeftParen)?;

        let parameters = self.parse_parameters()?;
        self.consume(TokenKind::RightParen)?;

        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.block_statement()?;

        Ok(Declaration::Function(FunctionDecl {
            name,
            parameters,
            return_type,
            body,
            range: self.create_range(),
        }))
    }
}
```

### Gramática Formal de Vela

#### Expressions (con Precedence)
```
expression ::= assignment

assignment ::= identifier "=" assignment | logic_or

logic_or ::= logic_and ("||" logic_and)*

logic_and ::= equality ("&&" equality)*

equality ::= comparison (("==" | "!=") comparison)*

comparison ::= term ((">" | ">=" | "<" | "<=") term)*

term ::= factor (("+" | "-") factor)*

factor ::= unary (("*" | "/" | "%") unary)*

unary ::= "!" unary | "-" unary | primary

primary ::= NUMBER | STRING | IDENTIFIER | "true" | "false"
          | "(" expression ")" | call | struct_literal
```

#### Statements
```
statement ::= expression_statement
            | if_statement
            | match_statement
            | return_statement
            | block_statement
            | declaration_statement

expression_statement ::= expression ";"

if_statement ::= "if" expression statement ("else" statement)?

return_statement ::= "return" expression? ";"

block_statement ::= "{" statement* "}"
```

#### Declarations
```
declaration ::= function_declaration
               | struct_declaration
               | enum_declaration
               | type_declaration
               | const_declaration

function_declaration ::= "fn" IDENTIFIER "(" parameters? ")" ("->" type)? block_statement

struct_declaration ::= "struct" IDENTIFIER "{" field* "}"

enum_declaration ::= "enum" IDENTIFIER "{" variant* "}"

type_declaration ::= "type" IDENTIFIER "=" type

const_declaration ::= "const" IDENTIFIER ":" type "=" expression
```

### Error Recovery

#### Synchronize Points
```rust
impl Parser {
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous_token_kind() == TokenKind::Semicolon {
                return;
            }

            match self.current_token_kind() {
                TokenKind::Fn | TokenKind::Struct | TokenKind::Enum |
                TokenKind::Type | TokenKind::Const | TokenKind::If |
                TokenKind::Return | TokenKind::Match => return,
                _ => self.advance(),
            }
        }
    }
}
```

#### Error Reporting
```rust
impl Parser {
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            location: self.current_token().range.start,
            expected: vec![], // TODO: add expected tokens
        }
    }

    fn consume(&mut self, kind: TokenKind) -> ParseResult<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(&format!("Expected {:?}", kind)))
        }
    }
}
```

### Parsing de Tipos

```rust
impl Parser {
    fn parse_type(&mut self) -> ParseResult<TypeAnnotation> {
        match self.current_token_kind() {
            TokenKind::Identifier => {
                let name = self.consume_identifier()?;
                self.parse_complex_type(name)
            }
            TokenKind::LeftBracket => self.array_type(),
            TokenKind::LeftParen => self.tuple_type(),
            TokenKind::Fn => self.function_type(),
            _ => Err(self.error("Expected type")),
        }
    }

    fn parse_complex_type(&mut self, base_name: String) -> ParseResult<TypeAnnotation> {
        let mut ty = TypeAnnotation::Named(base_name);

        // Handle generics
        if self.match_token(TokenKind::LeftAngle) {
            let mut args = Vec::new();
            while !self.check(TokenKind::RightAngle) && !self.is_at_end() {
                args.push(self.parse_type()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
            self.consume(TokenKind::RightAngle)?;
            ty = TypeAnnotation::Generic(Box::new(ty), args);
        }

        // Handle union types
        if self.match_token(TokenKind::Pipe) {
            let mut types = vec![ty];
            while self.match_token(TokenKind::Pipe) {
                types.push(self.parse_type()?);
            }
            ty = TypeAnnotation::Union(types);
        }

        Ok(ty)
    }
}
```

### Testing Strategy

#### Unit Tests por Componente
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Result<Program, Vec<ParseError>> {
        let mut lexer = Lexer::new(source, &PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens, &PathBuf::from("test.vela"));
        parser.parse().map_err(|_| parser.errors)
    }

    #[test]
    fn test_simple_function() {
        let source = r#"
            fn add(a: Number, b: Number) -> Number {
                return a + b;
            }
        "#;
        let program = parse(source).unwrap();
        assert_eq!(program.declarations.len(), 1);
        // TODO: more assertions
    }

    #[test]
    fn test_if_statement() {
        let source = r#"
            if x > 0 {
                return x;
            } else {
                return -x;
            }
        "#;
        let program = parse(source).unwrap();
        // TODO: verify AST structure
    }

    #[test]
    fn test_error_recovery() {
        let source = "fn broken( { return 1; } fn valid() { return 2; }";
        let result = parse(source);
        assert!(result.is_err());
        // Should still parse the valid function after error
    }
}
```

#### Integration Tests
- Parse complete Vela programs
- Verify AST structure matches expected
- Test precedence rules
- Test error recovery scenarios

### Performance Considerations

#### Memory Management
- Use `Box` for recursive structures to avoid stack overflow
- Pre-allocate vectors with estimated capacity
- Avoid unnecessary allocations in hot paths

#### Error Handling
- Collect all errors instead of stopping at first error
- Provide detailed error messages with location
- Support error recovery to continue parsing

### Features Avanzadas

#### Pattern Matching
```rust
fn match_statement(&mut self) -> ParseResult<Statement> {
    self.consume(TokenKind::Match)?;
    let expression = self.expression()?;
    self.consume(TokenKind::LeftBrace)?;

    let mut arms = Vec::new();
    while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
        let pattern = self.parse_pattern()?;
        self.consume(TokenKind::DoubleArrow)?;
        let body = self.statement()?;
        arms.push(MatchArm { pattern, body });

        if !self.match_token(TokenKind::Comma) {
            break;
        }
    }

    self.consume(TokenKind::RightBrace)?;
    Ok(Statement::Match(MatchStatement {
        expression: Box::new(expression),
        arms,
        range: self.create_range(),
    }))
}
```

#### Async Functions
```rust
fn function_declaration(&mut self) -> ParseResult<Declaration> {
    let is_async = self.match_token(TokenKind::Async);
    self.consume(TokenKind::Fn)?;
    // ... rest of function parsing
    Ok(Declaration::Function(FunctionDecl {
        is_async,
        // ... other fields
    }))
}
```

## ✅ Criterios de Aceptación

- [ ] **Gramática completa:** Parser maneja toda la sintaxis de Vela
- [ ] **Precedence correcta:** Expressions parseadas con precedencia correcta
- [ ] **Error recovery:** Continúa parsing después de errores
- [ ] **AST válido:** Produce AST correcto para todos los constructos
- [ ] **Performance:** < 2ms para archivos típicos de 1000 líneas
- [ ] **Tests:** Cobertura > 90% con tests unitarios e integración
- [ ] **Error messages:** Mensajes de error claros con ubicación precisa

## 🔗 Referencias

- **Historia:** [US-RUST-02](https://velalang.atlassian.net/browse/US-RUST-02)
- **TASK anterior:** [TASK-RUST-103](TASK-RUST-103.md)
- **AST Types:** `src/ast.rs`
- **Lexer:** `src/lexer.rs`
- **Especificación Vela:** `docs/specification/`

## 📊 Métricas Esperadas

- **Complejidad ciclomática:** < 20 por función
- **Coverage:** > 90%
- **Performance:** O(n) time complexity
- **Memory:** < 3x input size
- **Error recovery:** > 80% success rate</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\features\US-RUST-02\TASK-RUST-104.md