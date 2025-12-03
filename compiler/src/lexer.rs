/*
Módulo lexer - Analizador léxico básico

Este módulo se implementará completamente en TASK-RUST-103
*/

pub struct Lexer {
    // Placeholder para implementación futura
}

impl Lexer {
    pub fn new(_source: &str, _source_path: &std::path::Path) -> Self {
        Lexer {}
    }

    pub fn tokenize(&mut self) -> crate::error::CompileResult<Vec<Token>> {
        // Placeholder - será implementado en TASK-RUST-103
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: String,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: String, value: String, line: usize, column: usize) -> Self {
        Token { kind, value, line, column }
    }
}