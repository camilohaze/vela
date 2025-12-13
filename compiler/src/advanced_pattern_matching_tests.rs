/*
Tests exhaustivos de pattern matching avanzado

Implementación de: TASK-117F (Tests de pattern matching avanzado)
Historia: VELA-1099 (Pattern Matching Avanzado)
Fecha: 2025-12-13

Estos tests cubren exhaustivamente todas las features de pattern matching avanzado:
- Destructuring avanzado (arrays, structs, tuples con spread)
- Or patterns con operador |
- Range patterns con .. y ..=
- Patterns en lambdas
- Combinaciones complejas y casos edge
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    // ===================================================================
    // DESTRUCTURING AVANZADO TESTS
    // ===================================================================

    #[test]
    fn test_array_pattern_with_spread() {
        // Test que ArrayPattern con spread funciona
        let elements = vec![
            crate::ast::ArrayPatternElement::Pattern(Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 2), Position::new(1, 6)),
                "first".to_string()
            ))),
            crate::ast::ArrayPatternElement::Rest(Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 12), Position::new(1, 15)),
                "rest".to_string()
            ))),
        ];

        let array_pattern = ArrayPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 16)),
            elements,
        );

        assert_eq!(array_pattern.elements.len(), 2);
        assert!(matches!(array_pattern.elements[1], crate::ast::ArrayPatternElement::Rest(_)));
    }

    #[test]
    fn test_struct_pattern_with_rest() {
        // Test que StructPattern con rest funciona
        let fields = vec![
            StructPatternField::new(
                "name".to_string(),
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 2), Position::new(1, 5)),
                    "name".to_string()
                )),
                Range::new(Position::new(1, 2), Position::new(1, 5)),
            ),
            StructPatternField::new(
                "age".to_string(),
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 8), Position::new(1, 10)),
                    "age".to_string()
                )),
                Range::new(Position::new(1, 8), Position::new(1, 10)),
            ),
        ];

        let struct_pattern = StructPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 20)),
            "User".to_string(),
            fields,
            true, // has_rest
        );

        assert_eq!(struct_pattern.fields.len(), 2);
        assert_eq!(struct_pattern.struct_name, "User");
        assert!(struct_pattern.has_rest);
    }

    #[test]
    fn test_tuple_pattern_with_spread() {
        // Test que TuplePattern con spread funciona
        let elements = vec![
            Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 2), Position::new(1, 3)),
                "x".to_string()
            )),
            Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 5), Position::new(1, 6)),
                "y".to_string()
            )),
            Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 12), Position::new(1, 15)),
                "rest".to_string()
            )),
        ];

        let tuple_pattern = TuplePattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 16)),
            elements,
        );

        assert_eq!(tuple_pattern.elements.len(), 3);
    }

    #[test]
    fn test_nested_destructuring() {
        // Test destructuring anidado: { user: { name, age }, items: [first, ...rest] }
        let user_struct_fields = vec![
            StructPatternField::new(
                "name".to_string(),
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 10), Position::new(1, 13)),
                    "name".to_string()
                )),
                Range::new(Position::new(1, 10), Position::new(1, 13)),
            ),
            StructPatternField::new(
                "age".to_string(),
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 16), Position::new(1, 18)),
                    "age".to_string()
                )),
                Range::new(Position::new(1, 16), Position::new(1, 18)),
            ),
        ];

        let user_pattern = Pattern::Struct(StructPattern::new(
            Range::new(Position::new(1, 9), Position::new(1, 19)),
            "User".to_string(),
            user_struct_fields,
            false,
        ));

        let items_array_elements = vec![
            crate::ast::ArrayPatternElement::Pattern(Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 29), Position::new(1, 33)),
                "first".to_string()
            ))),
            crate::ast::ArrayPatternElement::Rest(Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 38), Position::new(1, 41)),
                "rest".to_string()
            ))),
        ];

        let items_pattern = Pattern::Array(ArrayPattern::new(
            Range::new(Position::new(1, 28), Position::new(1, 42)),
            items_array_elements,
        ));

        let outer_fields = vec![
            StructPatternField::new(
                "user".to_string(),
                user_pattern,
                Range::new(Position::new(1, 3), Position::new(1, 20)),
            ),
            StructPatternField::new(
                "items".to_string(),
                items_pattern,
                Range::new(Position::new(1, 23), Position::new(1, 43)),
            ),
        ];

        let nested_pattern = StructPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 44)),
            "Data".to_string(),
            outer_fields,
            false,
        );

        assert_eq!(nested_pattern.fields.len(), 2);
        assert_eq!(nested_pattern.struct_name, "Data");
    }

    // ===================================================================
    // OR PATTERNS TESTS
    // ===================================================================

    #[test]
    fn test_or_pattern_simple() {
        // Test pattern: Ok(value) | Err(error)
        let ok_pattern = Pattern::Enum(EnumPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 10)),
            "Ok".to_string(),
            Some(vec![Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 4), Position::new(1, 8)),
                "value".to_string()
            ))]),
        ));

        let err_pattern = Pattern::Enum(EnumPattern::new(
            Range::new(Position::new(1, 14), Position::new(1, 23)),
            "Err".to_string(),
            Some(vec![Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 18), Position::new(1, 22)),
                "error".to_string()
            ))]),
        ));

        let or_pattern = OrPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 23)),
            vec![ok_pattern, err_pattern],
        );

        assert_eq!(or_pattern.patterns.len(), 2);
        assert!(matches!(or_pattern.patterns[0], Pattern::Enum(_)));
        assert!(matches!(or_pattern.patterns[1], Pattern::Enum(_)));
    }

    #[test]
    fn test_or_pattern_literals() {
        // Test pattern: 1 | 2 | 3
        let patterns = vec![
            Pattern::Literal(LiteralPattern::new(
                Range::new(Position::new(1, 1), Position::new(1, 1)),
                serde_json::json!(1)
            )),
            Pattern::Literal(LiteralPattern::new(
                Range::new(Position::new(1, 5), Position::new(1, 5)),
                serde_json::json!(2)
            )),
            Pattern::Literal(LiteralPattern::new(
                Range::new(Position::new(1, 9), Position::new(1, 9)),
                serde_json::json!(3)
            )),
        ];

        let or_pattern = OrPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 9)),
            patterns,
        );

        assert_eq!(or_pattern.patterns.len(), 3);
        for pattern in &or_pattern.patterns {
            assert!(matches!(pattern, Pattern::Literal(_)));
        }
    }

    #[test]
    fn test_or_pattern_mixed() {
        // Test pattern: "admin" | "user" | Some(role)
        let string_patterns = vec![
            Pattern::Literal(LiteralPattern::new(
                Range::new(Position::new(1, 1), Position::new(1, 6)),
                serde_json::json!("admin")
            )),
            Pattern::Literal(LiteralPattern::new(
                Range::new(Position::new(1, 10), Position::new(1, 14)),
                serde_json::json!("user")
            )),
        ];

        let some_pattern = Pattern::Enum(EnumPattern::new(
            Range::new(Position::new(1, 18), Position::new(1, 27)),
            "Some".to_string(),
            Some(vec![Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 23), Position::new(1, 26)),
                "role".to_string()
            ))]),
        ));

        let or_pattern = OrPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 27)),
            vec![
                string_patterns[0].clone(),
                string_patterns[1].clone(),
                some_pattern,
            ],
        );

        assert_eq!(or_pattern.patterns.len(), 3);
    }

    // ===================================================================
    // RANGE PATTERNS TESTS
    // ===================================================================

    #[test]
    fn test_range_pattern_exclusive() {
        // Test pattern: 1..10
        let start_expr = Expression::Literal(Literal::new(
            Range::new(Position::new(1, 1), Position::new(1, 1)),
            serde_json::json!(1),
            "number".to_string()
        ));

        let end_expr = Expression::Literal(Literal::new(
            Range::new(Position::new(1, 4), Position::new(1, 5)),
            serde_json::json!(10),
            "number".to_string()
        ));

        let range_pattern = RangePattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 5)),
            start_expr,
            end_expr,
            false, // exclusive
        );

        assert!(!range_pattern.is_inclusive);
    }

    #[test]
    fn test_range_pattern_inclusive() {
        // Test pattern: 'a'..='z'
        let start_expr = Expression::Literal(Literal::new(
            Range::new(Position::new(1, 1), Position::new(1, 3)),
            serde_json::json!("a"),
            "string".to_string()
        ));

        let end_expr = Expression::Literal(Literal::new(
            Range::new(Position::new(1, 7), Position::new(1, 9)),
            serde_json::json!("z"),
            "string".to_string()
        ));

        let range_pattern = RangePattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 9)),
            start_expr,
            end_expr,
            true, // inclusive
        );

        assert!(range_pattern.is_inclusive);
    }

    #[test]
    fn test_range_pattern_variables() {
        // Test pattern: min..max
        let start_expr = Expression::Identifier(Identifier::new(
            Range::new(Position::new(1, 1), Position::new(1, 3)),
            "min".to_string()
        ));

        let end_expr = Expression::Identifier(Identifier::new(
            Range::new(Position::new(1, 6), Position::new(1, 8)),
            "max".to_string()
        ));

        let range_pattern = RangePattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 8)),
            start_expr,
            end_expr,
            false,
        );

        assert!(!range_pattern.is_inclusive);
    }

    // ===================================================================
    // PATTERNS EN LAMBDAS TESTS
    // ===================================================================

    #[test]
    fn test_lambda_with_pattern_parameters() {
        // Test lambda: |(x, y)| => x + y
        let tuple_elements = vec![
            Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 3), Position::new(1, 3)),
                "x".to_string()
            )),
            Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 6), Position::new(1, 6)),
                "y".to_string()
            )),
        ];

        let tuple_pattern = Pattern::Tuple(TuplePattern::new(
            Range::new(Position::new(1, 2), Position::new(1, 7)),
            tuple_elements,
        ));

        let parameters = vec![
            Parameter::new(
                tuple_pattern,
                None,
                None,
                Range::new(Position::new(1, 2), Position::new(1, 7)),
            )
        ];

        let body_expr = Expression::Binary(BinaryExpression::new(
            Range::new(Position::new(1, 12), Position::new(1, 15)),
            Expression::Identifier(Identifier::new(
                Range::new(Position::new(1, 12), Position::new(1, 12)),
                "x".to_string()
            )),
            "+".to_string(),
            Expression::Identifier(Identifier::new(
                Range::new(Position::new(1, 16), Position::new(1, 16)),
                "y".to_string()
            )),
        ));

        let lambda_body = LambdaBody::Expression(Box::new(body_expr));

        let lambda = LambdaExpression::new(
            Range::new(Position::new(1, 1), Position::new(1, 16)),
            parameters,
            lambda_body,
        );

        assert_eq!(lambda.parameters.len(), 1);
        assert!(matches!(lambda.parameters[0].pattern, Pattern::Tuple(_)));
    }

    #[test]
    fn test_lambda_with_struct_pattern() {
        // Test lambda: |{name, age}| => "Hello ${name}"
        let struct_fields = vec![
            StructPatternField::new(
                "name".to_string(),
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 3), Position::new(1, 6)),
                    "name".to_string()
                )),
                Range::new(Position::new(1, 3), Position::new(1, 6)),
            ),
            StructPatternField::new(
                "age".to_string(),
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 9), Position::new(1, 11)),
                    "age".to_string()
                )),
                Range::new(Position::new(1, 9), Position::new(1, 11)),
            ),
        ];

        let struct_pattern = Pattern::Struct(StructPattern::new(
            Range::new(Position::new(1, 2), Position::new(1, 12)),
            "User".to_string(),
            struct_fields,
            false,
        ));

        let parameters = vec![
            Parameter::new(
                struct_pattern,
                None,
                None,
                Range::new(Position::new(1, 2), Position::new(1, 12)),
            )
        ];

        let body_expr = Expression::StringInterpolation(StringInterpolation::new(
            Range::new(Position::new(1, 17), Position::new(1, 28)),
            vec![
                StringInterpolationPart::String("Hello ".to_string()),
                StringInterpolationPart::Expression(Box::new(Expression::Identifier(Identifier::new(
                    Range::new(Position::new(1, 25), Position::new(1, 28)),
                    "name".to_string()
                )))),
            ],
        ));

        let lambda_body = LambdaBody::Expression(Box::new(body_expr));

        let lambda = LambdaExpression::new(
            Range::new(Position::new(1, 1), Position::new(1, 28)),
            parameters,
            lambda_body,
        );

        assert_eq!(lambda.parameters.len(), 1);
        assert!(matches!(lambda.parameters[0].pattern, Pattern::Struct(_)));
    }

    // ===================================================================
    // COMBINACIONES COMPLEJAS TESTS
    // ===================================================================

    #[test]
    fn test_complex_pattern_combination() {
        // Test pattern complejo: Result<T, E> con destructuring anidado
        // Ok({user: {id, name}, items: [first, ...rest]}) | Err(error)

        // Parte Ok del pattern
        let user_struct_fields = vec![
            StructPatternField::new(
                "id".to_string(),
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 8), Position::new(1, 9)),
                    "id".to_string()
                )),
                Range::new(Position::new(1, 8), Position::new(1, 9)),
            ),
            StructPatternField::new(
                "name".to_string(),
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 12), Position::new(1, 15)),
                    "name".to_string()
                )),
                Range::new(Position::new(1, 12), Position::new(1, 15)),
            ),
        ];

        let user_pattern = Pattern::Struct(StructPattern::new(
            Range::new(Position::new(1, 7), Position::new(1, 16)),
            "User".to_string(),
            user_struct_fields,
            false,
        ));

        let items_array_elements = vec![
            crate::ast::ArrayPatternElement::Pattern(Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 26), Position::new(1, 30)),
                "first".to_string()
            ))),
            crate::ast::ArrayPatternElement::Rest(Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 35), Position::new(1, 38)),
                "rest".to_string()
            ))),
        ];

        let items_pattern = Pattern::Array(ArrayPattern::new(
            Range::new(Position::new(1, 25), Position::new(1, 39)),
            items_array_elements,
        ));

        let data_struct_fields = vec![
            StructPatternField::new(
                "user".to_string(),
                user_pattern,
                Range::new(Position::new(1, 5), Position::new(1, 17)),
            ),
            StructPatternField::new(
                "items".to_string(),
                items_pattern,
                Range::new(Position::new(1, 20), Position::new(1, 40)),
            ),
        ];

        let data_pattern = Pattern::Struct(StructPattern::new(
            Range::new(Position::new(1, 4), Position::new(1, 41)),
            "Data".to_string(),
            data_struct_fields,
            false,
        ));

        let ok_pattern = Pattern::Enum(EnumPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 42)),
            "Ok".to_string(),
            Some(vec![data_pattern]),
        ));

        let err_pattern = Pattern::Enum(EnumPattern::new(
            Range::new(Position::new(1, 46), Position::new(1, 55)),
            "Err".to_string(),
            Some(vec![Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 50), Position::new(1, 54)),
                "error".to_string()
            ))]),
        ));

        let or_pattern = OrPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 55)),
            vec![ok_pattern, err_pattern],
        );

        assert_eq!(or_pattern.patterns.len(), 2);
        assert!(matches!(or_pattern.patterns[0], Pattern::Enum(_)));
        assert!(matches!(or_pattern.patterns[1], Pattern::Enum(_)));
    }

    // ===================================================================
    // EDGE CASES TESTS
    // ===================================================================

    #[test]
    fn test_wildcard_pattern() {
        // Test pattern: _
        let wildcard = WildcardPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 1)),
        );

        let pattern = Pattern::Wildcard(wildcard);
        assert!(matches!(pattern, Pattern::Wildcard(_)));
    }

    #[test]
    fn test_empty_struct_pattern() {
        // Test pattern: Struct {}
        let struct_pattern = StructPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 10)),
            "Empty".to_string(),
            vec![], // no fields
            false,
        );

        assert_eq!(struct_pattern.fields.len(), 0);
        assert_eq!(struct_pattern.struct_name, "Empty");
    }

    #[test]
    fn test_enum_pattern_without_data() {
        // Test pattern: None
        let enum_pattern = EnumPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 4)),
            "None".to_string(),
            None, // no inner patterns
        );

        assert_eq!(enum_pattern.variant_name, "None");
        assert!(enum_pattern.inner_patterns.is_none());
    }

    #[test]
    fn test_single_element_or_pattern() {
        // Test pattern: x | (aunque no tiene mucho sentido, debería funcionar)
        let identifier_pattern = Pattern::Identifier(IdentifierPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 1)),
            "x".to_string()
        ));

        let or_pattern = OrPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 1)),
            vec![identifier_pattern],
        );

        assert_eq!(or_pattern.patterns.len(), 1);
    }

    #[test]
    fn test_range_pattern_edge_case() {
        // Test pattern: 0..=0 (rango de un solo valor)
        let start_expr = Expression::Literal(Literal::new(
            Range::new(Position::new(1, 1), Position::new(1, 1)),
            serde_json::json!(0),
            "number".to_string()
        ));

        let end_expr = Expression::Literal(Literal::new(
            Range::new(Position::new(1, 5), Position::new(1, 5)),
            serde_json::json!(0),
            "number".to_string()
        ));

        let range_pattern = RangePattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 5)),
            start_expr,
            end_expr,
            true, // inclusive
        );

        assert!(range_pattern.is_inclusive);
    }

    #[test]
    fn test_deeply_nested_patterns() {
        // Test pattern muy anidado: [[[x]]] con array destructuring
        let innermost = Pattern::Identifier(IdentifierPattern::new(
            Range::new(Position::new(1, 4), Position::new(1, 4)),
            "x".to_string()
        ));

        let level1 = Pattern::Array(ArrayPattern::new(
            Range::new(Position::new(1, 3), Position::new(1, 5)),
            vec![crate::ast::ArrayPatternElement::Pattern(innermost)],
        ));

        let level2 = Pattern::Array(ArrayPattern::new(
            Range::new(Position::new(1, 2), Position::new(1, 6)),
            vec![crate::ast::ArrayPatternElement::Pattern(level1)],
        ));

        let level3 = Pattern::Array(ArrayPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 7)),
            vec![crate::ast::ArrayPatternElement::Pattern(level2)],
        ));

        assert!(matches!(level3, Pattern::Array(_)));
    }

    #[test]
    fn test_pattern_with_guards_in_match() {
        // Test que patterns funcionan con guards en MatchExpression
        let pattern = Pattern::Identifier(IdentifierPattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 2)),
            "x".to_string()
        ));

        let guard = Some(Expression::Binary(BinaryExpression::new(
            Range::new(Position::new(1, 6), Position::new(1, 10)),
            Expression::Identifier(Identifier::new(
                Range::new(Position::new(1, 6), Position::new(1, 6)),
                "x".to_string()
            )),
            ">".to_string(),
            Expression::Literal(Literal::new(
                Range::new(Position::new(1, 10), Position::new(1, 10)),
                serde_json::json!(0),
                "number".to_string()
            )),
        )));

        let body = Expression::Literal(Literal::new(
            Range::new(Position::new(1, 14), Position::new(1, 18)),
            serde_json::json!("positive"),
            "string".to_string()
        ));

        let match_arm = MatchExpressionArm::new(
            pattern,
            guard,
            body,
            Range::new(Position::new(1, 1), Position::new(1, 18)),
        );

        assert!(match_arm.guard.is_some());
        assert!(matches!(match_arm.pattern, Pattern::Identifier(_)));
    }
}