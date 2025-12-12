/*!
# Serialization Decorators for Vela Compiler

This module implements @serializable, @serialize, @ignore, and @custom decorators
for automatic JSON serialization code generation.

Implementation of: TASK-113BJ, TASK-113BK, TASK-113BL, TASK-113BM, TASK-113BN
History: VELA-607
Date: 2025-01-30
*/

use crate::ast::{Decorator, Expression, create_range};
use crate::error::{CompileError, CompileResult};
use std::collections::HashMap;

/// Configuration for field serialization
#[derive(Debug, Clone)]
pub enum FieldConfig {
    /// Include field with optional custom name
    Include { serialized_name: String },
    /// Ignore field during serialization
    Ignore,
    /// Use custom serializer
    Custom { serializer: String },
}

/// Serializable class configuration
#[derive(Debug, Clone)]
pub struct SerializableClass {
    pub name: String,
    pub fields: HashMap<String, FieldConfig>,
    pub custom_serializer: Option<String>,
}

/// Code generator for serialization methods
pub struct SerializationCodeGenerator;

impl SerializationCodeGenerator {
    /// Generate toJson method
    pub fn generate_to_json(&self, class: &SerializableClass) -> String {
        let mut fields_json = Vec::new();

        for (field_name, config) in &class.fields {
            match config {
                FieldConfig::Include { serialized_name } => {
                    fields_json.push(format!("\"{}\": self.{}.toJson()", serialized_name, field_name));
                }
                FieldConfig::Ignore => {
                    // Skip ignored fields
                }
                FieldConfig::Custom { serializer } => {
                    fields_json.push(format!("\"{}\": {}::serialize(self.{})", field_name, serializer, field_name));
                }
            }
        }

        let body = if fields_json.is_empty() {
            "return \"{}\";".to_string()
        } else {
            format!("return format!(\"{{{}}}\", {});",
                   fields_json.join(", "),
                   fields_json.iter().map(|_| "{}").collect::<Vec<_>>().join(", "))
        };

        format!("fn toJson(self) -> String {{\n    {}\n}}", body)
    }

    /// Generate fromJson method
    pub fn generate_from_json(&self, class: &SerializableClass) -> String {
        format!("fn fromJson(json: String) -> Result<{}, Error> {{
            // TODO: Implement JSON parsing
            Err(Error::new(\"Not implemented\"))
        }}", class.name)
    }
}

/// Processor for serialization decorators
pub struct SerializationDecoratorProcessor;

impl SerializationDecoratorProcessor {
    /// Process field decorators and return field configuration
    pub fn process_field_decorators(&self, field_name: &str, decorators: &[&Decorator]) -> CompileResult<FieldConfig> {
        for decorator in decorators {
            match decorator.name.as_str() {
                "serialize" => {
                    if !decorator.arguments.is_empty() {
                        if let Expression::Literal(lit) = &decorator.arguments[0] {
                            if lit.kind == "string" {
                                if let serde_json::Value::String(name) = &lit.value {
                                    return Ok(FieldConfig::Include {
                                        serialized_name: name.clone(),
                                    });
                                }
                            }
                        }
                        return Err(CompileError::Internal(
                            format!("@serialize decorator requires a string literal argument for field {}", field_name)
                        ));
                    } else {
                        return Ok(FieldConfig::Include {
                            serialized_name: field_name.to_string(),
                        });
                    }
                }
                "ignore" => {
                    return Ok(FieldConfig::Ignore);
                }
                "custom" => {
                    if decorator.arguments.len() != 1 {
                        return Err(CompileError::Internal(
                            format!("@custom decorator requires exactly one argument for field {}", field_name)
                        ));
                    }
                    if let Expression::Identifier(serializer) = &decorator.arguments[0] {
                        return Ok(FieldConfig::Custom {
                            serializer: serializer.name.clone(),
                        });
                    }
                    return Err(CompileError::Internal(
                        format!("@custom decorator requires an identifier argument for field {}", field_name)
                    ));
                }
                _ => {
                    // Not a serialization decorator, continue
                }
            }
        }

        // Default: include field with original name
        Ok(FieldConfig::Include {
            serialized_name: field_name.to_string(),
        })
    }

    /// Process class decorators and return serializable class config
    pub fn process_class_decorators(&self, class_name: &str, decorators: &[Decorator]) -> CompileResult<Option<SerializableClass>> {
        let mut is_serializable = false;
        let mut custom_serializer = None;

        for decorator in decorators {
            match decorator.name.as_str() {
                "serializable" => {
                    is_serializable = true;
                    if !decorator.arguments.is_empty() {
                        if let Expression::Identifier(serializer) = &decorator.arguments[0] {
                            custom_serializer = Some(serializer.name.clone());
                        }
                    }
                }
                _ => {
                    // Not a serialization decorator
                }
            }
        }

        if is_serializable {
            Ok(Some(SerializableClass {
                name: class_name.to_string(),
                fields: HashMap::new(), // Will be populated by field processing
                custom_serializer,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_field_decorators_serialize_with_name() {
        let processor = SerializationDecoratorProcessor;

        // Mock decorator @serialize("user_id")
        let decorator = Decorator {
            name: "serialize".to_string(),
            arguments: vec![Expression::Literal(crate::ast::Literal {
                node: crate::ast::ASTNode { range: create_range(0, 0, 0, 0) },
                value: serde_json::Value::String("user_id".to_string()),
                kind: "string".to_string(),
            })],
            range: create_range(0, 0, 0, 0),
        };

        let config = processor.process_field_decorators("id", &[&decorator]).unwrap();

        match config {
            FieldConfig::Include { serialized_name } => {
                assert_eq!(serialized_name, "user_id");
            }
            _ => panic!("Expected Include config"),
        }
    }

    #[test]
    fn test_process_field_decorators_ignore() {
        let processor = SerializationDecoratorProcessor;

        // Mock decorator @ignore
        let decorator = Decorator {
            name: "ignore".to_string(),
            arguments: vec![],
            range: create_range(0, 0, 0, 0),
        };

        let config = processor.process_field_decorators("password", &[&decorator]).unwrap();

        match config {
            FieldConfig::Ignore => {
                // Correct
            }
            _ => panic!("Expected Ignore config"),
        }
    }

    #[test]
    fn test_process_field_decorators_custom() {
        let processor = SerializationDecoratorProcessor;

        // Mock decorator @custom(DateSerializer)
        let decorator = Decorator {
            name: "custom".to_string(),
            arguments: vec![Expression::Identifier(crate::ast::Identifier {
                node: crate::ast::ASTNode { range: create_range(0, 0, 0, 0) },
                name: "DateSerializer".to_string(),
            })],
            range: create_range(0, 0, 0, 0),
        };

        let config = processor.process_field_decorators("birthDate", &[&decorator]).unwrap();

        match config {
            FieldConfig::Custom { serializer } => {
                assert_eq!(serializer, "DateSerializer");
            }
            _ => panic!("Expected Custom config"),
        }
    }

    #[test]
    fn test_process_field_decorators_default() {
        let processor = SerializationDecoratorProcessor;

        // No decorators
        let config = processor.process_field_decorators("name", &[]).unwrap();

        match config {
            FieldConfig::Include { serialized_name } => {
                assert_eq!(serialized_name, "name");
            }
            _ => panic!("Expected Include config with default name"),
        }
    }
}