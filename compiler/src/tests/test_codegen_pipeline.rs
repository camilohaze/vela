/*
Tests unitarios para el pipeline de generación de código AST → IR → Bytecode

Implementación de: TASK-070 (Bytecode generator desde IR)
Fecha: 2025-01-30

Estos tests validan que el pipeline completo funcione correctamente,
convirtiendo código Vela desde AST hasta bytecode ejecutable.
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::ir::*;
    use crate::bytecode::*;
    use crate::codegen::ast_to_ir::ASTToIRConverter;
    use crate::codegen::ir_to_bytecode::IRToBytecodeGenerator;
    use crate::codegen::ir_to_bytecode::IROptimizer;
    use crate::codegen::CodeGenerator;

    #[test]
    fn test_simple_function_conversion() {
        // Crear AST para función simple: fn add(a: int, b: int) -> int { return a + b; }
        let func = FunctionDecl {
            name: "add".to_string(),
            params: vec![
                Parameter {
                    name: "a".to_string(),
                    ty: Type::Int,
                    position: Position::new(1, 10),
                },
                Parameter {
                    name: "b".to_string(),
                    ty: Type::Int,
                    position: Position::new(1, 18),
                },
            ],
            return_type: Type::Int,
            body: vec![
                Statement::Return(Some(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                    position: Position::new(1, 35),
                })),
            ],
            position: Position::new(1, 1),
        };

        let program = Program {
            declarations: vec![Declaration::Function(func)],
        };

        // Convertir AST → IR
        let mut converter = ASTToIRConverter::new();
        let ir_module = converter.convert_program(&program).unwrap();

        // Verificar que se creó la función
        assert_eq!(ir_module.functions.len(), 1);
        let ir_function = &ir_module.functions[0];
        assert_eq!(ir_function.name, "add");
        assert_eq!(ir_function.params.len(), 2);
        assert!(matches!(ir_function.return_type, IRType::Int));

        // Verificar instrucciones generadas
        assert!(ir_function.instructions.len() > 0);
    }

    #[test]
    fn test_variable_declaration() {
        // Crear AST para: let x: int = 42;
        let var_decl = Statement::VariableDecl(VariableDecl {
            name: "x".to_string(),
            ty: Type::Int,
            initializer: Some(Expression::Literal(Literal::Integer(42))),
            position: Position::new(1, 1),
        });

        let program = Program {
            declarations: vec![],
        };

        let mut converter = ASTToIRConverter::new();
        // Para probar statements individuales, necesitamos una función wrapper
        let func = FunctionDecl {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![var_decl, Statement::Return(None)],
            position: Position::new(1, 1),
        };

        let test_program = Program {
            declarations: vec![Declaration::Function(func)],
        };

        let ir_module = converter.convert_program(&test_program).unwrap();
        let ir_function = &ir_module.functions[0];

        // Verificar que se declaró la variable y se inicializó
        assert!(ir_function.locals.iter().any(|local| local.name == "x"));
        assert!(ir_function.instructions.iter().any(|instr| matches!(instr, IRInstruction::DeclareVar { name, .. } if name == "x")));
    }

    #[test]
    fn test_if_statement() {
        // Crear AST para: if (x > 0) { return 1; } else { return 0; }
        let if_stmt = Statement::If(IfStatement {
            condition: Expression::BinaryOp {
                left: Box::new(Expression::Identifier("x".to_string())),
                op: BinaryOperator::Greater,
                right: Box::new(Expression::Literal(Literal::Integer(0))),
                position: Position::new(1, 5),
            },
            then_block: vec![Statement::Return(Some(Expression::Literal(Literal::Integer(1))))],
            else_block: Some(vec![Statement::Return(Some(Expression::Literal(Literal::Integer(0))))]),
            position: Position::new(1, 1),
        });

        let func = FunctionDecl {
            name: "test_if".to_string(),
            params: vec![Parameter {
                name: "x".to_string(),
                ty: Type::Int,
                position: Position::new(1, 13),
            }],
            return_type: Type::Int,
            body: vec![if_stmt],
            position: Position::new(1, 1),
        };

        let program = Program {
            declarations: vec![Declaration::Function(func)],
        };

        let mut converter = ASTToIRConverter::new();
        let ir_module = converter.convert_program(&program).unwrap();
        let ir_function = &ir_module.functions[0];

        // Verificar que se generaron labels y jumps
        let has_jump_if = ir_function.instructions.iter().any(|instr| matches!(instr, IRInstruction::JumpIf(_)));
        let has_labels = ir_function.instructions.iter().any(|instr| matches!(instr, IRInstruction::Label(_)));

        assert!(has_jump_if, "Debe haber JumpIf para condición");
        assert!(has_labels, "Deben haber labels para else/endif");
    }

    #[test]
    fn test_while_loop() {
        // Crear AST para: while (i < 10) { i = i + 1; }
        let while_stmt = Statement::While(WhileStatement {
            condition: Expression::BinaryOp {
                left: Box::new(Expression::Identifier("i".to_string())),
                op: BinaryOperator::Less,
                right: Box::new(Expression::Literal(Literal::Integer(10))),
                position: Position::new(1, 8),
            },
            body: vec![Statement::Assignment(Assignment {
                target: Expression::Identifier("i".to_string()),
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("i".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Literal(Literal::Integer(1))),
                    position: Position::new(1, 20),
                },
                position: Position::new(1, 15),
            })],
            position: Position::new(1, 1),
        });

        let func = FunctionDecl {
            name: "test_while".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![while_stmt, Statement::Return(None)],
            position: Position::new(1, 1),
        };

        let program = Program {
            declarations: vec![Declaration::Function(func)],
        };

        let mut converter = ASTToIRConverter::new();
        let ir_module = converter.convert_program(&program).unwrap();
        let ir_function = &ir_module.functions[0];

        // Verificar estructura del loop
        let jump_count = ir_function.instructions.iter().filter(|instr| matches!(instr, IRInstruction::Jump(_))).count();
        let jump_if_count = ir_function.instructions.iter().filter(|instr| matches!(instr, IRInstruction::JumpIf(_))).count();
        let label_count = ir_function.instructions.iter().filter(|instr| matches!(instr, IRInstruction::Label(_))).count();

        assert!(jump_count >= 1, "Debe haber al menos un jump para loop");
        assert!(jump_if_count >= 1, "Debe haber jump if para condición");
        assert!(label_count >= 2, "Deben haber labels para start/end");
    }

    #[test]
    fn test_ir_to_bytecode_conversion() {
        // Crear módulo IR simple
        let mut ir_function = IRFunction::new("test".to_string(), IRType::Void);
        ir_function.add_param("x".to_string(), IRType::Int);
        ir_function.add_instruction(IRInstruction::LoadVar("x".to_string()));
        ir_function.add_instruction(IRInstruction::Return);

        let mut ir_module = IRModule::new("test".to_string());
        ir_module.add_function(ir_function);

        // Convertir IR → Bytecode
        let mut generator = IRToBytecodeGenerator::new();
        let bytecode_program = generator.generate_module(&ir_module).unwrap();

        // Verificar que se generó bytecode
        assert_eq!(bytecode_program.functions.len(), 1);
        assert!(!bytecode_program.functions[0].code.is_empty());
    }

    #[test]
    fn test_constant_folding_optimization() {
        // Crear IR con constantes que se pueden plegar
        let mut ir_function = IRFunction::new("test".to_string(), IRType::Int);
        ir_function.add_instruction(IRInstruction::LoadConst(Value::Int(2)));
        ir_function.add_instruction(IRInstruction::LoadConst(Value::Int(3)));
        ir_function.add_instruction(IRInstruction::BinaryOp(BinaryOp::Add));
        ir_function.add_instruction(IRInstruction::Return);

        let mut ir_module = IRModule::new("test".to_string());
        ir_module.add_function(ir_function);

        // Aplicar optimizaciones
        let optimizer = IROptimizer::new();
        optimizer.optimize_module(&mut ir_module);

        // Verificar que no se eliminaron instrucciones válidas
        let func = &ir_module.functions[0];
        assert!(!func.instructions.is_empty());
    }

    #[test]
    fn test_complete_pipeline() {
        // Programa Vela simple: fn main() { return 42; }
        let func = FunctionDecl {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![Statement::Return(Some(Expression::Literal(Literal::Integer(42))))],
            position: Position::new(1, 1),
        };

        let program = Program {
            declarations: vec![Declaration::Function(func)],
        };

        // Ejecutar pipeline completo
        let mut codegen = CodeGenerator::new();
        let bytecode_program = codegen.generate_program(&program).unwrap();

        // Verificar resultado
        assert_eq!(bytecode_program.functions.len(), 1);
        assert_eq!(bytecode_program.functions[0].name, "main");
        assert!(!bytecode_program.functions[0].code.is_empty());
        assert!(!bytecode_program.constants.is_empty()); // Debe tener la constante 42
    }

    #[test]
    fn test_pipeline_with_optimization() {
        // Programa más complejo para probar optimizaciones
        let func = FunctionDecl {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![
                Statement::VariableDecl(VariableDecl {
                    name: "x".to_string(),
                    ty: Type::Int,
                    initializer: Some(Expression::Literal(Literal::Integer(10))),
                    position: Position::new(1, 5),
                }),
                Statement::VariableDecl(VariableDecl {
                    name: "y".to_string(),
                    ty: Type::Int,
                    initializer: Some(Expression::Literal(Literal::Integer(20))),
                    position: Position::new(1, 5),
                }),
                Statement::Return(Some(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("y".to_string())),
                    position: Position::new(1, 15),
                })),
            ],
            position: Position::new(1, 1),
        };

        let program = Program {
            declarations: vec![Declaration::Function(func)],
        };

        let mut codegen = CodeGenerator::new();
        let bytecode_opt = codegen.generate_program(&program).unwrap();
        let bytecode_no_opt = codegen.generate_program_no_opt(&program).unwrap();

        // Ambos deben generar código válido
        assert!(!bytecode_opt.functions[0].code.is_empty());
        assert!(!bytecode_no_opt.functions[0].code.is_empty());
    }

    #[test]
    fn test_error_handling() {
        // Programa con variable indefinida
        let func = FunctionDecl {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![Statement::Return(Some(Expression::Identifier("undefined_var".to_string())))],
            position: Position::new(1, 1),
        };

        let program = Program {
            declarations: vec![Declaration::Function(func)],
        };

        let mut codegen = CodeGenerator::new();
        let result = codegen.generate_program(&program);

        // Debe fallar con error de variable indefinida
        assert!(result.is_err());
    }
}