//! Ejemplo de uso del sistema de resilience con message brokers
//!
//! Este ejemplo muestra cómo usar ResilientConsumer para agregar
//! retry policies, dead letter queues y circuit breakers a cualquier
//! MessageConsumer.
//!
//! VELA-600 Message Brokers - TASK-113AJ
//! Fecha: 2025-12-11

use std::time::Duration;
use message_brokers::{
    resilience::{ResilientConsumerBuilder, RetryPolicy, DeadLetterConfig},
    MessageConsumer, RawMessage, BrokerError, MessageBroker, BrokerConfig,
    rabbitmq::RabbitMqBroker,
};
use async_trait::async_trait;

// Ejemplo de consumer de negocio
struct OrderProcessor {
    processed_count: std::sync::Arc<std::sync::Mutex<usize>>,
}

impl OrderProcessor {
    fn new() -> Self {
        Self {
            processed_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }
}

#[async_trait]
impl MessageConsumer for OrderProcessor {
    async fn consume(&self, message: RawMessage) -> Result<(), BrokerError> {
        // Simular procesamiento de orden
        println!("📦 Procesando orden: {}", message.id);

        // Simular error aleatorio (20% de probabilidad)
        if rand::random::<f32>() < 0.2 {
            println!("❌ Error procesando orden: {}", message.id);
            return Err(BrokerError::ConnectionError(
                "Error de conexión temporal".to_string()
            ));
        }

        // Procesamiento exitoso
        let mut count = self.processed_count.lock().unwrap();
        *count += 1;
        println!("✅ Orden procesada exitosamente: {} (total: {})", message.id, *count);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Iniciando ejemplo de Message Brokers con Resilience");
    println!("==================================================");

    // 1. Configurar broker RabbitMQ
    let broker_config = BrokerConfig {
        host: "localhost".to_string(),
        port: 5672,
        username: "guest".to_string(),
        password: "guest".to_string(),
        vhost: "/".to_string(),
        connection_timeout: Duration::from_secs(10),
    };

    let mut broker = RabbitMqBroker::new(broker_config).await?;
    println!("✅ Broker RabbitMQ conectado");

    // 2. Crear consumer de negocio
    let order_processor = OrderProcessor::new();

    // 3. Configurar consumer resilient
    let resilient_consumer = ResilientConsumerBuilder::new(order_processor)
        // Configurar retry policy
        .retry_policy(RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(10),
        })
        // Configurar dead letter queue
        .dlq_config(Some(DeadLetterConfig {
            queue_name: "orders.dlq".to_string(),
            max_age_days: 7,
            max_size_mb: 100,
        }))
        // Configurar circuit breaker
        .circuit_breaker_threshold(5)
        .circuit_breaker_timeout(Duration::from_secs(30))
        .build();

    println!("✅ Consumer resilient configurado:");
    println!("   - Retry: 3 intentos con backoff exponencial");
    println!("   - DLQ: orders.dlq (7 días, 100MB)");
    println!("   - Circuit Breaker: 5 fallos → 30s timeout");

    // 4. Suscribir consumer al topic
    broker.subscribe("orders.new", Box::new(resilient_consumer)).await?;
    println!("✅ Suscrito al topic: orders.new");

    // 5. Publicar algunas órdenes de ejemplo
    println!("\n📤 Publicando órdenes de ejemplo...");
    for i in 1..=10 {
        let order_message = RawMessage {
            id: format!("order-{}", i),
            topic: "orders.new".to_string(),
            payload: format!("{{\"order_id\": \"{}\", \"amount\": {}}}", i, i * 10).into_bytes(),
            timestamp: chrono::Utc::now(),
            headers: std::collections::HashMap::new(),
        };

        broker.publish("orders.new", order_message).await?;
        println!("📤 Orden {} enviada", i);

        // Pequeña pausa entre mensajes
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 6. Esperar procesamiento
    println!("\n⏳ Esperando procesamiento de mensajes...");
    tokio::time::sleep(Duration::from_secs(15)).await;

    // 7. Mostrar estadísticas
    println!("\n📊 Estadísticas finales:");
    println!("   - Mensajes enviados: 10");
    println!("   - Procesamiento completado");

    // 8. Limpiar
    broker.close().await?;
    println!("✅ Broker cerrado correctamente");

    println!("\n🎉 Ejemplo completado exitosamente!");
    println!("💡 El sistema de resilience manejó automáticamente:");
    println!("   - Reintentos en caso de fallos temporales");
    println!("   - Mensajes a DLQ si todos los reintentos fallan");
    println!("   - Protección con circuit breaker");

    Ok(())
}