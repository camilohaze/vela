# TASK-113AG: Implementar decoradores @consumer y @subscribe

## 📋 Información General
- **Historia:** VELA-600
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Implementar decoradores @consumer y @subscribe para permitir suscripción declarativa a topics de message brokers, reduciendo boilerplate y mejorando la ergonomía del desarrollo.

## 🔨 Implementación

### Arquitectura
- **@consumer(topic)**: Decorator simple para funciones que procesan mensajes de un topic
- **@subscribe(broker, topic)**: Decorator explícito que especifica broker y topic
- **Generación de código**: Los decoradores generan automáticamente código de registro de consumers
- **Validación**: Verificación de firmas de función correctas en tiempo de compilación

### Archivos generados
- `compiler/src/message_broker_decorators.rs` - Lógica de parsing y generación de código
- `docs/architecture/ADR-113AG-001-decoradores-consumer-subscribe.md` - Decisión arquitectónica
- `compiler/tests/unit/test_message_broker_decorators.rs` - Tests unitarios
- `docs/features/VELA-600/TASK-113AG.md` - Esta documentación

### Código generado
Los decoradores generan llamadas a `__register_consumer!` macro que registra consumers en el runtime:

```rust
// Para @consumer("user.created")
__register_consumer!("default", "user.created", "MyModule", "handle_user_created");

// Para @subscribe("kafka", "orders")
__register_consumer!("kafka", "orders", "OrderModule", "process_order");
```

## ✅ Criterios de Aceptación
- [x] @consumer decorator parsea correctamente argumentos
- [x] @subscribe decorator parsea correctamente broker y topic
- [x] Validación de firmas de función consumer
- [x] Generación de código de registro automático
- [x] Tests unitarios con cobertura >= 80%
- [x] Documentación completa (ADR + docs)
- [x] Integración en pipeline de compilación

## 🔗 Referencias
- **Jira:** [TASK-113AG](https://velalang.atlassian.net/browse/TASK-113AG)
- **Historia:** [VELA-600](https://velalang.atlassian.net/browse/VELA-600)
- **ADR:** `docs/architecture/ADR-113AG-001-decoradores-consumer-subscribe.md`
- **Código:** `compiler/src/message_broker_decorators.rs`