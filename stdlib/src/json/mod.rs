//! JSON serialization and parsing module for Vela.
//!
//! This module provides comprehensive JSON parsing and encoding capabilities
//! according to RFC 8259 specification.

pub mod encoder;
pub mod parser;
pub mod serialization;

pub use encoder::{JsonEncoder, JsonEncoderConfig, convenience as json_encoder};
pub use parser::{JsonParser, JsonValue, JsonParseError};
pub use serialization::*;

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

/// Convenience function to serialize JsonValue with pretty printing
pub fn to_json_pretty(value: &JsonValue) -> String {
    json_encoder::encode_pretty(value)
}

/// Convenience function to serialize JsonValue with sorted keys
pub fn to_json_sorted(value: &JsonValue) -> String {
    json_encoder::encode_sorted(value)
}

/// Convenience function to encode to a writer (streaming)
pub fn encode_to_writer<W: std::io::Write>(value: &JsonValue, writer: &mut W) -> std::io::Result<()> {
    json_encoder::encode_to_writer(value, writer)
}