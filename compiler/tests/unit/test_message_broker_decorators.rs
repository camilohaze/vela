//! Unit tests for message broker decorators
//!
//! Tests for TASK-113AG: Implementar decoradores @consumer y @subscribe

use vela_compiler::message_broker_decorators::*;
use vela_compiler::ast::{Decorator, Expression, Literal, Range, Position, FunctionDeclaration, Parameter, TypeAnnotation, BlockStatement, Statement};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_string_expr(value: &str) -> Expression {
        Expression::Literal(Literal {
            node: Default::default(),
            value: serde_json::Value::String(value.to_string()),
            kind: "string".to_string(),
        })
    }

    fn create_decorator(name: &str, args: Vec<Expression>) -> Decorator {
        Decorator {
            name: name.to_string(),
            arguments: args,
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
        }
    }

    #[test]
    fn test_parse_consumer_decorator() {
        let args = vec![create_string_expr("user.created")];
        let config = parse_consumer_decorator(&args).unwrap();
        assert_eq!(config.topic, "user.created");
        assert!(config.broker_type.is_none());
    }

    #[test]
    fn test_parse_consumer_decorator_with_broker() {
        let args = vec![
            create_string_expr("user.created"),
            create_string_expr("rabbitmq"),
        ];
        let config = parse_consumer_decorator(&args).unwrap();
        assert_eq!(config.topic, "user.created");
        assert_eq!(config.broker_type.as_deref(), Some("rabbitmq"));
    }

    #[test]
    fn test_parse_consumer_decorator_invalid_args() {
        let args = vec![]; // No args
        let result = parse_consumer_decorator(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_subscribe_decorator() {
        let args = vec![
            create_string_expr("kafka"),
            create_string_expr("orders"),
        ];
        let config = parse_subscribe_decorator(&args).unwrap();
        assert_eq!(config.broker, "kafka");
        assert_eq!(config.topic, "orders");
    }

    #[test]
    fn test_parse_subscribe_decorator_invalid_args() {
        let args = vec![create_string_expr("kafka")]; // Missing topic
        let result = parse_subscribe_decorator(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_message_broker_decorators_consumer() {
        let decorators = vec![
            create_decorator("consumer", vec![create_string_expr("test.topic")])
        ];
        let result = parse_message_broker_decorators(&decorators).unwrap();
        match result {
            Some(MessageBrokerDecorator::Consumer(config)) => {
                assert_eq!(config.topic, "test.topic");
            }
            _ => panic!("Expected Consumer decorator"),
        }
    }

    #[test]
    fn test_parse_message_broker_decorators_subscribe() {
        let decorators = vec![
            create_decorator("subscribe", vec![
                create_string_expr("redis"),
                create_string_expr("notifications")
            ])
        ];
        let result = parse_message_broker_decorators(&decorators).unwrap();
        match result {
            Some(MessageBrokerDecorator::Subscribe(config)) => {
                assert_eq!(config.broker, "redis");
                assert_eq!(config.topic, "notifications");
            }
            _ => panic!("Expected Subscribe decorator"),
        }
    }

    #[test]
    fn test_parse_message_broker_decorators_no_decorators() {
        let decorators = vec![
            create_decorator("injectable", vec![])
        ];
        let result = parse_message_broker_decorators(&decorators).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_consumer_registration_consumer() {
        let config = ConsumerConfig {
            topic: "test.topic".to_string(),
            broker_type: Some("rabbitmq".to_string()),
        };
        let decorator = MessageBrokerDecorator::Consumer(config);
        let code = generate_consumer_registration(&decorator, "handle_message", "MyModule");
        assert!(code.contains("__register_consumer!"));
        assert!(code.contains("rabbitmq"));
        assert!(code.contains("test.topic"));
        assert!(code.contains("handle_message"));
        assert!(code.contains("MyModule"));
    }

    #[test]
    fn test_generate_consumer_registration_subscribe() {
        let config = SubscribeConfig {
            broker: "kafka".to_string(),
            topic: "orders".to_string(),
        };
        let decorator = MessageBrokerDecorator::Subscribe(config);
        let code = generate_consumer_registration(&decorator, "process_order", "OrderModule");
        assert!(code.contains("__register_consumer!"));
        assert!(code.contains("kafka"));
        assert!(code.contains("orders"));
        assert!(code.contains("process_order"));
        assert!(code.contains("OrderModule"));
    }

    #[test]
    fn test_validate_consumer_function_valid() {
        let func = FunctionDeclaration {
            node: Default::default(),
            is_public: false,
            name: "handle_message".to_string(),
            decorators: vec![],
            parameters: vec![Parameter::from_name("message".to_string()).with_type_annotation(Some(TypeAnnotation::Simple("String".to_string()))).with_range(Range::new(Position::new(0, 0), Position::new(0, 0)))],
            return_type: Some(TypeAnnotation::Simple("void".to_string())),
            body: BlockStatement {
                statements: vec![],
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            },
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
        };
        let result = validate_consumer_function(&func);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consumer_function_no_params() {
        let func = FunctionDeclaration {
            node: Default::default(),
            is_public: false,
            name: "handle_message".to_string(),
            decorators: vec![],
            parameters: vec![], // No parameters
            return_type: Some(TypeAnnotation::Simple("void".to_string())),
            body: BlockStatement {
                statements: vec![],
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            },
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
        };
        let result = validate_consumer_function(&func);
        assert!(result.is_err());
    }
}