/*
Code Generator para el lenguaje Vela

Implementación de: TASK-RUST-106 (Code Generator Implementation)
Historia: US-RUST-02 (Compiler Foundation)
Fecha: 2025-12-01

Este módulo convierte el AST en bytecode ejecutable para la VelaVM.
Implementa un generador recursivo que atraviesa el AST y emite instrucciones
de bytecode apropiadas para cada constructo del lenguaje.
*/

use std::collections::HashMap;
use crate::ast::*;
use crate::error::{CompileError, CompileResult, CodegenError};
use vela_vm::{Bytecode, Instruction, Value, Function};

/// Generador de código que convierte AST en bytecode
pub struct CodeGenerator {
    /// Bytecode resultante
    bytecode: Bytecode,
    /// Tabla de símbolos para variables y funciones
    symbol_table: HashMap<String, usize>, // nombre -> índice en bytecode.constants
    /// Funciones definidas
    functions: Vec<Function>,
}

impl CodeGenerator {
    /// Crear un nuevo generador de código
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            symbol_table: HashMap::new(),
            functions: Vec::new(),
        }
    }

    /// Generar bytecode desde un programa AST
    pub fn generate_program(&mut self, program: &Program) -> CompileResult<Bytecode> {
        // Procesar declaraciones
        for declaration in &program.declarations {
            self.generate_declaration(declaration)?;
        }

        // Agregar instrucción de retorno al final si no hay una
        if self.bytecode.instructions.is_empty() || !matches!(self.bytecode.instructions.last(), Some(Instruction::Return)) {
            self.bytecode.instructions.push(Instruction::Return);
        }

        Ok(self.bytecode.clone())
    }

    /// Generar código para una declaración
    fn generate_declaration(&mut self, declaration: &Declaration) -> CompileResult<()> {
        match declaration {
            Declaration::Function(func) => self.generate_function_declaration(func),
            Declaration::Variable(var) => self.generate_variable_declaration(var),
            // Otras declaraciones no soportadas aún
            _ => Err(CompileError::Codegen(CodegenError {
                message: format!("Unsupported declaration type: {:?}", declaration),
                location: None,
            })),
        }
    }

    /// Generar código para declaración de función
    fn generate_function_declaration(&mut self, func: &FunctionDeclaration) -> CompileResult<()> {
        // Crear un nuevo generador para el scope de la función
        let mut func_generator = CodeGenerator::new();

        // Generar el cuerpo de la función
        func_generator.generate_block_statement(&func.body)?;

        // Agregar RETURN si no está presente
        if !func_generator.bytecode.instructions.iter().any(|i| matches!(i, Instruction::Return)) {
            func_generator.bytecode.instructions.push(Instruction::Return);
        }

        // Crear la función
        let function = Function::new(
            func.name.clone(),
            func.parameters.iter().map(|p| p.name.clone()).collect(),
            0, // body_start - será establecido por el VM
            func_generator.bytecode.instructions.len(), // body_end
        );

        // Agregar a la lista de funciones del generador principal
        self.functions.push(function.clone());

        // Agregar al bytecode principal como constante
        let const_index = self.bytecode.add_constant(Value::Function(function));
        self.symbol_table.insert(func.name.clone(), const_index);

        Ok(())
    }

    /// Generar código para declaración de variable
    fn generate_variable_declaration(&mut self, var: &VariableDeclaration) -> CompileResult<()> {
        if let Some(initializer) = &var.initializer {
            self.generate_expression(initializer)?;
            // Registrar en tabla de símbolos primero
            let var_index = self.symbol_table.len();
            self.symbol_table.insert(var.name.clone(), var_index);
            self.bytecode.instructions.push(Instruction::Store(var_index));
        }

        Ok(())
    }

    /// Generar código para un statement
    fn generate_statement(&mut self, statement: &Statement) -> CompileResult<()> {
        match statement {
            Statement::Block(block) => self.generate_block_statement(block),
            Statement::Expression(expr_stmt) => self.generate_expression_statement(expr_stmt),
            Statement::Variable(var) => self.generate_variable_declaration(var),
            Statement::Assignment(assign) => self.generate_assignment_statement(assign),
            Statement::Return(ret) => self.generate_return_statement(ret),
            Statement::If(if_stmt) => self.generate_if_statement(if_stmt),
            // Otros statements no soportados aún
            _ => Err(CompileError::Codegen(CodegenError {
                message: format!("Unsupported statement type: {:?}", statement),
                location: None,
            })),
        }
    }

    /// Generar código para bloque de statements
    fn generate_block_statement(&mut self, block: &BlockStatement) -> CompileResult<()> {
        for statement in &block.statements {
            self.generate_statement(statement)?;
        }
        Ok(())
    }

    /// Generar código para expression statement
    fn generate_expression_statement(&mut self, expr_stmt: &ExpressionStatement) -> CompileResult<()> {
        self.generate_expression(&expr_stmt.expression)?;
        // Los resultados de expresiones en statements se descartan
        self.bytecode.instructions.push(Instruction::Pop);
        Ok(())
    }

    /// Generar código para assignment
    fn generate_assignment_statement(&mut self, assign: &AssignmentStatement) -> CompileResult<()> {
        self.generate_expression(&assign.value)?;
        match &assign.target {
            Expression::Identifier(ident) => {
                if let Some(&index) = self.symbol_table.get(&ident.name) {
                    self.bytecode.instructions.push(Instruction::Store(index));
                } else {
                    return Err(CompileError::Codegen(CodegenError {
                        message: format!("Undefined variable: {}", ident.name),
                        location: None,
                    }));
                }
            }
            _ => return Err(CompileError::Codegen(CodegenError {
                message: "Unsupported assignment target".to_string(),
                location: None,
            })),
        }
        Ok(())
    }

    /// Generar código para return
    fn generate_return_statement(&mut self, ret: &ReturnStatement) -> CompileResult<()> {
        if let Some(value) = &ret.value {
            self.generate_expression(value)?;
        } else {
            self.bytecode.instructions.push(Instruction::Push(0)); // Push None equivalent
        }
        self.bytecode.instructions.push(Instruction::Return);
        Ok(())
    }

    /// Generar código para if statement
    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> CompileResult<()> {
        // Generar condición
        self.generate_expression(&if_stmt.condition)?;

        // JumpIf al else branch
        let jump_false_index = self.bytecode.instructions.len();
        self.bytecode.instructions.push(Instruction::JumpIf(0)); // placeholder

        // Generar then branch
        self.generate_statement(&*if_stmt.then_branch)?;

        if let Some(else_branch) = &if_stmt.else_branch {
            // Jump al final después del then
            let jump_end_index = self.bytecode.instructions.len();
            self.bytecode.instructions.push(Instruction::Jump(0)); // placeholder

            // Actualizar el JumpIf
            let else_start = self.bytecode.instructions.len();
            if let Instruction::JumpIf(ref mut offset) = &mut self.bytecode.instructions[jump_false_index] {
                *offset = else_start;
            }

            // Generar else branch
            self.generate_statement(&*else_branch)?;

            // Actualizar el Jump
            let end_pos = self.bytecode.instructions.len();
            if let Instruction::Jump(ref mut offset) = &mut self.bytecode.instructions[jump_end_index] {
                *offset = end_pos;
            }
        } else {
            // Sin else, actualizar JumpIf al final del then
            let end_pos = self.bytecode.instructions.len();
            if let Instruction::JumpIf(ref mut offset) = &mut self.bytecode.instructions[jump_false_index] {
                *offset = end_pos;
            }
        }

        Ok(())
    }

    /// Generar código para una expresión
    fn generate_expression(&mut self, expression: &Expression) -> CompileResult<()> {
        match expression {
            Expression::Literal(lit) => self.generate_literal(lit),
            Expression::Identifier(ident) => self.generate_identifier(ident),
            Expression::Binary(binary) => self.generate_binary_expression(binary),
            Expression::Unary(unary) => self.generate_unary_expression(unary),
            Expression::Call(call) => self.generate_call_expression(call),
            Expression::ArrayLiteral(array) => self.generate_array_literal(array),
            Expression::StructLiteral(struct_lit) => self.generate_struct_literal(struct_lit),
            // Otras expresiones no soportadas aún
            _ => Err(CompileError::Codegen(CodegenError {
                message: format!("Unsupported expression type: {:?}", expression),
                location: None,
            })),
        }
    }

    /// Generar código para literal
    fn generate_literal(&mut self, literal: &Literal) -> CompileResult<()> {
        match literal.kind.as_str() {
            "number" => {
                if let Some(num) = literal.value.as_f64() {
                    if num.fract() == 0.0 {
                        self.bytecode.instructions.push(Instruction::Push(num as i64));
                    } else {
                        self.bytecode.instructions.push(Instruction::PushFloat(num));
                    }
                } else {
                    return Err(CompileError::Codegen(CodegenError {
                        message: "Invalid number literal".to_string(),
                        location: None,
                    }));
                }
            }
            "string" => {
                if let Some(s) = literal.value.as_str() {
                    self.bytecode.instructions.push(Instruction::PushString(s.to_string()));
                } else {
                    return Err(CompileError::Codegen(CodegenError {
                        message: "Invalid string literal".to_string(),
                        location: None,
                    }));
                }
            }
            "bool" => {
                if let Some(b) = literal.value.as_bool() {
                    self.bytecode.instructions.push(Instruction::PushBool(b));
                } else {
                    return Err(CompileError::Codegen(CodegenError {
                        message: "Invalid bool literal".to_string(),
                        location: None,
                    }));
                }
            }
            _ => return Err(CompileError::Codegen(CodegenError {
                message: format!("Unsupported literal kind: {}", literal.kind),
                location: None,
            })),
        };
        Ok(())
    }

    /// Generar código para identificador
    fn generate_identifier(&mut self, ident: &Identifier) -> CompileResult<()> {
        if let Some(&index) = self.symbol_table.get(&ident.name) {
            self.bytecode.instructions.push(Instruction::Load(index));
        } else {
            return Err(CompileError::Codegen(CodegenError {
                message: format!("Undefined variable: {}", ident.name),
                location: None,
            }));
        }
        Ok(())
    }

    /// Generar código para expresión binaria
    fn generate_binary_expression(&mut self, binary: &BinaryExpression) -> CompileResult<()> {
        // Generar operandos
        self.generate_expression(&binary.left)?;
        self.generate_expression(&binary.right)?;

        // Generar operación
        let instruction = match binary.operator.as_str() {
            "+" => Instruction::Add,
            "-" => Instruction::Sub,
            "*" => Instruction::Mul,
            "/" => Instruction::Div,
            "==" => Instruction::Eq,
            "!=" => Instruction::Ne,
            "<" => Instruction::Lt,
            "<=" => Instruction::Le,
            ">" => Instruction::Gt,
            ">=" => Instruction::Ge,
            _ => return Err(CompileError::Codegen(CodegenError {
                message: format!("Unsupported binary operator: {}", binary.operator),
                location: None,
            })),
        };

        self.bytecode.instructions.push(instruction);
        Ok(())
    }

    /// Generar código para expresión unaria
    fn generate_unary_expression(&mut self, unary: &UnaryExpression) -> CompileResult<()> {
        self.generate_expression(&unary.operand)?;

        match unary.operator.as_str() {
            "-" => {
                // Negación: 0 - x
                self.bytecode.instructions.push(Instruction::Push(0));
                self.bytecode.instructions.push(Instruction::Sub);
            }
            "!" => {
                // NOT lógico - por ahora no soportado
                return Err(CompileError::Codegen(CodegenError {
                    message: "Logical NOT not yet supported".to_string(),
                    location: None,
                }));
            }
            _ => return Err(CompileError::Codegen(CodegenError {
                message: format!("Unsupported unary operator: {}", unary.operator),
                location: None,
            })),
        };

        Ok(())
    }

    /// Generar código para llamada a función
    fn generate_call_expression(&mut self, call: &CallExpression) -> CompileResult<()> {
        // Generar argumentos en orden inverso (para el stack)
        for arg in call.arguments.iter().rev() {
            self.generate_expression(arg)?;
        }

        // Generar callee
        self.generate_expression(&call.callee)?;

        // Generar llamada
        self.bytecode.instructions.push(Instruction::Call(call.arguments.len()));
        Ok(())
    }

    /// Generar código para array literal
    fn generate_array_literal(&mut self, array: &ArrayLiteral) -> CompileResult<()> {
        // Generar elementos
        for element in &array.elements {
            self.generate_expression(element)?;
        }

        // Crear array
        self.bytecode.instructions.push(Instruction::ListNew);
        Ok(())
    }

    /// Generar código para struct literal
    fn generate_struct_literal(&mut self, struct_lit: &StructLiteral) -> CompileResult<()> {
        // Generar valores de campos
        for field in &struct_lit.fields {
            self.generate_expression(&field.value)?;
        }

        // Crear dict con los campos
        let field_names: Vec<String> = struct_lit.fields.iter().map(|f| f.name.clone()).collect();
        for name in field_names {
            self.bytecode.instructions.push(Instruction::PushString(name));
        }
        self.bytecode.instructions.push(Instruction::DictNew);
        Ok(())
    }

    /// Obtener el bytecode generado
    pub fn get_bytecode(&self) -> &Bytecode {
        &self.bytecode
    }

    /// Obtener las funciones generadas
    pub fn get_functions(&self) -> &[Function] {
        &self.functions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_codegen_creation() {
        let mut generator = CodeGenerator::new();
        assert!(generator.bytecode.instructions.is_empty());
        assert!(generator.symbol_table.is_empty());
    }

    #[test]
    fn test_generate_variable_declaration() {
        let mut generator = CodeGenerator::new();

        let range = create_range(1, 1, 1, 10);
        let initializer = Expression::Literal(Literal::new(
            create_range(1, 8, 1, 10),
            serde_json::json!(42),
            "number".to_string(),
        ));

        let var_decl = VariableDeclaration::new(
            range,
            "x".to_string(),
            None,
            Some(initializer),
            false,
        );

        let result = generator.generate_variable_declaration(&var_decl);
        assert!(result.is_ok());

        // Verificar que se generaron las instrucciones correctas
        assert!(!generator.bytecode.instructions.is_empty());
        assert!(generator.symbol_table.contains_key("x"));
    }

    #[test]
    fn test_generate_function_declaration() {
        let mut generator = CodeGenerator::new();

        let range = create_range(1, 1, 5, 2);
        let body = BlockStatement::new(create_range(4, 1, 5, 2), vec![]);

        let func_decl = FunctionDeclaration::new(
            range,
            true,
            "test_func".to_string(),
            vec![],
            None,
            body,
            false,
            vec![],
        );

        let result = generator.generate_function_declaration(&func_decl);
        assert!(result.is_ok());

        // Verificar que se agregó la función
        assert_eq!(generator.functions.len(), 1);
        assert_eq!(generator.functions[0].name, "test_func");
    }

    #[test]
    fn test_generate_binary_expression() {
        let mut generator = CodeGenerator::new();

        let left = Expression::Literal(Literal::new(
            create_range(1, 1, 1, 1),
            serde_json::json!(5),
            "number".to_string(),
        ));
        let right = Expression::Literal(Literal::new(
            create_range(1, 5, 1, 5),
            serde_json::json!(3),
            "number".to_string(),
        ));

        let binary = BinaryExpression::new(
            create_range(1, 1, 1, 5),
            left,
            "+".to_string(),
            right,
        );

        let result = generator.generate_binary_expression(&binary);
        assert!(result.is_ok());

        // Verificar que se generó Add
        assert!(generator.bytecode.instructions.contains(&Instruction::Add));
    }

    #[test]
    fn test_generate_call_expression() {
        let mut generator = CodeGenerator::new();

        // Agregar la función "add" a la tabla de símbolos
        let func = Function::new("add".to_string(), vec!["a".to_string(), "b".to_string()], 0, 0);
        let const_index = generator.bytecode.add_constant(Value::Function(func));
        generator.symbol_table.insert("add".to_string(), const_index);

        let callee = Expression::Identifier(Identifier::new(
            create_range(1, 1, 1, 3),
            "add".to_string(),
        ));

        let args = vec![
            Expression::Literal(Literal::new(
                create_range(1, 5, 1, 5),
                serde_json::json!(2),
                "number".to_string(),
            )),
            Expression::Literal(Literal::new(
                create_range(1, 8, 1, 8),
                serde_json::json!(3),
                "number".to_string(),
            )),
        ];

        let call = CallExpression::new(
            create_range(1, 1, 1, 9),
            callee,
            args,
        );

        let result = generator.generate_call_expression(&call);
        assert!(result.is_ok());

        // Verificar que se generó Call
        assert!(generator.bytecode.instructions.iter().any(|i| matches!(i, Instruction::Call(_))));
    }

    #[test]
    fn test_complex_program() {
        let mut generator = CodeGenerator::new();

        // Crear un programa simple: fn main() { let x = 5 + 3; return x; }
        let range = create_range(1, 1, 10, 1);

        // Declaración de función
        let func_range = create_range(1, 1, 3, 2);

        // Variable declaration: let x = 5 + 3
        let var_decl = Statement::Variable(VariableDeclaration::new(
            create_range(2, 5, 2, 15),
            "x".to_string(),
            None,
            Some(Expression::Binary(BinaryExpression::new(
                create_range(2, 13, 2, 15),
                Expression::Literal(Literal::new(create_range(2, 13, 2, 13), serde_json::json!(5), "number".to_string())),
                "+".to_string(),
                Expression::Literal(Literal::new(create_range(2, 15, 2, 15), serde_json::json!(3), "number".to_string())),
            ))),
            false,
        ));

        // Return statement: return x
        let return_stmt = Statement::Return(ReturnStatement::new(
            create_range(3, 5, 3, 10),
            Some(Expression::Identifier(Identifier::new(create_range(3, 12, 3, 12), "x".to_string()))),
        ));

        let body = BlockStatement::new(
            create_range(1, 15, 3, 2),
            vec![var_decl, return_stmt],
        );

        let func_decl = FunctionDeclaration::new(
            func_range,
            true,
            "main".to_string(),
            vec![],
            None,
            body,
            false,
            vec![],
        );

        let program = Program::new(
            range,
            vec![],
            vec![Declaration::Function(func_decl)],
        );

        let result = generator.generate_program(&program);
        assert!(result.is_ok());

        let bytecode = result.unwrap();
        assert!(!bytecode.instructions.is_empty());
        assert_eq!(generator.functions.len(), 1);
    }
}