/**
 * Tests unitarios para AST nodes
 *
 * Jira: VELA-1106
 * Historia: VELA-1106
 */

use vela_compiler::ast::*;
use vela_compiler::common::Range;

#[cfg(test)]
mod tests {
    use super::*;

    mod yield_expression_tests {
        use super::*;

        #[test]
        fn test_yield_expression_creation() {
            let range = Range::new(0, 5);
            let expr = Expression::Literal(Literal::new(range.clone(), serde_json::json!(42), "number".to_string()));
            let yield_expr = YieldExpression::new(range.clone(), Some(expr), false);

            assert_eq!(yield_expr.node.range, range);
            assert!(yield_expr.expression.is_some());
            assert!(!yield_expr.is_delegate);
        }

        #[test]
        fn test_yield_delegate_expression_creation() {
            let range = Range::new(0, 7);
            let expr = Expression::Literal(Literal::new(range.clone(), serde_json::json!("hello"), "string".to_string()));
            let yield_expr = YieldExpression::new(range.clone(), Some(expr), true);

            assert_eq!(yield_expr.node.range, range);
            assert!(yield_expr.expression.is_some());
            assert!(yield_expr.is_delegate);
        }

        #[test]
        fn test_yield_without_expression() {
            let range = Range::new(0, 5);
            let yield_expr = YieldExpression::new(range.clone(), None, false);

            assert_eq!(yield_expr.node.range, range);
            assert!(yield_expr.expression.is_none());
            assert!(!yield_expr.is_delegate);
        }

        #[test]
        fn test_expression_enum_yield_variant() {
            let range = Range::new(0, 5);
            let yield_expr = YieldExpression::new(range.clone(), None, false);
            let expr = Expression::Yield(yield_expr);

            match expr {
                Expression::Yield(y) => {
                    assert_eq!(y.node.range, range);
                    assert!(y.expression.is_none());
                    assert!(!y.is_delegate);
                }
                _ => panic!("Expected Yield expression"),
            }
        }

        #[test]
        fn test_expression_range_yield() {
            let range = Range::new(10, 15);
            let yield_expr = YieldExpression::new(range.clone(), None, false);
            let expr = Expression::Yield(yield_expr);

            assert_eq!(expr.range(), &range);
        }
    }

    mod function_declaration_tests {
        use super::*;

        #[test]
        fn test_function_declaration_with_generator() {
            let range = Range::new(0, 50);
            let body = BlockStatement::new(range.clone(), vec![]);
            let func = FunctionDeclaration::new(
                range.clone(),
                true, // is_public
                "myGenerator".to_string(),
                vec![], // decorators
                vec![], // parameters
                None, // return_type
                body,
                true, // is_async
                true, // is_generator
                vec![], // generic_params
            );

            assert_eq!(func.node.range, range);
            assert!(func.is_public);
            assert_eq!(func.name, "myGenerator");
            assert!(func.is_async);
            assert!(func.is_generator);
        }

        #[test]
        fn test_regular_async_function() {
            let range = Range::new(0, 30);
            let body = BlockStatement::new(range.clone(), vec![]);
            let func = FunctionDeclaration::new(
                range.clone(),
                false, // is_public
                "myAsyncFunc".to_string(),
                vec![], // decorators
                vec![], // parameters
                None, // return_type
                body,
                true, // is_async
                false, // is_generator
                vec![], // generic_params
            );

            assert!(!func.is_public);
            assert_eq!(func.name, "myAsyncFunc");
            assert!(func.is_async);
            assert!(!func.is_generator);
        }

        #[test]
        fn test_regular_sync_function() {
            let range = Range::new(0, 25);
            let body = BlockStatement::new(range.clone(), vec![]);
            let func = FunctionDeclaration::new(
                range.clone(),
                true, // is_public
                "myFunc".to_string(),
                vec![], // decorators
                vec![], // parameters
                None, // return_type
                body,
                false, // is_async
                false, // is_generator
                vec![], // generic_params
            );

            assert!(func.is_public);
            assert_eq!(func.name, "myFunc");
            assert!(!func.is_async);
            assert!(!func.is_generator);
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_async_generator_function_with_yield() {
            // Crear una función async generator
            let func_range = Range::new(0, 100);
            let yield_range = Range::new(50, 60);

            // Crear yield expression
            let literal = Expression::Literal(Literal::new(yield_range.clone(), serde_json::json!(42), "number".to_string()));
            let yield_expr = YieldExpression::new(yield_range.clone(), Some(literal), false);
            let yield_stmt = Statement::Expression(Expression::Yield(yield_expr));

            let body = BlockStatement::new(func_range.clone(), vec![yield_stmt]);

            let func = FunctionDeclaration::new(
                func_range.clone(),
                true,
                "asyncGenerator".to_string(),
                vec![],
                vec![],
                None,
                body,
                true, // async
                true, // generator
                vec![],
            );

            assert!(func.is_async);
            assert!(func.is_generator);
            assert_eq!(func.body.statements.len(), 1);

            // Verificar que el body contiene un yield
            match &func.body.statements[0] {
                Statement::Expression(Expression::Yield(y)) => {
                    assert!(!y.is_delegate);
                    assert!(y.expression.is_some());
                }
                _ => panic!("Expected yield statement"),
            }
        }
    }
}