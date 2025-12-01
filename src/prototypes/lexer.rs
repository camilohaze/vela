/// Prototype Lexer for Vela Language
///
/// JIRA: VELA-565 (Sprint 4 - Prototype & Validation)
/// TASK: TASK-000V - Implementar prototipo de lexer
/// Fecha: 2025-11-30
///
/// Este es un proof of concept para validar:
/// - State machine design para tokenización
/// - Performance de scanning básico
/// - Feasibility de implementación en Rust
///
/// Tokens soportados (~20): let, fn, if, else, return, true, false,
/// identifier, number, string, +, -, *, /, =, ==, !=, <, >, (, ), {, }, ;

use std::fmt;

/// Token types reconocidos por el lexer
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Let,
    Fn,
    If,
    Else,
    Return,
    True,
    False,

    // Literals
    Identifier(String),
    Number(i64),
    StringLit(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    Greater,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,

    // Special
    Eof,
    Error(String),
}

/// Token con ubicación en el código fuente
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            kind,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} '{}' at {}:{}",
            self.kind, self.lexeme, self.line, self.column
        )
    }
}

/// Lexer con state machine para tokenización
pub struct Lexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Crea un nuevo lexer a partir de código fuente
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokeniza todo el código fuente
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        tokens
    }

    /// Obtiene el siguiente token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start_column = self.column;

        if self.is_at_end() {
            return Token::new(TokenKind::Eof, String::new(), self.line, self.column);
        }

        let c = self.advance();

        match c {
            // Operators
            '+' => self.make_token(TokenKind::Plus, "+", start_column),
            '-' => self.make_token(TokenKind::Minus, "-", start_column),
            '*' => self.make_token(TokenKind::Star, "*", start_column),
            '/' => self.make_token(TokenKind::Slash, "/", start_column),
            '<' => self.make_token(TokenKind::Less, "<", start_column),
            '>' => self.make_token(TokenKind::Greater, ">", start_column),

            // Equal or EqualEqual
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::EqualEqual, "==", start_column)
                } else {
                    self.make_token(TokenKind::Equal, "=", start_column)
                }
            }

            // BangEqual
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::BangEqual, "!=", start_column)
                } else {
                    self.make_error("Expected '=' after '!'", start_column)
                }
            }

            // Delimiters
            '(' => self.make_token(TokenKind::LeftParen, "(", start_column),
            ')' => self.make_token(TokenKind::RightParen, ")", start_column),
            '{' => self.make_token(TokenKind::LeftBrace, "{", start_column),
            '}' => self.make_token(TokenKind::RightBrace, "}", start_column),
            ';' => self.make_token(TokenKind::Semicolon, ";", start_column),

            // String literals
            '"' => self.scan_string(start_column),

            // Numbers
            '0'..='9' => self.scan_number(c, start_column),

            // Keywords or Identifiers
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier_or_keyword(c, start_column),

            _ => self.make_error(&format!("Unexpected character: {}", c), start_column),
        }
    }

    // ===== State Machine Helpers =====

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] == expected {
            self.current += 1;
            self.column += 1;
            true
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    // ===== Token Scanners =====

    fn scan_string(&mut self, start_column: usize) -> Token {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            value.push(self.advance());
        }

        if self.is_at_end() {
            return self.make_error("Unterminated string", start_column);
        }

        // Consume closing "
        self.advance();

        Token::new(
            TokenKind::StringLit(value.clone()),
            format!("\"{}\"", value),
            self.line,
            start_column,
        )
    }

    fn scan_number(&mut self, first: char, start_column: usize) -> Token {
        let mut num_str = String::from(first);

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num_str.push(self.advance());
        }

        match num_str.parse::<i64>() {
            Ok(n) => Token::new(
                TokenKind::Number(n),
                num_str.clone(),
                self.line,
                start_column,
            ),
            Err(_) => self.make_error(&format!("Invalid number: {}", num_str), start_column),
        }
    }

    fn scan_identifier_or_keyword(&mut self, first: char, start_column: usize) -> Token {
        let mut ident = String::from(first);

        while !self.is_at_end() {
            let c = self.peek();
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.advance());
            } else {
                break;
            }
        }

        let kind = match ident.as_str() {
            "let" => TokenKind::Let,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "return" => TokenKind::Return,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(ident.clone()),
        };

        Token::new(kind, ident, self.line, start_column)
    }

    // ===== Token Builders =====

    fn make_token(&self, kind: TokenKind, lexeme: &str, column: usize) -> Token {
        Token::new(kind, lexeme.to_string(), self.line, column)
    }

    fn make_error(&self, message: &str, column: usize) -> Token {
        Token::new(
            TokenKind::Error(message.to_string()),
            String::new(),
            self.line,
            column,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("let fn if else return true false");
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 8); // 7 keywords + EOF

        assert!(matches!(tokens[0].kind, TokenKind::Let));
        assert!(matches!(tokens[1].kind, TokenKind::Fn));
        assert!(matches!(tokens[2].kind, TokenKind::If));
        assert!(matches!(tokens[3].kind, TokenKind::Else));
        assert!(matches!(tokens[4].kind, TokenKind::Return));
        assert!(matches!(tokens[5].kind, TokenKind::True));
        assert!(matches!(tokens[6].kind, TokenKind::False));
        assert!(matches!(tokens[7].kind, TokenKind::Eof));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / = == != < >");
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 10); // 9 operators + EOF

        assert!(matches!(tokens[0].kind, TokenKind::Plus));
        assert!(matches!(tokens[1].kind, TokenKind::Minus));
        assert!(matches!(tokens[2].kind, TokenKind::Star));
        assert!(matches!(tokens[3].kind, TokenKind::Slash));
        assert!(matches!(tokens[4].kind, TokenKind::Equal));
        assert!(matches!(tokens[5].kind, TokenKind::EqualEqual));
        assert!(matches!(tokens[6].kind, TokenKind::BangEqual));
        assert!(matches!(tokens[7].kind, TokenKind::Less));
        assert!(matches!(tokens[8].kind, TokenKind::Greater));
    }

    #[test]
    fn test_delimiters() {
        let mut lexer = Lexer::new("( ) { } ;");
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 6); // 5 delimiters + EOF

        assert!(matches!(tokens[0].kind, TokenKind::LeftParen));
        assert!(matches!(tokens[1].kind, TokenKind::RightParen));
        assert!(matches!(tokens[2].kind, TokenKind::LeftBrace));
        assert!(matches!(tokens[3].kind, TokenKind::RightBrace));
        assert!(matches!(tokens[4].kind, TokenKind::Semicolon));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 0 999");
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 4); // 3 numbers + EOF

        assert_eq!(tokens[0].kind, TokenKind::Number(42));
        assert_eq!(tokens[1].kind, TokenKind::Number(0));
        assert_eq!(tokens[2].kind, TokenKind::Number(999));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello" "world""#);
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 3); // 2 strings + EOF

        assert_eq!(tokens[0].kind, TokenKind::StringLit("hello".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::StringLit("world".to_string()));
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("foo bar baz123 _private");
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 5); // 4 identifiers + EOF

        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("bar".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("baz123".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Identifier("_private".to_string()));
    }

    #[test]
    fn test_simple_program() {
        let source = r#"
            let x = 42;
            fn add(a, b) {
                return a + b;
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        // Should have: let, x, =, 42, ;, fn, add, (, a, ,, b, ), {, return, a, +, b, ;, }, EOF
        assert!(tokens.len() > 15);

        // Verify first few tokens
        assert!(matches!(tokens[0].kind, TokenKind::Let));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert!(matches!(tokens[2].kind, TokenKind::Equal));
        assert_eq!(tokens[3].kind, TokenKind::Number(42));
        assert!(matches!(tokens[4].kind, TokenKind::Semicolon));
    }

    #[test]
    fn test_line_tracking() {
        let source = "let x = 1;\nlet y = 2;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        // First line
        assert_eq!(tokens[0].line, 1);

        // Second line
        let y_token = tokens.iter().find(|t| t.lexeme == "y").unwrap();
        assert_eq!(y_token.line, 2);
    }
}
