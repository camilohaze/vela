//! Unit tests for JSON encoder
//!
//! Tests the JsonEncoder implementation, JsonValue.to_json() method,
//! pretty printing, streaming encoding, and edge cases.

use std::collections::HashMap;
use std::io::Cursor;
use vela_stdlib::json::encoder::{JsonEncoder, JsonEncoderConfig, convenience};
use vela_stdlib::json::{JsonValue, to_json, to_json_pretty, to_json_sorted};

#[cfg(test)]
mod test_json_encoder {
    use super::*;

    // Test basic encoding via JsonValue.to_json()
    #[test]
    fn test_basic_to_json() {
        assert_eq!(JsonValue::Null.to_json(), "null");
        assert_eq!(JsonValue::Bool(true).to_json(), "true");
        assert_eq!(JsonValue::Bool(false).to_json(), "false");
        assert_eq!(JsonValue::Number(42.0).to_json(), "42");
        assert_eq!(JsonValue::Number(3.14).to_json(), "3.14");
        assert_eq!(JsonValue::String("hello".to_string()).to_json(), r#""hello""#);
    }

    #[test]
    fn test_array_to_json() {
        let arr = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(arr.to_json(), "[1,2,3]");
    }

    #[test]
    fn test_empty_array_to_json() {
        let arr = JsonValue::Array(vec![]);
        assert_eq!(arr.to_json(), "[]");
    }

    #[test]
    fn test_object_to_json() {
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), JsonValue::String("John".to_string()));
        obj.insert("age".to_string(), JsonValue::Number(30.0));
        let json_obj = JsonValue::Object(obj);
        let result = json_obj.to_json();

        // Order might vary, so check both possibilities
        assert!(result == r#"{"name":"John","age":30}"# || result == r#"{"age":30,"name":"John"}"#);
    }

    #[test]
    fn test_empty_object_to_json() {
        let obj = JsonValue::Object(HashMap::new());
        assert_eq!(obj.to_json(), "{}");
    }

    // Test string escaping
    #[test]
    fn test_string_escaping() {
        assert_eq!(JsonValue::String(r#"quote " here"#.to_string()).to_json(), r#""quote \" here""#);
        assert_eq!(JsonValue::String(r#"backslash \ here"#.to_string()).to_json(), r#""backslash \\ here""#);
        assert_eq!(JsonValue::String("newline\nhere".to_string()).to_json(), r#""newline\nhere""#);
        assert_eq!(JsonValue::String("tab\there".to_string()).to_json(), r#""tab\there""#);
        assert_eq!(JsonValue::String("backspace\x08here".to_string()).to_json(), r#""backspace\bhere""#);
        assert_eq!(JsonValue::String("formfeed\x0chere".to_string()).to_json(), r#""formfeed\fhere""#);
        assert_eq!(JsonValue::String("carriage\rreturn".to_string()).to_json(), r#""carriage\rreturn""#);
    }

    #[test]
    fn test_unicode_characters() {
        assert_eq!(JsonValue::String("🚀".to_string()).to_json(), r#""🚀""#);
        assert_eq!(JsonValue::String("Hello 世界".to_string()).to_json(), r#""Hello 世界""#);
    }

    #[test]
    fn test_control_characters() {
        // Control characters should be escaped as \uXXXX
        let control = "\x01\x02\x03"; // Some control characters
        let result = JsonValue::String(control.to_string()).to_json();
        assert!(result.contains(r#"\u0001"#));
        assert!(result.contains(r#"\u0002"#));
        assert!(result.contains(r#"\u0003"#));
    }

    // Test number encoding edge cases
    #[test]
    fn test_number_encoding() {
        // Integers
        assert_eq!(JsonValue::Number(0.0).to_json(), "0");
        assert_eq!(JsonValue::Number(123.0).to_json(), "123");
        assert_eq!(JsonValue::Number(-456.0).to_json(), "-456");

        // Floats
        assert_eq!(JsonValue::Number(3.14159).to_json(), "3.14159");
        assert_eq!(JsonValue::Number(0.5).to_json(), "0.5");
        assert_eq!(JsonValue::Number(-2.5).to_json(), "-2.5");

        // Scientific notation
        assert_eq!(JsonValue::Number(1e10).to_json(), "10000000000");
        assert_eq!(JsonValue::Number(1.5e-2).to_json(), "0.015");

        // Very large numbers (should use scientific notation)
        let large = JsonValue::Number(1e20).to_json();
        assert!(large.contains("e") || large.contains("E"));
    }

    #[test]
    fn test_special_numbers() {
        // NaN should become null
        assert_eq!(JsonValue::Number(f64::NAN).to_json(), "null");

        // Infinity should become null
        assert_eq!(JsonValue::Number(f64::INFINITY).to_json(), "null");
        assert_eq!(JsonValue::Number(f64::NEG_INFINITY).to_json(), "null");
    }

    // Test convenience functions
    #[test]
    fn test_convenience_functions() {
        let value = JsonValue::Bool(true);
        assert_eq!(to_json(&value), "true");

        let arr = JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]);
        assert_eq!(to_json(&arr), "[1,2]");
    }

    // Test JsonEncoder with default config
    #[test]
    fn test_json_encoder_basic() {
        let mut encoder = JsonEncoder::new();
        assert_eq!(encoder.encode(&JsonValue::Null), "null");
        assert_eq!(encoder.encode(&JsonValue::Bool(true)), "true");
        assert_eq!(encoder.encode(&JsonValue::String("test".to_string())), r#""test""#);
    }

    // Test pretty printing
    #[test]
    fn test_pretty_printing() {
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        obj.insert("age".to_string(), JsonValue::Number(25.0));
        obj.insert("active".to_string(), JsonValue::Bool(true));

        let value = JsonValue::Object(obj);
        let pretty = to_json_pretty(&value);

        // Should contain newlines and indentation
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
        assert!(pretty.contains(":\n") || pretty.contains(": "));

        // Should be valid JSON when parsed
        use vela_stdlib::json::parse;
        assert!(parse(&pretty.replace(" ", "").replace("\n", "")).is_ok());
    }

    #[test]
    fn test_pretty_print_array() {
        let arr = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);

        let pretty = to_json_pretty(&arr);
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }

    #[test]
    fn test_pretty_print_nested() {
        let mut inner = HashMap::new();
        inner.insert("value".to_string(), JsonValue::Number(42.0));

        let mut outer = HashMap::new();
        outer.insert("data".to_string(), JsonValue::Object(inner));
        outer.insert("count".to_string(), JsonValue::Number(1.0));

        let value = JsonValue::Object(outer);
        let pretty = to_json_pretty(&value);

        // Should have proper indentation levels
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("    ")); // Nested indentation
    }

    // Test sorted keys
    #[test]
    fn test_sorted_keys() {
        let mut obj = HashMap::new();
        obj.insert("zebra".to_string(), JsonValue::String("last".to_string()));
        obj.insert("alpha".to_string(), JsonValue::String("first".to_string()));
        obj.insert("beta".to_string(), JsonValue::String("middle".to_string()));

        let value = JsonValue::Object(obj);
        let sorted = to_json_sorted(&value);

        // Keys should be in alphabetical order
        let alpha_pos = sorted.find(r#""alpha""#).unwrap();
        let beta_pos = sorted.find(r#""beta""#).unwrap();
        let zebra_pos = sorted.find(r#""zebra""#).unwrap();

        assert!(alpha_pos < beta_pos);
        assert!(beta_pos < zebra_pos);
    }

    // Test custom configuration
    #[test]
    fn test_custom_config() {
        let config = JsonEncoderConfig {
            pretty: true,
            indent: "    ".to_string(), // 4 spaces
            sort_keys: true,
            escape_slashes: true,
            max_depth: 0,
            null_value: "nil".to_string(),
        };

        let mut encoder = JsonEncoder::with_config(config);

        let mut obj = HashMap::new();
        obj.insert("b".to_string(), JsonValue::String("second".to_string()));
        obj.insert("a".to_string(), JsonValue::String("first".to_string()));

        let value = JsonValue::Object(obj);
        let result = encoder.encode(&value);

        // Should be sorted and use 4-space indentation
        assert!(result.contains("    "));
        assert!(result.contains(":\n"));

        // Check order: "a" before "b"
        let a_pos = result.find(r#""a""#).unwrap();
        let b_pos = result.find(r#""b""#).unwrap();
        assert!(a_pos < b_pos);
    }

    #[test]
    fn test_escape_slashes() {
        let config = JsonEncoderConfig {
            escape_slashes: true,
            ..Default::default()
        };

        let mut encoder = JsonEncoder::with_config(config);
        let value = JsonValue::String("path/to/file".to_string());
        let result = encoder.encode(&value);

        assert_eq!(result, r#""path\/to\/file""#);
    }

    #[test]
    fn test_custom_null_value() {
        let config = JsonEncoderConfig {
            null_value: "nil".to_string(),
            ..Default::default()
        };

        let mut encoder = JsonEncoder::with_config(config);
        let result = encoder.encode(&JsonValue::Null);

        assert_eq!(result, "nil");
    }

    // Test max depth
    #[test]
    fn test_max_depth() {
        let config = JsonEncoderConfig {
            max_depth: 2,
            null_value: "null".to_string(),
            ..Default::default()
        };

        let mut encoder = JsonEncoder::with_config(config);

        // Create deeply nested structure
        let nested = JsonValue::Array(vec![
            JsonValue::Array(vec![
                JsonValue::Array(vec![
                    JsonValue::String("deep".to_string())
                ])
            ])
        ]);

        let result = encoder.encode(&nested);
        // Should be truncated at depth 2
        assert_eq!(result, "[[null]]");
    }

    // Test streaming encoding
    #[test]
    fn test_streaming_encoding() {
        let value = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);

        let mut buffer = Vec::new();
        {
            let mut encoder = JsonEncoder::new();
            encoder.encode_to_writer(&value, &mut buffer).unwrap();
        }

        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "[1,2,3]");
    }

    #[test]
    fn test_streaming_pretty() {
        let mut obj = HashMap::new();
        obj.insert("test".to_string(), JsonValue::String("value".to_string()));

        let value = JsonValue::Object(obj);

        let mut buffer = Vec::new();
        {
            let mut encoder = JsonEncoder::pretty();
            encoder.encode_to_writer(&value, &mut buffer).unwrap();
        }

        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains('\n'));
        assert!(result.contains("  "));
    }

    // Test convenience functions from encoder module
    #[test]
    fn test_convenience_encode_pretty() {
        let value = JsonValue::Bool(true);
        let result = convenience::encode_pretty(&value);
        assert_eq!(result, "true");
    }

    #[test]
    fn test_convenience_encode_sorted() {
        let mut obj = HashMap::new();
        obj.insert("z".to_string(), JsonValue::Number(1.0));
        obj.insert("a".to_string(), JsonValue::Number(2.0));

        let value = JsonValue::Object(obj);
        let result = convenience::encode_sorted(&value);

        let a_pos = result.find(r#""a""#).unwrap();
        let z_pos = result.find(r#""z""#).unwrap();
        assert!(a_pos < z_pos);
    }

    #[test]
    fn test_convenience_encode_to_writer() {
        let value = JsonValue::String("test".to_string());
        let mut buffer = Vec::new();

        convenience::encode_to_writer(&value, &mut buffer).unwrap();
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, r#""test""#);
    }

    // Test complex nested structures
    #[test]
    fn test_complex_nested_encoding() {
        // Create a complex nested structure
        let mut user1 = HashMap::new();
        user1.insert("id".to_string(), JsonValue::Number(1.0));
        user1.insert("name".to_string(), JsonValue::String("Alice".to_string()));

        let mut user2 = HashMap::new();
        user2.insert("id".to_string(), JsonValue::Number(2.0));
        user2.insert("name".to_string(), JsonValue::String("Bob".to_string()));

        let mut data = HashMap::new();
        data.insert("users".to_string(), JsonValue::Array(vec![
            JsonValue::Object(user1),
            JsonValue::Object(user2),
        ]));
        data.insert("total".to_string(), JsonValue::Number(2.0));
        data.insert("active".to_string(), JsonValue::Bool(true));

        let value = JsonValue::Object(data);
        let json = value.to_json();

        // Should be valid JSON
        use vela_stdlib::json::parse;
        assert!(parse(&json).is_ok());

        // Should contain expected content
        assert!(json.contains(r#""users""#));
        assert!(json.contains(r#""total":2"#));
        assert!(json.contains(r#""active":true"#));
    }

    // Test round-trip encoding/decoding
    #[test]
    fn test_round_trip() {
        use vela_stdlib::json::parse;

        let original_json = r#"{"users":[{"name":"Alice","age":25},{"name":"Bob","age":30}],"active":true}"#;

        // Parse
        let parsed = parse(original_json).unwrap();

        // Encode back
        let encoded = parsed.to_json();

        // Parse again
        let reparsed = parse(&encoded).unwrap();

        // Should be equivalent
        assert_eq!(parsed, reparsed);
    }

    // Test edge cases
    #[test]
    fn test_edge_cases() {
        // Empty structures
        assert_eq!(JsonValue::Array(vec![]).to_json(), "[]");
        assert_eq!(JsonValue::Object(HashMap::new()).to_json(), "{}");

        // Single element arrays
        assert_eq!(JsonValue::Array(vec![JsonValue::Number(1.0)]).to_json(), "[1]");

        // Arrays with nulls
        let arr_with_nulls = JsonValue::Array(vec![
            JsonValue::Null,
            JsonValue::Number(1.0),
            JsonValue::Null,
        ]);
        assert_eq!(arr_with_nulls.to_json(), "[null,1,null]");

        // Objects with various value types
        let mut mixed_obj = HashMap::new();
        mixed_obj.insert("null_val".to_string(), JsonValue::Null);
        mixed_obj.insert("bool_val".to_string(), JsonValue::Bool(true));
        mixed_obj.insert("num_val".to_string(), JsonValue::Number(42.0));
        mixed_obj.insert("str_val".to_string(), JsonValue::String("test".to_string()));
        mixed_obj.insert("arr_val".to_string(), JsonValue::Array(vec![JsonValue::Number(1.0)]));

        let json = JsonValue::Object(mixed_obj).to_json();
        assert!(json.contains(r#""null_val":null"#));
        assert!(json.contains(r#""bool_val":true"#));
        assert!(json.contains(r#""num_val":42"#));
        assert!(json.contains(r#""str_val":"test""#));
        assert!(json.contains(r#""arr_val":[1]"#));
    }
}</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\tests\unit\test_json_encoder.rs