"""
Tests unitarios para decoradores de observability

Jira: TASK-113AT
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

    def test_parse_traced_decorator_simple(self):
        """Test parsing de decorador @traced simple."""
        decorator = Decorator {
            name: "traced",
            arguments: [
                Expression::Literal(Literal::String("test_span"))
            ]
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Traced(traced) => {
                assert traced.name == "test_span"
                assert traced.tags.is_empty()
            }
            _ => panic!("Expected Traced decorator")
        }

    def test_parse_traced_decorator_with_tags(self):
        """Test parsing de decorador @traced con tags."""
        decorator = Decorator {
            name: "traced",
            arguments: [
                Expression::Literal(Literal::String("http_request")),
                Expression::Assignment {
                    left: Box::new(Expression::Identifier("method")),
                    right: Box::new(Expression::Literal(Literal::String("GET")))
                },
                Expression::Assignment {
                    left: Box::new(Expression::Identifier("endpoint")),
                    right: Box::new(Expression::Literal(Literal::String("/users")))
                }
            ]
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Traced(traced) => {
                assert traced.name == "http_request"
                assert traced.tags["method"] == "GET"
                assert traced.tags["endpoint"] == "/users"
            }
            _ => panic!("Expected Traced decorator")
        }

    def test_parse_metered_decorator_simple(self):
        """Test parsing de decorador @metered simple."""
        decorator = Decorator {
            name: "metered",
            arguments: [
                Expression::Literal(Literal::String("test_metric")),
                Expression::Literal(Literal::String("Test metric"))
            ]
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Metered(metered) => {
                assert metered.name == "test_metric"
                assert metered.help == Some("Test metric")
                assert metered.labels.is_empty()
            }
            _ => panic!("Expected Metered decorator")
        }

    def test_parse_logged_decorator_simple(self):
        """Test parsing de decorador @logged simple."""
        decorator = Decorator {
            name: "logged",
            arguments: [
                Expression::Literal(Literal::String("INFO")),
                Expression::Literal(Literal::String("Test message"))
            ]
        }

        result = parse_observability_decorators(&[decorator])

        assert result.is_ok()
        parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Logged(logged) => {
                assert logged.level == "INFO"
                assert logged.message == Some("Test message")
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

    def test_generate_traced_code(self):
        """Test generación de código para decorador @traced."""
        traced = TracedDecorator {
            name: "test_span".to_string(),
            tags: {
                "service": "test-service".to_string(),
                "operation": "test".to_string()
            }.into()
        }

        result = generate_observability_code(&ObservabilityDecorator::Traced(traced), "testFunction", "testModule")

        assert result.contains("get_tracer")
        assert result.contains("test_span")
        assert result.contains("test-service")
        assert result.contains("test")
        assert result.contains("testFunction")

    def test_generate_metered_code(self):
        """Test generación de código para decorador @metered."""
        metered = MeteredDecorator {
            name: "test_metric".to_string(),
            help: Some("Test metric".to_string()),
            labels: HashMap::new()
        }

        result = generate_observability_code(&ObservabilityDecorator::Metered(metered), "testFunction", "testModule")

        assert result.contains("get_metrics")
        assert result.contains("test_metric")
        assert result.contains("Test metric")
        assert result.contains("testFunction")

    def test_generate_logged_code(self):
        """Test generación de código para decorador @logged."""
        logged = LoggedDecorator {
            level: "info".to_string(),
            message: Some("Test message".to_string()),
            fields: HashMap::new()
        }

        result = generate_observability_code(&ObservabilityDecorator::Logged(logged), "testFunction", "testModule")

        assert result.contains("get_logger")
        assert result.contains("info")
        assert result.contains("Test message")
        assert result.contains("testFunction")

    def test_traced_decorator_validation(self):
        """Test validación de parámetros del decorador @traced."""
        # Test sin name (debería fallar)
        decorator = Decorator {
            name: "traced",
            arguments: [
                Expression::Assignment {
                    left: Box::new(Expression::Identifier("service")),
                    right: Box::new(Expression::Literal(Literal::String("test")))
                }
            ]
        }

        result = parse_observability_decorators(&[decorator])
        # Debería retornar None porque no es un decorador válido
        assert result.is_ok()
        assert result.unwrap().is_none()

    def test_multiple_decorators(self):
        """Test parsing de múltiples decoradores de observability."""
        decorators = [
            Decorator {
                name: "traced",
                arguments: [
                    Expression::Literal(Literal::String("test_operation"))
                ]
            },
            Decorator {
                name: "metered",
                arguments: [
                    Expression::Literal(Literal::String("test_metric")),
                    Expression::Literal(Literal::String("Test metric"))
                ]
            }
        ]

        result = parse_observability_decorators(&decorators)

        # Debería retornar el primer decorador encontrado (@traced)
        assert result.is_ok()
        parsed = result.unwrap()
        assert parsed.is_some()

        match parsed.unwrap() {
            ObservabilityDecorator::Traced(_) => {
                # OK - encontró el primer decorador
            }
            _ => panic!("Expected Traced decorator first")
        }


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

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