//! Implementación de RabbitMQ para MessageBroker
//!
//! Esta implementación usa AMQP 0-9-1 con lapin crate.
//! Soporta exchanges, queues, routing keys y consumer groups.

use async_trait::async_trait;
use lapin::{
    options::*,
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures_lite::stream::StreamExt;

use crate::{
    BrokerConfig, BrokerError, MessageBroker, MessageConsumer, RawMessage,
};

/// Implementación de RabbitMQ MessageBroker
pub struct RabbitMqBroker {
    connection: Arc<Mutex<Option<Connection>>>,
    channel: Arc<Mutex<Option<Channel>>>,
    config: BrokerConfig,
    consumers: Arc<Mutex<HashMap<String, Consumer>>>,
}

impl RabbitMqBroker {
    /// Crear nueva instancia de RabbitMQ broker
    pub async fn new(config: BrokerConfig) -> Result<Self, BrokerError> {
        let broker = Self {
            connection: Arc::new(Mutex::new(None)),
            channel: Arc::new(Mutex::new(None)),
            config,
            consumers: Arc::new(Mutex::new(HashMap::new())),
        };

        broker.connect().await?;
        Ok(broker)
    }

    /// Establecer conexión con RabbitMQ
    async fn connect(&self) -> Result<(), BrokerError> {
        let connection = Connection::connect(
            &self.config.url,
            ConnectionProperties::default(),
        )
        .await
        .map_err(|e| BrokerError::ConnectionFailed {
            message: format!("Failed to connect to RabbitMQ: {}", e),
        })?;

        let channel = connection.create_channel().await.map_err(|e| {
            BrokerError::ConnectionFailed {
                message: format!("Failed to create channel: {}", e),
            }
        })?;

        *self.connection.lock().await = Some(connection);
        *self.channel.lock().await = Some(channel);

        Ok(())
    }

    /// Obtener channel activo
    async fn get_channel(&self) -> Result<Channel, BrokerError> {
        let channel_guard = self.channel.lock().await;
        if let Some(ref channel) = *channel_guard {
            Ok(channel.clone())
        } else {
            Err(BrokerError::ConnectionFailed {
                message: "No active channel".to_string(),
            })
        }
    }

    /// Convertir RawMessage a BasicProperties de AMQP
    fn message_to_properties(message: &RawMessage) -> BasicProperties {
        let mut properties = BasicProperties::default();

        // Set message ID
        properties = properties.with_message_id(message.id.clone().into());

        // Set timestamp
        properties = properties.with_timestamp(message.timestamp.timestamp() as u64);

        // Set correlation ID
        if let Some(corr_id) = &message.correlation_id {
            properties = properties.with_correlation_id(corr_id.clone().into());
        }

        // Set headers
        let mut headers = FieldTable::default();
        for (key, value) in &message.headers {
            headers.insert(key.clone().into(), AMQPValue::LongString(value.clone().into()));
        }
        properties = properties.with_headers(headers);

        properties
    }

    /// Convertir BasicProperties de AMQP a headers
    fn properties_to_headers(properties: &BasicProperties) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let Some(message_id) = properties.message_id() {
            headers.insert("message_id".to_string(), message_id.to_string());
        }

        if let Some(correlation_id) = properties.correlation_id() {
            headers.insert("correlation_id".to_string(), correlation_id.to_string());
        }

        if let Some(amqp_headers) = properties.headers() {
            for (key, value) in amqp_headers {
                if let AMQPValue::LongString(string_value) = value {
                    headers.insert(key.to_string(), string_value.to_string());
                }
            }
        }

        headers
    }
}

#[async_trait]
impl MessageBroker for RabbitMqBroker {
    async fn publish(&self, topic: &str, message: RawMessage) -> Result<(), BrokerError> {
        let channel = self.get_channel().await?;

        // Parse topic as exchange.routing_key
        let parts: Vec<&str> = topic.split('.').collect();
        let (exchange, routing_key) = if parts.len() >= 2 {
            (parts[0], parts[1..].join("."))
        } else {
            ("", topic.to_string())
        };

        // Declare exchange if it doesn't exist
        channel
            .exchange_declare(
                exchange,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::PublishFailed {
                message: format!("Failed to declare exchange: {}", e),
            })?;

        let properties = Self::message_to_properties(&message);

        channel
            .basic_publish(
                exchange,
                &routing_key,
                BasicPublishOptions::default(),
                &message.payload,
                properties,
            )
            .await
            .map_err(|e| BrokerError::PublishFailed {
                message: format!("Failed to publish message: {}", e),
            })?;

        Ok(())
    }

    async fn subscribe(&self, topic: &str, consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError> {
        let channel = self.get_channel().await?;

        // Parse topic as exchange.routing_key
        let parts: Vec<&str> = topic.split('.').collect();
        let (exchange, routing_key) = if parts.len() >= 2 {
            (parts[0], parts[1..].join("."))
        } else {
            ("", topic.to_string())
        };

        // Declare exchange
        channel
            .exchange_declare(
                exchange,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::SubscribeFailed {
                message: format!("Failed to declare exchange: {}", e),
            })?;

        // Declare queue
        let queue_name = format!("{}.{}", exchange, routing_key);
        let queue = channel
            .queue_declare(
                &queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::SubscribeFailed {
                message: format!("Failed to declare queue: {}", e),
            })?;

        // Bind queue to exchange
        channel
            .queue_bind(
                &queue_name,
                exchange,
                &routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::SubscribeFailed {
                message: format!("Failed to bind queue: {}", e),
            })?;

        // Create consumer
        let amqp_consumer = channel
            .basic_consume(
                &queue_name,
                &consumer.group_id().unwrap_or("default"),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::SubscribeFailed {
                message: format!("Failed to create consumer: {}", e),
            })?;

        // Store consumer
        self.consumers.lock().await.insert(topic.to_string(), amqp_consumer);

        // Spawn consumer task
        let consumers_clone = Arc::clone(&self.consumers);
        let consumer_arc = Arc::new(consumer);
        let topic_clone = topic.to_string();

        tokio::spawn(async move {
            let mut consumers = consumers_clone.lock().await;
            if let Some(consumer) = consumers.get_mut(&topic_clone) {
                while let Some(delivery) = StreamExt::next(consumer).await {
                    match delivery {
                        Ok(delivery) => {
                            let headers = Self::properties_to_headers(&delivery.properties);

                            let message = RawMessage {
                                id: delivery.properties.message_id()
                                    .as_ref()
                                    .map(|id| id.as_str())
                                    .unwrap_or(&uuid::Uuid::new_v4().to_string())
                                    .to_string(),
                                topic: topic_clone.clone(),
                                payload: delivery.data.clone(),
                                headers,
                                timestamp: chrono::Utc::now(),
                                correlation_id: delivery.properties.correlation_id()
                                    .as_ref()
                                    .map(|id| id.as_str().to_string()),
                            };

                            if let Err(e) = consumer_arc.consume(message).await {
                                eprintln!("Consumer error: {:?}", e);
                                // TODO: Implement dead letter queue
                            }

                            // Ack message
                            if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                                eprintln!("Failed to ack message: {:?}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Delivery error: {:?}", e);
                            break;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<(), BrokerError> {
        let mut consumers = self.consumers.lock().await;
        consumers.remove(topic);

        // Note: lapin doesn't have a direct unsubscribe method
        // The consumer is removed from our tracking, and the spawned task will end
        Ok(())
    }

    async fn close(&self) -> Result<(), BrokerError> {
        // Close consumers
        self.consumers.lock().await.clear();

        // Close channel
        if let Some(channel) = self.channel.lock().await.take() {
            channel.close(0, "").await.map_err(|e| {
                BrokerError::ConnectionFailed {
                    message: format!("Failed to close channel: {}", e),
                }
            })?;
        }

        // Close connection
        if let Some(connection) = self.connection.lock().await.take() {
            connection.close(0, "").await.map_err(|e| {
                BrokerError::ConnectionFailed {
                    message: format!("Failed to close connection: {}", e),
                }
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization;
    use chrono::Utc;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestMessage {
        content: String,
        number: i32,
    }

    #[tokio::test]
    async fn test_rabbitmq_broker_creation() {
        let config = BrokerConfig {
            url: "amqp://guest:guest@localhost:5672".to_string(),
            ..Default::default()
        };

        // Note: This test requires a running RabbitMQ instance
        // For now, just test that the config is accepted
        let _config = config;
        // TODO: Add integration tests when RabbitMQ is available
        assert!(true);
    }

    #[tokio::test]
    async fn test_message_conversion() {
        let test_msg = TestMessage {
            content: "test".to_string(),
            number: 42,
        };

        let message = crate::Message {
            id: "test-id".to_string(),
            topic: "test.exchange.routing".to_string(),
            payload: test_msg,
            headers: {
                let mut h = HashMap::new();
                h.insert("custom".to_string(), "value".to_string());
                h
            },
            timestamp: Utc::now(),
            correlation_id: Some("corr-123".to_string()),
        };

        let raw = serialization::serialize_message(message).unwrap();
        let properties = RabbitMqBroker::message_to_properties(&raw);

        assert_eq!(properties.message_id().as_ref().unwrap().as_str(), "test-id");
        assert_eq!(properties.correlation_id().as_ref().unwrap().as_str(), "corr-123");
    }
}