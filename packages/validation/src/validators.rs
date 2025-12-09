//! Validadores built-in para el sistema de validación
//!
//! Este módulo contiene los validadores básicos que se pueden usar
//! tanto de manera declarativa (con decoradores) como programática.

use crate::error::{ValidationError, ValidationResult};
use regex::Regex;
use std::sync::LazyLock;

/// Validador para campos requeridos
pub fn required(value: &Option<serde_json::Value>, field: &str) -> ValidationResult {
    match value {
        Some(v) if !is_empty_value(v) => ValidationResult::valid(),
        _ => ValidationResult::invalid_one(ValidationError::required(field)),
    }
}

/// Validador para formato de email
pub fn email(value: &Option<serde_json::Value>, field: &str) -> ValidationResult {
    static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    });

    match value {
        Some(serde_json::Value::String(s)) if !s.is_empty() => {
            if EMAIL_REGEX.is_match(s) {
                ValidationResult::valid()
            } else {
                ValidationResult::invalid_one(ValidationError::invalid_email(field, s.clone()))
            }
        }
        Some(_) => ValidationResult::invalid_one(ValidationError::invalid_email(field, "non-string value")),
        None => ValidationResult::valid(), // Email validation only applies to present values
    }
}

/// Validador para valor mínimo
pub fn min(value: &Option<serde_json::Value>, field: &str, min_value: serde_json::Value) -> ValidationResult {
    match (value, &min_value) {
        (Some(serde_json::Value::Number(v)), serde_json::Value::Number(min_num)) => {
            if v.as_f64().unwrap_or(0.0) >= min_num.as_f64().unwrap_or(0.0) {
                ValidationResult::valid()
            } else {
                ValidationResult::invalid_one(ValidationError::min_violation(field, v.clone(), min_num.clone()))
            }
        }
        (Some(v), _) => ValidationResult::invalid_one(
            ValidationError::custom(field, "Value must be a number for min validation", v.clone())
        ),
        (None, _) => ValidationResult::valid(),
    }
}

/// Validador para valor máximo
pub fn max(value: &Option<serde_json::Value>, field: &str, max_value: serde_json::Value) -> ValidationResult {
    match (value, &max_value) {
        (Some(serde_json::Value::Number(v)), serde_json::Value::Number(max_num)) => {
            if v.as_f64().unwrap_or(0.0) <= max_num.as_f64().unwrap_or(0.0) {
                ValidationResult::valid()
            } else {
                ValidationResult::invalid_one(ValidationError::max_violation(field, v.clone(), max_num.clone()))
            }
        }
        (Some(v), _) => ValidationResult::invalid_one(
            ValidationError::custom(field, "Value must be a number for max validation", v.clone())
        ),
        (None, _) => ValidationResult::valid(),
    }
}

/// Validador para longitud de strings
pub fn length(value: &Option<serde_json::Value>, field: &str, min_len: Option<usize>, max_len: Option<usize>) -> ValidationResult {
    match value {
        Some(serde_json::Value::String(s)) => {
            let len = s.len();
            let min_ok = min_len.map_or(true, |min| len >= min);
            let max_ok = max_len.map_or(true, |max| len <= max);

            if min_ok && max_ok {
                ValidationResult::valid()
            } else {
                ValidationResult::invalid_one(ValidationError::length_violation(field, s.clone(), min_len, max_len))
            }
        }
        Some(v) => ValidationResult::invalid_one(
            ValidationError::custom(field, "Value must be a string for length validation", v.clone())
        ),
        None => ValidationResult::valid(),
    }
}

/// Validador para patrón regex
pub fn regex(value: &Option<serde_json::Value>, field: &str, pattern: &str) -> ValidationResult {
    match Regex::new(pattern) {
        Ok(re) => match value {
            Some(serde_json::Value::String(s)) => {
                if re.is_match(s) {
                    ValidationResult::valid()
                } else {
                    ValidationResult::invalid_one(ValidationError::regex_violation(field, s.clone(), pattern))
                }
            }
            Some(v) => ValidationResult::invalid_one(
                ValidationError::custom(field, "Value must be a string for regex validation", v.clone())
            ),
            None => ValidationResult::valid(),
        },
        Err(_) => ValidationResult::invalid_one(
            ValidationError::custom(field, "Invalid regex pattern", serde_json::Value::String(pattern.to_string()))
        ),
    }
}

/// Validador para URLs
pub fn url(value: &Option<serde_json::Value>, field: &str) -> ValidationResult {
    static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
    });

    match value {
        Some(serde_json::Value::String(s)) if !s.is_empty() => {
            if URL_REGEX.is_match(s) {
                ValidationResult::valid()
            } else {
                ValidationResult::invalid_one(
                    ValidationError::custom(field, "Invalid URL format", serde_json::Value::String(s.clone()))
                )
            }
        }
        Some(_) => ValidationResult::invalid_one(
            ValidationError::custom(field, "URL must be a non-empty string", serde_json::Value::String("non-string value".to_string()))
        ),
        None => ValidationResult::valid(),
    }
}

/// Validador custom que recibe una función
pub fn custom<F>(value: &Option<serde_json::Value>, field: &str, validator: F) -> ValidationResult
where
    F: Fn(&Option<serde_json::Value>) -> Result<(), String>,
{
    match validator(value) {
        Ok(()) => ValidationResult::valid(),
        Err(message) => ValidationResult::invalid_one(
            ValidationError::custom(field, message, value.as_ref().cloned().unwrap_or(serde_json::Value::Null))
        ),
    }
}

/// Función helper para determinar si un valor JSON se considera vacío
fn is_empty_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.trim().is_empty(),
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_required_validator() {
        assert!(required(&Some(json!("hello")), "field").is_valid);
        assert!(!required(&None, "field").is_valid);
        assert!(!required(&Some(json!("")), "field").is_valid);
        assert!(!required(&Some(json!([])), "field").is_valid);
    }

    #[test]
    fn test_email_validator() {
        assert!(email(&Some(json!("user@example.com")), "email").is_valid);
        assert!(!email(&Some(json!("invalid-email")), "email").is_valid);
        assert!(email(&None, "email").is_valid); // Optional field
    }

    #[test]
    fn test_min_validator() {
        assert!(min(&Some(json!(10)), "age", json!(5)).is_valid);
        assert!(!min(&Some(json!(3)), "age", json!(5)).is_valid);
        assert!(min(&None, "age", json!(5)).is_valid);
    }

    #[test]
    fn test_max_validator() {
        assert!(max(&Some(json!(3)), "age", json!(5)).is_valid);
        assert!(!max(&Some(json!(10)), "age", json!(5)).is_valid);
        assert!(max(&None, "age", json!(5)).is_valid);
    }

    #[test]
    fn test_length_validator() {
        assert!(length(&Some(json!("hello")), "name", Some(2), Some(10)).is_valid);
        assert!(!length(&Some(json!("x")), "name", Some(2), Some(10)).is_valid);
        assert!(!length(&Some(json!("this is a very long string")), "name", Some(2), Some(10)).is_valid);
        assert!(length(&None, "name", Some(2), Some(10)).is_valid);
    }

    #[test]
    fn test_regex_validator() {
        assert!(regex(&Some(json!("123")), "code", r"^\d+$").is_valid);
        assert!(!regex(&Some(json!("abc")), "code", r"^\d+$").is_valid);
        assert!(regex(&None, "code", r"^\d+$").is_valid);
    }

    #[test]
    fn test_url_validator() {
        assert!(url(&Some(json!("https://example.com")), "website").is_valid);
        assert!(!url(&Some(json!("not-a-url")), "website").is_valid);
        assert!(url(&None, "website").is_valid);
    }

    #[test]
    fn test_custom_validator() {
        let validator = |value: &Option<serde_json::Value>| {
            match value {
                Some(serde_json::Value::Number(n)) if n.as_i64() == Some(42) => Ok(()),
                _ => Err("Must be the answer to life".to_string()),
            }
        };

        assert!(custom(&Some(json!(42)), "answer", validator).is_valid);
        assert!(!custom(&Some(json!(24)), "answer", validator).is_valid);
    }
}