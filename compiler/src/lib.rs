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
pub mod bytecode;
pub mod config;
pub mod error;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod types;
pub mod codegen;
pub mod json_decorators;

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
        println!("🚀 Starting compile_string for file: {}", file_name);

        // Phase 1: Lexical Analysis
        println!("🔤 Phase 1: Starting lexical analysis...");
        let mut lexer = Lexer::new(source, source_path);
        println!("🔤 Lexer created, calling tokenize...");
        let lex_result = lexer.tokenize()?;
        println!("🔤 Tokenize completed, {} tokens, {} errors", lex_result.tokens.len(), lex_result.errors.len());
        self.diagnostics.extend_from_lexer(&lex_result.errors);
        println!("✅ Phase 1: Lexical analysis complete");

        // Phase 2: Parsing
        println!("🔧 Phase 2: Starting parsing...");
        let mut parser = Parser::new(lex_result.tokens);
        println!("🔧 Parser created, calling parse...");
        let ast = parser.parse()?;
        println!("✅ Phase 2: Parsing complete");

        // Phase 3: Semantic Analysis
        println!("🔍 Phase 3: Starting semantic analysis...");
        let mut analyzer = SemanticAnalyzer::new();
        println!("🔍 Analyzer created, calling analyze...");
        analyzer.analyze(&ast)?;
        println!("✅ Phase 3: Semantic analysis complete");

        // Phase 4: Code Generation
        println!("⚙️ Phase 4: Starting code generation...");
        let mut codegen = CodeGenerator::new();
        println!("⚙️ Codegen created, calling generate_program...");
        let bytecode = codegen.generate_program(&ast)?;
        println!("✅ Phase 4: Code generation complete");

        println!("📦 Serializing bytecode...");
        let result = bytecode.to_bytes().map_err(|e| CompileError::Io {
            path: std::path::PathBuf::from("bytecode"),
            error: format!("Failed to serialize bytecode: {}", e),
        })?;
        println!("✅ Bytecode serialization complete");

        Ok(result)
    }

    /// Compile a source file
    pub fn compile_file<P: AsRef<Path>>(&mut self, path: P) -> CompileResult<Vec<u8>> {
        let path = path.as_ref();
        println!("📁 Reading file: {}", path.display());
        let source = std::fs::read_to_string(path)
            .map_err(|e| CompileError::Io {
                path: path.to_path_buf(),
                error: e.to_string(),
            })?;
        println!("📄 File read successfully, {} bytes", source.len());

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