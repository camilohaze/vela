use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

use crate::{BrokerError, BrokerConfig, MessageBroker, MessageConsumer, RawMessage};

/// Simplified Kafka implementation of the MessageBroker trait
/// This is a mock implementation for development purposes.
/// In production, this should be replaced with a real Kafka client.
pub struct KafkaBroker {
    config: BrokerConfig,
    connected: Arc<Mutex<bool>>,
}

impl KafkaBroker {
    /// Create a new Kafka broker instance
    pub fn new(config: BrokerConfig) -> Result<Self, BrokerError> {
        Ok(Self {
            config,
            connected: Arc::new(Mutex::new(false)),
        })
    }

    /// Parse topic from routing key (format: "topic" or "topic.key")
    fn parse_topic(routing_key: &str) -> &str {
        routing_key.split('.').next().unwrap_or(routing_key)
    }
}

#[async_trait]
impl MessageBroker for KafkaBroker {
    async fn publish(&self, topic: &str, message: RawMessage) -> Result<(), BrokerError> {
        let connected = self.connected.lock().await;
        if !*connected {
            return Err(BrokerError::ConnectionFailed {
                message: "Not connected to Kafka".to_string(),
            });
        }

        // In a real implementation, this would publish to Kafka
        println!("KafkaBroker: Publishing to topic '{}' with message id '{}'",
                topic, message.id);

        Ok(())
    }

    async fn subscribe(&self, topic: &str, consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError> {
        let connected = self.connected.lock().await;
        if !*connected {
            return Err(BrokerError::ConnectionFailed {
                message: "Not connected to Kafka".to_string(),
            });
        }

        // In a real implementation, this would subscribe to the topic
        println!("KafkaBroker: Subscribed to topic '{}'", topic);

        // Simulate consuming messages
        let consumer_topic = topic.to_string();
        task::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                let simulated_message = RawMessage {
                    id: crate::utils::generate_message_id(),
                    topic: consumer_topic.clone(),
                    payload: format!(r#"{{"event": "simulated", "timestamp": "{}"}}"#, chrono::Utc::now().timestamp()).into_bytes(),
                    headers: std::collections::HashMap::new(),
                    timestamp: chrono::Utc::now(),
                    correlation_id: Some(crate::utils::generate_correlation_id()),
                };

                if let Err(e) = consumer.consume(simulated_message).await {
                    eprintln!("Error handling simulated message: {:?}", e);
                }
            }
        });

        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<(), BrokerError> {
        // In a real implementation, this would unsubscribe from the topic
        println!("KafkaBroker: Unsubscribed from topic '{}'", topic);
        Ok(())
    }

    async fn close(&self) -> Result<(), BrokerError> {
        let mut connected = self.connected.lock().await;
        *connected = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MessageConsumer;

    struct TestConsumer;

    #[async_trait::async_trait]
    impl MessageConsumer for TestConsumer {
        async fn consume(&self, message: RawMessage) -> Result<(), crate::ConsumerError> {
            println!("Consumed message: {}", message.id);
            Ok(())
        }

        fn topic(&self) -> &str {
            "test"
        }
    }

    #[tokio::test]
    async fn test_kafka_broker_creation() {
        let config = BrokerConfig {
            url: "localhost:9092".to_string(),
            ..Default::default()
        };

        let result = KafkaBroker::new(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_topic_parsing() {
        assert_eq!(KafkaBroker::parse_topic("orders"), "orders");
        assert_eq!(KafkaBroker::parse_topic("orders.created"), "orders");
        assert_eq!(KafkaBroker::parse_topic("user.events.login"), "user");
    }
}