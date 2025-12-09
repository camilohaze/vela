//! HTTP error types

use std::fmt;

/// HTTP-specific errors
#[derive(Debug, Clone)]
pub enum HttpError {
    /// Invalid HTTP method
    InvalidMethod(String),
    /// Invalid URI
    InvalidUri(String),
    /// Invalid status code
    InvalidStatusCode(u16),
    /// IO error
    Io(String),
    /// Timeout error
    Timeout(String),
    /// Connection error
    Connection(String),
    /// TLS error
    Tls(String),
    /// Parse error
    Parse(String),
    /// Other error
    Other(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::InvalidMethod(msg) => write!(f, "Invalid HTTP method: {}", msg),
            HttpError::InvalidUri(msg) => write!(f, "Invalid URI: {}", msg),
            HttpError::InvalidStatusCode(code) => write!(f, "Invalid status code: {}", code),
            HttpError::Io(msg) => write!(f, "IO error: {}", msg),
            HttpError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            HttpError::Connection(msg) => write!(f, "Connection error: {}", msg),
            HttpError::Tls(msg) => write!(f, "TLS error: {}", msg),
            HttpError::Parse(msg) => write!(f, "Parse error: {}", msg),
            HttpError::Other(msg) => write!(f, "HTTP error: {}", msg),
        }
    }
}

impl std::error::Error for HttpError {}

impl From<std::io::Error> for HttpError {
    fn from(err: std::io::Error) -> Self {
        HttpError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for HttpError {
    fn from(err: serde_json::Error) -> Self {
        HttpError::Parse(err.to_string())
    }
}

impl From<reqwest::Error> for HttpError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            HttpError::Timeout(err.to_string())
        } else if err.is_connect() {
            HttpError::Connection(err.to_string())
        } else {
            HttpError::Other(err.to_string())
        }
    }
}

impl From<hyper::Error> for HttpError {
    fn from(err: hyper::Error) -> Self {
        HttpError::Other(err.to_string())
    }
}

/// Result type alias for HTTP operations
pub type Result<T> = std::result::Result<T, HttpError>;