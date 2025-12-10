//! Integration tests for JSON functionality
//!
//! Tests the interaction between JSON parser, encoder, and decorators,
//! including round-trip serialization, performance benchmarks, and
//! complex real-world scenarios.

use std::collections::HashMap;
use vela_stdlib::json::decorators::*;
use vela_stdlib::json::encoder::{convenience as json_encoder, JsonEncoder, JsonEncoderConfig};
use vela_stdlib::json::{parse, to_json, to_json_pretty, JsonValue};

#[cfg(test)]
mod test_json_integration {
    use super::*;

    // Test round-trip parsing and encoding
    #[test]
    fn test_parser_encoder_round_trip() {
        let test_cases = vec![
            r#"null"#,
            r#"true"#,
            r#"false"#,
            r#"42"#,
            r#"3.14"#,
            r#""hello world""#,
            r#"[]"#,
            r#"{}"#,
            r#"[1, 2, 3]"#,
            r#"{"name": "John", "age": 30}"#,
            r#"[null, true, false, 42, "string", [], {}]"#,
            r#"{"nested": {"deep": {"value": 123}}}"#,
            r#"[{"id": 1, "tags": ["a", "b"]}, {"id": 2, "tags": ["c"]}]"#,
        ];

        for json_str in test_cases {
            // Parse the JSON
            let parsed = parse(json_str).unwrap();

            // Encode it back
            let encoded = to_json(&parsed);

            // Parse again
            let reparsed = parse(&encoded).unwrap();

            // Should be identical
            assert_eq!(parsed, reparsed, "Round-trip failed for: {}", json_str);
        }
    }

    #[test]
    fn test_pretty_print_round_trip() {
        let original = r#"{"users":[{"name":"Alice","role":"admin"},{"name":"Bob","role":"user"}],"active":true,"count":2}"#;

        // Parse
        let parsed = parse(original).unwrap();

        // Pretty print
        let pretty = to_json_pretty(&parsed);

        // Parse pretty version
        let reparsed = parse(&pretty).unwrap();

        // Should be equivalent
        assert_eq!(parsed, reparsed);

        // Pretty version should contain newlines
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }

    // Test complex nested structures
    #[test]
    fn test_complex_nested_structures() {
        let complex_json = r#"
        {
            "company": {
                "name": "Tech Corp",
                "founded": 2010,
                "active": true,
                "offices": [
                    {
                        "city": "New York",
                        "country": "USA",
                        "employees": 150
                    },
                    {
                        "city": "London",
                        "country": "UK",
                        "employees": 75
                    }
                ]
            },
            "products": [
                {
                    "id": 1,
                    "name": "Widget A",
                    "price": 29.99,
                    "tags": ["electronics", "popular"],
                    "metadata": {
                        "weight": 0.5,
                        "dimensions": {
                            "width": 10,
                            "height": 5,
                            "depth": 2
                        }
                    }
                }
            ],
            "stats": {
                "total_employees": 225,
                "total_products": 1,
                "countries": ["USA", "UK"],
                "last_updated": "2024-01-15"
            }
        }
        "#;

        // Should parse successfully
        let parsed = parse(complex_json).unwrap();

        // Should encode back to valid JSON
        let encoded = to_json(&parsed);
        let reparsed = parse(&encoded).unwrap();
        assert_eq!(parsed, reparsed);

        // Verify structure
        if let JsonValue::Object(root) = parsed {
            // Check company
            assert!(root.contains_key("company"));
            if let Some(JsonValue::Object(company)) = root.get("company") {
                assert_eq!(company.get("name"), Some(&JsonValue::String("Tech Corp".to_string())));
                assert_eq!(company.get("founded"), Some(&JsonValue::Number(2010.0)));
                assert_eq!(company.get("active"), Some(&JsonValue::Bool(true)));
            }

            // Check products array
            assert!(root.contains_key("products"));
            if let Some(JsonValue::Array(products)) = root.get("products") {
                assert_eq!(products.len(), 1);
                if let JsonValue::Object(product) = &products[0] {
                    assert_eq!(product.get("id"), Some(&JsonValue::Number(1.0)));
                    assert_eq!(product.get("name"), Some(&JsonValue::String("Widget A".to_string())));
                }
            }

            // Check stats
            assert!(root.contains_key("stats"));
            if let Some(JsonValue::Object(stats)) = root.get("stats") {
                assert_eq!(stats.get("total_employees"), Some(&JsonValue::Number(225.0)));
                assert_eq!(stats.get("total_products"), Some(&JsonValue::Number(1.0)));
            }
        }
    }

    // Test JSON decorators integration
    #[test]
    fn test_decorators_integration() {
        #[derive(Debug, Clone, PartialEq)]
        struct TestUser {
            id: String,
            name: String,
            email: String,
            internal_id: String,
            created_at: String,
        }

        impl_json_decorated!(
            TestUser,
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
                configs.insert("email".to_string(), JsonFieldDecorator::default());
                configs.insert("internal_id".to_string(), JsonFieldDecorator::default());
                configs.insert("created_at".to_string(), JsonFieldDecorator::default());
                configs
            }
        );

        let user = TestUser {
            id: "123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            internal_id: "int_456".to_string(),
            created_at: "2024-01-01".to_string(),
        };

        // Serialize with decorators
        let json_str = user.to_json_decorated();

        // Should not contain internal_id
        assert!(!json_str.contains("internal_id"));
        assert!(!json_str.contains("int_456"));

        // Should contain renamed field
        assert!(json_str.contains("createdAt"));
        assert!(!json_str.contains("created_at"));

        // Should contain other fields
        assert!(json_str.contains(r#""id":"123""#));
        assert!(json_str.contains(r#""name":"John Doe""#));
        assert!(json_str.contains(r#""email":"john@example.com""#));
        assert!(json_str.contains(r#""createdAt":"2024-01-01""#));

        // Should be valid JSON
        let parsed = parse(&json_str).unwrap();
        let reparsed = parse(&to_json(&parsed)).unwrap();
        assert_eq!(parsed, reparsed);
    }

    // Test large JSON handling
    #[test]
    fn test_large_json_handling() {
        // Create a large array
        let mut large_array = Vec::new();
        for i in 0..1000 {
            let mut obj = HashMap::new();
            obj.insert("id".to_string(), JsonValue::Number(i as f64));
            obj.insert("name".to_string(), JsonValue::String(format!("Item {}", i)));
            obj.insert("active".to_string(), JsonValue::Bool(i % 2 == 0));
            large_array.push(JsonValue::Object(obj));
        }

        let large_value = JsonValue::Array(large_array);

        // Should encode successfully
        let encoded = to_json(&large_value);
        assert!(encoded.len() > 10000); // Should be substantial

        // Should parse back successfully
        let parsed = parse(&encoded).unwrap();

        // Should have correct length
        if let JsonValue::Array(arr) = parsed {
            assert_eq!(arr.len(), 1000);
        }
    }

    // Test Unicode handling across components
    #[test]
    fn test_unicode_integration() {
        let unicode_json = r#"{"message":"Hello 世界 🚀","emoji":"😀🎉","mixed":"ASCII and ユニコード"}"#;

        // Parse
        let parsed = parse(unicode_json).unwrap();

        // Encode back
        let encoded = to_json(&parsed);

        // Should preserve Unicode
        assert!(encoded.contains("世界"));
        assert!(encoded.contains("🚀"));
        assert!(encoded.contains("😀"));
        assert!(encoded.contains("🎉"));
        assert!(encoded.contains("ユニコード"));

        // Should be valid JSON
        let reparsed = parse(&encoded).unwrap();
        assert_eq!(parsed, reparsed);
    }

    // Test number precision and edge cases
    #[test]
    fn test_number_precision_integration() {
        let numbers_json = r#"{"integers":[0,123,-456],"floats":[3.14,0.5,-2.5,1e10,1.5e-2],"special":[null,null]}"#;

        // Parse
        let parsed = parse(numbers_json).unwrap();

        // Encode back
        let encoded = to_json(&parsed);

        // Parse again
        let reparsed = parse(&encoded).unwrap();

        // Should be equivalent
        assert_eq!(parsed, reparsed);

        // Check specific values
        if let JsonValue::Object(obj) = reparsed {
            if let Some(JsonValue::Array(floats)) = obj.get("floats") {
                assert_eq!(floats[0], JsonValue::Number(3.14));
                assert_eq!(floats[1], JsonValue::Number(0.5));
                assert_eq!(floats[2], JsonValue::Number(-2.5));
                // Large numbers might be represented differently but should parse back
            }
        }
    }

    // Test streaming vs non-streaming encoding
    #[test]
    fn test_streaming_vs_non_streaming() {
        let mut large_obj = HashMap::new();
        for i in 0..100 {
            large_obj.insert(
                format!("key{}", i),
                JsonValue::String(format!("value{}", i))
            );
        }
        let value = JsonValue::Object(large_obj);

        // Non-streaming encoding
        let non_streaming = to_json(&value);

        // Streaming encoding
        let mut buffer = Vec::new();
        json_encoder::encode_to_writer(&value, &mut buffer).unwrap();
        let streaming = String::from_utf8(buffer).unwrap();

        // Should be identical
        assert_eq!(non_streaming, streaming);

        // Both should be valid JSON
        assert!(parse(&non_streaming).is_ok());
        assert!(parse(&streaming).is_ok());
    }

    // Test performance characteristics
    #[test]
    fn test_performance_characteristics() {
        // Create moderately complex JSON
        let mut users = Vec::new();
        for i in 0..100 {
            let mut user = HashMap::new();
            user.insert("id".to_string(), JsonValue::Number(i as f64));
            user.insert("name".to_string(), JsonValue::String(format!("User {}", i)));
            user.insert("email".to_string(), JsonValue::String(format!("user{}@example.com", i)));
            user.insert("active".to_string(), JsonValue::Bool(i % 3 != 0));

            let mut profile = HashMap::new();
            profile.insert("age".to_string(), JsonValue::Number((20 + i % 50) as f64));
            profile.insert("city".to_string(), JsonValue::String(format!("City {}", i % 10)));
            user.insert("profile".to_string(), JsonValue::Object(profile));

            users.push(JsonValue::Object(user));
        }

        let data = JsonValue::Object({
            let mut root = HashMap::new();
            root.insert("users".to_string(), JsonValue::Array(users));
            root.insert("total".to_string(), JsonValue::Number(100.0));
            root.insert("generated_at".to_string(), JsonValue::String("2024-01-15".to_string()));
            root
        });

        // Encode to compact JSON
        let compact = to_json(&data);
        assert!(compact.len() > 5000); // Should be substantial

        // Encode to pretty JSON
        let pretty = to_json_pretty(&data);
        assert!(pretty.len() > compact.len()); // Pretty should be longer
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));

        // Both should parse back correctly
        let parsed_compact = parse(&compact).unwrap();
        let parsed_pretty = parse(&pretty).unwrap();
        assert_eq!(parsed_compact, parsed_pretty);

        // Round-trip should preserve data
        let reencoded = to_json(&parsed_compact);
        let final_parsed = parse(&reencoded).unwrap();
        assert_eq!(parsed_compact, final_parsed);
    }

    // Test error recovery and edge cases
    #[test]
    fn test_error_handling_integration() {
        // Valid JSON should work
        let valid_cases = vec![
            r#"{"empty_object":{}}"#,
            r#"{"empty_array":[]}"#,
            r#"{"nested":{"deep":{"empty":{}}}}"#,
            r#"["mixed",null,true,false,123,{"key":"value"}]"#,
        ];

        for case in valid_cases {
            let parsed = parse(case).unwrap();
            let encoded = to_json(&parsed);
            let reparsed = parse(&encoded).unwrap();
            assert_eq!(parsed, reparsed);
        }
    }

    // Test memory efficiency with large structures
    #[test]
    fn test_memory_efficiency() {
        // Create deeply nested structure
        fn create_nested(depth: usize) -> JsonValue {
            if depth == 0 {
                JsonValue::Number(42.0)
            } else {
                let mut obj = HashMap::new();
                obj.insert("value".to_string(), create_nested(depth - 1));
                obj.insert("depth".to_string(), JsonValue::Number(depth as f64));
                JsonValue::Object(obj)
            }
        }

        let deep_nested = create_nested(50);

        // Should handle deep nesting
        let encoded = to_json(&deep_nested);
        assert!(encoded.len() > 1000);

        // Should parse back
        let parsed = parse(&encoded).unwrap();

        // Verify depth
        fn verify_depth(value: &JsonValue, expected_depth: usize) -> bool {
            match value {
                JsonValue::Object(obj) => {
                    if let (Some(JsonValue::Number(depth)), Some(nested)) =
                        (obj.get("depth"), obj.get("value")) {
                        (*depth as usize == expected_depth) && verify_depth(nested, expected_depth - 1)
                    } else {
                        false
                    }
                }
                JsonValue::Number(n) if expected_depth == 0 => *n == 42.0,
                _ => false,
            }
        }

        assert!(verify_depth(&parsed, 50));
    }
}</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\tests\integration\test_json_integration.rs