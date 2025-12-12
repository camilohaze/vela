/*!
# JWT (JSON Web Token) Module

Provides JWT creation and verification for Vela applications.
Supports HS256, RS256, and ES256 algorithms with secure defaults.
*/

use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// JWT header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTHeader {
    /// Algorithm used
    pub alg: String,

    /// Type (always "JWT")
    pub typ: String,

    /// Key ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
}

/// JWT payload (claims)
pub type JWTPayload = BTreeMap<String, serde_json::Value>;

/// Supported JWT algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JWTAlgorithm {
    /// HMAC using SHA-256
    HS256,

    /// RSASSA-PKCS1-v1_5 using SHA-256
    RS256,

    /// ECDSA using P-256 and SHA-256
    ES256,
}

impl fmt::Display for JWTAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JWTAlgorithm::HS256 => write!(f, "HS256"),
            JWTAlgorithm::RS256 => write!(f, "RS256"),
            JWTAlgorithm::ES256 => write!(f, "ES256"),
        }
    }
}

impl From<JWTAlgorithm> for crate::key::KeyType {
    fn from(algo: JWTAlgorithm) -> crate::key::KeyType {
        match algo {
            JWTAlgorithm::HS256 => crate::key::KeyType::Symmetric { size: 32 },
            JWTAlgorithm::RS256 => crate::key::KeyType::Rsa { bits: 2048 },
            JWTAlgorithm::ES256 => crate::key::KeyType::Ecdsa { curve: crate::key::EcdsaCurve::P256 },
        }
    }
}

/// JWT operations
pub struct CryptoJWT;

impl CryptoJWT {
    /// Create a JWT token
    pub async fn create<T: Serialize>(
        payload: &T,
        secret: impl AsRef<[u8]> + Send + 'static,
        algorithm: JWTAlgorithm,
    ) -> CryptoResult<String> {
        match algorithm {
            JWTAlgorithm::HS256 => Self::create_hs256(payload, secret).await,
            JWTAlgorithm::RS256 => Self::create_rs256(payload, secret).await,
            JWTAlgorithm::ES256 => Self::create_es256(payload, secret).await,
        }
    }

    /// Verify and decode a JWT token
    pub async fn verify<T: for<'de> Deserialize<'de>>(
        token: &str,
        secret: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<T> {
        // Split the token into parts
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(CryptoError::JWTError("Invalid JWT format".to_string()));
        }

        let header_b64 = parts[0];
        let payload_b64 = parts[1];
        let signature_b64 = parts[2];

        // Decode header to determine algorithm
        let header_json = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, header_b64)
            .map_err(|_| CryptoError::JWTError("Invalid base64 in JWT header".to_string()))?;

        let header: JWTHeader = serde_json::from_slice(&header_json)
            .map_err(|_| CryptoError::JWTError("Invalid JWT header JSON".to_string()))?;

        // Verify signature based on algorithm
        let signature_bytes = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, signature_b64)
            .map_err(|_| CryptoError::JWTError("Invalid base64 in JWT signature".to_string()))?;

        let message = format!("{}.{}", header_b64, payload_b64);

        let is_valid = match header.alg.as_str() {
            "HS256" => Self::verify_hs256(message.clone(), signature_bytes.clone(), secret).await?,
            "RS256" => Self::verify_rs256(message.clone(), signature_bytes.clone(), secret).await?,
            "ES256" => Self::verify_es256(message.clone(), signature_bytes.clone(), secret).await?,
            _ => return Err(CryptoError::unsupported_algorithm(format!("Unsupported JWT algorithm: {}", header.alg))),
        };

        if !is_valid {
            return Err(CryptoError::auth_error("Invalid JWT signature"));
        }

        // Decode payload
        let payload_json = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload_b64)
            .map_err(|_| CryptoError::JWTError("Invalid base64 in JWT payload".to_string()))?;

        let payload: T = serde_json::from_slice(&payload_json)
            .map_err(|_| CryptoError::JWTError("Invalid JWT payload JSON".to_string()))?;

        Ok(payload)
    }

    /// Create JWT with HS256
    async fn create_hs256<T: Serialize>(
        payload: &T,
        secret: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<String> {
        // Create header
        let header = JWTHeader {
            alg: "HS256".to_string(),
            typ: "JWT".to_string(),
            kid: None,
        };

        // Encode header
        let header_json = serde_json::to_string(&header)
            .map_err(|_| CryptoError::EncodingError("Failed to encode JWT header".to_string()))?;

        let header_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, header_json.as_bytes());

        // Encode payload
        let payload_json = serde_json::to_string(payload)
            .map_err(|_| CryptoError::EncodingError("Failed to encode JWT payload".to_string()))?;

        let payload_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload_json.as_bytes());

        // Create message to sign
        let message = format!("{}.{}", header_b64, payload_b64);

        // Sign the message
        let signature = tokio::task::spawn_blocking({
            let message_clone = message.clone();
            let key_vec = secret.as_ref().to_vec();
            move || {
                use hmac::{Hmac, Mac};
                use sha2::Sha256;

                let mut mac = Hmac::<Sha256>::new_from_slice(&key_vec)
                    .map_err(|_| CryptoError::KeyError("Invalid HMAC key".to_string()))?;

                mac.update(message_clone.as_bytes());
                let result = mac.finalize();
                Ok::<Vec<u8>, CryptoError>(result.into_bytes().to_vec())
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("HS256 sign task failed: {}", e)))??;

        // Encode signature
        let signature_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, signature);

        // Create final token
        Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
    }

    /// Verify JWT with HS256
    async fn verify_hs256(
        message: String,
        signature: Vec<u8>,
        secret: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<bool> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_ref())
            .map_err(|_| CryptoError::KeyError("Invalid HMAC key".to_string()))?;

        mac.update(message.as_bytes());
        Ok(mac.verify_slice(&signature).is_ok())
    }

    /// Create JWT with RS256
    async fn create_rs256<T: Serialize>(
        payload: &T,
        private_key: impl AsRef<[u8]>,
    ) -> CryptoResult<String> {
        // Create header
        let header = JWTHeader {
            alg: "RS256".to_string(),
            typ: "JWT".to_string(),
            kid: None,
        };

        // Encode header
        let header_json = serde_json::to_string(&header)
            .map_err(|_| CryptoError::EncodingError("Failed to encode JWT header".to_string()))?;

        let header_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, header_json.as_bytes());

        // Encode payload
        let payload_json = serde_json::to_string(payload)
            .map_err(|_| CryptoError::EncodingError("Failed to encode JWT payload".to_string()))?;

        let payload_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload_json.as_bytes());

        // Create message to sign
        let message = format!("{}.{}", header_b64, payload_b64);

        // Sign the message
        let signature = tokio::task::spawn_blocking({
            let message_clone = message.clone();
            let key_vec = private_key.as_ref().to_vec();
            move || {
                use rsa::pkcs8::DecodePrivateKey;
                use rsa::signature::{Signer, SignatureEncoding};

                let private_key_der = rsa::RsaPrivateKey::from_pkcs8_der(&key_vec)
                    .map_err(|e| CryptoError::KeyError(format!("Invalid RSA private key: {}", e)))?;

                let signing_key = rsa::pkcs1v15::SigningKey::<sha2::Sha256>::new_unprefixed(private_key_der);
                let signature: rsa::pkcs1v15::Signature = Signer::sign(&signing_key, message_clone.as_bytes());

                Ok::<Vec<u8>, CryptoError>(signature.to_bytes().to_vec())
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("RSA sign task failed: {}", e)))??;

        let signature_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &signature);

        // Combine parts
        Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
    }

    /// Verify JWT with RS256
    async fn verify_rs256(
        message: String,
        signature: Vec<u8>,
        public_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<bool> {
        tokio::task::spawn_blocking(move || {
            use rsa::pkcs8::DecodePublicKey;
            use rsa::signature::Verifier;

            let public_key_der = rsa::RsaPublicKey::from_public_key_der(public_key.as_ref())
                .map_err(|e| CryptoError::KeyError(format!("Invalid RSA public key: {}", e)))?;

            let verifying_key = rsa::pkcs1v15::VerifyingKey::<sha2::Sha256>::new_unprefixed(public_key_der);

            let signature = rsa::pkcs1v15::Signature::try_from(signature.as_slice())
                .map_err(|e| CryptoError::SignatureError(format!("Invalid signature format: {}", e)))?;

            match verifying_key.verify(message.as_bytes(), &signature) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("RSA verify task failed: {}", e)))?
    }

    /// Create JWT with ES256
    async fn create_es256<T: Serialize>(
        payload: &T,
        private_key: impl AsRef<[u8]>,
    ) -> CryptoResult<String> {
        // Create header
        let header = JWTHeader {
            alg: "ES256".to_string(),
            typ: "JWT".to_string(),
            kid: None,
        };

        // Encode header
        let header_json = serde_json::to_string(&header)
            .map_err(|_| CryptoError::EncodingError("Failed to encode JWT header".to_string()))?;

        let header_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, header_json.as_bytes());

        // Encode payload
        let payload_json = serde_json::to_string(payload)
            .map_err(|_| CryptoError::EncodingError("Failed to encode JWT payload".to_string()))?;

        let payload_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload_json.as_bytes());

        // Create message to sign
        let message = format!("{}.{}", header_b64, payload_b64);

        // Sign the message
        let signature = tokio::task::spawn_blocking({
            let message_clone = message.clone();
            let key_vec = private_key.as_ref().to_vec();
            move || {
                use p256::ecdsa::{SigningKey, signature::Signer};

                let signing_key = SigningKey::from_bytes(key_vec.as_slice().into())
                    .map_err(|e| CryptoError::KeyError(format!("Invalid ECDSA private key: {}", e)))?;

                let signature: p256::ecdsa::Signature = Signer::sign(&signing_key, message_clone.as_bytes());
                Ok::<Vec<u8>, CryptoError>(signature.to_bytes().to_vec())
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("ECDSA sign task failed: {}", e)))??;

        let signature_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &signature);

        // Combine parts
        Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
    }

    /// Verify JWT with ES256
    async fn verify_es256(
        message: String,
        signature: Vec<u8>,
        public_key: impl AsRef<[u8]> + Send + 'static,
    ) -> CryptoResult<bool> {
        tokio::task::spawn_blocking(move || {
            use p256::ecdsa::{VerifyingKey, Signature, signature::Verifier};

            let verifying_key = VerifyingKey::from_sec1_bytes(public_key.as_ref())
                .map_err(|e| CryptoError::KeyError(format!("Invalid ECDSA public key: {}", e)))?;

            let signature = Signature::from_bytes(signature.as_slice().into())
                .map_err(|e| CryptoError::SignatureError(format!("Invalid signature format: {}", e)))?;

            match verifying_key.verify(message.as_bytes(), &signature) {
                Ok(()) => Ok(true),
                Err(_) => Ok(false),
            }
        })
        .await
        .map_err(|e| CryptoError::SignatureError(format!("ECDSA verify task failed: {}", e)))?
    }

    /// Decode JWT without verification (for inspection)
    pub fn decode_unchecked<T: for<'de> Deserialize<'de>>(token: &str) -> CryptoResult<T> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(CryptoError::JWTError("Invalid JWT format".to_string()));
        }

        let payload_b64 = parts[1];
        let payload_json = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload_b64)
            .map_err(|_| CryptoError::JWTError("Invalid base64 in JWT payload".to_string()))?;

        let payload: T = serde_json::from_slice(&payload_json)
            .map_err(|_| CryptoError::JWTError("Invalid JWT payload JSON".to_string()))?;

        Ok(payload)
    }

    /// Get JWT header without verification
    pub fn get_header(token: &str) -> CryptoResult<JWTHeader> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(CryptoError::JWTError("Invalid JWT format".to_string()));
        }

        let header_b64 = parts[0];
        let header_json = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, header_b64)
            .map_err(|_| CryptoError::JWTError("Invalid base64 in JWT header".to_string()))?;

        let header: JWTHeader = serde_json::from_slice(&header_json)
            .map_err(|_| CryptoError::JWTError("Invalid JWT header JSON".to_string()))?;

        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::{CryptoKey, KeyType};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestClaims {
        sub: String,
        exp: u64,
        iat: u64,
    }

    #[tokio::test]
    async fn test_hs256_jwt_create_verify() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1000000000,
        };

        let secret = b"my-secret-key-that-is-long-enough";

        let token = CryptoJWT::create(&claims, secret, JWTAlgorithm::HS256).await.unwrap();
        let decoded: TestClaims = CryptoJWT::verify(&token, secret).await.unwrap();

        assert_eq!(claims, decoded);
    }

    #[tokio::test]
    async fn test_rs256_jwt_create_verify() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1000000000,
        };

        let keypair = CryptoKey::generate_keypair(KeyType::Rsa { bits: 2048 }).await.unwrap();

        let token = CryptoJWT::create(&claims, keypair.private_key.clone(), JWTAlgorithm::RS256).await.unwrap();
        let decoded: TestClaims = CryptoJWT::verify(&token, keypair.public_key.clone()).await.unwrap();

        assert_eq!(claims, decoded);
    }

    #[tokio::test]
    async fn test_es256_jwt_create_verify() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1000000000,
        };

        let keypair = CryptoKey::generate_keypair(KeyType::Ecdsa { curve: crate::key::EcdsaCurve::P256 }).await.unwrap();

        let token = CryptoJWT::create(&claims, keypair.private_key.clone(), JWTAlgorithm::ES256).await.unwrap();
        let decoded: TestClaims = CryptoJWT::verify(&token, keypair.public_key.clone()).await.unwrap();

        assert_eq!(claims, decoded);
    }

    #[tokio::test]
    async fn test_invalid_signature() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1000000000,
        };

        let secret = b"my-secret-key";
        let wrong_secret = b"wrong-secret";

        let token = CryptoJWT::create(&claims, secret, JWTAlgorithm::HS256).await.unwrap();
        let result: CryptoResult<TestClaims> = CryptoJWT::verify(&token, wrong_secret).await;

        assert!(matches!(result, Err(CryptoError::AuthenticationError(_))));
    }

    #[tokio::test]
    async fn test_invalid_jwt_format() {
        let result: CryptoResult<TestClaims> = CryptoJWT::verify("invalid.jwt.token", b"secret").await;
        assert!(matches!(result, Err(CryptoError::JWTError(_))));
    }

    #[tokio::test]
    async fn test_decode_unchecked() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1000000000,
        };

        let secret = b"my-secret-key-that-is-long-enough";
        let token = CryptoJWT::create(&claims, secret, JWTAlgorithm::HS256).await.unwrap();

        let decoded = CryptoJWT::decode_unchecked::<TestClaims>(&token).unwrap();
        assert_eq!(claims, decoded);
    }

    #[tokio::test]
    async fn test_get_header() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1000000000,
        };

        let secret = b"my-secret-key-that-is-long-enough";
        let token = CryptoJWT::create(&claims, secret, JWTAlgorithm::HS256).await.unwrap();

        let header = CryptoJWT::get_header(&token).unwrap();
        assert_eq!(header.alg, "HS256");
        assert_eq!(header.typ, "JWT");
    }

    #[tokio::test]
    async fn test_malformed_payload() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid_base64. signature";
        let result: CryptoResult<TestClaims> = CryptoJWT::verify(token, b"secret").await;
        assert!(matches!(result, Err(CryptoError::JWTError(_))));
    }

    #[tokio::test]
    async fn test_unsupported_algorithm() {
        let token = "eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9.payload.c2lnbmF0dXJl";
        let result: CryptoResult<TestClaims> = CryptoJWT::verify(token, b"secret").await;
        assert!(matches!(result, Err(CryptoError::UnsupportedAlgorithm(_))));
    }
}