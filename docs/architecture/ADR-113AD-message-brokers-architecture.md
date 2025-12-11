# ADR-113AD: Arquitectura de Message Brokers para Vela

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
Vela necesita soporte para message brokers para implementar arquitecturas event-driven en microservicios. Los desarrolladores requieren un sistema pub/sub que sea:

- **Type-safe**: Mensajes con tipos estáticos
- **Multi-broker**: Soporte para RabbitMQ, Kafka, Redis
- **Resilient**: Retry, dead letter queues, circuit breakers
- **Observable**: Tracing y métricas integradas
- **Declarativo**: Decoradores para consumers y publishers

## Decisión
Implementar arquitectura de message brokers con:

### 1. MessageBroker Interface Genérica
```rust
#[async_trait]
pub trait MessageBroker: Send + Sync {
    async fn publish(&self, topic: &str, message: Vec<u8>) -> Result<(), BrokerError>;
    async fn subscribe(&self, topic: &str, consumer: Box<dyn MessageConsumer>) -> Result<(), BrokerError>;
    async fn unsubscribe(&self, topic: &str) -> Result<(), BrokerError>;
    async fn close(&self) -> Result<(), BrokerError>;
}
```

### 2. Message Types con Type Safety
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

### 3. Consumer Interface
```rust
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    async fn consume(&self, message: RawMessage) -> Result<(), ConsumerError>;
    fn topic(&self) -> &str;
    fn group_id(&self) -> Option<&str>;
}
```

### 4. Brokers Soportados
- **RabbitMQ**: AMQP 0-9-1 con exchanges y queues
- **Kafka**: Topics con partitions y consumer groups
- **Redis**: Pub/Sub nativo y Streams

### 5. Resilience Patterns
- **Retry**: Backoff exponencial con jitter
- **Dead Letter Queue**: Mensajes fallidos van a DLQ
- **Circuit Breaker**: Protección contra brokers caídos
- **Idempotency**: Evitar procesamiento duplicado

### 6. Decoradores Vela
```vela
// Publisher
@injectable
service OrderService {
    broker: MessageBroker = inject(MessageBroker)
    
    @transactional
    async fn createOrder(order: Order) -> Result<OrderId> {
        // Crear orden en DB
        orderId = await this.orderRepo.save(order)
        
        // Publicar evento
        await this.broker.publish("orders.created", Message {
            payload: OrderCreatedEvent { orderId, customerId: order.customerId }
        })
        
        return orderId
    }
}

// Consumer
@consumer(topic="orders.created", groupId="order-processor")
service OrderProcessor {
    @injectable
    repo: OrderRepository = inject(OrderRepository)
    
    @consume
    async fn processOrderCreated(event: OrderCreatedEvent) -> Result<void> {
        // Procesar evento
        await this.repo.updateOrderStatus(event.orderId, "processing")
        
        // Enviar email de confirmación
        await this.emailService.sendConfirmation(event.customerId)
    }
}
```

## Consecuencias

### Positivas
- ✅ **Type Safety**: Mensajes strongly typed evitan errores runtime
- ✅ **Multi-broker**: Fácil migración entre brokers
- ✅ **Resilience**: Manejo robusto de fallos
- ✅ **Observability**: Tracing y métricas integradas
- ✅ **Developer Experience**: Decoradores declarativos

### Negativas
- ❌ **Complejidad**: Múltiples brokers requieren configuración compleja
- ❌ **Performance**: Serialización/deserialización agrega overhead
- ❌ **Learning Curve**: Nuevos conceptos (topics, partitions, consumer groups)

## Alternativas Consideradas

### 1. Broker Específico (Solo Kafka)
**Rechazado**: Limita opciones de despliegue y vendor lock-in

### 2. Sin Type Safety
**Rechazado**: Aumenta bugs y reduce DX

### 3. Solo HTTP Webhooks
**Rechazado**: No soporta event-driven architecture completa

## Referencias
- Jira: [VELA-600](https://velalang.atlassian.net/browse/VELA-600)
- Documentación: [Message Brokers Pattern](https://microservices.io/patterns/communication-style/messaging.html)
- Inspiración: Spring Cloud Stream, NATS, Apache Pulsar

## Implementación
Ver código en: `packages/message-brokers/`
Próxima tarea: TASK-113AE - Implementar MessageBroker interface</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-113AD-message-brokers-architecture.md