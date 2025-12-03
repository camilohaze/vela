# ADR-501: Vela Concurrency System Architecture

## Estado
✅ **Aceptado** - 2025-01-15

## Contexto

Vela necesita un sistema de concurrencia robusto que combine:
- **Actor Model** (Erlang/Akka-style) para procesamiento distribuido
- **Worker Pools** para paralelización de tareas CPU-bound
- **Async Channels** para comunicación inter-thread type-safe
- **Thread Safety** garantizada en tiempo de compilación

El sistema debe ser:
- **Type-safe**: Sin data races en tiempo de compilación (Rust `Send`/`Sync`)
- **Fault-tolerant**: Supervisión de actores con estrategias de recuperación
- **Performant**: Minimal overhead, zero-copy message passing cuando sea posible
- **Scalable**: Soportar miles de actores concurrentes

## Decisión

### 1. Actor System (Erlang-inspired)

#### 1.1 Actor Trait

```rust
pub trait Actor: Send + 'static {
    type Message: Send;
    
    /// Handle incoming message
    fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>);
    
    /// Actor lifecycle hooks
    fn started(&mut self, ctx: &mut ActorContext<Self>) {}
    fn stopped(&mut self) {}
    fn restarting(&mut self, ctx: &mut ActorContext<Self>) {}
}
```

**Reglas:**
- `Send + 'static`: Actor debe ser transferible entre threads
- `Message: Send`: Mensajes deben ser thread-safe
- Mutable state interno (`&mut self`)
- Lifecycle hooks para inicialización/cleanup

#### 1.2 ActorContext

```rust
pub struct ActorContext<A: Actor> {
    address: ActorAddress<A>,
    supervisor: Option<ActorAddress<Supervisor>>,
    children: Vec<ActorAddress<dyn Actor>>,
}

impl<A: Actor> ActorContext<A> {
    /// Send message to another actor
    pub fn send<M>(&self, addr: &ActorAddress<M>, msg: M::Message)
    where
        M: Actor;
    
    /// Spawn child actor
    pub fn spawn<C: Actor>(&mut self, actor: C) -> ActorAddress<C>;
    
    /// Stop this actor
    pub fn stop(&mut self);
}
```

**Características:**
- `address`: Para enviar mensajes al mismo actor
- `supervisor`: Para notificar errores al supervisor
- `children`: Para gestionar jerarquía de actores

#### 1.3 ActorAddress (Type-safe Handle)

```rust
pub struct ActorAddress<A: Actor> {
    sender: mpsc::UnboundedSender<A::Message>,
    actor_id: ActorId,
    _phantom: PhantomData<A>,
}

impl<A: Actor> ActorAddress<A> {
    /// Send message (non-blocking)
    pub fn send(&self, msg: A::Message) -> Result<(), SendError>;
    
    /// Try to send message (returns error if mailbox full)
    pub fn try_send(&self, msg: A::Message) -> Result<(), TrySendError>;
    
    /// Send message and wait for response (async)
    pub async fn ask<R>(&self, msg: A::Message) -> Result<R, AskError>
    where
        A::Message: ResponseMessage<Response = R>;
}

impl<A: Actor> Clone for ActorAddress<A> {
    fn clone(&self) -> Self { /* cheap clone */ }
}
```

**Ventajas:**
- Type-safe: Solo mensajes del tipo correcto
- Cheap clone: Usa `Arc` internamente
- Non-blocking: `send()` nunca bloquea
- Async ask: Para request-response patterns

#### 1.4 Mailbox (FIFO Queue)

```rust
pub struct Mailbox<A: Actor> {
    receiver: mpsc::UnboundedReceiver<A::Message>,
    capacity: usize, // backpressure limit
}

impl<A: Actor> Mailbox<A> {
    pub async fn recv(&mut self) -> Option<A::Message> {
        self.receiver.recv().await
    }
    
    pub fn len(&self) -> usize {
        self.receiver.len()
    }
}
```

**Garantías:**
- **FIFO ordering**: Mensajes procesados en orden de envío
- **At-most-once delivery**: No duplicados
- **Backpressure**: Opcional con bounded channels

#### 1.5 Supervisor (Fault Tolerance)

```rust
pub enum SupervisionStrategy {
    /// Stop all actors in group
    OneForAll,
    /// Restart only failed actor
    OneForOne,
    /// Restart failed actor and actors started after it
    RestForOne,
}

pub struct Supervisor {
    strategy: SupervisionStrategy,
    max_restarts: u32,
    within_seconds: u64,
    restart_count: HashMap<ActorId, (u32, Instant)>,
}

impl Actor for Supervisor {
    type Message = SupervisorMessage;
    
    fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
        match msg {
            SupervisorMessage::ActorFailed(actor_id, error) => {
                self.handle_failure(actor_id, error, ctx);
            }
        }
    }
}
```

**Estrategias de Restart:**
1. **OneForOne**: Solo reinicia el actor fallido
2. **OneForAll**: Reinicia todos los hijos
3. **RestForOne**: Reinicia el fallido y los siguientes

**Backoff exponencial**: Espera incrementalmente entre reintentos

### 2. Worker Pools (Tokio + Rayon Integration)

#### 2.1 WorkerPool Trait

```rust
pub trait WorkerPool: Send + Sync {
    type Task: Send;
    type Result: Send;
    
    /// Submit task to pool
    fn submit(&self, task: Self::Task) -> JoinHandle<Self::Result>;
    
    /// Wait for all tasks to complete
    async fn join_all(&self) -> Vec<Self::Result>;
}
```

#### 2.2 ThreadPool (CPU-bound tasks with Rayon)

```rust
pub struct ThreadPool {
    pool: rayon::ThreadPool,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();
        
        Self { pool }
    }
    
    pub fn spawn<F, R>(&self, f: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        
        self.pool.spawn(move || {
            let result = f();
            let _ = tx.send(result);
        });
        
        JoinHandle { receiver: rx }
    }
}
```

**Características:**
- **Work stealing**: Threads roban tareas de otros threads inactivos
- **Thread-local storage**: Para optimizaciones (evita contención)
- **Panic recovery**: Threads se recrean automáticamente

#### 2.3 AsyncPool (IO-bound tasks with Tokio)

```rust
pub struct AsyncPool {
    runtime: tokio::runtime::Runtime,
}

impl AsyncPool {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(num_cpus::get())
            .enable_all()
            .build()
            .unwrap();
        
        Self { runtime }
    }
    
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }
}
```

**Características:**
- **Async runtime**: Tokio para IO-bound tasks
- **Multi-threaded**: Múltiples worker threads
- **Cooperative multitasking**: `.await` yields control

#### 2.4 Hybrid Pool (CPU + IO)

```rust
pub struct HybridPool {
    async_pool: AsyncPool,
    thread_pool: ThreadPool,
}

impl HybridPool {
    pub fn spawn_blocking<F, R>(&self, f: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.thread_pool.spawn(f)
    }
    
    pub fn spawn_async<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.async_pool.spawn(future)
    }
}
```

**Decisión de Routing:**
- CPU-bound → `ThreadPool` (Rayon)
- IO-bound → `AsyncPool` (Tokio)

### 3. Channels (Type-safe Communication)

#### 3.1 MPSC (Multi-Producer, Single-Consumer)

```rust
pub fn mpsc<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
    tokio::sync::mpsc::channel(buffer)
}

pub fn unbounded_mpsc<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    tokio::sync::mpsc::unbounded_channel()
}
```

**Uso:**
- Actor mailboxes (unbounded)
- Task queues (bounded con backpressure)

#### 3.2 Broadcast (Multi-Producer, Multi-Consumer)

```rust
pub fn broadcast<T: Clone>(capacity: usize) -> (broadcast::Sender<T>, broadcast::Receiver<T>) {
    tokio::sync::broadcast::channel(capacity)
}
```

**Uso:**
- Event bus
- Notifications (múltiples subscribers)
- Signal propagation (reactive system integration)

#### 3.3 Oneshot (Single-use channel)

```rust
pub fn oneshot<T>() -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
    tokio::sync::oneshot::channel()
}
```

**Uso:**
- Request-response patterns (`ActorAddress::ask`)
- Future results
- Cancelation tokens

#### 3.4 Watch (Single-Producer, Multi-Consumer with Last Value)

```rust
pub fn watch<T: Clone>(initial: T) -> (watch::Sender<T>, watch::Receiver<T>) {
    tokio::sync::watch::channel(initial)
}
```

**Uso:**
- Configuration updates (todos los subscribers ven cambio)
- State broadcasting
- Signal<T> integration (reactive system)

### 4. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Vela Concurrency System                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │              Actor System Layer                    │    │
│  │  ┌──────────┐    ┌──────────┐    ┌──────────┐     │    │
│  │  │ Actor A  │───▶│ Actor B  │───▶│ Actor C  │     │    │
│  │  │ Mailbox  │    │ Mailbox  │    │ Mailbox  │     │    │
│  │  └──────────┘    └──────────┘    └──────────┘     │    │
│  │        │               │               │           │    │
│  │        └───────────────┼───────────────┘           │    │
│  │                        ▼                           │    │
│  │                 ┌─────────────┐                    │    │
│  │                 │ Supervisor  │                    │    │
│  │                 │  (Restart)  │                    │    │
│  │                 └─────────────┘                    │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │              Worker Pool Layer                     │    │
│  │  ┌──────────────┐          ┌──────────────┐        │    │
│  │  │ ThreadPool   │          │  AsyncPool   │        │    │
│  │  │  (Rayon)     │          │  (Tokio)     │        │    │
│  │  │ Work Stealing│          │ Async Runtime│        │    │
│  │  └──────────────┘          └──────────────┘        │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │              Channel Layer                         │    │
│  │  ┌────────┐  ┌───────────┐  ┌─────────┐           │    │
│  │  │  MPSC  │  │ Broadcast │  │ Oneshot │           │    │
│  │  │ Bounded│  │Multi-cast │  │Single-use│          │    │
│  │  └────────┘  └───────────┘  └─────────┘           │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │        Integration with Vela Runtime               │    │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐   │    │
│  │  │  Signals   │  │   HTTP     │  │   Events   │   │    │
│  │  │ (Reactive) │  │ (Server)   │  │  (Bus)     │   │    │
│  │  └────────────┘  └────────────┘  └────────────┘   │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5. Thread Safety Guarantees

#### 5.1 Rust Type System

```rust
// Send: Tipo puede ser transferido entre threads
pub trait Send {}

// Sync: Tipo puede ser compartido entre threads (via &T)
pub trait Sync {}
```

**Reglas automáticas:**
- `Actor: Send + 'static` → Actor ejecuta en cualquier thread
- `Message: Send` → Mensajes cruzan thread boundaries
- `&T where T: Sync` → Referencia compartida es `Send`

#### 5.2 Data Race Prevention

```rust
// ✅ CORRECTO: Actor con estado mutable privado
pub struct Counter {
    count: u32, // No Sync, pero encapsulado
}

impl Actor for Counter {
    type Message = Increment;
    
    fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
        self.count += msg.amount; // Seguro: solo un thread accede
    }
}

// ❌ PROHIBIDO: Shared mutable state sin sincronización
pub struct UnsafeCounter {
    count: Rc<RefCell<u32>>, // !Send → No compila
}
```

**Garantías:**
- **No data races**: Rust rechaza código con data races en compilación
- **No deadlocks por diseño**: Actor model evita locks explícitos

### 6. Performance Characteristics

#### Actor System
- **Message send**: O(1) - lock-free mpsc
- **Message receive**: O(1) - FIFO queue
- **Actor spawn**: O(1) - spawn new task
- **Memory overhead**: ~1KB por actor (mailbox + state)

#### Worker Pools
- **Task dispatch**: O(1) - lock-free work stealing
- **Thread overhead**: ~2MB stack per thread
- **Context switch**: ~1-5μs (kernel scheduling)

#### Channels
- **MPSC send**: O(1) - lock-free ring buffer
- **Broadcast send**: O(n) - copia a n subscribers
- **Oneshot send**: O(1) - single allocation
- **Watch send**: O(1) - atomic swap

### 7. Integration with Vela Ecosystem

#### 7.1 Reactive System Integration

```rust
// Signal propagation via broadcast channels
let (signal_tx, signal_rx) = broadcast::channel(100);

// Actor subscribe to signal changes
impl Actor for ReactiveActor {
    fn handle(&mut self, msg: SignalChange, ctx: &mut ActorContext<Self>) {
        // React to signal updates
        self.recompute(msg.new_value);
    }
}
```

#### 7.2 HTTP Framework Integration

```rust
// HTTP handler as actor
pub struct HttpHandler {
    pool: ThreadPool,
}

impl Actor for HttpHandler {
    type Message = HttpRequest;
    
    fn handle(&mut self, req: Self::Message, ctx: &mut ActorContext<Self>) {
        // Dispatch CPU-bound work to thread pool
        self.pool.spawn(move || {
            process_request(req)
        });
    }
}
```

#### 7.3 Event System Integration

```rust
// Event bus via broadcast channel
pub struct EventBus {
    channel: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.channel.subscribe()
    }
    
    pub fn publish(&self, event: Event) {
        let _ = self.channel.send(event);
    }
}
```

### 8. Error Handling Strategies

#### 8.1 Actor Failure Recovery

```rust
pub enum RestartDecision {
    /// Restart actor immediately
    Restart,
    /// Stop actor permanently
    Stop,
    /// Escalate to parent supervisor
    Escalate,
}

impl Supervisor {
    fn handle_failure(&mut self, actor_id: ActorId, error: ActorError) -> RestartDecision {
        // Check restart limits
        if self.should_restart(actor_id) {
            RestartDecision::Restart
        } else {
            RestartDecision::Stop
        }
    }
}
```

#### 8.2 Backpressure Handling

```rust
// Bounded channel with backpressure
let (tx, rx) = mpsc::channel(100); // max 100 pending messages

match tx.try_send(msg) {
    Ok(()) => { /* message sent */ }
    Err(TrySendError::Full(_)) => {
        // Mailbox full → apply backpressure
        warn!("Actor mailbox full, dropping message");
    }
    Err(TrySendError::Closed(_)) => {
        // Actor stopped → handle gracefully
        error!("Actor no longer accepting messages");
    }
}
```

### 9. Testing Strategy

#### 9.1 Race Condition Tests

```rust
#[tokio::test]
async fn test_concurrent_message_handling() {
    let actor = MyActor::new();
    let addr = actor.start();
    
    // Send 1000 messages concurrently
    let handles: Vec<_> = (0..1000)
        .map(|i| {
            let addr = addr.clone();
            tokio::spawn(async move {
                addr.send(Message { value: i }).await
            })
        })
        .collect();
    
    // Wait for all messages
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify state consistency
    assert_eq!(actor.total(), 1000);
}
```

#### 9.2 Deadlock Prevention Tests

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_no_deadlocks_circular_messaging() {
    let actor_a = ActorA::new().start();
    let actor_b = ActorB::new().start();
    
    // Circular message pattern
    actor_a.send(SendTo(actor_b.clone()));
    actor_b.send(SendTo(actor_a.clone()));
    
    // Should complete without deadlock
    timeout(Duration::from_secs(5), async {
        actor_a.wait_for_completion().await;
        actor_b.wait_for_completion().await;
    })
    .await
    .expect("Deadlock detected");
}
```

#### 9.3 Supervisor Tests

```rust
#[tokio::test]
async fn test_actor_restart_on_failure() {
    let supervisor = Supervisor::new(SupervisionStrategy::OneForOne);
    let actor = FailingActor::new();
    let addr = supervisor.spawn(actor);
    
    // Trigger failure
    addr.send(Crash).await;
    
    // Wait for restart
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify actor is alive again
    assert!(addr.send(Ping).await.is_ok());
}
```

## Consecuencias

### Positivas

1. **Type Safety**: Rust previene data races en compilación
2. **Fault Tolerance**: Supervisors reinician actores automáticamente
3. **Performance**: Lock-free channels, work stealing, zero-copy messaging
4. **Scalability**: Miles de actores con minimal overhead
5. **Integration**: Fácil integración con reactive system, HTTP, events
6. **Testing**: Estrategias completas para race conditions y deadlocks

### Negativas

1. **Complexity**: Actor model añade abstracción vs threading directo
2. **Overhead**: Mailbox per actor (~1KB memory)
3. **Learning Curve**: Desarrolladores deben entender message passing
4. **Debugging**: Más difícil que código secuencial (async stack traces)

### Mitigaciones

1. **Documentation**: Ejemplos extensivos de actor patterns
2. **Tooling**: Tracing para observabilidad de actores
3. **Best Practices**: Guías de diseño de actores
4. **Testing**: Suite de tests para patrones comunes

## Alternativas Consideradas

### 1. Plain Threading (std::thread)

**Rechazado porque:**
- Requiere locks explícitos (Mutex, RwLock)
- Propenso a deadlocks
- No fault tolerance
- Difícil de escalar

### 2. Green Threads (Tokio only)

**Rechazado porque:**
- No optimizado para CPU-bound tasks
- Context switching costoso para compute-heavy
- Necesitamos hybrid (async + threads)

### 3. CSP (Go-style channels)

**Rechazado porque:**
- No supervisión automática
- No jerarquía de actores
- Menos fault tolerance que Actor Model

## Referencias

- **Erlang/OTP**: Actor model reference implementation
- **Akka**: JVM actor framework (Scala/Java)
- **Tokio**: Async runtime for Rust
- **Rayon**: Data parallelism for Rust
- **actix**: Popular Rust actor framework

## Implementación

```
concurrency/
├── src/
│   ├── actors/
│   │   ├── actor.rs         # Actor trait
│   │   ├── context.rs       # ActorContext
│   │   ├── address.rs       # ActorAddress
│   │   ├── mailbox.rs       # Mailbox implementation
│   │   └── supervisor.rs    # Supervisor strategies
│   ├── pools/
│   │   ├── thread_pool.rs   # Rayon ThreadPool
│   │   ├── async_pool.rs    # Tokio AsyncPool
│   │   └── hybrid_pool.rs   # HybridPool
│   ├── channels/
│   │   ├── mpsc.rs          # Multi-producer, single-consumer
│   │   ├── broadcast.rs     # Multi-producer, multi-consumer
│   │   ├── oneshot.rs       # Single-use channel
│   │   └── watch.rs         # State broadcasting
│   └── lib.rs               # Public exports
└── Cargo.toml
```

---

**Fecha de Creación**: 2025-01-15  
**Autor**: Vela Team  
**Sprint**: 6 (Concurrency Migration)  
**Epic**: EPIC-RUST-06
