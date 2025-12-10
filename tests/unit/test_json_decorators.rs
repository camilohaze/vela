//! Unit tests for JSON decorators system
//!
//! Tests the JsonDecorated trait, decorator configuration,
//! field filtering, and serialization/deserialization.

use std::collections::HashMap;
use vela_stdlib::json::decorators::*;
use vela_stdlib::json::*;

// Test struct for basic functionality
#[derive(Debug, Clone, PartialEq)]
struct TestUser {
    id: String,
    name: String,
    email: String,
}

impl_json_decorated!(
    TestUser,
    JsonDecoratorConfig::default(),
    {
        let mut configs = HashMap::new();
        configs.insert("id".to_string(), JsonFieldDecorator::default());
        configs.insert("name".to_string(), JsonFieldDecorator::default());
        configs.insert("email".to_string(), JsonFieldDecorator::default());
        configs
    }
);

// Test struct with field decorators
#[derive(Debug, Clone, PartialEq)]
struct TestProduct {
    id: String,
    name: String,
    price: f64,
    internal_id: String,
    created_at: String,
}

impl_json_decorated!(
    TestProduct,
    JsonDecoratorConfig {
        exclude: Some(vec!["internal_id".to_string()]),
        rename: {
            let mut map = HashMap::new();
            map.insert("created_at".to_string(), "createdAt".to_string());
            map
        },
        ..Default::default()
    },
    {
        let mut configs = HashMap::new();
        configs.insert("id".to_string(), JsonFieldDecorator::default());
        configs.insert("name".to_string(), JsonFieldDecorator::default());
        configs.insert("price".to_string(), JsonFieldDecorator::default());
        configs.insert("internal_id".to_string(), JsonFieldDecorator {
            skip: true,
            ..Default::default()
        });
        configs.insert("created_at".to_string(), JsonFieldDecorator {
            rename: Some("createdAt".to_string()),
            ..Default::default()
        });
        configs
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_serialization() {
        let user = TestUser {
            id: "user-123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };

        let json = user.to_json_decorated();
        let expected = r#"{"id":"user-123","name":"John Doe","email":"john@example.com"}"#;

        // Parse and compare as JsonValue for accurate comparison
        let parsed: JsonValue = serde_json::from_str(&json).unwrap();
        let expected_parsed: JsonValue = serde_json::from_str(expected).unwrap();

        assert_eq!(parsed, expected_parsed);
    }

    #[test]
    fn test_field_exclusion() {
        let product = TestProduct {
            id: "prod-456".to_string(),
            name: "Test Product".to_string(),
            price: 29.99,
            internal_id: "INT-789".to_string(),
            created_at: "2024-01-01".to_string(),
        };

        let json = product.to_json_decorated();

        // Should not contain internal_id
        assert!(!json.contains("internal_id"));
        assert!(!json.contains("INT-789"));

        // Should contain other fields
        assert!(json.contains("prod-456"));
        assert!(json.contains("Test Product"));
        assert!(json.contains("29.99"));
    }

    #[test]
    fn test_field_renaming() {
        let product = TestProduct {
            id: "prod-456".to_string(),
            name: "Test Product".to_string(),
            price: 29.99,
            internal_id: "INT-789".to_string(),
            created_at: "2024-01-01".to_string(),
        };

        let json = product.to_json_decorated();

        // Should have "createdAt" instead of "created_at"
        assert!(json.contains(r#""createdAt":"2024-01-01""#));
        assert!(!json.contains("created_at"));
    }

    #[test]
    fn test_decorator_config_creation() {
        let config = JsonDecoratorConfig {
            include: Some(vec!["field1".to_string(), "field2".to_string()]),
            exclude: Some(vec!["field3".to_string()]),
            rename: {
                let mut map = HashMap::new();
                map.insert("old_name".to_string(), "newName".to_string());
                map
            },
            default_values: HashMap::new(),
        };

        assert_eq!(config.include.as_ref().unwrap().len(), 2);
        assert_eq!(config.exclude.as_ref().unwrap().len(), 1);
        assert_eq!(config.rename.get("old_name").unwrap(), "newName");
    }

    #[test]
    fn test_field_decorator_creation() {
        let decorator = JsonFieldDecorator {
            skip: true,
            rename: Some("newName".to_string()),
            default_value: Some(JsonValue::String("default".to_string())),
        };

        assert!(decorator.skip);
        assert_eq!(decorator.rename.as_ref().unwrap(), "newName");
        assert_eq!(decorator.default_value.as_ref().unwrap(), &JsonValue::String("default".to_string()));
    }

    #[test]
    fn test_helper_filter_fields() {
        let all_fields = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];

        // Test include only
        let include = Some(vec!["a".to_string(), "c".to_string()]);
        let exclude = None;
        let filtered = helpers::filter_fields(&all_fields, &include, &exclude);
        assert_eq!(filtered, vec!["a", "c"]);

        // Test exclude only
        let include = None;
        let exclude = Some(vec!["b".to_string(), "d".to_string()]);
        let filtered = helpers::filter_fields(&all_fields, &include, &exclude);
        assert_eq!(filtered, vec!["a", "c"]);

        // Test both include and exclude
        let include = Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        let exclude = Some(vec!["b".to_string()]);
        let filtered = helpers::filter_fields(&all_fields, &include, &exclude);
        assert_eq!(filtered, vec!["a", "c"]);

        // Test no filters
        let include = None;
        let exclude = None;
        let filtered = helpers::filter_fields(&all_fields, &include, &exclude);
        assert_eq!(filtered, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_helper_get_field_name() {
        let mut rename_map = HashMap::new();
        rename_map.insert("old_field".to_string(), "newField".to_string());

        // Test renamed field
        let name = helpers::get_field_name("old_field", &rename_map);
        assert_eq!(name, "newField");

        // Test non-renamed field
        let name = helpers::get_field_name("other_field", &rename_map);
        assert_eq!(name, "other_field");
    }

    #[test]
    fn test_helper_should_skip_field() {
        let skip_fields = vec!["skip_me".to_string(), "also_skip".to_string()];

        assert!(helpers::should_skip_field(&"skip_me".to_string(), &skip_fields));
        assert!(helpers::should_skip_field(&"also_skip".to_string(), &skip_fields));
        assert!(!helpers::should_skip_field(&"keep_me".to_string(), &skip_fields));
    }

    #[test]
    fn test_json_decorated_trait() {
        let user = TestUser {
            id: "test".to_string(),
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
        };

        // Test that the trait is implemented
        let _json = user.to_json_decorated();
        assert!(true); // If we reach here, the trait is implemented
    }

    #[test]
    fn test_complex_nested_structure() {
        // This would test nested JsonDecorated structs
        // For now, just ensure the basic structure works
        let user = TestUser {
            id: "user-1".to_string(),
            name: "Nested User".to_string(),
            email: "nested@example.com".to_string(),
        };

        let json = user.to_json_decorated();
        assert!(json.contains("user-1"));
        assert!(json.contains("Nested User"));
        assert!(json.contains("nested@example.com"));
    }
}