/*
Módulo de Bytecode para Vela Compiler

Implementación de: TASK-070 (Bytecode generator desde IR)
Fecha: 2025-01-30

Este módulo define las estructuras de bytecode ejecutable por la VelaVM.
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Programa bytecode completo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BytecodeProgram {
    /// Funciones del programa
    pub functions: Vec<BytecodeFunction>,
    /// Constantes del programa
    pub constants: Vec<Value>,
}

impl BytecodeProgram {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Agregar una función al programa
    pub fn add_function(&mut self, function: BytecodeFunction) {
        self.functions.push(function);
    }

    /// Serializar el programa a bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        // TODO: Implementar serialización real
        // Por ahora, devolver un error
        Err("to_bytes not implemented yet".to_string())
    }
}

/// Función en bytecode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BytecodeFunction {
    /// Nombre de la función
    pub name: String,
    /// Código bytecode
    pub code: Vec<u8>,
    /// Número de variables locales
    pub locals_count: usize,
    /// Número de parámetros
    pub params_count: usize,
}

impl BytecodeFunction {
    pub fn new(name: String, locals_count: usize, params_count: usize) -> Self {
        Self {
            name,
            code: Vec::new(),
            locals_count,
            params_count,
        }
    }
}

/// Valores constantes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// Opcodes del bytecode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    // Stack operations
    Pop = 0x00,
    Dup = 0x01,
    Swap = 0x02,

    // Constants
    LoadConst = 0x10,

    // Variables
    LoadLocal = 0x20,
    StoreLocal = 0x21,

    // Arithmetic
    Add = 0x30,
    Sub = 0x31,
    Mul = 0x32,
    Div = 0x33,
    Mod = 0x34,
    Neg = 0x35,

    // Comparison
    Equal = 0x40,
    NotEqual = 0x41,
    Less = 0x42,
    LessEqual = 0x43,
    Greater = 0x44,
    GreaterEqual = 0x45,

    // Logical
    And = 0x50,
    Or = 0x51,
    Not = 0x52,

    // Control flow
    Jump = 0x60,
    JumpIf = 0x61,
    Call = 0x62,
    Return = 0x63,

    // Objects
    NewObject = 0x70,
    LoadField = 0x71,
    StoreField = 0x72,

    // Arrays
    NewArray = 0x80,
    LoadArray = 0x81,
    StoreArray = 0x82,

    // Special
    Throw = 0xF0,
    Nop = 0xFF,
}