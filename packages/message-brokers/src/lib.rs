//! # Message Brokers para Vela
//!
//! Sistema de mensajería pub/sub con soporte para múltiples brokers
//! (RabbitMQ, Kafka, Redis) con type safety y resilience patterns.
//!
//! ## Arquitectura
//!
//! - **MessageBroker**: Interface genérica para brokers
//! - **Message<T>**: Mensajes type-safe con serialización
//! - **MessageConsumer**: Interface para consumidores
//! - **Resilience**: Retry, DLQ, circuit breaker
//!
//! ## Ejemplo de uso
//!
//! ```rust,no_run
//! use message_brokers::{MessageBroker, Message, MessageConsumer, RawMessage};
//!
//! // Publicar mensaje
//! let message = Message {
//!     id: "msg-123".to_string(),
//!     topic: "orders.created".to_string(),
//!     payload: vec![1, 2, 3], // bytes
//!     headers: std::collections::HashMap::new(),
//!     timestamp: chrono::Utc::now(),
//!     correlation_id: Some("req-456".to_string()),
//! };
//!
//! // broker.publish("orders.created", message).await?;
//!
//! // Consumir mensajes
//! #[derive(Clone)]
//! struct OrderProcessor;
//!
//! #[async_trait::async_trait]
//! impl MessageConsumer for OrderProcessor {
//!     async fn consume(&self, message: RawMessage) -> Result<(), message_brokers::ConsumerError> {
//!         // Procesar mensaje
//!         Ok(())
//!     }
//!
//!     fn topic(&self) -> &str {
//!         "orders.created"
//!     }
//!
//!     fn group_id(&self) -> Option<&str> {
//!         Some("order-processor")
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Errores del sistema de message brokers
#[derive(Error, Debug)]
pub enum BrokerError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Publish failed: {message}")]
    PublishFailed { message: String },

    #[error("Subscribe failed: {message}")]
    SubscribeFailed { message: String },

    #[error("Unsubscribe failed: {message}")]
    UnsubscribeFailed { message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Circuit breaker open")]
    CircuitBreakerOpen,

    #[error("Timeout: {message}")]
    Timeout { message: String },

    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },
}

/// Errores de consumidores
#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("Processing failed: {message}")]
    ProcessingFailed { message: String },

    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("Retry exhausted")]
    RetryExhausted,

    #[error("Invalid message format: {message}")]
    InvalidMessageFormat { message: String },
    
    /// Temporary error, e.g. for retryable failures
    #[error("Temporary error: {0}")]
    Temporary(String),
}

/// Mensaje type-safe con serialización automática
#[derive(Debug, Clone)]
pub struct Message<T> {
    /// ID único del mensaje
    pub id: String,
    /// Topic/canal del mensaje
    pub topic: String,
    /// Payload serializable
    pub payload: T,
    /// Headers adicionales
    pub headers: HashMap<String, String>,
    /// Timestamp de creación
    pub timestamp: DateTime<Utc>,
    /// ID de correlación para tracing
    pub correlation_id: Option<String>,
}

/// Alias para mensajes con payload raw (bytes)
pub type RawMessage = Message<Vec<u8>>;

/// Interface genérica para message brokers
#[async_trait]
pub trait MessageBroker: Send + Sync {
    /// Publica un mensaje en un topic
    async fn publish(&self, topic: &str, message: RawMessage) -> Result<(), BrokerError>;

    /// Suscribe un consumer a un topic
    async fn subscribe(&self, topic: &str, consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError>;

    /// Desuscribe consumers de un topic
    async fn unsubscribe(&self, topic: &str) -> Result<(), BrokerError>;

    /// Cierra conexiones y libera recursos
    async fn close(&self) -> Result<(), BrokerError>;

    /// Verifica si el broker está saludable
    async fn health_check(&self) -> Result<(), BrokerError> {
        Ok(())
    }
}

/// Interface para consumidores de mensajes
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    /// Procesa un mensaje recibido
    async fn consume(&self, message: RawMessage) -> Result<(), ConsumerError>;

    /// Retorna el topic al que está suscrito
    fn topic(&self) -> &str;

    /// Retorna el group ID (opcional, para brokers que lo soporten)
    fn group_id(&self) -> Option<&str> {
        None
    }

    /// Número máximo de reintentos por defecto
    fn max_retries(&self) -> u32 {
        3
    }

    /// Timeout por defecto para procesamiento
    fn processing_timeout_secs(&self) -> u64 {
        30
    }
}

/// Configuración común para brokers
#[derive(Debug, Clone)]
pub struct BrokerConfig {
    /// URL de conexión al broker
    pub url: String,
    /// Timeout de conexión en segundos
    pub connection_timeout_secs: u64,
    /// Timeout de operaciones en segundos
    pub operation_timeout_secs: u64,
    /// Habilitar circuit breaker
    pub enable_circuit_breaker: bool,
    /// Umbral de fallos para circuit breaker
    pub circuit_breaker_threshold: u32,
    /// Timeout de recuperación del circuit breaker
    pub circuit_breaker_timeout_secs: u64,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            url: "localhost:5672".to_string(), // RabbitMQ default
            connection_timeout_secs: 10,
            operation_timeout_secs: 30,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 60,
        }
    }
}

/// Utilidades para serialización de mensajes
pub mod serialization {
    use super::{Message, RawMessage};
    use serde::{Serialize, Deserialize};

    /// Serializa un Message<T> a RawMessage
    pub fn serialize_message<T: Serialize>(message: Message<T>) -> Result<RawMessage, serde_json::Error> {
        let payload_bytes = serde_json::to_vec(&message.payload)?;
        Ok(RawMessage {
            id: message.id,
            topic: message.topic,
            payload: payload_bytes,
            headers: message.headers,
            timestamp: message.timestamp,
            correlation_id: message.correlation_id,
        })
    }

    /// Deserializa un RawMessage a Message<T>
    pub fn deserialize_message<T: for<'de> Deserialize<'de>>(raw: RawMessage) -> Result<Message<T>, serde_json::Error> {
        let payload: T = serde_json::from_slice(&raw.payload)?;
        Ok(Message {
            id: raw.id,
            topic: raw.topic,
            payload,
            headers: raw.headers,
            timestamp: raw.timestamp,
            correlation_id: raw.correlation_id,
        })
    }
}

/// Utilidades para generar IDs únicos
pub mod utils {
    use uuid::Uuid;

    /// Genera un ID único para mensajes
    pub fn generate_message_id() -> String {
        format!("msg-{}", Uuid::new_v4().simple())
    }

    /// Genera un ID de correlación único
    pub fn generate_correlation_id() -> String {
        format!("corr-{}", Uuid::new_v4().simple())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestPayload {
        name: String,
        value: i32,
    }

    #[test]
    fn test_message_serialization() {
        let payload = TestPayload {
            name: "test".to_string(),
            value: 42,
        };

        let message = Message {
            id: "test-id".to_string(),
            topic: "test.topic".to_string(),
            payload,
            headers: HashMap::new(),
            timestamp: Utc::now(),
            correlation_id: Some("corr-123".to_string()),
        };

        // Serializar
        let raw = serialization::serialize_message(message.clone()).unwrap();

        // Deserializar
        let deserialized: Message<TestPayload> = serialization::deserialize_message(raw).unwrap();

        assert_eq!(deserialized.payload.name, "test");
        assert_eq!(deserialized.payload.value, 42);
        assert_eq!(deserialized.topic, "test.topic");
    }

    #[test]
    fn test_generate_ids() {
        let msg_id1 = utils::generate_message_id();
        let msg_id2 = utils::generate_message_id();
        let corr_id = utils::generate_correlation_id();

        assert_ne!(msg_id1, msg_id2);
        assert!(msg_id1.starts_with("msg-"));
        assert!(corr_id.starts_with("corr-"));
    }
}

// Módulos de implementaciones concretas
pub mod rabbitmq;
pub mod kafka;
pub mod redis;
pub mod resilience;

#[cfg(test)]
mod resilience_tests;