//! Integración del Sistema de Validación con DTOs y Controllers
//!
//! Este módulo proporciona la integración del sistema de validación
//! con DTOs (Data Transfer Objects) y controllers HTTP, permitiendo
//! validación automática de requests y responses.

use crate::validation::error::ValidationResult;
use crate::validation::errors::ValidationErrors;
use crate::validation::schema::Schema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait para structs que pueden ser validadas
pub trait Validatable {
    /// Valida la instancia y retorna resultado
    fn validate(&self) -> ValidationResult;

    /// Valida la instancia y retorna ValidationErrors mejorado
    fn validate_errors(&self) -> ValidationErrors {
        ValidationErrors::from_result(self.validate())
    }
}

/// Trait para DTOs que tienen schemas de validación
pub trait ValidatableWithSchema {
    /// Retorna el schema de validación para este DTO
    fn validation_schema() -> Schema;

    /// Valida contra el schema definido
    fn validate_with_schema(&self) -> ValidationResult
    where
        Self: Serialize,
    {
        let schema = Self::validation_schema();
        let value = serde_json::to_value(self)
            .map_err(|_| ValidationResult::invalid_one(
                crate::validation::error::ValidationError::custom(
                    "serialization",
                    "Failed to serialize DTO for validation",
                    serde_json::Value::Null
                )
            ))
            .unwrap_or_else(|e| e);

        if value.is_valid {
            schema.validate(&serde_json::to_value(self).unwrap_or(serde_json::Value::Null))
        } else {
            value
        }
    }
}

/// DTO para creación de usuarios con validación integrada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDTO {
    pub name: String,
    pub email: String,
    pub age: Option<i32>,
    pub password: String,
}

impl Validatable for CreateUserDTO {
    fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // Validar nombre
        if self.name.trim().is_empty() {
            errors.push(crate::validation::error::ValidationError::required(
                "name", serde_json::json!(self.name)
            ));
        } else if self.name.len() < 2 || self.name.len() > 50 {
            errors.push(crate::validation::error::ValidationError::length(
                "name", serde_json::json!(self.name), Some(2), Some(50)
            ));
        }

        // Validar email
        if self.email.trim().is_empty() {
            errors.push(crate::validation::error::ValidationError::required(
                "email", serde_json::json!(self.email)
            ));
        } else if !crate::validation::validators::is_valid_email(&self.email) {
            errors.push(crate::validation::error::ValidationError::email(
                "email", serde_json::json!(self.email)
            ));
        }

        // Validar edad
        if let Some(age) = self.age {
            if age < 18 || age > 120 {
                errors.push(crate::validation::error::ValidationError::min_max(
                    "age", serde_json::json!(age), Some(18), Some(120)
                ));
            }
        }

        // Validar password
        if self.password.len() < 8 {
            errors.push(crate::validation::error::ValidationError::custom(
                "password",
                "Password must be at least 8 characters long",
                serde_json::json!(self.password),
            ));
        }

        if errors.is_empty() {
            ValidationResult::valid()
        } else {
            ValidationResult::invalid(errors)
        }
    }
}

impl ValidatableWithSchema for CreateUserDTO {
    fn validation_schema() -> Schema {
        use crate::validation::schema::types::*;

        Schema::new()
            .field("name", string().required().length(Some(2), Some(50)))
            .field("email", string().required().email())
            .field("age", number().min(18).max(120))
            .field("password", string().required().length(Some(8), None))
    }
}

/// DTO para actualización de usuarios (campos opcionales)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserDTO {
    pub name: Option<String>,
    pub email: Option<String>,
    pub age: Option<i32>,
}

impl Validatable for UpdateUserDTO {
    fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // Validar nombre si está presente
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                errors.push(crate::validation::error::ValidationError::required(
                    "name", serde_json::json!(name)
                ));
            } else if name.len() < 2 || name.len() > 50 {
                errors.push(crate::validation::error::ValidationError::length(
                    "name", serde_json::json!(name), Some(2), Some(50)
                ));
            }
        }

        // Validar email si está presente
        if let Some(email) = &self.email {
            if email.trim().is_empty() {
                errors.push(crate::validation::error::ValidationError::required(
                    "email", serde_json::json!(email)
                ));
            } else if !crate::validation::validators::is_valid_email(email) {
                errors.push(crate::validation::error::ValidationError::email(
                    "email", serde_json::json!(email)
                ));
            }
        }

        // Validar edad si está presente
        if let Some(age) = self.age {
            if age < 18 || age > 120 {
                errors.push(crate::validation::error::ValidationError::min_max(
                    "age", serde_json::json!(age), Some(18), Some(120)
                ));
            }
        }

        if errors.is_empty() {
            ValidationResult::valid()
        } else {
            ValidationResult::invalid(errors)
        }
    }
}

/// Trait para controllers que manejan validación automática
pub trait ValidationController {
    /// Valida un DTO y retorna resultado o error HTTP
    fn validate_dto<T: Validatable>(&self, dto: &T) -> Result<(), ValidationErrors> {
        let validation_result = dto.validate_errors();
        if validation_result.is_empty() {
            Ok(())
        } else {
            Err(validation_result)
        }
    }

    /// Valida un DTO con schema y retorna resultado o error HTTP
    fn validate_dto_with_schema<T: ValidatableWithSchema + Serialize>(
        &self,
        dto: &T
    ) -> Result<(), ValidationErrors> {
        let validation_result = dto.validate_with_schema();
        if validation_result.is_valid {
            Ok(())
        } else {
            Err(ValidationErrors::from_result(validation_result))
        }
    }
}

/// Controller de ejemplo para usuarios
pub struct UserController;

impl UserController {
    /// Crear usuario con validación automática
    pub fn create_user(&self, dto: CreateUserDTO) -> Result<String, ValidationErrors> {
        // Validar DTO
        self.validate_dto(&dto)?;

        // Simular creación de usuario
        Ok(format!("User {} created successfully", dto.name))
    }

    /// Actualizar usuario con validación automática
    pub fn update_user(&self, id: i32, dto: UpdateUserDTO) -> Result<String, ValidationErrors> {
        // Validar DTO
        self.validate_dto(&dto)?;

        // Simular actualización
        Ok(format!("User {} updated successfully", id))
    }

    /// Crear usuario usando schema validation
    pub fn create_user_with_schema(&self, dto: CreateUserDTO) -> Result<String, ValidationErrors> {
        // Validar con schema
        self.validate_dto_with_schema(&dto)?;

        // Simular creación
        Ok(format!("User {} created with schema validation", dto.name))
    }
}

impl ValidationController for UserController {}

/// Middleware para validación automática en endpoints HTTP
pub struct ValidationMiddleware;

impl ValidationMiddleware {
    /// Valida request body como JSON contra un schema
    pub fn validate_request_body(
        &self,
        body: &str,
        schema: &Schema
    ) -> Result<serde_json::Value, ValidationErrors> {
        let value: serde_json::Value = serde_json::from_str(body)
            .map_err(|_| ValidationErrors::one(
                crate::validation::error::ValidationError::custom(
                    "body",
                    "Invalid JSON in request body",
                    serde_json::json!(body)
                )
            ))?;

        let validation_result = schema.validate(&value);
        if validation_result.is_valid {
            Ok(value)
        } else {
            Err(ValidationErrors::from_result(validation_result))
        }
    }

    /// Valida query parameters
    pub fn validate_query_params(
        &self,
        query: &str,
        schema: &Schema
    ) -> Result<HashMap<String, String>, ValidationErrors> {
        // Parse query string (simplified implementation)
        let params: HashMap<String, String> = query
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                Some((parts.next()?.to_string(), parts.next()?.to_string()))
            })
            .collect();

        let value = serde_json::to_value(&params)
            .map_err(|_| ValidationErrors::one(
                crate::validation::error::ValidationError::custom(
                    "query",
                    "Failed to parse query parameters",
                    serde_json::Value::Null
                )
            ))?;

        let validation_result = schema.validate(&value);
        if validation_result.is_valid {
            Ok(params)
        } else {
            Err(ValidationErrors::from_result(validation_result))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_dto_validation() {
        // Valid DTO
        let valid_dto = CreateUserDTO {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: Some(30),
            password: "password123".to_string(),
        };
        assert!(valid_dto.validate().is_valid);

        // Invalid DTO - empty name
        let invalid_dto = CreateUserDTO {
            name: "".to_string(),
            email: "john@example.com".to_string(),
            age: Some(30),
            password: "password123".to_string(),
        };
        assert!(!invalid_dto.validate().is_valid);

        // Invalid DTO - invalid email
        let invalid_dto2 = CreateUserDTO {
            name: "John Doe".to_string(),
            email: "invalid-email".to_string(),
            age: Some(30),
            password: "password123".to_string(),
        };
        assert!(!invalid_dto2.validate().is_valid);

        // Invalid DTO - age too low
        let invalid_dto3 = CreateUserDTO {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: Some(15),
            password: "password123".to_string(),
        };
        assert!(!invalid_dto3.validate().is_valid);
    }

    #[test]
    fn test_update_user_dto_validation() {
        // Valid DTO - all fields present
        let valid_dto = UpdateUserDTO {
            name: Some("John Doe".to_string()),
            email: Some("john@example.com".to_string()),
            age: Some(30),
        };
        assert!(valid_dto.validate().is_valid);

        // Valid DTO - only some fields
        let valid_dto2 = UpdateUserDTO {
            name: Some("John Doe".to_string()),
            email: None,
            age: None,
        };
        assert!(valid_dto2.validate().is_valid);

        // Invalid DTO - empty name
        let invalid_dto = UpdateUserDTO {
            name: Some("".to_string()),
            email: None,
            age: None,
        };
        assert!(!invalid_dto.validate().is_valid);
    }

    #[test]
    fn test_user_controller_validation() {
        let controller = UserController;

        // Valid creation
        let valid_dto = CreateUserDTO {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: Some(30),
            password: "password123".to_string(),
        };
        assert!(controller.create_user(valid_dto).is_ok());

        // Invalid creation
        let invalid_dto = CreateUserDTO {
            name: "".to_string(),
            email: "john@example.com".to_string(),
            age: Some(30),
            password: "password123".to_string(),
        };
        assert!(controller.create_user(invalid_dto).is_err());
    }

    #[test]
    fn test_validation_middleware() {
        let middleware = ValidationMiddleware;
        let schema = CreateUserDTO::validation_schema();

        // Valid JSON
        let valid_json = r#"{
            "name": "John Doe",
            "email": "john@example.com",
            "age": 30,
            "password": "password123"
        }"#;
        assert!(middleware.validate_request_body(valid_json, &schema).is_ok());

        // Invalid JSON - missing required field
        let invalid_json = r#"{
            "email": "john@example.com",
            "age": 30,
            "password": "password123"
        }"#;
        assert!(middleware.validate_request_body(invalid_json, &schema).is_err());
    }

    #[test]
    fn test_dto_schema_validation() {
        let dto = CreateUserDTO {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: Some(30),
            password: "password123".to_string(),
        };

        let result = dto.validate_with_schema();
        assert!(result.is_valid);

        let invalid_dto = CreateUserDTO {
            name: "X".to_string(), // Too short
            email: "invalid-email".to_string(),
            age: Some(15), // Too young
            password: "123".to_string(), // Too short
        };

        let result2 = invalid_dto.validate_with_schema();
        assert!(!result2.is_valid);
        assert_eq!(result2.errors.len(), 4); // 4 validation errors
    }
}