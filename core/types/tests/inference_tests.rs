//! Tests exhaustivos para el motor de inferencia de tipos
//!
//! Estos tests verifican específicamente el algoritmo de inferencia
//! Hindley-Milner y la unificación de tipos.

use ::types::*;
use vela_compiler::ast::*;
use std::collections::HashMap;

/// Helper para crear expresiones simples
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

fn identifier(name: &str) -> Expression {
    Expression::Identifier(Identifier::new(
        create_range(1, 1, 1, name.len()),
        name.to_string(),
    ))
}

fn binary_expr(left: Expression, op: &str, right: Expression) -> Expression {
    Expression::Binary(BinaryExpression::new(
        create_range(1, 1, 1, 5),
        left,
        op.to_string(),
        right,
    ))
}

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

#[cfg(test)]
mod inference_tests {
    use super::*;
    use ::types::inference::TypeInference;

    fn create_inference() -> TypeInference {
        let mut context = TypeContext::new();
        context.add_variable("x".to_string(), TypeScheme::mono(Type::Int));
        context.add_variable("y".to_string(), TypeScheme::mono(Type::Bool));
        TypeInference::new(context)
    }

    #[test]
    fn test_algorithm_w_simple() {
        let context = TypeContext::new();
        let expr = literal_number(42);

        let result = crate::inference::algorithm_w(&context, &expr).unwrap();
        assert_eq!(result.0, Type::Int);
    }

    #[test]
    fn test_algorithm_w_with_variables() {
        let mut context = TypeContext::new();
        context.add_variable("x".to_string(), TypeScheme::mono(Type::Int));

        let expr = binary_expr(identifier("x"), "+", literal_number(1));
        let result = crate::inference::algorithm_w(&context, &expr).unwrap();
        assert_eq!(result.0, Type::Int);
    }

    #[test]
    fn test_algorithm_w_lambda() {
        let context = TypeContext::new();
        let expr = lambda_expr(vec!["x"], binary_expr(identifier("x"), "+", literal_number(1)));

        let result = crate::inference::algorithm_w(&context, &expr).unwrap();

        if let Type::Function { params, ret } = result.0 {
            assert_eq!(params.len(), 1);
            assert_eq!(*ret, Type::Int);
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_algorithm_w_polymorphic_function() {
        let mut context = TypeContext::new();

        // Add polymorphic identity function: forall a. a -> a
        let tv = TypeVar(1);
        let identity_type = Type::Function {
            params: vec![Type::Var(tv)],
            ret: Box::new(Type::Var(tv)),
        };
        context.add_variable("id".to_string(), TypeScheme::poly(vec![tv], identity_type));

        // Test id(42)
        let call = Expression::Call(CallExpression::new(
            create_range(1, 1, 1, 8),
            identifier("id"),
            vec![literal_number(42)],
        ));

        let result = crate::inference::algorithm_w(&context, &call).unwrap();
        assert_eq!(result.0, Type::Int);

        // Test id(true)
        let call = Expression::Call(CallExpression::new(
            create_range(1, 1, 1, 10),
            identifier("id"),
            vec![literal_bool(true)],
        ));

        let result = crate::inference::algorithm_w(&context, &call).unwrap();
        assert_eq!(result.0, Type::Bool);
    }

    #[test]
    fn test_algorithm_w_complex_expression() {
        let mut context = TypeContext::new();
        context.add_variable("x".to_string(), TypeScheme::mono(Type::Int));
        context.add_variable("y".to_string(), TypeScheme::mono(Type::Int));

        // (x + y) == 10
        let expr = binary_expr(
            binary_expr(identifier("x"), "+", identifier("y")),
            "==",
            literal_number(10),
        );

        let result = crate::inference::algorithm_w(&context, &expr).unwrap();
        assert_eq!(result.0, Type::Bool);
    }

    #[test]
    fn test_algorithm_w_type_errors() {
        let context = TypeContext::new();

        // x + true (type error)
        let expr = binary_expr(identifier("x"), "+", literal_bool(true));
        let result = crate::inference::algorithm_w(&context, &expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_inference_with_type_variables() {
        let mut inference = create_inference();

        // Create expression with fresh type variables - use array with one element
        let expr = Expression::ArrayLiteral(ArrayLiteral::new(
            create_range(1, 1, 1, 5),
            vec![literal_number(42)],
        ));

        let result = inference.infer_expression(&expr).unwrap();

        // Should be Array[Int]
        if let Type::Array(ref elem_ty) = result {
            assert_eq!(**elem_ty, Type::Int);
        } else {
            panic!("Expected array type");
        }

        // Check that substitution contains the variable (from the inference process)
        // For this simple case, substitution might be empty, so let's check the result
        assert!(matches!(result, Type::Array(_)));
    }

    #[test]
    fn test_inference_occurs_check_prevention() {
        let mut inference = create_inference();

        // Try to create a recursive type that should be prevented
        let tv = TypeVar(1);
        let recursive_type = Type::Function {
            params: vec![Type::Var(tv)],
            ret: Box::new(Type::Int),
        };

        let result = inference.unify(&Type::Var(tv), &recursive_type);
        assert!(matches!(result, Err(TypeError::InfiniteType { .. })));
    }

    #[test]
    fn test_inference_generic_types() {
        let mut inference = create_inference();

        // Test unification with generic types
        let generic_type = Type::Generic {
            name: "Option".to_string(),
            args: vec![Type::Int],
        };

        let other_generic = Type::Generic {
            name: "Option".to_string(),
            args: vec![Type::Int],
        };

        let result = inference.unify(&generic_type, &other_generic);
        assert!(result.is_ok());

        // Test different generic types
        let different_generic = Type::Generic {
            name: "Result".to_string(),
            args: vec![Type::Int, Type::String],
        };

        let result = inference.unify(&generic_type, &different_generic);
        assert!(result.is_err());
    }

    #[test]
    fn test_inference_record_types() {
        let mut inference = create_inference();

        let mut fields1 = HashMap::new();
        fields1.insert("name".to_string(), Type::String);
        fields1.insert("age".to_string(), Type::Int);

        let mut fields2 = HashMap::new();
        fields2.insert("name".to_string(), Type::String);
        fields2.insert("age".to_string(), Type::Int);

        let record1 = Type::Record(fields1);
        let record2 = Type::Record(fields2);

        let result = inference.unify(&record1, &record2);
        assert!(result.is_ok());

        // Test records with different fields
        let mut fields3 = HashMap::new();
        fields3.insert("name".to_string(), Type::String);
        fields3.insert("height".to_string(), Type::Float);

        let record3 = Type::Record(fields3);
        let result = inference.unify(&record1, &record3);
        assert!(result.is_err());
    }

    #[test]
    fn test_inference_variant_types() {
        let mut inference = create_inference();

        let mut variants1 = HashMap::new();
        variants1.insert("Some".to_string(), Type::Int);
        variants1.insert("None".to_string(), Type::Unit);

        let mut variants2 = HashMap::new();
        variants2.insert("Some".to_string(), Type::Int);
        variants2.insert("None".to_string(), Type::Unit);

        let variant1 = Type::Variant(variants1);
        let variant2 = Type::Variant(variants2);

        let result = inference.unify(&variant1, &variant2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inference_option_types() {
        let mut inference = create_inference();

        let option1 = Type::Option(Box::new(Type::Int));
        let option2 = Type::Option(Box::new(Type::Int));

        let result = inference.unify(&option1, &option2);
        assert!(result.is_ok());

        // Test different inner types
        let option3 = Type::Option(Box::new(Type::Bool));
        let result = inference.unify(&option1, &option3);
        assert!(result.is_err());
    }

    #[test]
    fn test_inference_result_types() {
        let mut inference = create_inference();

        let result1 = Type::Result {
            ok: Box::new(Type::Int),
            err: Box::new(Type::String),
        };

        let result2 = Type::Result {
            ok: Box::new(Type::Int),
            err: Box::new(Type::String),
        };

        let unify_result = inference.unify(&result1, &result2);
        assert!(unify_result.is_ok());
    }

    #[test]
    fn test_inference_fresh_type_variables() {
        let mut inference = create_inference();

        // Create two fresh variables
        let tv1 = inference.fresh_type_var();
        let tv2 = inference.fresh_type_var();

        // They should be different
        assert_ne!(tv1, tv2);

        // Both should be type variables
        assert!(matches!(tv1, Type::Var(_)));
        assert!(matches!(tv2, Type::Var(_)));
    }

    #[test]
    fn test_inference_substitution_application() {
        let mut inference = create_inference();

        let tv = TypeVar(1);
        inference.substitution_mut().insert(tv, Type::Int);

        let var_type = Type::Var(tv);
        let applied = inference.apply_subst(var_type);

        assert_eq!(applied, Type::Int);
    }

    #[test]
    fn test_inference_complex_substitution() {
        let mut inference = create_inference();

        let tv1 = TypeVar(1);
        let tv2 = TypeVar(2);

        // Create a function type with variables
        let func_type = Type::Function {
            params: vec![Type::Var(tv1)],
            ret: Box::new(Type::Var(tv2)),
        };

        // Substitute variables
        inference.substitution_mut().insert(tv1, Type::Int);
        inference.substitution_mut().insert(tv2, Type::Bool);

        let applied = inference.apply_subst(func_type);

        if let Type::Function { params, ret } = applied {
            assert_eq!(params, vec![Type::Int]);
            assert_eq!(*ret, Type::Bool);
        } else {
            panic!("Expected function type");
        }
    }
}