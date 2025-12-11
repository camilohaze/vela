/*
Convertidor AST → IR para Vela Compiler

Implementación de: TASK-070 (Bytecode generator desde IR)
Fecha: 2025-01-30

Este módulo convierte el AST generado por el parser en Representación
Intermedia (IR) que es más fácil de optimizar y generar bytecode.
*/

use std::collections::HashMap;
use crate::ast::{
    Program, Declaration, FunctionDeclaration, Statement, Expression, Literal, TypeAnnotation, IfStatement,
    BinaryExpression, UnaryExpression, CallExpression, Identifier, AssignmentStatement, VariableDeclaration,
    ReturnStatement, BlockStatement
};
use crate::types::Type;
use crate::ir::{IRModule, IRFunction, IRInstruction, IRType, Value as IRValue, BinaryOp, UnaryOp, IRExpr, Label};
use crate::error::{CompileError, CompileResult, CodegenError};
use crate::message_broker_decorators::{parse_message_broker_decorators, validate_consumer_function, generate_consumer_registration, MessageBrokerDecorator};
use crate::observability_decorators::{parse_observability_decorators, generate_observability_code, ObservabilityDecorator};
use crate::orm_decorators::{parse_orm_decorators, generate_orm_code, OrmDecorator};

/// Convertidor de AST a IR
pub struct ASTToIRConverter {
    /// Contador para generar labels únicos
    label_counter: usize,
    /// Tabla de símbolos para variables locales
    local_symbols: HashMap<String, IRType>,
    /// Función actual siendo convertida
    current_function: Option<String>,
}

impl ASTToIRConverter {
    pub fn new() -> Self {
        Self {
            label_counter: 0,
            local_symbols: HashMap::new(),
            current_function: None,
        }
    }

    /// Convertir programa completo a IR
    pub fn convert_program(&mut self, program: &Program) -> CompileResult<IRModule> {
        let mut module = IRModule::new("main".to_string());

        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(func) => {
                    let ir_function = self.convert_function(func)?;
                    module.add_function(ir_function);
                }
                Declaration::Variable(_) => {
                    // TODO: Implementar conversión de variables globales
                }
                Declaration::Struct(_) => {
                    // TODO: Implementar conversión de structs
                }
                Declaration::Enum(_) => {
                    // TODO: Implementar conversión de enums
                }
                _ => {
                    // Otros tipos de declaraciones
                }
            }
        }

        Ok(module)
    }

    /// Convertir función AST a IR
    fn convert_function(&mut self, func: &FunctionDeclaration) -> CompileResult<IRFunction> {
        self.current_function = Some(func.name.clone());
        self.local_symbols.clear();

        // Process message broker decorators
        if let Some(decorator) = parse_message_broker_decorators(&func.decorators)? {
            validate_consumer_function(func)?;

            // Generate consumer registration code
            let registration_code = generate_consumer_registration(
                &decorator,
                &func.name,
                "main" // TODO: Get actual module name
            );

            // Add registration as metadata to the function
            // This will be used by the runtime to register consumers
            println!("Generated consumer registration: {}", registration_code);
        }

        // Process observability decorators
        if let Some(decorator) = parse_observability_decorators(&func.decorators)? {
            // Generate observability instrumentation code
            let instrumentation_code = generate_observability_code(
                &decorator,
                &func.name,
                "main" // TODO: Get actual module name
            );

            // Add instrumentation as metadata to the function
            // This will be used by the runtime to inject observability
            println!("Generated observability instrumentation: {}", instrumentation_code);
        }

        // Process ORM decorators
        let orm_decorators = parse_orm_decorators(&func.decorators)?;
        if !orm_decorators.is_empty() {
            // Generate ORM code for entity operations
            let orm_code = generate_orm_code(
                &orm_decorators,
                &func.name,
                &[] // TODO: Get actual fields from struct
            )?;

            // Add ORM code as metadata to the function
            // This will be used by the runtime to generate entity implementations
            println!("Generated ORM code: {}", orm_code);
        }

        let return_type = if let Some(return_type_annotation) = &func.return_type {
            self.convert_type_annotation(return_type_annotation)?
        } else {
            IRType::Void
        };

        let mut ir_function = IRFunction::new(func.name.clone(), return_type);
        ir_function.position = Some(func.node.range.start.clone());

        // Agregar parámetros
        for param in &func.parameters {
            let param_type = if let Some(type_annotation) = &param.type_annotation {
                self.convert_type_annotation(type_annotation)?
            } else {
                IRType::Void // Inferir tipo si no hay anotación
            };
            ir_function.add_param(param.name.clone(), param_type.clone());
            self.local_symbols.insert(param.name.clone(), param_type);
        }

        // Convertir cuerpo de la función
        for stmt in &func.body.statements {
            let instructions = self.convert_statement(stmt)?;
            for instruction in instructions {
                ir_function.add_instruction(instruction);
            }
        }

        // Asegurar que hay un return al final si no es void
        if !matches!(ir_function.return_type, IRType::Void) {
            // TODO: Verificar si ya hay return
            ir_function.add_instruction(IRInstruction::Return);
        }

        self.current_function = None;
        Ok(ir_function)
    }

    /// Convertir statement AST a instrucciones IR
    fn convert_statement(&mut self, stmt: &Statement) -> CompileResult<Vec<IRInstruction>> {
        match stmt {
            Statement::Expression(expr) => {
                let (instructions, _) = self.convert_expression(&expr.expression)?;
                Ok(instructions)
            }
            Statement::Return(ret) => {
                let mut instructions = Vec::new();

                if let Some(expr) = &ret.value {
                    let (expr_instructions, _) = self.convert_expression(expr)?;
                    instructions.extend(expr_instructions);
                }

                instructions.push(IRInstruction::Return);
                Ok(instructions)
            }
            Statement::Variable(var_decl) => {
                let var_type = if let Some(type_annotation) = &var_decl.type_annotation {
                    self.convert_type_annotation(type_annotation)?
                } else {
                    IRType::Void // Inferir tipo si no hay anotación
                };
                self.local_symbols.insert(var_decl.name.clone(), var_type.clone());

                let mut instructions = vec![IRInstruction::DeclareVar {
                    name: var_decl.name.clone(),
                    ty: var_type,
                }];

                if let Some(initializer) = &var_decl.initializer {
                    let (expr_instructions, _) = self.convert_expression(initializer)?;
                    instructions.extend(expr_instructions);
                    instructions.push(IRInstruction::StoreVar(var_decl.name.clone()));
                }

                Ok(instructions)
            }
            Statement::Assignment(assignment) => {
                let mut instructions = Vec::new();

                // Convertir expresión del lado derecho
                let (rhs_instructions, _) = self.convert_expression(&assignment.value)?;
                instructions.extend(rhs_instructions);

                // Asignar a variable
                match &assignment.target {
                    Expression::Identifier(name) => {
                        instructions.push(IRInstruction::StoreVar(name.name.clone()));
                    }
                    _ => {
                        return Err(CompileError::Codegen(CodegenError {
                            message: "Complex assignments not yet supported in IR".to_string(),
                            location: None,
                        }));
                    }
                }

                Ok(instructions)
            }
            Statement::If(if_stmt) => {
                self.convert_if_statement(if_stmt)
            }
            Statement::Block(block) => {
                let mut instructions = Vec::new();
                for stmt in &block.statements {
                    let stmt_instructions = self.convert_statement(stmt)?;
                    instructions.extend(stmt_instructions);
                }
                Ok(instructions)
            }
            _ => {
                // Otros statements no implementados aún
                Ok(Vec::new())
            }
        }
    }

    /// Convertir expresión AST a instrucciones IR + tipo resultante
    fn convert_expression(&mut self, expr: &Expression) -> CompileResult<(Vec<IRInstruction>, IRType)> {
        match expr {
            Expression::Literal(lit) => {
                let (value, ty) = self.convert_literal(lit)?;
                let instructions = vec![IRInstruction::LoadConst(value)];
                Ok((instructions, ty))
            }
            Expression::Identifier(name) => {
                if let Some(var_type) = self.local_symbols.get(&name.name) {
                    let instructions = vec![IRInstruction::LoadVar(name.name.clone())];
                    Ok((instructions, var_type.clone()))
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: format!("Undefined variable: {}", name.name),
                        location: None,
                    }))
                }
            }
            Expression::Binary(binary_expr) => {
                let (left_instructions, left_type) = self.convert_expression(&binary_expr.left)?;
                let (right_instructions, right_type) = self.convert_expression(&binary_expr.right)?;
                
                let binary_op = self.convert_binary_op(&binary_expr.operator)?;
                
                let mut instructions = left_instructions;
                instructions.extend(right_instructions);
                instructions.push(IRInstruction::BinaryOp(binary_op));
                
                // TODO: Determinar tipo resultante basado en operación
                let result_type = left_type; // Simplificado
                
                Ok((instructions, result_type))
            }
            Expression::Unary(unary_expr) => {
                let (operand_instructions, operand_type) = self.convert_expression(&unary_expr.operand)?;

                let unary_op = self.convert_unary_op(&unary_expr.operator)?;
                let mut instructions = operand_instructions;
                instructions.push(IRInstruction::UnaryOp(unary_op));

                Ok((instructions, operand_type))
            }
            Expression::Call(call) => {
                let mut instructions = Vec::new();

                // Convertir argumentos (de derecha a izquierda para el stack)
                for arg in call.arguments.iter().rev() {
                    let (arg_instructions, _) = self.convert_expression(arg)?;
                    instructions.extend(arg_instructions);
                }

                // Por ahora asumimos que callee es un identificador
                let function_name = match &*call.callee {
                    Expression::Identifier(id) => id.name.clone(),
                    _ => return Err(CompileError::Codegen(CodegenError {
                        message: "Complex function calls not yet supported in IR".to_string(),
                        location: None,
                    })),
                };

                instructions.push(IRInstruction::Call {
                    function: function_name,
                    arg_count: call.arguments.len(),
                });

                // TODO: Inferir tipo de retorno
                Ok((instructions, IRType::Void))
            }
            _ => {
                // Expresiones complejas no implementadas
                Ok((Vec::new(), IRType::Void))
            }
        }
    }

    /// Convertir literal a Value + IRType
    fn convert_literal(&self, lit: &Literal) -> CompileResult<(IRValue, IRType)> {
        match lit.kind.as_str() {
            "bool" => {
                if let Some(b) = lit.value.as_bool() {
                    Ok((IRValue::Bool(b), IRType::Bool))
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: "Invalid boolean literal".to_string(),
                        location: None,
                    }))
                }
            }
            "number" => {
                if let Some(f) = lit.value.as_f64() {
                    Ok((IRValue::Float(f), IRType::Float))
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: "Invalid number literal".to_string(),
                        location: None,
                    }))
                }
            }
            "float" => {
                if let Some(f) = lit.value.as_f64() {
                    Ok((IRValue::Float(f), IRType::Float))
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: "Invalid float literal".to_string(),
                        location: None,
                    }))
                }
            }
            "string" => {
                if let Some(s) = lit.value.as_str() {
                    Ok((IRValue::String(s.to_string()), IRType::String))
                } else {
                    Err(CompileError::Codegen(CodegenError {
                        message: "Invalid string literal".to_string(),
                        location: None,
                    }))
                }
            }
            "none" => Ok((IRValue::Null, IRType::Void)),
            _ => Err(CompileError::Codegen(CodegenError {
                message: format!("Literal type '{}' not supported in IR", lit.kind),
                location: None,
            })),
        }
    }

    /// Convertir operador binario
    fn convert_binary_op(&self, op: &str) -> CompileResult<BinaryOp> {
        match op {
            "+" => Ok(BinaryOp::Add),
            "-" => Ok(BinaryOp::Sub),
            "*" => Ok(BinaryOp::Mul),
            "/" => Ok(BinaryOp::Div),
            "%" => Ok(BinaryOp::Mod),
            "==" => Ok(BinaryOp::Eq),
            "!=" => Ok(BinaryOp::Ne),
            "<" => Ok(BinaryOp::Lt),
            "<=" => Ok(BinaryOp::Le),
            ">" => Ok(BinaryOp::Gt),
            ">=" => Ok(BinaryOp::Ge),
            "&&" => Ok(BinaryOp::And),
            "||" => Ok(BinaryOp::Or),
            _ => Err(CompileError::Codegen(CodegenError {
                message: "Binary operator not supported in IR".to_string(),
                location: None,
            })),
        }
    }

    /// Convertir operador unario
    fn convert_unary_op(&self, op: &str) -> CompileResult<UnaryOp> {
        match op {
            "-" => Ok(UnaryOp::Neg),
            "!" => Ok(UnaryOp::Not),
            _ => Err(CompileError::Codegen(CodegenError {
                message: "Unary operator not supported in IR".to_string(),
                location: None,
            })),
        }
    }

    /// Convertir tipo AST a IRType
    fn convert_type_annotation(&self, ty: &TypeAnnotation) -> CompileResult<IRType> {
        match ty {
            TypeAnnotation::Primitive(prim) => match prim.name.as_str() {
                "Number" => Ok(IRType::Int),
                "Float" => Ok(IRType::Float),
                "String" => Ok(IRType::String),
                "Bool" => Ok(IRType::Bool),
                "void" => Ok(IRType::Void),
                _ => Ok(IRType::Void),
            },
            TypeAnnotation::Array(arr) => {
                // TODO: Implementar conversión de arrays
                Ok(IRType::Void)
            },
            _ => Ok(IRType::Void), // Para tipos complejos
        }
    }

    /// Convertir statement if
    fn convert_if_statement(&mut self, if_stmt: &IfStatement) -> CompileResult<Vec<IRInstruction>> {
        let mut instructions = Vec::new();

        // Generar labels
        let else_label = self.generate_label("else");
        let end_label = self.generate_label("endif");

        // Condición
        let (condition_instructions, _) = self.convert_expression(&if_stmt.condition)?;
        instructions.extend(condition_instructions);

        // Jump to else if condition is false
        instructions.push(IRInstruction::JumpIf(else_label.clone()));

        // Then branch
        let stmt_instructions = self.convert_statement(&if_stmt.then_branch)?;
        instructions.extend(stmt_instructions);

        // Jump to end
        instructions.push(IRInstruction::Jump(end_label.clone()));

        // Else branch
        instructions.push(IRInstruction::Label(else_label));
        if let Some(else_branch) = &if_stmt.else_branch {
            let stmt_instructions = self.convert_statement(else_branch)?;
            instructions.extend(stmt_instructions);
        }

        // End label
        instructions.push(IRInstruction::Label(end_label));

        Ok(instructions)
    }

    /// Convertir statement while
    /// Generar label único
    fn generate_label(&mut self, prefix: &str) -> Label {
        self.label_counter += 1;
        format!("{}_{}", prefix, self.label_counter)
    }
}