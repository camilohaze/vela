/*!
# Serialization Decorators

Implementation of serialization decorators for Vela.
Provides @serializable, @serialize, @ignore, and @custom decorators.
*/

use crate::ast::{Decorator, Expr, Type};
use crate::error::{Result, VelaError};
use crate::semantic::SemanticAnalyzer;
use std::collections::HashMap;

/// Serializable class information
#[derive(Debug, Clone)]
pub struct SerializableClass {
    /// Class name
    pub name: String,
    /// Fields with their serialization config
    pub fields: HashMap<String, FieldConfig>,
    /// Custom serializer if specified
    pub custom_serializer: Option<String>,
}

/// Field serialization configuration
#[derive(Debug, Clone)]
pub enum FieldConfig {
    /// Field included with custom name
    Include { serialized_name: String },
    /// Field excluded from serialization
    Ignore,
    /// Field uses custom serializer
    Custom { serializer: String },
}

/// Serialization decorator processor
pub struct SerializationDecoratorProcessor;

impl SerializationDecoratorProcessor {
    /// Process @serializable decorator
    pub fn process_serializable(
        &self,
        decorator: &Decorator,
        class_name: &str,
        fields: &HashMap<String, Type>,
        analyzer: &mut SemanticAnalyzer,
    ) -> Result<SerializableClass> {
        // Validate decorator has no arguments
        if !decorator.args.is_empty() {
            return Err(VelaError::semantic(
                decorator.span,
                "@serializable decorator takes no arguments",
            ));
        }

        let mut serializable_fields = HashMap::new();

        // Process field decorators
        for (field_name, field_type) in fields {
            let field_config = self.process_field_decorators(
                field_name,
                analyzer.get_field_decorators(class_name, field_name),
            )?;
            serializable_fields.insert(field_name.clone(), field_config);
        }

        // Check for @custom class decorator
        let custom_serializer = self.extract_custom_serializer(decorator, analyzer)?;

        Ok(SerializableClass {
            name: class_name.to_string(),
            fields: serializable_fields,
            custom_serializer,
        })
    }

    /// Process field-level decorators
    fn process_field_decorators(
        &self,
        field_name: &str,
        decorators: Vec<&Decorator>,
    ) -> Result<FieldConfig> {
        let mut config = FieldConfig::Include {
            serialized_name: field_name.to_string(),
        };

        for decorator in decorators {
            match decorator.name.as_str() {
                "serialize" => {
                    // @serialize("custom_name")
                    if decorator.args.len() != 1 {
                        return Err(VelaError::semantic(
                            decorator.span,
                            "@serialize decorator takes exactly one string argument",
                        ));
                    }

                    if let Expr::StringLiteral(name) = &decorator.args[0] {
                        config = FieldConfig::Include {
                            serialized_name: name.clone(),
                        };
                    } else {
                        return Err(VelaError::semantic(
                            decorator.span,
                            "@serialize argument must be a string literal",
                        ));
                    }
                }
                "ignore" => {
                    // @ignore
                    if !decorator.args.is_empty() {
                        return Err(VelaError::semantic(
                            decorator.span,
                            "@ignore decorator takes no arguments",
                        ));
                    }
                    config = FieldConfig::Ignore;
                }
                "custom" => {
                    // @custom(MySerializer)
                    if decorator.args.len() != 1 {
                        return Err(VelaError::semantic(
                            decorator.span,
                            "@custom decorator takes exactly one argument",
                        ));
                    }

                    if let Expr::Identifier(serializer) = &decorator.args[0] {
                        config = FieldConfig::Custom {
                            serializer: serializer.clone(),
                        };
                    } else {
                        return Err(VelaError::semantic(
                            decorator.span,
                            "@custom argument must be an identifier",
                        ));
                    }
                }
                _ => {
                    // Other decorators are allowed, just ignore for serialization
                }
            }
        }

        Ok(config)
    }

    /// Extract custom serializer from class decorators
    fn extract_custom_serializer(
        &self,
        class_decorator: &Decorator,
        analyzer: &SemanticAnalyzer,
    ) -> Result<Option<String>> {
        // Look for @custom decorator on the class
        // This would need to be implemented in the semantic analyzer
        // For now, return None
        Ok(None)
    }
}

/// Code generator for serialization
pub struct SerializationCodeGenerator;

impl SerializationCodeGenerator {
    /// Generate toJson method for a serializable class
    pub fn generate_to_json(&self, class: &SerializableClass) -> String {
        let mut code = format!(
            "    fn toJson(self) -> String {{\n"
        );

        // Start JSON object
        code.push_str("        var json = \"{\";\n");
        code.push_str("        var first = true;\n");

        // Generate field serialization
        for (field_name, config) in &class.fields {
            match config {
                FieldConfig::Include { serialized_name } => {
                    code.push_str(&format!(
                        "        if !first {{ json += \",\"; }}\n"
                    ));
                    code.push_str(&format!(
                        "        json += \"\\\"{}\\\": \";\n",
                        serialized_name
                    ));
                    code.push_str(&self.generate_field_value(field_name));
                    code.push_str("        first = false;\n");
                }
                FieldConfig::Ignore => {
                    // Skip ignored fields
                }
                FieldConfig::Custom { serializer } => {
                    code.push_str(&format!(
                        "        if !first {{ json += \",\"; }}\n"
                    ));
                    code.push_str(&format!(
                        "        json += {}::serialize(self.{});\n",
                        serializer, field_name
                    ));
                    code.push_str("        first = false;\n");
                }
            }
        }

        code.push_str("        json += \"}\";\n");
        code.push_str("        return json;\n");
        code.push_str("    }\n");

        code
    }

    /// Generate fromJson method for a serializable class
    pub fn generate_from_json(&self, class: &SerializableClass) -> String {
        let mut code = format!(
            "    fn fromJson(json: String) -> Result<{}, Error> {{\n",
            class.name
        );

        code.push_str("        // Parse JSON and create instance\n");
        code.push_str("        // This is a simplified implementation\n");
        code.push_str(&format!("        return Ok({} {{}});\n", class.name));
        code.push_str("    }\n");

        code
    }

    /// Generate value serialization for a field
    fn generate_field_value(&self, field_name: &str) -> String {
        // Simplified field value generation
        // In a real implementation, this would handle different types
        format!("        json += self.{}.toString();\n", field_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_field_decorators_serialize() {
        let processor = SerializationDecoratorProcessor;

        // Mock decorator @serialize("user_id")
        let decorator = Decorator {
            name: "serialize".to_string(),
            args: vec![Expr::StringLiteral("user_id".to_string())],
            span: 0..0,
        };

        let config = processor.process_field_decorators("id", vec![&decorator]).unwrap();

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
            args: vec![],
            span: 0..0,
        };

        let config = processor.process_field_decorators("password", vec![&decorator]).unwrap();

        match config {
            FieldConfig::Ignore => {
                // Correct
            }
            _ => panic!("Expected Ignore config"),
        }
    }
}