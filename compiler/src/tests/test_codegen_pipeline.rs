/*
Tests unitarios para el pipeline de generación de código AST → IR → Bytecode

Implementación de: TASK-171 (Constant folding optimizations)
Historia: VELA-1184 (Performance optimizations)
Fecha: 2025-01-30

Estos tests validan las optimizaciones de constant folding en el pipeline de IR.
*/

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::ast::*;
    use crate::ir::{IRExpr, Value, BinaryOp, UnaryOp, IRInstruction, IRFunction, IRType};
    use crate::codegen::ir_to_bytecode::IROptimizer;
    use std::collections::HashMap;

    fn create_position(line: usize, column: usize) -> Range {
        Range::new(
            Position::new(line, column),
            Position::new(line, column + 1)
        )
    }

    fn create_ir_const_number(value: i64) -> IRExpr {
        IRExpr::Const(Value::Int(value))
    }

    fn create_ir_const_float(value: f64) -> IRExpr {
        IRExpr::Const(Value::Float(value))
    }

    fn create_ir_const_bool(value: bool) -> IRExpr {
        IRExpr::Const(Value::Bool(value))
    }

    fn create_ir_const_string(value: &str) -> IRExpr {
        IRExpr::Const(Value::String(value.to_string()))
    }

    fn create_ir_binary_expr(left: IRExpr, op: BinaryOp, right: IRExpr) -> IRExpr {
        IRExpr::BinaryOp(op, Box::new(left), Box::new(right))
    }

    fn create_ir_call_expr(function: &str, args: Vec<IRExpr>) -> IRExpr {
        IRExpr::Call {
            function: function.to_string(),
            args: args,
        }
    }

    fn create_ir_identifier(name: &str) -> IRExpr {
        IRExpr::Var(name.to_string())
    }

    fn create_ir_binary_expr_from_str(left: IRExpr, op: &str, right: IRExpr) -> IRExpr {
        let binary_op = match op {
            "+" => BinaryOp::Add,
            "-" => BinaryOp::Sub,
            "*" => BinaryOp::Mul,
            "/" => BinaryOp::Div,
            "%" => BinaryOp::Mod,
            "==" => BinaryOp::Eq,
            "!=" => BinaryOp::Ne,
            "<" => BinaryOp::Lt,
            "<=" => BinaryOp::Le,
            ">" => BinaryOp::Gt,
            ">=" => BinaryOp::Ge,
            "&&" => BinaryOp::And,
            "||" => BinaryOp::Or,
            _ => panic!("Unsupported binary operator: {}", op),
        };
        IRExpr::BinaryOp(binary_op, Box::new(left), Box::new(right))
    }

    #[test]
    fn test_constant_folding_arithmetic() {
        // Test: 2 + 3 * 4 should become 2 + 12 = 14
        let expr = create_ir_binary_expr_from_str(
            create_ir_const_number(2),
            "+",
            create_ir_binary_expr_from_str(
                create_ir_const_number(3),
                "*",
                create_ir_const_number(4),
                ),
        );

        let optimizer = IROptimizer::new();
        let optimized = optimizer.evaluate_constant_expr(&expr);

        match optimized {
            Some(Value::Int(14)) => assert!(true),
            _ => panic!("Expected Some(Int(14)), got {:?}", optimized),
        }
    }

    #[test]
    fn test_constant_folding_boolean_expressions() {
        // Test: true && false should become false
        let expr = create_ir_binary_expr_from_str(
            create_ir_const_bool(true),
            "&&",
            create_ir_const_bool(false),
        );

        let optimizer = IROptimizer::new();
        let optimized = optimizer.evaluate_constant_expr(&expr);

        match optimized {
            Some(Value::Bool(false)) => assert!(true),
            _ => panic!("Expected Some(Bool(false)), got {:?}", optimized),
        }
    }

    #[test]
    fn test_constant_folding_string_operations() {
        // Test: "Hello" + " " + "World" should become "Hello World"
        let expr = create_ir_binary_expr_from_str(
            create_ir_binary_expr_from_str(
                create_ir_const_string("Hello"),
                "+",
                create_ir_const_string(" "),
            ),
            "+",
            create_ir_const_string("World"),
        );

        let optimizer = IROptimizer::new();
        let optimized = optimizer.evaluate_constant_expr(&expr);

        match optimized {
            Some(Value::String(s)) if s == "Hello World" => assert!(true),
            _ => panic!("Expected Some(String(\"Hello World\")), got {:?}", optimized),
        }
    }

    #[test]
    fn test_constant_folding_pure_function_calls() {
        // Test: len("hello") should become 5
        let expr = create_ir_call_expr(
            "len",
            vec![create_ir_const_string("hello")],
        );

        let optimizer = IROptimizer::new();
        let optimized = optimizer.evaluate_constant_expr(&expr);

        match optimized {
            Some(Value::Int(5)) => assert!(true),
            _ => panic!("Expected Some(Int(5)), got {:?}", optimized),
        }
    }

    #[test]
    fn test_algebraic_simplifications() {
        // Test: x * 0 should become 0 (identity operations)
        let mut expr = create_ir_binary_expr_from_str(
            create_ir_identifier("x"),
            "*",
            create_ir_const_number(0),
        );

        let optimizer = IROptimizer::new();
        optimizer.simplify_expr(&mut expr);

        match expr {
            IRExpr::Const(Value::Int(0)) => assert!(true),
            _ => panic!("Expected Const(Int(0)), got {:?}", expr),
        }
    }

    #[test]
    fn test_constant_folding_floats() {
        // Test: 3.14 * 2.0 should become 6.28
        let expr = create_ir_binary_expr_from_str(
            create_ir_const_float(3.14),
            "*",
            create_ir_const_float(2.0),
        );

        let optimizer = IROptimizer::new();
        let optimized = optimizer.evaluate_constant_expr(&expr);

        match optimized {
            Some(Value::Float(f)) if (f - 6.28).abs() < 0.001 => assert!(true),
            _ => panic!("Expected Some(Float(~6.28)), got {:?}", optimized),
        }
    }

    #[test]
    fn test_dead_code_elimination_after_return() {
        let mut function = IRFunction::new("test".to_string(), IRType::Void);

        // Agregar instrucciones: LoadConst(42), Return, LoadConst(99) (dead)
        function.body.push(IRInstruction::LoadConst(Value::Int(42)));
        function.body.push(IRInstruction::Return);
        function.body.push(IRInstruction::LoadConst(Value::Int(99))); // Esta debería ser eliminada

        let optimizer = IROptimizer::new();
        optimizer.dead_code_elimination(&mut function);

        // Debería quedar solo LoadConst(42) y Return
        assert_eq!(function.body.len(), 2);
        assert!(matches!(function.body[0], IRInstruction::LoadConst(Value::Int(42))));
        assert!(matches!(function.body[1], IRInstruction::Return));
    }

    #[test]
    fn test_dead_code_elimination_unused_variables() {
        let mut function = IRFunction::new("test".to_string(), IRType::Void);

        // Agregar instrucciones: asignar variable no usada, luego return
        function.body.push(IRInstruction::LoadConst(Value::Int(42)));
        function.body.push(IRInstruction::AssignVar {
            name: "unused_var".to_string(),
            value: IRExpr::Const(Value::Int(42)),
        });
        function.body.push(IRInstruction::LoadConst(Value::Int(0)));
        function.body.push(IRInstruction::Return);

        let optimizer = IROptimizer::new();
        optimizer.dead_code_elimination(&mut function);

        // La asignación a unused_var debería ser eliminada
        assert_eq!(function.body.len(), 2);
        assert!(matches!(function.body[0], IRInstruction::LoadConst(Value::Int(0))));
        assert!(matches!(function.body[1], IRInstruction::Return));
    }

    #[test]
    fn test_dead_code_elimination_used_variables() {
        let mut function = IRFunction::new("test".to_string(), IRType::Void);

        // Agregar instrucciones: asignar variable usada
        function.body.push(IRInstruction::LoadConst(Value::Int(42)));
        function.body.push(IRInstruction::AssignVar {
            name: "used_var".to_string(),
            value: IRExpr::Const(Value::Int(42)),
        });
        function.body.push(IRInstruction::LoadVar("used_var".to_string()));
        function.body.push(IRInstruction::Return);

        let optimizer = IROptimizer::new();
        optimizer.dead_code_elimination(&mut function);

        // La asignación debería conservarse porque la variable se usa
        assert_eq!(function.body.len(), 4);
        assert!(matches!(function.body[1], IRInstruction::AssignVar { .. }));
    }

    #[test]
    fn test_dead_code_elimination_no_changes() {
        let mut function = IRFunction::new("test".to_string(), IRType::Void);

        // Función simple sin código dead
        function.body.push(IRInstruction::LoadConst(Value::Int(42)));
        function.body.push(IRInstruction::Return);

        let original_len = function.body.len();
        let optimizer = IROptimizer::new();
        optimizer.dead_code_elimination(&mut function);

        // No debería cambiar nada
        assert_eq!(function.body.len(), original_len);
    }
}