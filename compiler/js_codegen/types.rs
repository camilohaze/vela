//! Type System Mapping for JavaScript Code Generation
//!
//! Handles conversion between Vela types and JavaScript types.

use crate::ir::IRType;

/// Type mapping utilities for JavaScript generation
pub struct TypeGenerator;

impl TypeGenerator {
    /// Convert Vela IR type to JavaScript type annotation
    pub fn to_js_type(ty: &IRType) -> String {
        match ty {
            IRType::Void => "void".to_string(),
            IRType::Bool => "boolean".to_string(),
            IRType::Int => "number".to_string(),
            IRType::Float => "number".to_string(),
            IRType::String => "string".to_string(),
            IRType::Array(element_type) => format!("{}[]", Self::to_js_type(element_type)),
            IRType::Object(name) => Self::map_custom_type(name),
        }
    }

    /// Convert Vela IR type to JavaScript runtime type
    pub fn to_js_runtime_type(ty: &IRType) -> String {
        match ty {
            IRType::Void => "undefined".to_string(),
            IRType::Bool => "boolean".to_string(),
            IRType::Int | IRType::Float => "number".to_string(),
            IRType::String => "string".to_string(),
            IRType::Array(_) => "array".to_string(),
            IRType::Object(name) => Self::map_custom_type(name),
        }
    }

    /// Generate type guard function for runtime type checking
    pub fn generate_type_guard(ty: &IRType, value_expr: &str) -> String {
        match ty {
            IRType::Void => format!("{} === undefined", value_expr),
            IRType::Bool => format!("typeof {} === 'boolean'", value_expr),
            IRType::Int | IRType::Float => format!("typeof {} === 'number'", value_expr),
            IRType::String => format!("typeof {} === 'string'", value_expr),
            IRType::Array(element_type) => {
                format!("Array.isArray({}) && {}.every(item => {})",
                    value_expr,
                    value_expr,
                    Self::generate_type_guard(element_type, "item")
                )
            }
            IRType::Object(name) => {
                format!("{} && typeof {} === 'object' && {}.constructor.name === '{}'",
                    value_expr, value_expr, value_expr, name
                )
            }
        }
    }

    /// Generate default value for a type
    pub fn generate_default_value(ty: &IRType) -> String {
        match ty {
            IRType::Void => "undefined".to_string(),
            IRType::Bool => "false".to_string(),
            IRType::Int | IRType::Float => "0".to_string(),
            IRType::String => "\"\"".to_string(),
            IRType::Array(_) => "[]".to_string(),
            IRType::Object(name) => format!("new {}()", name),
        }
    }

    /// Map custom Vela types to JavaScript equivalents
    fn map_custom_type(name: &str) -> String {
        match name {
            // Vela built-in types
            "Option" => "Option".to_string(),
            "Result" => "Result".to_string(),
            "List" => "Array".to_string(),
            "Map" => "Map".to_string(),
            "Set" => "Set".to_string(),

            // User-defined types
            _ => name.to_string(),
        }
    }

    /// Check if a type needs special handling in JavaScript
    pub fn needs_special_handling(ty: &IRType) -> bool {
        matches!(ty, IRType::Object(name) if name == "Option" || name == "Result")
    }

    /// Generate type assertion function
    pub fn generate_type_assertion(ty: &IRType, value_expr: &str, error_msg: &str) -> String {
        let guard = Self::generate_type_guard(ty, value_expr);
        format!("if (!({})) {{ throw new Error({}); }}", guard, error_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::IRType;

    #[test]
    fn test_primitive_type_mapping() {
        assert_eq!(TypeGenerator::to_js_type(&IRType::Void), "void");
        assert_eq!(TypeGenerator::to_js_type(&IRType::Bool), "boolean");
        assert_eq!(TypeGenerator::to_js_type(&IRType::Int), "number");
        assert_eq!(TypeGenerator::to_js_type(&IRType::Float), "number");
        assert_eq!(TypeGenerator::to_js_type(&IRType::String), "string");
    }

    #[test]
    fn test_array_type_mapping() {
        let array_type = IRType::Array(Box::new(IRType::String));
        assert_eq!(TypeGenerator::to_js_type(&array_type), "string[]");

        let nested_array = IRType::Array(Box::new(IRType::Array(Box::new(IRType::Int))));
        assert_eq!(TypeGenerator::to_js_type(&nested_array), "number[][]");
    }

    #[test]
    fn test_object_type_mapping() {
        assert_eq!(TypeGenerator::to_js_type(&IRType::Object("User".to_string())), "User");
        assert_eq!(TypeGenerator::to_js_type(&IRType::Object("Option".to_string())), "Option");
        assert_eq!(TypeGenerator::to_js_type(&IRType::Object("Result".to_string())), "Result");
    }

    #[test]
    fn test_runtime_type_mapping() {
        assert_eq!(TypeGenerator::to_js_runtime_type(&IRType::Void), "undefined");
        assert_eq!(TypeGenerator::to_js_runtime_type(&IRType::Bool), "boolean");
        assert_eq!(TypeGenerator::to_js_runtime_type(&IRType::Int), "number");
        assert_eq!(TypeGenerator::to_js_runtime_type(&IRType::Float), "number");
        assert_eq!(TypeGenerator::to_js_runtime_type(&IRType::String), "string");
    }

    #[test]
    fn test_type_guards() {
        assert_eq!(
            TypeGenerator::generate_type_guard(&IRType::Bool, "value"),
            "typeof value === 'boolean'"
        );

        assert_eq!(
            TypeGenerator::generate_type_guard(&IRType::String, "value"),
            "typeof value === 'string'"
        );

        assert_eq!(
            TypeGenerator::generate_type_guard(&IRType::Void, "value"),
            "value === undefined"
        );
    }

    #[test]
    fn test_array_type_guard() {
        let array_type = IRType::Array(Box::new(IRType::Int));
        let guard = TypeGenerator::generate_type_guard(&array_type, "arr");

        assert!(guard.contains("Array.isArray(arr)"));
        assert!(guard.contains("arr.every"));
        assert!(guard.contains("typeof item === 'number'"));
    }

    #[test]
    fn test_object_type_guard() {
        let object_type = IRType::Object("User".to_string());
        let guard = TypeGenerator::generate_type_guard(&object_type, "obj");

        assert!(guard.contains("obj && typeof obj === 'object'"));
        assert!(guard.contains("obj.constructor.name === 'User'"));
    }

    #[test]
    fn test_default_values() {
        assert_eq!(TypeGenerator::generate_default_value(&IRType::Bool), "false");
        assert_eq!(TypeGenerator::generate_default_value(&IRType::Int), "0");
        assert_eq!(TypeGenerator::generate_default_value(&IRType::String), "\"\"");
        assert_eq!(TypeGenerator::generate_default_value(&IRType::Array(Box::new(IRType::Int))), "[]");
    }

    #[test]
    fn test_special_handling_detection() {
        assert!(TypeGenerator::needs_special_handling(&IRType::Object("Option".to_string())));
        assert!(TypeGenerator::needs_special_handling(&IRType::Object("Result".to_string())));
        assert!(!TypeGenerator::needs_special_handling(&IRType::Object("User".to_string())));
        assert!(!TypeGenerator::needs_special_handling(&IRType::String));
    }

    #[test]
    fn test_type_assertion() {
        let assertion = TypeGenerator::generate_type_assertion(
            &IRType::String,
            "input",
            "\"Expected string\""
        );

        assert!(assertion.contains("if (!("));
        assert!(assertion.contains("typeof input === 'string'"));
        assert!(assertion.contains("throw new Error"));
        assert!(assertion.contains("\"Expected string\""));
    }
}