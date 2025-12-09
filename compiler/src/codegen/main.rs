/*
Code Generator para el lenguaje Vela - API Principal

Implementación de: TASK-070 (Bytecode generator desde IR)
Fecha: 2025-01-30

Este archivo contiene la API principal del módulo de code generation.
*/

use crate::ast::*;
use crate::error::{CompileError, CompileResult};
use crate::codegen::ast_to_ir::ASTToIRConverter;
use crate::codegen::ir_to_bytecode::IRToBytecodeGenerator;
use crate::codegen::ir_to_bytecode::IROptimizer;

/// Generador de código principal con pipeline AST → IR → Bytecode
pub struct CodeGenerator {
    /// Convertidor AST → IR
    ast_converter: ASTToIRConverter,
    /// Generador IR → Bytecode
    bytecode_generator: IRToBytecodeGenerator,
    /// Optimizador IR
    optimizer: IROptimizer,
}

impl CodeGenerator {
    /// Crear nuevo generador con pipeline completo
    pub fn new() -> Self {
        Self {
            ast_converter: ASTToIRConverter::new(),
            bytecode_generator: IRToBytecodeGenerator::new(),
            optimizer: IROptimizer::new(),
        }
    }

    /// Generar bytecode desde programa AST usando pipeline completo
    pub fn generate_program(&mut self, program: &Program) -> CompileResult<crate::bytecode::BytecodeProgram> {
        // Fase 1: AST → IR
        let mut ir_module = self.ast_converter.convert_program(program)?;

        // Fase 2: Optimizaciones IR
        self.optimizer.optimize_module(&mut ir_module);

        // Fase 3: IR → Bytecode
        let bytecode_program = self.bytecode_generator.generate_module(&ir_module)?;

        Ok(bytecode_program)
    }

    /// Generar bytecode desde programa AST (sin optimizaciones)
    pub fn generate_program_no_opt(&mut self, program: &Program) -> CompileResult<crate::bytecode::BytecodeProgram> {
        // Fase 1: AST → IR
        let ir_module = self.ast_converter.convert_program(program)?;

        // Fase 2: IR → Bytecode (sin optimizaciones)
        let bytecode_program = self.bytecode_generator.generate_module(&ir_module)?;

        Ok(bytecode_program)
    }

    /// Generar solo IR desde programa AST (para debugging/optimización)
    pub fn generate_ir(&mut self, program: &Program) -> CompileResult<crate::ir::IRModule> {
        self.ast_converter.convert_program(program)
    }
}