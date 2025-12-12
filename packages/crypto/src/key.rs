/*!
# Key Management Module

Provides secure key generation, import/export, and management for Vela applications.
Supports symmetric and asymmetric key types with proper entropy.
*/

use crate::error::{CryptoError, CryptoResult};
use std::fmt;

/// Supported key types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyType {
    /// Symmetric key for encryption
    Symmetric {
        /// Key size in bytes
        size: usize,
    },

    /// RSA key pair
    Rsa {
        /// Key size in bits (2048, 3072, 4096)
        bits: usize,
    },

    /// ECDSA key pair
    Ecdsa {
        /// Curve type
        curve: EcdsaCurve,
    },
}

/// ECDSA curve types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EcdsaCurve {
    /// NIST P-256 curve
    P256,

    /// NIST P-384 curve
    P384,

    /// NIST P-521 curve
    P521,
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyType::Symmetric { size } => write!(f, "Symmetric({} bytes)", size),
            KeyType::Rsa { bits } => write!(f, "RSA({} bits)", bits),
            KeyType::Ecdsa { curve } => write!(f, "ECDSA({})", curve),
        }
    }
}

impl fmt::Display for EcdsaCurve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcdsaCurve::P256 => write!(f, "P-256"),
            EcdsaCurve::P384 => write!(f, "P-384"),
            EcdsaCurve::P521 => write!(f, "P-521"),
        }
    }
}

/// Key format for import/export
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyFormat {
    /// Raw bytes (for symmetric keys)
    Raw,

    /// PKCS#8 DER format (for private keys)
    Pkcs8Der,

    /// PKCS#8 PEM format (for private keys)
    Pkcs8Pem,

    /// SPKI DER format (for public keys)
    SpkiDer,

    /// SPKI PEM format (for public keys)
    SpkiPem,

    /// JSON Web Key (JWK) format
    Jwk,
}

impl fmt::Display for KeyFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyFormat::Raw => write!(f, "Raw"),
            KeyFormat::Pkcs8Der => write!(f, "PKCS#8 DER"),
            KeyFormat::Pkcs8Pem => write!(f, "PKCS#8 PEM"),
            KeyFormat::SpkiDer => write!(f, "SPKI DER"),
            KeyFormat::SpkiPem => write!(f, "SPKI PEM"),
            KeyFormat::Jwk => write!(f, "JWK"),
        }
    }
}

/// Key pair structure
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// Private key bytes
    pub private_key: Vec<u8>,

    /// Public key bytes
    pub public_key: Vec<u8>,
}

/// Key management operations
pub struct CryptoKey;

impl CryptoKey {
    /// Generate a new key or key pair
    pub async fn generate(key_type: KeyType) -> CryptoResult<Vec<u8>> {
        match key_type {
            KeyType::Symmetric { size } => Self::generate_symmetric_key(size).await,
            KeyType::Rsa { bits } => {
                let keypair = Self::generate_rsa_keypair(bits).await?;
                // Return private key for RSA
                Ok(keypair.private_key)
            }
            KeyType::Ecdsa { curve } => {
                let keypair = Self::generate_ecdsa_keypair(curve).await?;
                // Return private key for ECDSA
                Ok(keypair.private_key)
            }
        }
    }

    /// Generate a key pair
    pub async fn generate_keypair(key_type: KeyType) -> CryptoResult<KeyPair> {
        match key_type {
            KeyType::Symmetric { .. } => {
                Err(CryptoError::invalid_input("Symmetric keys don't have key pairs"))
            }
            KeyType::Rsa { bits } => Self::generate_rsa_keypair(bits).await,
            KeyType::Ecdsa { curve } => Self::generate_ecdsa_keypair(curve).await,
        }
    }

    /// Generate a symmetric key
    async fn generate_symmetric_key(size: usize) -> CryptoResult<Vec<u8>> {
        // Validate key size
        if size == 0 || size > 64 {
            return Err(CryptoError::invalid_input("Invalid symmetric key size"));
        }

        use rand::RngCore;
        let mut key = vec![0u8; size];
        rand::thread_rng().fill_bytes(&mut key);
        Ok(key)
    }

    /// Generate an RSA key pair
    async fn generate_rsa_keypair(bits: usize) -> CryptoResult<KeyPair> {
        // Validate key size
        if ![2048, 3072, 4096].contains(&bits) {
            return Err(CryptoError::invalid_input("RSA key size must be 2048, 3072, or 4096 bits"));
        }

        tokio::task::spawn_blocking(move || {
            use rsa::{RsaPrivateKey, pkcs8::EncodePrivateKey, pkcs8::EncodePublicKey};

            // Generate private key
            let mut rng = rand::thread_rng();
            let private_key = RsaPrivateKey::new(&mut rng, bits)
                .map_err(|e| CryptoError::KeyError(format!("RSA key generation failed: {}", e)))?;

            // Derive public key
            let public_key = private_key.to_public_key();

            // Export to PKCS#8 DER format
            let private_key_der = private_key.to_pkcs8_der()
                .map_err(|e| CryptoError::EncodingError(format!("Private key encoding failed: {}", e)))?
                .as_bytes()
                .to_vec();

            let public_key_der = public_key.to_public_key_der()
                .map_err(|e| CryptoError::EncodingError(format!("Public key encoding failed: {}", e)))?
                .as_bytes()
                .to_vec();

            Ok(KeyPair {
                private_key: private_key_der,
                public_key: public_key_der,
            })
        })
        .await
        .map_err(|e| CryptoError::KeyError(format!("RSA key generation task failed: {}", e)))?
    }

    /// Generate an ECDSA key pair
    async fn generate_ecdsa_keypair(curve: EcdsaCurve) -> CryptoResult<KeyPair> {
        tokio::task::spawn_blocking(move || {
            use p256::ecdsa::{SigningKey, VerifyingKey};
            use p384::ecdsa::{SigningKey as SigningKeyP384, VerifyingKey as VerifyingKeyP384};
            use p521::ecdsa::{SigningKey as SigningKeyP521, VerifyingKey as VerifyingKeyP521};

            match curve {
                EcdsaCurve::P256 => {
                    let signing_key = SigningKey::random(&mut rand::thread_rng());
                    let verifying_key = VerifyingKey::from(&signing_key);

                    let private_key = signing_key.to_bytes().to_vec();
                    let public_key = verifying_key.to_encoded_point(false).as_bytes().to_vec();

                    Ok(KeyPair { private_key, public_key })
                }
                EcdsaCurve::P384 => {
                    let signing_key = SigningKeyP384::random(&mut rand::thread_rng());
                    let verifying_key = VerifyingKeyP384::from(&signing_key);

                    let private_key = signing_key.to_bytes().to_vec();
                    let public_key = verifying_key.to_encoded_point(false).as_bytes().to_vec();

                    Ok(KeyPair { private_key, public_key })
                }
                EcdsaCurve::P521 => {
                    let signing_key = SigningKeyP521::random(&mut rand::thread_rng());
                    let verifying_key = VerifyingKeyP521::from(&signing_key);

                    let private_key = signing_key.to_bytes().to_vec();
                    let public_key = verifying_key.to_encoded_point(false).as_bytes().to_vec();

                    Ok(KeyPair { private_key, public_key })
                }
            }
        })
        .await
        .map_err(|e| CryptoError::KeyError(format!("ECDSA key generation task failed: {}", e)))?
    }

    /// Export a key to the specified format
    pub async fn export_key(
        key_data: impl AsRef<[u8]> + Send + 'static,
        key_type: KeyType,
        format: KeyFormat,
    ) -> CryptoResult<String> {
        match (key_type, format) {
            (KeyType::Symmetric { .. }, KeyFormat::Raw) => {
                Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key_data))
            }
            (KeyType::Rsa { .. }, KeyFormat::Pkcs8Pem) => {
                Self::export_rsa_private_pem(key_data).await
            }
            (KeyType::Rsa { .. }, KeyFormat::SpkiPem) => {
                Self::export_rsa_public_pem(key_data).await
            }
            (KeyType::Ecdsa { .. }, KeyFormat::Raw) => {
                Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key_data))
            }
            _ => Err(CryptoError::unsupported_algorithm(
                format!("Export format {} not supported for key type {}", format, key_type)
            )),
        }
    }

    /// Import a key from the specified format
    pub async fn import_key(
        key_string: &str,
        key_type: KeyType,
        format: KeyFormat,
    ) -> CryptoResult<Vec<u8>> {
        match (key_type, format) {
            (KeyType::Symmetric { .. }, KeyFormat::Raw) => {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, key_string)
                    .map_err(|_| CryptoError::EncodingError("Invalid base64 for symmetric key".to_string()))
            }
            (KeyType::Rsa { .. }, KeyFormat::Pkcs8Pem) => {
                Self::import_rsa_private_pem(key_string).await
            }
            (KeyType::Rsa { .. }, KeyFormat::SpkiPem) => {
                Self::import_rsa_public_pem(key_string).await
            }
            (KeyType::Ecdsa { .. }, KeyFormat::Raw) => {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, key_string)
                    .map_err(|_| CryptoError::EncodingError("Invalid base64 for ECDSA key".to_string()))
            }
            _ => Err(CryptoError::unsupported_algorithm(
                format!("Import format {} not supported for key type {}", format, key_type)
            )),
        }
    }

    /// Export RSA private key to PEM format
    async fn export_rsa_private_pem(key_data: impl AsRef<[u8]> + Send + 'static) -> CryptoResult<String> {
        use rsa::pkcs8::EncodePrivateKey;

        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePrivateKey;

            let private_key = rsa::RsaPrivateKey::from_pkcs8_der(key_data.as_ref())
                .map_err(|e| CryptoError::EncodingError(format!("Invalid RSA private key: {}", e)))?;

            let pem = private_key.to_pkcs8_pem(Default::default())
                .map_err(|e| CryptoError::EncodingError(format!("PEM encoding failed: {}", e)))?;

            Ok(pem.to_string())
        })
        .await
        .map_err(|e| CryptoError::EncodingError(format!("RSA export task failed: {}", e)))?
    }

    /// Export RSA public key to PEM format
    async fn export_rsa_public_pem(key_data: impl AsRef<[u8]> + Send + 'static) -> CryptoResult<String> {
        use rsa::pkcs8::EncodePublicKey;

        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePublicKey;

            let public_key = rsa::RsaPublicKey::from_public_key_der(key_data.as_ref())
                .map_err(|e| CryptoError::EncodingError(format!("Invalid RSA public key: {}", e)))?;

            let pem = public_key.to_public_key_pem(Default::default())
                .map_err(|e| CryptoError::EncodingError(format!("PEM encoding failed: {}", e)))?;

            Ok(pem.to_string())
        })
        .await
        .map_err(|e| CryptoError::EncodingError(format!("RSA export task failed: {}", e)))?
    }

    /// Import RSA private key from PEM format
    async fn import_rsa_private_pem(pem: &str) -> CryptoResult<Vec<u8>> {
        use rsa::pkcs8::EncodePrivateKey;

        let pem_string = pem.to_string();

        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePrivateKey;

            let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(&pem_string)
                .map_err(|e| CryptoError::EncodingError(format!("Invalid RSA PEM: {}", e)))?;

            private_key.to_pkcs8_der()
                .map_err(|e| CryptoError::EncodingError(format!("DER encoding failed: {}", e)))
                .map(|der| der.as_bytes().to_vec())
        })
        .await
        .map_err(|e| CryptoError::EncodingError(format!("RSA import task failed: {}", e)))?
    }

    /// Import RSA public key from PEM format
    async fn import_rsa_public_pem(pem: &str) -> CryptoResult<Vec<u8>> {
        use rsa::pkcs8::EncodePublicKey;

        let pem_string = pem.to_string();

        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePublicKey;

            let public_key = rsa::RsaPublicKey::from_public_key_pem(&pem_string)
                .map_err(|e| CryptoError::EncodingError(format!("Invalid RSA PEM: {}", e)))?;

            public_key.to_public_key_der()
                .map_err(|e| CryptoError::EncodingError(format!("DER encoding failed: {}", e)))
                .map(|der| der.as_bytes().to_vec())
        })
        .await
        .map_err(|e| CryptoError::EncodingError(format!("RSA import task failed: {}", e)))?
    }

    /// Derive a key from a password using PBKDF2
    pub async fn derive_key_from_password(
        password: impl AsRef<[u8]> + Send + 'static,
        salt: impl AsRef<[u8]> + Send + 'static,
        key_size: usize,
        iterations: u32,
    ) -> CryptoResult<Vec<u8>> {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;

        if key_size == 0 || key_size > 64 {
            return Err(CryptoError::invalid_input("Invalid derived key size"));
        }
        if iterations < 1000 {
            return Err(CryptoError::invalid_input("PBKDF2 iterations must be at least 1000"));
        }

        let mut key = vec![0u8; key_size];

        tokio::task::spawn_blocking(move || {
            pbkdf2_hmac::<Sha256>(
                password.as_ref(),
                salt.as_ref(),
                iterations,
                &mut key,
            );
            Ok(key)
        })
        .await
        .map_err(|e| CryptoError::KeyError(format!("PBKDF2 derivation failed: {}", e)))?
    }

    /// Generate a random salt
    pub fn generate_salt(size: usize) -> CryptoResult<Vec<u8>> {
        if size == 0 || size > 64 {
            return Err(CryptoError::invalid_input("Invalid salt size"));
        }

        use rand::RngCore;
        let mut salt = vec![0u8; size];
        rand::thread_rng().fill_bytes(&mut salt);
        Ok(salt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_symmetric_key() {
        let key = CryptoKey::generate(KeyType::Symmetric { size: 32 }).await.unwrap();
        assert_eq!(key.len(), 32);
    }

    #[tokio::test]
    async fn test_generate_rsa_keypair() {
        let keypair = CryptoKey::generate_keypair(KeyType::Rsa { bits: 2048 }).await.unwrap();
        assert!(!keypair.private_key.is_empty());
        assert!(!keypair.public_key.is_empty());
    }

    #[tokio::test]
    async fn test_generate_ecdsa_keypair() {
        let keypair = CryptoKey::generate_keypair(KeyType::Ecdsa { curve: EcdsaCurve::P256 }).await.unwrap();
        assert!(!keypair.private_key.is_empty());
        assert!(!keypair.public_key.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_rsa_key_size() {
        let result = CryptoKey::generate_keypair(KeyType::Rsa { bits: 1024 }).await;
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_invalid_symmetric_key_size() {
        let result = CryptoKey::generate(KeyType::Symmetric { size: 0 }).await;
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_symmetric_keypair_error() {
        let result = CryptoKey::generate_keypair(KeyType::Symmetric { size: 32 }).await;
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_derive_key_from_password() {
        let password = b"mysecretpassword";
        let salt = CryptoKey::generate_salt(16).unwrap();

        let key = CryptoKey::derive_key_from_password(password, salt.clone(), 32, 10000).await.unwrap();
        assert_eq!(key.len(), 32);

        // Same password and salt should produce same key
        let key2 = CryptoKey::derive_key_from_password(password, salt.clone(), 32, 10000).await.unwrap();
        assert_eq!(key, key2);

        // Different salt should produce different key
        let salt2 = CryptoKey::generate_salt(16).unwrap();
        let key3 = CryptoKey::derive_key_from_password(password, salt2, 32, 10000).await.unwrap();
        assert_ne!(key, key3);
    }

    #[tokio::test]
    async fn test_invalid_pbkdf2_iterations() {
        let password = b"password";
        let salt = CryptoKey::generate_salt(16).unwrap();

        let result = CryptoKey::derive_key_from_password(password, salt, 32, 500).await;
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_generate_salt() {
        let salt = CryptoKey::generate_salt(16).unwrap();
        assert_eq!(salt.len(), 16);
    }

    #[test]
    fn test_invalid_salt_size() {
        let result = CryptoKey::generate_salt(0);
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }
}