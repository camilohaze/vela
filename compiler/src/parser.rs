/*
Módulo parser - Analizador sintáctico básico

Este módulo se implementará completamente en TASK-RUST-104 y TASK-RUST-105
*/

use crate::ast::Program;
use crate::error::CompileResult;

pub struct Parser {
    // Placeholder para implementación futura
}

impl Parser {
    pub fn new(_tokens: Vec<crate::lexer::Token>, _source_path: &std::path::Path) -> Self {
        Parser {}
    }

    pub fn parse(&mut self) -> CompileResult<Program> {
        // Placeholder - será implementado en TASK-RUST-104/TASK-RUST-105
        Ok(Program::new(crate::ast::create_range(1, 1, 1, 1), vec![], vec![]))
    }
}