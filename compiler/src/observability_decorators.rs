//! Observability decorators for Vela compiler
//!
//! This module handles parsing and code generation for observability decorators
//! like @traced, @metered, and @logged.

use crate::ast::{Decorator, Expression, Literal, Statement, TypeAnnotation, BinaryExpression};
use crate::error::{CompileError, ParseError, SourceLocation};
use std::collections::HashMap;

/// Enum representing all observability decorators
#[derive(Debug, Clone)]
pub enum ObservabilityDecorator {
    Traced(TracedDecorator),
    Metered(MeteredDecorator),
    Logged(LoggedDecorator),
}

/// Traced decorator configuration
#[derive(Debug, Clone)]
pub struct TracedDecorator {
    pub name: String,
    pub tags: HashMap<String, String>,
}

/// Metered decorator configuration
#[derive(Debug, Clone)]
pub struct MeteredDecorator {
    pub name: String,
    pub help: Option<String>,
    pub labels: HashMap<String, String>,
}

/// Logged decorator configuration
#[derive(Debug, Clone)]
pub struct LoggedDecorator {
    pub level: String,
    pub message: Option<String>,
    pub fields: HashMap<String, String>,
}

/// Parse observability decorators from a list of decorators
pub fn parse_observability_decorators(
    decorators: &[Decorator],
) -> Result<Option<ObservabilityDecorator>, CompileError> {
    for decorator in decorators {
        match decorator.name.as_str() {
            "traced" => {
                let config = parse_traced_decorator(decorator)?;
                return Ok(Some(ObservabilityDecorator::Traced(config)));
            }
            "metered" => {
                let config = parse_metered_decorator(decorator)?;
                return Ok(Some(ObservabilityDecorator::Metered(config)));
            }
            "logged" => {
                let config = parse_logged_decorator(decorator)?;
                return Ok(Some(ObservabilityDecorator::Logged(config)));
            }
            _ => {
                // Not an observability decorator, continue
            }
        }
    }
    Ok(None)
}

/// Generate observability instrumentation code
pub fn generate_observability_code(
    decorator: &ObservabilityDecorator,
    function_name: &str,
    module_name: &str,
) -> String {
    match decorator {
        ObservabilityDecorator::Traced(traced) => {
            generate_traced_decorator_code(traced, function_name)
                .unwrap_or_else(|_| "// Error generating traced code".to_string())
        }
        ObservabilityDecorator::Metered(metered) => {
            generate_metered_decorator_code(metered, function_name)
                .unwrap_or_else(|_| "// Error generating metered code".to_string())
        }
        ObservabilityDecorator::Logged(logged) => {
            generate_logged_decorator_code(logged, function_name)
                .unwrap_or_else(|_| "// Error generating logged code".to_string())
        }
    }
}

/// Parse traced decorator arguments
pub fn parse_traced_decorator(
    decorator: &Decorator,
) -> Result<TracedDecorator, CompileError> {
    let mut name = String::new();
    let mut tags = HashMap::new();

    for (i, arg) in decorator.arguments.iter().enumerate() {
        match i {
            0 => {
                // First argument is the name
                match arg {
                    Expression::Literal(lit) if lit.kind == "string" => {
                        name = lit.value.as_str()
                            .ok_or_else(|| CompileError::Parse(ParseError {
                                message: "Invalid string literal for traced decorator name".to_string(),
                                location: SourceLocation::new(lit.node.range.start.line, lit.node.range.start.column, 0),
                                expected: vec!["string".to_string()],
                            }))?
                            .to_string();
                    }
                    _ => {
                        return Err(CompileError::Parse(ParseError {
                            message: format!("Expected string literal for traced decorator name, got {:?}", arg),
                            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
                            expected: vec!["string".to_string()],
                        }));
                    }
                }
            }
            _ => {
                // Additional arguments are key-value pairs for tags
                if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
                    if operator == "=" {
                        if let Expression::Identifier(key) = left.as_ref() {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    let key_str = key.name.clone();
                                    let value_str = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: format!("Invalid string value for tag '{}'", key_str),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                    tags.insert(key_str, value_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if name.is_empty() {
        return Err(CompileError::Parse(ParseError {
            message: "Traced decorator requires a name argument".to_string(),
            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
            expected: vec!["name string".to_string()],
        }));
    }

    Ok(TracedDecorator { name, tags })
}

/// Parse metered decorator arguments
pub fn parse_metered_decorator(
    decorator: &Decorator,
) -> Result<MeteredDecorator, CompileError> {
    let mut name = String::new();
    let mut help = None;
    let mut labels = HashMap::new();

    for (i, arg) in decorator.arguments.iter().enumerate() {
        match i {
            0 => {
                // First argument is the metric name
                match arg {
                    Expression::Literal(lit) if lit.kind == "string" => {
                        name = lit.value.as_str()
                            .ok_or_else(|| CompileError::Parse(ParseError {
                                message: "Invalid string literal for metered decorator name".to_string(),
                                location: SourceLocation::new(lit.node.range.start.line, lit.node.range.start.column, 0),
                                expected: vec!["string".to_string()],
                            }))?
                            .to_string();
                    }
                    _ => {
                        return Err(CompileError::Parse(ParseError {
                            message: format!("Expected string literal for metered decorator name, got {:?}", arg),
                            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
                            expected: vec!["string".to_string()],
                        }));
                    }
                }
            }
            1 => {
                // Second argument is optional help text
                match arg {
                    Expression::Literal(lit) if lit.kind == "string" => {
                        help = Some(lit.value.as_str()
                            .ok_or_else(|| CompileError::Parse(ParseError {
                                message: "Invalid string literal for metered decorator help".to_string(),
                                location: SourceLocation::new(lit.node.range.start.line, lit.node.range.start.column, 0),
                                expected: vec!["string".to_string()],
                            }))?
                            .to_string());
                    }
                    _ => {
                        return Err(CompileError::Parse(ParseError {
                            message: format!("Expected string literal for metered decorator help, got {:?}", arg),
                            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
                            expected: vec!["string".to_string()],
                        }));
                    }
                }
            }
            _ => {
                // Additional arguments are key-value pairs for labels
                if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
                    if operator == "=" {
                        if let Expression::Identifier(key) = left.as_ref() {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    let key_str = key.name.clone();
                                    let value_str = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: format!("Invalid string value for label '{}'", key_str),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                    labels.insert(key_str, value_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if name.is_empty() {
        return Err(CompileError::Parse(ParseError {
            message: "Metered decorator requires a name argument".to_string(),
            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
            expected: vec!["name string".to_string()],
        }));
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

    for (i, arg) in decorator.arguments.iter().enumerate() {
        match i {
            0 => {
                // First argument is the log level
                match arg {
                    Expression::Literal(lit) if lit.kind == "string" => {
                        level = lit.value.as_str()
                            .ok_or_else(|| CompileError::Parse(ParseError {
                                message: "Invalid string literal for logged decorator level".to_string(),
                                location: SourceLocation::new(lit.node.range.start.line, lit.node.range.start.column, 0),
                                expected: vec!["string".to_string()],
                            }))?
                            .to_string();
                    }
                    Expression::Identifier(id) => {
                        level = id.name.clone();
                    }
                    _ => {
                        return Err(CompileError::Parse(ParseError {
                            message: format!("Expected string literal or identifier for logged decorator level, got {:?}", arg),
                            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
                            expected: vec!["string or identifier".to_string()],
                        }));
                    }
                }
            }
            1 => {
                // Second argument is optional message
                match arg {
                    Expression::Literal(lit) if lit.kind == "string" => {
                        message = Some(lit.value.as_str()
                            .ok_or_else(|| CompileError::Parse(ParseError {
                                message: "Invalid string literal for logged decorator message".to_string(),
                                location: SourceLocation::new(lit.node.range.start.line, lit.node.range.start.column, 0),
                                expected: vec!["string".to_string()],
                            }))?
                            .to_string());
                    }
                    _ => {
                        return Err(CompileError::Parse(ParseError {
                            message: format!("Expected string literal for logged decorator message, got {:?}", arg),
                            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
                            expected: vec!["string".to_string()],
                        }));
                    }
                }
            }
            _ => {
                // Additional arguments are key-value pairs for fields
                if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
                    if operator == "=" {
                        if let Expression::Identifier(key) = left.as_ref() {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    let key_str = key.name.clone();
                                    let value_str = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: format!("Invalid string value for field '{}'", key_str),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                    fields.insert(key_str, value_str);
                                }
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
    code.push_str("use vela_runtime::observability::{get_tracer};\n");

    // Start span with proper error handling
    code.push_str(&format!(
        "let __tracing_span = match get_tracer().await {{\n\
         Some(tracer) => {{\n\
             let mut span = tracer.start_span(\"{}\");\n",
        decorator.name
    ));

    // Set attributes
    for (key, value) in &decorator.tags {
        code.push_str(&format!(
            "            span.set_attribute(\"{}\", \"{}\");\n",
            key, value
        ));
    }

    // Add function name attribute
    code.push_str(&format!(
        "            span.set_attribute(\"function\", \"{}\");\n",
        function_name
    ));

    // Add service name if available
    code.push_str("            span.set_attribute(\"service.name\", env!(\"CARGO_PKG_NAME\", \"vela-service\"));\n");

    code.push_str("            Some(span)\n\
         }},\n\
         None => None,\n\
         };\n");

    // Create a guard to automatically end the span
    code.push_str("let __span_guard = __tracing_span.as_ref().map(|s| s.clone());\n");

    // Function call wrapper
    code.push_str("let __result = async move {\n");

    Ok(code)
}

/// Generate code for metered decorator
pub fn generate_metered_decorator_code(
    decorator: &MeteredDecorator,
    function_name: &str,
) -> Result<String, CompileError> {
    let mut code = String::new();

    // Import observability functions
    code.push_str("use vela_runtime::observability::{get_metrics};\n");

    // Get metrics registry
    code.push_str("let __metrics_registry = match get_metrics().await {\n\
                   Some(registry) => registry,\n\
                   None => return Ok(()), // Gracefully degrade if metrics not available\n\
                   };\n");

    // Store metric name for cleanup
    code.push_str(&format!("let __metric_name = \"{}\";\n", decorator.name));

    // Register histogram for duration
    let histogram_name = format!("{}_duration_seconds", decorator.name);
    let help_text = decorator.help.as_ref()
        .map(|s| s.clone())
        .unwrap_or_else(|| format!("Duration of {} function", function_name));
    code.push_str(&format!(
        "let __duration_histogram_result = __metrics_registry.register_histogram(\n\
         \"{}\",\n\
         \"{}\",\n\
         vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0]\n\
         ).await;\n",
        histogram_name, help_text
    ));

    // Register counter for calls
    let counter_name = format!("{}_total", decorator.name);
    code.push_str(&format!(
        "let __calls_counter_result = __metrics_registry.register_counter(\n\
         \"{}\",\n\
         \"Total number of {} calls\"\n\
         ).await;\n",
        counter_name, function_name
    ));

    // Start timing
    code.push_str("let __start_time = std::time::Instant::now();\n");

    // Record call (with error handling)
    code.push_str("if let Ok(Some(counter)) = __calls_counter_result.as_ref() {\n");
    code.push_str("    let _ = counter.increment().await;\n");
    code.push_str("}\n");

    // Function call wrapper
    code.push_str("let __result = async move {\n");

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
    let level_str = decorator.level.clone();
    let message_str = decorator.message.as_ref()
        .map(|s| s.clone())
        .unwrap_or_else(|| format!("Calling {}", function_name));

    code.push_str("let __logger = match get_logger().await {\n\
                   Some(logger) => logger,\n\
                   None => {\n\
                       // Fallback to println if logging not initialized\n\
                       println!(\"[{}] {}: Function {} called\", \"");
    code.push_str(&level_str);
    code.push_str("\", \"");
    code.push_str(&message_str);
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

    let default_message = format!("Calling {}", function_name);
    let message = decorator.message.as_ref().unwrap_or(&default_message);

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

    // Execute the function call
    code.push_str("    __result\n");
    code.push_str("}.await;\n");

    // Cleanup code
    for decorator_type in decorators {
        match decorator_type.as_str() {
            "traced" => {
                code.push_str("// End tracing span\n");
                code.push_str("if let Some(mut span) = __tracing_span {\n");
                code.push_str("    span.set_status(opentelemetry::trace::Status::Ok);\n");
                code.push_str("    span.end();\n");
                code.push_str("}\n");
            }
            "metered" => {
                code.push_str("// Record metrics\n");
                code.push_str("let __duration = __start_time.elapsed().as_secs_f64();\n");
                code.push_str("if let Ok(Some(histogram)) = __duration_histogram_result.as_ref() {\n");
                code.push_str("    let _ = histogram.observe(__duration).await;\n");
                code.push_str("}\n");
            }
            "logged" => {
                // Logging cleanup is handled in the function wrapper
                code.push_str("// Logging cleanup handled in wrapper\n");
            }
            _ => {}
        }
    }

    code.push_str("__result\n");
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
        assert!(config.labels.is_empty());
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