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
    use crate::ir::{IRExpr, Value as IRValue, BinaryOp, UnaryOp};
    use crate::codegen::ir_to_bytecode::IROptimizer;
    use std::collections::HashMap;

    fn create_position(line: usize, column: usize) -> Range {
        Range::new(
            Position::new(line, column),
            Position::new(line, column + 1)
        )
    }

    fn create_ir_const_number(value: i64) -> IRExpr {
        IRExpr::Const(IRValue::Int(value))
    }

    fn create_ir_const_float(value: f64) -> IRExpr {
        IRExpr::Const(IRValue::Float(value))
    }

    fn create_ir_const_bool(value: bool) -> IRExpr {
        IRExpr::Const(IRValue::Bool(value))
    }

    fn create_ir_const_string(value: &str) -> IRExpr {
        IRExpr::Const(IRValue::String(value.to_string()))
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
            Some(IRValue::Int(14)) => assert!(true),
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
            Some(IRValue::Bool(false)) => assert!(true),
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
            Some(IRValue::String(s)) if s == "Hello World" => assert!(true),
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
            Some(IRValue::Int(5)) => assert!(true),
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
            IRExpr::Const(IRValue::Int(0)) => assert!(true),
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
            Some(IRValue::Float(f)) if (f - 6.28).abs() < 0.001 => assert!(true),
            _ => panic!("Expected Some(Float(~6.28)), got {:?}", optimized),
        }
    }
}