//! Resilience Patterns for Message Brokers
//!
//! This module implements retry policies and dead letter queues for robust
//! message processing in VELA-600.
//!
//! Implementation of: TASK-113AH
//! History: VELA-600
//! Date: 2025-12-11

use crate::{MessageBroker, MessageConsumer, BrokerError, ConsumerError, RawMessage};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

/// Configuration for retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            exponential_backoff: true,
        }
    }
}

impl RetryPolicy {
    /// Calculate delay for the given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if !self.exponential_backoff {
            return self.initial_delay.min(self.max_delay);
        }

        let delay = if attempt == 0 {
            self.initial_delay
        } else {
            let multiplier = self.backoff_multiplier.powi(attempt as i32);
            self.initial_delay.mul_f64(multiplier)
        };

        delay.min(self.max_delay)
    }
}

/// Configuration for dead letter queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterConfig {
    /// Name of the dead letter exchange/queue/topic
    pub queue_name: String,
    /// Maximum age of messages in DLQ before deletion
    pub max_age: Option<u64>,
    /// Maximum size of DLQ
    pub max_size: Option<u64>,
}

impl Default for DeadLetterConfig {
    fn default() -> Self {
        Self {
            queue_name: "dead-letter-queue".to_string(),
            max_age: Some(7), // 7 days
            max_size: Some(10000),
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Timeout to attempt reset from open to half-open
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
        }
    }

    /// Check if request should be allowed
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            _ => {
                self.failure_count = 0;
            }
        }
    }

    /// Record failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    /// Get the current state of the circuit breaker
    pub fn state(&self) -> &CircuitState {
        &self.state
    }
}

/// Error classification for retry decisions
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Temporary error that should be retried
    Retryable,
    /// Permanent error that should go to DLQ
    NonRetryable,
    /// Circuit breaker should be triggered
    CircuitBreak,
}

/// Classify broker errors for retry/DLQ decisions
pub fn classify_error(error: &BrokerError) -> ErrorKind {
    match error {
        BrokerError::ConnectionFailed { .. } => ErrorKind::Retryable,
        BrokerError::Timeout { .. } => ErrorKind::Retryable,
        BrokerError::SerializationError(_) => ErrorKind::NonRetryable,
        BrokerError::PublishFailed { .. } => ErrorKind::Retryable,
        BrokerError::SubscribeFailed { .. } => ErrorKind::Retryable,
        BrokerError::InvalidConfiguration { .. } => ErrorKind::NonRetryable,
        BrokerError::CircuitBreakerOpen => ErrorKind::CircuitBreak,
        BrokerError::UnsubscribeFailed { .. } => ErrorKind::Retryable,
    }
}

/// Calculate delay for retry attempt
pub fn calculate_retry_delay(policy: &RetryPolicy, attempt: u32) -> Duration {
    if !policy.exponential_backoff {
        return policy.initial_delay;
    }

    let delay = policy.initial_delay.as_millis() as f64 *
                policy.backoff_multiplier.powi(attempt.saturating_sub(1) as i32);

    let max_delay_ms = policy.max_delay.as_millis() as f64;
    let final_delay_ms = delay.min(max_delay_ms);

    Duration::from_millis(final_delay_ms as u64)
}

/// Resilient message consumer with retry and DLQ support
pub struct ResilientConsumer<C: MessageConsumer> {
    consumer: C,
    retry_policy: RetryPolicy,
    dlq_config: Option<DeadLetterConfig>,
    circuit_breaker: Option<CircuitBreaker>,
}

impl<C: MessageConsumer> ResilientConsumer<C> {
    pub fn new(
        consumer: C,
        retry_policy: RetryPolicy,
        dlq_config: Option<DeadLetterConfig>,
        circuit_breaker: Option<CircuitBreaker>,
    ) -> Self {
        Self {
            consumer,
            retry_policy,
            dlq_config,
            circuit_breaker,
        }
    }

    /// Getter for the consumer field
    pub fn consumer(&self) -> &C {
        &self.consumer
    }

    /// Check if an error is retryable
    fn is_retryable_error(&self, error: &ConsumerError) -> bool {
        match error {
            ConsumerError::ProcessingFailed { .. } => true,
            ConsumerError::DeserializationError(_) => false, // Usually not retryable
            ConsumerError::RetryExhausted => false,
            ConsumerError::InvalidMessageFormat { .. } => false,
            ConsumerError::Temporary(_) => true,
        }
    }

    /// Process message with resilience patterns (implements MessageConsumer)
    pub async fn consume(&self, message: RawMessage) -> Result<(), ConsumerError> {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.retry_policy.max_attempts {
            attempt += 1;

            match self.consumer.consume(message.clone()).await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    last_error = Some(error);

                    // If this is the last attempt, don't retry
                    if attempt >= self.retry_policy.max_attempts {
                        break;
                    }

                    // Check if error is retryable
                    if !self.is_retryable_error(&last_error.as_ref().unwrap()) {
                        break;
                    }

                    // Calculate delay
                    let delay = self.retry_policy.calculate_delay(attempt - 1);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        // If we get here, all attempts failed
        Err(last_error.unwrap_or(ConsumerError::RetryExhausted))
    }

    /// Process message with full resilience patterns (requires broker access)
    pub async fn process_with_resilience(
        &mut self,
        message: RawMessage,
        broker: &mut impl MessageBroker,
    ) -> Result<(), BrokerError> {
        // Check circuit breaker
        if let Some(cb) = &mut self.circuit_breaker {
            if !cb.allow_request() {
                return Err(BrokerError::CircuitBreakerOpen);
            }
        }

        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.retry_policy.max_attempts {
            attempt += 1;

            match self.consumer.consume(message.clone()).await {
                Ok(()) => {
                    // Success - record in circuit breaker
                    if let Some(cb) = &mut self.circuit_breaker {
                        cb.record_success();
                    }
                    return Ok(());
                }
                Err(error) => {
                    // Convert ConsumerError to BrokerError for classification
                    let broker_error = match error {
                        ConsumerError::ProcessingFailed { message } =>
                            BrokerError::PublishFailed { message },
                        ConsumerError::DeserializationError(e) =>
                            BrokerError::SerializationError(e),
                        ConsumerError::RetryExhausted =>
                            BrokerError::Timeout { message: "Retry exhausted".to_string() },
                        ConsumerError::InvalidMessageFormat { message } =>
                            BrokerError::InvalidConfiguration { message },
                        ConsumerError::Temporary(msg) =>
                            BrokerError::Timeout { message: format!("Temporary error: {}", msg) },
                    };

                    last_error = Some(format!("{:?}", broker_error));

                    match classify_error(&broker_error) {
                        ErrorKind::NonRetryable => {
                            // Send to DLQ if configured
                            if let Some(_dlq) = &self.dlq_config {
                                self.send_to_dlq(broker, message.clone(), &broker_error).await?;
                            }
                            // Record failure in circuit breaker
                            if let Some(cb) = &mut self.circuit_breaker {
                                cb.record_failure();
                            }
                            return Err(broker_error);
                        }
                        ErrorKind::CircuitBreak => {
                            // Trigger circuit breaker
                            if let Some(cb) = &mut self.circuit_breaker {
                                cb.record_failure();
                            }
                            return Err(broker_error);
                        }
                        ErrorKind::Retryable => {
                            // Retry after delay
                            if attempt < self.retry_policy.max_attempts {
                                let delay = calculate_retry_delay(&self.retry_policy, attempt);
                                sleep(delay).await;
                                continue;
                            } else {
                                // Max retries reached - send to DLQ if configured
                                if let Some(_dlq) = &self.dlq_config {
                                    self.send_to_dlq(broker, message.clone(), &broker_error).await?;
                                }
                                // Record failure in circuit breaker
                                if let Some(cb) = &mut self.circuit_breaker {
                                    cb.record_failure();
                                }
                                return Err(broker_error);
                            }
                        }
                    }
                }
            }
        }

        // This should not be reached, but just in case
        Err(last_error.map(|msg| BrokerError::PublishFailed { message: msg }).unwrap_or_else(|| {
            BrokerError::InvalidConfiguration { message: "Unknown error in resilient processing".to_string() }
        }))
    }

    /// Send message to dead letter queue
    async fn send_to_dlq(
        &self,
        broker: &mut impl MessageBroker,
        message: RawMessage,
        error: &BrokerError,
    ) -> Result<(), BrokerError> {
        if let Some(dlq) = &self.dlq_config {
            // Create DLQ message with error information
            let dlq_message = RawMessage {
                id: format!("dlq-{}", message.id),
                topic: dlq.queue_name.clone(),
                payload: format!(
                    "{{\"original_topic\": \"{}\", \"payload\": {}, \"error\": \"{}\", \"timestamp\": \"{}\"}}",
                    message.topic,
                    String::from_utf8_lossy(&message.payload),
                    error,
                    chrono::Utc::now().to_rfc3339()
                ).into_bytes(),
                headers: {
                    let mut headers = message.headers.clone();
                    headers.insert("x-death-reason".to_string(), format!("{:?}", error));
                    headers.insert("x-death-count".to_string(), "1".to_string());
                    headers.insert("x-original-topic".to_string(), message.topic.clone());
                    headers
                },
                timestamp: chrono::Utc::now(),
                correlation_id: message.correlation_id.clone(),
            };

            broker.publish(&dlq_message.topic.clone(), dlq_message).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<C: MessageConsumer> MessageConsumer for ResilientConsumer<C> {
    async fn consume(&self, message: RawMessage) -> Result<(), ConsumerError> {
        // Simplified version without broker access for DLQ
        // In production, this would need a way to access the broker
        self.consumer.consume(message).await
    }

    fn topic(&self) -> &str {
        self.consumer.topic()
    }

    fn group_id(&self) -> Option<&str> {
        self.consumer.group_id()
    }

    fn max_retries(&self) -> u32 {
        self.retry_policy.max_attempts
    }
}

/// Builder for resilient consumers
pub struct ResilientConsumerBuilder<C: MessageConsumer> {
    consumer: C,
    retry_policy: RetryPolicy,
    dlq_config: Option<DeadLetterConfig>,
    circuit_breaker: Option<CircuitBreaker>,
}

impl<C: MessageConsumer> ResilientConsumerBuilder<C> {
    pub fn new(consumer: C) -> Self {
        Self {
            consumer,
            retry_policy: RetryPolicy::default(),
            dlq_config: Some(DeadLetterConfig::default()),
            circuit_breaker: Some(CircuitBreaker::new(CircuitBreakerConfig::default())),
        }
    }

    pub fn retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    pub fn dlq_config(mut self, config: Option<DeadLetterConfig>) -> Self {
        self.dlq_config = config;
        self
    }

    pub fn circuit_breaker(mut self, breaker: CircuitBreaker) -> Self {
        self.circuit_breaker = Some(breaker);
        self
    }

    pub fn build(self) -> ResilientConsumer<C> {
        ResilientConsumer::new(
            self.consumer,
            self.retry_policy,
            self.dlq_config,
            self.circuit_breaker,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawMessage;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    struct MockMessageConsumer {
        should_fail: Arc<Mutex<bool>>,
        fail_count: Arc<Mutex<u32>>,
    }

    #[async_trait]
    #[async_trait]
    impl MessageConsumer for MockMessageConsumer {
        async fn consume(&self, _message: RawMessage) -> Result<(), ConsumerError> {
            let mut fail_count = self.fail_count.lock().await;
            *fail_count += 1;

            let should_fail = *self.should_fail.lock().await;
            if should_fail && *fail_count < 3 {
                Err(ConsumerError::Temporary("Temporary failure".to_string()))
            } else {
                Ok(())
            }
        }

        fn topic(&self) -> &str {
            "test-topic"
        }
    }

    #[tokio::test]
    async fn test_retry_policy_success_after_failures() {
        let consumer = MockMessageConsumer {
            should_fail: Arc::new(Mutex::new(true)),
            fail_count: Arc::new(Mutex::new(0)),
        };

        let retry_policy = RetryPolicy {
            max_attempts: 5,
            initial_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let mut resilient = ResilientConsumerBuilder::new(consumer)
            .retry_policy(retry_policy)
            .dlq_config(None)
            .circuit_breaker(CircuitBreaker::new(CircuitBreakerConfig::default()))
            .build();

        // Mock broker (not used in this test)
        struct MockBroker;
        #[async_trait]
        impl MessageBroker for MockBroker {
            async fn publish(&self, _topic: &str, _message: RawMessage) -> Result<(), BrokerError> {
                Ok(())
            }
            async fn subscribe(&self, _topic: &str, _consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError> {
                Ok(())
            }
            async fn unsubscribe(&self, _topic: &str) -> Result<(), BrokerError> {
                Ok(())
            }
            async fn close(&self) -> Result<(), BrokerError> {
                Ok(())
            }
        }

        let mut broker = MockBroker;
        let message = RawMessage {
            correlation_id: Some("test-correlation-id".to_string()),
            id: "test-id".to_string(),
            topic: "test".to_string(),
            payload: b"test".to_vec(),
            headers: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        let result = resilient.process_with_resilience(message, &mut broker).await;
        assert!(result.is_ok());

        let fail_count = *resilient.consumer.fail_count.lock().await;
        assert_eq!(fail_count, 3); // Failed twice, succeeded on third attempt
    }

    #[test]
    fn test_calculate_retry_delay() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            exponential_backoff: true,
        };

        assert_eq!(calculate_retry_delay(&policy, 1), Duration::from_millis(1000));
        assert_eq!(calculate_retry_delay(&policy, 2), Duration::from_millis(2000));
        assert_eq!(calculate_retry_delay(&policy, 3), Duration::from_millis(4000));
    }

    #[test]
    fn test_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
        };

        let mut cb = CircuitBreaker::new(config);

        // Initially closed
        assert!(cb.allow_request());

        // Record failures
        cb.record_failure();
        cb.record_failure();
        assert!(cb.allow_request()); // Still closed

        cb.record_failure();
        assert!(!cb.allow_request()); // Now open

        // Wait for timeout and try again
        std::thread::sleep(Duration::from_secs(2));
        assert!(cb.allow_request()); // Half-open

        cb.record_success();
        assert!(cb.allow_request()); // Still half-open

        cb.record_success();
        assert!(cb.allow_request()); // Now closed again
    }
}