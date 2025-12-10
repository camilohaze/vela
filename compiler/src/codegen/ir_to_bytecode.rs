/*
Generador de Bytecode desde IR para Vela Compiler

Implementación de: TASK-070 (Bytecode generator desde IR)
Fecha: 2025-01-30

Este módulo convierte la Representación Intermedia (IR) en bytecode
ejecutable por la VelaVM. Implementa optimizaciones básicas y
genera código eficiente.
*/

use std::collections::HashMap;
use crate::ir::{IRModule, IRFunction, IRInstruction, IRType, Value as IRValue, BinaryOp, UnaryOp, Label};
use crate::bytecode::{BytecodeProgram, BytecodeFunction, Value as BytecodeValue, Opcode};
use crate::error::{CompileError, CompileResult, CodegenError};

/// Generador de bytecode desde IR
pub struct IRToBytecodeGenerator {
    /// Tabla de símbolos para variables locales
    local_symbols: HashMap<String, usize>,
    /// Contador de variables locales
    local_counter: usize,
    /// Tabla de constantes
    constants: Vec<BytecodeValue>,
    /// Tabla de funciones
    functions: HashMap<String, usize>,
    /// Contador de funciones
    function_counter: usize,
    /// Labels y sus posiciones
    labels: HashMap<Label, usize>,
    /// Labels pendientes de resolver
    pending_labels: HashMap<Label, Vec<usize>>,
}

impl IRToBytecodeGenerator {
    pub fn new() -> Self {
        Self {
            local_symbols: HashMap::new(),
            local_counter: 0,
            constants: Vec::new(),
            functions: HashMap::new(),
            function_counter: 0,
            labels: HashMap::new(),
            pending_labels: HashMap::new(),
        }
    }

    /// Generar bytecode completo desde módulo IR
    pub fn generate_module(&mut self, module: &IRModule) -> CompileResult<BytecodeProgram> {
        let mut program = BytecodeProgram::new();

        // Registrar todas las funciones primero
        for function in &module.functions {
            let function_index = self.function_counter;
            self.functions.insert(function.name.clone(), function_index);
            self.function_counter += 1;

            program.add_function(BytecodeFunction {
                name: function.name.clone(),
                code: Vec::new(), // Se llenará después
                locals_count: function.locals.len(),
                params_count: function.params.len(),
            });
        }

        // Generar código para cada función
        for (i, function) in module.functions.iter().enumerate() {
            let bytecode = self.generate_function(function)?;
            program.functions[i].code = bytecode;
        }

        // Agregar constantes
        program.constants = self.constants.clone();

        Ok(program)
    }

    /// Generar bytecode para una función IR
    fn generate_function(&mut self, function: &IRFunction) -> CompileResult<Vec<u8>> {
        self.local_symbols.clear();
        self.local_counter = 0;
        self.labels.clear();
        self.pending_labels.clear();

        // Registrar parámetros como variables locales
        for param in &function.params {
            let local_index = self.local_counter;
            self.local_symbols.insert(param.name.clone(), local_index);
            self.local_counter += 1;
        }

        // Registrar otras variables locales
        for local in &function.locals {
            let local_index = self.local_counter;
            self.local_symbols.insert(local.name.clone(), local_index);
            self.local_counter += 1;
        }

        let mut bytecode = Vec::new();

        // Primera pasada: generar bytecode y registrar labels
        for instruction in &function.body {
            match instruction {
                IRInstruction::Label(label) => {
                    let position = bytecode.len();
                    self.labels.insert(label.clone(), position);
                }
                _ => {
                    let instr_bytes = self.generate_instruction(instruction)?;
                    bytecode.extend(instr_bytes);
                }
            }
        }

        // Resolver labels pendientes
        self.resolve_labels(&mut bytecode)?;

        Ok(bytecode)
    }

    /// Generar bytecode para una instrucción IR
    fn generate_instruction(&mut self, instruction: &IRInstruction) -> CompileResult<Vec<u8>> {
        match instruction {
            IRInstruction::LoadConst(value) => {
                let bytecode_value = self.convert_ir_value_to_bytecode(value);
                let const_index = self.add_constant(bytecode_value);
                Ok(vec![Opcode::LoadConst as u8, (const_index >> 8) as u8, const_index as u8])
            }
            IRInstruction::LoadVar(name) => {
                if let Some(&local_index) = self.local_symbols.get(name) {
                    Ok(vec![Opcode::LoadLocal as u8, local_index as u8])
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: format!("Undefined variable: {}", name),
                        location: None,
                    }))
                }
            }
            IRInstruction::StoreVar(name) => {
                if let Some(&local_index) = self.local_symbols.get(name) {
                    Ok(vec![Opcode::StoreLocal as u8, local_index as u8])
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: format!("Undefined variable: {}", name),
                        location: None,
                    }))
                }
            }
            IRInstruction::DeclareVar { .. } => {
                // Las declaraciones ya se manejaron en generate_function
                Ok(Vec::new())
            }
            IRInstruction::BinaryOp(op) => {
                let opcode = match op {
                    BinaryOp::Add => Opcode::Add,
                    BinaryOp::Sub => Opcode::Sub,
                    BinaryOp::Mul => Opcode::Mul,
                    BinaryOp::Div => Opcode::Div,
                    BinaryOp::Mod => Opcode::Mod,
                    BinaryOp::Eq => Opcode::Equal,
                    BinaryOp::Ne => Opcode::NotEqual,
                    BinaryOp::Lt => Opcode::Less,
                    BinaryOp::Le => Opcode::LessEqual,
                    BinaryOp::Gt => Opcode::Greater,
                    BinaryOp::Ge => Opcode::GreaterEqual,
                    BinaryOp::And => Opcode::And,
                    BinaryOp::Or => Opcode::Or,
                };
                Ok(vec![opcode as u8])
            }
            IRInstruction::UnaryOp(op) => {
                let opcode = match op {
                    UnaryOp::Neg => Opcode::Neg,
                    UnaryOp::Not => Opcode::Not,
                };
                Ok(vec![opcode as u8])
            }
            IRInstruction::Call { function, arg_count } => {
                if let Some(&function_index) = self.functions.get(function) {
                    Ok(vec![
                        Opcode::Call as u8,
                        function_index as u8,
                        *arg_count as u8,
                    ])
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: format!("Undefined function: {}", function),
                        location: None,
                    }))
                }
            }
            IRInstruction::Return => {
                Ok(vec![Opcode::Return as u8])
            }
            IRInstruction::Jump(label) => {
                // Jump incondicional - resolver después
                self.add_pending_label(label.clone(), 0); // Placeholder
                Ok(vec![Opcode::Jump as u8, 0, 0]) // Placeholder addresses
            }
            IRInstruction::JumpIf(label) => {
                // Jump condicional - resolver después
                self.add_pending_label(label.clone(), 0); // Placeholder
                Ok(vec![Opcode::JumpIf as u8, 0, 0]) // Placeholder addresses
            }
            IRInstruction::Label(_) => {
                // Labels ya se manejaron en generate_function
                Ok(Vec::new())
            }
            IRInstruction::CreateArray { element_type: _, size } => {
                Ok(vec![Opcode::NewArray as u8, *size as u8])
            }
            IRInstruction::ArrayAccess => {
                Ok(vec![Opcode::LoadArray as u8])
            }
            IRInstruction::ArrayStore => {
                Ok(vec![Opcode::StoreArray as u8])
            }
            IRInstruction::CreateObject(class_name) => {
                let class_index = self.add_constant(BytecodeValue::String(class_name.clone()));
                Ok(vec![Opcode::NewObject as u8, (class_index >> 8) as u8, class_index as u8])
            }
            IRInstruction::PropertyAccess(prop_name) => {
                let prop_index = self.add_constant(BytecodeValue::String(prop_name.clone()));
                Ok(vec![Opcode::LoadField as u8, (prop_index >> 8) as u8, prop_index as u8])
            }
            IRInstruction::PropertyStore(prop_name) => {
                let prop_index = self.add_constant(BytecodeValue::String(prop_name.clone()));
                Ok(vec![Opcode::StoreField as u8, (prop_index >> 8) as u8, prop_index as u8])
            }
            IRInstruction::AssignVar { name, value } => {
                // Primero generar bytecode para el valor
                self.generate_instruction(value)?;
                // Buscar el índice de la variable local
                if let Some(&local_index) = self.local_symbols.get(name) {
                    Ok(vec![Opcode::StoreLocal as u8, local_index as u8])
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: format!("Undefined variable: {}", name),
                        location: None,
                    }))
                }
            }
        }
    }

    /// Agregar constante y retornar su índice
    fn add_constant(&mut self, value: BytecodeValue) -> usize {
        // Buscar si la constante ya existe
        for (i, existing) in self.constants.iter().enumerate() {
            if existing == &value {
                return i;
            }
        }
        // Si no existe, agregarla
        let index = self.constants.len();
        self.constants.push(value);
        index
    }

    /// Convertir valor IR a valor bytecode
    fn convert_ir_value_to_bytecode(&self, value: &IRValue) -> BytecodeValue {
        match value {
            IRValue::Bool(b) => BytecodeValue::Bool(*b),
            IRValue::Int(i) => BytecodeValue::Int(*i),
            IRValue::Float(f) => BytecodeValue::Float(*f),
            IRValue::String(s) => BytecodeValue::String(s.clone()),
            IRValue::Null => BytecodeValue::Null,
        }
    }

    /// Agregar label pendiente de resolver
    fn add_pending_label(&mut self, label: Label, position: usize) {
        self.pending_labels.entry(label).or_insert_with(Vec::new).push(position);
    }

    /// Resolver todas las labels pendientes
    fn resolve_labels(&self, bytecode: &mut Vec<u8>) -> CompileResult<()> {
        for (label, positions) in &self.pending_labels {
            if let Some(&target_position) = self.labels.get(label) {
                for &pos in positions {
                    // Calcular offset relativo
                    let offset = target_position as i16 - pos as i16 - 3; // -3 por el tamaño de la instrucción
                    bytecode[pos + 1] = (offset >> 8) as u8;
                    bytecode[pos + 2] = offset as u8;
                }
            } else {
                return Err(CompileError::Codegen(CodegenError {
                    message: format!("Undefined label: {}", label),
                    location: None,
                }));
            }
        }
        Ok(())
    }
}

/// Optimizaciones básicas del IR
pub struct IROptimizer;

impl IROptimizer {
    pub fn new() -> Self {
        Self
    }

    /// Aplicar optimizaciones básicas al módulo IR
    pub fn optimize_module(&self, module: &mut IRModule) {
        for function in &mut module.functions {
            self.optimize_function(function);
        }
    }

    /// Optimizar función IR
    fn optimize_function(&self, function: &mut IRFunction) {
        // Optimización básica: eliminar NOPs
        // No hay Nop en IR, así que no eliminamos nada
    // function.body.retain(|instr| !matches!(instr, IRInstruction::Nop));

        // Constant folding: simplificar expresiones constantes
        self.constant_folding(function);

        // Dead code elimination: eliminar código inalcanzable
        self.dead_code_elimination(function);
    }

    /// Constant folding: simplificar expresiones constantes
    fn constant_folding(&self, function: &mut IRFunction) {
        let mut i = 0;
        while i < function.body.len() {
            match &function.body[i] {
                IRInstruction::BinaryOp(op) => {
                    // Verificar si los dos valores anteriores son constantes
                    if i >= 2 {
                        if let (IRInstruction::LoadConst(a), IRInstruction::LoadConst(b)) =
                            (&function.body[i-2], &function.body[i-1]) {
                            if let Some(result) = self.fold_binary_op(*op, a.clone(), b.clone()) {
                                // Reemplazar las 3 instrucciones con una sola LoadConst
                                function.body.splice(i-2..=i, vec![IRInstruction::LoadConst(result)]);
                                continue; // No incrementar i porque eliminamos elementos
                            }
                        }
                    }
                }
                IRInstruction::UnaryOp(op) => {
                    // Verificar si el valor anterior es constante
                    if i >= 1 {
                        if let IRInstruction::LoadConst(val) = &function.body[i-1] {
                            if let Some(result) = self.fold_unary_op(*op, val.clone()) {
                                // Reemplazar las 2 instrucciones con una sola LoadConst
                                function.body.splice(i-1..=i, vec![IRInstruction::LoadConst(result)]);
                                continue;
                            }
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Aplicar operación binaria a constantes
    fn fold_binary_op(&self, op: BinaryOp, a: Value, b: Value) -> Option<Value> {
        match (op, a, b) {
            (BinaryOp::Add, Value::Int(x), Value::Int(y)) => Some(Value::Int(x + y)),
            (BinaryOp::Sub, Value::Int(x), Value::Int(y)) => Some(Value::Int(x - y)),
            (BinaryOp::Mul, Value::Int(x), Value::Int(y)) => Some(Value::Int(x * y)),
            (BinaryOp::Div, Value::Int(x), Value::Int(y)) if y != 0 => Some(Value::Int(x / y)),
            (BinaryOp::Add, Value::Float(x), Value::Float(y)) => Some(Value::Float(x + y)),
            (BinaryOp::Sub, Value::Float(x), Value::Float(y)) => Some(Value::Float(x - y)),
            (BinaryOp::Mul, Value::Float(x), Value::Float(y)) => Some(Value::Float(x * y)),
            (BinaryOp::Div, Value::Float(x), Value::Float(y)) if y != 0.0 => Some(Value::Float(x / y)),
            (BinaryOp::Eq, Value::Int(x), Value::Int(y)) => Some(Value::Bool(x == y)),
            (BinaryOp::Ne, Value::Int(x), Value::Int(y)) => Some(Value::Bool(x != y)),
            (BinaryOp::Lt, Value::Int(x), Value::Int(y)) => Some(Value::Bool(x < y)),
            (BinaryOp::Le, Value::Int(x), Value::Int(y)) => Some(Value::Bool(x <= y)),
            (BinaryOp::Gt, Value::Int(x), Value::Int(y)) => Some(Value::Bool(x > y)),
            (BinaryOp::Ge, Value::Int(x), Value::Int(y)) => Some(Value::Bool(x >= y)),
            _ => None,
        }
    }

    /// Aplicar operación unaria a constante
    fn fold_unary_op(&self, op: UnaryOp, val: Value) -> Option<Value> {
        match (op, val) {
            (UnaryOp::Neg, Value::Int(x)) => Some(Value::Int(-x)),
            (UnaryOp::Neg, Value::Float(x)) => Some(Value::Float(-x)),
            (UnaryOp::Not, Value::Bool(x)) => Some(Value::Bool(!x)),
            _ => None,
        }
    }

    /// Dead code elimination: eliminar código inalcanzable después de Return
    fn dead_code_elimination(&self, function: &mut IRFunction) {
        for i in 0..function.body.len() {
            if matches!(function.body[i], IRInstruction::Return) {
                // Eliminar todo después del Return
                function.body.truncate(i + 1);
                break;
            }
        }
    }

    /// Convertir valor IR a valor bytecode
    fn convert_ir_value_to_bytecode(&self, value: &crate::ir::Value) -> BytecodeValue {
        match value {
            crate::ir::Value::Bool(b) => BytecodeValue::Bool(*b),
            crate::ir::Value::Int(i) => BytecodeValue::Int(*i),
            crate::ir::Value::Float(f) => BytecodeValue::Float(*f),
            crate::ir::Value::String(s) => BytecodeValue::String(s.clone()),
            crate::ir::Value::Null => BytecodeValue::Null,
        }
    }
}