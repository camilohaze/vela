/*
Módulo de Code Generation para Vela Compiler

Implementación de: TASK-070 (Bytecode generator desde IR), TASK-118 (WASM generator), TASK-119 (JS-WASM glue), TASK-124 (Linking pipeline)
Fecha: 2025-01-30, 2025-12-14

Este módulo contiene el pipeline completo de generación de código:
- ast_to_ir.rs: Convertidor AST → IR
- ir_to_bytecode.rs: Generador IR → Bytecode con optimizaciones
- ir_to_wasm.rs: Generador IR → WebAssembly para web performance
- js_wasm_glue.rs: Generador de código glue JavaScript para WASM interop
- ir_to_llvm.rs: Generador LLVM IR para código nativo
- linking.rs: Pipeline de linking para ejecutables nativos
- main.rs: API principal con CodeGenerator
- mod.rs: Módulo principal con API unificada
*/

pub mod ast_to_ir;
pub mod ir_to_bytecode;
pub mod ir_to_wasm;
pub mod js_wasm_glue;
#[cfg(feature = "llvm_backend")]
pub mod ir_to_llvm;
#[cfg(feature = "llvm_backend")]
pub mod linking;
pub mod main;

pub use ast_to_ir::ASTToIRConverter;
pub use ir_to_bytecode::{IRToBytecodeGenerator, IROptimizer};
pub use ir_to_wasm::{WasmGenerator, WasmError};
pub use js_wasm_glue::{JSGlueGenerator, TypeScriptGenerator};
#[cfg(feature = "llvm_backend")]
pub use ir_to_llvm::LLVMGenerator;
#[cfg(feature = "llvm_backend")]
pub use linking::LinkingPipeline;
pub use main::CodeGenerator;