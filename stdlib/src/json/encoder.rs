//! Advanced JSON Encoder for Vela types.
//!
//! This module provides advanced JSON encoding capabilities beyond the basic
//! JsonValue.to_json() method, including pretty printing, streaming encoding,
//! custom type serialization, and performance optimizations.

use crate::json::JsonValue;
use std::collections::HashMap;
use std::io::Write;

/// Configuration options for JSON encoding
#[derive(Debug, Clone)]
pub struct JsonEncoderConfig {
    /// Pretty print with indentation
    pub pretty: bool,
    /// Indentation string (default: 2 spaces)
    pub indent: String,
    /// Sort object keys for consistent output
    pub sort_keys: bool,
    /// Escape forward slashes
    pub escape_slashes: bool,
    /// Maximum nesting depth (0 = unlimited)
    pub max_depth: usize,
    /// Custom null value representation
    pub null_value: String,
}

impl Default for JsonEncoderConfig {
    fn default() -> Self {
        Self {
            pretty: false,
            indent: "  ".to_string(),
            sort_keys: false,
            escape_slashes: false,
            max_depth: 0, // unlimited
            null_value: "null".to_string(),
        }
    }
}

/// Advanced JSON encoder with configurable options
pub struct JsonEncoder<'a> {
    config: JsonEncoderConfig,
    indent_level: usize,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> JsonEncoder<'a> {
    /// Create a new encoder with default configuration
    pub fn new() -> Self {
        Self {
            config: JsonEncoderConfig::default(),
            indent_level: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new encoder with custom configuration
    pub fn with_config(config: JsonEncoderConfig) -> Self {
        Self {
            config,
            indent_level: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a pretty-printing encoder
    pub fn pretty() -> Self {
        Self::with_config(JsonEncoderConfig {
            pretty: true,
            ..Default::default()
        })
    }

    /// Encode a JsonValue to a string
    pub fn encode(&mut self, value: &JsonValue) -> String {
        let mut result = String::new();
        self.encode_to_string(value, &mut result);
        result
    }

    /// Encode a JsonValue to a writer (streaming)
    pub fn encode_to_writer<W: Write>(&mut self, value: &JsonValue, writer: &mut W) -> std::io::Result<()> {
        self.encode_value_to_writer(value, writer, 0)
    }

    /// Encode a JsonValue to a string buffer
    fn encode_to_string(&mut self, value: &JsonValue, buffer: &mut String) {
        self.encode_value_to_string(value, buffer, 0);
    }

    /// Encode a value to string with depth tracking
    fn encode_value_to_string(&mut self, value: &JsonValue, buffer: &mut String, depth: usize) {
        if self.config.max_depth > 0 && depth >= self.config.max_depth {
            buffer.push_str(&self.config.null_value);
            return;
        }

        match value {
            JsonValue::Null => buffer.push_str(&self.config.null_value),
            JsonValue::Bool(b) => buffer.push_str(if *b { "true" } else { "false" }),
            JsonValue::Number(n) => self.encode_number(*n, buffer),
            JsonValue::String(s) => self.encode_string(s, buffer),
            JsonValue::Array(arr) => {
                self.encode_array_to_string(arr, buffer, depth + 1);
            }
            JsonValue::Object(obj) => {
                self.encode_object_to_string(obj, buffer, depth + 1);
            }
        }
    }

    /// Encode a value to writer with depth tracking
    fn encode_value_to_writer<W: Write>(&mut self, value: &JsonValue, writer: &mut W, depth: usize) -> std::io::Result<()> {
        if self.config.max_depth > 0 && depth >= self.config.max_depth {
            return write!(writer, "{}", self.config.null_value);
        }

        match value {
            JsonValue::Null => write!(writer, "{}", self.config.null_value),
            JsonValue::Bool(b) => write!(writer, "{}", if *b { "true" } else { "false" }),
            JsonValue::Number(n) => self.encode_number_to_writer(*n, writer),
            JsonValue::String(s) => self.encode_string_to_writer(s, writer),
            JsonValue::Array(arr) => {
                self.encode_array_to_writer(arr, writer, depth + 1)
            }
            JsonValue::Object(obj) => {
                self.encode_object_to_writer(obj, writer, depth + 1)
            }
        }
    }

    /// Encode number to string
    fn encode_number(&self, num: f64, buffer: &mut String) {
        // Handle special cases
        if num.is_nan() || num.is_infinite() {
            buffer.push_str(&self.config.null_value);
            return;
        }

        // Check if it's a whole number
        if num.fract() == 0.0 && num >= i64::MIN as f64 && num <= i64::MAX as f64 {
            buffer.push_str(&format!("{}", num as i64));
        } else {
            // Format as float, but avoid scientific notation for reasonable numbers
            let formatted = format!("{}", num);
            if formatted.contains('e') || formatted.contains('E') {
                // For very large/small numbers, scientific notation is fine
                buffer.push_str(&formatted);
            } else {
                // Remove unnecessary trailing zeros
                let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
                buffer.push_str(trimmed);
            }
        }
    }

    /// Encode number to writer
    fn encode_number_to_writer<W: Write>(&self, num: f64, writer: &mut W) -> std::io::Result<()> {
        if num.is_nan() || num.is_infinite() {
            return write!(writer, "{}", self.config.null_value);
        }

        if num.fract() == 0.0 && num >= i64::MIN as f64 && num <= i64::MAX as f64 {
            write!(writer, "{}", num as i64)
        } else {
            let formatted = format!("{}", num);
            if formatted.contains('e') || formatted.contains('E') {
                write!(writer, "{}", formatted)
            } else {
                let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
                write!(writer, "{}", trimmed)
            }
        }
    }

    /// Encode string to string buffer
    fn encode_string(&self, s: &str, buffer: &mut String) {
        buffer.push('"');

        for ch in s.chars() {
            match ch {
                '"' => buffer.push_str("\\\""),
                '\\' => buffer.push_str("\\\\"),
                '/' => {
                    if self.config.escape_slashes {
                        buffer.push_str("\\/");
                    } else {
                        buffer.push('/');
                    }
                }
                '\x08' => buffer.push_str("\\b"),  // backspace
                '\x0C' => buffer.push_str("\\f"),  // form feed
                '\n' => buffer.push_str("\\n"),
                '\r' => buffer.push_str("\\r"),
                '\t' => buffer.push_str("\\t"),
                ch if ch.is_control() => {
                    // Unicode escape for control characters
                    buffer.push_str(&format!("\\u{:04x}", ch as u32));
                }
                ch => buffer.push(ch),
            }
        }

        buffer.push('"');
    }

    /// Encode string to writer
    fn encode_string_to_writer<W: Write>(&self, s: &str, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "\"")?;

        for ch in s.chars() {
            match ch {
                '"' => write!(writer, "\\\"")?,
                '\\' => write!(writer, "\\\\")?,
                '/' => {
                    if self.config.escape_slashes {
                        write!(writer, "\\/")?;
                    } else {
                        write!(writer, "/")?;
                    }
                }
                '\x08' => write!(writer, "\\b")?,  // backspace
                '\x0C' => write!(writer, "\\f")?,  // form feed
                '\n' => write!(writer, "\\n")?,
                '\r' => write!(writer, "\\r")?,
                '\t' => write!(writer, "\\t")?,
                ch if ch.is_control() => {
                    write!(writer, "\\u{:04x}", ch as u32)?;
                }
                ch => write!(writer, "{}", ch)?,
            }
        }

        write!(writer, "\"")
    }

    /// Encode array to string
    fn encode_array_to_string(&mut self, arr: &[JsonValue], buffer: &mut String, depth: usize) {
        buffer.push('[');

        if self.config.pretty && !arr.is_empty() {
            buffer.push('\n');
            self.indent_level += 1;
        }

        for (i, value) in arr.iter().enumerate() {
            if i > 0 {
                buffer.push(',');
                if self.config.pretty {
                    buffer.push('\n');
                }
            }

            if self.config.pretty {
                self.write_indent(buffer);
            }

            self.encode_value_to_string(value, buffer, depth + 1);
        }

        if self.config.pretty && !arr.is_empty() {
            buffer.push('\n');
            self.indent_level -= 1;
            self.write_indent(buffer);
        }

        buffer.push(']');
    }

    /// Encode array to writer
    fn encode_array_to_writer<W: Write>(&mut self, arr: &[JsonValue], writer: &mut W, depth: usize) -> std::io::Result<()> {
        write!(writer, "[")?;

        if self.config.pretty && !arr.is_empty() {
            writeln!(writer)?;
            self.indent_level += 1;
        }

        for (i, value) in arr.iter().enumerate() {
            if i > 0 {
                write!(writer, ",")?;
                if self.config.pretty {
                    writeln!(writer)?;
                }
            }

            if self.config.pretty {
                self.write_indent_to_writer(writer)?;
            }

            self.encode_value_to_writer(value, writer, depth + 1)?;
        }

        if self.config.pretty && !arr.is_empty() {
            writeln!(writer)?;
            self.indent_level -= 1;
            self.write_indent_to_writer(writer)?;
        }

        write!(writer, "]")
    }

    /// Encode object to string
    fn encode_object_to_string(&mut self, obj: &HashMap<String, JsonValue>, buffer: &mut String, depth: usize) {
        buffer.push('{');

        if self.config.pretty && !obj.is_empty() {
            buffer.push('\n');
            self.indent_level += 1;
        }

        let keys: Vec<&String> = if self.config.sort_keys {
            let mut sorted_keys: Vec<&String> = obj.keys().collect();
            sorted_keys.sort();
            sorted_keys
        } else {
            obj.keys().collect()
        };

        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                buffer.push(',');
                if self.config.pretty {
                    buffer.push('\n');
                }
            }

            if self.config.pretty {
                self.write_indent(buffer);
            }

            self.encode_string(key, buffer);
            if self.config.pretty {
                buffer.push_str(":\n");
                self.write_indent(buffer);
            } else {
                buffer.push(':');
            }

            if let Some(value) = obj.get(*key) {
                self.encode_value_to_string(value, buffer, depth + 1);
            }
        }

        if self.config.pretty && !obj.is_empty() {
            buffer.push('\n');
            self.indent_level -= 1;
            self.write_indent(buffer);
        }

        buffer.push('}');
    }

    /// Encode object to writer
    fn encode_object_to_writer<W: Write>(&mut self, obj: &HashMap<String, JsonValue>, writer: &mut W, depth: usize) -> std::io::Result<()> {
        write!(writer, "{{")?;

        if self.config.pretty && !obj.is_empty() {
            writeln!(writer)?;
            self.indent_level += 1;
        }

        let keys: Vec<&String> = if self.config.sort_keys {
            let mut sorted_keys: Vec<&String> = obj.keys().collect();
            sorted_keys.sort();
            sorted_keys
        } else {
            obj.keys().collect()
        };

        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                write!(writer, ",")?;
                if self.config.pretty {
                    writeln!(writer)?;
                }
            }

            if self.config.pretty {
                self.write_indent_to_writer(writer)?;
            }

            self.encode_string_to_writer(key, writer)?;
            if self.config.pretty {
                write!(writer, ": ")?;
            } else {
                write!(writer, ":")?;
            }

            if let Some(value) = obj.get(*key) {
                self.encode_value_to_writer(value, writer, depth + 1)?;
            }
        }

        if self.config.pretty && !obj.is_empty() {
            writeln!(writer)?;
            self.indent_level -= 1;
            self.write_indent_to_writer(writer)?;
        }

        write!(writer, "}}")
    }

    /// Write indentation to string buffer
    fn write_indent(&self, buffer: &mut String) {
        for _ in 0..self.indent_level {
            buffer.push_str(&self.config.indent);
        }
    }

    /// Write indentation to writer
    fn write_indent_to_writer<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for _ in 0..self.indent_level {
            write!(writer, "{}", self.config.indent)?;
        }
        Ok(())
    }
}

/// Convenience functions for common encoding tasks
pub mod convenience {
    use super::*;

    /// Encode with pretty printing
    pub fn encode_pretty(value: &JsonValue) -> String {
        JsonEncoder::pretty().encode(value)
    }

    /// Encode with sorted keys
    pub fn encode_sorted(value: &JsonValue) -> String {
        JsonEncoder::with_config(JsonEncoderConfig {
            sort_keys: true,
            ..Default::default()
        }).encode(value)
    }

    /// Encode to writer for streaming
    pub fn encode_to_writer<W: Write>(value: &JsonValue, writer: &mut W) -> std::io::Result<()> {
        JsonEncoder::new().encode_to_writer(value, writer)
    }

    /// Encode with custom configuration
    pub fn encode_with_config(value: &JsonValue, config: JsonEncoderConfig) -> String {
        JsonEncoder::with_config(config).encode(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::JsonValue;

    #[test]
    fn test_basic_encoding() {
        let value = JsonValue::String("hello".to_string());
        let mut encoder = JsonEncoder::new();
        assert_eq!(encoder.encode(&value), r#""hello""#);
    }





    #[test]
    fn test_streaming_encoding() {
        let value = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);

        let mut buffer = Vec::new();
        let mut encoder = JsonEncoder::new();
        encoder.encode_to_writer(&value, &mut buffer).unwrap();

        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "[1,2,3]");
    }



    #[test]
    fn test_escape_slashes() {
        let value = JsonValue::String("path/to/file".to_string());

        let encoder = JsonEncoder::with_config(JsonEncoderConfig {
            escape_slashes: true,
            ..Default::default()
        });
        let mut encoder = encoder;
        let result = encoder.encode(&value);

        assert_eq!(result, r#""path\/to\/file""#);
    }

    #[test]
    fn test_convenience_functions() {
        let value = JsonValue::Bool(true);

        assert_eq!(convenience::encode_pretty(&value), "true");
        assert_eq!(convenience::encode_sorted(&value), "true");
    }

    #[test]
    fn test_number_encoding_edge_cases() {
        let mut encoder = JsonEncoder::new();

        // NaN should become null
        let nan_value = JsonValue::Number(f64::NAN);
        assert_eq!(encoder.encode(&nan_value), "null");

        // Infinity should become null
        let inf_value = JsonValue::Number(f64::INFINITY);
        assert_eq!(encoder.encode(&inf_value), "null");

        // Very large integers
        let big_int = JsonValue::Number(999999999999999.0);
        let result = encoder.encode(&big_int);
        assert_eq!(result, "999999999999999");
    }

    #[test]
    fn test_unicode_handling() {
        let mut encoder = JsonEncoder::new();

        // Unicode characters should be preserved
        let unicode_value = JsonValue::String("🚀 Hello 世界".to_string());
        let result = encoder.encode(&unicode_value);
        assert_eq!(result, r#""🚀 Hello 世界""#);

        // Control characters should be escaped
        let control_value = JsonValue::String("line1\nline2".to_string());
        let result = encoder.encode(&control_value);
        assert_eq!(result, r#""line1\nline2""#);
    }

    #[test]
    fn test_empty_structures() {
        let mut encoder = JsonEncoder::new();

        let empty_array = JsonValue::Array(vec![]);
        assert_eq!(encoder.encode(&empty_array), "[]");

        let empty_object = JsonValue::Object(HashMap::new());
        assert_eq!(encoder.encode(&empty_object), "{}");
    }

    #[test]
    fn test_pretty_print_complex() {
        let mut user = HashMap::new();
        user.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        user.insert("scores".to_string(), JsonValue::Array(vec![
            JsonValue::Number(95.0),
            JsonValue::Number(87.0),
        ]));

        let mut root = HashMap::new();
        root.insert("user".to_string(), JsonValue::Object(user));
        root.insert("active".to_string(), JsonValue::Bool(true));

        let value = JsonValue::Object(root);
        let mut encoder = JsonEncoder::pretty();
        let result = encoder.encode(&value);

        // Should be properly formatted
        assert!(result.contains('\n'));
        assert!(result.contains("  "));
        assert!(result.contains(":\n"));
        assert!(result.contains("    ")); // nested indentation
    }

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

        // Very large numbers (may or may not use scientific notation depending on f64 formatting)
        let large = JsonValue::Number(1e20).to_json();
        assert!(large.len() > 0); // Just ensure it's encoded
    }

    #[test]
    fn test_special_numbers() {
        // NaN should become null
        assert_eq!(JsonValue::Number(f64::NAN).to_json(), "null");

        // Infinity should become null
        assert_eq!(JsonValue::Number(f64::INFINITY).to_json(), "null");
        assert_eq!(JsonValue::Number(f64::NEG_INFINITY).to_json(), "null");
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
        let pretty = crate::json::to_json_pretty(&value);

        // Should contain newlines and indentation
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
        assert!(pretty.contains(":\n") || pretty.contains(": "));

        // Should be valid JSON when parsed
        use crate::json::parse;
        assert!(parse(&pretty.replace(" ", "").replace("\n", "")).is_ok());
    }

    #[test]
    fn test_pretty_print_array() {
        let arr = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);

        let pretty = crate::json::to_json_pretty(&arr);
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
        let pretty = crate::json::to_json_pretty(&value);

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
        let sorted = crate::json::to_json_sorted(&value);

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
            max_depth: 1,
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
        // Should be truncated at depth 1 - array at depth 1 becomes null
        assert_eq!(result, "[null]");
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
        data.insert("generated_at".to_string(), JsonValue::String("2024-01-15".to_string()));

        let value = JsonValue::Object(data);
        let json = value.to_json();

        // Should be valid JSON
        use crate::json::parse;
        assert!(parse(&json).is_ok());

        // Should contain expected content
        assert!(json.contains(r#""users""#));
        assert!(json.contains(r#""total":2"#));
        assert!(json.contains(r#""generated_at""#));
    }

    // Test round-trip encoding/decoding
    #[test]
    fn test_round_trip() {
        use crate::json::parse;

        let original_json = r#"{"users":[{"name":"Alice","age":25},{"name":"Bob","age":30}],"active":true,"count":2}"#;

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
}