//! Resilience decorators for Vela compiler
//!
//! This module handles parsing and code generation for resilience decorators
//! like @circuitBreaker, @retry, @timeout, @bulkhead, and @fallback.

use crate::ast::{Decorator, Expression, Statement, TypeAnnotation};
use crate::error::CompileError;
use std::collections::HashMap;

/// Circuit Breaker decorator configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerDecorator {
    pub failure_threshold: u32,
    pub recovery_timeout: u64, // milliseconds
    pub success_threshold: u32,
    pub call_timeout: u64, // milliseconds
}

/// Retry decorator configuration
#[derive(Debug, Clone)]
pub struct RetryDecorator {
    pub max_attempts: u32,
    pub base_delay: u64, // milliseconds
    pub max_delay: Option<u64>, // milliseconds
    pub backoff_multiplier: f64,
}

/// Timeout decorator configuration
#[derive(Debug, Clone)]
pub struct TimeoutDecorator {
    pub duration: u64, // milliseconds
}

/// Bulkhead decorator configuration
#[derive(Debug, Clone)]
pub struct BulkheadDecorator {
    pub max_concurrent: usize,
    pub queue_size: usize,
}

/// Fallback decorator configuration
#[derive(Debug, Clone)]
pub struct FallbackDecorator {
    pub fallback_fn: String,
    pub exceptions: Vec<String>,
}

/// Parse circuit breaker decorator arguments
pub fn parse_circuit_breaker_decorator(
    decorator: &Decorator,
) -> Result<CircuitBreakerDecorator, CompileError> {
    let mut config = CircuitBreakerDecorator {
        failure_threshold: 5,
        recovery_timeout: 30000, // 30 seconds
        success_threshold: 2,
        call_timeout: 10000, // 10 seconds
    };

    // Arguments are positional: failure_threshold, recovery_timeout, success_threshold, call_timeout
    if decorator.arguments.len() >= 1 {
        if let Expression::Literal(lit) = &decorator.arguments[0] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.failure_threshold = val as u32;
                    }
                }
            }
        }
    }

    if decorator.arguments.len() >= 2 {
        if let Expression::Literal(lit) = &decorator.arguments[1] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.recovery_timeout = val;
                    }
                }
            }
        }
    }

    if decorator.arguments.len() >= 3 {
        if let Expression::Literal(lit) = &decorator.arguments[2] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.success_threshold = val as u32;
                    }
                }
            }
        }
    }

    if decorator.arguments.len() >= 4 {
        if let Expression::Literal(lit) = &decorator.arguments[3] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.call_timeout = val;
                    }
                }
            }
        }
    }

    Ok(config)
}

/// Parse retry decorator arguments
pub fn parse_retry_decorator(
    decorator: &Decorator,
) -> Result<RetryDecorator, CompileError> {
    let mut config = RetryDecorator {
        max_attempts: 3,
        base_delay: 1000, // 1 second
        max_delay: Some(30000), // 30 seconds
        backoff_multiplier: 2.0,
    };

    // Arguments are positional: max_attempts, base_delay, max_delay, backoff_multiplier
    if decorator.arguments.len() >= 1 {
        if let Expression::Literal(lit) = &decorator.arguments[0] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.max_attempts = val as u32;
                    }
                }
            }
        }
    }

    if decorator.arguments.len() >= 2 {
        if let Expression::Literal(lit) = &decorator.arguments[1] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.base_delay = val;
                    }
                }
            }
        }
    }

    if decorator.arguments.len() >= 3 {
        if let Expression::Literal(lit) = &decorator.arguments[2] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.max_delay = Some(val);
                    }
                }
            }
        }
    }

    if decorator.arguments.len() >= 4 {
        if let Expression::Literal(lit) = &decorator.arguments[3] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_f64() {
                        config.backoff_multiplier = val;
                    }
                }
            }
        }
    }

    Ok(config)
}

/// Parse timeout decorator arguments
pub fn parse_timeout_decorator(
    decorator: &Decorator,
) -> Result<TimeoutDecorator, CompileError> {
    let mut config = TimeoutDecorator {
        duration: 30000, // 30 seconds
    };

    // Arguments are positional: duration
    if decorator.arguments.len() >= 1 {
        if let Expression::Literal(lit) = &decorator.arguments[0] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.duration = val;
                    }
                }
            }
        }
    }

    Ok(config)
}

/// Parse bulkhead decorator arguments
pub fn parse_bulkhead_decorator(
    decorator: &Decorator,
) -> Result<BulkheadDecorator, CompileError> {
    let mut config = BulkheadDecorator {
        max_concurrent: 10,
        queue_size: 50,
    };

    // Arguments are positional: max_concurrent, queue_size
    if decorator.arguments.len() >= 1 {
        if let Expression::Literal(lit) = &decorator.arguments[0] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.max_concurrent = val as usize;
                    }
                }
            }
        }
    }

    if decorator.arguments.len() >= 2 {
        if let Expression::Literal(lit) = &decorator.arguments[1] {
            if lit.kind == "number" {
                if let serde_json::Value::Number(num) = &lit.value {
                    if let Some(val) = num.as_u64() {
                        config.queue_size = val as usize;
                    }
                }
            }
        }
    }

    Ok(config)
}

/// Parse fallback decorator arguments
pub fn parse_fallback_decorator(
    decorator: &Decorator,
) -> Result<FallbackDecorator, CompileError> {
    let mut config = FallbackDecorator {
        fallback_fn: String::new(),
        exceptions: Vec::new(),
    };

    // Arguments are positional: fallback_fn, exceptions
    if decorator.arguments.len() >= 1 {
        if let Expression::Literal(lit) = &decorator.arguments[0] {
            if lit.kind == "string" {
                if let serde_json::Value::String(val) = &lit.value {
                    config.fallback_fn = val.clone();
                }
            }
        }
    }

    if decorator.arguments.len() >= 2 {
        if let Expression::ArrayLiteral(array_lit) = &decorator.arguments[1] {
            for element in &array_lit.elements {
                if let Expression::Literal(lit) = element {
                    if lit.kind == "string" {
                        if let serde_json::Value::String(s) = &lit.value {
                            config.exceptions.push(s.clone());
                        }
                    }
                }
            }
        }
    }

    Ok(config)
}

/// Generate Rust code for circuit breaker
pub fn generate_circuit_breaker_code(
    config: &CircuitBreakerDecorator,
    function_name: &str,
    original_body: &str,
) -> String {
    format!(
        r#"async fn {}(/* original params */) -> /* original return type */ {{
    let cb_config = vela_runtime::resilience::CircuitBreakerConfig {{
        failure_threshold: {},
        recovery_timeout: std::time::Duration::from_millis({}),
        success_threshold: {},
        call_timeout: std::time::Duration::from_millis({}),
    }};

    vela_runtime::resilience::with_circuit_breaker(
        cb_config,
        "{}.{}",
        || async {{
            {}
        }}
    ).await
}}"#,
        function_name,
        config.failure_threshold,
        config.recovery_timeout,
        config.success_threshold,
        config.call_timeout,
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "vela_app".to_string()),
        function_name,
        original_body
    )
}

/// Generate Rust code for retry
pub fn generate_retry_code(
    config: &RetryDecorator,
    function_name: &str,
    original_body: &str,
) -> String {
    let max_delay_str = match config.max_delay {
        Some(delay) => format!("Some(std::time::Duration::from_millis({}))", delay),
        None => "None".to_string(),
    };

    format!(
        r#"async fn {}(/* original params */) -> /* original return type */ {{
    let retry_config = vela_runtime::resilience::RetryConfig {{
        max_attempts: {},
        base_delay: std::time::Duration::from_millis({}),
        max_delay: {},
        backoff_multiplier: {:.1},
    }};

    vela_runtime::resilience::with_retry(
        retry_config,
        || async {{
            {}
        }}
    ).await
}}"#,
        function_name,
        config.max_attempts,
        config.base_delay,
        max_delay_str,
        config.backoff_multiplier,
        original_body
    )
}

/// Generate Rust code for timeout
pub fn generate_timeout_code(
    config: &TimeoutDecorator,
    function_name: &str,
    original_body: &str,
) -> String {
    format!(
        r#"async fn {}(/* original params */) -> /* original return type */ {{
    let timeout_config = vela_runtime::resilience::TimeoutConfig {{
        duration: std::time::Duration::from_millis({}),
    }};

    vela_runtime::resilience::with_timeout(
        timeout_config,
        || async {{
            {}
        }}
    ).await
}}"#,
        function_name,
        config.duration,
        original_body
    )
}

/// Generate Rust code for bulkhead
pub fn generate_bulkhead_code(
    config: &BulkheadDecorator,
    function_name: &str,
    original_body: &str,
) -> String {
    format!(
        r#"async fn {}(/* original params */) -> /* original return type */ {{
    let bulkhead_config = vela_runtime::resilience::BulkheadConfig {{
        max_concurrent: {},
        queue_size: {},
    }};

    vela_runtime::resilience::with_bulkhead(
        bulkhead_config,
        || async {{
            {}
        }}
    ).await
}}"#,
        function_name,
        config.max_concurrent,
        config.queue_size,
        original_body
    )
}

/// Generate Rust code for fallback
pub fn generate_fallback_code(
    config: &FallbackDecorator,
    function_name: &str,
    original_body: &str,
) -> String {
    let exceptions_str = config.exceptions.iter()
        .map(|e| format!("\"{}\".to_string()", e))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"async fn {}(/* original params */) -> /* original return type */ {{
    let fallback_config = vela_runtime::resilience::FallbackConfig {{
        exceptions: vec![{}],
    }};

    vela_runtime::resilience::with_fallback(
        fallback_config,
        || async {{
            {}
        }},
        || async {{
            {}({})
        }}
    ).await
}}"#,
        function_name,
        exceptions_str,
        original_body,
        config.fallback_fn,
        "/* original params */"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Decorator, Expression};

    #[test]
    fn test_parse_circuit_breaker_decorator() {
        let range = crate::ast::create_range(1, 1, 1, 10);
        let decorator = Decorator {
            name: "circuitBreaker".to_string(),
            arguments: vec![
                Expression::Literal(crate::ast::Literal::new(range, serde_json::json!(3), "number".to_string())),
                Expression::Literal(crate::ast::Literal::new(range, serde_json::json!(15000), "number".to_string())),
            ],
            range,
        };

        let config = parse_circuit_breaker_decorator(&decorator).unwrap();
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.recovery_timeout, 15000);
        assert_eq!(config.success_threshold, 2); // default
        assert_eq!(config.call_timeout, 10000); // default
    }

    #[test]
    fn test_parse_retry_decorator() {
        let range = crate::ast::create_range(1, 1, 1, 10);
        let decorator = Decorator {
            name: "retry".to_string(),
            arguments: vec![
                Expression::Literal(crate::ast::Literal::new(range, serde_json::json!(5), "number".to_string())),
                Expression::Literal(crate::ast::Literal::new(range, serde_json::json!(500), "number".to_string())),
                Expression::Literal(crate::ast::Literal::new(range, serde_json::json!(1.5), "number".to_string())),
            ],
            range,
        };

        let config = parse_retry_decorator(&decorator).unwrap();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.base_delay, 500);
        assert_eq!(config.backoff_multiplier, 1.5);
    }

    #[test]
    fn test_generate_circuit_breaker_code() {
        let config = CircuitBreakerDecorator {
            failure_threshold: 3,
            recovery_timeout: 15000,
            success_threshold: 1,
            call_timeout: 5000,
        };

        let code = generate_circuit_breaker_code(&config, "test_function", "original_body();");

        assert!(code.contains("failure_threshold: 3"));
        assert!(code.contains("recovery_timeout: std::time::Duration::from_millis(15000)"));
        assert!(code.contains("success_threshold: 1"));
        assert!(code.contains("call_timeout: std::time::Duration::from_millis(5000)"));
        assert!(code.contains("vela_runtime::resilience::with_circuit_breaker"));
    }
}