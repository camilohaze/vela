//! Implementación real de Redis para MessageBroker
//!
//! Esta implementación usa Redis Pub/Sub para mensajería.
//! Requiere un servidor Redis corriendo.

use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use futures_lite::StreamExt;

use crate::{
    BrokerConfig, BrokerError, MessageBroker, MessageConsumer, RawMessage,
};

/// Implementación real de Redis MessageBroker usando Pub/Sub
pub struct RedisBroker {
    config: BrokerConfig,
    client: Client,
    connection: Arc<Mutex<Option<Connection>>>,
    subscribers: Arc<Mutex<HashMap<String, task::JoinHandle<()>>>>,
}

impl RedisBroker {
    /// Crear nueva instancia de Redis broker
    pub async fn new(config: BrokerConfig) -> Result<Self, BrokerError> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| BrokerError::ConnectionFailed {
                message: format!("Failed to create Redis client: {}", e),
            })?;

        Ok(Self {
            config,
            client,
            connection: Arc::new(Mutex::new(None)),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Parsear topic Redis (convierte dots a colons para Redis channels)
    fn parse_topic(&self, topic: &str) -> String {
        topic.replace(".", ":").replace("/", ":")
    }

    /// Obtener conexión Redis
    async fn get_connection(&self) -> Result<Connection, BrokerError> {
        self.client.get_async_connection().await
            .map_err(|e| BrokerError::ConnectionFailed {
                message: format!("Failed to connect to Redis: {}", e),
            })
    }
}

#[async_trait]
impl MessageBroker for RedisBroker {
    async fn publish(&self, topic: &str, message: RawMessage) -> Result<(), BrokerError> {
        let redis_topic = self.parse_topic(topic);
        let mut conn = self.get_connection().await?;

        // Crear payload JSON manualmente para RawMessage
        let payload_json = format!(
            r#"{{"id":"{}","topic":"{}","payload":{},"headers":{},"timestamp":"{}","correlation_id":{}}}"#,
            message.id,
            message.topic,
            format!("{:?}", message.payload).replace(" ", "").replace("[", "[").replace("]", "]"),
            serde_json::to_string(&message.headers).unwrap_or_default(),
            message.timestamp.to_rfc3339(),
            message.correlation_id.as_ref().map(|s| format!("\"{}\"", s)).unwrap_or("null".to_string())
        );

        // Publicar en Redis Pub/Sub
        let _: () = conn.publish(&redis_topic, payload_json).await
            .map_err(|e| BrokerError::PublishFailed {
                message: format!("Failed to publish to Redis topic '{}': {}", redis_topic, e),
            })?;

        println!("RedisBroker: Published message to topic '{}'", redis_topic);
        Ok(())
    }

    async fn subscribe(&self, topic: &str, consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError> {
        let redis_topic = self.parse_topic(topic);
        let consumer_topic = topic.to_string();
        let consumer: Arc<dyn MessageConsumer> = Arc::from(consumer);

        // Crear una nueva conexión para el subscriber
        let conn = self.client.get_async_connection().await
            .map_err(|e| BrokerError::ConnectionFailed {
                message: format!("Failed to create subscriber connection: {}", e),
            })?;

        let consumer_clone = Arc::clone(&consumer);
        let topic_clone = consumer_topic.clone();
        let redis_topic_clone = redis_topic.clone();
        let redis_topic_error = redis_topic.clone();
        let redis_topic_log = redis_topic.clone();

        // Crear task para manejar mensajes entrantes
        let handle = tokio::task::spawn(async move {
            let mut pubsub_conn = conn.into_pubsub();
            if let Err(e) = pubsub_conn.subscribe(&redis_topic_clone).await {
                eprintln!("Redis subscriber error for topic {}: {}", redis_topic_error, e);
                return;
            }

            println!("RedisBroker: Subscribed to topic '{}'", redis_topic_log);

            // Loop para recibir mensajes
            while let Some(msg) = pubsub_conn.on_message().next().await {
                if let Ok(payload) = msg.get_payload::<String>() {
                    // Deserializar el mensaje manualmente
                    match serde_json::from_str::<serde_json::Value>(&payload) {
                        Ok(json_value) => {
                            // Reconstruir RawMessage desde JSON
                            if let (Some(id), Some(topic), Some(headers), Some(timestamp_str)) = (
                                json_value.get("id").and_then(|v| v.as_str()),
                                json_value.get("topic").and_then(|v| v.as_str()),
                                json_value.get("headers").and_then(|v| v.as_object()),
                                json_value.get("timestamp").and_then(|v| v.as_str()),
                            ) {
                                if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                                    let raw_message = RawMessage {
                                        id: id.to_string(),
                                        topic: topic.to_string(),
                                        payload: json_value.get("payload")
                                            .and_then(|v| v.as_array())
                                            .map(|arr| arr.iter().filter_map(|n| n.as_u64().map(|n| n as u8)).collect())
                                            .unwrap_or_default(),
                                        headers: headers.iter()
                                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                            .collect(),
                                        timestamp: timestamp.with_timezone(&chrono::Utc),
                                        correlation_id: json_value.get("correlation_id")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string()),
                                    };

                                    if let Err(e) = consumer_clone.consume(raw_message).await {
                                        eprintln!("Redis consumer error for topic {}: {}", topic_clone, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Redis message JSON parse error for topic {}: {}", topic_clone, e);
                        }
                    }
                }
            }
        });

        // Guardar el handle del subscriber
        self.subscribers.lock().await.insert(redis_topic, handle);

        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<(), BrokerError> {
        let redis_topic = self.parse_topic(topic);

        if let Some(handle) = self.subscribers.lock().await.remove(&redis_topic) {
            handle.abort();
            println!("RedisBroker: Unsubscribed from topic '{}'", redis_topic);
        }

        Ok(())
    }

    async fn close(&self) -> Result<(), BrokerError> {
        // Abortar todos los subscribers
        let mut subscribers = self.subscribers.lock().await;
        for (topic, handle) in subscribers.drain() {
            handle.abort();
            println!("RedisBroker: Closed subscriber for topic '{}'", topic);
        }

        // Cerrar conexión
        let mut conn_guard = self.connection.lock().await;
        *conn_guard = None;

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

        let consumer = TestConsumer::new("test.topic");
        let consumer_box = Box::new(consumer.clone()) as Box<dyn MessageConsumer>;

        // Test subscribe (this will fail without Redis server, but tests the API)
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
            // If Redis is not available, just test that the API doesn't panic
            println!("Redis not available for testing, skipping publish/subscribe test");
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