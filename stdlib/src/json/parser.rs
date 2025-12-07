//! JSON Parser implementation for Vela.
//!
//! This module implements a complete JSON parser according to RFC 8259.
//! It provides both a streaming parser interface and convenience functions.

use std::collections::HashMap;

/// Represents a JSON value that can be parsed from or serialized to JSON.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// JSON null value
    Null,
    /// JSON boolean value
    Bool(bool),
    /// JSON number (stored as f64)
    Number(f64),
    /// JSON string
    String(String),
    /// JSON array
    Array(Vec<JsonValue>),
    /// JSON object
    Object(HashMap<String, JsonValue>),
}

/// Errors that can occur during JSON parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonParseError {
    /// Unexpected end of input
    UnexpectedEndOfInput,
    /// Unexpected character at given position
    UnexpectedCharacter(char, usize),
    /// Invalid number format
    InvalidNumber(String),
    /// Invalid string format
    InvalidString(String),
    /// Invalid Unicode escape sequence
    InvalidUnicode(String),
    /// Trailing characters after valid JSON
    TrailingCharacters,
    /// Expected comma or closing bracket/braces
    ExpectedCommaOrClosingBrace,
    /// Expected colon after object key
    ExpectedColon,
    /// Expected a JSON value
    ExpectedValue,
}

impl std::fmt::Display for JsonParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            JsonParseError::UnexpectedCharacter(c, pos) => {
                write!(f, "Unexpected character '{}' at position {}", c, pos)
            }
            JsonParseError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            JsonParseError::InvalidString(s) => write!(f, "Invalid string: {}", s),
            JsonParseError::InvalidUnicode(s) => write!(f, "Invalid Unicode escape: {}", s),
            JsonParseError::TrailingCharacters => write!(f, "Trailing characters after JSON value"),
            JsonParseError::ExpectedCommaOrClosingBrace => {
                write!(f, "Expected comma or closing bracket/brace")
            }
            JsonParseError::ExpectedColon => write!(f, "Expected colon after object key"),
            JsonParseError::ExpectedValue => write!(f, "Expected JSON value"),
        }
    }
}

impl std::error::Error for JsonParseError {}

/// JSON parser that can parse JSON strings into JsonValue structures.
pub struct JsonParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> JsonParser<'a> {
    /// Create a new JSON parser for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Get the current parsing position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Parse the entire input as a JSON value.
    pub fn parse(&mut self) -> Result<JsonValue, JsonParseError> {
        self.skip_whitespace()?;
        let value = self.parse_value()?;
        self.skip_whitespace()?;

        if self.position < self.input.len() {
            return Err(JsonParseError::TrailingCharacters);
        }

        Ok(value)
    }

    /// Skip whitespace characters (space, tab, newline, carriage return).
    fn skip_whitespace(&mut self) -> Result<(), JsonParseError> {
        while self.position < self.input.len() {
            let ch = self.input.as_bytes()[self.position] as char;
            if !ch.is_whitespace() {
                break;
            }
            self.position += 1;
        }
        Ok(())
    }

    /// Parse a JSON value (null, boolean, number, string, array, object).
    fn parse_value(&mut self) -> Result<JsonValue, JsonParseError> {
        if self.position >= self.input.len() {
            return Err(JsonParseError::UnexpectedEndOfInput);
        }

        let ch = self.input.as_bytes()[self.position] as char;
        match ch {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_boolean(),
            '"' => self.parse_string().map(JsonValue::String),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '0'..='9' | '-' => self.parse_number().map(JsonValue::Number),
            _ => Err(JsonParseError::UnexpectedCharacter(ch, self.position)),
        }
    }

    /// Parse JSON null literal.
    fn parse_null(&mut self) -> Result<JsonValue, JsonParseError> {
        if self.input[self.position..].starts_with("null") {
            self.position += 4;
            Ok(JsonValue::Null)
        } else {
            Err(JsonParseError::ExpectedValue)
        }
    }

    /// Parse JSON boolean literals (true/false).
    fn parse_boolean(&mut self) -> Result<JsonValue, JsonParseError> {
        if self.input[self.position..].starts_with("true") {
            self.position += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.position..].starts_with("false") {
            self.position += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(JsonParseError::ExpectedValue)
        }
    }

    /// Parse JSON number.
    fn parse_number(&mut self) -> Result<f64, JsonParseError> {
        let start = self.position;

        // Optional minus sign
        if self.peek_char() == Some('-') {
            self.position += 1;
        }

        // Integer part
        if self.peek_char() == Some('0') {
            self.position += 1;
        } else if let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() && ch != '0' {
                self.position += 1;
                while let Some(ch) = self.peek_char() {
                    if ch.is_ascii_digit() {
                        self.position += 1;
                    } else {
                        break;
                    }
                }
            } else {
                return Err(JsonParseError::InvalidNumber(
                    self.input[start..self.position + 1].to_string()
                ));
            }
        }

        // Optional fractional part
        if self.peek_char() == Some('.') {
            self.position += 1;
            if let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    self.position += 1;
                    while let Some(ch) = self.peek_char() {
                        if ch.is_ascii_digit() {
                            self.position += 1;
                        } else {
                            break;
                        }
                    }
                } else {
                    return Err(JsonParseError::InvalidNumber(
                        self.input[start..self.position + 1].to_string()
                    ));
                }
            }
        }

        // Optional exponent part
        if let Some(ch) = self.peek_char() {
            if ch == 'e' || ch == 'E' {
                self.position += 1;

                // Optional sign
                if let Some(ch) = self.peek_char() {
                    if ch == '+' || ch == '-' {
                        self.position += 1;
                    }
                }

                // Exponent digits
                if let Some(ch) = self.peek_char() {
                    if ch.is_ascii_digit() {
                        self.position += 1;
                        while let Some(ch) = self.peek_char() {
                            if ch.is_ascii_digit() {
                                self.position += 1;
                            } else {
                                break;
                            }
                        }
                    } else {
                        return Err(JsonParseError::InvalidNumber(
                            self.input[start..self.position + 1].to_string()
                        ));
                    }
                }
            }
        }

        let number_str = &self.input[start..self.position];
        match number_str.parse::<f64>() {
            Ok(num) => Ok(num),
            Err(_) => Err(JsonParseError::InvalidNumber(number_str.to_string())),
        }
    }

    /// Parse JSON string with escape sequences.
    fn parse_string(&mut self) -> Result<String, JsonParseError> {
        if !self.expect_char('"') {
            return Err(JsonParseError::ExpectedValue);
        }

        let mut result = String::new();

        while self.position < self.input.len() {
            let ch = self.input.as_bytes()[self.position] as char;
            self.position += 1;

            match ch {
                '"' => return Ok(result),
                '\\' => {
                    if self.position >= self.input.len() {
                        return Err(JsonParseError::InvalidString(
                            self.input[self.position-1..].to_string()
                        ));
                    }

                    let escape_ch = self.input.as_bytes()[self.position] as char;
                    self.position += 1;

                    match escape_ch {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'), // backspace
                        'f' => result.push('\x0C'), // form feed
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        'u' => {
                            // Unicode escape: \uXXXX
                            if self.position + 4 > self.input.len() {
                                return Err(JsonParseError::InvalidUnicode(
                                    self.input[self.position-2..].to_string()
                                ));
                            }

                            let hex_str = &self.input[self.position..self.position + 4];
                            self.position += 4;

                            match u16::from_str_radix(hex_str, 16) {
                                Ok(code) => {
                                    if let Some(ch) = std::char::from_u32(code as u32) {
                                        result.push(ch);
                                    } else {
                                        return Err(JsonParseError::InvalidUnicode(hex_str.to_string()));
                                    }
                                }
                                Err(_) => {
                                    return Err(JsonParseError::InvalidUnicode(hex_str.to_string()));
                                }
                            }
                        }
                        _ => {
                            return Err(JsonParseError::InvalidString(
                                format!("\\{}", escape_ch)
                            ));
                        }
                    }
                }
                ch if ch.is_control() => {
                    return Err(JsonParseError::InvalidString(
                        format!("Control character: {}", ch as u32)
                    ));
                }
                ch => result.push(ch),
            }
        }

        Err(JsonParseError::InvalidString(
            self.input[self.position.saturating_sub(1)..].to_string()
        ))
    }

    /// Parse JSON array.
    fn parse_array(&mut self) -> Result<JsonValue, JsonParseError> {
        if !self.expect_char('[') {
            return Err(JsonParseError::ExpectedValue);
        }

        let mut elements = Vec::new();

        loop {
            self.skip_whitespace()?;

            if self.peek_char() == Some(']') {
                self.position += 1;
                break;
            }

            if !elements.is_empty() {
                if !self.expect_char(',') {
                    return Err(JsonParseError::ExpectedCommaOrClosingBrace);
                }
                self.skip_whitespace()?;
            }

            let value = self.parse_value()?;
            elements.push(value);
        }

        Ok(JsonValue::Array(elements))
    }

    /// Parse JSON object.
    fn parse_object(&mut self) -> Result<JsonValue, JsonParseError> {
        if !self.expect_char('{') {
            return Err(JsonParseError::ExpectedValue);
        }

        let mut object = HashMap::new();

        loop {
            self.skip_whitespace()?;

            if self.peek_char() == Some('}') {
                self.position += 1;
                break;
            }

            if !object.is_empty() {
                if !self.expect_char(',') {
                    return Err(JsonParseError::ExpectedCommaOrClosingBrace);
                }
                self.skip_whitespace()?;
            }

            // Parse key
            let key = self.parse_string()?;

            self.skip_whitespace()?;
            if !self.expect_char(':') {
                return Err(JsonParseError::ExpectedColon);
            }
            self.skip_whitespace()?;

            // Parse value
            let value = self.parse_value()?;
            object.insert(key, value);
        }

        Ok(JsonValue::Object(object))
    }

    /// Peek at the next character without consuming it.
    fn peek_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input.as_bytes()[self.position] as char)
        } else {
            None
        }
    }

    /// Expect a specific character and consume it.
    fn expect_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_boolean() {
        assert_eq!(parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse("3.14").unwrap(), JsonValue::Number(3.14));
        assert_eq!(parse("-123").unwrap(), JsonValue::Number(-123.0));
        assert_eq!(parse("1e10").unwrap(), JsonValue::Number(1e10));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse(r#""hello""#).unwrap(), JsonValue::String("hello".to_string()));
        assert_eq!(parse(r#""hello\nworld""#).unwrap(), JsonValue::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_parse_array() {
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0)
        ]);
        assert_eq!(parse("[1,2,3]").unwrap(), expected);
    }

    #[test]
    fn test_parse_object() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("John".to_string()));
        map.insert("age".to_string(), JsonValue::Number(30.0));
        let expected = JsonValue::Object(map);
        assert_eq!(parse(r#"{"name":"John","age":30}"#).unwrap(), expected);
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse("").is_err());
        assert!(parse("{").is_err());
        assert!(parse("[1,2,]").is_err());
        assert!(parse(r#""unclosed"#).is_err());
        assert!(parse("tru").is_err());
        assert!(parse("null extra").is_err());
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(parse("  null  ").unwrap(), JsonValue::Null);
        assert_eq!(parse("[\n  1  ,\n  2  \n]").unwrap(),
                   JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]));
    }
}