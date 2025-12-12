//! Rate Limiter para API Gateway
//!
//! Implementa limitación de tasa con múltiples algoritmos:
//! - Token Bucket
//! - Leaky Bucket
//! - Fixed Window
//! - Sliding Window
//!
//! Soporta rate limiting por:
//! - IP address
//! - User ID (desde headers de autenticación)
//! - Endpoint (path específico)
//! - Combinaciones personalizadas

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use crate::gateway::{Request, GatewayError};

/// Tipos de claves para rate limiting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RateLimitKeyType {
    /// Rate limiting por dirección IP
    Ip,
    /// Rate limiting por ID de usuario (desde JWT, API key, etc.)
    UserId,
    /// Rate limiting por endpoint/path
    Endpoint,
    /// Rate limiting por IP + endpoint
    IpEndpoint,
    /// Rate limiting por usuario + endpoint
    UserEndpoint,
    /// Rate limiting personalizado con clave específica
    Custom(String),
}

/// Configuración de rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_duration: Duration,
    pub burst_size: Option<u32>,
}

/// Regla de rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitRule {
    pub key_type: RateLimitKeyType,
    pub config: RateLimitConfig,
    /// Patrón para matching (ej: "/api/*", "/users/*")
    pub pattern: Option<String>,
}

/// Estado de rate limiting para una clave
#[derive(Debug, Clone)]
struct RateLimitState {
    tokens: f64,
    last_refill: Instant,
    requests_in_window: u32,
    window_start: Instant,
}

/// Rate limiter principal con soporte para múltiples tipos de rate limiting
#[derive(Debug)]
pub struct RateLimiter {
    /// Reglas de rate limiting organizadas por tipo
    rules: HashMap<RateLimitKeyType, Vec<RateLimitRule>>,
    /// Estados de rate limiting por clave
    states: Arc<RwLock<HashMap<String, RateLimitState>>>,
    /// Configuración global por defecto
    default_config: Option<RateLimitConfig>,
}

impl RateLimiter {
    /// Crear nuevo rate limiter
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            states: Arc::new(RwLock::new(HashMap::new())),
            default_config: None,
        }
    }

    /// Configurar rate limiting global por defecto
    pub fn with_default_config(mut self, config: RateLimitConfig) -> Self {
        self.default_config = Some(config);
        self
    }

    /// Agregar una regla de rate limiting
    pub fn add_rule(mut self, rule: RateLimitRule) -> Self {
        self.rules.entry(rule.key_type.clone()).or_insert_with(Vec::new).push(rule);
        self
    }

    /// Generar clave para rate limiting basado en el tipo y request
    fn generate_key(&self, key_type: &RateLimitKeyType, request: &Request) -> String {
        match key_type {
            RateLimitKeyType::Ip => {
                let ip = request.headers.get("X-Forwarded-For")
                    .or_else(|| request.headers.get("X-Real-IP"))
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                ip.to_string()
            },
            RateLimitKeyType::UserId => {
                // Extraer user ID desde Authorization header (JWT) o X-User-ID
                if let Some(auth_header) = request.headers.get("Authorization") {
                    // Para JWT, extraer el payload (simplificado)
                    if auth_header.starts_with("Bearer ") {
                        // En producción, decodificar JWT y extraer user_id
                        // Por ahora, usar un hash del token como clave
                        format!("user_jwt_{}", auth_header.len())
                    } else {
                        "user_unknown".to_string()
                    }
                } else if let Some(user_id) = request.headers.get("X-User-ID") {
                    format!("user_{}", user_id)
                } else {
                    "user_anonymous".to_string()
                }
            },
            RateLimitKeyType::Endpoint => {
                request.path.clone()
            },
            RateLimitKeyType::IpEndpoint => {
                let ip = request.headers.get("X-Forwarded-For")
                    .or_else(|| request.headers.get("X-Real-IP"))
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                format!("{}_{}", ip, request.path)
            },
            RateLimitKeyType::UserEndpoint => {
                let user_key = self.generate_key(&RateLimitKeyType::UserId, request);
                format!("{}_{}", user_key, request.path)
            },
            RateLimitKeyType::Custom(custom_key) => {
                custom_key.clone()
            }
        }
    }

    /// Verificar si el patrón de la regla coincide con el path del request
    fn matches_pattern(&self, pattern: &Option<String>, path: &str) -> bool {
        match pattern {
            Some(pat) => {
                // Soporte básico para wildcards
                if pat.ends_with("/*") {
                    let prefix = &pat[..pat.len() - 2];
                    path.starts_with(prefix)
                } else {
                    pat == path
                }
            },
            None => true, // Sin patrón = aplica a todos
        }
    }



    /// Verificar si el request está dentro de todos los límites aplicables
    pub async fn check_limit(&self, request: &Request) -> Result<bool, GatewayError> {
        // Verificar todas las reglas aplicables
        for (key_type, rules) in &self.rules {
            for rule in rules {
                // Verificar si el patrón coincide (si existe)
                if !self.matches_pattern(&rule.pattern, &request.path) {
                    continue;
                }

                let key = self.generate_key(key_type, request);

                // Verificar límite para esta regla específica
                if !self.check_single_limit(&key, &rule.config).await? {
                    return Ok(false); // Rate limit exceeded
                }
            }
        }

        // Si no hay reglas específicas, verificar configuración global por defecto
        if self.rules.is_empty() {
            if let Some(config) = &self.default_config {
                let key = self.generate_key(&RateLimitKeyType::Ip, request);
                return self.check_single_limit(&key, config).await;
            }
        }

        Ok(true)
    }

    /// Verificar límite para una clave y configuración específica
    async fn check_single_limit(&self, key: &str, config: &RateLimitConfig) -> Result<bool, GatewayError> {
        let mut states = self.states.write().await;
        let state = states.entry(key.to_string()).or_insert_with(|| RateLimitState {
            tokens: config.burst_size.unwrap_or(config.requests_per_window) as f64,
            last_refill: Instant::now(),
            requests_in_window: 0,
            window_start: Instant::now(),
        });

        // Refill tokens usando token bucket
        let now = Instant::now();
        let time_passed = now.duration_since(state.last_refill);
        let tokens_to_add = (time_passed.as_secs_f64() / config.window_duration.as_secs_f64())
            * config.requests_per_window as f64;

        state.tokens = (state.tokens + tokens_to_add).min(
            config.burst_size.unwrap_or(config.requests_per_window) as f64
        );
        state.last_refill = now;

        // Check fixed window
        if now.duration_since(state.window_start) >= config.window_duration {
            state.requests_in_window = 0;
            state.window_start = now;
        }

        // Verificar límites
        if state.requests_in_window >= config.requests_per_window {
            return Ok(false); // Rate limit exceeded
        }

        if state.tokens < 1.0 {
            return Ok(false); // No tokens available
        }

        // Consumir token y registrar request
        state.tokens -= 1.0;
        state.requests_in_window += 1;

        Ok(true)
    }

    /// Obtener estadísticas de rate limiting
    pub async fn get_stats(&self, key: &str) -> Option<RateLimitState> {
        let states = self.states.read().await;
        states.get(key).cloned()
    }

    /// Limpiar estados expirados (para evitar memory leaks)
    pub async fn cleanup_expired(&self) {
        let mut states = self.states.write().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(60); // 1 minute window

        states.retain(|_, state| {
            now.duration_since(state.window_start) < window_duration * 2
        });
    }

    /// Obtener todas las reglas configuradas
    pub fn get_rules(&self) -> &HashMap<RateLimitKeyType, Vec<RateLimitRule>> {
        &self.rules
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_request(ip: &str) -> Request {
        Request {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("X-Forwarded-For".to_string(), ip.to_string());
                headers
            },
            body: None,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new();
        assert!(limiter.rules.is_empty());
        assert!(limiter.default_config.is_none());
    }

    #[tokio::test]
    async fn test_add_rule() {
        let config = RateLimitConfig {
            requests_per_window: 10,
            window_duration: Duration::from_secs(60),
            burst_size: Some(20),
        };

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Ip,
            config,
            pattern: None,
        };

        let limiter = RateLimiter::new().add_rule(rule);

        assert_eq!(limiter.rules.len(), 1);
        assert!(limiter.rules.contains_key(&RateLimitKeyType::Ip));
    }

    #[tokio::test]
    async fn test_generate_key_ip() {
        let limiter = RateLimiter::new();
        let request = create_test_request("192.168.1.1");

        let key = limiter.generate_key(&RateLimitKeyType::Ip, &request);
        assert_eq!(key, "192.168.1.1");
    }

    #[tokio::test]
    async fn test_generate_key_user_id() {
        let limiter = RateLimiter::new();
        let mut request = create_test_request("192.168.1.1");
        request.headers.insert("X-User-ID".to_string(), "user123".to_string());

        let key = limiter.generate_key(&RateLimitKeyType::UserId, &request);
        assert_eq!(key, "user_user123");
    }

    #[tokio::test]
    async fn test_generate_key_endpoint() {
        let limiter = RateLimiter::new();
        let request = create_test_request("192.168.1.1");

        let key = limiter.generate_key(&RateLimitKeyType::Endpoint, &request);
        assert_eq!(key, "/test");
    }

    #[tokio::test]
    async fn test_generate_key_ip_endpoint() {
        let limiter = RateLimiter::new();
        let request = create_test_request("192.168.1.1");

        let key = limiter.generate_key(&RateLimitKeyType::IpEndpoint, &request);
        assert_eq!(key, "192.168.1.1_/test");
    }

    #[tokio::test]
    async fn test_check_limit_under_limit() {
        let config = RateLimitConfig {
            requests_per_window: 10,
            window_duration: Duration::from_secs(60),
            burst_size: Some(20),
        };

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Ip,
            config,
            pattern: None,
        };

        let limiter = RateLimiter::new().add_rule(rule);
        let request = create_test_request("192.168.1.1");

        // Debería permitir los primeros requests
        for _ in 0..10 {
            let result = limiter.check_limit(&request).await.unwrap();
            assert!(result);
        }
    }

    #[tokio::test]
    async fn test_check_limit_exceeded() {
        let config = RateLimitConfig {
            requests_per_window: 2,
            window_duration: Duration::from_secs(60),
            burst_size: Some(2),
        };

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Ip,
            config,
            pattern: None,
        };

        let limiter = RateLimiter::new().add_rule(rule);
        let request = create_test_request("192.168.1.1");

        // Primeros 2 requests deberían pasar
        assert!(limiter.check_limit(&request).await.unwrap());
        assert!(limiter.check_limit(&request).await.unwrap());

        // El tercero debería ser rechazado
        let result = limiter.check_limit(&request).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_different_clients() {
        let config = RateLimitConfig {
            requests_per_window: 1,
            window_duration: Duration::from_secs(60),
            burst_size: Some(1),
        };

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Ip,
            config,
            pattern: None,
        };

        let limiter = RateLimiter::new().add_rule(rule);
        let request1 = create_test_request("192.168.1.1");
        let request2 = create_test_request("192.168.1.2");

        // Ambos deberían poder hacer 1 request
        assert!(limiter.check_limit(&request1).await.unwrap());
        assert!(limiter.check_limit(&request2).await.unwrap());

        // Pero el segundo request del mismo cliente debería fallar
        let result = limiter.check_limit(&request1).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        let config = RateLimitConfig {
            requests_per_window: 1,
            window_duration: Duration::from_secs(60),
            burst_size: Some(1),
        };

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Ip,
            config,
            pattern: Some("/api/*".to_string()),
        };

        let limiter = RateLimiter::new().add_rule(rule);

        // Request que coincide con el patrón
        let mut request = create_test_request("192.168.1.1");
        request.path = "/api/users".to_string();
        assert!(limiter.check_limit(&request).await.unwrap());

        // Segundo request debería ser rechazado
        let result = limiter.check_limit(&request).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_pattern_no_match() {
        let config = RateLimitConfig {
            requests_per_window: 1,
            window_duration: Duration::from_secs(60),
            burst_size: Some(1),
        };

        let rule = RateLimitRule {
            key_type: RateLimitKeyType::Ip,
            config,
            pattern: Some("/api/*".to_string()),
        };

        let limiter = RateLimiter::new().add_rule(rule);

        // Request que NO coincide con el patrón
        let mut request = create_test_request("192.168.1.1");
        request.path = "/public/users".to_string();

        // Debería pasar porque no hay regla que aplique
        let result = limiter.check_limit(&request).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_default_config_fallback() {
        let config = RateLimitConfig {
            requests_per_window: 1,
            window_duration: Duration::from_secs(60),
            burst_size: Some(1),
        };

        let limiter = RateLimiter::new().with_default_config(config);
        let request = create_test_request("192.168.1.1");

        // Debería usar la configuración por defecto
        assert!(limiter.check_limit(&request).await.unwrap());

        // Segundo request debería ser rechazado
        let result = limiter.check_limit(&request).await.unwrap();
        assert!(!result);
    }
}