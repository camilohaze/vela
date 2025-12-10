//! Unit tests for JSON parser
//!
//! Tests the JsonParser implementation, JsonValue enum,
//! error handling, and edge cases.

use std::collections::HashMap;
use vela_stdlib::json::parser::{JsonParser, JsonValue, JsonParseError};
use vela_stdlib::json::parse;

#[cfg(test)]
mod test_json_parser {
    use super::*;

    // Test parsing of primitive values
    #[test]
    fn test_parse_null() {
        assert_eq!(parse("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_boolean_true() {
        assert_eq!(parse("true").unwrap(), JsonValue::Bool(true));
    }

    #[test]
    fn test_parse_boolean_false() {
        assert_eq!(parse("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse("0").unwrap(), JsonValue::Number(0.0));
        assert_eq!(parse("-42").unwrap(), JsonValue::Number(-42.0));
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse("3.14").unwrap(), JsonValue::Number(3.14));
        assert_eq!(parse("0.5").unwrap(), JsonValue::Number(0.5));
        assert_eq!(parse("-2.5").unwrap(), JsonValue::Number(-2.5));
    }

    #[test]
    fn test_parse_scientific_notation() {
        assert_eq!(parse("1e10").unwrap(), JsonValue::Number(1e10));
        assert_eq!(parse("1.5e-2").unwrap(), JsonValue::Number(1.5e-2));
        assert_eq!(parse("-2.3E+5").unwrap(), JsonValue::Number(-2.3E+5));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse(r#""hello""#).unwrap(), JsonValue::String("hello".to_string()));
        assert_eq!(parse(r#""world""#).unwrap(), JsonValue::String("world".to_string()));
        assert_eq!(parse(r#""""#).unwrap(), JsonValue::String("".to_string()));
    }

    #[test]
    fn test_parse_string_with_escapes() {
        assert_eq!(parse(r#""hello\nworld""#).unwrap(), JsonValue::String("hello\nworld".to_string()));
        assert_eq!(parse(r#""quote: \"test\"""#).unwrap(), JsonValue::String("quote: \"test\"".to_string()));
        assert_eq!(parse(r#""backslash: \\""#).unwrap(), JsonValue::String("backslash: \\".to_string()));
        assert_eq!(parse(r#""tab:\there""#).unwrap(), JsonValue::String("tab:\there".to_string()));
    }

    #[test]
    fn test_parse_unicode_escapes() {
        assert_eq!(parse(r#""\u0041""#).unwrap(), JsonValue::String("A".to_string()));
        assert_eq!(parse(r#""\u00A9""#).unwrap(), JsonValue::String("©".to_string()));
        assert_eq!(parse(r#""\uD83D\uDE00""#).unwrap(), JsonValue::String("😀".to_string()));
    }

    // Test parsing of arrays
    #[test]
    fn test_parse_empty_array() {
        assert_eq!(parse("[]").unwrap(), JsonValue::Array(vec![]));
    }

    #[test]
    fn test_parse_simple_array() {
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parse("[1,2,3]").unwrap(), expected);
    }

    #[test]
    fn test_parse_mixed_array() {
        let expected = JsonValue::Array(vec![
            JsonValue::String("hello".to_string()),
            JsonValue::Number(42.0),
            JsonValue::Bool(true),
            JsonValue::Null,
        ]);
        assert_eq!(parse(r#"["hello", 42, true, null]"#).unwrap(), expected);
    }

    #[test]
    fn test_parse_nested_array() {
        let expected = JsonValue::Array(vec![
            JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
            JsonValue::Array(vec![JsonValue::Number(3.0)]),
        ]);
        assert_eq!(parse("[[1,2],[3]]").unwrap(), expected);
    }

    // Test parsing of objects
    #[test]
    fn test_parse_empty_object() {
        let expected = JsonValue::Object(HashMap::new());
        assert_eq!(parse("{}").unwrap(), expected);
    }

    #[test]
    fn test_parse_simple_object() {
        let mut expected = HashMap::new();
        expected.insert("name".to_string(), JsonValue::String("John".to_string()));
        expected.insert("age".to_string(), JsonValue::Number(30.0));

        assert_eq!(parse(r#"{"name":"John","age":30}"#).unwrap(), JsonValue::Object(expected));
    }

    #[test]
    fn test_parse_nested_object() {
        let mut address = HashMap::new();
        address.insert("street".to_string(), JsonValue::String("123 Main St".to_string()));
        address.insert("city".to_string(), JsonValue::String("Anytown".to_string()));

        let mut person = HashMap::new();
        person.insert("name".to_string(), JsonValue::String("John".to_string()));
        person.insert("address".to_string(), JsonValue::Object(address));

        assert_eq!(parse(r#"{"name":"John","address":{"street":"123 Main St","city":"Anytown"}}"#).unwrap(), JsonValue::Object(person));
    }

    // Test whitespace handling
    #[test]
    fn test_parse_with_whitespace() {
        assert_eq!(parse("  null  ").unwrap(), JsonValue::Null);
        assert_eq!(parse("  [ 1 , 2 ]  ").unwrap(), JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]));
        assert_eq!(parse("  { \"key\" : \"value\" }  ").unwrap(), JsonValue::Object({
            let mut map = HashMap::new();
            map.insert("key".to_string(), JsonValue::String("value".to_string()));
            map
        }));
    }

    // Test error cases
    #[test]
    fn test_parse_invalid_json() {
        assert!(matches!(parse(""), Err(JsonParseError::UnexpectedEndOfInput)));
        assert!(matches!(parse("{"), Err(JsonParseError::UnexpectedEndOfInput)));
        assert!(matches!(parse("["), Err(JsonParseError::UnexpectedEndOfInput)));
        assert!(matches!(parse(r#""unclosed"#), Err(JsonParseError::UnexpectedEndOfInput)));
    }

    #[test]
    fn test_parse_trailing_characters() {
        assert!(matches!(parse("null extra"), Err(JsonParseError::TrailingCharacters)));
        assert!(matches!(parse("[1,2] garbage"), Err(JsonParseError::TrailingCharacters)));
    }

    #[test]
    fn test_parse_invalid_number() {
        assert!(matches!(parse("12.34.56"), Err(JsonParseError::InvalidNumber(_))));
        assert!(matches!(parse("00123"), Err(JsonParseError::InvalidNumber(_))));
        assert!(matches!(parse("12e"), Err(JsonParseError::InvalidNumber(_))));
    }

    #[test]
    fn test_parse_invalid_string() {
        assert!(matches!(parse(r#""unclosed"#), Err(JsonParseError::UnexpectedEndOfInput)));
        assert!(matches!(parse(r#""invalid\escape""#), Err(JsonParseError::InvalidString(_))));
    }

    #[test]
    fn test_parse_invalid_unicode() {
        assert!(matches!(parse(r#""\u123""#), Err(JsonParseError::InvalidUnicode(_))));
        assert!(matches!(parse(r#""\uGGGG""#), Err(JsonParseError::InvalidUnicode(_))));
    }

    #[test]
    fn test_parse_missing_comma() {
        assert!(matches!(parse(r#"[1 2]"#), Err(JsonParseError::ExpectedCommaOrClosingBrace)));
        assert!(matches!(parse(r#"{"a":1 "b":2}"#), Err(JsonParseError::ExpectedCommaOrClosingBrace)));
    }

    #[test]
    fn test_parse_missing_colon() {
        assert!(matches!(parse(r#"{"key" "value"}"#), Err(JsonParseError::ExpectedColon)));
    }

    // Test complex nested structures
    #[test]
    fn test_parse_complex_nested() {
        let json = r#"
        {
            "users": [
                {
                    "id": 1,
                    "name": "Alice",
                    "active": true,
                    "tags": ["admin", "developer"]
                },
                {
                    "id": 2,
                    "name": "Bob",
                    "active": false,
                    "tags": ["user"]
                }
            ],
            "metadata": {
                "version": "1.0",
                "count": 2,
                "settings": {
                    "theme": "dark",
                    "notifications": true
                }
            }
        }
        "#;

        let result = parse(json);
        assert!(result.is_ok());

        if let JsonValue::Object(root) = result.unwrap() {
            // Check users array
            if let Some(JsonValue::Array(users)) = root.get("users") {
                assert_eq!(users.len(), 2);

                // Check first user
                if let JsonValue::Object(user1) = &users[0] {
                    assert_eq!(user1.get("id"), Some(&JsonValue::Number(1.0)));
                    assert_eq!(user1.get("name"), Some(&JsonValue::String("Alice".to_string())));
                    assert_eq!(user1.get("active"), Some(&JsonValue::Bool(true)));
                }

                // Check second user
                if let JsonValue::Object(user2) = &users[1] {
                    assert_eq!(user2.get("id"), Some(&JsonValue::Number(2.0)));
                    assert_eq!(user2.get("name"), Some(&JsonValue::String("Bob".to_string())));
                    assert_eq!(user2.get("active"), Some(&JsonValue::Bool(false)));
                }
            }

            // Check metadata object
            if let Some(JsonValue::Object(metadata)) = root.get("metadata") {
                assert_eq!(metadata.get("version"), Some(&JsonValue::String("1.0".to_string())));
                assert_eq!(metadata.get("count"), Some(&JsonValue::Number(2.0)));

                if let Some(JsonValue::Object(settings)) = metadata.get("settings") {
                    assert_eq!(settings.get("theme"), Some(&JsonValue::String("dark".to_string())));
                    assert_eq!(settings.get("notifications"), Some(&JsonValue::Bool(true)));
                }
            }
        }
    }

    // Test edge cases
    #[test]
    fn test_parse_edge_cases() {
        // Empty string
        assert!(matches!(parse(""), Err(JsonParseError::UnexpectedEndOfInput)));

        // Only whitespace
        assert!(matches!(parse("   "), Err(JsonParseError::UnexpectedEndOfInput)));

        // Very nested structure (should work)
        let nested = "[{\"a\":{\"b\":{\"c\":{\"d\":[1,2,3]}}}}]".to_string();
        assert!(parse(&nested).is_ok());

        // Large numbers
        assert_eq!(parse("999999999999999").unwrap(), JsonValue::Number(999999999999999.0));
        assert_eq!(parse("0.000000000000001").unwrap(), JsonValue::Number(0.000000000000001));
    }

    // Test parser position tracking
    #[test]
    fn test_parser_position() {
        let mut parser = JsonParser::new(r#"{"key": "value"} extra"#);
        let result = parser.parse();
        assert!(matches!(result, Err(JsonParseError::TrailingCharacters)));
        assert_eq!(parser.position(), 16); // Position after the closing brace
    }

    #[test]
    fn test_parse_with_position() {
        use vela_stdlib::json::parse_with_position;

        let (value, pos) = parse_with_position(r#"{"test": 123}"#).unwrap();
        assert!(matches!(value, JsonValue::Object(_)));
        assert_eq!(pos, 13); // Length of the JSON string
    }
}</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\tests\unit\test_json_parser.rs