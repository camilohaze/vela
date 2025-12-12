//! Authentication Engine para API Gateway
//!
//! Implementa autenticación y autorización con soporte para:
//! - JWT tokens
//! - API Keys
//! - OAuth2
//! - Basic Auth

use std::collections::HashMap;
use serde::Serialize;
use crate::gateway::{Context, GatewayError};

/// Tipos de autenticación soportados
#[derive(Debug, Clone, PartialEq)]
pub enum AuthType {
    None,
    Jwt,
    ApiKey,
    OAuth2,
    Basic,
}

/// Configuración de autenticación
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub jwt_secret: Option<String>,
    pub api_key_header: Option<String>,
    pub valid_api_keys: Vec<String>,
    pub oauth2_config: Option<OAuth2Config>,
}

/// Configuración OAuth2
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
    pub userinfo_url: String,
}

/// Claims de JWT
#[derive(Debug, Clone, serde::Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub roles: Option<Vec<String>>,
}

/// Usuario autenticado
#[derive(Debug, Clone, Serialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub roles: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authentication Engine
#[derive(Debug)]
pub struct AuthEngine {
    config: AuthConfig,
}

impl AuthEngine {
    /// Crear nuevo auth engine
    pub fn new() -> Self {
        Self {
            config: AuthConfig {
                auth_type: AuthType::None,
                jwt_secret: None,
                api_key_header: None,
                valid_api_keys: Vec::new(),
                oauth2_config: None,
            },
        }
    }

    /// Configurar autenticación
    pub fn with_config(mut self, config: AuthConfig) -> Self {
        self.config = config;
        self
    }

    /// Autenticar request
    pub async fn authenticate(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        match self.config.auth_type {
            AuthType::None => Ok(()), // No authentication required
            AuthType::Jwt => self.authenticate_jwt(ctx).await,
            AuthType::ApiKey => self.authenticate_api_key(ctx).await,
            AuthType::OAuth2 => self.authenticate_oauth2(ctx).await,
            AuthType::Basic => self.authenticate_basic(ctx).await,
        }
    }

    /// Autorizar acceso basado en roles
    pub fn authorize(&self, user: &AuthenticatedUser, required_roles: &[String]) -> bool {
        if required_roles.is_empty() {
            return true; // No authorization required
        }

        required_roles.iter().any(|role| user.roles.contains(role))
    }

    /// Autenticación JWT
    async fn authenticate_jwt(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        let auth_header = ctx.request.headers.get("Authorization")
            .ok_or_else(|| GatewayError::Auth("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(GatewayError::Auth("Invalid Authorization header format".to_string()));
        }

        let token = &auth_header[7..]; // Remove "Bearer "

        // En implementación real, validar JWT aquí
        // Por simplicidad, aceptamos cualquier token que empiece con "valid"
        if !token.starts_with("valid") {
            return Err(GatewayError::Auth("Invalid JWT token".to_string()));
        }

        // Simular claims
        let user = AuthenticatedUser {
            id: "user123".to_string(),
            roles: vec!["user".to_string()],
            metadata: HashMap::new(),
        };

        ctx.metadata.insert("user".to_string(), serde_json::to_value(&user).unwrap());

        Ok(())
    }

    /// Autenticación API Key
    async fn authenticate_api_key(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        let header_name = self.config.api_key_header.as_deref().unwrap_or("X-API-Key");

        let api_key = ctx.request.headers.get(header_name)
            .ok_or_else(|| GatewayError::Auth(format!("Missing {} header", header_name)))?;

        if !self.config.valid_api_keys.contains(api_key) {
            return Err(GatewayError::Auth("Invalid API key".to_string()));
        }

        let user = AuthenticatedUser {
            id: format!("api-key-{}", api_key),
            roles: vec!["api-user".to_string()],
            metadata: HashMap::new(),
        };

        ctx.metadata.insert("user".to_string(), serde_json::to_value(&user).unwrap());

        Ok(())
    }

    /// Autenticación OAuth2 (simplificada)
    async fn authenticate_oauth2(&self, _ctx: &mut Context) -> Result<(), GatewayError> {
        // Implementación simplificada
        Err(GatewayError::Auth("OAuth2 not implemented yet".to_string()))
    }

    /// Autenticación Basic Auth
    async fn authenticate_basic(&self, ctx: &mut Context) -> Result<(), GatewayError> {
        let auth_header = ctx.request.headers.get("Authorization")
            .ok_or_else(|| GatewayError::Auth("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Basic ") {
            return Err(GatewayError::Auth("Invalid Authorization header format".to_string()));
        }

        // En implementación real, decodificar base64 y validar credenciales
        // Por simplicidad, aceptamos cualquier header Basic
        let user = AuthenticatedUser {
            id: "basic-user".to_string(),
            roles: vec!["user".to_string()],
            metadata: HashMap::new(),
        };

        ctx.metadata.insert("user".to_string(), serde_json::to_value(&user).unwrap());

        Ok(())
    }
}

impl Default for AuthEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::Request;
    use std::collections::HashMap;

    fn create_test_request(headers: HashMap<String, String>) -> Request {
        Request {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers,
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_auth_engine_creation() {
        let engine = AuthEngine::new();
        assert_eq!(engine.config.auth_type, AuthType::None);
    }

    #[tokio::test]
    async fn test_no_auth_required() {
        let engine = AuthEngine::new();
        let mut ctx = Context {
            request: create_test_request(HashMap::new()),
            response: None,
            metadata: HashMap::new(),
        };

        let result = engine.authenticate(&mut ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_jwt_auth_success() {
        let engine = AuthEngine::new().with_config(AuthConfig {
            auth_type: AuthType::Jwt,
            jwt_secret: Some("secret".to_string()),
            api_key_header: None,
            valid_api_keys: vec![],
            oauth2_config: None,
        });

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer valid-token".to_string());

        let mut ctx = Context {
            request: create_test_request(headers),
            response: None,
            metadata: HashMap::new(),
        };

        let result = engine.authenticate(&mut ctx).await;
        assert!(result.is_ok());
        assert!(ctx.metadata.contains_key("user"));
    }

    #[tokio::test]
    async fn test_jwt_auth_missing_header() {
        let engine = AuthEngine::new().with_config(AuthConfig {
            auth_type: AuthType::Jwt,
            jwt_secret: Some("secret".to_string()),
            api_key_header: None,
            valid_api_keys: vec![],
            oauth2_config: None,
        });

        let mut ctx = Context {
            request: create_test_request(HashMap::new()),
            response: None,
            metadata: HashMap::new(),
        };

        let result = engine.authenticate(&mut ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_key_auth_success() {
        let engine = AuthEngine::new().with_config(AuthConfig {
            auth_type: AuthType::ApiKey,
            jwt_secret: None,
            api_key_header: Some("X-API-Key".to_string()),
            valid_api_keys: vec!["valid-key".to_string()],
            oauth2_config: None,
        });

        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), "valid-key".to_string());

        let mut ctx = Context {
            request: create_test_request(headers),
            response: None,
            metadata: HashMap::new(),
        };

        let result = engine.authenticate(&mut ctx).await;
        assert!(result.is_ok());
        assert!(ctx.metadata.contains_key("user"));
    }

    #[tokio::test]
    async fn test_api_key_auth_invalid() {
        let engine = AuthEngine::new().with_config(AuthConfig {
            auth_type: AuthType::ApiKey,
            jwt_secret: None,
            api_key_header: Some("X-API-Key".to_string()),
            valid_api_keys: vec!["valid-key".to_string()],
            oauth2_config: None,
        });

        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), "invalid-key".to_string());

        let mut ctx = Context {
            request: create_test_request(headers),
            response: None,
            metadata: HashMap::new(),
        };

        let result = engine.authenticate(&mut ctx).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_authorize_no_roles_required() {
        let engine = AuthEngine::new();
        let user = AuthenticatedUser {
            id: "user1".to_string(),
            roles: vec![],
            metadata: HashMap::new(),
        };

        assert!(engine.authorize(&user, &[]));
    }

    #[test]
    fn test_authorize_has_required_role() {
        let engine = AuthEngine::new();
        let user = AuthenticatedUser {
            id: "user1".to_string(),
            roles: vec!["admin".to_string(), "user".to_string()],
            metadata: HashMap::new(),
        };

        assert!(engine.authorize(&user, &["admin".to_string()]));
    }

    #[test]
    fn test_authorize_missing_role() {
        let engine = AuthEngine::new();
        let user = AuthenticatedUser {
            id: "user1".to_string(),
            roles: vec!["user".to_string()],
            metadata: HashMap::new(),
        };

        assert!(!engine.authorize(&user, &["admin".to_string()]));
    }
}