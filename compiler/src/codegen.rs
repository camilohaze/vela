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
use vela_vm::{Bytecode, Instruction, Value, CodeObject};

/// Generador de código que convierte AST en bytecode
pub struct CodeGenerator {
    /// Bytecode resultante
    bytecode: Bytecode,
    /// Tabla de símbolos para variables y funciones (nombre -> índice en constantes/strings)
    symbol_table: HashMap<String, u16>,
    /// Funciones built-in disponibles
    builtin_functions: HashMap<String, u16>,
    /// Índice del code object actual
    current_code_object: usize,
}

impl CodeGenerator {
    /// Crear un nuevo generador de código
    pub fn new() -> Self {
        let mut bytecode = Bytecode::new();
        // Crear el code object principal
        bytecode.code_objects.push(CodeObject::new(0, 0)); // name=0, filename=0

        // Inicializar funciones built-in
        let mut builtin_functions = HashMap::new();
        let println_str = bytecode.add_string("println".to_string());
        builtin_functions.insert("println".to_string(), println_str);

        Self {
            bytecode,
            symbol_table: HashMap::new(),
            builtin_functions,
            current_code_object: 0,
        }
    }

    /// Generar bytecode desde un programa AST
    pub fn generate_program(&mut self, program: &Program) -> CompileResult<Bytecode> {
        // Procesar declaraciones
        for declaration in &program.declarations {
            self.generate_declaration(declaration)?;
        }

        // Agregar instrucción de retorno al final si no hay una
        let code_obj = &self.bytecode.code_objects[self.current_code_object];
        if code_obj.bytecode.is_empty() || !matches!(code_obj.bytecode.last(), Some(0x51)) { // 0x51 = Return
            self.emit_instruction(Instruction::Return);
        }

        Ok(self.bytecode.clone())
    }

    /// Emitir una instrucción al code object actual
    fn emit_instruction(&mut self, instruction: Instruction) {
        self.bytecode.push(instruction);
    }

    /// Agregar una constante y retornar su índice
    fn add_constant(&mut self, constant: vela_vm::Constant) -> u16 {
        self.bytecode.constants.push(constant);
        (self.bytecode.constants.len() - 1) as u16
    }

    /// Agregar una string y retornar su índice
    fn add_string(&mut self, string: String) -> u16 {
        self.bytecode.strings.push(string);
        (self.bytecode.strings.len() - 1) as u16
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
        // Crear un nuevo code object para la función
        let name_idx = self.add_string(func.name.clone());
        let filename_idx = self.add_string("main.vela".to_string()); // TODO: obtener del contexto
        let code_obj_idx = self.bytecode.code_objects.len() as u16;

        let mut code_obj = CodeObject::new(name_idx, filename_idx);
        code_obj.arg_count = func.parameters.len() as u16;
        code_obj.local_count = func.parameters.len() as u16; // Por ahora, parámetros = locales

        // Guardar el code object actual
        let prev_code_object = self.current_code_object;

        // Agregar el nuevo code object y cambiar al contexto de la función
        self.bytecode.code_objects.push(code_obj);
        self.current_code_object = self.bytecode.code_objects.len() - 1;

        // Generar el cuerpo de la función
        self.generate_block_statement(&func.body)?;

        // Agregar RETURN si no está presente
        let current_code = &self.bytecode.code_objects[self.current_code_object];
        if current_code.bytecode.is_empty() || !matches!(current_code.bytecode.last(), Some(0x51)) { // 0x51 = Return
            self.emit_instruction(Instruction::Return);
        }

        // Restaurar el code object anterior
        self.current_code_object = prev_code_object;

        // Agregar el índice del code object como constante
        let const_idx = self.add_constant(vela_vm::Constant::Code(code_obj_idx));
        self.symbol_table.insert(func.name.clone(), const_idx);

        Ok(())
    }

    /// Generar código para declaración de variable
    fn generate_variable_declaration(&mut self, var: &VariableDeclaration) -> CompileResult<()> {
        if let Some(initializer) = &var.initializer {
            self.generate_expression(initializer)?;
            // Registrar en tabla de símbolos
            let var_index = self.symbol_table.len() as u16;
            self.symbol_table.insert(var.name.clone(), var_index);
            self.emit_instruction(Instruction::StoreLocal(var_index));
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
        self.emit_instruction(Instruction::Pop);
        Ok(())
    }

    /// Generar código para assignment
    fn generate_assignment_statement(&mut self, assign: &AssignmentStatement) -> CompileResult<()> {
        self.generate_expression(&assign.value)?;
        match &assign.target {
            Expression::Identifier(ident) => {
                if let Some(&index) = self.symbol_table.get(&ident.name) {
                    self.emit_instruction(Instruction::StoreLocal(index));
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
            // Push null for void return
            let null_idx = self.add_constant(vela_vm::Constant::Null);
            self.emit_instruction(Instruction::LoadConst(null_idx));
        }
        self.emit_instruction(Instruction::Return);
        Ok(())
    }

    /// Generar código para if statement
    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> CompileResult<()> {
        // Generar condición
        self.generate_expression(&if_stmt.condition)?;

        // JumpIfFalse al else branch (placeholder por ahora)
        self.emit_instruction(Instruction::JumpIfFalse(0)); // será actualizado

        // Generar then branch
        self.generate_statement(&*if_stmt.then_branch)?;

        if let Some(else_branch) = &if_stmt.else_branch {
            // Jump al final después del then (placeholder)
            self.emit_instruction(Instruction::Jump(0)); // será actualizado

            // Aquí iría la lógica para backpatch los jumps
            // Por simplicidad, por ahora solo generamos el else
            self.generate_statement(&*else_branch)?;
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
                        let const_idx = self.add_constant(vela_vm::Constant::Int(num as i64));
                        self.emit_instruction(Instruction::LoadConst(const_idx));
                    } else {
                        let const_idx = self.add_constant(vela_vm::Constant::Float(num));
                        self.emit_instruction(Instruction::LoadConst(const_idx));
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
                    let string_idx = self.add_string(s.to_string());
                    let const_idx = self.add_constant(vela_vm::Constant::String(string_idx));
                    self.emit_instruction(Instruction::LoadConst(const_idx));
                } else {
                    return Err(CompileError::Codegen(CodegenError {
                        message: "Invalid string literal".to_string(),
                        location: None,
                    }));
                }
            }
            "bool" => {
                if let Some(b) = literal.value.as_bool() {
                    let const_idx = self.add_constant(vela_vm::Constant::Bool(b));
                    self.emit_instruction(Instruction::LoadConst(const_idx));
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
        // Primero buscar en variables locales
        if let Some(&index) = self.symbol_table.get(&ident.name) {
            self.emit_instruction(Instruction::LoadLocal(index));
        }
        // Luego buscar en funciones built-in
        else if let Some(&index) = self.builtin_functions.get(&ident.name) {
            self.emit_instruction(Instruction::LoadConst(index));
        }
        else {
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

        self.emit_instruction(instruction);
        Ok(())
    }

    /// Generar código para expresión unaria
    fn generate_unary_expression(&mut self, unary: &UnaryExpression) -> CompileResult<()> {
        self.generate_expression(&unary.operand)?;

        match unary.operator.as_str() {
            "-" => {
                // Negación: 0 - x
                let zero_idx = self.add_constant(vela_vm::Constant::Int(0));
                self.emit_instruction(Instruction::LoadConst(zero_idx));
                self.emit_instruction(Instruction::Sub);
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
        self.emit_instruction(Instruction::Call(call.arguments.len() as u8));
        Ok(())
    }

    /// Generar código para array literal
    fn generate_array_literal(&mut self, array: &ArrayLiteral) -> CompileResult<()> {
        // Generar elementos
        for element in &array.elements {
            self.generate_expression(element)?;
        }

        // Crear array
        self.emit_instruction(Instruction::BuildList(array.elements.len() as u16));
        Ok(())
    }

    /// Generar código para struct literal
    fn generate_struct_literal(&mut self, struct_lit: &StructLiteral) -> CompileResult<()> {
        // Generar valores de campos
        for field in &struct_lit.fields {
            self.generate_expression(&field.value)?;
        }

        // Generar nombres de campos como strings
        for field in &struct_lit.fields {
            let string_idx = self.add_string(field.name.clone());
            let const_idx = self.add_constant(vela_vm::Constant::String(string_idx));
            self.emit_instruction(Instruction::LoadConst(const_idx));
        }

        // Crear dict con los campos (clave + valor por cada campo)
        self.emit_instruction(Instruction::BuildDict(struct_lit.fields.len() as u16));
        Ok(())
    }

    /// Obtener el bytecode generado
    pub fn get_bytecode(&self) -> &Bytecode {
        &self.bytecode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_codegen_creation() {
        let generator = CodeGenerator::new();
        // Check that bytecode has expected initial state
        assert_eq!(generator.bytecode.magic, vela_vm::Bytecode::MAGIC);
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
        assert!(!generator.bytecode.code_objects[0].bytecode.is_empty());
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

        // Verificar que se agregó un code object
        assert!(!generator.bytecode.code_objects.is_empty());
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

        // Verificar que se generaron instrucciones en el code object actual
        assert!(!generator.bytecode.code_objects[generator.current_code_object].bytecode.is_empty());
    }

    #[test]
    fn test_generate_call_expression() {
        let mut generator = CodeGenerator::new();

        // Agregar una función built-in a la tabla de símbolos
        let func_name = "println";
        let const_index = generator.bytecode.add_string(func_name.to_string());
        generator.symbol_table.insert(func_name.to_string(), const_index);

        let callee = Expression::Identifier(Identifier::new(
            create_range(1, 1, 1, 7),
            func_name.to_string(),
        ));

        let args = vec![
            Expression::Literal(Literal::new(
                create_range(1, 9, 1, 13),
                serde_json::json!("hello"),
                "string".to_string(),
            )),
        ];

        let call = CallExpression::new(
            create_range(1, 1, 1, 15),
            callee,
            args,
        );

        let result = generator.generate_call_expression(&call);
        assert!(result.is_ok());

        // Verificar que se generaron instrucciones en el code object actual
        assert!(!generator.bytecode.code_objects[generator.current_code_object].bytecode.is_empty());
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
        assert!(!bytecode.code_objects.is_empty());
        assert!(bytecode.code_objects.len() >= 2); // Principal + función
        assert!(!bytecode.code_objects[1].bytecode.is_empty()); // La función debe tener bytecode
    }
}