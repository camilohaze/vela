/*!
# Digital Signature Module

Provides digital signature operations for Vela applications.
Supports RSA and ECDSA algorithms with secure key handling.
*/

use crate::error::{CryptoError, CryptoResult};
use std::fmt;

/// Supported signature algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignatureAlgorithm {
    /// RSA with PKCS#1 v1.5 padding
    RsaPkcs1v15,

    /// RSA with PSS padding
    RsaPss,

    /// ECDSA on P-256 curve
    EcdsaP256,

    /// ECDSA on P-384 curve
    EcdsaP384,

    /// ECDSA on P-521 curve
    EcdsaP521,
}

impl fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignatureAlgorithm::RsaPkcs1v15 => write!(f, "RSA-PKCS#1-v1.5"),
            SignatureAlgorithm::RsaPss => write!(f, "RSA-PSS"),
            SignatureAlgorithm::EcdsaP256 => write!(f, "ECDSA-P-256"),
            SignatureAlgorithm::EcdsaP384 => write!(f, "ECDSA-P-384"),
            SignatureAlgorithm::EcdsaP521 => write!(f, "ECDSA-P-521"),
        }
    }
}

impl From<SignatureAlgorithm> for crate::key::KeyType {
    fn from(algo: SignatureAlgorithm) -> crate::key::KeyType {
        match algo {
            SignatureAlgorithm::RsaPkcs1v15 | SignatureAlgorithm::RsaPss => {
                crate::key::KeyType::Rsa { bits: 2048 }
            }
            SignatureAlgorithm::EcdsaP256 => {
                crate::key::KeyType::Ecdsa { curve: crate::key::EcdsaCurve::P256 }
            }
            SignatureAlgorithm::EcdsaP384 => {
                crate::key::KeyType::Ecdsa { curve: crate::key::EcdsaCurve::P384 }
            }
            SignatureAlgorithm::EcdsaP521 => {
                crate::key::KeyType::Ecdsa { curve: crate::key::EcdsaCurve::P521 }
            }
        }
    }
}

/// Digital signature operations
pub struct CryptoSign;

impl CryptoSign {
    /// Sign data with a private key
    pub async fn sign(
        data: impl AsRef<[u8]> + Send + 'static,
        private_key: impl AsRef<[u8]> + Send + 'static,
        algorithm: SignatureAlgorithm,
    ) -> CryptoResult<Vec<u8>> {
        match algorithm {
            SignatureAlgorithm::RsaPkcs1v15 => {
                Self::sign_rsa_pkcs1v15(data, private_key).await
            }
            SignatureAlgorithm::RsaPss => {
                Self::sign_rsa_pss(data, private_key).await
            }
            SignatureAlgorithm::EcdsaP256 => {
                Self::sign_ecdsa_p256(data, private_key).await
            }
            SignatureAlgorithm::EcdsaP384 => {
                Self::sign_ecdsa_p384(data, private_key).await
            }
            SignatureAlgorithm::EcdsaP521 => {
                Self::sign_ecdsa_p521(data, private_key).await
            }
        }
    }

    /// Verify a signature with a public key
    pub async fn verify(
        data: impl AsRef<[u8]> + Send + 'static,
        signature: impl AsRef<[u8]> + Send + 'static,
        public_key: impl AsRef<[u8]> + Send + 'static,
        algorithm: SignatureAlgorithm,
    ) -> CryptoResult<bool> {
        match algorithm {
            SignatureAlgorithm::RsaPkcs1v15 => {
                Self::verify_rsa_pkcs1v15(data, signature, public_key).await
            }
            SignatureAlgorithm::RsaPss => {
                Self::verify_rsa_pss(data, signature, public_key).await
            }
            SignatureAlgorithm::EcdsaP256 => {
                Self::verify_ecdsa_p256(data, signature, public_key).await
            }
            SignatureAlgorithm::EcdsaP384 => {
                Self::verify_ecdsa_p384(data, signature, public_key).await
            }
            SignatureAlgorithm::EcdsaP521 => {
                Self::verify_ecdsa_p521(data, signature, public_key).await
            }
        }
    }

    /// Sign with RSA PKCS#1 v1.5
    async fn sign_rsa_pkcs1v15(
        data: impl AsRef<[u8]>,
        private_key: impl AsRef<[u8]>,
    ) -> CryptoResult<Vec<u8>> {
        let data_vec = data.as_ref().to_vec();
        let key_vec = private_key.as_ref().to_vec();

        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePrivateKey;
            use rsa::signature::{Signer, SignatureEncoding};

            let private_key_der = rsa::RsaPrivateKey::from_pkcs8_der(&key_vec)
                .map_err(|e| CryptoError::KeyError(format!("Invalid RSA private key: {}", e)))?;

            let signing_key = rsa::pkcs1v15::SigningKey::<sha2::Sha256>::new_unprefixed(private_key_der);
            let signature: rsa::pkcs1v15::Signature = signing_key.sign(&data_vec);

            Ok(signature.to_bytes().to_vec())
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("RSA PKCS#1 v1.5 sign task failed: {}", e)))?
    }

    /// Verify RSA PKCS#1 v1.5 signature
    async fn verify_rsa_pkcs1v15(
        data: impl AsRef<[u8]> + Send + 'static,
        signature: impl AsRef<[u8]> + Send + 'static,
        public_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<bool> {
        let data_vec = data.as_ref().to_vec();
        let sig_vec = signature.as_ref().to_vec();
        let key_vec = public_key.as_ref().to_vec();

        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePublicKey;
            use rsa::signature::Verifier;

            let public_key_der = rsa::RsaPublicKey::from_public_key_der(&key_vec)
                .map_err(|e| CryptoError::KeyError(format!("Invalid RSA public key: {}", e)))?;

            let verifying_key = rsa::pkcs1v15::VerifyingKey::<sha2::Sha256>::new_unprefixed(public_key_der);

            let signature = rsa::pkcs1v15::Signature::try_from(&sig_vec[..])
                .map_err(|e| CryptoError::SignatureError(format!("Invalid signature format: {}", e)))?;

            match verifying_key.verify(&data_vec, &signature) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("RSA PKCS#1 v1.5 verify task failed: {}", e)))?
    }

    /// Sign with RSA PSS
    async fn sign_rsa_pss(
        data: impl AsRef<[u8]> + Send + 'static,
        private_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<Vec<u8>> {
        let data_vec = data.as_ref().to_vec();
        let key_vec = private_key.as_ref().to_vec();

        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePrivateKey;
            use rsa::signature::{RandomizedSigner, SignatureEncoding};

            let private_key_der = rsa::RsaPrivateKey::from_pkcs8_der(&key_vec)
                .map_err(|e| CryptoError::KeyError(format!("Invalid RSA private key: {}", e)))?;

            let signing_key = rsa::pss::SigningKey::<sha2::Sha256>::new(private_key_der);
            let signature: rsa::pss::Signature = signing_key.sign_with_rng(&mut rand::thread_rng(), &data_vec);

            Ok(signature.to_bytes().to_vec())
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("RSA PSS sign task failed: {}", e)))?
    }

    /// Verify RSA PSS signature
    async fn verify_rsa_pss(
        data: impl AsRef<[u8]> + Send + 'static,
        signature: impl AsRef<[u8]> + Send + 'static,
        public_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<bool> {
        let data_vec = data.as_ref().to_vec();
        let sig_vec = signature.as_ref().to_vec();
        let key_vec = public_key.as_ref().to_vec();

        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePublicKey;
            use rsa::signature::Verifier;

            let public_key_der = rsa::RsaPublicKey::from_public_key_der(&key_vec)
                .map_err(|e| CryptoError::KeyError(format!("Invalid RSA public key: {}", e)))?;

            let verifying_key = rsa::pss::VerifyingKey::<sha2::Sha256>::new(public_key_der);

            let signature = rsa::pss::Signature::try_from(&sig_vec[..])
                .map_err(|e| CryptoError::SignatureError(format!("Invalid signature format: {}", e)))?;

            match verifying_key.verify(&data_vec, &signature) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("RSA PSS verify task failed: {}", e)))?
    }

    /// Sign with ECDSA P-256
    async fn sign_ecdsa_p256(
        data: impl AsRef<[u8]> + Send + 'static,
        private_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<Vec<u8>> {
        tokio::task::spawn_blocking(move || {
            use p256::ecdsa::{SigningKey, Signature, signature::Signer};

            let signing_key = SigningKey::from_bytes(private_key.as_ref().into())
                .map_err(|e| CryptoError::KeyError(format!("Invalid ECDSA P-256 private key: {}", e)))?;

            let signature: Signature = signing_key.sign(data.as_ref());
            Ok(signature.to_bytes().to_vec())
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("ECDSA P-256 sign task failed: {}", e)))?
    }

    /// Verify ECDSA P-256 signature
    async fn verify_ecdsa_p256(
        data: impl AsRef<[u8]> + Send + 'static,
        signature: impl AsRef<[u8]> + Send + 'static,
        public_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<bool> {
        tokio::task::spawn_blocking(move || {
            use p256::ecdsa::{VerifyingKey, Signature, signature::Verifier};

            let verifying_key = VerifyingKey::from_sec1_bytes(public_key.as_ref())
                .map_err(|e| CryptoError::KeyError(format!("Invalid ECDSA P-256 public key: {}", e)))?;

            let signature = Signature::from_bytes(signature.as_ref().into())
                .map_err(|e| CryptoError::SignatureError(format!("Invalid signature format: {}", e)))?;

            match verifying_key.verify(data.as_ref(), &signature) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("ECDSA P-256 verify task failed: {}", e)))?
    }

    /// Sign with ECDSA P-384
    async fn sign_ecdsa_p384(
        data: impl AsRef<[u8]> + Send + 'static,
        private_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<Vec<u8>> {
        tokio::task::spawn_blocking(move || {
            use p384::ecdsa::{SigningKey, Signature, signature::Signer};

            let signing_key = SigningKey::from_bytes(private_key.as_ref().into())
                .map_err(|e| CryptoError::KeyError(format!("Invalid ECDSA P-384 private key: {}", e)))?;

            let signature: Signature = signing_key.sign(data.as_ref());
            Ok(signature.to_bytes().to_vec())
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("ECDSA P-384 sign task failed: {}", e)))?
    }

    /// Verify ECDSA P-384 signature
    async fn verify_ecdsa_p384(
        data: impl AsRef<[u8]> + Send + 'static,
        signature: impl AsRef<[u8]> + Send + 'static,
        public_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<bool> {
        tokio::task::spawn_blocking(move || {
            use p384::ecdsa::{VerifyingKey, Signature, signature::Verifier};

            let verifying_key = VerifyingKey::from_sec1_bytes(public_key.as_ref())
                .map_err(|e| CryptoError::KeyError(format!("Invalid ECDSA P-384 public key: {}", e)))?;

            let signature = Signature::from_bytes(signature.as_ref().into())
                .map_err(|e| CryptoError::SignatureError(format!("Invalid signature format: {}", e)))?;

            match verifying_key.verify(data.as_ref(), &signature) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("ECDSA P-384 verify task failed: {}", e)))?
    }

    /// Sign with ECDSA P-521
    async fn sign_ecdsa_p521(
        data: impl AsRef<[u8]> + Send + 'static,
        private_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<Vec<u8>> {
        tokio::task::spawn_blocking(move || {
            use p521::ecdsa::{SigningKey, Signature, signature::Signer};

            let signing_key = SigningKey::from_bytes(private_key.as_ref().into())
                .map_err(|e| CryptoError::KeyError(format!("Invalid ECDSA P-521 private key: {}", e)))?;

            let signature: Signature = signing_key.sign(data.as_ref());
            Ok(signature.to_bytes().to_vec())
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("ECDSA P-521 sign task failed: {}", e)))?
    }

    /// Verify ECDSA P-521 signature
    async fn verify_ecdsa_p521(
        data: impl AsRef<[u8]> + Send + 'static,
        signature: impl AsRef<[u8]> + Send + 'static,
        public_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<bool> {
        tokio::task::spawn_blocking(move || {
            use p521::ecdsa::{VerifyingKey, Signature, signature::Verifier};

            let verifying_key = VerifyingKey::from_sec1_bytes(public_key.as_ref())
                .map_err(|e| CryptoError::KeyError(format!("Invalid ECDSA P-521 public key: {}", e)))?;

            let signature = Signature::from_bytes(signature.as_ref().into())
                .map_err(|e| CryptoError::SignatureError(format!("Invalid signature format: {}", e)))?;

            match verifying_key.verify(data.as_ref(), &signature) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("ECDSA P-521 verify task failed: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::{CryptoKey, KeyType};

    #[tokio::test]
    async fn test_rsa_pkcs1v15_sign_verify() {
        let keypair = CryptoKey::generate_keypair(KeyType::Rsa { bits: 2048 }).await.unwrap();
        let data = b"Hello, World!";

        let signature = CryptoSign::sign(data, keypair.private_key.clone(), SignatureAlgorithm::RsaPkcs1v15).await.unwrap();
        let is_valid = CryptoSign::verify(data, signature, keypair.public_key.clone(), SignatureAlgorithm::RsaPkcs1v15).await.unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_rsa_pss_sign_verify() {
        let keypair = CryptoKey::generate_keypair(KeyType::Rsa { bits: 2048 }).await.unwrap();
        let data = b"Hello, World!";

        let signature = CryptoSign::sign(data, keypair.private_key.clone(), SignatureAlgorithm::RsaPss).await.unwrap();
        let is_valid = CryptoSign::verify(data, signature, keypair.public_key.clone(), SignatureAlgorithm::RsaPss).await.unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_ecdsa_p256_sign_verify() {
        let keypair = CryptoKey::generate_keypair(KeyType::Ecdsa { curve: crate::key::EcdsaCurve::P256 }).await.unwrap();
        let data = b"Hello, World!";

        let signature = CryptoSign::sign(data, keypair.private_key.clone(), SignatureAlgorithm::EcdsaP256).await.unwrap();
        let is_valid = CryptoSign::verify(data, signature, keypair.public_key.clone(), SignatureAlgorithm::EcdsaP256).await.unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_ecdsa_p384_sign_verify() {
        let keypair = CryptoKey::generate_keypair(KeyType::Ecdsa { curve: crate::key::EcdsaCurve::P384 }).await.unwrap();
        let data = b"Hello, World!";

        let signature = CryptoSign::sign(data, keypair.private_key.clone(), SignatureAlgorithm::EcdsaP384).await.unwrap();
        let is_valid = CryptoSign::verify(data, signature, keypair.public_key.clone(), SignatureAlgorithm::EcdsaP384).await.unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_ecdsa_p521_sign_verify() {
        let keypair = CryptoKey::generate_keypair(KeyType::Ecdsa { curve: crate::key::EcdsaCurve::P521 }).await.unwrap();
        let data = b"Hello, World!";

        let signature = CryptoSign::sign(data, keypair.private_key.clone(), SignatureAlgorithm::EcdsaP521).await.unwrap();
        let is_valid = CryptoSign::verify(data, signature, keypair.public_key.clone(), SignatureAlgorithm::EcdsaP521).await.unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_wrong_signature_verification() {
        let keypair = CryptoKey::generate_keypair(KeyType::Rsa { bits: 2048 }).await.unwrap();
        let data = b"Hello, World!";
        let wrong_data = b"Goodbye, World!";

        let signature = CryptoSign::sign(data, keypair.private_key.clone(), SignatureAlgorithm::RsaPkcs1v15).await.unwrap();
        let is_valid = CryptoSign::verify(wrong_data, signature, keypair.public_key.clone(), SignatureAlgorithm::RsaPkcs1v15).await.unwrap();

        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_invalid_private_key() {
        let invalid_key = vec![0u8; 32];
        let data = b"test";

        let result = CryptoSign::sign(data, invalid_key, SignatureAlgorithm::RsaPkcs1v15).await;
        assert!(matches!(result, Err(CryptoError::KeyError(_))));
    }

    #[tokio::test]
    async fn test_invalid_public_key() {
        let keypair = CryptoKey::generate_keypair(KeyType::Rsa { bits: 2048 }).await.unwrap();
        let invalid_key = vec![0u8; 32];
        let data = b"test";

        let signature = CryptoSign::sign(data, keypair.private_key.clone(), SignatureAlgorithm::RsaPkcs1v15).await.unwrap();
        let result = CryptoSign::verify(data, signature, invalid_key, SignatureAlgorithm::RsaPkcs1v15).await;
        assert!(matches!(result, Err(CryptoError::KeyError(_))));
    }

    #[tokio::test]
    async fn test_invalid_signature_format() {
        let keypair = CryptoKey::generate_keypair(KeyType::Rsa { bits: 2048 }).await.unwrap();
        let data = b"test";
        let invalid_signature = vec![0u8; 10]; // Too short for RSA signature

        let result = CryptoSign::verify(data, invalid_signature, keypair.public_key.clone(), SignatureAlgorithm::RsaPkcs1v15).await;
        assert!(result == Ok(false)); // Invalid signature should return false, not an error
    }
}