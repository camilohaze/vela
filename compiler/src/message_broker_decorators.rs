//! Message Broker Decorators for Vela Compiler
//!
//! This module implements @consumer and @subscribe decorators for automatic
//! message broker consumer registration.
//!
//! Implementation of: TASK-113AG
//! History: VELA-600
//! Date: 2025-12-11

use crate::ast::{Decorator, Expression, FunctionDeclaration, ASTNode, create_range};
use crate::error::{CompileError, CompileResult};
use std::collections::HashMap;

/// Configuration for @consumer decorator
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    pub topic: String,
    pub broker_type: Option<String>, // Optional broker specification
}

/// Configuration for @subscribe decorator
#[derive(Debug, Clone)]
pub struct SubscribeConfig {
    pub broker: String,
    pub topic: String,
}

/// Parsed decorator configurations
#[derive(Debug, Clone)]
pub enum MessageBrokerDecorator {
    Consumer(ConsumerConfig),
    Subscribe(SubscribeConfig),
}

/// Parse @consumer decorator arguments
fn parse_consumer_decorator(args: &[Expression]) -> CompileResult<ConsumerConfig> {
    if args.is_empty() {
        return Err(CompileError::Internal("Decorator @consumer requires at least a topic argument".to_string()));
    }

    let topic = match &args[0] {
        Expression::Literal(lit) if lit.kind == "string" => {
            if let serde_json::Value::String(s) = &lit.value {
                s.clone()
            } else {
                return Err(CompileError::Internal("@consumer topic must be a string literal".to_string()));
            }
        }
        _ => return Err(CompileError::Internal("@consumer topic must be a string literal".to_string())),
    };

    let broker_type = if args.len() > 1 {
        match &args[1] {
            Expression::Literal(lit) if lit.kind == "string" => {
                if let serde_json::Value::String(s) = &lit.value {
                    Some(s.clone())
                } else {
                    return Err(CompileError::Internal("@consumer broker type must be a string literal".to_string()));
                }
            }
            _ => return Err(CompileError::Internal("@consumer broker type must be a string literal".to_string())),
        }
    } else {
        None
    };

    Ok(ConsumerConfig { topic, broker_type })
}

/// Parse @subscribe decorator arguments
fn parse_subscribe_decorator(args: &[Expression]) -> CompileResult<SubscribeConfig> {
    if args.len() < 2 {
        return Err(CompileError::Internal("Decorator @subscribe requires broker and topic arguments".to_string()));
    }

    let broker = match &args[0] {
        Expression::Literal(lit) if lit.kind == "string" => {
            if let serde_json::Value::String(s) = &lit.value {
                s.clone()
            } else {
                return Err(CompileError::Internal("@subscribe broker must be a string literal".to_string()));
            }
        }
        _ => return Err(CompileError::Internal("@subscribe broker must be a string literal".to_string())),
    };

    let topic = match &args[1] {
        Expression::Literal(lit) if lit.kind == "string" => {
            if let serde_json::Value::String(s) = &lit.value {
                s.clone()
            } else {
                return Err(CompileError::Internal("@subscribe topic must be a string literal".to_string()));
            }
        }
        _ => return Err(CompileError::Internal("@subscribe topic must be a string literal".to_string())),
    };

    Ok(SubscribeConfig { broker, topic })
}

/// Parse message broker decorators from a function
pub fn parse_message_broker_decorators(
    decorators: &[Decorator]
) -> CompileResult<Option<MessageBrokerDecorator>> {
    for decorator in decorators {
        match decorator.name.as_str() {
            "consumer" => {
                let config = parse_consumer_decorator(&decorator.arguments)?;
                return Ok(Some(MessageBrokerDecorator::Consumer(config)));
            }
            "subscribe" => {
                let config = parse_subscribe_decorator(&decorator.arguments)?;
                return Ok(Some(MessageBrokerDecorator::Subscribe(config)));
            }
            _ => continue,
        }
    }
    Ok(None)
}

/// Generate consumer registration code for the runtime
pub fn generate_consumer_registration(
    decorator: &MessageBrokerDecorator,
    function_name: &str,
    module_name: &str,
) -> String {
    match decorator {
        MessageBrokerDecorator::Consumer(config) => {
            let broker_type = config.broker_type.as_deref().unwrap_or("default");
            format!(
                r#"
// Auto-generated consumer registration for @consumer
__register_consumer!(
    "{}",
    "{}",
    "{}",
    "{}"
);"#,
                broker_type, config.topic, module_name, function_name
            )
        }
        MessageBrokerDecorator::Subscribe(config) => {
            format!(
                r#"
// Auto-generated consumer registration for @subscribe
__register_consumer!(
    "{}",
    "{}",
    "{}",
    "{}"
);"#,
                config.broker, config.topic, module_name, function_name
            )
        }
    }
}

/// Validate that decorated function has correct signature
pub fn validate_consumer_function(func: &FunctionDeclaration) -> CompileResult<()> {
    // Check if function returns void or Result
    match &func.return_type {
        Some(_) => {
            // Allow any return type for now
            // TODO: Validate specific return types
        }
        None => {
            // Void functions are OK
        }
    }

    // Check parameters - should accept a message parameter
    if func.parameters.is_empty() {
        return Err(CompileError::Internal(
            "Consumer functions must accept at least one parameter (the message)".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Literal, Expression, Range, Position};

    fn create_string_expr(value: &str) -> Expression {
        Expression::Literal(Literal {
            node: ASTNode::new(create_range(1, 1, 1, 1)),
            value: serde_json::Value::String(value.to_string()),
            kind: "string".to_string(),
        })
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
    fn test_generate_consumer_registration() {
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
    }
}