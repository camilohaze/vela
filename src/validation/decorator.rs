//! Decoradores para validación declarativa
//!
//! Este módulo proporciona los decoradores que permiten usar
//! el sistema de validación de manera declarativa en structs.

use crate::validation::error::ValidationResult;
use crate::validation::validators::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait que deben implementar los tipos que quieren ser validados
pub trait Validatable {
    /// Valida la instancia y retorna el resultado
    fn validate(&self) -> ValidationResult;
}

/// Macro para generar código de validación para structs
#[macro_export]
macro_rules! validation_impl {
    ($struct_name:ident { $($field:ident: $validator:expr),* $(,)? }) => {
        impl Validatable for $struct_name {
            fn validate(&self) -> ValidationResult {
                let mut result = ValidationResult::valid();
                $(
                    let field_result = $validator(&self.$field, stringify!($field));
                    result = result.combine(field_result);
                )*
                result
            }
        }
    };
}

/// Decorador para marcar campos como requeridos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Required;

/// Decorador para validar formato de email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email;

/// Decorador para validar valor mínimo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Min {
    pub value: serde_json::Value,
}

/// Decorador para validar valor máximo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Max {
    pub value: serde_json::Value,
}

/// Decorador para validar longitud de strings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Length {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

/// Decorador para validar patrón regex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regex {
    pub pattern: String,
}

/// Decorador para validar URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url;

/// Decorador para validación custom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Custom {
    pub validator: fn(&Option<serde_json::Value>) -> Result<(), String>,
}

/// Enum que representa todos los decoradores de validación disponibles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationDecorator {
    Required(Required),
    Email(Email),
    Min(Min),
    Max(Max),
    Length(Length),
    Regex(Regex),
    Url(Url),
    Custom(Custom),
}

/// Metadata de validación para un campo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidation {
    pub decorators: Vec<ValidationDecorator>,
}

/// Metadata de validación para una struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructValidation {
    pub fields: HashMap<String, FieldValidation>,
}

impl StructValidation {
    /// Crea una nueva instancia de StructValidation
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Agrega validación para un campo
    pub fn field(mut self, field_name: impl Into<String>, validation: FieldValidation) -> Self {
        self.fields.insert(field_name.into(), validation);
        self
    }

    /// Valida un valor usando la metadata de validación
    pub fn validate_value(&self, field_name: &str, value: &Option<serde_json::Value>) -> ValidationResult {
        if let Some(field_validation) = self.fields.get(field_name) {
            let mut result = ValidationResult::valid();

            for decorator in &field_validation.decorators {
                let decorator_result = match decorator {
                    ValidationDecorator::Required(_) => required(value, field_name),
                    ValidationDecorator::Email(_) => email(value, field_name),
                    ValidationDecorator::Min(min) => min(value, field_name, min.value.clone()),
                    ValidationDecorator::Max(max) => max(value, field_name, max.value.clone()),
                    ValidationDecorator::Length(length) => length(value, field_name, length.min, length.max),
                    ValidationDecorator::Regex(regex) => regex(value, field_name, &regex.pattern),
                    ValidationDecorator::Url(_) => url(value, field_name),
                    ValidationDecorator::Custom(custom) => custom(value, field_name, custom.validator),
                };

                result = result.combine(decorator_result);
            }

            result
        } else {
            ValidationResult::valid()
        }
    }
}

/// Helper functions para crear decoradores fácilmente
pub mod decorators {
    use super::*;

    pub fn required() -> ValidationDecorator {
        ValidationDecorator::Required(Required)
    }

    pub fn email() -> ValidationDecorator {
        ValidationDecorator::Email(Email)
    }

    pub fn min(value: impl Into<serde_json::Value>) -> ValidationDecorator {
        ValidationDecorator::Min(Min { value: value.into() })
    }

    pub fn max(value: impl Into<serde_json::Value>) -> ValidationDecorator {
        ValidationDecorator::Max(Max { value: value.into() })
    }

    pub fn length(min: Option<usize>, max: Option<usize>) -> ValidationDecorator {
        ValidationDecorator::Length(Length { min, max })
    }

    pub fn regex(pattern: impl Into<String>) -> ValidationDecorator {
        ValidationDecorator::Regex(Regex { pattern: pattern.into() })
    }

    pub fn url() -> ValidationDecorator {
        ValidationDecorator::Url(Url)
    }

    pub fn custom(validator: fn(&Option<serde_json::Value>) -> Result<(), String>) -> ValidationDecorator {
        ValidationDecorator::Custom(Custom { validator })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_struct_validation_creation() {
        let validation = StructValidation::new()
            .field("name", FieldValidation {
                decorators: vec![decorators::required(), decorators::length(Some(2), Some(50))],
            })
            .field("email", FieldValidation {
                decorators: vec![decorators::required(), decorators::email()],
            });

        assert!(validation.fields.contains_key("name"));
        assert!(validation.fields.contains_key("email"));
    }

    #[test]
    fn test_field_validation_required() {
        let validation = StructValidation::new()
            .field("name", FieldValidation {
                decorators: vec![decorators::required()],
            });

        assert!(validation.validate_value("name", &Some(json!("John"))).is_valid);
        assert!(!validation.validate_value("name", &None).is_valid);
    }

    #[test]
    fn test_field_validation_email() {
        let validation = StructValidation::new()
            .field("email", FieldValidation {
                decorators: vec![decorators::email()],
            });

        assert!(validation.validate_value("email", &Some(json!("user@example.com"))).is_valid);
        assert!(!validation.validate_value("email", &Some(json!("invalid"))).is_valid);
    }

    #[test]
    fn test_field_validation_min_max() {
        let validation = StructValidation::new()
            .field("age", FieldValidation {
                decorators: vec![decorators::min(18), decorators::max(120)],
            });

        assert!(validation.validate_value("age", &Some(json!(25))).is_valid);
        assert!(!validation.validate_value("age", &Some(json!(15))).is_valid);
        assert!(!validation.validate_value("age", &Some(json!(150))).is_valid);
    }

    #[test]
    fn test_field_validation_length() {
        let validation = StructValidation::new()
            .field("name", FieldValidation {
                decorators: vec![decorators::length(Some(2), Some(10))],
            });

        assert!(validation.validate_value("name", &Some(json!("John"))).is_valid);
        assert!(!validation.validate_value("name", &Some(json!("X"))).is_valid);
        assert!(!validation.validate_value("name", &Some(json!("This is a very long name"))).is_valid);
    }

    #[test]
    fn test_multiple_decorators_on_field() {
        let validation = StructValidation::new()
            .field("email", FieldValidation {
                decorators: vec![decorators::required(), decorators::email()],
            });

        // Valid case
        assert!(validation.validate_value("email", &Some(json!("user@example.com"))).is_valid);

        // Missing required
        let result = validation.validate_value("email", &None);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "REQUIRED");

        // Invalid email format
        let result = validation.validate_value("email", &Some(json!("invalid")));
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "EMAIL");
    }
}