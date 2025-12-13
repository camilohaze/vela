//! Expression Code Generation for JavaScript
//!
//! Handles generation of JavaScript code from Vela IR expressions.

use crate::ir::{IRExpr, Value, BinaryOp, UnaryOp};

/// Generate JavaScript code for expressions
pub struct ExpressionGenerator;

impl ExpressionGenerator {
    /// Generate JavaScript for a Vela IR expression
    pub fn generate(expr: &IRExpr) -> String {
        match expr {
            IRExpr::Var(name) => name.clone(),
            IRExpr::Const(value) => Self::generate_value(value),
            IRExpr::BinaryOp(op, left, right) => {
                let left_js = Self::generate(left);
                let right_js = Self::generate(right);
                let op_js = Self::binary_op_to_js(op);
                format!("({} {} {})", left_js, op_js, right_js)
            }
            IRExpr::UnaryOp(op, expr) => {
                let expr_js = Self::generate(expr);
                let op_js = Self::unary_op_to_js(op);
                format!("({}{})", op_js, expr_js)
            }
            IRExpr::Call { function, args } => {
                let args_js: Vec<String> = args.iter()
                    .map(|arg| Self::generate(arg))
                    .collect();
                format!("{}({})", function, args_js.join(", "))
            }
            IRExpr::ArrayAccess { array, index } => {
                let array_js = Self::generate(array);
                let index_js = Self::generate(index);
                format!("{}[{}]", array_js, index_js)
            }
            IRExpr::PropertyAccess { object, property } => {
                let object_js = Self::generate(object);
                format!("{}.{}", object_js, property)
            }
        }
    }

    /// Generate JavaScript for literal values
    fn generate_value(value: &Value) -> String {
        match value {
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => {
                let s = f.to_string();
                // Ensure floats have decimal point
                if s.contains('.') { s } else { format!("{}.0", s) }
            }
            Value::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
            Value::Null => "null".to_string(),
        }
    }

    /// Convert binary operator to JavaScript
    fn binary_op_to_js(op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "===",
            BinaryOp::Ne => "!==",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }

    /// Convert unary operator to JavaScript
    fn unary_op_to_js(op: &UnaryOp) -> &'static str {
        match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IRExpr, Value, BinaryOp, UnaryOp};

    #[test]
    fn test_generate_literal_values() {
        assert_eq!(ExpressionGenerator::generate_value(&Value::Bool(true)), "true");
        assert_eq!(ExpressionGenerator::generate_value(&Value::Int(42)), "42");
        assert_eq!(ExpressionGenerator::generate_value(&Value::Float(3.14)), "3.14");
        assert_eq!(ExpressionGenerator::generate_value(&Value::String("hello".to_string())), "\"hello\"");
        assert_eq!(ExpressionGenerator::generate_value(&Value::Null), "null");
    }

    #[test]
    fn test_generate_binary_operations() {
        let expr = IRExpr::BinaryOp(
            BinaryOp::Add,
            Box::new(IRExpr::Const(Value::Int(1))),
            Box::new(IRExpr::Const(Value::Int(2)))
        );
        assert_eq!(ExpressionGenerator::generate(&expr), "(1 + 2)");

        let expr = IRExpr::BinaryOp(
            BinaryOp::Eq,
            Box::new(IRExpr::Var("x".to_string())),
            Box::new(IRExpr::Const(Value::Int(0)))
        );
        assert_eq!(ExpressionGenerator::generate(&expr), "(x === 0)");
    }

    #[test]
    fn test_generate_unary_operations() {
        let expr = IRExpr::UnaryOp(
            UnaryOp::Not,
            Box::new(IRExpr::Var("flag".to_string()))
        );
        assert_eq!(ExpressionGenerator::generate(&expr), "(!flag)");

        let expr = IRExpr::UnaryOp(
            UnaryOp::Neg,
            Box::new(IRExpr::Const(Value::Int(5)))
        );
        assert_eq!(ExpressionGenerator::generate(&expr), "(-5)");
    }

    #[test]
    fn test_generate_function_calls() {
        let expr = IRExpr::Call {
            function: "add".to_string(),
            args: vec![
                IRExpr::Const(Value::Int(1)),
                IRExpr::Const(Value::Int(2))
            ]
        };
        assert_eq!(ExpressionGenerator::generate(&expr), "add(1, 2)");
    }

    #[test]
    fn test_generate_array_access() {
        let expr = IRExpr::ArrayAccess {
            array: Box::new(IRExpr::Var("arr".to_string())),
            index: Box::new(IRExpr::Const(Value::Int(0)))
        };
        assert_eq!(ExpressionGenerator::generate(&expr), "arr[0]");
    }

    #[test]
    fn test_generate_property_access() {
        let expr = IRExpr::PropertyAccess {
            object: Box::new(IRExpr::Var("obj".to_string())),
            property: "name".to_string()
        };
        assert_eq!(ExpressionGenerator::generate(&expr), "obj.name");
    }

    #[test]
    fn test_string_escaping() {
        let expr = IRExpr::Const(Value::String("hello \"world\"".to_string()));
        assert_eq!(ExpressionGenerator::generate(&expr), "\"hello \\\"world\\\"\"");
    }
}