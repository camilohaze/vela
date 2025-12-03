# ADR-301: Arquitectura del Crate vela-runtime

## Estado
✅ Aceptado

## Fecha
2025-12-03

## Contexto
Después de completar la migración del sistema de tipos (EPIC-RUST-03), necesitamos diseñar la arquitectura del runtime de Vela en Rust. El runtime es el núcleo de ejecución que maneja:

- **Async/Await**: Executor, futures, promises
- **Concurrencia**: Channels, actores, workers
- **DI Container**: Inyección de dependencias
- **Event System**: Bus de eventos y handlers
- **HTTP Framework**: Servidor/cliente HTTP

El runtime debe ser modular, eficiente y seguro, aprovechando las fortalezas de Rust (ownership, borrowing, zero-cost abstractions).

## Decisión
Implementar una arquitectura modular del runtime con los siguientes componentes:

### 1. Async Runtime (`async/`)
- **Executor**: Tokio-based con work-stealing scheduler
- **Futures**: Zero-cost futures con pinning
- **Promises**: API compatible con JavaScript promises
- **Tasks**: Lightweight green threads

### 2. Concurrency (`concurrency/`)
- **Channels**: MPSC channels con backpressure
- **Actors**: Actor model con mailboxes
- **Workers**: Thread pools con load balancing

### 3. DI Container (`di/`)
- **Container**: Singleton container con scopes
- **Providers**: Factory functions y singletons
- **Injection**: Constructor injection y property injection

### 4. Event System (`events/`)
- **Bus**: Publish-subscribe event bus
- **Handlers**: Async event handlers
- **Middleware**: Event processing pipeline

### 5. HTTP Framework (`http/`)
- **Server**: Async HTTP server (Hyper-based)
- **Client**: HTTP client con connection pooling
- **Router**: Trie-based routing
- **Middleware**: Request/response pipeline

### 6. Core Runtime (`core/`)
- **Runtime**: Main runtime struct
- **Lifecycle**: Startup/shutdown hooks
- **Configuration**: Runtime configuration
- **Metrics**: Performance monitoring

## Consecuencias

### Positivas
- **Modularidad**: Componentes independientes y testeables
- **Performance**: Zero-cost abstractions aprovechando Rust
- **Safety**: Memory safety garantizada por el compilador
- **Scalability**: Arquitectura preparada para alta concurrencia
- **Ecosystem**: Compatible con crates existentes (Tokio, Hyper)

### Negativas
- **Complejidad**: Mayor complejidad inicial vs runtime monolítico
- **Dependencias**: Más crates externos (Tokio, Hyper, etc.)
- **Learning Curve**: Nuevos conceptos (ownership, borrowing)
- **Migration Cost**: Reescribir lógica existente de Python

## Alternativas Consideradas

### 1. Runtime Monolítico
**Descripción**: Un solo crate con todo el runtime
**Rechazada porque**: Difícil de mantener y testear

### 2. Usar Runtime Existente (Tokio)
**Descripción**: Extender Tokio directamente
**Rechazada porque**: No cubre DI, events, HTTP específicos de Vela

### 3. Runtime en Python con Bindings
**Descripción**: Mantener Python runtime con FFI
**Rechazada porque**: Pierde beneficios de Rust (safety, performance)

## Implementación
El crate `vela-runtime` se estructura como:

```
vela-runtime/
├── src/
│   ├── lib.rs           # Re-exports principales
│   ├── core/            # Core runtime
│   ├── async/           # Async runtime
│   ├── concurrency/     # Concurrency primitives
│   ├── di/              # DI container
│   ├── events/          # Event system
│   └── http/            # HTTP framework
├── benches/             # Benchmarks
└── tests/               # Integration tests
```

### API Principal

```rust
use vela_runtime::{Runtime, RuntimeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuntimeConfig::default();
    let runtime = Runtime::new(config).await?;
    
    // Registrar servicios
    runtime.register_service(MyService::new()).await?;
    
    // Iniciar runtime
    runtime.start().await?;
    
    Ok(())
}
```

## Referencias
- **Epic:** EPIC-RUST-04
- **Tarea:** TASK-RUST-301
- **Dependencias:** EPIC-RUST-03 completada
- **Tecnologías:** Tokio, Hyper, async-std
- **Patrones:** Actor model, DI, Event-driven