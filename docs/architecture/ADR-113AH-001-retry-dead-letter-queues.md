# ADR-113AH-001: Implementación de Retry y Dead Letter Queues

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
En el contexto de VELA-600 Message Brokers, necesitamos manejar errores de procesamiento de mensajes de forma robusta. Los consumidores pueden fallar temporalmente, y necesitamos estrategias para reintentar el procesamiento sin perder mensajes, y manejar fallos permanentes enviándolos a dead letter queues.

## Decisión
Implementaremos un sistema de resilience con:

1. **Retry Policy**: Configurable con backoff exponencial, max attempts, y timeouts
2. **Dead Letter Queue (DLQ)**: Cola separada para mensajes que fallan permanentemente
3. **Circuit Breaker**: Protección contra fallos en cascada
4. **Error Classification**: Diferenciación entre errores retryable y no retryable

El sistema será genérico y aplicable a todos los brokers (RabbitMQ, Kafka, Redis).

## Consecuencias

### Positivas
- ✅ Robustez en procesamiento de mensajes
- ✅ Prevención de pérdida de mensajes
- ✅ Recuperación automática de fallos temporales
- ✅ Monitoreo y observabilidad de errores
- ✅ Configuración flexible por topic/consumer

### Negativas
- ❌ Complejidad añadida al sistema
- ❌ Overhead de performance en retries
- ❌ Almacenamiento adicional para DLQ

## Alternativas Consideradas
1. **Retry solo en aplicación**: Rechazada por pérdida de mensajes en crashes
2. **DLQ manual**: Rechazada por boilerplate y errores humanos
3. **Sin resilience**: Rechazada por falta de robustez en producción

## Referencias
- Jira: [VELA-600](https://velalang.atlassian.net/browse/VELA-600)
- Task: [TASK-113AH](https://velalang.atlassian.net/browse/TASK-113AH)
- Documentación: Message Broker Architecture (TASK-113AD)

## Implementación
Ver código en: `packages/message-brokers/src/resilience.rs`