/*
Módulo lexer - Analizador léxico completo para Vela

Implementación completa del lexer que tokeniza código fuente Vela
en una secuencia de tokens con información de posición.
*/

use crate::error::{CompileResult, CompileError, LexicalError, SourceLocation};
use std::path::Path;

/// Tipos de tokens reconocidos por el lexer
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn, Async, Await, Return, If, Else, Match, State, Const, Dispatch,
    Struct, Enum, Interface, Impl, Type, Import, Export,
    Public, Private, Package, Module, Extension, Library,

    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BoolLiteral(bool),

    // Operators
    Plus, Minus, Star, Slash, Percent,           // + - * / %
    Equal, NotEqual, Less, LessEqual,            // == != < <=
    Greater, GreaterEqual,                       // > >=
    And, Or, Not,                                // && || !
    Assign, PlusAssign, MinusAssign,             // = += -=
    StarAssign, SlashAssign, PercentAssign,      // *= /= %=
    Arrow, DoubleArrow,                          // -> =>
    Dot, DoubleDot, TripleDot,                   // . .. ...
    Question, Colon, DoubleColon,                // ? : ::
    Semicolon, Comma,                            // ; ,

    // Delimiters
    LeftParen, RightParen,                       // ( )
    LeftBracket, RightBracket,                   // [ ]
    LeftBrace, RightBrace,                       // { }
    LeftAngle, RightAngle,                       // < >

    // Special
    At, Hash, Dollar, Backtick,                  // @ # $ `
    EOF,
}

/// Representa un token individual con su información de posición
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub range: crate::ast::Range,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, range: crate::ast::Range) -> Self {
        Token { kind, lexeme, range }
    }
}

/// Resultado del proceso de lexing
#[derive(Debug)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
}

/// Errores que pueden ocurrir durante el lexing
#[derive(Debug, Clone)]
pub enum LexError {
    UnexpectedCharacter(char, crate::ast::Position),
    UnterminatedString(crate::ast::Position),
    InvalidEscapeSequence(String, crate::ast::Position),
    InvalidNumberLiteral(String, crate::ast::Position),
}

/// Lexer principal que convierte código fuente en tokens
pub struct Lexer {
    source: String,
    source_path: std::path::PathBuf,
    tokens: Vec<Token>,
    errors: Vec<LexError>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Crea un nuevo lexer para el código fuente dado
    pub fn new(source: &str, source_path: &Path) -> Self {
        Lexer {
            source: source.to_string(),
            source_path: source_path.to_path_buf(),
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
        }
    }

    /// Ejecuta el proceso completo de tokenización
    pub fn tokenize(&mut self) -> CompileResult<LexResult> {
        println!("🔤 Starting tokenize loop, source length: {}", self.source.len());
        let mut iteration_count = 0;
        let max_iterations = self.source.len() * 2; // Safety limit

        while !self.is_at_end() {
            iteration_count += 1;
            if iteration_count > max_iterations {
                println!("🚨 INFINITE LOOP DETECTED! Iteration: {}, current: {}, source.len(): {}", iteration_count, self.current, self.source.len());
                return Err(CompileError::Lexical(LexicalError {
                    message: "Infinite loop detected in lexer".to_string(),
                    location: SourceLocation {
                        line: self.line,
                        column: self.column,
                        offset: self.current,
                    },
                }));
            }

            if iteration_count % 100 == 0 {
                println!("🔄 Tokenize iteration: {}, current: {}, remaining: {}", iteration_count, self.current, self.source.len() - self.current);
            }

            self.start = self.current;
            println!("🔤 Processing char at position {}: '{}' (ascii: {})", self.current, self.peek(), self.peek() as u8);

            if let Err(e) = self.scan_token() {
                println!("❌ Lex error: {:?}", e);
                self.errors.push(e);
                // Error recovery: skip invalid character and reset start position
                self.advance();
                self.start = self.current;
                // Additional safety: if we've had too many consecutive errors, skip ahead
                if self.errors.len() > 10 {
                    println!("🚨 Too many consecutive errors, skipping to next line or whitespace");
                    while !self.is_at_end() && !matches!(self.peek(), ' ' | '\t' | '\n' | '\r') {
                        self.advance();
                    }
                    self.start = self.current;
                }
            }
        }

        println!("✅ Tokenize loop completed, {} iterations, {} tokens generated", iteration_count, self.tokens.len());

        // Add EOF token
        let eof_pos = crate::ast::Position { line: self.line, column: self.column };
        let eof_range = crate::ast::Range {
            start: eof_pos.clone(),
            end: eof_pos,
        };
        self.tokens.push(Token::new(TokenKind::EOF, "".to_string(), eof_range));

        Ok(LexResult {
            tokens: self.tokens.clone(),
            errors: self.errors.clone(),
        })
    }

    /// Escanea un token individual
    fn scan_token(&mut self) -> Result<(), LexError> {
        let c = self.advance();
        println!("🔤 scan_token processing char: '{}' (ascii: {})", c, c as u8);

        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '[' => self.add_token(TokenKind::LeftBracket),
            ']' => self.add_token(TokenKind::RightBracket),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            '<' => self.less_or_less_equal(),
            '>' => self.greater_or_greater_equal(),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.dot(),
            '-' => self.minus_or_arrow(),
            '+' => self.plus_or_plus_assign(),
            '*' => self.star_or_star_assign(),
            '/' => self.slash_or_comment()?,
            '%' => self.percent_or_percent_assign(),
            '=' => self.equal_or_double_arrow(),
            '!' => self.not_or_not_equal(),
            '&' => self.and(),
            '|' => self.or(),
            '?' => self.add_token(TokenKind::Question),
            ':' => self.colon_or_double_colon(),
            ';' => self.add_token(TokenKind::Semicolon),
            '@' => self.add_token(TokenKind::At),
            '#' => self.add_token(TokenKind::Hash),
            '$' => self.add_token(TokenKind::Dollar),
            '`' => self.add_token(TokenKind::Backtick),
            '"' => self.string()?,
            '\'' => self.raw_string()?,
            '0'..='9' => self.number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.identifier()?,
            ' ' | '\r' | '\t' => {}, // ignore whitespace
            '\n' => self.newline(),
            _ => return Err(LexError::UnexpectedCharacter(c, self.current_pos())),
        }
        Ok(())
    }

    /// Maneja el token punto y secuencias de puntos
    fn dot(&mut self) {
        if self.match_char('.') {
            if self.match_char('.') {
                self.add_token(TokenKind::TripleDot);
            } else {
                self.add_token(TokenKind::DoubleDot);
            }
        } else {
            self.add_token(TokenKind::Dot);
        }
    }

    /// Maneja - y ->
    fn minus_or_arrow(&mut self) {
        if self.match_char('>') {
            self.add_token(TokenKind::Arrow);
        } else if self.match_char('=') {
            self.add_token(TokenKind::MinusAssign);
        } else {
            self.add_token(TokenKind::Minus);
        }
    }

    /// Maneja + y +=
    fn plus_or_plus_assign(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenKind::PlusAssign);
        } else {
            self.add_token(TokenKind::Plus);
        }
    }

    /// Maneja * y *=
    fn star_or_star_assign(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenKind::StarAssign);
        } else {
            self.add_token(TokenKind::Star);
        }
    }

    /// Maneja /, /= y comentarios
    fn slash_or_comment(&mut self) -> Result<(), LexError> {
        if self.match_char('/') {
            // Single-line comment
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
        } else if self.match_char('*') {
            // Multi-line comment
            self.multi_line_comment()?;
        } else if self.match_char('=') {
            self.add_token(TokenKind::SlashAssign);
        } else {
            self.add_token(TokenKind::Slash);
        }
        Ok(())
    }

    /// Maneja comentarios multi-línea
    fn multi_line_comment(&mut self) -> Result<(), LexError> {
        println!("🔤 Starting multi-line comment processing");
        let mut iterations = 0;
        while !self.is_at_end() {
            iterations += 1;
            if iterations > 1000 {
                println!("🚨 Multi-line comment processing too many iterations: {}", iterations);
                return Err(LexError::UnexpectedCharacter('*', self.current_pos()));
            }

            let current_char = self.peek();
            let next_char = self.peek_next();
            println!("🔤 Multi-line comment iteration {}: current='{}' ({}), next='{}' ({})", iterations, current_char, current_char as u8, next_char, next_char as u8);

            if current_char == '*' && next_char == '/' {
                // Found end of comment
                println!("🔤 Found end of multi-line comment");
                self.advance(); // consume *
                self.advance(); // consume /
                println!("🔤 Multi-line comment ended successfully");
                return Ok(());
            }
            if current_char == '\n' {
                self.newline();
                self.advance(); // Consume the newline character
            } else {
                self.advance();
            }
        }

        println!("🚨 Multi-line comment never closed, reached end of file");
        // If we reach here, comment was never closed
        Err(LexError::UnexpectedCharacter('*', self.current_pos()))
    }

    /// Maneja % y %=
    fn percent_or_percent_assign(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenKind::PercentAssign);
        } else {
            self.add_token(TokenKind::Percent);
        }
    }

    /// Maneja = y =>
    fn equal_or_double_arrow(&mut self) {
        if self.match_char('>') {
            self.add_token(TokenKind::DoubleArrow);
        } else if self.match_char('=') {
            self.add_token(TokenKind::Equal);
        } else {
            self.add_token(TokenKind::Assign);
        }
    }

    /// Maneja ! y !=
    fn not_or_not_equal(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenKind::NotEqual);
        } else {
            self.add_token(TokenKind::Not);
        }
    }

    /// Maneja && (and lógico)
    fn and(&mut self) {
        if self.match_char('&') {
            self.add_token(TokenKind::And);
        } else {
            // Single & is not valid in Vela
            self.errors.push(LexError::UnexpectedCharacter('&', self.current_pos()));
        }
    }

    /// Maneja || (or lógico)
    fn or(&mut self) {
        if self.match_char('|') {
            self.add_token(TokenKind::Or);
        } else {
            // Single | is not valid in Vela
            self.errors.push(LexError::UnexpectedCharacter('|', self.current_pos()));
        }
    }

    /// Maneja : y ::
    fn colon_or_double_colon(&mut self) {
        if self.match_char(':') {
            self.add_token(TokenKind::DoubleColon);
        } else {
            self.add_token(TokenKind::Colon);
        }
    }

    /// Maneja < y <=
    fn less_or_less_equal(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenKind::LessEqual);
        } else {
            self.add_token(TokenKind::Less);
        }
    }

    /// Maneja > y >=
    fn greater_or_greater_equal(&mut self) {
        if self.match_char('=') {
            self.add_token(TokenKind::GreaterEqual);
        } else {
            self.add_token(TokenKind::Greater);
        }
    }

    /// Maneja strings con interpolación
    fn string(&mut self) -> Result<(), LexError> {
        let mut result = String::new();

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                return Err(LexError::UnterminatedString(self.current_pos()));
            }
            if self.peek() == '$' && self.peek_next() == '{' {
                // Interpolation start
                self.advance(); // consume $
                self.advance(); // consume {
                result.push_str("${");
                // For now, just consume until }
                while self.peek() != '}' && !self.is_at_end() {
                    result.push(self.advance());
                }
                if self.is_at_end() {
                    return Err(LexError::UnterminatedString(self.current_pos()));
                }
                result.push(self.advance()); // consume }
            } else if self.peek() == '\\' {
                // Escape sequence
                self.advance(); // consume \
                match self.peek() {
                    'n' => { result.push('\n'); self.advance(); }
                    't' => { result.push('\t'); self.advance(); }
                    'r' => { result.push('\r'); self.advance(); }
                    '"' => { result.push('"'); self.advance(); }
                    '\\' => { result.push('\\'); self.advance(); }
                    '$' => { result.push('$'); self.advance(); }
                    _ => {
                        let seq = format!("\\{}", self.peek());
                        return Err(LexError::InvalidEscapeSequence(seq, self.current_pos()));
                    }
                }
            } else {
                result.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(LexError::UnterminatedString(self.current_pos()));
        }

        self.advance(); // closing quote
        self.add_token(TokenKind::StringLiteral(result));
        Ok(())
    }

    /// Maneja strings raw (sin interpolación)
    fn raw_string(&mut self) -> Result<(), LexError> {
        let mut result = String::new();

        while self.peek() != '\'' && !self.is_at_end() {
            if self.peek() == '\n' {
                return Err(LexError::UnterminatedString(self.current_pos()));
            }
            if self.peek() == '\\' {
                // In raw strings, only escape ' and \
                self.advance(); // consume \
                match self.peek() {
                    '\'' => { result.push('\''); self.advance(); }
                    '\\' => { result.push('\\'); self.advance(); }
                    _ => result.push(self.advance()), // Keep other escapes as-is
                }
            } else {
                result.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(LexError::UnterminatedString(self.current_pos()));
        }

        self.advance(); // closing quote
        self.add_token(TokenKind::StringLiteral(result));
        Ok(())
    }

    /// Maneja números (enteros y flotantes)
    fn number(&mut self) -> Result<(), LexError> {
        let mut has_dot = false;

        // Integer part
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            has_dot = true;
            self.advance(); // consume '.'
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        // Check for invalid number formats
        let num_str = &self.source[self.start..self.current];
        if has_dot && !num_str.contains('.') {
            return Err(LexError::InvalidNumberLiteral(num_str.to_string(), self.current_pos()));
        }

        self.add_token(TokenKind::NumberLiteral(num_str.to_string()));
        Ok(())
    }

    /// Maneja identificadores y keywords
    fn identifier(&mut self) -> Result<(), LexError> {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let kind = match text {
            // Keywords
            "fn" => TokenKind::Fn,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "state" => TokenKind::State,
            "const" => TokenKind::Const,
            "dispatch" => TokenKind::Dispatch,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "interface" => TokenKind::Interface,
            "impl" => TokenKind::Impl,
            "type" => TokenKind::Type,
            "import" => TokenKind::Import,
            "export" => TokenKind::Export,
            "public" => TokenKind::Public,
            "private" => TokenKind::Private,
            "package" => TokenKind::Package,
            "module" => TokenKind::Module,
            "extension" => TokenKind::Extension,
            "library" => TokenKind::Library,
            // Boolean literals
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            // Identifier
            _ => TokenKind::Identifier(text.to_string()),
        };

        self.add_token(kind);
        Ok(())
    }

    /// Agrega un token a la lista
    fn add_token(&mut self, kind: TokenKind) {
        // Bounds checking to prevent overflow from error recovery
        if self.current < self.start {
            println!("⚠️  Token range error: current ({}) < start ({}), skipping token", self.current, self.start);
            return;
        }

        let lexeme = self.source[self.start..self.current].to_string();
        let token_length = self.current - self.start;

        let range = crate::ast::Range {
            start: crate::ast::Position {
                line: self.line,
                column: if self.column >= token_length { self.column - token_length } else { 0 },
            },
            end: crate::ast::Position {
                line: self.line,
                column: self.column,
            },
        };
        self.tokens.push(Token::new(kind, lexeme, range));
    }

    /// Avanza al siguiente carácter
    fn advance(&mut self) -> char {
        if self.is_at_end() {
            '\0' // Return null character if at end
        } else {
            let c = self.peek();
            self.current += c.len_utf8();
            self.column += 1;
            c
        }
    }

    /// Verifica si el siguiente carácter coincide
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    /// Mira el siguiente carácter sin consumirlo
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current..].chars().next().unwrap_or('\0')
        }
    }

    /// Mira dos caracteres adelante
    fn peek_next(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            let mut chars = self.source[self.current..].chars();
            chars.next(); // skip current
            chars.next().unwrap_or('\0')
        }
    }

    /// Verifica si hemos llegado al final del código fuente
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Maneja nueva línea
    fn newline(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    /// Obtiene la posición actual
    fn current_pos(&self) -> crate::ast::Position {
        crate::ast::Position {
            line: self.line,
            column: self.column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn lex(source: &str) -> Result<Vec<Token>, Vec<LexError>> {
        let mut lexer = Lexer::new(source, &PathBuf::from("test.vela"));
        match lexer.tokenize() {
            Ok(result) => {
                if result.errors.is_empty() {
                    Ok(result.tokens)
                } else {
                    Err(result.errors)
                }
            }
            Err(_) => panic!("Unexpected compilation error in lexer test"),
        }
    }

    #[test]
    fn test_keywords() {
        let tokens = lex("fn async return if else match state const dispatch").unwrap();
        assert_eq!(tokens.len(), 10); // 9 keywords + EOF
        assert!(matches!(tokens[0].kind, TokenKind::Fn));
        assert!(matches!(tokens[1].kind, TokenKind::Async));
        assert!(matches!(tokens[2].kind, TokenKind::Return));
        assert!(matches!(tokens[3].kind, TokenKind::If));
        assert!(matches!(tokens[4].kind, TokenKind::Else));
        assert!(matches!(tokens[5].kind, TokenKind::Match));
        assert!(matches!(tokens[6].kind, TokenKind::State));
        assert!(matches!(tokens[7].kind, TokenKind::Const));
        assert!(matches!(tokens[8].kind, TokenKind::Dispatch));
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex("variable_name _private camelCase").unwrap();
        assert_eq!(tokens.len(), 4); // 3 identifiers + EOF
        match &tokens[0].kind {
            TokenKind::Identifier(name) => assert_eq!(name, "variable_name"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_string_literals() {
        let tokens = lex(r#""hello world""#).unwrap();
        assert_eq!(tokens.len(), 2); // string + EOF
        match &tokens[0].kind {
            TokenKind::StringLiteral(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_number_literals() {
        let tokens = lex("42 3.14").unwrap();
        assert_eq!(tokens.len(), 3); // 2 numbers + EOF
        match &tokens[0].kind {
            TokenKind::NumberLiteral(n) => assert_eq!(n, "42"),
            _ => panic!("Expected number literal"),
        }
        match &tokens[1].kind {
            TokenKind::NumberLiteral(n) => assert_eq!(n, "3.14"),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_boolean_literals() {
        let tokens = lex("true false").unwrap();
        assert_eq!(tokens.len(), 3); // 2 booleans + EOF
        assert!(matches!(tokens[0].kind, TokenKind::BoolLiteral(true)));
        assert!(matches!(tokens[1].kind, TokenKind::BoolLiteral(false)));
    }

    #[test]
    fn test_operators() {
        let tokens = lex("+ - * / % == != < <= > >=").unwrap();
        assert_eq!(tokens.len(), 12); // 11 operators + EOF
        assert!(matches!(tokens[0].kind, TokenKind::Plus));
        assert!(matches!(tokens[1].kind, TokenKind::Minus));
        assert!(matches!(tokens[2].kind, TokenKind::Star));
        assert!(matches!(tokens[3].kind, TokenKind::Slash));
        assert!(matches!(tokens[4].kind, TokenKind::Percent));
        assert!(matches!(tokens[5].kind, TokenKind::Equal));
        assert!(matches!(tokens[6].kind, TokenKind::NotEqual));
        assert!(matches!(tokens[7].kind, TokenKind::Less));
        assert!(matches!(tokens[8].kind, TokenKind::LessEqual));
        assert!(matches!(tokens[9].kind, TokenKind::Greater));
        assert!(matches!(tokens[10].kind, TokenKind::GreaterEqual));
    }

    #[test]
    fn test_delimiters() {
        let tokens = lex("() [] {} , ; : :: . .. ...").unwrap();
        assert_eq!(tokens.len(), 14); // 13 delimiters + EOF
        assert!(matches!(tokens[0].kind, TokenKind::LeftParen));
        assert!(matches!(tokens[1].kind, TokenKind::RightParen));
        assert!(matches!(tokens[2].kind, TokenKind::LeftBracket));
        assert!(matches!(tokens[3].kind, TokenKind::RightBracket));
        assert!(matches!(tokens[4].kind, TokenKind::LeftBrace));
        assert!(matches!(tokens[5].kind, TokenKind::RightBrace));
        assert!(matches!(tokens[6].kind, TokenKind::Comma));
        assert!(matches!(tokens[7].kind, TokenKind::Semicolon));
        assert!(matches!(tokens[8].kind, TokenKind::Colon));
        assert!(matches!(tokens[9].kind, TokenKind::DoubleColon));
        assert!(matches!(tokens[10].kind, TokenKind::Dot));
        assert!(matches!(tokens[11].kind, TokenKind::DoubleDot));
        assert!(matches!(tokens[12].kind, TokenKind::TripleDot));
    }

    #[test]
    fn test_comments() {
        let tokens = lex("a // comment\nb /* multi\nline */ c").unwrap();
        assert_eq!(tokens.len(), 4); // 3 identifiers + EOF
        match &tokens[0].kind {
            TokenKind::Identifier(name) => assert_eq!(name, "a"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_unterminated_string() {
        let result = lex(r#""unterminated"#);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err()[0], LexError::UnterminatedString(_)));
    }
}