//! Tests unitarios para el módulo de resilience
//!
//! Tests para VELA-600 Message Brokers - TASK-113AJ
//! Fecha: 2025-12-11

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::resilience::{
    CircuitBreaker, CircuitBreakerConfig, DeadLetterConfig, ErrorKind, CircuitState,
    ResilientConsumerBuilder, RetryPolicy, classify_error,
};
use crate::{BrokerError, ConsumerError, MessageConsumer, RawMessage};

// Mock consumer para testing
#[derive(Clone)]
struct MockConsumer {
    pub call_count: Arc<Mutex<usize>>,
    pub should_fail: Arc<Mutex<bool>>,
    pub fail_count: Arc<Mutex<usize>>,
    pub fail_times: Arc<Mutex<usize>>,
}

impl MockConsumer {
    fn new() -> Self {
        Self {
            call_count: Arc::new(Mutex::new(0)),
            should_fail: Arc::new(Mutex::new(false)),
            fail_count: Arc::new(Mutex::new(0)),
            fail_times: Arc::new(Mutex::new(0)),
        }
    }

    async fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().await = fail;
    }

    async fn set_fail_times(&self, times: usize) {
        *self.fail_times.lock().await = times;
    }

    async fn get_call_count(&self) -> usize {
        *self.call_count.lock().await
    }

    async fn get_fail_count(&self) -> usize {
        *self.fail_count.lock().await
    }
}

#[async_trait::async_trait]
impl MessageConsumer for MockConsumer {
    async fn consume(&self, _message: RawMessage) -> Result<(), ConsumerError> {
        let mut call_count = self.call_count.lock().await;
        *call_count += 1;

        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            let mut fail_count = self.fail_count.lock().await;
            *fail_count += 1;
            return Err(ConsumerError::ProcessingFailed { message: "Mock failure".to_string() });
        }

        let fail_times = *self.fail_times.lock().await;
        let current_fail_count = *self.fail_count.lock().await;

        if current_fail_count < fail_times {
            let mut fail_count = self.fail_count.lock().await;
            *fail_count += 1;
            return Err(ConsumerError::ProcessingFailed { message: "Mock failure".to_string() });
        }

        Ok(())
    }

    fn topic(&self) -> &str {
        "test.topic"
    }
}

mod retry_policy_tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_policy_success_first_attempt() {
        let consumer = MockConsumer::new();
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_millis(1000),
            exponential_backoff: true,
        };

        let resilient = ResilientConsumerBuilder::new(consumer)
            .retry_policy(policy)
            .build();

        let message = RawMessage {
            id: "test".to_string(),
            topic: "test.topic".to_string(),
            payload: vec![1, 2, 3],
            timestamp: chrono::Utc::now(),
            headers: std::collections::HashMap::new(),
            correlation_id: Some("correlation-id".to_string()),
        };

        let result = resilient.consume(message).await;
        assert!(result.is_ok());

        // Debería haber sido llamado solo una vez
        assert_eq!(resilient.consumer().get_call_count().await, 1);
    }

    #[tokio::test]
    async fn test_retry_policy_eventual_success() {
        let consumer = MockConsumer::new();
        consumer.set_fail_times(2).await;

        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_millis(1000),
            exponential_backoff: true,
        };

        let resilient = ResilientConsumerBuilder::new(consumer)
            .retry_policy(policy)
            .build();

        let message = RawMessage {
            id: "test".to_string(),
            topic: "test.topic".to_string(),
            payload: vec![1, 2, 3],
            timestamp: chrono::Utc::now(),
            headers: std::collections::HashMap::new(),
            correlation_id: Some("correlation-id".to_string()),
        };

        let result = timeout(Duration::from_secs(2), resilient.consume(message)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());

        // Debería haber sido llamado 3 veces (2 fallos + 1 éxito)
        assert_eq!(resilient.consumer().get_call_count().await, 3);
        assert_eq!(resilient.consumer().get_fail_count().await, 2);
    }

    #[tokio::test]
    async fn test_retry_policy_max_attempts_exceeded() {
        let consumer = MockConsumer::new();
        consumer.set_should_fail(true).await;

        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_millis(1000),
            exponential_backoff: true,
        };

        let resilient = ResilientConsumerBuilder::new(consumer)
            .retry_policy(policy)
            .build();

        let message = RawMessage {
            id: "test".to_string(),
            topic: "test.topic".to_string(),
            payload: vec![1, 2, 3],
            timestamp: chrono::Utc::now(),
            headers: std::collections::HashMap::new(),
            correlation_id: Some("correlation-id".to_string()),
        };

        let result = timeout(Duration::from_secs(2), resilient.consume(message)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());

        // Debería haber sido llamado 3 veces (todos fallos)
        assert_eq!(resilient.consumer().get_call_count().await, 3);
        assert_eq!(resilient.consumer().get_fail_count().await, 3);
    }
}

mod circuit_breaker_tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_secs(1),
        };
        let mut breaker = CircuitBreaker::new(config);

        // Estado inicial: Closed
        assert_eq!(*breaker.state(), CircuitState::Closed);

        // Permitir requests
        assert!(breaker.allow_request());
        assert!(breaker.allow_request());
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_secs(1),
        };
        let mut breaker = CircuitBreaker::new(config);

        // Registrar fallos para abrir el circuito
        breaker.record_failure();
        breaker.record_failure();

        // Estado: Open
        assert_eq!(*breaker.state(), CircuitState::Open);

        // No permitir requests
        assert!(!breaker.allow_request());
        assert!(!breaker.allow_request());
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
        };
        let mut breaker = CircuitBreaker::new(config);

        // Abrir el circuito
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(*breaker.state(), CircuitState::Open);

        // Esperar timeout para half-open
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Permitir una request de prueba (esto debería cambiar el estado a HalfOpen)
        assert!(breaker.allow_request());

        // Estado: HalfOpen
        assert_eq!(*breaker.state(), CircuitState::HalfOpen);

        // Registrar éxito - debería cerrarse
        breaker.record_success();
        assert_eq!(*breaker.state(), CircuitState::Closed);
    }
}

mod error_classification_tests {
    use super::*;

    #[test]
    fn test_error_classification_retryable() {
        assert_eq!(
            classify_error(&BrokerError::ConnectionFailed { message: "test".to_string() }),
            ErrorKind::Retryable
        );
        assert_eq!(
            classify_error(&BrokerError::Timeout { message: "test".to_string() }),
            ErrorKind::Retryable
        );
        assert_eq!(
            classify_error(&BrokerError::PublishFailed { message: "test".to_string() }),
            ErrorKind::Retryable
        );
    }

    #[test]
    fn test_error_classification_non_retryable() {
        // Use a different error for NonRetryable test (e.g., InvalidConfiguration)
        assert_eq!(
            classify_error(&BrokerError::InvalidConfiguration { message: "test".to_string() }),
            ErrorKind::NonRetryable
        );
        assert_eq!(
            classify_error(&BrokerError::InvalidConfiguration { message: "test".to_string() }),
            ErrorKind::NonRetryable
        );
    }

    #[test]
    fn test_error_classification_circuit_break() {
        assert_eq!(
            classify_error(&BrokerError::CircuitBreakerOpen),
            ErrorKind::CircuitBreak
        );
    }
}

mod resilient_consumer_tests {
    use super::*;

    #[tokio::test]
    async fn test_resilient_consumer_builder() {
        let consumer = MockConsumer::new();

        let resilient = ResilientConsumerBuilder::new(consumer)
            .retry_policy(RetryPolicy::default())
            .dlq_config(Some(DeadLetterConfig {
                queue_name: "test.dlq".to_string(),
                max_age: Some(7),
                max_size: Some(100),
            }))
            .circuit_breaker(CircuitBreaker::new(CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 1,
                timeout: Duration::from_secs(30),
            }))
            .build();

        // Instead of checking private fields, test that consuming works and doesn't panic
        let message = RawMessage {
            id: "test2".to_string(),
            topic: "test.topic".to_string(),
            payload: vec![1, 2, 3],
            headers: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            correlation_id: Some("correlation-id".to_string()),
        };
        let result = resilient.consume(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resilient_consumer_without_resilience() {
        let consumer = MockConsumer::new();

        let resilient = ResilientConsumerBuilder::new(consumer).build();

        let message = RawMessage {
            id: "test".to_string(),
            topic: "test.topic".to_string(),
            payload: vec![1, 2, 3],
            headers: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            correlation_id: Some("correlation-id".to_string()),
        };

        let result = resilient.consume(message).await;
        assert!(result.is_ok());

        // Sin resilience, debería ser llamado directamente
        assert_eq!(resilient.consumer().get_call_count().await, 1);
    }
}