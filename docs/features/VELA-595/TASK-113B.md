# TASK-113B: Implementar EventBus<T> core

## 📋 Información General
- **Historia:** VELA-595
- **Estado:** Completada ✅
- **Fecha:** 2024-12-30

## 🎯 Objetivo
Implementar el núcleo del EventBus<T> con type safety, múltiples listeners y gestión de subscripciones.

## 🔨 Implementación

### Componentes Implementados

#### 1. EventBus<T> Struct
```rust
pub struct EventBus<T> {
    listeners: HashMap<TypeId, Vec<Box<dyn Fn(&T) + Send + Sync>>>,
}
```

**Métodos:**
- `new()` - Constructor
- `emit(&self, event: &T)` - Emitir evento a todos los listeners
- `on<F>(&mut self, listener: F) -> Subscription` - Suscribirse a eventos

#### 2. Subscription Struct
```rust
pub struct Subscription {
    _unsubscribe: Box<dyn FnOnce() + Send + Sync>,
}
```

**Características:**
- RAII pattern para cleanup automático
- Thread-safe con `Send + Sync`
- Placeholder para futura implementación de unsubscribe

#### 3. EventEmitter Trait
```rust
pub trait EventEmitter<T> {
    fn emit(&self, event: T);
    fn on<F>(&self, listener: F) -> Subscription
    where F: Fn(&T) + Send + Sync + 'static;
}
```

#### 4. Event<T> Type
```rust
pub struct Event<T> {
    pub data: T,
    pub timestamp: Instant,
    pub source: Option<String>,
}
```

### Event Types de Ejemplo
- `UserLoggedIn` - Evento de login de usuario
- `DataUpdated` - Evento de actualización de datos

### Tests Implementados
- ✅ `test_event_bus_creation()` - Creación básica
- ✅ `test_event_emission()` - Emisión y recepción de eventos
- ✅ `test_multiple_listeners()` - Múltiples listeners por evento
- ✅ `test_event_creation()` - Creación de eventos con metadata
- ✅ `test_event_with_source()` - Eventos con información de origen

## ✅ Criterios de Aceptación
- [x] EventBus<T> implementado con type safety
- [x] Soporte para múltiples listeners
- [x] Subscription type con RAII
- [x] EventEmitter trait definido
- [x] Event<T> type con metadata
- [x] Tests unitarios completos (5 tests)
- [x] Código compila sin errores

## 🔗 Referencias
- **Jira:** VELA-595
- **Código:** src/event_bus.rs
- **Historia:** VELA-595