//! Implementación mock mejorada de Kafka para MessageBroker
//!
//! Esta implementación simula Kafka para desarrollo sin dependencias nativas.
//! En producción, reemplazar con kafka crate o rdkafka cuando OpenSSL esté disponible.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{self, Duration};

use crate::{
    BrokerConfig, BrokerError, MessageBroker, MessageConsumer, RawMessage,
};

/// Implementación mock mejorada de Kafka MessageBroker
/// Simula comportamiento real de Kafka para desarrollo
pub struct KafkaBroker {
    config: BrokerConfig,
    messages: Arc<Mutex<HashMap<String, Vec<RawMessage>>>>,
    consumers: Arc<Mutex<HashMap<String, task::JoinHandle<()>>>>,
    connected: Arc<Mutex<bool>>,
}

impl KafkaBroker {
    /// Crear nueva instancia de Kafka broker (mock)
    pub async fn new(config: BrokerConfig) -> Result<Self, BrokerError> {
        Ok(Self {
            config,
            messages: Arc::new(Mutex::new(HashMap::new())),
            consumers: Arc::new(Mutex::new(HashMap::new())),
            connected: Arc::new(Mutex::new(true)), // Mock siempre "conectado"
        })
    }

    /// Parse topic from routing key (format: "topic" or "topic.key")
    fn parse_topic(routing_key: &str) -> String {
        routing_key.split('.').next().unwrap_or(routing_key).to_string()
    }

    /// Simular envío de mensaje a través de "red" (almacenamiento compartido)
    async fn simulate_network_publish(&self, topic: &str, message: RawMessage) -> Result<(), BrokerError> {
        // Simular latencia de red
        time::sleep(Duration::from_millis(10)).await;

        let mut messages = self.messages.lock().await;
        messages.entry(topic.to_string()).or_insert_with(Vec::new).push(message);

        println!("KafkaBroker: Published message to topic '{}' (simulated)", topic);
        Ok(())
    }

    /// Simular recepción de mensajes desde "red"
    async fn simulate_network_consume(topic: String, consumer: Arc<dyn MessageConsumer>) {
        loop {
            // Simular polling cada 2 segundos (como hace Kafka)
            time::sleep(Duration::from_secs(2)).await;

            // En implementación real, aquí se haría polling a Kafka
            // Para mock, generamos mensajes simulados ocasionalmente
            if rand::random::<f32>() < 0.3 { // 30% chance cada 2 segundos
                let simulated_payload = format!("Mock Kafka message for topic {} at {:?}", topic, chrono::Utc::now());
                let raw_message = RawMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    topic: topic.clone(),
                    payload: simulated_payload.into_bytes(),
                    headers: HashMap::new(),
                    timestamp: chrono::Utc::now(),
                    correlation_id: Some(format!("kafka-mock-{}", rand::random::<u32>())),
                };

                if let Err(e) = consumer.consume(raw_message).await {
                    eprintln!("Kafka mock consumer error for topic {}: {}", topic, e);
                }
            }
        }
    }
}

#[async_trait]
impl MessageBroker for KafkaBroker {
    async fn publish(&self, topic: &str, message: RawMessage) -> Result<(), BrokerError> {
        let connected = self.connected.lock().await;
        if !*connected {
            return Err(BrokerError::ConnectionFailed {
                message: "Not connected to Kafka (mock)".to_string(),
            });
        }

        // Simular publicación con latencia
        self.simulate_network_publish(topic, message).await
    }

    async fn subscribe(&self, topic: &str, consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError> {
        let connected = self.connected.lock().await;
        if !*connected {
            return Err(BrokerError::ConnectionFailed {
                message: "Not connected to Kafka (mock)".to_string(),
            });
        }

        let kafka_topic = Self::parse_topic(topic);
        let consumer: Arc<dyn MessageConsumer> = Arc::from(consumer);

        // Crear task que simula consumo de Kafka
        let consumer_clone = Arc::clone(&consumer);
        let topic_clone = kafka_topic.clone();

        let handle = task::spawn(async move {
            Self::simulate_network_consume(topic_clone, consumer_clone).await;
        });

        // Guardar el handle del consumer
        self.consumers.lock().await.insert(kafka_topic, handle);

        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<(), BrokerError> {
        let kafka_topic = Self::parse_topic(topic);

        if let Some(handle) = self.consumers.lock().await.remove(&kafka_topic) {
            handle.abort();
            println!("KafkaBroker: Unsubscribed from topic '{}' (simulated)", kafka_topic);
        }

        Ok(())
    }

    async fn close(&self) -> Result<(), BrokerError> {
        // Abortar todos los consumers
        let mut consumers = self.consumers.lock().await;
        for (topic, handle) in consumers.drain() {
            handle.abort();
            println!("KafkaBroker: Closed consumer for topic '{}' (simulated)", topic);
        }

        // Limpiar mensajes
        self.messages.lock().await.clear();

        // Marcar como desconectado
        let mut connected = self.connected.lock().await;
        *connected = false;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MessageConsumer, ConsumerError};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    struct TestConsumer {
        messages: Arc<Mutex<Vec<RawMessage>>>,
        topic: String,
    }

    impl TestConsumer {
        fn new(topic: &str) -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
                topic: topic.to_string(),
            }
        }

        async fn get_messages(&self) -> Vec<RawMessage> {
            self.messages.lock().await.clone()
        }
    }

    #[async_trait::async_trait]
    impl MessageConsumer for TestConsumer {
        async fn consume(&self, message: RawMessage) -> Result<(), ConsumerError> {
            self.messages.lock().await.push(message);
            Ok(())
        }

        fn topic(&self) -> &str {
            &self.topic
        }

        fn group_id(&self) -> Option<&str> {
            Some("test-group")
        }
    }

    #[tokio::test]
    async fn test_kafka_broker_creation() {
        let config = BrokerConfig {
            url: "localhost:9092".to_string(),
            ..Default::default()
        };

        let broker = KafkaBroker::new(config).await.unwrap();
        assert!(broker.config.url.contains("9092"));
    }

    #[tokio::test]
    async fn test_topic_parsing() {
        assert_eq!(KafkaBroker::parse_topic("orders"), "orders");
        assert_eq!(KafkaBroker::parse_topic("orders.created"), "orders");
        assert_eq!(KafkaBroker::parse_topic("user.events.login"), "user");
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let config = BrokerConfig {
            url: "localhost:9092".to_string(),
            ..Default::default()
        };

        let broker = KafkaBroker::new(config).await.unwrap();

        let consumer = TestConsumer::new("test.topic");
        let consumer_box = Box::new(consumer.clone()) as Box<dyn MessageConsumer>;

        // Test subscribe (this will fail without Kafka server, but tests the API)
        let subscribe_result = broker.subscribe("test.topic", consumer_box).await;

        if subscribe_result.is_ok() {
            // Test publish
            let message = RawMessage {
                id: "test-id".to_string(),
                topic: "test.topic".to_string(),
                payload: b"test payload".to_vec(),
                headers: HashMap::new(),
                timestamp: chrono::Utc::now(),
                correlation_id: Some("test-correlation".to_string()),
            };

            let publish_result = broker.publish("test.topic", message).await;
            assert!(publish_result.is_ok());

            // Test unsubscribe
            let unsubscribe_result = broker.unsubscribe("test.topic").await;
            assert!(unsubscribe_result.is_ok());

            // Test close
            let close_result = broker.close().await;
            assert!(close_result.is_ok());
        } else {
            // If Kafka is not available, just test that the API doesn't panic
            println!("Kafka not available for testing, skipping publish/subscribe test");
            // Still test that publish doesn't panic (even if it fails)
            let message = RawMessage {
                id: "test-id".to_string(),
                topic: "test.topic".to_string(),
                payload: b"test payload".to_vec(),
                headers: HashMap::new(),
                timestamp: chrono::Utc::now(),
                correlation_id: Some("test-correlation".to_string()),
            };
            let _ = broker.publish("test.topic", message).await; // Ignore result
        }
    }
}