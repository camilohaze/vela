/*!
# Hashing Module

Provides secure hashing algorithms for Vela applications.
Supports password hashing with bcrypt and argon2, and general-purpose hashing with SHA256.
*/

use crate::error::{CryptoError, CryptoResult};
use std::fmt;

/// Supported hash algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum HashAlgorithm {
    /// SHA256 for general-purpose hashing
    Sha256,

    /// bcrypt for password hashing (cost factor)
    Bcrypt {
        /// Cost parameter (4-31, higher = more secure but slower)
        cost: u32,
    },

    /// Argon2 for password hashing
    Argon2 {
        /// Number of iterations
        iterations: u32,
        /// Memory usage in KiB
        memory: u32,
        /// Degree of parallelism
        parallelism: u32,
    },
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashAlgorithm::Sha256 => write!(f, "SHA256"),
            HashAlgorithm::Bcrypt { cost } => write!(f, "bcrypt(cost={})", cost),
            HashAlgorithm::Argon2 { iterations, memory, parallelism } => {
                write!(f, "argon2(iterations={}, memory={}KiB, parallelism={})",
                       iterations, memory, parallelism)
            }
        }
    }
}

/// Hashing operations
pub struct CryptoHash;

impl CryptoHash {
    /// Hash data using the specified algorithm
    pub async fn hash(data: impl AsRef<[u8]>, algorithm: HashAlgorithm) -> CryptoResult<String> {
        match algorithm {
            HashAlgorithm::Sha256 => Self::hash_sha256(data),
            HashAlgorithm::Bcrypt { cost } => Self::hash_bcrypt(data, cost).await,
            HashAlgorithm::Argon2 { iterations, memory, parallelism } => {
                Self::hash_argon2(data, iterations, memory, parallelism).await
            }
        }
    }

    /// Verify data against a hash
    pub async fn verify(data: impl AsRef<[u8]>, hash: &str) -> CryptoResult<bool> {
        // Determine algorithm from hash format
        if hash.starts_with("$2b$") || hash.starts_with("$2a$") || hash.starts_with("$2y$") {
            // bcrypt format
            Self::verify_bcrypt(data, hash).await
        } else if hash.starts_with("$argon2") {
            // argon2 format
            Self::verify_argon2(data, hash).await
        } else if hash.starts_with("sha256:") {
            // SHA256 format
            Self::verify_sha256(data, hash)
        } else {
            Err(CryptoError::invalid_input("Unknown hash format"))
        }
    }

    /// Hash with SHA256
    fn hash_sha256(data: impl AsRef<[u8]>) -> CryptoResult<String> {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let hash_hex = hex::encode(result);

        Ok(format!("sha256:{}", hash_hex))
    }

    /// Verify SHA256 hash
    fn verify_sha256(data: impl AsRef<[u8]>, hash: &str) -> CryptoResult<bool> {
        if !hash.starts_with("sha256:") {
            return Err(CryptoError::invalid_input("Invalid SHA256 hash format"));
        }

        let expected_hex = &hash[7..]; // Remove "sha256:" prefix
        let expected = hex::decode(expected_hex)
            .map_err(|_| CryptoError::invalid_input("Invalid hex in SHA256 hash"))?;

        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let computed = hasher.finalize();

        Ok(computed.as_slice() == expected)
    }

    /// Hash with bcrypt
    async fn hash_bcrypt(data: impl AsRef<[u8]>, cost: u32) -> CryptoResult<String> {
        // Validate cost parameter
        if cost < 4 || cost > 31 {
            return Err(CryptoError::invalid_input("bcrypt cost must be between 4 and 31"));
        }

        // Convert data to string for bcrypt
        let password = std::str::from_utf8(data.as_ref())
            .map_err(|_| CryptoError::invalid_input("Password must be valid UTF-8"))?;

        // Use tokio::task::spawn_blocking for CPU-intensive bcrypt operation
        let cost_clone = cost;
        let password_clone = password.to_string();

        tokio::task::spawn_blocking(move || {
            bcrypt::hash(password_clone, cost_clone)
                .map_err(|e| CryptoError::HashError(format!("bcrypt hash failed: {}", e)))
        })
        .await
        .map_err(|e| CryptoError::HashError(format!("bcrypt task failed: {}", e)))?
    }

    /// Verify bcrypt hash
    async fn verify_bcrypt(data: impl AsRef<[u8]>, hash: &str) -> CryptoResult<bool> {
        let password = std::str::from_utf8(data.as_ref())
            .map_err(|_| CryptoError::invalid_input("Password must be valid UTF-8"))?;

        let password_clone = password.to_string();
        let hash_clone = hash.to_string();

        tokio::task::spawn_blocking(move || {
            bcrypt::verify(password_clone, &hash_clone)
                .map_err(|e| CryptoError::HashError(format!("bcrypt verify failed: {}", e)))
        })
        .await
        .map_err(|e| CryptoError::HashError(format!("bcrypt verify task failed: {}", e)))?
    }

    /// Hash with Argon2
    async fn hash_argon2(
        data: impl AsRef<[u8]>,
        iterations: u32,
        memory: u32,
        parallelism: u32,
    ) -> CryptoResult<String> {
        use argon2::{Argon2, Algorithm, Version, Params};

        // Validate parameters
        if iterations == 0 {
            return Err(CryptoError::invalid_input("iterations must be > 0"));
        }
        if memory < 8 {
            return Err(CryptoError::invalid_input("memory must be at least 8 KiB"));
        }
        if parallelism == 0 {
            return Err(CryptoError::invalid_input("parallelism must be > 0"));
        }

        let params = Params::new(memory * 1024, iterations, parallelism, Some(32))
            .map_err(|e| CryptoError::invalid_input(format!("Invalid Argon2 parameters: {}", e)))?;

        let _argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let salt = Self::generate_salt()?;
        let data_clone = data.as_ref().to_vec();
        let salt_clone = salt.clone();
        let memory_clone = memory;
        let iterations_clone = iterations;
        let parallelism_clone = parallelism;

        let hash = tokio::task::spawn_blocking(move || {
            use argon2::{Argon2, Algorithm, Version, Params, PasswordHasher};
            use argon2::password_hash::SaltString;

            let params = Params::new(memory_clone * 1024, iterations_clone, parallelism_clone, Some(32))
                .map_err(|e| CryptoError::invalid_input(format!("Invalid Argon2 parameters: {}", e)))?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

            let salt_string = SaltString::encode_b64(&salt_clone)
                .map_err(|e| CryptoError::HashError(format!("Salt encoding failed: {}", e)))?;

            let hash_result = argon2.hash_password(&data_clone, &salt_string)
                .map_err(|e| CryptoError::HashError(format!("Argon2 hash failed: {}", e)))?;

            Ok::<String, CryptoError>(hash_result.to_string())
        })
        .await
        .map_err(|e| CryptoError::HashError(format!("Argon2 task failed: {}", e)))??;

        Ok(hash.to_string())
    }

    /// Verify Argon2 hash
    async fn verify_argon2(data: impl AsRef<[u8]>, hash: &str) -> CryptoResult<bool> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let data_vec = data.as_ref().to_vec();
        let hash_string = hash.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash_string)
                .map_err(|e| CryptoError::invalid_input(format!("Invalid Argon2 hash format: {}", e)))?;

            let argon2 = Argon2::default();
            match argon2.verify_password(&data_vec, &parsed_hash) {
                Ok(()) => Ok(true), // Verification successful
                Err(_) => Ok(false), // Verification failed (wrong password)
            }
        })
        .await
        .map_err(|e| CryptoError::HashError(format!("Argon2 verify task failed: {}", e)))?;

        result
    }

    /// Generate a random salt for password hashing
    fn generate_salt() -> CryptoResult<Vec<u8>> {
        use rand::RngCore;
        let mut salt = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        Ok(salt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sha256_hash() {
        let data = b"hello world";
        let hash = CryptoHash::hash(data, HashAlgorithm::Sha256).await.unwrap();

        assert!(hash.starts_with("sha256:"));
        assert!(CryptoHash::verify(data, &hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_sha256_verify_wrong_data() {
        let data = b"hello world";
        let hash = CryptoHash::hash(data, HashAlgorithm::Sha256).await.unwrap();

        assert!(!CryptoHash::verify(b"wrong data", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_bcrypt_hash_and_verify() {
        let password = b"mysecretpassword";
        let hash = CryptoHash::hash(password, HashAlgorithm::Bcrypt { cost: 8 }).await.unwrap();

        assert!(hash.starts_with("$2b$"));
        assert!(CryptoHash::verify(password, &hash).await.unwrap());
        assert!(!CryptoHash::verify(b"wrongpassword", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_argon2_hash_and_verify() {
        let password = b"mysecretpassword";
        let hash = CryptoHash::hash(password, HashAlgorithm::Argon2 {
            iterations: 2,
            memory: 64, // 64 KiB instead of 65536
            parallelism: 1,
        }).await.unwrap();

        assert!(hash.starts_with("$argon2id$"));
        assert!(CryptoHash::verify(password, &hash).await.unwrap());
        assert!(!CryptoHash::verify(b"wrongpassword", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_invalid_bcrypt_cost() {
        let result = CryptoHash::hash(b"password", HashAlgorithm::Bcrypt { cost: 2 }).await;
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_unknown_hash_format() {
        let result = CryptoHash::verify(b"data", "unknown:hash").await;
        assert!(matches!(result, Err(CryptoError::InvalidInput(_))));
    }
}