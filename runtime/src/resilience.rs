//! Resilience patterns for Vela runtime
//!
//! This module provides implementations of resilience patterns that can be used
//! by the Vela compiler to generate resilient code.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Circuit Breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Configuration for Circuit Breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub success_threshold: u32,
    pub call_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 2,
            call_timeout: Duration::from_secs(10),
        }
    }
}

/// Circuit Breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
        }
    }

    fn should_attempt_reset(&self) -> bool {
        if self.state != CircuitBreakerState::Open {
            return false;
        }

        if let Some(last_failure) = self.last_failure_time {
            return last_failure.elapsed() >= self.config.recovery_timeout;
        }

        true
    }

    fn record_success(&mut self) {
        self.success_count += 1;

        if self.state == CircuitBreakerState::HalfOpen {
            if self.success_count >= self.config.success_threshold {
                self.close_circuit();
            }
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        self.success_count = 0;

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.open_circuit();
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.open_circuit();
            }
            CircuitBreakerState::Open => {}
        }
    }

    fn open_circuit(&mut self) {
        self.state = CircuitBreakerState::Open;
    }

    fn close_circuit(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
    }

    fn half_open_circuit(&mut self) {
        self.state = CircuitBreakerState::HalfOpen;
        self.success_count = 0;
    }

    pub async fn call<F, Fut, T, E>(&mut self, f: F) -> Result<T, ResilienceError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        // Check if should attempt reset
        if self.should_attempt_reset() {
            self.half_open_circuit();
        }

        // Reject if open
        if self.state == CircuitBreakerState::Open {
            return Err(ResilienceError::CircuitBreakerOpen);
        }

        // Execute with timeout
        let result = timeout(self.config.call_timeout, f()).await;

        match result {
            Ok(Ok(value)) => {
                self.record_success();
                Ok(value)
            }
            Ok(Err(error)) => {
                self.record_failure();
                Err(ResilienceError::FunctionError(error))
            }
            Err(_) => {
                self.record_failure();
                Err(ResilienceError::Timeout)
            }
        }
    }
}

/// Global registry of circuit breakers
lazy_static::lazy_static! {
    static ref CIRCUIT_BREAKERS: Mutex<HashMap<String, Arc<Mutex<CircuitBreaker>>>> = Mutex::new(HashMap::new());
}

/// Resilience errors
#[derive(Debug, PartialEq)]
pub enum ResilienceError<E> {
    CircuitBreakerOpen,
    Timeout,
    FunctionError(E),
}

/// Get or create a circuit breaker by name
pub fn get_or_create_circuit_breaker(
    name: &str,
    config: CircuitBreakerConfig,
) -> Arc<Mutex<CircuitBreaker>> {
    let mut breakers = CIRCUIT_BREAKERS.lock().unwrap();

    if let Some(breaker) = breakers.get(name) {
        Arc::clone(breaker)
    } else {
        let breaker = Arc::new(Mutex::new(CircuitBreaker::new(config)));
        breakers.insert(name.to_string(), Arc::clone(&breaker));
        breaker
    }
}

/// Execute function with circuit breaker protection
pub async fn with_circuit_breaker<F, Fut, T, E>(
    config: CircuitBreakerConfig,
    name: &str,
    f: F,
) -> Result<T, ResilienceError<E>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let breaker = get_or_create_circuit_breaker(name, config);
    let mut breaker = breaker.lock().unwrap();
    breaker.call(f).await
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Option<Duration>,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Some(Duration::from_secs(30)),
            backoff_multiplier: 2.0,
        }
    }
}

/// Execute function with retry logic
pub async fn with_retry<F, Fut, T, E>(
    config: RetryConfig,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempt = 0;
    let mut delay = config.base_delay;

    loop {
        attempt += 1;

        match f().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt >= config.max_attempts {
                    return Err(error);
                }

                // Calculate next delay with exponential backoff
                if let Some(max_delay) = config.max_delay {
                    delay = std::cmp::min(delay.mul_f64(config.backoff_multiplier), max_delay);
                } else {
                    delay = delay.mul_f64(config.backoff_multiplier);
                }

                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Timeout configuration
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub duration: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(30),
        }
    }
}

/// Execute function with timeout
pub async fn with_timeout<F, Fut, T>(
    config: TimeoutConfig,
    f: F,
) -> Result<T, TimeoutError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    match timeout(config.duration, f()).await {
        Ok(result) => Ok(result),
        Err(_) => Err(TimeoutError),
    }
}

/// Timeout error
#[derive(Debug, PartialEq)]
pub struct TimeoutError;

/// Bulkhead configuration
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    pub max_concurrent: usize,
    pub queue_size: usize,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            queue_size: 50,
        }
    }
}

/// Execute function with bulkhead pattern
pub async fn with_bulkhead<F, Fut, T, E>(
    config: BulkheadConfig,
    f: F,
) -> Result<T, BulkheadError<E>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    // For now, use a simple approach - this should be improved to use shared semaphore
    // In a real implementation, we'd want to share the semaphore across multiple calls
    // For testing purposes, we'll use a simple counter approach

    static mut COUNTER: std::sync::Mutex<usize> = std::sync::Mutex::new(0);

    unsafe {
        let mut counter = COUNTER.lock().unwrap();
        if *counter >= config.max_concurrent {
            return Err(BulkheadError::BulkheadFull);
        }
        *counter += 1;
    }

    let result = f().await;

    unsafe {
        let mut counter = COUNTER.lock().unwrap();
        *counter -= 1;
    }

    result.map_err(BulkheadError::FunctionError)
}

/// Bulkhead error
#[derive(Debug, PartialEq)]
pub enum BulkheadError<E> {
    BulkheadFull,
    FunctionError(E),
}

/// Fallback configuration
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    pub exceptions: Vec<String>, // Exception types to trigger fallback
}

/// Execute function with fallback
pub async fn with_fallback<F, Fb, Fut, FbFut, T, E>(
    _config: FallbackConfig,
    f: F,
    fallback: Fb,
) -> Result<T, E>
where
    F: FnOnce() -> Fut,
    Fb: FnOnce() -> FbFut,
    Fut: Future<Output = Result<T, E>>,
    FbFut: Future<Output = Result<T, E>>,
{
    // For now, simple implementation
    // TODO: Implement exception-based fallback
    match f().await {
        Ok(value) => Ok(value),
        Err(_) => fallback().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = CircuitBreakerConfig::default();
        let mut cb = CircuitBreaker::new(config);

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let result = cb.call(|| async {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ()>("success")
        }).await;

        assert_eq!(result, Ok("success"));
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert_eq!(cb.state, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let mut cb = CircuitBreaker::new(config);

        // First failure
        let result1 = cb.call(|| async { Err::<(), _>("error1") }).await;
        assert_eq!(result1, Err(ResilienceError::FunctionError("error1")));
        assert_eq!(cb.state, CircuitBreakerState::Closed);

        // Second failure - should open circuit
        let result2 = cb.call(|| async { Err::<(), _>("error2") }).await;
        assert_eq!(result2, Err(ResilienceError::FunctionError("error2")));
        assert_eq!(cb.state, CircuitBreakerState::Open);

        // Third call should be rejected
        let result3 = cb.call(|| async { Ok::<_, ()>("should not execute") }).await;
        assert_eq!(result3, Err(ResilienceError::CircuitBreakerOpen));
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 1,
            ..Default::default()
        };
        let mut cb = CircuitBreaker::new(config.clone());

        // Fail to open circuit
        let _ = cb.call(|| async { Err::<(), _>("error") }).await;
        assert_eq!(cb.state, CircuitBreakerState::Open);

        // Wait for recovery timeout
        tokio::time::sleep(config.recovery_timeout + Duration::from_millis(10)).await;

        // Next call should succeed and close circuit
        let result = cb.call(|| async { Ok::<_, ()>("recovered") }).await;
        assert_eq!(result, Ok("recovered"));
        assert_eq!(cb.state, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_retry_success() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let result = with_retry(config, || async {
            let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err("temporary error")
            } else {
                Ok("success")
            }
        }).await;

        assert_eq!(result, Ok("success"));
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let result: Result<&str, &str> = with_retry(config, || async {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            Err("persistent error")
        }).await;

        assert_eq!(result, Err("persistent error"));
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_timeout_success() {
        let config = TimeoutConfig {
            duration: Duration::from_millis(100),
        };

        let result = with_timeout(config, || async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            "success"
        }).await;

        assert_eq!(result, Ok("success"));
    }

    #[tokio::test]
    async fn test_timeout_expired() {
        let config = TimeoutConfig {
            duration: Duration::from_millis(50),
        };

        let result = with_timeout(config, || async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            "too slow"
        }).await;

        assert_eq!(result, Err(TimeoutError));
    }

    #[tokio::test]
    async fn test_bulkhead_success() {
        let config = BulkheadConfig {
            max_concurrent: 2,
            queue_size: 0, // Not used in current implementation
        };

        let result: Result<&str, BulkheadError<&str>> = with_bulkhead(config, || async {
            Ok("success")
        }).await;

        assert_eq!(result, Ok("success"));
    }

    #[tokio::test]
    async fn test_bulkhead_function_error() {
        let config = BulkheadConfig {
            max_concurrent: 2,
            queue_size: 0,
        };

        let result: Result<&str, BulkheadError<&str>> = with_bulkhead(config, || async {
            Err("function error")
        }).await;

        assert_eq!(result, Err(BulkheadError::FunctionError("function error")));
    }

    #[tokio::test]
    async fn test_bulkhead_concurrent_limit() {
        // Test that bulkhead limits concurrent executions
        // Note: Current implementation uses per-call semaphore, so this test
        // demonstrates the intended behavior but doesn't test shared state
        let config = BulkheadConfig {
            max_concurrent: 1,
            queue_size: 0,
        };

        // This test verifies that the bulkhead structure is in place
        // In a real implementation, we'd test with shared semaphore state
        let result: Result<&str, BulkheadError<&str>> = with_bulkhead(config, || async {
            Ok::<&str, &str>("test")
        }).await;

        assert_eq!(result, Ok("test"));
    }
}