# ADR-005: Event System Architecture

## Estado
✅ Aceptado

## Fecha
2025-12-03

## Contexto
Vela necesita un sistema de eventos robusto para manejar comunicación asíncrona entre componentes. El runtime requiere un event bus thread-safe que soporte:

- Publicación/subscripción de eventos
- Handlers asíncronos
- Tipos de eventos fuertemente tipados
- Thread-safety para entornos concurrentes
- Detección de errores en handlers

## Decisión
Implementar un event system con las siguientes características:

### Arquitectura
- **EventBus**: Bus central thread-safe con RwLock
- **Event**: Trait para tipos de eventos
- **EventHandler**: Trait para handlers con async support
- **EventPublisher**: API para publicar eventos
- **EventSubscriber**: API para suscribirse a eventos

### Componentes
1. **Event Trait**: Define la estructura de eventos
2. **EventHandler Trait**: Define cómo manejar eventos
3. **EventBus**: Gestiona registro y distribución de eventos
4. **Error Handling**: EventError enum para errores del sistema

### Thread-Safety
- EventBus usa RwLock para acceso concurrente
- Handlers pueden ser Send + Sync
- Eventos deben ser Send + Sync + Clone

## Consecuencias

### Positivas
- ✅ Comunicación desacoplada entre componentes
- ✅ Soporte completo para async/await
- ✅ Type safety en eventos
- ✅ Thread-safety garantizada
- ✅ Fácil testing y mocking

### Negativas
- ❌ Overhead de locking en acceso concurrente
- ❌ Complejidad en gestión de lifetimes
- ❌ Posible latencia en handlers lentos

## Alternativas Consideradas

### 1. Canal-based Event System
**Descripción**: Usar tokio::sync::broadcast para eventos
**Rechazada porque**: No permite type safety fuerte, difícil de testear

### 2. Observer Pattern Manual
**Descripción**: Implementación manual de observer pattern
**Rechazada porque**: Boilerplate excesivo, no async-friendly

### 3. Actix Event System
**Descripción**: Integrar con actix actors
**Rechazada porque**: Acoplamiento fuerte con actix, complejidad innecesaria

## Implementación
Ver código en: `runtime/src/event/`

## Referencias
- Jira: TASK-RUST-305
- Epic: EPIC-RUST-04
- Documentación: docs/features/TASK-RUST-305/</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-005-event-system.md