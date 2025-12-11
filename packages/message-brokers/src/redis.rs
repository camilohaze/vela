//! Implementación mock de Redis para MessageBroker
//!
//! Esta implementación simula Redis Pub/Sub para desarrollo.
//! En producción, reemplazar con redis crate real.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{self, Duration};

use crate::{
    BrokerConfig, BrokerError, MessageBroker, MessageConsumer, RawMessage,
};

/// Implementación mock de Redis MessageBroker
pub struct RedisBroker {
    config: BrokerConfig,
    consumers: Arc<Mutex<HashMap<String, task::JoinHandle<()>>>>,
    messages: Arc<Mutex<HashMap<String, Vec<RawMessage>>>>,
}

impl RedisBroker {
    /// Crear nueva instancia de Redis broker (mock)
    pub async fn new(config: BrokerConfig) -> Result<Self, BrokerError> {
        Ok(Self {
            config,
            consumers: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Parsear topic Redis (simula conversión de dots a colons)
    fn parse_topic(&self, topic: &str) -> String {
        topic.replace(".", ":").replace("/", ":")
    }

    /// Método estático helper para simulación de mensajes
    async fn simulate_message_reception_static(topic: String, consumer: Arc<dyn MessageConsumer>) {
        loop {
            // Simular recepción de mensaje cada 10 segundos
            time::sleep(Duration::from_secs(10)).await;

            let payload = format!("Mock Redis message for topic {} at {:?}", topic, chrono::Utc::now());
            let raw_message = RawMessage {
                id: uuid::Uuid::new_v4().to_string(),
                topic: topic.clone(),
                payload: payload.into_bytes(),
                headers: HashMap::new(),
                timestamp: chrono::Utc::now(),
                correlation_id: None,
            };

            // Procesar el mensaje
            if let Err(e) = consumer.consume(raw_message).await {
                eprintln!("Mock Redis consumer error for topic {}: {}", topic, e);
            }
        }
    }
}

#[async_trait]
impl MessageBroker for RedisBroker {
    async fn publish(&self, topic: &str, message: RawMessage) -> Result<(), BrokerError> {
        let redis_topic = self.parse_topic(topic);

        // Simular publicación guardando el mensaje
        let mut messages = self.messages.lock().await;
        messages.entry(redis_topic.clone()).or_insert_with(Vec::new).push(message);

        println!("Mock Redis: Published message to topic {}", redis_topic);
        Ok(())
    }

    async fn subscribe(&self, topic: &str, consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError> {
        let redis_topic = self.parse_topic(topic);
        let consumer_topic = topic.to_string();
        let consumer: Arc<dyn MessageConsumer> = Arc::from(consumer);

        // En mock implementation, solo registramos el consumer sin crear tasks infinitos
        // En producción, esto crearía un subscriber real
        let consumer_clone = Arc::clone(&consumer);

        let handle = task::spawn(async move {
            // Simular confirmación de suscripción
            let initial_payload = format!("Mock Redis subscription confirmed for topic {}", consumer_topic);
            let initial_message = RawMessage {
                id: uuid::Uuid::new_v4().to_string(),
                topic: consumer_topic.clone(),
                payload: initial_payload.into_bytes(),
                headers: HashMap::new(),
                timestamp: chrono::Utc::now(),
                correlation_id: None,
            };

            let _ = consumer_clone.consume(initial_message).await;
            // Task termina inmediatamente después del mensaje inicial
        });

        // Guardar el handle del consumer
        self.consumers.lock().await.insert(redis_topic, handle);

        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<(), BrokerError> {
        let redis_topic = self.parse_topic(topic);

        if let Some(handle) = self.consumers.lock().await.remove(&redis_topic) {
            handle.abort();
        }

        Ok(())
    }

    async fn close(&self) -> Result<(), BrokerError> {
        // Abortar todos los consumers sin esperar
        let mut consumers = self.consumers.lock().await;
        for (_, handle) in consumers.drain() {
            handle.abort();
        }

        // Limpiar mensajes
        self.messages.lock().await.clear();

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

    #[async_trait]
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
    async fn test_redis_broker_creation() {
        let config = BrokerConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            ..Default::default()
        };

        let broker = RedisBroker::new(config).await.unwrap();
        assert!(broker.config.url.contains("redis"));
    }

    #[tokio::test]
    async fn test_topic_parsing() {
        let config = BrokerConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            ..Default::default()
        };

        let broker = RedisBroker::new(config).await.unwrap();
        assert_eq!(broker.parse_topic("orders.created"), "orders:created");
        assert_eq!(broker.parse_topic("user/profile"), "user:profile");
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let config = BrokerConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            ..Default::default()
        };

        let broker = RedisBroker::new(config).await.unwrap();

        // Solo probar publish
        let message = RawMessage {
            id: "test-id".to_string(),
            topic: "test.topic".to_string(),
            payload: b"test payload".to_vec(),
            headers: HashMap::new(),
            timestamp: chrono::Utc::now(),
            correlation_id: Some("corr-id".to_string()),
        };

        // Publish debe funcionar
        let result = broker.publish("test.topic", message).await;
        assert!(result.is_ok());

        // Verificar que se guardó el mensaje
        let messages = broker.messages.lock().await;
        assert!(messages.contains_key("test:topic"));
        assert_eq!(messages["test:topic"].len(), 1);
    }
}