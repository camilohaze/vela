"""
Tests unitarios para Config Decorators

Jira: VELA-609
Historia: VELA-609
"""

use crate::config_decorators::{ConfigDecoratorProcessor, ConfigCodeGenerator, ConfigClassInfo, ConfigFieldInfo};
use crate::ast::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_decorator_processor_creation() {
        let processor = ConfigDecoratorProcessor::new();
        assert!(processor.config_classes.is_empty());
    }

    #[test]
    fn test_process_class_decorators_with_config() {
        let mut processor = ConfigDecoratorProcessor::new();

        let class = ClassDeclaration {
            name: "AppConfig".to_string(),
            decorators: vec![Decorator {
                name: "config".to_string(),
                arguments: None,
            }],
            fields: vec![],
            methods: vec![],
            visibility: Visibility::Public,
            extends: None,
            implements: vec![],
        };

        let result = processor.process_class_decorators(&class);
        assert!(result.is_ok());
        assert!(processor.config_classes.contains_key("AppConfig"));
    }

    #[test]
    fn test_process_class_decorators_without_config() {
        let mut processor = ConfigDecoratorProcessor::new();

        let class = ClassDeclaration {
            name: "RegularClass".to_string(),
            decorators: vec![], // Sin @config
            fields: vec![],
            methods: vec![],
            visibility: Visibility::Public,
            extends: None,
            implements: vec![],
        };

        let result = processor.process_class_decorators(&class);
        assert!(result.is_ok());
        assert!(!processor.config_classes.contains_key("RegularClass"));
    }

    #[test]
    fn test_extract_config_field_info_required() {
        let processor = ConfigDecoratorProcessor::new();

        let field = FieldDeclaration {
            name: "api_key".to_string(),
            field_type: Type::Simple("String".to_string()),
            decorators: vec![Decorator {
                name: "required".to_string(),
                arguments: None,
            }],
            visibility: Visibility::Public,
        };

        let field_info = processor.extract_config_field_info(&field).unwrap();

        assert_eq!(field_info.name, "api_key");
        assert_eq!(field_info.key, "api_key"); // Default key
        assert!(field_info.required);
        assert!(field_info.validator.is_none());
    }

    #[test]
    fn test_extract_config_field_info_with_custom_key() {
        let processor = ConfigDecoratorProcessor::new();

        let field = FieldDeclaration {
            name: "database_url".to_string(),
            field_type: Type::Simple("String".to_string()),
            decorators: vec![Decorator {
                name: "key".to_string(),
                arguments: Some(vec![Expression::StringLiteral("db.connection.url".to_string())]),
            }],
            visibility: Visibility::Public,
        };

        let field_info = processor.extract_config_field_info(&field).unwrap();

        assert_eq!(field_info.name, "database_url");
        assert_eq!(field_info.key, "db.connection.url");
        assert!(!field_info.required);
    }

    #[test]
    fn test_extract_config_field_info_range_validator() {
        let processor = ConfigDecoratorProcessor::new();

        let field = FieldDeclaration {
            name: "port".to_string(),
            field_type: Type::Simple("Number".to_string()),
            decorators: vec![Decorator {
                name: "range".to_string(),
                arguments: Some(vec![
                    Expression::NamedArgument("min".to_string(), Box::new(Expression::NumberLiteral(1024.0))),
                    Expression::NamedArgument("max".to_string(), Box::new(Expression::NumberLiteral(65535.0))),
                ]),
            }],
            visibility: Visibility::Public,
        };

        let field_info = processor.extract_config_field_info(&field).unwrap();

        assert_eq!(field_info.name, "port");
        assert_eq!(field_info.validator, Some("RangeValidator { min: Some(1024), max: Some(65535) }".to_string()));
    }

    #[test]
    fn test_extract_config_field_info_min_validator() {
        let processor = ConfigDecoratorProcessor::new();

        let field = FieldDeclaration {
            name: "timeout".to_string(),
            field_type: Type::Simple("Number".to_string()),
            decorators: vec![Decorator {
                name: "min".to_string(),
                arguments: Some(vec![Expression::NumberLiteral(0.0)]),
            }],
            visibility: Visibility::Public,
        };

        let field_info = processor.extract_config_field_info(&field).unwrap();

        assert_eq!(field_info.validator, Some("RangeValidator { min: Some(0), max: None }".to_string()));
    }

    #[test]
    fn test_extract_config_field_info_max_validator() {
        let processor = ConfigDecoratorProcessor::new();

        let field = FieldDeclaration {
            name: "max_connections".to_string(),
            field_type: Type::Simple("Number".to_string()),
            decorators: vec![Decorator {
                name: "max".to_string(),
                arguments: Some(vec![Expression::NumberLiteral(100.0)]),
            }],
            visibility: Visibility::Public,
        };

        let field_info = processor.extract_config_field_info(&field).unwrap();

        assert_eq!(field_info.validator, Some("RangeValidator { min: None, max: Some(100) }".to_string()));
    }

    #[test]
    fn test_extract_config_field_info_email_validator() {
        let processor = ConfigDecoratorProcessor::new();

        let field = FieldDeclaration {
            name: "admin_email".to_string(),
            field_type: Type::Simple("String".to_string()),
            decorators: vec![Decorator {
                name: "email".to_string(),
                arguments: None,
            }],
            visibility: Visibility::Public,
        };

        let field_info = processor.extract_config_field_info(&field).unwrap();

        assert_eq!(field_info.validator, Some("EmailValidator".to_string()));
    }

    #[test]
    fn test_code_generator_generate_config_class() {
        let generator = ConfigCodeGenerator::new();

        let class_info = ConfigClassInfo {
            class_name: "TestConfig".to_string(),
            fields: vec![
                ConfigFieldInfo {
                    name: "app_name".to_string(),
                    field_type: "String".to_string(),
                    key: "app.name".to_string(),
                    required: true,
                    validator: None,
                },
                ConfigFieldInfo {
                    name: "port".to_string(),
                    field_type: "Number".to_string(),
                    key: "app.port".to_string(),
                    required: false,
                    validator: Some("RangeValidator { min: Some(1024), max: Some(65535) }".to_string()),
                },
                ConfigFieldInfo {
                    name: "debug".to_string(),
                    field_type: "Bool".to_string(),
                    key: "app.debug".to_string(),
                    required: false,
                    validator: None,
                },
            ],
            validations: vec![],
        };

        let code = generator.generate_config_class(&class_info);

        // Verificar que contiene elementos clave
        assert!(code.contains("pub struct TestConfig"));
        assert!(code.contains("pub fn load()"));
        assert!(code.contains("ConfigLoader::new()"));
        assert!(code.contains("RequiredValidator"));
        assert!(code.contains("RangeValidator"));
        assert!(code.contains("app_name:"));
        assert!(code.contains("port:"));
        assert!(code.contains("debug:"));
    }

    #[test]
    fn test_code_generator_type_mapping() {
        let generator = ConfigCodeGenerator::new();

        assert_eq!(generator.map_type_to_rust("Number"), "i64");
        assert_eq!(generator.map_type_to_rust("String"), "String");
        assert_eq!(generator.map_type_to_rust("Bool"), "bool");
        assert_eq!(generator.map_type_to_rust("Float"), "f64");
        assert_eq!(generator.map_type_to_rust("Unknown"), "String"); // Default
    }

    #[test]
    fn test_code_generator_field_getters() {
        let generator = ConfigCodeGenerator::new();

        let string_field = ConfigFieldInfo {
            name: "name".to_string(),
            field_type: "String".to_string(),
            key: "app.name".to_string(),
            required: false,
            validator: None,
        };

        let number_field = ConfigFieldInfo {
            name: "count".to_string(),
            field_type: "Number".to_string(),
            key: "app.count".to_string(),
            required: false,
            validator: None,
        };

        let bool_field = ConfigFieldInfo {
            name: "enabled".to_string(),
            field_type: "Bool".to_string(),
            key: "app.enabled".to_string(),
            required: false,
            validator: None,
        };

        assert!(generator.generate_field_getter(&string_field).contains("get_string"));
        assert!(generator.generate_field_getter(&number_field).contains("get_int"));
        assert!(generator.generate_field_getter(&bool_field).contains("get_bool"));
    }

    #[test]
    fn test_process_field_decorators() {
        let mut processor = ConfigDecoratorProcessor::new();

        // Primero crear una clase config
        let class = ClassDeclaration {
            name: "TestConfig".to_string(),
            decorators: vec![Decorator { name: "config".to_string(), arguments: None }],
            fields: vec![],
            methods: vec![],
            visibility: Visibility::Public,
            extends: None,
            implements: vec![],
        };

        processor.process_class_decorators(&class).unwrap();

        // Ahora procesar un campo
        let field = FieldDeclaration {
            name: "test_field".to_string(),
            field_type: Type::Simple("String".to_string()),
            decorators: vec![Decorator { name: "required".to_string(), arguments: None }],
            visibility: Visibility::Public,
        };

        processor.process_field_decorators("TestConfig", &field).unwrap();

        let class_info = processor.config_classes.get("TestConfig").unwrap();
        assert_eq!(class_info.fields.len(), 1);
        assert_eq!(class_info.fields[0].name, "test_field");
        assert!(class_info.fields[0].required);
    }
}