//! Observability decorators for Vela compiler
//!
//! This module handles parsing and code generation for observability decorators
//! like @traced, @metered, and @logged.

use crate::ast::{Decorator, Expression, Literal, Statement, TypeAnnotation};
use crate::error::CompileError;
use std::collections::HashMap;

/// Traced decorator configuration
#[derive(Debug, Clone)]
pub struct TracedDecorator {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

/// Metered decorator configuration
#[derive(Debug, Clone)]
pub struct MeteredDecorator {
    pub name: String,
    pub help: Option<String>,
    pub labels: Vec<String>,
}

/// Logged decorator configuration
#[derive(Debug, Clone)]
pub struct LoggedDecorator {
    pub level: String,
    pub message: Option<String>,
    pub fields: HashMap<String, String>,
}

/// Parse traced decorator arguments
pub fn parse_traced_decorator(
    decorator: &Decorator,
) -> Result<TracedDecorator, CompileError> {
    let mut name = String::new();
    let mut attributes = HashMap::new();

    if let Some(args) = &decorator.arguments {
        for (i, arg) in args.iter().enumerate() {
            match i {
                0 => {
                    // First argument is the name
                    match arg {
                        Expression::Literal(Literal::String(s)) => {
                            name = s.clone();
                        }
                        _ => {
                            return Err(CompileError::ParseError(format!(
                                "Expected string literal for traced decorator name, got {:?}",
                                arg
                            )));
                        }
                    }
                }
                _ => {
                    // Additional arguments are key-value pairs for attributes
                    if let Expression::Assignment { left, right } = arg {
                        if let Expression::Identifier(key) = left.as_ref() {
                            if let Expression::Literal(Literal::String(value)) = right.as_ref() {
                                attributes.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    if name.is_empty() {
        return Err(CompileError::ParseError(
            "Traced decorator requires a name argument".to_string(),
        ));
    }

    Ok(TracedDecorator { name, attributes })
}

/// Parse metered decorator arguments
pub fn parse_metered_decorator(
    decorator: &Decorator,
) -> Result<MeteredDecorator, CompileError> {
    let mut name = String::new();
    let mut help = None;
    let mut labels = Vec::new();

    if let Some(args) = &decorator.arguments {
        for (i, arg) in args.iter().enumerate() {
            match i {
                0 => {
                    // First argument is the metric name
                    match arg {
                        Expression::Literal(Literal::String(s)) => {
                            name = s.clone();
                        }
                        _ => {
                            return Err(CompileError::ParseError(format!(
                                "Expected string literal for metered decorator name, got {:?}",
                                arg
                            )));
                        }
                    }
                }
                1 => {
                    // Second argument is optional help text
                    match arg {
                        Expression::Literal(Literal::String(s)) => {
                            help = Some(s.clone());
                        }
                        _ => {
                            return Err(CompileError::ParseError(format!(
                                "Expected string literal for metered decorator help, got {:?}",
                                arg
                            )));
                        }
                    }
                }
                _ => {
                    // Additional arguments are label names
                    match arg {
                        Expression::Literal(Literal::String(s)) => {
                            labels.push(s.clone());
                        }
                        Expression::Identifier(s) => {
                            labels.push(s.clone());
                        }
                        _ => {
                            return Err(CompileError::ParseError(format!(
                                "Expected string literal or identifier for metered decorator label, got {:?}",
                                arg
                            )));
                        }
                    }
                }
            }
        }
    }

    if name.is_empty() {
        return Err(CompileError::ParseError(
            "Metered decorator requires a name argument".to_string(),
        ));
    }

    Ok(MeteredDecorator { name, help, labels })
}

/// Parse logged decorator arguments
pub fn parse_logged_decorator(
    decorator: &Decorator,
) -> Result<LoggedDecorator, CompileError> {
    let mut level = "INFO".to_string(); // Default level
    let mut message = None;
    let mut fields = HashMap::new();

    if let Some(args) = &decorator.arguments {
        for (i, arg) in args.iter().enumerate() {
            match i {
                0 => {
                    // First argument is the log level
                    match arg {
                        Expression::Literal(Literal::String(s)) => {
                            level = s.clone();
                        }
                        Expression::Identifier(s) => {
                            level = s.clone();
                        }
                        _ => {
                            return Err(CompileError::ParseError(format!(
                                "Expected string literal or identifier for logged decorator level, got {:?}",
                                arg
                            )));
                        }
                    }
                }
                1 => {
                    // Second argument is optional message
                    match arg {
                        Expression::Literal(Literal::String(s)) => {
                            message = Some(s.clone());
                        }
                        _ => {
                            return Err(CompileError::ParseError(format!(
                                "Expected string literal for logged decorator message, got {:?}",
                                arg
                            )));
                        }
                    }
                }
                _ => {
                    // Additional arguments are key-value pairs for fields
                    if let Expression::Assignment { left, right } = arg {
                        if let Expression::Identifier(key) = left.as_ref() {
                            if let Expression::Literal(Literal::String(value)) = right.as_ref() {
                                fields.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(LoggedDecorator { level, message, fields })
}

/// Generate code for traced decorator
pub fn generate_traced_decorator_code(
    decorator: &TracedDecorator,
    function_name: &str,
) -> Result<String, CompileError> {
    let mut code = String::new();

    // Import observability functions
    code.push_str("use vela_runtime::observability::{get_tracer, SpanContext};\n");

    // Start span
    code.push_str(&format!(
        "let mut __tracing_span = match get_tracer().await {{\n\
         Some(tracer) => tracer.start_span(\"{}\"),\n\
         None => return Err(\"Tracing not initialized\".into()),\n\
         }};\n",
        decorator.name
    ));

    // Set attributes
    for (key, value) in &decorator.attributes {
        code.push_str(&format!(
            "__tracing_span.set_attribute(\"{}\", opentelemetry::Value::String(\"{}\".to_string()));\n",
            key, value
        ));
    }

    // Add function name attribute
    code.push_str(&format!(
        "__tracing_span.set_attribute(\"function\", opentelemetry::Value::String(\"{}\".to_string()));\n",
        function_name
    ));

    // Set span status and end it in a defer-like pattern
    code.push_str("let __span_result = (|| async move {\n");

    Ok(code)
}

/// Generate code for metered decorator
pub fn generate_metered_decorator_code(
    decorator: &MeteredDecorator,
    function_name: &str,
) -> Result<String, CompileError> {
    let mut code = String::new();

    // Import observability functions
    code.push_str("use vela_runtime::observability::{get_metrics, Counter, Histogram};\n");

    // Get metrics registry
    code.push_str("let __metrics_registry = match get_metrics().await {\n\
                   Some(registry) => registry,\n\
                   None => return Err(\"Metrics not initialized\".into()),\n\
                   };\n");

    // Register histogram for duration
    let histogram_name = format!("{}_duration_seconds", decorator.name);
    let help_text = decorator.help.as_ref().unwrap_or(&format!("Duration of {} function", function_name));
    code.push_str(&format!(
        "let __duration_histogram = __metrics_registry.register_histogram(\n\
         \"{}\",\n\
         \"{}\",\n\
         vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0]\n\
         ).await.map_err(|e| format!(\"Failed to register histogram: {{}}\", e))?;\n",
        histogram_name, help_text
    ));

    // Register counter for calls
    let counter_name = format!("{}_total", decorator.name);
    code.push_str(&format!(
        "let __calls_counter = __metrics_registry.register_counter(\n\
         \"{}\",\n\
         \"Total number of {} calls\"\n\
         ).await.map_err(|e| format!(\"Failed to register counter: {{}}\", e))?;\n",
        counter_name, function_name
    ));

    // Start timing
    code.push_str("let __start_time = std::time::Instant::now();\n");

    // Record call
    code.push_str("if let Some(counter) = __metrics_registry.get_counter(\"");
    code.push_str(&counter_name);
    code.push_str("\").await {\n");
    code.push_str("    counter.increment().await;\n");
    code.push_str("}\n");

    // Function call wrapper
    code.push_str("let __result = (|| async move {\n");

    Ok(code)
}

/// Generate code for logged decorator
pub fn generate_logged_decorator_code(
    decorator: &LoggedDecorator,
    function_name: &str,
) -> Result<String, CompileError> {
    let mut code = String::new();

    // Import logging functions
    code.push_str("use vela_runtime::observability::{get_logger, LogRecord, Level};\n");

    // Get logger
    code.push_str("let __logger = match get_logger().await {\n\
                   Some(logger) => logger,\n\
                   None => {\n\
                       // Fallback to println if logging not initialized\n\
                       println!(\"[{}] {}: Function {} called\", \"");
    code.push_str(&decorator.level);
    code.push_str("\", \"");
    code.push_str(&decorator.message.as_ref().unwrap_or(&format!("Calling {}", function_name)));
    code.push_str("\", \"");
    code.push_str(function_name);
    code.push_str("\");\n\
                       return Ok(());\n\
                   }\n\
                   };\n");

    // Create log record
    code.push_str("let mut __log_fields = std::collections::HashMap::new();\n");

    // Add function name field
    code.push_str(&format!(
        "__log_fields.insert(\"function\".to_string(), serde_json::Value::String(\"{}\".to_string()));\n",
        function_name
    ));

    // Add custom fields
    for (key, value) in &decorator.fields {
        code.push_str(&format!(
            "__log_fields.insert(\"{}\".to_string(), serde_json::Value::String(\"{}\".to_string()));\n",
            key, value
        ));
    }

    // Log the call
    let level_enum = match decorator.level.to_uppercase().as_str() {
        "TRACE" => "Level::TRACE",
        "DEBUG" => "Level::DEBUG",
        "INFO" => "Level::INFO",
        "WARN" => "Level::WARN",
        "ERROR" => "Level::ERROR",
        _ => "Level::INFO",
    };

    let message = decorator.message.as_ref().unwrap_or(&format!("Calling {}", function_name));

    code.push_str(&format!(
        "__logger.log_with_fields({}, \"{}\", __log_fields).await?;\n",
        level_enum, message
    ));

    Ok(code)
}

/// Generate cleanup code for decorators
pub fn generate_decorator_cleanup(
    decorators: &[String],
) -> String {
    let mut code = String::new();

    for decorator_type in decorators {
        match decorator_type.as_str() {
            "traced" => {
                code.push_str("// End tracing span\n");
                code.push_str("__tracing_span.set_status(opentelemetry::trace::Status::Ok);\n");
                code.push_str("__tracing_span.end();\n");
            }
            "metered" => {
                code.push_str("// Record metrics\n");
                code.push_str("let __duration = __start_time.elapsed().as_secs_f64();\n");
                code.push_str("if let Some(histogram) = __metrics_registry.get_histogram(\"");
                code.push_str(&format!("{}_duration_seconds", "__metric_name"));
                code.push_str("\").await {\n");
                code.push_str("    histogram.observe(__duration).await;\n");
                code.push_str("}\n");
            }
            "logged" => {
                // Logging cleanup is handled in the function wrapper
            }
            _ => {}
        }
    }

    code.push_str("Ok(__result)\n");
    code.push_str("})().await;\n");

    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Decorator, Expression, Literal};

    #[test]
    fn test_parse_traced_decorator() {
        let decorator = Decorator {
            name: "traced".to_string(),
            arguments: Some(vec![
                Expression::Literal(Literal::String("http_request".to_string())),
            ]),
        };

        let result = parse_traced_decorator(&decorator);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "http_request");
    }

    #[test]
    fn test_parse_metered_decorator() {
        let decorator = Decorator {
            name: "metered".to_string(),
            arguments: Some(vec![
                Expression::Literal(Literal::String("requests_total".to_string())),
                Expression::Literal(Literal::String("Total requests".to_string())),
            ]),
        };

        let result = parse_metered_decorator(&decorator);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "requests_total");
        assert_eq!(config.help, Some("Total requests".to_string()));
    }

    #[test]
    fn test_parse_logged_decorator() {
        let decorator = Decorator {
            name: "logged".to_string(),
            arguments: Some(vec![
                Expression::Literal(Literal::String("INFO".to_string())),
                Expression::Literal(Literal::String("Processing request".to_string())),
            ]),
        };

        let result = parse_logged_decorator(&decorator);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.level, "INFO");
        assert_eq!(config.message, Some("Processing request".to_string()));
    }
}