/*!
# Vela Compiler

The Vela compiler transforms Vela source code into executable bytecode
through lexical analysis, parsing, semantic analysis, and code generation.

## Architecture

The compiler follows a modular pipeline architecture:

1. **Lexer**: Tokenizes source code into lexical tokens
2. **Parser**: Builds an Abstract Syntax Tree (AST) from tokens
3. **Semantic Analyzer**: Performs type checking and symbol resolution
4. **Code Generator**: Emits bytecode from the analyzed AST

## Features

- **Memory Safe**: Built with Rust for guaranteed memory safety
- **Fast Compilation**: Optimized for quick compile times
- **Rich Diagnostics**: Detailed error messages with source locations
- **Extensible**: Modular design for easy extension and maintenance
*/

pub mod ast;
pub mod config;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod codegen;

use std::path::Path;
use config::Config;
use error::{CompileError, CompileResult, Diagnostics};
use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::CodeGenerator;

/// The main Vela compiler
pub struct Compiler {
    config: Config,
    diagnostics: Diagnostics,
}

impl Compiler {
    /// Create a new compiler with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            diagnostics: Diagnostics::new(),
        }
    }

    /// Create a compiler with default configuration
    pub fn default() -> Self {
        Self::new(Config::default())
    }

    /// Compile source code from a string
    pub fn compile_string(&mut self, source: &str, file_name: &str) -> CompileResult<Vec<u8>> {
        let source_path = Path::new(file_name);

        // Phase 1: Lexical Analysis
        let mut lexer = Lexer::new(source, source_path);
        let lex_result = lexer.tokenize()?;
        self.diagnostics.extend_from_lexer(&lex_result.errors);

        // Phase 2: Parsing
        let mut parser = Parser::new(lex_result.tokens);
        let ast = parser.parse()?;

        // Phase 3: Semantic Analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)?;

        // Phase 4: Code Generation
        let mut codegen = CodeGenerator::new();
        let bytecode = codegen.generate_program(&ast)?;

        Ok(bytecode.into_bytes())
    }

    /// Compile a source file
    pub fn compile_file<P: AsRef<Path>>(&mut self, path: P) -> CompileResult<Vec<u8>> {
        let path = path.as_ref();
        let source = std::fs::read_to_string(path)
            .map_err(|e| CompileError::Io {
                path: path.to_path_buf(),
                error: e.to_string(),
            })?;

        self.compile_string(&source, &path.to_string_lossy())
    }

    /// Get compilation diagnostics
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Check if compilation was successful (no errors)
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::default();
        assert!(!compiler.has_errors());
    }

    #[test]
    fn test_empty_source_compilation() {
        let mut compiler = Compiler::default();
        // Empty source should compile to minimal bytecode
        let result = compiler.compile_string("", "empty.vela");
        assert!(result.is_ok());
    }
}