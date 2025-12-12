//! ValidationErrors: Tipo mejorado para manejo de errores agregados
//!
//! Este módulo proporciona un tipo ValidationErrors que mejora
//! el manejo de errores de validación agregados, permitiendo
//! acceso indexado, filtrado y transformación de errores.

use crate::error::{ValidationError, ValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Colección mejorada de errores de validación
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
    field_errors: HashMap<String, Vec<ValidationError>>,
}

impl ValidationErrors {
    /// Crea una nueva colección de errores vacía
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            field_errors: HashMap::new(),
        }
    }

    /// Crea una colección con un error
    pub fn one(error: ValidationError) -> Self {
        let mut errors = Self::new();
        errors.add(error);
        errors
    }

    /// Crea una colección con múltiples errores
    pub fn many(errors: Vec<ValidationError>) -> Self {
        let mut validation_errors = Self::new();
        for error in errors {
            validation_errors.add(error);
        }
        validation_errors
    }

    /// Agrega un error a la colección
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error.clone());
        self.field_errors
            .entry(error.field.clone())
            .or_insert_with(Vec::new)
            .push(error);
    }

    /// Combina con otra colección de errores
    pub fn combine(mut self, other: Self) -> Self {
        for error in other.errors {
            self.add(error);
        }
        self
    }

    /// Verifica si hay errores
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Obtiene el número total de errores
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Obtiene todos los errores
    pub fn all(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Obtiene errores de un campo específico
    pub fn field(&self, field_name: &str) -> &[ValidationError] {
        self.field_errors
            .get(field_name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Verifica si un campo tiene errores
    pub fn has_field_errors(&self, field_name: &str) -> bool {
        self.field_errors.contains_key(field_name)
    }

    /// Obtiene el primer error
    pub fn first(&self) -> Option<&ValidationError> {
        self.errors.first()
    }

    /// Obtiene el primer error de un campo
    pub fn first_field_error(&self, field_name: &str) -> Option<&ValidationError> {
        self.field_errors.get(field_name)?.first()
    }

    /// Filtra errores por código
    pub fn filter_by_code(&self, code: &str) -> Vec<&ValidationError> {
        self.errors.iter().filter(|e| e.code == code).collect()
    }

    /// Obtiene todos los campos con errores
    pub fn fields_with_errors(&self) -> Vec<&String> {
        self.field_errors.keys().collect()
    }

    /// Convierte a ValidationResult
    pub fn into_result(self) -> ValidationResult {
        if self.is_empty() {
            ValidationResult::valid()
        } else {
            ValidationResult::invalid(self.errors)
        }
    }

    /// Convierte desde ValidationResult
    pub fn from_result(result: ValidationResult) -> Self {
        if result.is_valid {
            Self::new()
        } else {
            Self::many(result.errors)
        }
    }

    /// Obtiene un resumen de errores por campo
    pub fn summary(&self) -> HashMap<String, Vec<&str>> {
        self.field_errors
            .iter()
            .map(|(field, errors)| {
                (field.clone(), errors.iter().map(|e| e.code.as_str()).collect())
            })
            .collect()
    }

    /// Obtiene mensajes de error formateados
    pub fn messages(&self) -> Vec<String> {
        self.errors.iter().map(|e| e.message.clone()).collect()
    }

    /// Obtiene mensajes de error por campo
    pub fn field_messages(&self, field_name: &str) -> Vec<String> {
        self.field_errors
            .get(field_name)
            .map(|errors| errors.iter().map(|e| e.message.clone()).collect())
            .unwrap_or_default()
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ValidationResult> for ValidationErrors {
    fn from(result: ValidationResult) -> Self {
        Self::from_result(result)
    }
}

impl From<ValidationErrors> for ValidationResult {
    fn from(errors: ValidationErrors) -> Self {
        errors.into_result()
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "No validation errors")
        } else {
            writeln!(f, "Validation errors ({}):", self.len())?;
            for error in &self.errors {
                writeln!(f, "  - {}: {}", error.field, error.message)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_error(field: &str, code: &str, message: &str) -> ValidationError {
        ValidationError {
            field: field.to_string(),
            code: code.to_string(),
            message: message.to_string(),
            value: json!("test"),
            constraints: HashMap::new(),
        }
    }

    #[test]
    fn test_validation_errors_new() {
        let errors = ValidationErrors::new();
        assert!(errors.is_empty());
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_validation_errors_add() {
        let mut errors = ValidationErrors::new();
        let error = create_test_error("name", "REQUIRED", "Name is required");

        errors.add(error.clone());

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.all().len(), 1);
        assert_eq!(errors.field("name").len(), 1);
        assert!(errors.has_field_errors("name"));
    }

    #[test]
    fn test_validation_errors_combine() {
        let mut errors1 = ValidationErrors::new();
        errors1.add(create_test_error("name", "REQUIRED", "Name is required"));

        let mut errors2 = ValidationErrors::new();
        errors2.add(create_test_error("email", "EMAIL", "Invalid email"));

        let combined = errors1.combine(errors2);

        assert_eq!(combined.len(), 2);
        assert!(combined.has_field_errors("name"));
        assert!(combined.has_field_errors("email"));
    }

    #[test]
    fn test_validation_errors_field_operations() {
        let mut errors = ValidationErrors::new();
        errors.add(create_test_error("name", "REQUIRED", "Name is required"));
        errors.add(create_test_error("name", "LENGTH", "Name too short"));
        errors.add(create_test_error("email", "EMAIL", "Invalid email"));

        assert_eq!(errors.field("name").len(), 2);
        assert_eq!(errors.field("email").len(), 1);
        assert_eq!(errors.field("age").len(), 0);

        assert!(errors.has_field_errors("name"));
        assert!(!errors.has_field_errors("age"));
    }

    #[test]
    fn test_validation_errors_filter_by_code() {
        let mut errors = ValidationErrors::new();
        errors.add(create_test_error("name", "REQUIRED", "Name is required"));
        errors.add(create_test_error("email", "EMAIL", "Invalid email"));
        errors.add(create_test_error("age", "MIN", "Age too low"));

        let required_errors = errors.filter_by_code("REQUIRED");
        assert_eq!(required_errors.len(), 1);
        assert_eq!(required_errors[0].field, "name");
    }

    #[test]
    fn test_validation_errors_fields_with_errors() {
        let mut errors = ValidationErrors::new();
        errors.add(create_test_error("name", "REQUIRED", "Name is required"));
        errors.add(create_test_error("email", "EMAIL", "Invalid email"));
        errors.add(create_test_error("name", "LENGTH", "Name too short"));

        let fields = errors.fields_with_errors();
        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&&"name".to_string()));
        assert!(fields.contains(&&"email".to_string()));
    }

    #[test]
    fn test_validation_errors_conversion() {
        let mut errors = ValidationErrors::new();
        errors.add(create_test_error("name", "REQUIRED", "Name is required"));

        // To ValidationResult
        let result: ValidationResult = errors.clone().into();
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);

        // From ValidationResult
        let back_to_errors = ValidationErrors::from(result);
        assert_eq!(back_to_errors.len(), 1);
    }

    #[test]
    fn test_validation_errors_summary() {
        let mut errors = ValidationErrors::new();
        errors.add(create_test_error("name", "REQUIRED", "Name is required"));
        errors.add(create_test_error("name", "LENGTH", "Name too short"));
        errors.add(create_test_error("email", "EMAIL", "Invalid email"));

        let summary = errors.summary();

        assert_eq!(summary.get(&"name".to_string()).unwrap().len(), 2);
        assert_eq!(summary.get(&"email".to_string()).unwrap().len(), 1);
        assert!(summary.get(&"name".to_string()).unwrap().contains(&"REQUIRED"));
        assert!(summary.get(&"name".to_string()).unwrap().contains(&"LENGTH"));
    }

    #[test]
    fn test_validation_errors_messages() {
        let mut errors = ValidationErrors::new();
        errors.add(create_test_error("name", "REQUIRED", "Name is required"));
        errors.add(create_test_error("email", "EMAIL", "Invalid email"));

        let messages = errors.messages();
        assert_eq!(messages.len(), 2);
        assert!(messages.contains(&"Name is required".to_string()));
        assert!(messages.contains(&"Invalid email".to_string()));

        let field_messages = errors.field_messages("name");
        assert_eq!(field_messages.len(), 1);
        assert_eq!(field_messages[0], "Name is required");
    }

    #[test]
    fn test_validation_errors_display() {
        let mut errors = ValidationErrors::new();
        errors.add(create_test_error("name", "REQUIRED", "Name is required"));

        let display = format!("{}", errors);
        assert!(display.contains("Validation errors (1)"));
        assert!(display.contains("name: Name is required"));
    }
}