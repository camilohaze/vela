//! Tests exhaustivos para el Type Checker de Vela
//!
//! Estos tests verifican la funcionalidad completa del type checker,
//! incluyendo inferencia de tipos, unificación, y verificación de expresiones.

use ::types::*;
use vela_compiler::ast::*;
use std::collections::HashMap;

/// Helper para crear un type checker con contexto básico
fn create_type_checker() -> TypeChecker {
    let mut context = TypeContext::new();

    // Agregar tipos básicos al contexto
    context.add_variable("x".to_string(), TypeScheme::mono(Type::Int));
    context.add_variable("y".to_string(), TypeScheme::mono(Type::Bool));
    context.add_variable("z".to_string(), TypeScheme::mono(Type::String));

    TypeChecker::with_context(context)
}

/// Helper para crear expresiones literales
fn literal_number(value: i64) -> Expression {
    Expression::Literal(Literal::new(
        create_range(1, 1, 1, 3),
        serde_json::json!(value),
        "number".to_string(),
    ))
}

fn literal_bool(value: bool) -> Expression {
    Expression::Literal(Literal::new(
        create_range(1, 1, 1, 5),
        serde_json::json!(value),
        "bool".to_string(),
    ))
}

fn literal_string(value: &str) -> Expression {
    Expression::Literal(Literal::new(
        create_range(1, 1, 1, (value.len() + 2) as usize),
        serde_json::json!(value),
        "string".to_string(),
    ))
}

/// Helper para crear identificadores
fn identifier(name: &str) -> Expression {
    Expression::Identifier(Identifier::new(
        create_range(1, 1, 1, name.len()),
        name.to_string(),
    ))
}

/// Helper para crear expresiones binarias
fn binary_expr(left: Expression, op: &str, right: Expression) -> Expression {
    Expression::Binary(BinaryExpression::new(
        create_range(1, 1, 1, 5),
        left,
        op.to_string(),
        right,
    ))
}

/// Helper para crear expresiones unarias
fn unary_expr(op: &str, operand: Expression) -> Expression {
    Expression::Unary(UnaryExpression::new(
        create_range(1, 1, 1, 3),
        op.to_string(),
        operand,
    ))
}

/// Helper para crear llamadas a función
fn call_expr(callee: Expression, args: Vec<Expression>) -> Expression {
    Expression::Call(CallExpression::new(
        create_range(1, 1, 1, 10),
        callee,
        args,
    ))
}

/// Helper para crear acceso a miembros
fn member_access(object: Expression, member: &str) -> Expression {
    Expression::MemberAccess(MemberAccessExpression::new(
        create_range(1, 1, 1, (member.len() + 2) as usize),
        object,
        member.to_string(),
        false,
    ))
}

/// Helper para crear arrays
fn array_literal(elements: Vec<Expression>) -> Expression {
    Expression::ArrayLiteral(ArrayLiteral::new(
        create_range(1, 1, 1, 10),
        elements,
    ))
}

/// Helper para crear tuples
fn tuple_literal(elements: Vec<Expression>) -> Expression {
    Expression::TupleLiteral(TupleLiteral::new(
        create_range(1, 1, 1, 10),
        elements,
    ))
}

/// Helper para crear structs
fn struct_literal(name: &str, fields: Vec<(&str, Expression)>) -> Expression {
    let fields = fields.into_iter()
        .map(|(name, value)| StructLiteralField::new(
            name.to_string(),
            value,
            create_range(1, 1, 1, 10),
        ))
        .collect();

    Expression::StructLiteral(StructLiteral::new(
        create_range(1, 1, 1, 20),
        name.to_string(),
        fields,
    ))
}

/// Helper para crear lambdas
fn lambda_expr(params: Vec<&str>, body: Expression) -> Expression {
    let parameters = params.into_iter()
        .map(|name| Parameter::new(
            name.to_string(),
            None,
            None,
            create_range(1, 1, 1, name.len()),
        ))
        .collect();

    Expression::Lambda(LambdaExpression::new(
        create_range(1, 1, 1, 15),
        parameters,
        LambdaBody::Expression(Box::new(body)),
    ))
}

/// Helper para crear if expressions
fn if_expr(condition: Expression, then_branch: Expression, else_branch: Expression) -> Expression {
    Expression::If(IfExpression::new(
        create_range(1, 1, 1, 25),
        condition,
        then_branch,
        else_branch,
    ))
}

#[cfg(test)]
mod type_checker_tests {
    use super::*;

    #[test]
    fn test_literal_inference() {
        let mut checker = create_type_checker();

        // Test number literal
        let num_lit = literal_number(42);
        let result = checker.check_expression(&num_lit).unwrap();
        assert_eq!(result.ty, Type::Int);

        // Test bool literal
        let bool_lit = literal_bool(true);
        let result = checker.check_expression(&bool_lit).unwrap();
        assert_eq!(result.ty, Type::Bool);

        // Test string literal
        let str_lit = literal_string("hello");
        let result = checker.check_expression(&str_lit).unwrap();
        assert_eq!(result.ty, Type::String);
    }

    #[test]
    fn test_identifier_lookup() {
        let mut checker = create_type_checker();

        // Test existing identifier
        let ident = identifier("x");
        let result = checker.check_expression(&ident).unwrap();
        assert_eq!(result.ty, Type::Int);

        // Test non-existing identifier
        let ident = identifier("nonexistent");
        let result = checker.check_expression(&ident);
        assert!(matches!(result, Err(TypeError::VariableNotFound { .. })));
    }

    #[test]
    fn test_binary_operations() {
        let mut checker = create_type_checker();

        // Test arithmetic operations
        let expr = binary_expr(literal_number(1), "+", literal_number(2));
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Int);

        // Test comparison operations
        let expr = binary_expr(literal_number(1), "==", literal_number(2));
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Bool);

        // Test logical operations
        let expr = binary_expr(literal_bool(true), "&&", literal_bool(false));
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Bool);

        // Test type mismatch
        let expr = binary_expr(literal_number(1), "+", literal_bool(true));
        let result = checker.check_expression(&expr);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));
    }

    #[test]
    fn test_unary_operations() {
        let mut checker = create_type_checker();

        // Test negation
        let expr = unary_expr("-", literal_number(5));
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Int);

        // Test logical not
        let expr = unary_expr("!", literal_bool(true));
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Bool);

        // Test type mismatch
        let expr = unary_expr("-", literal_bool(true));
        let result = checker.check_expression(&expr);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));
    }

    #[test]
    fn test_array_literals() {
        let mut checker = create_type_checker();

        // Test homogeneous array
        let expr = array_literal(vec![
            literal_number(1),
            literal_number(2),
            literal_number(3),
        ]);
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Array(Box::new(Type::Int)));

        // Test empty array
        let expr = array_literal(vec![]);
        let result = checker.check_expression(&expr).unwrap();
        // Should be Array with fresh type variable
        assert!(matches!(result.ty, Type::Array(_)));

        // Test heterogeneous array (should fail)
        let expr = array_literal(vec![
            literal_number(1),
            literal_bool(true),
        ]);
        let result = checker.check_expression(&expr);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));
    }

    #[test]
    fn test_tuple_literals() {
        let mut checker = create_type_checker();

        // Test tuple
        let expr = tuple_literal(vec![
            literal_number(1),
            literal_bool(true),
            literal_string("hello"),
        ]);
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Tuple(vec![
            Type::Int,
            Type::Bool,
            Type::String,
        ]));
    }

    #[test]
    fn test_lambda_expressions() {
        let mut checker = create_type_checker();

        // Test simple lambda
        let expr = lambda_expr(vec!["x"], binary_expr(identifier("x"), "+", literal_number(1)));
        let result = checker.check_expression(&expr).unwrap();

        // Should be a function type
        assert!(matches!(result.ty, Type::Function { .. }));

        if let Type::Function { params, ret } = result.ty {
            assert_eq!(params.len(), 1);
            assert_eq!(*ret, Type::Int);
        }
    }

    #[test]
    fn test_if_expressions() {
        let mut checker = create_type_checker();

        // Test if with same return types
        let expr = if_expr(
            literal_bool(true),
            literal_number(1),
            literal_number(2),
        );
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Int);

        // Test if with different return types (should fail)
        let expr = if_expr(
            literal_bool(true),
            literal_number(1),
            literal_bool(false),
        );
        let result = checker.check_expression(&expr);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));

        // Test if with non-bool condition (should fail)
        let expr = if_expr(
            literal_number(1),
            literal_number(2),
            literal_number(3),
        );
        let result = checker.check_expression(&expr);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));
    }

    #[test]
    fn test_member_access() {
        let mut checker = create_type_checker();

        // Create a record type in context
        let mut record_fields = HashMap::new();
        record_fields.insert("name".to_string(), Type::String);
        record_fields.insert("age".to_string(), Type::Int);

        let record_type = Type::Record(record_fields);
        checker.context_mut().add_variable("person".to_string(), TypeScheme::mono(record_type));

        // Test valid member access
        let expr = member_access(identifier("person"), "name");
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::String);

        // Test invalid member access
        let expr = member_access(identifier("person"), "invalid_field");
        let result = checker.check_expression(&expr);
        assert!(matches!(result, Err(TypeError::FieldNotFound { .. })));
    }

    #[test]
    fn test_function_calls() {
        let mut checker = create_type_checker();

        // Add a function to context
        let func_type = Type::Function {
            params: vec![Type::Int, Type::Bool],
            ret: Box::new(Type::String),
        };
        checker.context_mut().add_variable("my_func".to_string(), TypeScheme::mono(func_type));

        // Test valid function call
        let expr = call_expr(
            identifier("my_func"),
            vec![literal_number(42), literal_bool(true)],
        );
        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::String);

        // Test wrong number of arguments
        let expr = call_expr(
            identifier("my_func"),
            vec![literal_number(42)],
        );
        let result = checker.check_expression(&expr);
        assert!(matches!(result, Err(TypeError::WrongNumberOfArguments { .. })));

        // Test wrong argument types
        let expr = call_expr(
            identifier("my_func"),
            vec![literal_bool(true), literal_number(42)],
        );
        let result = checker.check_expression(&expr);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));
    }

    #[test]
    fn test_complex_expressions() {
        let mut checker = create_type_checker();

        // Test complex expression: (x + 1) == 5 && y
        let expr = binary_expr(
            binary_expr(
                binary_expr(identifier("x"), "+", literal_number(1)),
                "==",
                literal_number(5),
            ),
            "&&",
            identifier("y"),
        );

        let result = checker.check_expression(&expr).unwrap();
        assert_eq!(result.ty, Type::Bool);
    }

    #[test]
    fn test_type_variable_substitution() {
        let mut checker = create_type_checker();

        // Test that empty array literals have generic array type
        let expr = array_literal(vec![]);
        let result = checker.check_expression(&expr).unwrap();

        // Empty array should have type Array(Var(_))
        match result.ty {
            Type::Array(elem_ty) => {
                match *elem_ty {
                    Type::Var(_) => {} // Correct - generic element type
                    _ => panic!("Expected generic element type for empty array"),
                }
            }
            _ => panic!("Expected Array type for empty array literal"),
        }
    }

    #[test]
    fn test_polymorphic_types() {
        let mut checker = create_type_checker();

        // Add a polymorphic identity function
        let tv = TypeVar(1);
        let identity_type = Type::Function {
            params: vec![Type::Var(tv)],
            ret: Box::new(Type::Var(tv)),
        };
        checker.context_mut().add_variable("id".to_string(), TypeScheme::poly(vec![tv], identity_type.clone()));

        // Test instantiation with different types
        let call1 = call_expr(identifier("id"), vec![literal_number(42)]);
        let result1 = checker.check_expression(&call1).unwrap();
        assert_eq!(result1.ty, Type::Int);

        // Reset checker for second test
        let mut checker = create_type_checker();
        checker.context_mut().add_variable("id".to_string(), TypeScheme::poly(vec![tv], identity_type.clone()));

        let call2 = call_expr(identifier("id"), vec![literal_bool(true)]);
        let result2 = checker.check_expression(&call2).unwrap();
        assert_eq!(result2.ty, Type::Bool);
    }
}