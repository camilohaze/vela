# ADR-113AG-001: Implementación de decoradores @consumer y @subscribe

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
En el contexto de VELA-600 Message Brokers, necesitamos una forma declarativa para que los desarrolladores puedan suscribirse a topics de message brokers sin escribir código boilerplate. Los decoradores @consumer y @subscribe permiten definir consumers de forma declarativa, similar a NestJS o Spring Boot.

## Decisión
Implementaremos decoradores @consumer y @subscribe en el compiler frontend que:

1. **@consumer(topic)**: Decorator para funciones que procesan mensajes de un topic específico
2. **@subscribe(broker, topic)**: Decorator más explícito que especifica broker y topic

Los decoradores generarán código que registra automáticamente los consumers en el broker al inicio de la aplicación.

## Consecuencias

### Positivas
- ✅ API declarativa y limpia para message brokers
- ✅ Reducción de boilerplate code
- ✅ Type safety en tiempo de compilación
- ✅ Integración natural con el sistema de DI de Vela
- ✅ Soporte para múltiples brokers (RabbitMQ, Kafka, Redis)

### Negativas
- ❌ Complejidad añadida al compiler
- ❌ Dependencia del runtime para registrar consumers

## Alternativas Consideradas
1. **Configuración manual**: Rechazada por verbose y propenso a errores
2. **Fluent API**: Rechazada por menos declarativo
3. **Anotaciones en YAML**: Rechazada por separar lógica del código

## Referencias
- Jira: [VELA-600](https://velalang.atlassian.net/browse/VELA-600)
- Task: [TASK-113AG](https://velalang.atlassian.net/browse/TASK-113AG)
- Documentación: Message Broker Architecture (TASK-113AD)

## Implementación
Ver código en: `compiler/src/message_broker_decorators.rs`