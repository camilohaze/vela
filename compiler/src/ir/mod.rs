/*
Intermediate Representation (IR) para Vela Compiler

Implementación de: TASK-070 (Bytecode generator desde IR)
Historia: VELA-070
Fecha: 2025-01-30

La IR es una representación intermedia entre AST y bytecode que facilita
optimizaciones y generación de código para múltiples backends.
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::ast::Position;

/// Identificador único para labels en control flow
pub type Label = String;

/// Tipos de datos en IR (simplificados del AST)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IRType {
    Void,
    Bool,
    Int,
    Float,
    String,
    Array(Box<IRType>),
    Object(String), // Nombre del tipo custom
}

/// Expresiones en IR (simplificadas)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IRExpr {
    /// Variable local
    Var(String),
    /// Constante
    Const(Value),
    /// Operación binaria
    BinaryOp(BinaryOp, Box<IRExpr>, Box<IRExpr>),
    /// Operación unaria
    UnaryOp(UnaryOp, Box<IRExpr>),
    /// Llamada a función
    Call { function: String, args: Vec<IRExpr> },
    /// Acceso a array
    ArrayAccess { array: Box<IRExpr>, index: Box<IRExpr> },
    /// Acceso a propiedad
    PropertyAccess { object: Box<IRExpr>, property: String },
}

/// Valores constantes en IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Null,
}

/// Operadores binarios
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

/// Operadores unarios
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg, Not,
}

/// Instrucciones de la IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IRInstruction {
    /// Declarar variable local
    DeclareVar { name: String, ty: IRType },

    /// Asignar a variable
    AssignVar { name: String, value: IRExpr },

    /// Cargar constante en stack
    LoadConst(Value),

    /// Cargar variable en stack
    LoadVar(String),

    /// Almacenar en variable (pop del stack)
    StoreVar(String),

    /// Operación binaria (pop 2, push 1)
    BinaryOp(BinaryOp),

    /// Operación unaria (pop 1, push 1)
    UnaryOp(UnaryOp),

    /// Llamada a función
    Call { function: String, arg_count: usize },

    /// Retorno de función
    Return,

    /// Salto incondicional
    Jump(Label),

    /// Salto condicional (pop condition)
    JumpIf(Label),

    /// Definir label
    Label(Label),

    /// Crear array
    CreateArray { element_type: IRType, size: usize },

    /// Acceder a elemento de array (pop index, pop array, push element)
    ArrayAccess,

    /// Almacenar en array (pop value, pop index, pop array)
    ArrayStore,

    /// Crear objeto
    CreateObject(String), // type name

    /// Acceder a propiedad (pop object, push property)
    PropertyAccess(String), // property name

    /// Almacenar propiedad (pop value, pop object)
    PropertyStore(String), // property name
}

/// Función en IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<IRParam>,
    pub return_type: IRType,
    pub locals: Vec<IRLocal>,
    pub body: Vec<IRInstruction>,
    pub position: Option<Position>,
    pub is_async: bool,
}

/// Parámetro de función
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRParam {
    pub name: String,
    pub ty: IRType,
}

/// Variable local
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRLocal {
    pub name: String,
    pub ty: IRType,
}

/// Variable global
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRGlobal {
    pub name: String,
    pub ty: IRType,
    pub initial_value: Option<Value>,
}

/// Módulo completo en IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRModule {
    pub name: String,
    pub functions: Vec<IRFunction>,
    pub globals: Vec<IRGlobal>,
    pub position: Option<Position>,
}

impl IRModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: Vec::new(),
            globals: Vec::new(),
            position: None,
        }
    }

    pub fn add_function(&mut self, function: IRFunction) {
        self.functions.push(function);
    }

    pub fn add_global(&mut self, global: IRGlobal) {
        self.globals.push(global);
    }
}

impl IRFunction {
    pub fn new(name: String, return_type: IRType) -> Self {
        Self {
            name,
            params: Vec::new(),
            return_type,
            locals: Vec::new(),
            body: Vec::new(),
            position: None,
            is_async: false,
        }
    }

    pub fn add_param(&mut self, name: String, ty: IRType) {
        self.params.push(IRParam { name, ty });
    }

    pub fn add_local(&mut self, name: String, ty: IRType) {
        self.locals.push(IRLocal { name, ty });
    }

    pub fn add_instruction(&mut self, instruction: IRInstruction) {
        self.body.push(instruction);
    }

    pub fn set_async(&mut self, is_async: bool) {
        self.is_async = is_async;
    }
}