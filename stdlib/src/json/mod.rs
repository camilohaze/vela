//! JSON serialization and parsing module for Vela.
//!
//! This module provides comprehensive JSON parsing and encoding capabilities
//! according to RFC 8259 specification.

pub mod parser;

pub use parser::{JsonParser, JsonValue, JsonParseError};

/// Convenience function to parse JSON string
pub fn parse(input: &str) -> Result<JsonValue, JsonParseError> {
    JsonParser::new(input).parse()
}

/// Convenience function to parse JSON string with position information
pub fn parse_with_position(input: &str) -> Result<(JsonValue, usize), JsonParseError> {
    let mut parser = JsonParser::new(input);
    let result = parser.parse()?;
    Ok((result, parser.position()))
}

/// Convenience function to serialize JsonValue to JSON string
pub fn to_json(value: &JsonValue) -> String {
    value.to_json()
}