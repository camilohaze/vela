# ADR-XXX: Arquitectura del Sistema de Eventos

## Estado
✅ Aceptado

## Fecha
2024-12-30

## Contexto
Necesitamos implementar un sistema de eventos genérico para comunicación desacoplada en Vela. Este sistema debe ser type-safe, eficiente y fácil de usar. Los eventos son fundamentales para arquitecturas reactivas y desacopladas.

## Decisión
Implementaremos un sistema de eventos basado en EventBus<T> con las siguientes características:

### Arquitectura General
- **EventBus<T>**: Bus central type-safe para eventos
- **EventEmitter**: Interface para objetos que emiten eventos
- **Subscription**: Tipo para manejar subscripciones con cleanup automático
- **Event<T>**: Tipo genérico para eventos con metadata

### Componentes Principales

#### 1. EventBus<T>
```rust
pub struct EventBus<T> {
    listeners: HashMap<TypeId, Vec<Box<dyn Fn(&T) + Send + Sync>>>,
    // ... campos adicionales
}
```

#### 2. EventEmitter Trait
```rust
pub trait EventEmitter<T> {
    fn emit(&self, event: T);
    fn on<F>(&self, listener: F) -> Subscription where F: Fn(&T) + Send + Sync + 'static;
}
```

#### 3. Subscription Type
```rust
pub struct Subscription {
    unsubscribe: Box<dyn FnOnce() + Send + Sync>,
}
```

### Keywords de Lenguaje
- `on event_type => handler` - Para suscripción
- `emit event` - Para emisión
- `off subscription` - Para unsubscribe

## Consecuencias

### Positivas
- **Type Safety**: Eventos strongly typed evitan errores en runtime
- **Performance**: Zero-cost abstractions con monomorphization
- **Memory Safe**: RAII para cleanup automático de subscriptions
- **Composición**: Fácil composición con el sistema reactivo
- **Escalabilidad**: Soporte para múltiples listeners por evento

### Negativas
- **Complejidad**: Sistema más complejo que callbacks simples
- **Overhead**: Pequeño overhead por type erasure en algunos casos
- **Learning Curve**: Nuevos conceptos para desarrolladores

## Alternativas Consideradas

### 1. Sistema Callback Simple
```rust
// Alternativa rechazada
pub struct SimpleEventBus {
    listeners: Vec<Box<dyn Fn()>>,
}
```
**Rechazada porque**: No type-safe, difícil de mantener, memory leaks.

### 2. Sistema Basado en Strings
```rust
// Alternativa rechazada
event_bus.emit("user:login", &user_data);
```
**Rechazada porque**: No type-safe, errores en runtime, refactoring difícil.

### 3. Sistema Actor-Based
```rust
// Considerada pero rechazada
// Complejidad alta, overhead significativo para casos simples
```

## Implementación
Ver código en: `src/event_bus.rs`

## Referencias
- Jira: VELA-595
- Documentación: docs/features/VELA-595/TASK-113A.md