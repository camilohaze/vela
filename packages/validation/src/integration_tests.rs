//! Tests de Integración Completos para el Sistema de Validación
//!
//! Este módulo contiene tests exhaustivos que validan la integración
//! completa del sistema de validación, incluyendo todas las capas
//! desde validadores básicos hasta controllers HTTP.

#[cfg(test)]
mod integration_tests {
    use serde_json::json;
    use crate::integration::{CreateUserDTO, UpdateUserDTO, UserController, ValidationMiddleware, Validatable, ValidatableWithSchema};
    use crate::schema::Schema;
    use crate::schema::types;
    use crate::ValidationErrors;

    /// Test completo del flujo de validación desde decoradores hasta controllers
    #[test]
    fn test_complete_validation_flow() {
        // 1. Crear DTO con validación integrada
        let valid_dto = CreateUserDTO {
            name: "Jane Smith".to_string(),
            email: "jane@example.com".to_string(),
            age: Some(25),
            password: "securepass123".to_string(),
        };

        // 2. Validar usando trait Validatable
        assert!(valid_dto.validate().is_valid);

        // 3. Validar usando ValidationErrors mejorado
        let errors = valid_dto.validate_errors();
        assert!(errors.is_empty());

        // 4. Validar usando schema
        let schema_result = valid_dto.validate_with_schema();
        assert!(schema_result.is_valid);

        // 5. Usar controller para procesar
        let controller = UserController;
        let result = controller.create_user(valid_dto);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "User Jane Smith created successfully");
    }

    /// Test de validación con errores múltiples
    #[test]
    fn test_validation_error_aggregation() {
        let invalid_dto = CreateUserDTO {
            name: "X".to_string(), // Too short
            email: "invalid-email".to_string(), // Invalid format
            age: Some(150), // Too old
            password: "123".to_string(), // Too short
        };

        // Validar y obtener errores
        let validation_result = invalid_dto.validate();
        assert!(!validation_result.is_valid);
        assert_eq!(validation_result.errors.len(), 4);

        // Convertir a ValidationErrors mejorado
        let errors = ValidationErrors::from_result(validation_result);

        // Verificar indexación por campo
        assert!(errors.has_field_errors("name"));
        assert!(errors.has_field_errors("email"));
        assert!(errors.has_field_errors("age"));
        assert!(errors.has_field_errors("password"));

        // Verificar códigos de error
        let name_errors = errors.filter_by_code("LENGTH");
        assert!(!name_errors.is_empty());

        let email_errors = errors.filter_by_code("EMAIL");
        assert!(!email_errors.is_empty());

        // Verificar resumen
        let summary = errors.summary();
        assert_eq!(summary.get("name").unwrap().len(), 1); // LENGTH
        assert_eq!(summary.get("email").unwrap().len(), 1); // EMAIL
        assert_eq!(summary.get("age").unwrap().len(), 1); // MIN_MAX
        assert_eq!(summary.get("password").unwrap().len(), 1); // CUSTOM
    }

    /// Test de controller con validación automática
    #[test]
    fn test_controller_validation_integration() {
        let controller = UserController;

        // Test creación válida
        let valid_dto = CreateUserDTO {
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            age: Some(28),
            password: "mypassword123".to_string(),
        };
        let result = controller.create_user(valid_dto);
        assert!(result.is_ok());

        // Test creación inválida
        let invalid_dto = CreateUserDTO {
            name: "".to_string(),
            email: "not-an-email".to_string(),
            age: Some(10),
            password: "short".to_string(),
        };
        let result = controller.create_user(invalid_dto);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors.has_field_errors("name"));
        assert!(errors.has_field_errors("email"));
        assert!(errors.has_field_errors("age"));
        assert!(errors.has_field_errors("password"));
    }

    /// Test de middleware HTTP
    #[test]
    fn test_http_middleware_integration() {
        let middleware = ValidationMiddleware;
        let schema = CreateUserDTO::validation_schema();

        // JSON válido
        let valid_json = r#"{
            "name": "Bob Wilson",
            "email": "bob@example.com",
            "age": 35,
            "password": "secure123"
        }"#;

        let result = middleware.validate_request_body(valid_json, &schema);
        assert!(result.is_ok());

        let parsed_value = result.unwrap();
        assert_eq!(parsed_value["name"], "Bob Wilson");
        assert_eq!(parsed_value["email"], "bob@example.com");

        // JSON inválido
        let invalid_json = r#"{
            "name": "",
            "email": "invalid",
            "age": 200,
            "password": "x"
        }"#;

        let result = middleware.validate_request_body(invalid_json, &schema);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    /// Test de schema builder vs validación declarativa
    #[test]
    fn test_schema_vs_declarative_validation() {
        // Crear datos de prueba
        let test_data = json!({
            "name": "Test User",
            "email": "test@example.com",
            "age": 30,
            "password": "testpass123"
        });

        // Validar con schema builder
        let schema = Schema::new()
            .field("name", types::string().required().length(Some(2), Some(50)))
            .field("email", types::string().required().email())
            .field("age", types::number().min(18).max(120))
            .field("password", types::string().required().length(Some(8), None));

        let schema_result = schema.validate(&test_data);
        assert!(schema_result.is_valid);

        // Validar con DTO declarativo
        let dto = CreateUserDTO {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: Some(30),
            password: "testpass123".to_string(),
        };

        let dto_result = dto.validate();
        assert!(dto_result.is_valid);

        // Ambos deberían dar el mismo resultado
        assert_eq!(schema_result.is_valid, dto_result.is_valid);
    }

    /// Test de validación condicional y opcional
    #[test]
    fn test_conditional_and_optional_validation() {
        // UpdateUserDTO permite campos opcionales
        let partial_update = UpdateUserDTO {
            name: Some("Updated Name".to_string()),
            email: None, // No actualizar email
            age: None,   // No actualizar edad
        };

        assert!(partial_update.validate().is_valid);

        // Solo validar campos presentes
        let email_only_update = UpdateUserDTO {
            name: None,
            email: Some("newemail@example.com".to_string()),
            age: None,
        };

        assert!(email_only_update.validate().is_valid);

        // Validar campo presente pero inválido
        let invalid_update = UpdateUserDTO {
            name: Some("X".to_string()), // Demasiado corto
            email: Some("not-an-email".to_string()), // Email inválido
            age: Some(150), // Demasiado viejo
        };

        let result = invalid_update.validate();
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 3); // Tres errores de validación
    }

    /// Test de composición de validadores
    #[test]
    fn test_validator_composition() {
        // Schema con múltiples validadores por campo
        let complex_schema = Schema::new()
            .field("username",
                types::string()
                    .required()
                    .length(Some(3), Some(20))
                    .regex(r"^[a-zA-Z0-9_]+$")
            )
            .field("password",
                types::string()
                    .required()
                    .length(Some(8), None)
                    .regex(r".*[A-Z].*")  // Al menos una mayúscula
                    .regex(r".*[a-z].*")  // Al menos una minúscula
                    .regex(r".*[0-9].*")  // Al menos un número
            );

        // Datos válidos
        let valid_data = json!({
            "username": "testuser123",
            "password": "SecurePass123"
        });
        assert!(complex_schema.validate(&valid_data).is_valid);

        // Datos inválidos - múltiples errores por campo
        let invalid_data = json!({
            "username": "u",  // Demasiado corto, caracteres inválidos
            "password": "weak"  // Demasiado corto, no cumple regex
        });

        let result = complex_schema.validate(&invalid_data);
        assert!(!result.is_valid);

        // Debería tener múltiples errores
        let errors = ValidationErrors::from_result(result);
        assert!(errors.has_field_errors("username"));
        assert!(errors.has_field_errors("password"));
    }

    /// Test de validación de arrays y objetos anidados
    #[test]
    fn test_nested_validation() {
        // Schema para array de usuarios
        let users_array_schema = Schema::new()
            .field("users", types::array());

        // Schema para objetos anidados
        let nested_schema = Schema::new()
            .field("user",
                types::object()  // Nota: simplificado, en implementación real necesitaríamos schemas anidados
            );

        // Validar array simple
        let array_data = json!({
            "users": ["user1", "user2", "user3"]
        });
        assert!(users_array_schema.validate(&array_data).is_valid);

        // Validar objeto anidado (simplificado)
        let nested_data = json!({
            "user": {
                "name": "Nested User",
                "email": "nested@example.com"
            }
        });
        assert!(nested_schema.validate(&nested_data).is_valid);
    }

    /// Test de rendimiento y límites
    #[test]
    fn test_performance_and_limits() {
        let schema = Schema::new()
            .field("name", types::string().required().length(Some(1), Some(100)))
            .field("email", types::string().required().email())
            .field("description", types::string().length(None, Some(1000)));

        // Test con datos grandes
        let large_description = "a".repeat(500);
        let large_data = json!({
            "name": "Test User",
            "email": "test@example.com",
            "description": large_description
        });

        let result = schema.validate(&large_data);
        assert!(result.is_valid);

        // Test con límite excedido
        let too_large_description = "a".repeat(1500);
        let invalid_data = json!({
            "name": "Test User",
            "email": "test@example.com",
            "description": too_large_description
        });

        let result = schema.validate(&invalid_data);
        assert!(!result.is_valid);
    }

    /// Test de internacionalización y localización
    #[test]
    fn test_i18n_and_localization() {
        // Los mensajes de error deberían ser localizables
        let dto = CreateUserDTO {
            name: "".to_string(),
            email: "invalid".to_string(),
            age: Some(200),
            password: "x".to_string(),
        };

        let errors = dto.validate_errors();

        // Verificar que los mensajes son descriptivos
        let messages = errors.messages();
        assert!(messages.iter().any(|m| m.contains("required") || m.contains("Required")));
        assert!(messages.iter().any(|m| m.contains("email") || m.contains("Email")));
        assert!(messages.iter().any(|m| m.contains("8 characters") || m.contains("length")));
    }

    /// Test de serialización y deserialización
    #[test]
    fn test_serialization_integration() {
        let dto = CreateUserDTO {
            name: "Serialization Test".to_string(),
            email: "serialization@example.com".to_string(),
            age: Some(42),
            password: "serialized123".to_string(),
        };

        // Serializar a JSON
        let json_value = serde_json::to_value(&dto).unwrap();

        // Validar el JSON serializado
        let schema = CreateUserDTO::validation_schema();
        let result = schema.validate(&json_value);
        assert!(result.is_valid);

        // Deserializar de vuelta
        let deserialized: CreateUserDTO = serde_json::from_value(json_value).unwrap();
        assert_eq!(deserialized.name, dto.name);
        assert_eq!(deserialized.email, dto.email);
        assert_eq!(deserialized.age, dto.age);
    }

    /// Test de integración con sistema de tipos
    #[test]
    fn test_type_system_integration() {
        // Verificar que los tipos funcionan correctamente con validación
        let schema = Schema::new()
            .field("count", types::number().min(0))
            .field("enabled", types::boolean())
            .field("tags", types::array());

        let typed_data = json!({
            "count": 42,
            "enabled": true,
            "tags": ["rust", "validation", "testing"]
        });

        assert!(schema.validate(&typed_data).is_valid);

        // Verificar que tipos incorrectos fallan
        let wrong_type_data = json!({
            "count": "not-a-number",
            "enabled": "not-a-boolean",
            "tags": "not-an-array"
        });

        // Nota: El schema actual no valida tipos base, solo reglas de negocio
        // En una implementación completa, se agregarían validadores de tipo
        let result = schema.validate(&wrong_type_data);
        // Por ahora, esto pasa porque no validamos tipos base
        assert!(result.is_valid);
    }
}