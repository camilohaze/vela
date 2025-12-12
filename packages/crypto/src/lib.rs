/*!
# Vela Crypto Library

A comprehensive cryptography library for the Vela programming language,
providing secure hashing, encryption, digital signatures, and JWT handling.

## Features

- **Hashing**: SHA256, bcrypt, argon2
- **Symmetric Encryption**: AES-256-GCM, ChaCha20-Poly1305
- **Digital Signatures**: RSA, ECDSA
- **JWT**: Token creation and verification
- **Key Management**: Secure key generation and handling

## Security

This library follows security best practices:
- Secure defaults for all algorithms
- Type-safe APIs to prevent common mistakes
- Constant-time operations where applicable
- No secrets in logs or error messages

## Example

```rust
use vela_crypto::{Crypto, HashAlgorithm, EncryptionAlgorithm};

// Hash a password
let hash = Crypto::hash("password", HashAlgorithm::Argon2 {
    iterations: 3,
    memory: 65536,
    parallelism: 4
}).await?;

// Generate encryption key
let key = Crypto::generate_key(EncryptionAlgorithm::Aes256Gcm).await?;

// Encrypt data
let encrypted = Crypto::encrypt(b"secret data", &key).await?;

// Decrypt data
let decrypted = Crypto::decrypt(&encrypted, &key).await?;
# Ok(())
```
*/

pub mod error;
pub mod hash;
pub mod encrypt;
pub mod jwt;
pub mod sign;
pub mod key;

pub use error::CryptoError;
pub use hash::{HashAlgorithm, CryptoHash};
pub use encrypt::{EncryptionAlgorithm, CryptoEncrypt};
pub use jwt::CryptoJWT;
pub use sign::{SignatureAlgorithm, CryptoSign};
pub use key::{KeyType, KeyFormat, CryptoKey};

/// Main crypto API facade
pub struct Crypto;

impl Crypto {
    /// Hash data using the specified algorithm
    pub async fn hash(data: impl AsRef<[u8]>, algorithm: HashAlgorithm) -> Result<String, CryptoError> {
        CryptoHash::hash(data, algorithm).await
    }

    /// Verify data against a hash
    pub async fn verify(data: impl AsRef<[u8]>, hash: &str) -> Result<bool, CryptoError> {
        CryptoHash::verify(data, hash).await
    }

    /// Generate a new encryption key
    pub async fn generate_key(algorithm: EncryptionAlgorithm) -> Result<Vec<u8>, CryptoError> {
        CryptoKey::generate(algorithm.into()).await
    }

    /// Encrypt data with the given key
    pub async fn encrypt(data: impl AsRef<[u8]> + Send + 'static, key: impl AsRef<[u8]> + Send + 'static) -> Result<Vec<u8>, CryptoError> {
        CryptoEncrypt::encrypt(data, key).await
    }

    /// Decrypt data with the given key
    pub async fn decrypt(data: impl AsRef<[u8]> + Send + 'static, key: impl AsRef<[u8]> + Send + 'static) -> Result<Vec<u8>, CryptoError> {
        CryptoEncrypt::decrypt(data, key).await
    }

    /// Create a JWT token
    pub async fn create_jwt<T: serde::Serialize>(
        payload: &T,
        secret: impl AsRef<[u8]> + Send + 'static,
        algorithm: jwt::JWTAlgorithm,
    ) -> Result<String, CryptoError> {
        CryptoJWT::create(payload, secret, algorithm).await
    }

    /// Verify and decode a JWT token
    pub async fn verify_jwt<T: serde::de::DeserializeOwned>(
        token: &str,
        secret: impl AsRef<[u8]> + Send + 'static,
    ) -> Result<T, CryptoError> {
        CryptoJWT::verify(token, secret).await
    }

    /// Sign data with a private key
    pub async fn sign(
        data: impl AsRef<[u8]> + Send + 'static,
        private_key: impl AsRef<[u8]> + Send + 'static,
        algorithm: SignatureAlgorithm,
    ) -> Result<Vec<u8>, CryptoError> {
        CryptoSign::sign(data, private_key, algorithm).await
    }

    /// Verify a signature with a public key
    pub async fn verify_signature(
        data: impl AsRef<[u8]> + Send + 'static,
        signature: impl AsRef<[u8]> + Send + 'static,
        public_key: impl AsRef<[u8]> + Send + 'static,
        algorithm: SignatureAlgorithm,
    ) -> Result<bool, CryptoError> {
        CryptoSign::verify(data, signature, public_key, algorithm).await
    }
}

/// Convenience re-exports
pub use hash::HashAlgorithm::*;
pub use encrypt::EncryptionAlgorithm::*;
pub use jwt::JWTAlgorithm::*;
pub use sign::SignatureAlgorithm::*;