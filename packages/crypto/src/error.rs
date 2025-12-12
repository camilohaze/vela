/*!
# Crypto Error Types

Unified error handling for all cryptographic operations in Vela.
All errors implement `std::error::Error` and `Send + Sync` for async compatibility.
*/

use std::fmt;

/// Main error type for cryptographic operations
#[derive(Debug, Clone, PartialEq)]
pub enum CryptoError {
    /// Invalid input parameters
    InvalidInput(String),

    /// Key-related errors
    KeyError(String),

    /// Hashing operation failed
    HashError(String),

    /// Encryption/decryption failed
    EncryptionError(String),

    /// JWT operation failed
    JWTError(String),

    /// Digital signature operation failed
    SignatureError(String),

    /// Random number generation failed
    RandomError(String),

    /// Algorithm not supported
    UnsupportedAlgorithm(String),

    /// Authentication failed (wrong key, signature, etc.)
    AuthenticationError(String),

    /// Data integrity check failed
    IntegrityError(String),

    /// Encoding/decoding error
    EncodingError(String),

    /// IO operation failed
    IoError(String),

    /// Operation timed out
    TimeoutError(String),

    /// Generic crypto operation error
    GenericError(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CryptoError::KeyError(msg) => write!(f, "Key error: {}", msg),
            CryptoError::HashError(msg) => write!(f, "Hash error: {}", msg),
            CryptoError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            CryptoError::JWTError(msg) => write!(f, "JWT error: {}", msg),
            CryptoError::SignatureError(msg) => write!(f, "Signature error: {}", msg),
            CryptoError::RandomError(msg) => write!(f, "Random error: {}", msg),
            CryptoError::UnsupportedAlgorithm(msg) => write!(f, "Unsupported algorithm: {}", msg),
            CryptoError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            CryptoError::IntegrityError(msg) => write!(f, "Integrity error: {}", msg),
            CryptoError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            CryptoError::IoError(msg) => write!(f, "IO error: {}", msg),
            CryptoError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            CryptoError::GenericError(msg) => write!(f, "Crypto error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Result type alias for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Convert from other error types to CryptoError
impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for CryptoError {
    fn from(err: serde_json::Error) -> Self {
        CryptoError::EncodingError(format!("JSON error: {}", err))
    }
}

impl From<base64::DecodeError> for CryptoError {
    fn from(err: base64::DecodeError) -> Self {
        CryptoError::EncodingError(format!("Base64 decode error: {}", err))
    }
}

impl From<hex::FromHexError> for CryptoError {
    fn from(err: hex::FromHexError) -> Self {
        CryptoError::EncodingError(format!("Hex decode error: {}", err))
    }
}

#[cfg(feature = "jwt")]
impl From<jsonwebtoken::errors::Error> for CryptoError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        CryptoError::JWTError(err.to_string())
    }
}

#[cfg(feature = "rsa")]
impl From<rsa::errors::Error> for CryptoError {
    fn from(err: rsa::errors::Error) -> Self {
        CryptoError::SignatureError(format!("RSA error: {}", err))
    }
}

#[cfg(feature = "ecdsa")]
impl From<ecdsa::Error> for CryptoError {
    fn from(err: ecdsa::Error) -> Self {
        CryptoError::SignatureError(format!("ECDSA error: {}", err))
    }
}

#[cfg(feature = "aes-gcm")]
impl From<aes_gcm::Error> for CryptoError {
    fn from(err: aes_gcm::Error) -> Self {
        CryptoError::EncryptionError(format!("AES-GCM error: {}", err))
    }
}

/// Helper functions for common error patterns
impl CryptoError {
    /// Create an invalid input error
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        CryptoError::InvalidInput(msg.into())
    }

    /// Create a key error
    pub fn key_error<S: Into<String>>(msg: S) -> Self {
        CryptoError::KeyError(msg.into())
    }

    /// Create an authentication error
    pub fn auth_error<S: Into<String>>(msg: S) -> Self {
        CryptoError::AuthenticationError(msg.into())
    }

    /// Create an integrity error
    pub fn integrity_error<S: Into<String>>(msg: S) -> Self {
        CryptoError::IntegrityError(msg.into())
    }

    /// Create an unsupported algorithm error
    pub fn unsupported_algorithm<S: Into<String>>(algorithm: S) -> Self {
        CryptoError::UnsupportedAlgorithm(algorithm.into())
    }
}