//! Tipos de error para el sistema de validación
//!
//! Define los tipos `ValidationError` y `ValidationResult` que se usan
//! en todo el sistema de validación para reportar errores de manera
//! consistente y detallada.

use std::collections::HashMap;

/// Representa un error de validación individual
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// Campo que falló la validación
    pub field: String,
    /// Código de error estandarizado
    pub code: String,
    /// Mensaje de error legible por humanos
    pub message: String,
    /// Valor que causó el error
    pub value: serde_json::Value,
    /// Constraints adicionales del validador
    pub constraints: HashMap<String, serde_json::Value>,
}

impl ValidationError {
    /// Crea un nuevo ValidationError
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
            value: value.into(),
            constraints: HashMap::new(),
        }
    }

    /// Agrega una constraint al error
    pub fn with_constraint(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.constraints.insert(key.into(), value.into());
        self
    }

    /// Crea un error para campo requerido faltante
    pub fn required(field: impl Into<String>) -> Self {
        Self::new(
            field,
            "REQUIRED",
            "This field is required",
            serde_json::Value::Null,
        )
    }

    /// Crea un error para email inválido
    pub fn invalid_email(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(
            field,
            "EMAIL",
            "Invalid email format",
            value.into(),
        )
    }

    /// Crea un error para valor menor al mínimo
    pub fn min_violation(field: impl Into<String>, value: impl Into<serde_json::Value>, min: impl Into<serde_json::Value>) -> Self {
        Self::new(
            field,
            "MIN",
            "Value is below minimum",
            value,
        ).with_constraint("min", min)
    }

    /// Crea un error para valor mayor al máximo
    pub fn max_violation(field: impl Into<String>, value: impl Into<serde_json::Value>, max: impl Into<serde_json::Value>) -> Self {
        Self::new(
            field,
            "MAX",
            "Value is above maximum",
            value,
        ).with_constraint("max", max)
    }

    /// Crea un error para longitud fuera de rango
    pub fn length_violation(field: impl Into<String>, value: impl Into<String>, min: Option<usize>, max: Option<usize>) -> Self {
        let mut error = Self::new(
            field,
            "LENGTH",
            "Length is out of range",
            value,
        );

        if let Some(min) = min {
            error = error.with_constraint("min", min);
        }
        if let Some(max) = max {
            error = error.with_constraint("max", max);
        }

        error
    }

    /// Crea un error para patrón regex no cumplido
    pub fn regex_violation(field: impl Into<String>, value: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(
            field,
            "REGEX",
            "Value does not match required pattern",
            value,
        ).with_constraint("pattern", pattern)
    }

    /// Crea un error para validador custom
    pub fn custom(field: impl Into<String>, message: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Self::new(
            field,
            "CUSTOM",
            message,
            value,
        )
    }
}

/// Resultado de una validación
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Indica si la validación fue exitosa
    pub is_valid: bool,
    /// Lista de errores encontrados (vacía si is_valid = true)
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Crea un resultado válido (sin errores)
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    /// Crea un resultado inválido con errores
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    /// Crea un resultado inválido con un error
    pub fn invalid_one(error: ValidationError) -> Self {
        Self::invalid(vec![error])
    }

    /// Agrega un error al resultado
    pub fn add_error(mut self, error: ValidationError) -> Self {
        self.is_valid = false;
        self.errors.push(error);
        self
    }

    /// Combina dos resultados de validación
    pub fn combine(mut self, other: ValidationResult) -> Self {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
        self
    }

    /// Retorna el primer error si existe
    pub fn first_error(&self) -> Option<&ValidationError> {
        self.errors.first()
    }

    /// Retorna todos los errores para un campo específico
    pub fn errors_for_field(&self, field: &str) -> Vec<&ValidationError> {
        self.errors.iter().filter(|e| e.field == field).collect()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError::required("name");
        assert_eq!(error.field, "name");
        assert_eq!(error.code, "REQUIRED");
        assert_eq!(error.message, "This field is required");
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_result_invalid() {
        let error = ValidationError::required("email");
        let result = ValidationResult::invalid_one(error.clone());
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], error);
    }

    #[test]
    fn test_validation_result_combine() {
        let error1 = ValidationError::required("name");
        let error2 = ValidationError::invalid_email("email", "invalid");
        let result1 = ValidationResult::invalid_one(error1);
        let result2 = ValidationResult::invalid_one(error2);

        let combined = result1.combine(result2);
        assert!(!combined.is_valid);
        assert_eq!(combined.errors.len(), 2);
    }

    #[test]
    fn test_validation_error_with_constraints() {
        let error = ValidationError::min_violation("age", 15, 18);
        assert_eq!(error.constraints.get("min"), Some(&serde_json::json!(18)));
    }
}