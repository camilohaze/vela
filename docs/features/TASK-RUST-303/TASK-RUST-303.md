# TASK-RUST-303: Implementar Channels Asíncronos

## 📋 Información General
- **Historia:** EPIC-RUST-04 (Runtime Migration)
- **Estado:** En curso ✅
- **Fecha:** 2024-12-30
- **Dependencias:** TASK-RUST-302 (Async Runtime)

## 🎯 Objetivo
Implementar un sistema completo de channels asíncronos para el runtime de Vela, permitiendo comunicación segura y eficiente entre tareas concurrentes.

## 🔨 Arquitectura de Channels

### Diseño General
Los channels en Vela siguen el patrón de Tokio's mpsc (multi-producer, single-consumer) channels, con las siguientes características:

- **Multi-producer**: Múltiples senders pueden enviar mensajes
- **Single-consumer**: Un solo receiver consume mensajes
- **Async-first**: Todas las operaciones son asíncronas
- **Type-safe**: Channels tipados genéricamente
- **Bounded/Unbounded**: Soporte para ambos tipos

### Componentes Principales

#### 1. VelaChannel<T>
```rust
pub struct VelaChannel<T> {
    sender: VelaSender<T>,
    receiver: VelaReceiver<T>,
}
```

**Métodos principales:**
- `new(capacity: usize) -> Self` - Crear channel bounded
- `unbounded() -> Self` - Crear channel unbounded
- `split(self) -> (VelaSender<T>, VelaReceiver<T>)` - Separar sender/receiver

#### 2. VelaSender<T>
```rust
pub struct VelaSender<T> {
    inner: mpsc::Sender<T>,
}
```

**Métodos principales:**
- `send(&self, value: T) -> Result<(), SendError<T>>` - Enviar mensaje
- `try_send(&self, value: T) -> Result<(), TrySendError<T>>` - Enviar sin bloquear
- `is_closed(&self) -> bool` - Verificar si receiver cerrado
- `clone(&self) -> Self` - Clonar sender para multi-producer

#### 3. VelaReceiver<T>
```rust
pub struct VelaReceiver<T> {
    inner: mpsc::Receiver<T>,
}
```

**Métodos principales:**
- `recv(&mut self) -> Option<T>` - Recibir mensaje (async)
- `try_recv(&mut self) -> Result<T, TryRecvError>` - Recibir sin bloquear
- `close(&mut self)` - Cerrar receiver

### Utilities y Helpers

#### Channel Operations
```rust
pub mod utils {
    // Timeout operations
    pub async fn send_with_timeout<T>(
        sender: &VelaSender<T>,
        value: T,
        timeout: Duration
    ) -> Result<(), SendTimeoutError<T>> { ... }

    pub async fn recv_with_timeout<T>(
        receiver: &mut VelaReceiver<T>,
        timeout: Duration
    ) -> Result<T, RecvTimeoutError> { ... }

    // Select operations
    pub async fn select_first<T>(
        channels: Vec<&mut VelaReceiver<T>>
    ) -> (usize, T) { ... }

    // Fan-out operations
    pub fn fan_out<T: Clone>(
        receiver: VelaReceiver<T>,
        num_senders: usize
    ) -> Vec<VelaSender<T>> { ... }
}
```

## 🔧 Implementación Técnica

### Dependencies
```toml
[dependencies]
tokio = { version = "1.0", features = ["sync", "rt-multi-thread"] }
async-trait = "0.1"
thiserror = "1.0"
tracing = "0.1"
```

### Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("Channel is closed")]
    Closed,

    #[error("Channel is full")]
    Full,

    #[error("Send operation timed out")]
    SendTimeout,

    #[error("Receive operation timed out")]
    RecvTimeout,
}
```

### Integration con Runtime
```rust
// En runtime/src/lib.rs
pub mod channels {
    pub use crate::channels::*;
}

// En runtime/src/async/mod.rs - integración
impl AsyncExecutor {
    pub async fn spawn_with_channel<T, F>(
        &self,
        f: F
    ) -> (Task<T>, VelaReceiver<T>)
    where
        F: FnOnce(VelaSender<T>) -> T + Send + 'static,
        T: Send + 'static,
    {
        let (sender, receiver) = VelaChannel::unbounded().split();
        let task = self.spawn(async move {
            f(sender)
        });
        (task, receiver)
    }
}
```

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn test_basic_send_recv() {
        let (tx, mut rx) = VelaChannel::unbounded().split();

        tx.send(42).await.unwrap();
        assert_eq!(rx.recv().await, Some(42));
    }

    #[tokio::test]
    async fn test_bounded_channel() {
        let (tx, mut rx) = VelaChannel::new(1).split();

        // Fill channel
        tx.send(1).await.unwrap();

        // This should wait or fail depending on implementation
        let result = tokio::time::timeout(
            Duration::from_millis(10),
            tx.send(2)
        ).await;

        assert!(result.is_err()); // Should timeout
    }

    #[tokio::test]
    async fn test_multi_producer() {
        let (tx1, mut rx) = VelaChannel::unbounded().split();
        let tx2 = tx1.clone();

        let handle1 = tokio::spawn(async move {
            tx1.send("hello").await.unwrap();
        });

        let handle2 = tokio::spawn(async move {
            tx2.send("world").await.unwrap();
        });

        let mut messages = Vec::new();
        messages.push(rx.recv().await.unwrap());
        messages.push(rx.recv().await.unwrap());

        // Should receive both messages (order not guaranteed)
        assert_eq!(messages.len(), 2);
        assert!(messages.contains(&"hello"));
        assert!(messages.contains(&"world"));
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::async::{AsyncExecutor, VelaFuture};

    #[tokio::test]
    async fn test_channel_with_executor() {
        let executor = AsyncExecutor::new();

        let (task, mut receiver) = executor.spawn_with_channel(|sender| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            sender.send("completed").await.unwrap();
            "task_done"
        }).await;

        // Wait for message
        let message = receiver.recv().await.unwrap();
        assert_eq!(message, "completed");

        // Wait for task completion
        let result = task.await;
        assert_eq!(result, "task_done");
    }
}
```

## 📊 Métricas de Calidad

### Cobertura de Tests
- **Objetivo:** >= 80% cobertura de código
- **Métricas esperadas:**
  - Basic send/recv: ✅
  - Bounded channels: ✅
  - Multi-producer: ✅
  - Error handling: ✅
  - Utils functions: ✅
  - Integration con runtime: ✅

### Performance Benchmarks
```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use tokio::test;
    use std::time::Instant;

    #[tokio::test]
    async fn bench_channel_throughput() {
        let (tx, mut rx) = VelaChannel::unbounded().split();
        let num_messages = 100_000;

        let producer = tokio::spawn(async move {
            for i in 0..num_messages {
                tx.send(i).await.unwrap();
            }
        });

        let consumer = tokio::spawn(async move {
            let mut received = 0;
            while rx.recv().await.is_some() {
                received += 1;
                if received >= num_messages {
                    break;
                }
            }
            received
        });

        let start = Instant::now();
        let ((), received) = tokio::join!(producer, consumer);
        let duration = start.elapsed();

        println!("Processed {} messages in {:?}", received, duration);
        println!("Throughput: {:.0} msg/sec",
            received as f64 / duration.as_secs_f64());
    }
}
```

## 🔗 Referencias

### Tokio Channels
- [Tokio mpsc documentation](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html)
- [Channel patterns](https://tokio.rs/tokio/tutorial/channels)

### Design Patterns
- [Go channels](https://gobyexample.com/channels)
- [CSP (Communicating Sequential Processes)](https://en.wikipedia.org/wiki/Communicating_sequential_processes)

### Related Tasks
- **TASK-RUST-302:** Async runtime implementation
- **TASK-RUST-304:** Actor system (future)
- **TASK-RUST-305:** Worker pools (future)

## ✅ Checklist de Implementación

### Core Implementation
- [x] VelaChannel<T> struct
- [x] VelaSender<T> implementation
- [x] VelaReceiver<T> implementation
- [x] Bounded and unbounded variants
- [x] Error types and handling

### Utilities
- [x] send_with_timeout
- [x] recv_with_timeout
- [x] select_first
- [x] fan_out utility

### Integration
- [x] Runtime integration
- [x] AsyncExecutor integration
- [x] Module exports

### Testing
- [x] Unit tests (basic operations)
- [x] Integration tests (with runtime)
- [x] Error handling tests
- [x] Performance benchmarks
- [x] Coverage >= 80%

### Documentation
- [x] API documentation
- [x] Usage examples
- [x] Error handling guide
- [x] Performance considerations