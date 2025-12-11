"""
Tests unitarios para decoradores de observability

Jira: TASK-113AS
Historia: VELA-602
"""

import pytest
from compiler.observability_decorators import (
    parse_observability_decorators,
    generate_observability_code,
    ObservabilityDecorator,
    MeteredDecorator,
    TracedDecorator,
    LoggedDecorator
)
from compiler.ast import Decorator, Expression, Literal
from compiler.error import CompileError


class TestObservabilityDecorators:
    """Suite de tests para decoradores de observability."""

    def test_parse_metered_decorator_simple(self):
        """Test parsing de decorador @metered simple."""
        decorator = Decorator {
            name: "metered",
            arguments: [
                Expression::Call(CallExpression {
                    callee: Identifier("name"),
                    arguments: [Literal::String("test_metric")]
                }),
                Expression::Call(CallExpression {
                    callee: Identifier("help"),
                    arguments: [Literal::String("Test metric")]
                })
            ]
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        let parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Metered(metered) => {
                assert metered.name == "test_metric"
                assert metered.help == "Test metric"
                assert metered.labels.is_empty()
            }
            _ => panic!("Expected Metered decorator")
        }

    def test_parse_metered_decorator_with_labels(self):
        """Test parsing de decorador @metered con labels."""
        decorator = Decorator {
            name: "metered",
            arguments: [
                Expression::Call(CallExpression {
                    callee: Identifier("name"),
                    arguments: [Literal::String("http_requests")]
                }),
                Expression::Call(CallExpression {
                    callee: Identifier("help"),
                    arguments: [Literal::String("HTTP requests")]
                }),
                Expression::Call(CallExpression {
                    callee: Identifier("labels"),
                    arguments: [Expression::HashMap({
                        "method": Literal::String("GET"),
                        "endpoint": Literal::String("/users")
                    })]
                })
            ]
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        let parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Metered(metered) => {
                assert metered.name == "http_requests"
                assert metered.help == "HTTP requests"
                assert metered.labels.len() == 2
                assert metered.labels["method"] == "GET"
                assert metered.labels["endpoint"] == "/users"
            }
            _ => panic!("Expected Metered decorator")
        }

    def test_parse_traced_decorator(self):
        """Test parsing de decorador @traced."""
        decorator = Decorator {
            name: "traced",
            arguments: [
                Expression::Call(CallExpression {
                    callee: Identifier("name"),
                    arguments: [Literal::String("test_operation")]
                }),
                Expression::Call(CallExpression {
                    callee: Identifier("tags"),
                    arguments: [Expression::HashMap({
                        "service": Literal::String("test-service"),
                        "operation": Literal::String("test")
                    })]
                })
            ]
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        let parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Traced(traced) => {
                assert traced.name == "test_operation"
                assert traced.tags.len() == 2
                assert traced.tags["service"] == "test-service"
                assert traced.tags["operation"] == "test"
            }
            _ => panic!("Expected Traced decorator")
        }

    def test_parse_logged_decorator(self):
        """Test parsing de decorador @logged."""
        decorator = Decorator {
            name: "logged",
            arguments: [
                Expression::Call(CallExpression {
                    callee: Identifier("level"),
                    arguments: [Literal::String("info")]
                }),
                Expression::Call(CallExpression {
                    callee: Identifier("message"),
                    arguments: [Literal::String("Operation completed")]
                })
            ]
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        let parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Logged(logged) => {
                assert logged.level == "info"
                assert logged.message == "Operation completed"
                assert logged.fields.is_empty()
            }
            _ => panic!("Expected Logged decorator")
        }

    def test_parse_no_observability_decorator(self):
        """Test que retorna None cuando no hay decoradores de observability."""
        decorator = Decorator {
            name: "injectable",
            arguments: []
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        assert result.unwrap().is_none()

    def test_generate_metered_code(self):
        """Test generación de código para decorador @metered."""
        metered = MeteredDecorator {
            name: "test_metric".to_string(),
            help: "Test metric".to_string(),
            labels: HashMap::new()
        }

        let decorator = ObservabilityDecorator::Metered(metered)
        let code = generate_observability_code(&decorator, "testFunction", "testModule")

        assert code.contains("test_metric")
        assert code.contains("Test metric")
        assert code.contains("testFunction")
        assert code.contains("testModule")

    def test_generate_traced_code(self):
        """Test generación de código para decorador @traced."""
        traced = TracedDecorator {
            name: "test_span".to_string(),
            tags: {
                "service": "test-service".to_string(),
                "operation": "test".to_string()
            }.into()
        }

        let decorator = ObservabilityDecorator::Traced(traced)
        let code = generate_observability_code(&decorator, "testFunction", "testModule")

        assert code.contains("test_span")
        assert code.contains("test-service")
        assert code.contains("test")
        assert code.contains("testFunction")

    def test_generate_logged_code(self):
        """Test generación de código para decorador @logged."""
        logged = LoggedDecorator {
            level: "info".to_string(),
            message: "Test message".to_string(),
            fields: HashMap::new()
        }

        let decorator = ObservabilityDecorator::Logged(logged)
        let code = generate_observability_code(&decorator, "testFunction", "testModule")

        assert code.contains("info")
        assert code.contains("Test message")
        assert code.contains("testFunction")

    def test_metered_decorator_validation(self):
        """Test validación de parámetros del decorador @metered."""
        // Test sin name (debería fallar)
        decorator = Decorator {
            name: "metered",
            arguments: [
                Expression::Call(CallExpression {
                    callee: Identifier("help"),
                    arguments: [Literal::String("Test metric")]
                })
            ]
        }

        result = parse_observability_decorators(&[decorator])
        assert result.is_err()

        // Test sin help (debería fallar)
        decorator = Decorator {
            name: "metered",
            arguments: [
                Expression::Call(CallExpression {
                    callee: Identifier("name"),
                    arguments: [Literal::String("test_metric")]
                })
            ]
        }

        result = parse_observability_decorators(&[decorator])
        assert result.is_err()

    def test_multiple_decorators(self):
        """Test parsing de múltiples decoradores de observability."""
        decorators = vec![
            Decorator {
                name: "metered",
                arguments: [
                    Expression::Call(CallExpression {
                        callee: Identifier("name"),
                        arguments: [Literal::String("test_metric")]
                    }),
                    Expression::Call(CallExpression {
                        callee: Identifier("help"),
                        arguments: [Literal::String("Test metric")]
                    })
                ]
            },
            Decorator {
                name: "traced",
                arguments: [
                    Expression::Call(CallExpression {
                        callee: Identifier("name"),
                        arguments: [Literal::String("test_span")]
                    })
                ]
            }
        ]

        result = parse_observability_decorators(&decorators)

        // Debería retornar el primer decorador encontrado
        assert result.is_ok()
        let parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Metered(_) => {
                // OK - encontró el primer decorador
            }
            _ => panic!("Expected Metered decorator first")
        }


if __name__ == "__main__":
    pytest.main([__file__, "-v"])