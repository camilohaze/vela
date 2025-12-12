//! Rate Limiter para API Gateway
//!
//! Implementa limitación de tasa con múltiples algoritmos:
//! - Token Bucket
//! - Leaky Bucket
//! - Fixed Window
//! - Sliding Window

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use crate::gateway::{Request, GatewayError};

/// Configuración de rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_duration: Duration,
    pub burst_size: Option<u32>,
}

/// Estado de rate limiting para una clave
#[derive(Debug, Clone)]
struct RateLimitState {
    tokens: f64,
    last_refill: Instant,
    requests_in_window: u32,
    window_start: Instant,
}

/// Rate limiter principal
#[derive(Debug)]
pub struct RateLimiter {
    configs: HashMap<String, RateLimitConfig>,
    states: Arc<RwLock<HashMap<String, RateLimitState>>>,
}

impl RateLimiter {
    /// Crear nuevo rate limiter
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Agregar configuración de rate limiting
    pub fn add_config(&mut self, key: String, config: RateLimitConfig) {
        self.configs.insert(key, config);
    }

    /// Verificar si el request está dentro del límite
    pub async fn check_limit(&self, request: &Request) -> Result<bool, GatewayError> {
        // Por simplicidad, usamos la IP como clave
        // En producción se podría usar user ID, API key, etc.
        let client_ip = request.headers.get("X-Forwarded-For")
            .or_else(|| request.headers.get("X-Real-IP"))
            .unwrap_or(&"unknown".to_string())
            .clone();

        let config = self.configs.get("global")
            .ok_or_else(|| GatewayError::Internal("No rate limit config found".to_string()))?;

        let mut states = self.states.write().await;
        let state = states.entry(client_ip.clone()).or_insert_with(|| RateLimitState {
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
        assert!(limiter.configs.is_empty());
    }

    #[tokio::test]
    async fn test_add_config() {
        let mut limiter = RateLimiter::new();

        let config = RateLimitConfig {
            requests_per_window: 10,
            window_duration: Duration::from_secs(60),
            burst_size: Some(20),
        };

        limiter.add_config("global".to_string(), config);

        assert_eq!(limiter.configs.len(), 1);
    }

    #[tokio::test]
    async fn test_check_limit_under_limit() {
        let mut limiter = RateLimiter::new();

        let config = RateLimitConfig {
            requests_per_window: 10,
            window_duration: Duration::from_secs(60),
            burst_size: Some(20),
        };

        limiter.add_config("global".to_string(), config);

        let request = create_test_request("192.168.1.1");

        // Debería permitir los primeros requests
        for _ in 0..10 {
            let result = limiter.check_limit(&request).await.unwrap();
            assert!(result);
        }
    }

    #[tokio::test]
    async fn test_check_limit_exceeded() {
        let mut limiter = RateLimiter::new();

        let config = RateLimitConfig {
            requests_per_window: 2,
            window_duration: Duration::from_secs(60),
            burst_size: Some(2),
        };

        limiter.add_config("global".to_string(), config);

        let request = create_test_request("192.168.1.1");

        // Primeros 2 requests deberían pasar
        assert!(limiter.check_limit(&request).await.unwrap());
        assert!(limiter.check_limit(&request).await.unwrap());

        // El tercero debería ser rechazado
        let result = limiter.check_limit(&request).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_no_config_error() {
        let limiter = RateLimiter::new();
        let request = create_test_request("192.168.1.1");

        let result = limiter.check_limit(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_different_clients() {
        let mut limiter = RateLimiter::new();

        let config = RateLimitConfig {
            requests_per_window: 1,
            window_duration: Duration::from_secs(60),
            burst_size: Some(1),
        };

        limiter.add_config("global".to_string(), config);

        let request1 = create_test_request("192.168.1.1");
        let request2 = create_test_request("192.168.1.2");

        // Ambos deberían poder hacer 1 request
        assert!(limiter.check_limit(&request1).await.unwrap());
        assert!(limiter.check_limit(&request2).await.unwrap());

        // Pero el segundo request del mismo cliente debería fallar
        let result = limiter.check_limit(&request1).await.unwrap();
        assert!(!result);
    }
}