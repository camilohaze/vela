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
pub mod debug_info;
#[cfg(test)]
pub mod debug_info_tests;
pub mod config;
pub mod error;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod types;
pub mod codegen;
#[path = "../js_codegen/lib.rs"]
pub mod js_codegen;
pub mod json_decorators;
pub mod message_broker_decorators;
pub mod resilience_decorators;
pub mod observability_decorators;
pub mod orm_decorators;
pub mod serialization_decorators;
pub mod serialization_tests;
pub mod config_loader;
pub mod config_tests;
pub mod config_decorators;
pub mod config_decorator_tests;
pub mod hot_reload;
pub mod hot_reload_tests;
pub mod config_integration_tests;
pub mod gateway;
pub mod gateway_decorators;
pub mod gateway_decorator_tests;
pub mod grpc_decorators;
pub mod grpc_decorator_tests;
pub mod lambda_patterns_test;
pub mod advanced_pattern_matching_tests;
pub mod router;
pub mod load_balancer;
pub mod rate_limiter;
pub mod auth;
pub mod plugins;
pub mod metrics;
pub mod gateway_tests;
pub mod dynamic_router;
pub mod service_discovery;

use std::path::Path;
use config::Config;
use error::{CompileError, CompileResult, Diagnostics, CodegenError};
use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::CodeGenerator;
use vela_vm;

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
        // Convertir BytecodeProgram a Bytecode de la VM
        let vm_bytecode = self.convert_to_vm_bytecode(bytecode)?;
        let result = vm_bytecode.to_bytes().map_err(|e| CompileError::Io {
            path: std::path::PathBuf::from("bytecode"),
            error: format!("Failed to serialize bytecode: {}", e),
        })?;
        println!("✅ Bytecode serialization complete");

        Ok(result)
    }

    /// Convertir BytecodeProgram del compiler a Bytecode de la VM
    fn convert_to_vm_bytecode(&self, program: crate::bytecode::BytecodeProgram) -> CompileResult<vela_vm::bytecode::Bytecode> {
        use vela_vm::bytecode::{Bytecode, Instruction, Constant, CodeObject};
        
        let mut bytecode = Bytecode::new();
        
        // Convertir constantes
        for value in program.constants {
            let constant = match value {
                crate::bytecode::Value::Null => Constant::Null,
                crate::bytecode::Value::Bool(b) => Constant::Bool(b),
                crate::bytecode::Value::Int(i) => Constant::Int(i),
                crate::bytecode::Value::Float(f) => Constant::Float(f),
                crate::bytecode::Value::String(s) => {
                    let idx = bytecode.strings.len() as u16;
                    bytecode.strings.push(s);
                    Constant::String(idx)
                }
                _ => return Err(CompileError::Codegen(CodegenError {
                    message: "Unsupported constant type".to_string(),
                    location: None,
                })),
            };
            bytecode.constants.push(constant);
        }
        
        // Convertir funciones a code objects
        for function in program.functions {
            // Convertir bytecode del compiler al formato VM
            let mut vm_bytecode = Vec::new();
            let mut i = 0;
            while i < function.code.len() {
                let opcode = function.code[i];
                match opcode {
                    0x10 => { // Compiler LoadConst
                        if i + 2 < function.code.len() {
                            let const_index = ((function.code[i + 1] as u16) << 8) | (function.code[i + 2] as u16);
                            // VM LoadConst(u16)
                            vm_bytecode.push(0x00); // LoadConst opcode
                            vm_bytecode.push((const_index >> 8) as u8);
                            vm_bytecode.push(const_index as u8);
                            i += 3;
                        } else {
                            i += 1;
                        }
                    }
                    0x63 => { // Compiler Return
                        // VM Return
                        vm_bytecode.push(0x51);
                        i += 1;
                    }
                    _ => {
                        // Copiar opcode desconocido (aunque esto probablemente cause problemas)
                        vm_bytecode.push(opcode);
                        i += 1;
                    }
                }
            }
            
            let mut code_obj = CodeObject {
                name: 0, // TODO: agregar nombres de función
                filename: 0, // TODO: agregar filename
                arg_count: function.params_count as u16,
                local_count: function.locals_count as u16,
                stack_size: 256, // TODO: calcular stack size real
                flags: 0, // TODO: agregar flags apropiados
                bytecode: vm_bytecode,
                constants: Vec::new(), // TODO: constantes locales por función
                names: Vec::new(), // TODO: nombres de variables
                line_numbers: Vec::new(), // TODO: información de línea
            };
            
            bytecode.code_objects.push(code_obj);
        }
        
        // Incluir debug info en metadata si existe
        if let Some(debug_info) = program.debug_info {
            let debug_bytes = bincode::serialize(&debug_info)
                .map_err(|e| CompileError::Codegen(CodegenError {
                    message: format!("Failed to serialize debug info: {}", e),
                    location: None,
                }))?;
            bytecode.metadata.insert("debug_info".to_string(), debug_bytes);
        }
        
        Ok(bytecode)
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

    #[test]
    fn test_or_pattern_simple_literals() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match value {
                    1 | 2 | 3 => "small"
                    _ => "other"
                }
            }
        "#;

        let result = compiler.compile_string(source, "or_pattern_test.vela");
        assert!(result.is_ok(), "Or pattern compilation should succeed");
    }

    #[test]
    fn test_or_pattern_with_identifiers() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match result {
                    Ok(value) | Err(error) => "handled"
                    _ => "unhandled"
                }
            }
        "#;

        let result = compiler.compile_string(source, "or_pattern_identifiers.vela");
        assert!(result.is_ok(), "Or pattern with identifiers should succeed");
    }

    #[test]
    fn test_or_pattern_mixed_types() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match data {
                    [] | [x] => "array variations"
                    "empty" | "full" => "string variations"
                    _ => "other"
                }
            }
        "#;

        let result = compiler.compile_string(source, "or_pattern_mixed.vela");
        assert!(result.is_ok(), "Or pattern with mixed types should succeed");
    }

    #[test]
    fn test_or_pattern_with_guards() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match number {
                    1 | 2 | 3 if number > 0 => "positive small"
                    4 | 5 | 6 if number < 10 => "medium"
                    _ => "other"
                }
            }
        "#;

        let result = compiler.compile_string(source, "or_pattern_guards.vela");
        assert!(result.is_ok(), "Or pattern with guards should succeed");
    }

    #[test]
    fn test_or_pattern_complex_combinations() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match event {
                    Click(x, y) | Touch(x, y) | MouseMove(x, y) => "pointer event"
                    KeyPress(key) | KeyRelease(key) => "keyboard event"
                    Resize(w, h) | FocusChange(hasFocus) => "window event"
                    _ => "unknown"
                }
            }
        "#;

        let result = compiler.compile_string(source, "or_pattern_complex.vela");
        assert!(result.is_ok(), "Complex or pattern combinations should succeed");
    }

    #[test]
    fn test_or_pattern_nested() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match complex {
                    (1 | 2) | (3 | 4) => "nested or"
                    _ => "other"
                }
            }
        "#;

        let result = compiler.compile_string(source, "or_pattern_nested.vela");
        assert!(result.is_ok(), "Nested or patterns should succeed");
    }

    #[test]
    fn test_or_pattern_precedence() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match data {
                    [x | y] => "array with or"
                    (a | b, c | d) => "tuple with or"
                    _ => "other"
                }
            }
        "#;

        let result = compiler.compile_string(source, "or_pattern_precedence.vela");
        assert!(result.is_ok(), "Or pattern precedence should work correctly");
    }

    #[test]
    fn test_range_pattern_exclusive() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match value {
                    1..10 => "small"
                    11..100 => "medium"
                    _ => "large"
                }
            }
        "#;

        let result = compiler.compile_string(source, "range_pattern_exclusive.vela");
        assert!(result.is_ok(), "Range pattern exclusive compilation should succeed");
    }

    #[test]
    fn test_range_pattern_inclusive() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match value {
                    1..=10 => "small"
                    11..=100 => "medium"
                    _ => "large"
                }
            }
        "#;

        let result = compiler.compile_string(source, "range_pattern_inclusive.vela");
        assert!(result.is_ok(), "Range pattern inclusive compilation should succeed");
    }

    #[test]
    fn test_range_pattern_mixed() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match value {
                    0..10 => "single digit"
                    10..=99 => "double digit"
                    100..=999 => "triple digit"
                    _ => "other"
                }
            }
        "#;

        let result = compiler.compile_string(source, "range_pattern_mixed.vela");
        assert!(result.is_ok(), "Mixed range patterns should compile correctly");
    }

    #[test]
    fn test_range_pattern_with_guards() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match value {
                    x if x > 0 && x < 10 => "positive single"
                    10..=99 if value % 2 == 0 => "even double"
                    100..=999 => "triple"
                    _ => "other"
                }
            }
        "#;

        let result = compiler.compile_string(source, "range_pattern_guards.vela");
        assert!(result.is_ok(), "Range patterns with guards should compile correctly");
    }

    #[test]
    fn test_range_pattern_complex() {
        let mut compiler = Compiler::default();
        let source = r#"
            fn test() {
                match value {
                    1..=9 | 11..=99 => "valid range"
                    10 | 100..=999 => "invalid range"
                    _ => "other"
                }
            }
        "#;

        let result = compiler.compile_string(source, "range_pattern_complex.vela");
        assert!(result.is_ok(), "Complex range patterns with or combinations should compile");
    }
}

