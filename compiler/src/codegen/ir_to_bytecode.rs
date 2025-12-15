/*
Generador de Bytecode desde IR para Vela Compiler

Implementación de: TASK-070 (Bytecode generator desde IR)
Fecha: 2025-01-30

Este módulo convierte la Representación Intermedia (IR) en bytecode
ejecutable por la VelaVM. Implementa optimizaciones básicas y
genera código eficiente.
*/

use std::collections::HashMap;
use crate::ir::{IRModule, IRFunction, IRInstruction, IRExpr, IRType, Value as IRValue, BinaryOp, UnaryOp, Label};
use crate::bytecode::{BytecodeProgram, BytecodeFunction, Value as BytecodeValue, Opcode};
use crate::debug_info::{DebugInfoGenerator, SourceLocation};
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
    /// Generador de información de debug
    debug_generator: DebugInfoGenerator,
    /// Ubicación fuente actual
    current_source_location: Option<SourceLocation>,
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
            debug_generator: DebugInfoGenerator::new(),
            current_source_location: None,
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

        // Agregar información de debug
        let debug_info = self.debug_generator.get_debug_info().clone();
        program.set_debug_info(debug_info);

        Ok(program)
    }

    /// Generar bytecode para una función IR
    fn generate_function(&mut self, function: &IRFunction) -> CompileResult<Vec<u8>> {
        self.local_symbols.clear();
        self.local_counter = 0;
        self.labels.clear();
        self.pending_labels.clear();

        // Iniciar tracking de debug info para esta función
        let function_start_offset = 0; // Se calculará después
        self.debug_generator.start_function(function.name.clone(), function_start_offset);

        // Registrar parámetros como variables locales
        for (i, param) in function.params.iter().enumerate() {
            let local_index = self.local_counter;
            self.local_symbols.insert(param.name.clone(), local_index);
            self.local_counter += 1;

            // Agregar parámetro a debug info
            self.debug_generator.add_parameter(
                param.name.clone(),
                format!("{:?}", param.ty), // Convertir tipo a string
                i
            );
        }

        // Registrar otras variables locales
        for local in &function.locals {
            let local_index = self.local_counter;
            self.local_symbols.insert(local.name.clone(), local_index);
            self.local_counter += 1;

            // Agregar variable local a debug info
            self.debug_generator.add_local(
                local.name.clone(),
                format!("{:?}", local.ty), // Convertir tipo a string
                crate::debug_info::VariableLocation::Stack(local_index)
            );
        }

        let mut bytecode = Vec::new();

        // Iniciar tracking de debug info para esta función con offset correcto
        let function_start_offset = bytecode.len();
        self.debug_generator.start_function(function.name.clone(), function_start_offset);
        for instruction in &function.body {
            match instruction {
                IRInstruction::Label(label) => {
                    let position = bytecode.len();
                    self.labels.insert(label.clone(), position);
                }
                _ => {
                    self.generate_instruction_with_debug(instruction, &mut bytecode)?;
                }
            }
        }

        // Resolver labels pendientes
        self.resolve_labels(&mut bytecode)?;

        // Finalizar tracking de debug info para esta función
        let function_end_offset = bytecode.len();
        self.debug_generator.end_function(function_end_offset);

        Ok(bytecode)
    }

    /// Registrar ubicación fuente actual
    pub fn set_source_location(&mut self, line: usize, column: usize) {
        self.current_source_location = Some(SourceLocation::new(line, column));
    }

    /// Generar bytecode para una instrucción IR con tracking de debug
    fn generate_instruction_with_debug(&mut self, instruction: &IRInstruction, bytecode: &mut Vec<u8>) -> CompileResult<()> {
        let instr_bytes = self.generate_instruction(instruction)?;
        let offset = bytecode.len();

        // Registrar mapeo de línea si hay ubicación fuente
        if let Some(location) = self.current_source_location {
            self.debug_generator.record_line_mapping(location.line, location.column, offset);
        }

        bytecode.extend(instr_bytes);
        Ok(())
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
                // TODO: Implement AssignVar - currently handled by StoreVar in AST conversion
                todo!("AssignVar instruction not yet implemented - use StoreVar instead")
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
        // Primero, optimizar expresiones en asignaciones
        self.optimize_assignments(function);

        // Luego, optimizar instrucciones secuenciales (implementación original mejorada)
        let mut i = 0;
        while i < function.body.len() {
            match &function.body[i] {
                IRInstruction::BinaryOp(op) => {
                    // Verificar si los dos valores anteriores son constantes
                    if i >= 2 {
                        if let (IRInstruction::LoadConst(a), IRInstruction::LoadConst(b)) =
                            (&function.body[i-2], &function.body[i-1]) {
                            if let Some(result) = self.fold_binary_op(op.clone(), a.clone(), b.clone()) {
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
                            if let Some(result) = self.fold_unary_op(op.clone(), val.clone()) {
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

    /// Optimizar asignaciones constantes reemplazando expresiones con valores constantes
    fn optimize_assignments(&self, function: &mut IRFunction) {
        for instruction in &mut function.body {
            if let IRInstruction::AssignVar { name, value } = instruction {
                if let Some(constant_value) = self.evaluate_constant_expr(value) {
                    // Reemplazar la asignación con una asignación constante
                    *value = IRExpr::Const(constant_value);
                } else {
                    // Intentar simplificar la expresión parcialmente
                    self.simplify_expr(value);
                }
            }
        }
    }

    /// Evaluar si una expresión IRExpr es constante y devolver su valor
    /// Evaluar expresión constante en compile-time
    pub fn evaluate_constant_expr(&self, expr: &IRExpr) -> Option<IRValue> {
        match expr {
            IRExpr::Const(val) => Some(val.clone()),
            IRExpr::BinaryOp(op, left, right) => {
                let left_val = self.evaluate_constant_expr(left)?;
                let right_val = self.evaluate_constant_expr(right)?;
                self.fold_binary_op_expr(op.clone(), left_val, right_val)
            }
            IRExpr::UnaryOp(op, operand) => {
                let operand_val = self.evaluate_constant_expr(operand)?;
                self.fold_unary_op_expr(op.clone(), operand_val)
            }
            IRExpr::Call { function, args } => {
                self.evaluate_constant_call(function, args)
            }
            // No podemos evaluar variables, accesos a arrays, o propiedades en compile-time
            IRExpr::Var(_) | IRExpr::ArrayAccess { .. } | IRExpr::PropertyAccess { .. } => None,
        }
    }

    /// Simplificar expresión parcialmente (no completamente constante pero optimizable)
    pub fn simplify_expr(&self, expr: &mut IRExpr) {
        match expr {
            IRExpr::BinaryOp(op, left, right) => {
                // Simplificar subexpresiones
                self.simplify_expr(left);
                self.simplify_expr(right);

                // Aplicar reglas algebraicas simples
                self.apply_algebraic_simplification(expr);
            }
            IRExpr::UnaryOp(op, operand) => {
                self.simplify_expr(operand);

                // Simplificar operaciones unarias dobles
                if let IRExpr::UnaryOp(inner_op, inner_operand) = operand.as_ref() {
                    if op == inner_op && matches!(op, UnaryOp::Not | UnaryOp::Neg) {
                        // !!x -> x, --x -> x (para booleanos y números)
                        *expr = (**inner_operand).clone();
                    }
                }
            }
            IRExpr::Call { args, .. } => {
                // Simplificar argumentos de llamadas
                for arg in args {
                    self.simplify_expr(arg);
                }
            }
            IRExpr::ArrayAccess { array, index } => {
                self.simplify_expr(array);
                self.simplify_expr(index);
            }
            IRExpr::PropertyAccess { object, .. } => {
                self.simplify_expr(object);
            }
            // No simplificar constantes o variables
            IRExpr::Const(_) | IRExpr::Var(_) => {}
        }
    }

    /// Aplicar simplificaciones algebraicas a expresiones binarias
    fn apply_algebraic_simplification(&self, expr: &mut IRExpr) {
        if let IRExpr::BinaryOp(op, left, right) = expr {
            // x * 0 -> 0, x * 1 -> x
            if *op == BinaryOp::Mul {
                if let IRExpr::Const(IRValue::Int(0)) = **right {
                    *expr = IRExpr::Const(IRValue::Int(0));
                    return;
                }
                if let IRExpr::Const(IRValue::Int(1)) = **right {
                    *expr = (**left).clone();
                    return;
                }
                if let IRExpr::Const(IRValue::Int(0)) = **left {
                    *expr = IRExpr::Const(IRValue::Int(0));
                    return;
                }
                if let IRExpr::Const(IRValue::Int(1)) = **left {
                    *expr = (**right).clone();
                    return;
                }
            }

            // x + 0 -> x, x - 0 -> x
            if matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                if let IRExpr::Const(IRValue::Int(0)) = **right {
                    *expr = (**left).clone();
                    return;
                }
            }

            // x / 1 -> x
            if *op == BinaryOp::Div {
                if let IRExpr::Const(IRValue::Int(1)) = **right {
                    *expr = (**left).clone();
                    return;
                }
            }

            // x * 0 -> 0, x * 1 -> x para floats también
            if *op == BinaryOp::Mul {
                if let IRExpr::Const(IRValue::Float(0.0)) = **right {
                    *expr = IRExpr::Const(IRValue::Float(0.0));
                    return;
                }
                if let IRExpr::Const(IRValue::Float(1.0)) = **right {
                    *expr = (**left).clone();
                    return;
                }
                if let IRExpr::Const(IRValue::Float(0.0)) = **left {
                    *expr = IRExpr::Const(IRValue::Float(0.0));
                    return;
                }
                if let IRExpr::Const(IRValue::Float(1.0)) = **left {
                    *expr = (**right).clone();
                    return;
                }
            }

            // x + 0 -> x, x - 0 -> x para floats
            if matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                if let IRExpr::Const(IRValue::Float(0.0)) = **right {
                    *expr = (**left).clone();
                    return;
                }
            }

            // x / 1 -> x para floats
            if *op == BinaryOp::Div {
                if let IRExpr::Const(IRValue::Float(1.0)) = **right {
                    *expr = (**left).clone();
                    return;
                }
            }

            // Optimizaciones booleanas
            if *op == BinaryOp::And {
                // true && x -> x, false && x -> false
                if let IRExpr::Const(IRValue::Bool(true)) = **left {
                    *expr = (**right).clone();
                    return;
                }
                if let IRExpr::Const(IRValue::Bool(false)) = **left {
                    *expr = IRExpr::Const(IRValue::Bool(false));
                    return;
                }
                if let IRExpr::Const(IRValue::Bool(true)) = **right {
                    *expr = (**left).clone();
                    return;
                }
                if let IRExpr::Const(IRValue::Bool(false)) = **right {
                    *expr = IRExpr::Const(IRValue::Bool(false));
                    return;
                }
            }

            if *op == BinaryOp::Or {
                // true || x -> true, false || x -> x
                if let IRExpr::Const(IRValue::Bool(true)) = **left {
                    *expr = IRExpr::Const(IRValue::Bool(true));
                    return;
                }
                if let IRExpr::Const(IRValue::Bool(false)) = **left {
                    *expr = (**right).clone();
                    return;
                }
                if let IRExpr::Const(IRValue::Bool(true)) = **right {
                    *expr = IRExpr::Const(IRValue::Bool(true));
                    return;
                }
                if let IRExpr::Const(IRValue::Bool(false)) = **right {
                    *expr = (**left).clone();
                    return;
                }
            }
        }
    }

    /// Evaluar llamada a función constante (funciones puras conocidas)
    fn evaluate_constant_call(&self, function: &str, args: &[IRExpr]) -> Option<IRValue> {
        // Solo evaluar funciones puras que no tienen efectos secundarios
        match function {
            "len" | "length" => {
                // len(string) -> longitud de string constante
                if args.len() == 1 {
                    if let Some(IRValue::String(s)) = self.evaluate_constant_expr(&args[0]) {
                        return Some(IRValue::Int(s.len() as i64));
                    }
                }
                None
            }
            "is_empty" => {
                // is_empty(string) -> verificar si string está vacío
                if args.len() == 1 {
                    if let Some(IRValue::String(s)) = self.evaluate_constant_expr(&args[0]) {
                        return Some(IRValue::Bool(s.is_empty()));
                    }
                }
                None
            }
            "abs" => {
                // abs(number) -> valor absoluto
                if args.len() == 1 {
                    match self.evaluate_constant_expr(&args[0]) {
                        Some(IRValue::Int(x)) => Some(IRValue::Int(x.abs())),
                        Some(IRValue::Float(x)) => Some(IRValue::Float(x.abs())),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "min" => {
                // min(a, b) -> mínimo de dos números
                if args.len() == 2 {
                    let a = self.evaluate_constant_expr(&args[0])?;
                    let b = self.evaluate_constant_expr(&args[1])?;
                    match (a, b) {
                        (IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Int(x.min(y))),
                        (IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Float(x.min(y))),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "max" => {
                // max(a, b) -> máximo de dos números
                if args.len() == 2 {
                    let a = self.evaluate_constant_expr(&args[0])?;
                    let b = self.evaluate_constant_expr(&args[1])?;
                    match (a, b) {
                        (IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Int(x.max(y))),
                        (IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Float(x.max(y))),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "pow" => {
                // pow(base, exp) -> base^exp para enteros pequeños
                if args.len() == 2 {
                    let base = self.evaluate_constant_expr(&args[0])?;
                    let exp = self.evaluate_constant_expr(&args[1])?;
                    match (base, exp) {
                        (IRValue::Int(b), IRValue::Int(e)) if e >= 0 && e <= 10 => {
                            Some(IRValue::Int(b.pow(e as u32)))
                        }
                        (IRValue::Float(b), IRValue::Float(e)) => Some(IRValue::Float(b.powf(e))),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None, // No evaluar funciones desconocidas o con efectos secundarios
        }
    }

    /// Aplicar operación binaria a valores constantes (versión para expresiones)
    fn fold_binary_op_expr(&self, op: BinaryOp, a: IRValue, b: IRValue) -> Option<IRValue> {
        match (op, a, b) {
            // Operaciones aritméticas enteras
            (BinaryOp::Add, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Int(x + y)),
            (BinaryOp::Sub, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Int(x - y)),
            (BinaryOp::Mul, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Int(x * y)),
            (BinaryOp::Div, IRValue::Int(x), IRValue::Int(y)) if y != 0 => Some(IRValue::Int(x / y)),
            (BinaryOp::Mod, IRValue::Int(x), IRValue::Int(y)) if y != 0 => Some(IRValue::Int(x % y)),

            // Operaciones aritméticas flotantes
            (BinaryOp::Add, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Float(x + y)),
            (BinaryOp::Sub, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Float(x - y)),
            (BinaryOp::Mul, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Float(x * y)),
            (BinaryOp::Div, IRValue::Float(x), IRValue::Float(y)) if y != 0.0 => Some(IRValue::Float(x / y)),

            // Comparaciones enteras
            (BinaryOp::Eq, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x == y)),
            (BinaryOp::Ne, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x != y)),
            (BinaryOp::Lt, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x < y)),
            (BinaryOp::Le, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x <= y)),
            (BinaryOp::Gt, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x > y)),
            (BinaryOp::Ge, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x >= y)),

            // Comparaciones flotantes
            (BinaryOp::Eq, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Bool(x == y)),
            (BinaryOp::Ne, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Bool(x != y)),
            (BinaryOp::Lt, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Bool(x < y)),
            (BinaryOp::Le, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Bool(x <= y)),
            (BinaryOp::Gt, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Bool(x > y)),
            (BinaryOp::Ge, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Bool(x >= y)),

            // Comparaciones booleanas
            (BinaryOp::Eq, IRValue::Bool(x), IRValue::Bool(y)) => Some(IRValue::Bool(x == y)),
            (BinaryOp::Ne, IRValue::Bool(x), IRValue::Bool(y)) => Some(IRValue::Bool(x != y)),

            // Comparaciones de strings
            (BinaryOp::Eq, IRValue::String(x), IRValue::String(y)) => Some(IRValue::Bool(x == y)),
            (BinaryOp::Ne, IRValue::String(x), IRValue::String(y)) => Some(IRValue::Bool(x != y)),
            (BinaryOp::Lt, IRValue::String(x), IRValue::String(y)) => Some(IRValue::Bool(x < y)),
            (BinaryOp::Le, IRValue::String(x), IRValue::String(y)) => Some(IRValue::Bool(x <= y)),
            (BinaryOp::Gt, IRValue::String(x), IRValue::String(y)) => Some(IRValue::Bool(x > y)),
            (BinaryOp::Ge, IRValue::String(x), IRValue::String(y)) => Some(IRValue::Bool(x >= y)),

            // Operaciones lógicas booleanas
            (BinaryOp::And, IRValue::Bool(x), IRValue::Bool(y)) => Some(IRValue::Bool(x && y)),
            (BinaryOp::Or, IRValue::Bool(x), IRValue::Bool(y)) => Some(IRValue::Bool(x || y)),

            // Concatenación de strings
            (BinaryOp::Add, IRValue::String(x), IRValue::String(y)) => Some(IRValue::String(x + &y)),

            _ => None,
        }
    }

    /// Aplicar operación unaria a valor constante (versión para expresiones)
    fn fold_unary_op_expr(&self, op: UnaryOp, val: IRValue) -> Option<IRValue> {
        match (op, val) {
            (UnaryOp::Neg, IRValue::Int(x)) => Some(IRValue::Int(-x)),
            (UnaryOp::Neg, IRValue::Float(x)) => Some(IRValue::Float(-x)),
            (UnaryOp::Not, IRValue::Bool(x)) => Some(IRValue::Bool(!x)),
            _ => None,
        }
    }

    /// Aplicar operación binaria a constantes
    fn fold_binary_op(&self, op: BinaryOp, a: IRValue, b: IRValue) -> Option<IRValue> {
        match (op, a, b) {
            (BinaryOp::Add, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Int(x + y)),
            (BinaryOp::Sub, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Int(x - y)),
            (BinaryOp::Mul, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Int(x * y)),
            (BinaryOp::Div, IRValue::Int(x), IRValue::Int(y)) if y != 0 => Some(IRValue::Int(x / y)),
            (BinaryOp::Add, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Float(x + y)),
            (BinaryOp::Sub, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Float(x - y)),
            (BinaryOp::Mul, IRValue::Float(x), IRValue::Float(y)) => Some(IRValue::Float(x * y)),
            (BinaryOp::Div, IRValue::Float(x), IRValue::Float(y)) if y != 0.0 => Some(IRValue::Float(x / y)),
            (BinaryOp::Eq, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x == y)),
            (BinaryOp::Ne, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x != y)),
            (BinaryOp::Lt, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x < y)),
            (BinaryOp::Le, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x <= y)),
            (BinaryOp::Gt, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x > y)),
            (BinaryOp::Ge, IRValue::Int(x), IRValue::Int(y)) => Some(IRValue::Bool(x >= y)),
            _ => None,
        }
    }

    /// Aplicar operación unaria a constante
    fn fold_unary_op(&self, op: UnaryOp, val: IRValue) -> Option<IRValue> {
        match (op, val) {
            (UnaryOp::Neg, IRValue::Int(x)) => Some(IRValue::Int(-x)),
            (UnaryOp::Neg, IRValue::Float(x)) => Some(IRValue::Float(-x)),
            (UnaryOp::Not, IRValue::Bool(x)) => Some(IRValue::Bool(!x)),
            _ => None,
        }
    }

    /// Dead code elimination: eliminar código inalcanzable y no utilizado
    pub fn dead_code_elimination(&self, function: &mut IRFunction) {
        // Fase 1: Eliminar código inalcanzable después de Return/Break/Continue
        self.eliminate_unreachable_code(function);

        // Fase 2: Eliminar variables no utilizadas
        self.eliminate_unused_variables(function);

        // Fase 3: Eliminar instrucciones dead (efectos secundarios mínimos)
        self.eliminate_dead_instructions(function);
    }

    /// Eliminar código inalcanzable después de Return/Break/Continue
    fn eliminate_unreachable_code(&self, function: &mut IRFunction) {
        let mut i = 0;
        while i < function.body.len() {
            match function.body[i] {
                IRInstruction::Return => {
                    // Eliminar todo después del Return
                    function.body.truncate(i + 1);
                    break;
                }
                IRInstruction::Jump(_) => {
                    // Para saltos incondicionales, marcar como potencialmente unreachable
                    // pero por ahora conservamos (análisis de flujo más complejo)
                    i += 1;
                }
                IRInstruction::JumpIf(_) => {
                    // Ramas condicionales - por ahora conservamos
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
    }

    /// Eliminar variables no utilizadas
    fn eliminate_unused_variables(&self, function: &mut IRFunction) {
        use std::collections::HashSet;

        // Paso 1: Identificar variables definidas
        let mut defined_vars = HashSet::new();
        let mut used_vars = HashSet::new();

        // Paso 2: Recopilar todas las variables definidas y usadas
        for instruction in &function.body {
            match instruction {
                IRInstruction::AssignVar { name, .. } => {
                    defined_vars.insert(name.clone());
                }
                IRInstruction::LoadVar(name) => {
                    used_vars.insert(name.clone());
                }
                IRInstruction::BinaryOp(_) | IRInstruction::UnaryOp(_) => {
                    // Estas instrucciones usan los valores de la pila, no variables nombradas
                }
                IRInstruction::Call { .. } => {
                    // Las llamadas pueden usar variables, pero por simplicidad las conservamos
                }
                IRInstruction::Return => {
                    // Return puede devolver una variable
                }
                IRInstruction::Jump(_) | IRInstruction::JumpIf(_) => {
                    // Control flow - conservamos
                }
                IRInstruction::LoadConst(_) => {
                    // Constantes - no afectan variables
                }
                IRInstruction::Label(_) => {
                    // Labels - no afectan variables
                }
                _ => {
                    // Otros casos - conservamos por simplicidad
                }
            }
        }

        // Paso 3: Identificar variables no utilizadas
        let unused_vars: HashSet<_> = defined_vars.difference(&used_vars).cloned().collect();

        // Paso 4: Eliminar asignaciones a variables no utilizadas y sus instrucciones precedentes
        let mut instructions_to_remove = HashSet::new();
        
        for (i, instruction) in function.body.iter().enumerate() {
            if let IRInstruction::AssignVar { name, .. } = instruction {
                if unused_vars.contains(name) {
                    // Marcar esta asignación para eliminación
                    instructions_to_remove.insert(i);
                    
                    // Si la instrucción anterior es LoadConst, también marcarla
                    if i > 0 {
                        if let IRInstruction::LoadConst(_) = &function.body[i - 1] {
                            instructions_to_remove.insert(i - 1);
                        }
                    }
                }
            }
        }
        
        // Eliminar las instrucciones marcadas (en orden inverso para no afectar índices)
        let mut indices_to_remove: Vec<_> = instructions_to_remove.into_iter().collect();
        indices_to_remove.sort_by(|a, b| b.cmp(a)); // Ordenar en reversa
        
        for index in indices_to_remove {
            function.body.remove(index);
        }
    }

    /// Eliminar instrucciones dead con efectos secundarios mínimos
    fn eliminate_dead_instructions(&self, function: &mut IRFunction) {
        // Por ahora, solo eliminamos LoadConst seguidos de otros LoadConst sin uso
        // Esto es conservador para evitar eliminar código con efectos secundarios

        let mut i = 0;
        while i < function.body.len().saturating_sub(1) {
            match (&function.body[i], &function.body[i + 1]) {
                (IRInstruction::LoadConst(_), IRInstruction::LoadConst(_)) => {
                    // Si hay dos LoadConst seguidos, el primero es potencialmente dead
                    // Pero conservamos por ahora para evitar errores
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
    }
}