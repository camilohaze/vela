//! Config Decorators para Vela
//!
//! Implementación de: VELA-609
//! Historia: VELA-609
//! Fecha: 2025-12-11
//!
//! Descripción:
//! Decoradores compile-time para configuración type-safe.
//! Genera código que integra con ConfigLoader y validación automática.

use crate::ast::*;
use crate::error::CompileResult;
use std::collections::HashMap;

/// Processor para decoradores de configuración
pub struct ConfigDecoratorProcessor {
    config_classes: HashMap<String, ConfigClassInfo>,
}

#[derive(Debug, Clone)]
pub struct ConfigClassInfo {
    pub class_name: String,
    pub fields: Vec<ConfigFieldInfo>,
    pub validations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigFieldInfo {
    pub name: String,
    pub field_type: String,
    pub key: String,
    pub required: bool,
    pub validator: Option<String>,
}

impl ConfigDecoratorProcessor {
    pub fn new() -> Self {
        Self {
            config_classes: HashMap::new(),
        }
    }

    /// Procesar decoradores @config en clases
    pub fn process_class_decorators(&mut self, class: &ClassDeclaration) -> CompileResult<()> {
        // Buscar decorador @config
        let has_config_decorator = class.decorators.iter().any(|d| d.name == "config");

        if !has_config_decorator {
            return Ok(());
        }

        let class_info = self.extract_config_class_info(class)?;
        self.config_classes.insert(class.name.clone(), class_info);

        Ok(())
    }

    /// Procesar decoradores en campos de configuración
    pub fn process_field_decorators(&mut self, class_name: &str, field: &StructField) -> CompileResult<()> {
        let field_type = self.type_annotation_to_string(&field.type_annotation);
        
        if let Some(class_info) = self.config_classes.get_mut(class_name) {
            let field_info = ConfigFieldInfo {
                name: field.name.clone(),
                field_type,
                key: field.name.clone(), // Usar nombre del campo como key por defecto
                required: false, // Por ahora no required
                validator: None,
            };
            class_info.fields.push(field_info);
        }

        Ok(())
    }

    /// Extraer información de clase de configuración
    fn extract_config_class_info(&self, class: &ClassDeclaration) -> CompileResult<ConfigClassInfo> {
        let mut validations = Vec::new();

        // Buscar decoradores de validación en la clase
        for decorator in &class.decorators {
            match decorator.name.as_str() {
                "validate" => {
                    // @validate podría tener parámetros específicos
                    validations.push("validate_all".to_string());
                }
                _ => {}
            }
        }

        Ok(ConfigClassInfo {
            class_name: class.name.clone(),
            fields: Vec::new(),
            validations,
        })
    }

    /// Procesar una declaración de struct con decoradores de configuración
    pub fn process_struct(&mut self, struct_decl: &StructDeclaration) -> CompileResult<()> {
        // Verificar si tiene decorador @config
        let has_config_decorator = struct_decl.decorators.iter().any(|d| d.name == "config");
        
        if !has_config_decorator {
            return Ok(());
        }

        let mut fields = Vec::new();
        
        // Procesar campos (por ahora sin decoradores individuales)
        for field in &struct_decl.fields {
            let field_info = ConfigFieldInfo {
                name: field.name.clone(),
                field_type: self.type_annotation_to_string(&field.type_annotation),
                key: field.name.clone(), // Usar nombre del campo como key por defecto
                required: false, // Por ahora no required
                validator: None,
            };
            fields.push(field_info);
        }

        let class_info = ConfigClassInfo {
            class_name: struct_decl.name.clone(),
            fields,
            validations: Vec::new(),
        };
        
        self.config_classes.insert(struct_decl.name.clone(), class_info);
        Ok(())
    }

    /// Convertir TypeAnnotation a string
    fn type_annotation_to_string(&self, type_annotation: &TypeAnnotation) -> String {
        match type_annotation {
            TypeAnnotation::Named(named) => named.name.clone(),
            TypeAnnotation::Primitive(primitive) => primitive.name.clone(),
            _ => "String".to_string(), // Default
        }
    }
}

/// Generador de código para configuración
pub struct ConfigCodeGenerator;

impl ConfigCodeGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generar código para una clase de configuración
    pub fn generate_config_class(&self, class_info: &ConfigClassInfo) -> String {
        let mut lines = Vec::new();

        // Generar struct
        lines.push(format!("pub struct {} {{", class_info.class_name));

        for field in &class_info.fields {
            lines.push(format!("    pub {}: {},", field.name, self.map_type_to_rust(&field.field_type)));
        }

        lines.push("}".to_string());
        lines.push("".to_string());

        // Generar implementación
        lines.push(format!("impl {} {{", class_info.class_name));

        // Constructor
        lines.push("    pub fn load() -> Result<Self, ConfigError> {".to_string());
        lines.push("        let mut loader = ConfigLoader::new();".to_string());
        lines.push("".to_string());

        // Agregar validadores
        for field in &class_info.fields {
            if let Some(validator) = &field.validator {
                let line = format!("        loader = loader.add_validator(\"{}\".to_string(), ", field.key) + validator + ");";
                lines.push(line);
            } else if field.required {
                lines.push(format!("        loader = loader.add_validator(\"{}\".to_string(), RequiredValidator);", field.key));
            }
        }

        lines.push("        loader.load()?;".to_string());
        lines.push("".to_string());

        // Construir instancia
        lines.push(format!("        Ok({} {{", class_info.class_name));

        for field in &class_info.fields {
            let getter = self.generate_field_getter(field);
            lines.push(format!("            {}: {},", field.name, getter));
        }

        lines.push("        })".to_string());
        lines.push("    }".to_string());
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generar getter para un campo
    fn generate_field_getter(&self, field: &ConfigFieldInfo) -> String {
        match field.field_type.as_str() {
            "Number" => format!("loader.get_int(\"{}\").unwrap().unwrap_or(0) as Number", field.key),
            "String" => format!("loader.get_string(\"{}\").unwrap_or_else(|| \"default\".to_string())", field.key),
            "Bool" => format!("loader.get_bool(\"{}\").unwrap().unwrap_or(false)", field.key),
            "Float" => format!("loader.get_string(\"{}\").unwrap().parse().unwrap_or(0.0)", field.key),
            _ => format!("loader.get_string(\"{}\").unwrap_or_else(|| \"default\".to_string())", field.key),
        }
    }

    /// Mapear tipos de Vela a Rust
    fn map_type_to_rust(&self, vela_type: &str) -> &str {
        match vela_type {
            "Number" => "i64",
            "String" => "String",
            "Bool" => "bool",
            "Float" => "f64",
            _ => "String", // Default
        }
    }

    /// Generar código de uso del config loader
    pub fn generate_config_usage(&self, class_info: &ConfigClassInfo) -> String {
        format!(r#"
// Uso de configuración
let config = {}::load().expect("Failed to load config");
println!("App name: {{}}", config.app_name);
println!("Port: {{}}", config.port);
println!("Debug: {{}}", config.debug);
"#, class_info.class_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_decorator_processor_creation() {
        let processor = ConfigDecoratorProcessor::new();
        assert!(processor.config_classes.is_empty());
    }

    #[test]
    fn test_extract_config_field_info() {
        let processor = ConfigDecoratorProcessor::new();

        // Simular un field con decoradores
        let field_decorators = vec![
            Decorator {
                name: "required".to_string(),
                arguments: None,
            },
            Decorator {
                name: "key".to_string(),
                arguments: Some(vec![Expression::StringLiteral("app.name".to_string())]),
            },
        ];

        let field = FieldDeclaration {
            name: "app_name".to_string(),
            field_type: Type::Simple("String".to_string()),
            decorators: field_decorators,
            visibility: Visibility::Public,
        };

        let field_info = processor.extract_config_field_info(&field).unwrap();

        assert_eq!(field_info.name, "app_name");
        assert_eq!(field_info.key, "app.name");
        assert!(field_info.required);
    }

    #[test]
    fn test_code_generator() {
        let generator = ConfigCodeGenerator::new();

        let class_info = ConfigClassInfo {
            class_name: "AppConfig".to_string(),
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
            ],
            validations: vec![],
        };

        let code = generator.generate_config_class(&class_info);
        assert!(code.contains("pub struct AppConfig"));
        assert!(code.contains("pub fn load()"));
        assert!(code.contains("RequiredValidator"));
        assert!(code.contains("RangeValidator"));
    }
}