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
    fn test_pretty_printing() {
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        obj.insert("age".to_string(), JsonValue::Number(25.0));

        let value = JsonValue::Object(obj);
        let mut encoder = JsonEncoder::pretty();
        let result = encoder.encode(&value);

        // Should contain newlines and indentation
        assert!(result.contains('\n'));
        assert!(result.contains("  "));
        assert!(result.contains(":\n"));
    }

    #[test]
    fn test_sorted_keys() {
        let mut obj = HashMap::new();
        obj.insert("zebra".to_string(), JsonValue::String("last".to_string()));
        obj.insert("alpha".to_string(), JsonValue::String("first".to_string()));

        let value = JsonValue::Object(obj);
        let mut encoder = JsonEncoder::with_config(JsonEncoderConfig {
            sort_keys: true,
            ..Default::default()
        });
        let result = encoder.encode(&value);

        // "alpha" should come before "zebra"
        let alpha_pos = result.find(r#""alpha""#).unwrap();
        let zebra_pos = result.find(r#""zebra""#).unwrap();
        assert!(alpha_pos < zebra_pos);
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
    fn test_max_depth() {
        let nested = JsonValue::Array(vec![
            JsonValue::Array(vec![
                JsonValue::Array(vec![
                    JsonValue::String("deep".to_string())
                ])
            ])
        ]);

        let encoder = JsonEncoder::with_config(JsonEncoderConfig {
            max_depth: 2,
            ..Default::default()
        });
        let mut encoder = encoder;
        let result = encoder.encode(&nested);

        // Should be truncated at depth 2
        assert_eq!(result, "[[null]]");
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
}