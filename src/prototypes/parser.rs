/// Prototype Parser for Vela Language
///
/// JIRA: VELA-565 (Sprint 4 - Prototype & Validation)
/// TASK: TASK-000W - Implementar prototipo de parser
/// Fecha: 2025-11-30
///
/// Este es un proof of concept para validar:
/// - Recursive descent parsing design
/// - AST structure básico
/// - Memory usage del AST
/// - Feasibility de error recovery (futuro)
///
/// Construcciones soportadas (~5):
/// - let bindings: `let x = expr;`
/// - function definitions: `fn name(params) { body }`
/// - if expressions: `if cond { then } else { else_branch }`
/// - binary expressions: `a + b`, `x == y`
/// - literals y identificadores

use crate::prototypes::lexer::{Lexer, Token, TokenKind};
use std::fmt;

// ===== AST Node Types =====

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Number(i64),
    String(String),
    Bool(bool),
    Identifier(String),

    // Binary operations
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    // Function call
    Call {
        callee: String,
        args: Vec<Expr>,
    },

    // If expression
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },

    // Block expression (for if branches)
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Gt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // Let binding
    Let { name: String, value: Expr },

    // Function definition
    Fn {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    // Return statement
    Return(Option<Expr>),

    // Expression statement
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{}", n),
            Expr::String(s) => write!(f, "\"{}\"", s),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Identifier(id) => write!(f, "{}", id),
            Expr::Binary { left, op, right } => {
                write!(f, "({:?} {} {})", op, left, right)
            }
            Expr::Call { callee, args } => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                write!(f, "if {} {{ {} }}", cond, then_branch)?;
                if let Some(else_br) = else_branch {
                    write!(f, " else {{ {} }}", else_br)?;
                }
                Ok(())
            }
            Expr::Block(stmts) => {
                write!(f, "{{ {} stmts }}", stmts.len())
            }
        }
    }
}

// ===== Parser =====

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    /// Parse a complete program
    pub fn parse(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            stmts.push(self.statement()?);
        }

        Ok(Program { stmts })
    }

    // ===== Statement Parsing =====

    fn statement(&mut self) -> Result<Stmt, String> {
        match self.peek().kind {
            TokenKind::Let => self.let_statement(),
            TokenKind::Fn => self.fn_statement(),
            TokenKind::Return => self.return_statement(),
            _ => {
                let expr = self.expression()?;
                self.expect(TokenKind::Semicolon, "Expected ';' after expression")?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn let_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'let'

        let name = match &self.peek().kind {
            TokenKind::Identifier(id) => {
                let name = id.clone();
                self.advance();
                name
            }
            _ => return Err(format!("Expected identifier after 'let', got {:?}", self.peek())),
        };

        self.expect(TokenKind::Equal, "Expected '=' after variable name")?;

        let value = self.expression()?;

        self.expect(TokenKind::Semicolon, "Expected ';' after let statement")?;

        Ok(Stmt::Let { name, value })
    }

    fn fn_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'fn'

        let name = match &self.peek().kind {
            TokenKind::Identifier(id) => {
                let name = id.clone();
                self.advance();
                name
            }
            _ => return Err(format!("Expected function name, got {:?}", self.peek())),
        };

        self.expect(TokenKind::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();

        if !matches!(self.peek().kind, TokenKind::RightParen) {
            loop {
                match &self.peek().kind {
                    TokenKind::Identifier(id) => {
                        params.push(id.clone());
                        self.advance();
                    }
                    _ => return Err(format!("Expected parameter name, got {:?}", self.peek())),
                }

                if !matches!(self.peek().kind, TokenKind::RightParen) {
                    // Note: Simplified - no comma support in this prototype
                    break;
                }
            }
        }

        self.expect(TokenKind::RightParen, "Expected ')' after parameters")?;

        self.expect(TokenKind::LeftBrace, "Expected '{' before function body")?;

        let mut body = Vec::new();

        while !matches!(self.peek().kind, TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.statement()?);
        }

        self.expect(TokenKind::RightBrace, "Expected '}' after function body")?;

        Ok(Stmt::Fn { name, params, body })
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'return'

        if matches!(self.peek().kind, TokenKind::Semicolon) {
            self.advance();
            return Ok(Stmt::Return(None));
        }

        let value = self.expression()?;
        self.expect(TokenKind::Semicolon, "Expected ';' after return value")?;

        Ok(Stmt::Return(Some(value)))
    }

    // ===== Expression Parsing =====

    fn expression(&mut self) -> Result<Expr, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut left = self.comparison()?;

        while matches!(
            self.peek().kind,
            TokenKind::EqualEqual | TokenKind::BangEqual
        ) {
            let op = match self.peek().kind {
                TokenKind::EqualEqual => BinaryOp::Eq,
                TokenKind::BangEqual => BinaryOp::Ne,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.term()?;

        while matches!(self.peek().kind, TokenKind::Less | TokenKind::Greater) {
            let op = match self.peek().kind {
                TokenKind::Less => BinaryOp::Lt,
                TokenKind::Greater => BinaryOp::Gt,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.term()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut left = self.factor()?;

        while matches!(self.peek().kind, TokenKind::Plus | TokenKind::Minus) {
            let op = match self.peek().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.factor()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut left = self.primary()?;

        while matches!(self.peek().kind, TokenKind::Star | TokenKind::Slash) {
            let op = match self.peek().kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.primary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match &self.peek().kind {
            TokenKind::Number(n) => {
                let num = *n;
                self.advance();
                Ok(Expr::Number(num))
            }

            TokenKind::StringLit(s) => {
                let string = s.clone();
                self.advance();
                Ok(Expr::String(string))
            }

            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }

            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }

            TokenKind::Identifier(id) => {
                let name = id.clone();
                self.advance();

                // Check for function call
                if matches!(self.peek().kind, TokenKind::LeftParen) {
                    self.advance(); // consume '('

                    let mut args = Vec::new();

                    if !matches!(self.peek().kind, TokenKind::RightParen) {
                        loop {
                            args.push(self.expression()?);

                            if matches!(self.peek().kind, TokenKind::RightParen) {
                                break;
                            }
                            // Note: Simplified - no comma support
                        }
                    }

                    self.expect(TokenKind::RightParen, "Expected ')' after arguments")?;

                    Ok(Expr::Call { callee: name, args })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }

            TokenKind::If => self.if_expression(),

            TokenKind::LeftParen => {
                self.advance(); // consume '('
                let expr = self.expression()?;
                self.expect(TokenKind::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }

            _ => Err(format!("Unexpected token in expression: {:?}", self.peek())),
        }
    }

    fn if_expression(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'if'

        let cond = self.expression()?;

        self.expect(TokenKind::LeftBrace, "Expected '{' after if condition")?;

        let mut then_stmts = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace) && !self.is_at_end() {
            then_stmts.push(self.statement()?);
        }

        self.expect(TokenKind::RightBrace, "Expected '}' after if body")?;

        let else_branch = if matches!(self.peek().kind, TokenKind::Else) {
            self.advance(); // consume 'else'

            self.expect(TokenKind::LeftBrace, "Expected '{' after else")?;

            let mut else_stmts = Vec::new();
            while !matches!(self.peek().kind, TokenKind::RightBrace) && !self.is_at_end() {
                else_stmts.push(self.statement()?);
            }

            self.expect(TokenKind::RightBrace, "Expected '}' after else body")?;

            Some(Box::new(Expr::Block(else_stmts)))
        } else {
            None
        };

        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(Expr::Block(then_stmts)),
            else_branch,
        })
    }

    // ===== Parser Helpers =====

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<(), String> {
        if std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind) {
            self.advance();
            Ok(())
        } else {
            Err(format!("{}: expected {:?}, got {:?}", message, kind, self.peek()))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }
}

/// Helper function to parse from source string
pub fn parse_source(source: &str) -> Result<Program, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let_statement() {
        let source = "let x = 42;";
        let program = parse_source(source).unwrap();

        assert_eq!(program.stmts.len(), 1);

        match &program.stmts[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "x");
                assert_eq!(value, &Expr::Number(42));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let source = "let result = 10 + 20 * 2;";
        let program = parse_source(source).unwrap();

        match &program.stmts[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "result");

                // Should be: 10 + (20 * 2) due to precedence
                if let Expr::Binary { left, op, right } = value {
                    assert_eq!(op, &BinaryOp::Add);
                    assert_eq!(**left, Expr::Number(10));

                    if let Expr::Binary { left, op, right } = &**right {
                        assert_eq!(op, &BinaryOp::Mul);
                        assert_eq!(**left, Expr::Number(20));
                        assert_eq!(**right, Expr::Number(2));
                    } else {
                        panic!("Expected multiplication on right side");
                    }
                } else {
                    panic!("Expected binary expression");
                }
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_parse_function() {
        let source = r#"
            fn add(a, b) {
                return a + b;
            }
        "#;
        let program = parse_source(source).unwrap();

        assert_eq!(program.stmts.len(), 1);

        match &program.stmts[0] {
            Stmt::Fn { name, params, body } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], "a");
                assert_eq!(params[1], "b");
                assert_eq!(body.len(), 1);

                match &body[0] {
                    Stmt::Return(Some(expr)) => {
                        assert!(matches!(expr, Expr::Binary { .. }));
                    }
                    _ => panic!("Expected return statement"),
                }
            }
            _ => panic!("Expected Fn statement"),
        }
    }

    #[test]
    fn test_parse_if_expression() {
        let source = r#"
            let x = if true {
                let y = 10;
                y;
            } else {
                let z = 20;
                z;
            };
        "#;
        let program = parse_source(source).unwrap();

        match &program.stmts[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "x");

                if let Expr::If {
                    cond,
                    then_branch,
                    else_branch,
                } = value
                {
                    assert_eq!(**cond, Expr::Bool(true));
                    assert!(matches!(**then_branch, Expr::Block(_)));
                    assert!(else_branch.is_some());
                } else {
                    panic!("Expected if expression");
                }
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_precedence() {
        let source = "let x = 2 + 3 * 4;";
        let program = parse_source(source).unwrap();

        match &program.stmts[0] {
            Stmt::Let { name: _, value } => {
                // Should parse as: 2 + (3 * 4)
                if let Expr::Binary { left, op, right } = value {
                    assert_eq!(op, &BinaryOp::Add);
                    assert_eq!(**left, Expr::Number(2));

                    // Right should be 3 * 4
                    assert!(matches!(
                        **right,
                        Expr::Binary {
                            op: BinaryOp::Mul,
                            ..
                        }
                    ));
                } else {
                    panic!("Expected binary expression");
                }
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_function_call() {
        let source = "let result = add(10, 20);";
        let program = parse_source(source).unwrap();

        match &program.stmts[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "result");

                if let Expr::Call { callee, args } = value {
                    assert_eq!(callee, "add");
                    assert_eq!(args.len(), 2);
                    assert_eq!(args[0], Expr::Number(10));
                    assert_eq!(args[1], Expr::Number(20));
                } else {
                    panic!("Expected function call");
                }
            }
            _ => panic!("Expected Let statement"),
        }
    }
}
