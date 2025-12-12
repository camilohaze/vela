/*!
# Encryption Module

Provides authenticated encryption with associated data (AEAD) for Vela applications.
Supports AES-256-GCM and ChaCha20-Poly1305 algorithms.
*/

use crate::error::{CryptoError, CryptoResult};
use std::fmt;

/// Supported encryption algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (NIST standard, fast on hardware with AES-NI)
    Aes256Gcm,

    /// ChaCha20-Poly1305 (secure, fast on software)
    ChaCha20Poly1305,
}

impl fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionAlgorithm::Aes256Gcm => write!(f, "AES-256-GCM"),
            EncryptionAlgorithm::ChaCha20Poly1305 => write!(f, "ChaCha20-Poly1305"),
        }
    }
}

impl From<EncryptionAlgorithm> for crate::key::KeyType {
    fn from(algo: EncryptionAlgorithm) -> crate::key::KeyType {
        match algo {
            EncryptionAlgorithm::Aes256Gcm => crate::key::KeyType::Symmetric { size: 32 },
            EncryptionAlgorithm::ChaCha20Poly1305 => crate::key::KeyType::Symmetric { size: 32 },
        }
    }
}

/// Encryption operations
pub struct CryptoEncrypt;

impl CryptoEncrypt {
    /// Encrypt data with the given key
    pub async fn encrypt(
        data: impl AsRef<[u8]>,
        key: impl AsRef<[u8]>,
    ) -> CryptoResult<Vec<u8>> {
        Self::encrypt_with_aad(data, key, &[]).await
    }

    /// Encrypt data with associated authenticated data (AAD)
    pub async fn encrypt_with_aad(
        data: impl AsRef<[u8]>,
        key: impl AsRef<[u8]>,
        aad: impl AsRef<[u8]>,
    ) -> CryptoResult<Vec<u8>> {
        let key_bytes = key.as_ref();

        // Determine algorithm from key size
        match key_bytes.len() {
            32 => Self::encrypt_aes256_gcm(data, key_bytes, aad).await,
            _ => Err(CryptoError::invalid_input("Invalid key size for encryption")),
        }
    }

    /// Decrypt data with the given key
    pub async fn decrypt(
        encrypted_data: impl AsRef<[u8]>,
        key: impl AsRef<[u8]>,
    ) -> CryptoResult<Vec<u8>> {
        Self::decrypt_with_aad(encrypted_data, key, &[]).await
    }

    /// Decrypt data with associated authenticated data (AAD)
    pub async fn decrypt_with_aad(
        encrypted_data: impl AsRef<[u8]>,
        key: impl AsRef<[u8]>,
        aad: impl AsRef<[u8]>,
    ) -> CryptoResult<Vec<u8>> {
        let key_bytes = key.as_ref();
        let data = encrypted_data.as_ref();

        if data.len() < 28 { // Minimum: nonce(12) + tag(16) + 0 data
            return Err(CryptoError::invalid_input("Encrypted data too short"));
        }

        // Determine algorithm from key size
        match key_bytes.len() {
            32 => Self::decrypt_aes256_gcm(data, key_bytes, aad).await,
            _ => Err(CryptoError::invalid_input("Invalid key size for decryption")),
        }
    }

    /// Encrypt with AES-256-GCM
    async fn encrypt_aes256_gcm(
        data: impl AsRef<[u8]>,
        key: &[u8],
        aad: impl AsRef<[u8]>,
    ) -> CryptoResult<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, Key};
        use aes_gcm::aead::{Aead, AeadCore, KeyInit};

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Aes256Gcm::generate_nonce(&mut rand::thread_rng());

        let data_clone = data.as_ref().to_vec();
        let aad_clone = aad.as_ref().to_vec();

        tokio::task::spawn_blocking(move || {
        let ciphertext = cipher.encrypt(&nonce, aes_gcm::aead::Payload {
            msg: &data_clone,
            aad: &aad_clone,
        }).map_err(|e| CryptoError::EncryptionError(format!("AES-GCM encryption failed: {}", e)))?;

            // Format: nonce(12) + ciphertext + tag(16)
            let mut result = nonce.to_vec();
            result.extend_from_slice(&ciphertext);

            Ok(result)
        })
        .await
        .map_err(|e| CryptoError::EncryptionError(format!("AES-GCM task failed: {}", e)))?
    }

    /// Decrypt with AES-256-GCM
    async fn decrypt_aes256_gcm(
        encrypted_data: impl AsRef<[u8]>,
        key: impl AsRef<[u8]>,
        aad: impl AsRef<[u8]>,
    ) -> CryptoResult<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, Key, Nonce};
        use aes_gcm::aead::{Aead, KeyInit};

        let encrypted_vec = encrypted_data.as_ref().to_vec();
        let key_vec = key.as_ref().to_vec();
        let aad_vec = aad.as_ref().to_vec();

        tokio::task::spawn_blocking(move || {
            if encrypted_vec.len() < 28 {
                return Err(CryptoError::invalid_input("AES-GCM data too short"));
            }

            let (nonce_bytes, ciphertext_and_tag) = encrypted_vec.split_at(12);
            let nonce = Nonce::from_slice(nonce_bytes);

            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_vec));

            cipher.decrypt(nonce, aes_gcm::aead::Payload {
                msg: ciphertext_and_tag,
                aad: &aad_vec,
            }).map_err(|_| CryptoError::auth_error("AES-GCM decryption failed - invalid key or corrupted data"))
        })
        .await
        .map_err(|e| CryptoError::EncryptionError(format!("AES-GCM task failed: {}", e)))?
    }

    /// Generate a random nonce for encryption
    pub fn generate_nonce() -> [u8; 12] {
        use rand::RngCore;
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }

    /// Get the key size for an algorithm
    pub const fn key_size(algorithm: EncryptionAlgorithm) -> usize {
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 32,
            EncryptionAlgorithm::ChaCha20Poly1305 => 32,
        }
    }

    /// Get the nonce size for an algorithm
    pub const fn nonce_size(algorithm: EncryptionAlgorithm) -> usize {
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 12,
            EncryptionAlgorithm::ChaCha20Poly1305 => 12,
        }
    }

    /// Get the authentication tag size for an algorithm
    pub const fn tag_size(algorithm: EncryptionAlgorithm) -> usize {
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 16,
            EncryptionAlgorithm::ChaCha20Poly1305 => 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aes256_gcm_encrypt_decrypt() {
        let data = b"Hello, World! This is a test message.";
        let key = vec![0u8; 32]; // 32 bytes for AES-256

        let encrypted = CryptoEncrypt::encrypt(data, &key).await.unwrap();
        let decrypted = CryptoEncrypt::decrypt(&encrypted, &key).await.unwrap();

        assert_eq!(data.as_ref(), decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_aes256_gcm_encrypt_decrypt_with_aad() {
        let data = b"Secret message";
        let key = vec![1u8; 32];
        let aad = b"Associated data";

        let encrypted = CryptoEncrypt::encrypt_with_aad(data, &key, aad).await.unwrap();
        let decrypted = CryptoEncrypt::decrypt_with_aad(&encrypted, &key, aad).await.unwrap();

        assert_eq!(data.as_ref(), decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_aes256_gcm_wrong_key() {
        let data = b"Secret data";
        let key = vec![0u8; 32];
        let wrong_key = vec![1u8; 32];

        let encrypted = CryptoEncrypt::encrypt(data, &key).await.unwrap();
        let result = CryptoEncrypt::decrypt(&encrypted, &wrong_key).await;

        assert!(matches!(result, Err(CryptoError::AuthenticationError(_))));
    }

    #[tokio::test]
    async fn test_aes256_gcm_wrong_aad() {
        let data = b"Secret data";
        let key = vec![0u8; 32];
        let aad = b"Correct AAD";
        let wrong_aad = b"Wrong AAD";

        let encrypted = CryptoEncrypt::encrypt_with_aad(data, &key, aad).await.unwrap();
        let result = CryptoEncrypt::decrypt_with_aad(&encrypted, &key, wrong_aad).await;

        assert!(matches!(result, Err(CryptoError::AuthenticationError(_))));
    }

    #[tokio::test]
    async fn test_aes256_gcm_tampered_data() {
        let data = b"Secret data";
        let key = vec![0u8; 32];

        let mut encrypted = CryptoEncrypt::encrypt(data, &key).await.unwrap();

        // Tamper with the ciphertext
        if encrypted.len() > 20 {
            encrypted[20] ^= 1;
        }

        let result = CryptoEncrypt::decrypt(&encrypted, &key).await;
        assert!(matches!(result, Err(CryptoError::AuthenticationError(_))));
    }

    #[tokio::test]
    async fn test_invalid_key_size() {
        let data = b"test";
        let invalid_key = vec![0u8; 16]; // Wrong size

        let result = CryptoEncrypt::encrypt(data, &invalid_key).await;
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_too_short_encrypted_data() {
        let key = vec![0u8; 32];
        let too_short_data = vec![0u8; 10];

        let result = CryptoEncrypt::decrypt(&too_short_data, &key).await;
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }

    #[test]
    fn test_key_sizes() {
        assert_eq!(CryptoEncrypt::key_size(EncryptionAlgorithm::Aes256Gcm), 32);
        assert_eq!(CryptoEncrypt::key_size(EncryptionAlgorithm::ChaCha20Poly1305), 32);
    }

    #[test]
    fn test_nonce_sizes() {
        assert_eq!(CryptoEncrypt::nonce_size(EncryptionAlgorithm::Aes256Gcm), 12);
        assert_eq!(CryptoEncrypt::nonce_size(EncryptionAlgorithm::ChaCha20Poly1305), 12);
    }

    #[test]
    fn test_tag_sizes() {
        assert_eq!(CryptoEncrypt::tag_size(EncryptionAlgorithm::Aes256Gcm), 16);
        assert_eq!(CryptoEncrypt::tag_size(EncryptionAlgorithm::ChaCha20Poly1305), 16);
    }
}