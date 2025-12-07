//! JSON Serialization helpers for Vela types
//!
//! This module provides utilities for automatic JSON serialization
//! of Vela structs and types using functional programming patterns.

use crate::json::JsonValue;
use std::collections::HashMap;

/// Configuration for JSON field serialization
#[derive(Debug, Clone)]
pub struct JsonFieldConfig {
    /// Alternative name for the field in JSON
    pub name: Option<String>,
    /// Whether to skip this field during serialization
    pub skip: bool,
    /// Default value if field is missing during deserialization
    pub default_value: Option<JsonValue>,
}

impl Default for JsonFieldConfig {
    fn default() -> Self {
        Self {
            name: None,
            skip: false,
            default_value: None,
        }
    }
}

/// Configuration for JSON struct serialization
#[derive(Debug, Clone)]
pub struct JsonStructConfig {
    /// Field configurations
    pub fields: HashMap<String, JsonFieldConfig>,
}

impl Default for JsonStructConfig {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

/// Trait for types that can be serialized to JSON
pub trait JsonSerializable {
    fn to_json(&self) -> String;
    fn to_json_value(&self) -> JsonValue;
}

/// Trait for types that can be deserialized from JSON
pub trait JsonDeserializable: Sized {
    fn from_json(json: &str) -> Result<Self, String>;
    fn from_json_value(value: &JsonValue) -> Result<Self, String>;
}

/// Helper function to serialize a struct to JSON
/// This function takes a struct represented as a HashMap and serializes it
pub fn serialize_struct(fields: HashMap<String, JsonValue>, config: &JsonStructConfig) -> String {
    let mut json_obj = HashMap::new();
    let default_config = JsonFieldConfig::default();

    for (field_name, field_value) in fields {
        let field_config = config.fields.get(&field_name).unwrap_or(&default_config);

        if field_config.skip {
            continue;
        }

        let json_name = field_config.name.as_ref().unwrap_or(&field_name).clone();
        json_obj.insert(json_name, field_value);
    }

    JsonValue::Object(json_obj).to_json()
}

/// Helper function to deserialize a struct from JSON
/// This function takes a JSON string and returns a HashMap representing the struct
pub fn deserialize_struct(json: &str, config: &JsonStructConfig) -> Result<HashMap<String, JsonValue>, String> {
    let parsed = crate::json::parse(json).map_err(|e| format!("Parse error: {}", e))?;

    match parsed {
        JsonValue::Object(obj) => {
            let mut result = HashMap::new();

            // First pass: collect all fields that exist in JSON
            for (json_name, json_value) in obj {
                // Find the field name that maps to this JSON name
                let field_name = config.fields.iter()
                    .find(|(_, config)| config.name.as_ref() == Some(&json_name))
                    .map(|(name, _)| name.clone())
                    .unwrap_or(json_name);

                result.insert(field_name, json_value);
            }

            // Second pass: apply defaults for missing fields
            for (field_name, field_config) in &config.fields {
                if !result.contains_key(field_name) {
                    if let Some(default_value) = &field_config.default_value {
                        result.insert(field_name.clone(), default_value.clone());
                    } else if !field_config.skip {
                        return Err(format!("Missing required field: {}", field_name));
                    }
                }
            }

            Ok(result)
        }
        _ => Err("Expected JSON object".to_string()),
    }
}

/// Macro-like function to create JSON configuration for a struct
/// This simulates the decorator functionality in a functional way
pub fn json_struct_config(field_configs: Vec<(String, JsonFieldConfig)>) -> JsonStructConfig {
    let mut config = JsonStructConfig::default();
    for (field_name, field_config) in field_configs {
        config.fields.insert(field_name, field_config);
    }
    config
}

/// Helper to create a field configuration with custom name
pub fn json_field_name(name: String) -> JsonFieldConfig {
    JsonFieldConfig {
        name: Some(name),
        ..Default::default()
    }
}

/// Helper to create a field configuration that skips serialization
pub fn json_field_skip() -> JsonFieldConfig {
    JsonFieldConfig {
        skip: true,
        ..Default::default()
    }
}

/// Helper to create a field configuration with default value
pub fn json_field_default(value: JsonValue) -> JsonFieldConfig {
    JsonFieldConfig {
        default_value: Some(value),
        ..Default::default()
    }
}

/// Convenience function to serialize a simple struct without configuration
pub fn serialize_simple_struct(fields: HashMap<String, JsonValue>) -> String {
    serialize_struct(fields, &JsonStructConfig::default())
}

/// Convenience function to deserialize a simple struct without configuration
pub fn deserialize_simple_struct(json: &str) -> Result<HashMap<String, JsonValue>, String> {
    deserialize_struct(json, &JsonStructConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::JsonValue;

    #[test]
    fn test_serialize_simple_struct() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        fields.insert("age".to_string(), JsonValue::Number(25.0));
        fields.insert("active".to_string(), JsonValue::Bool(true));

        let json = serialize_simple_struct(fields);
        let expected = r#"{"active":true,"age":25,"name":"Alice"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_deserialize_simple_struct() {
        let json = r#"{"name":"Bob","age":30,"active":false}"#;
        let result = deserialize_simple_struct(json).unwrap();

        assert_eq!(result.get("name"), Some(&JsonValue::String("Bob".to_string())));
        assert_eq!(result.get("age"), Some(&JsonValue::Number(30.0)));
        assert_eq!(result.get("active"), Some(&JsonValue::Bool(false)));
    }

    #[test]
    fn test_serialize_with_custom_field_names() {
        let mut fields = HashMap::new();
        fields.insert("user_name".to_string(), JsonValue::String("Charlie".to_string()));
        fields.insert("user_age".to_string(), JsonValue::Number(35.0));

        let config = json_struct_config(vec![
            ("user_name".to_string(), json_field_name("name".to_string())),
            ("user_age".to_string(), json_field_name("age".to_string())),
        ]);

        let json = serialize_struct(fields, &config);
        let expected = r#"{"age":35,"name":"Charlie"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_deserialize_with_custom_field_names() {
        let json = r#"{"name":"David","age":40}"#;

        let config = json_struct_config(vec![
            ("user_name".to_string(), json_field_name("name".to_string())),
            ("user_age".to_string(), json_field_name("age".to_string())),
        ]);

        let result = deserialize_struct(json, &config).unwrap();

        assert_eq!(result.get("user_name"), Some(&JsonValue::String("David".to_string())));
        assert_eq!(result.get("user_age"), Some(&JsonValue::Number(40.0)));
    }

    #[test]
    fn test_skip_field() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), JsonValue::String("Eve".to_string()));
        fields.insert("password".to_string(), JsonValue::String("secret".to_string()));
        fields.insert("age".to_string(), JsonValue::Number(28.0));

        let config = json_struct_config(vec![
            ("password".to_string(), json_field_skip()),
        ]);

        let json = serialize_struct(fields, &config);
        // Password should not appear in JSON
        assert!(!json.contains("password"));
        assert!(json.contains("name"));
        assert!(json.contains("age"));
    }

    #[test]
    fn test_default_values() {
        let json = r#"{"name":"Frank"}"#; // Missing age and active fields

        let config = json_struct_config(vec![
            ("age".to_string(), json_field_default(JsonValue::Number(18.0))),
            ("active".to_string(), json_field_default(JsonValue::Bool(true))),
        ]);

        let result = deserialize_struct(json, &config).unwrap();

        assert_eq!(result.get("name"), Some(&JsonValue::String("Frank".to_string())));
        assert_eq!(result.get("age"), Some(&JsonValue::Number(18.0)));
        assert_eq!(result.get("active"), Some(&JsonValue::Bool(true)));
    }

    #[test]
    fn test_missing_required_field() {
        let json = r#"{"name":"Grace"}"#; // Missing age field without default

        let config = json_struct_config(vec![
            ("name".to_string(), JsonFieldConfig::default()),
            ("age".to_string(), JsonFieldConfig::default()), // Required, no default
        ]);

        let result = deserialize_struct(json, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field: age"));
    }

    #[test]
    fn test_round_trip_with_config() {
        let mut original_fields = HashMap::new();
        original_fields.insert("user_name".to_string(), JsonValue::String("Helen".to_string()));
        original_fields.insert("user_age".to_string(), JsonValue::Number(45.0));
        original_fields.insert("is_active".to_string(), JsonValue::Bool(true));

        let config = json_struct_config(vec![
            ("user_name".to_string(), json_field_name("name".to_string())),
            ("user_age".to_string(), json_field_name("age".to_string())),
            ("is_active".to_string(), json_field_name("active".to_string())),
        ]);

        // Serialize
        let json = serialize_struct(original_fields, &config);

        // Deserialize
        let deserialized_fields = deserialize_struct(&json, &config).unwrap();

        // Verify round trip
        assert_eq!(deserialized_fields.get("user_name"), Some(&JsonValue::String("Helen".to_string())));
        assert_eq!(deserialized_fields.get("user_age"), Some(&JsonValue::Number(45.0)));
        assert_eq!(deserialized_fields.get("is_active"), Some(&JsonValue::Bool(true)));
    }
}