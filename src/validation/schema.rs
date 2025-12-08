//! Schema Builder API para validación programática
//!
//! Este módulo proporciona una API fluent para construir schemas
//! de validación de manera programática, complementando los
//! decoradores declarativos.

use crate::validation::error::{ValidationError, ValidationResult};
use crate::validation::validators::*;
use serde_json::Value;
use std::collections::HashMap;

/// Schema de validación que se puede construir programáticamente
#[derive(Debug, Clone)]
pub struct Schema {
    fields: HashMap<String, FieldSchema>,
}

impl Schema {
    /// Crea un nuevo schema vacío
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Agrega un campo al schema
    pub fn field(mut self, name: impl Into<String>, field_schema: FieldSchema) -> Self {
        self.fields.insert(name.into(), field_schema);
        self
    }

    /// Valida un objeto JSON contra el schema
    pub fn validate(&self, value: &Value) -> ValidationResult {
        if let Value::Object(obj) = value {
            let mut result = ValidationResult::valid();

            // Validate defined fields
            for (field_name, field_schema) in &self.fields {
                let field_value = obj.get(field_name);
                let field_result = field_schema.validate(field_value);
                result = result.combine(field_result);
            }

            result
        } else {
            ValidationResult::invalid_one(
                ValidationError::custom("", "Root value must be an object", value.clone())
            )
        }
    }

    /// Valida un HashMap contra el schema
    pub fn validate_map(&self, map: &HashMap<String, Value>) -> ValidationResult {
        let mut result = ValidationResult::valid();

        for (field_name, field_schema) in &self.fields {
            let field_value = map.get(field_name);
            let field_result = field_schema.validate(field_value);
            result = result.combine(field_result);
        }

        result
    }
}

/// Schema para un campo individual
#[derive(Debug, Clone)]
pub struct FieldSchema {
    validators: Vec<Box<dyn Fn(&Option<&Value>) -> ValidationResult + Send + Sync>>,
    required: bool,
}

impl FieldSchema {
    /// Crea un nuevo schema de campo
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            required: false,
        }
    }

    /// Marca el campo como requerido
    pub fn required(mut self) -> Self {
        self.required = true;
        self.validators.push(Box::new(|value| {
            required(&value.map(|v| (*v).clone()), "field")
        }));
        self
    }

    /// Agrega validador de email
    pub fn email(mut self) -> Self {
        self.validators.push(Box::new(|value| {
            email(&value.map(|v| (*v).clone()), "field")
        }));
        self
    }

    /// Agrega validador de valor mínimo
    pub fn min(mut self, min_value: impl Into<Value>) -> Self {
        let min_val = min_value.into();
        self.validators.push(Box::new(move |value| {
            min(&value.map(|v| (*v).clone()), "field", min_val.clone())
        }));
        self
    }

    /// Agrega validador de valor máximo
    pub fn max(mut self, max_value: impl Into<Value>) -> Self {
        let max_val = max_value.into();
        self.validators.push(Box::new(move |value| {
            max(&value.map(|v| (*v).clone()), "field", max_val.clone())
        }));
        self
    }

    /// Agrega validador de longitud
    pub fn length(mut self, min_len: Option<usize>, max_len: Option<usize>) -> Self {
        self.validators.push(Box::new(move |value| {
            length(&value.map(|v| (*v).clone()), "field", min_len, max_len)
        }));
        self
    }

    /// Agrega validador de regex
    pub fn regex(mut self, pattern: impl Into<String>) -> Self {
        let pat = pattern.into();
        self.validators.push(Box::new(move |value| {
            regex(&value.map(|v| (*v).clone()), "field", &pat)
        }));
        self
    }

    /// Agrega validador de URL
    pub fn url(mut self) -> Self {
        self.validators.push(Box::new(|value| {
            url(&value.map(|v| (*v).clone()), "field")
        }));
        self
    }

    /// Agrega validador custom
    pub fn custom<F>(mut self, validator: F) -> Self
    where
        F: Fn(&Option<&Value>) -> ValidationResult + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }

    /// Valida un valor contra este schema de campo
    pub fn validate(&self, value: Option<&Value>) -> ValidationResult {
        let mut result = ValidationResult::valid();

        for validator in &self.validators {
            let validator_result = validator(&value);
            result = result.combine(validator_result);
        }

        result
    }
}

/// Funciones helper para crear tipos comunes de schemas
pub mod types {
    use super::*;

    /// Schema para strings
    pub fn string() -> FieldSchema {
        FieldSchema::new()
    }

    /// Schema para números
    pub fn number() -> FieldSchema {
        FieldSchema::new()
    }

    /// Schema para booleanos
    pub fn boolean() -> FieldSchema {
        FieldSchema::new()
    }

    /// Schema para arrays
    pub fn array() -> FieldSchema {
        FieldSchema::new()
    }

    /// Schema para objetos
    pub fn object() -> FieldSchema {
        FieldSchema::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_field_schema_required() {
        let schema = types::string().required();

        assert!(schema.validate(Some(&json!("hello"))).is_valid);
        assert!(!schema.validate(None).is_valid);
        assert!(!schema.validate(Some(&json!(""))).is_valid);
    }

    #[test]
    fn test_field_schema_email() {
        let schema = types::string().email();

        assert!(schema.validate(Some(&json!("user@example.com"))).is_valid);
        assert!(!schema.validate(Some(&json!("invalid"))).is_valid);
        assert!(schema.validate(None).is_valid); // Email is optional by default
    }

    #[test]
    fn test_field_schema_min_max() {
        let schema = types::number().min(18).max(120);

        assert!(schema.validate(Some(&json!(25))).is_valid);
        assert!(!schema.validate(Some(&json!(15))).is_valid);
        assert!(!schema.validate(Some(&json!(150))).is_valid);
    }

    #[test]
    fn test_field_schema_length() {
        let schema = types::string().length(Some(2), Some(10));

        assert!(schema.validate(Some(&json!("John"))).is_valid);
        assert!(!schema.validate(Some(&json!("X"))).is_valid);
        assert!(!schema.validate(Some(&json!("Very long name"))).is_valid);
    }

    #[test]
    fn test_field_schema_regex() {
        let schema = types::string().regex(r"^\d{3}-\d{2}-\d{4}$");

        assert!(schema.validate(Some(&json!("123-45-6789"))).is_valid);
        assert!(!schema.validate(Some(&json!("invalid"))).is_valid);
    }

    #[test]
    fn test_schema_validation() {
        let schema = Schema::new()
            .field("name", types::string().required().length(Some(2), Some(50)))
            .field("email", types::string().required().email())
            .field("age", types::number().min(18).max(120));

        // Valid object
        let valid_obj = json!({
            "name": "John Doe",
            "email": "john@example.com",
            "age": 30
        });
        assert!(schema.validate(&valid_obj).is_valid);

        // Invalid object - missing required field
        let invalid_obj = json!({
            "email": "john@example.com",
            "age": 30
        });
        assert!(!schema.validate(&invalid_obj).is_valid);

        // Invalid object - invalid email
        let invalid_obj2 = json!({
            "name": "John Doe",
            "email": "invalid-email",
            "age": 30
        });
        assert!(!schema.validate(&invalid_obj2).is_valid);

        // Invalid object - age too low
        let invalid_obj3 = json!({
            "name": "John Doe",
            "email": "john@example.com",
            "age": 15
        });
        assert!(!schema.validate(&invalid_obj3).is_valid);
    }

    #[test]
    fn test_schema_validate_map() {
        let schema = Schema::new()
            .field("name", types::string().required())
            .field("email", types::string().email());

        let mut valid_map = HashMap::new();
        valid_map.insert("name".to_string(), json!("John"));
        valid_map.insert("email".to_string(), json!("john@example.com"));

        assert!(schema.validate_map(&valid_map).is_valid);

        let mut invalid_map = HashMap::new();
        invalid_map.insert("email".to_string(), json!("invalid"));

        assert!(!schema.validate_map(&invalid_map).is_valid);
    }

    #[test]
    fn test_field_schema_custom_validator() {
        let schema = types::number().custom(|value| {
            match value {
                Some(Value::Number(n)) if n.as_i64() == Some(42) => ValidationResult::valid(),
                _ => ValidationResult::invalid_one(
                    ValidationError::custom("field", "Must be 42", Value::Null)
                ),
            }
        });

        assert!(schema.validate(Some(&json!(42))).is_valid);
        assert!(!schema.validate(Some(&json!(24))).is_valid);
    }
}