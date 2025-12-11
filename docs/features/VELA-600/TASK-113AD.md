# TASK-113AD: Diseñar arquitectura de message brokers

## 📋 Información General
- **Historia:** VELA-600
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Diseñar la arquitectura completa para message brokers en Vela, incluyendo interfaces genéricas, tipos de mensajes type-safe, y soporte para múltiples brokers (RabbitMQ, Kafka, Redis).

## 🔨 Implementación

### Arquitectura Diseñada

#### 1. **MessageBroker Interface Genérica**
```rust
#[async_trait]
pub trait MessageBroker: Send + Sync {
    async fn publish(&self, topic: &str, message: Vec<u8>) -> Result<(), BrokerError>;
    async fn subscribe(&self, topic: &str, consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError>;
    async fn unsubscribe(&self, topic: &str) -> Result<(), BrokerError>;
    async fn close(&self) -> Result<(), BrokerError>;
}
```

#### 2. **Message Types con Type Safety**
```rust
#[derive(Serialize, Deserialize)]
pub struct Message<T: Serialize> {
    pub id: String,
    pub topic: String,
    pub payload: T,
    pub headers: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
}

pub type RawMessage = Message<Vec<u8>>;
```

#### 3. **Consumer Pattern**
```rust
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    async fn consume(&self, message: RawMessage) -> Result<(), ConsumerError>;
    fn topic(&self) -> &str;
    fn group_id(&self) -> Option<&str>;
}
```

#### 4. **Brokers Soportados**
- **RabbitMQ**: Exchanges, queues, routing keys
- **Kafka**: Topics, partitions, consumer groups
- **Redis**: Pub/Sub y Streams

#### 5. **Resilience Features**
- Retry con backoff exponencial
- Dead Letter Queues
- Circuit breaker protection
- Idempotency keys

### Decoradores Vela Planeados

#### Publisher Example
```vela
@injectable
service OrderService {
    broker: MessageBroker = inject(MessageBroker)
    
    @transactional
    async fn createOrder(order: Order) -> Result<OrderId> {
        // Business logic
        orderId = await this.orderRepo.save(order)
        
        // Publish event
        await this.broker.publish("orders.created", Message {
            payload: OrderCreatedEvent {
                orderId,
                customerId: order.customerId,
                amount: order.total
            }
        })
        
        return orderId
    }
}
```

#### Consumer Example
```vela
@consumer(topic="orders.created", groupId="order-processor")
service OrderProcessor {
    @injectable
    emailService: EmailService = inject(EmailService)
    
    @consume
    async fn processOrderCreated(event: OrderCreatedEvent) -> Result<void> {
        // Send confirmation email
        await this.emailService.sendOrderConfirmation(
            event.customerId,
            event.orderId
        )
        
        // Update order status
        await this.orderRepo.updateStatus(event.orderId, "confirmed")
    }
}
```

## ✅ Criterios de Aceptación
- [x] ADR creado con arquitectura completa
- [x] Interfaces genéricas definidas
- [x] Tipos de mensajes type-safe diseñados
- [x] Soporte multi-broker especificado
- [x] Resilience patterns incluidos
- [x] Ejemplos de código Vela incluidos

## 🔗 Referencias
- **Jira:** [TASK-113AD](https://velalang.atlassian.net/browse/TASK-113AD)
- **Historia:** [VELA-600](https://velalang.atlassian.net/browse/VELA-600)
- **ADR:** [ADR-113AD-message-brokers-architecture.md](../architecture/ADR-113AD-message-brokers-architecture.md)</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-600\TASK-113AD.md