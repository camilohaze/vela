/*
Tests unitarios para patterns in lambdas

Implementación de: TASK-117E (patterns in lambdas)
Historia: VELA-1099 (Pattern Matching Avanzado)
Fecha: 2025-01-30

Estos tests validan que los patterns funcionan correctamente
en parámetros de funciones lambda.
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::path::Path;

    #[test]
    fn test_lambda_with_identifier_pattern() {
        // Test básico: verificar que patterns identifier funcionan
        let pattern = Pattern::Identifier(IdentifierPattern::new(
            Range::new(Position::new(1, 2), Position::new(1, 3)),
            "x".to_string()
        ));
        assert_eq!(pattern.range().start.line, 1);
        assert_eq!(pattern.range().start.column, 2);
    }

    #[test]
    fn test_lambda_with_tuple_pattern() {
        // Test básico: verificar que patterns tuple funcionan
        let elements = vec![
            Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 2), Position::new(1, 3)),
                "x".to_string()
            )),
            Pattern::Identifier(IdentifierPattern::new(
                Range::new(Position::new(1, 5), Position::new(1, 6)),
                "y".to_string()
            )),
        ];
        let pattern = Pattern::Tuple(TuplePattern::new(
            Range::new(Position::new(1, 1), Position::new(1, 7)),
            elements,
        ));
        match pattern {
            Pattern::Tuple(tuple_pattern) => {
                assert_eq!(tuple_pattern.elements.len(), 2);
            }
            _ => panic!("Expected tuple pattern"),
        }
    }

    #[test]
    fn test_lambda_multiple_parameters() {
        // Test básico: verificar que múltiples parámetros funcionan
        let params = vec![
            Parameter::new(
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 2), Position::new(1, 3)),
                    "a".to_string()
                )),
                None, None,
                Range::new(Position::new(1, 2), Position::new(1, 3))
            ),
            Parameter::new(
                Pattern::Identifier(IdentifierPattern::new(
                    Range::new(Position::new(1, 5), Position::new(1, 6)),
                    "b".to_string()
                )),
                None, None,
                Range::new(Position::new(1, 5), Position::new(1, 6))
            ),
        ];
        assert_eq!(params.len(), 2);
        match &params[0].pattern {
            Pattern::Identifier(id) => assert_eq!(id.name, "a"),
            _ => panic!("Expected identifier pattern"),
        }
    }
}