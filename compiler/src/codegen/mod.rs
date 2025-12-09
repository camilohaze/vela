/*
Módulo de Code Generation para Vela Compiler

Implementación de: TASK-070 (Bytecode generator desde IR)
Fecha: 2025-01-30

Este módulo contiene el pipeline completo de generación de código:
- ast_to_ir.rs: Convertidor AST → IR
- ir_to_bytecode.rs: Generador IR → Bytecode con optimizaciones
- main.rs: API principal con CodeGenerator
- mod.rs: Módulo principal con API unificada
*/

pub mod ast_to_ir;
pub mod ir_to_bytecode;
pub mod main;

pub use ast_to_ir::ASTToIRConverter;
pub use ir_to_bytecode::{IRToBytecodeGenerator, IROptimizer};
pub use main::CodeGenerator;