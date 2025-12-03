# ADR-014: Channel<T> API Design

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

**Necesidad:** Workers (TASK-050) permiten computación paralela en threads, pero necesitan un mecanismo para **comunicación inter-thread thread-safe**.

**Problema:** 
- Shared memory + locks es error-prone (race conditions, deadlocks)
- Necesitamos abstracción de alto nivel para message passing
- Debe integrarse con Worker API y async/await runtime

**Casos de Uso:**
1. **Producer/Consumer:** Worker produce datos, otro consume
2. **Pipeline:** Chain de workers procesando datos secuencialmente
3. **Fan-out/Fan-in:** Distribuir trabajo entre N workers, recolectar resultados
4. **Pub/Sub:** Multiple workers recibiendo mismo mensaje

## Decisión

Implementamos **Channel<T>** inspirado en:
- **Go channels:** Simple API, first-class citizen
- **Rust std::sync::mpsc:** Type-safe, ownership semantics
- **Kotlin channels:** Suspendable send/receive

### API Surface

```vela
# Crear channel
channel = Channel<Number>(capacity=10)  # Bounded
unbounded = Channel<String>()  # Unbounded

# Send (blocking si full)
channel.send(42)

# Receive (blocking si empty)
value = channel.receive()  # Some(42) o None si closed

# Non-blocking variants
channel.try_send(42)  # Ok() o Err(ChannelFull)
channel.try_receive()  # Some(value) o None

# Async variants (integration con async/await)
await channel.send_async(42)
value = await channel.receive_async()

# Close channel
channel.close()

# Check state
channel.is_closed()  # Bool
channel.len()  # Current buffered count
channel.capacity()  # Max capacity (None si unbounded)
```

### Channel Types

#### 1. Bounded Channel

```vela
# Fixed capacity buffer
channel = Channel<T>(capacity=100)

# Behaviors:
# - send() blocks cuando full
# - receive() blocks cuando empty
# - Backpressure natural
```

**Pros:**
- Memory bounded (no OOM)
- Natural backpressure
- Predictable performance

**Cons:**
- Can block sender
- Needs tuning capacity

#### 2. Unbounded Channel

```vela
# Unlimited capacity
channel = Channel<T>()  # o capacity=None

# Behaviors:
# - send() nunca bloquea
# - receive() blocks cuando empty
# - No backpressure
```

**Pros:**
- Never blocks sender
- Simple to use
- No capacity tuning

**Cons:**
- Can exhaust memory
- No backpressure
- Unpredictable memory usage

### Ownership Model

**Sender/Receiver Pattern** (Rust-inspired):

```vela
# Crear channel retorna (sender, receiver)
(sender, receiver) = Channel<T>.new(capacity=10)

# Sender solo puede send
sender.send(42)
# sender.receive()  # ERROR: Sender no tiene receive()

# Receiver solo puede receive
value = receiver.receive()
# receiver.send(42)  # ERROR: Receiver no tiene send()

# Multiple senders, single receiver (MPSC)
sender2 = sender.clone()
sender3 = sender.clone()

# Cuando todos los senders se dropean, channel se cierra
```

**Ventajas:**
- Type-safe: No confundir send/receive
- Clear ownership semantics
- Facilita MPSC (Multiple Producer Single Consumer)
- Auto-close cuando no hay senders

**Alternativa Considerada:** Single Channel object con send/receive
- **Rechazada:** Menos type-safe, ownership unclear

### Blocking Behavior

#### Send Blocking

```python
# Bounded channel
channel = Channel<T>(capacity=10)

# Si channel está full:
channel.send(value)  # BLOCKS hasta que haya espacio

# Non-blocking variant:
result = channel.try_send(value)
match result:
    Ok(_) => print("Sent")
    Err(ChannelFull) => print("Full, try later")
```

#### Receive Blocking

```python
# Si channel está empty:
value = channel.receive()  # BLOCKS hasta que haya dato

# Non-blocking variant:
result = channel.try_receive()
match result:
    Some(value) => print(f"Got {value}")
    None => print("Empty or closed")
```

### Close Semantics

```python
# Close channel
channel.close()

# Después de close:
channel.send(42)  # Err(ChannelClosed)
channel.receive()  # None (si empty) o buffered values

# Auto-close cuando todos los senders se dropean
(sender, receiver) = Channel<T>.new(capacity=10)
sender = None  # Drop sender
# Channel se cierra automáticamente
```

**Reglas:**
1. Close es **idempotent** (multiple closes OK)
2. Send después de close → Error
3. Receive después de close → None (si empty) o buffered values (si quedan)
4. Auto-close cuando no quedan senders

### Select Operation

**Inspirado en Go select:**

```vela
# Wait en múltiples channels
select {
    value = <-channel1 => {
        print(f"Got from channel1: {value}")
    }
    value = <-channel2 => {
        print(f"Got from channel2: {value}")
    }
    timeout(1.0) => {
        print("Timeout after 1s")
    }
}
```

**Implementación Fase 2** (no en TASK-051):
- Select es complejo (necesita polling simultáneo)
- Defer para Sprint 20
- Por ahora: receive con timeout

### Integration con Workers

```vela
# Producer/Consumer pattern
(sender, receiver) = Channel<Number>.new(capacity=100)

# Producer worker
producer = Worker.spawn(() => {
    for i in 0..1000 {
        sender.send(i)
    }
    sender.close()  # Signal completion
})

# Consumer worker
consumer = Worker.spawn(() => {
    while let Some(value) = receiver.receive() {
        process(value)
    }
})

await producer
await consumer
```

### Integration con Async/Await

```vela
# Async send/receive
async fn processAsync() {
    channel = Channel<String>(capacity=10)
    
    # Async send (no bloquea event loop)
    await channel.send_async("hello")
    
    # Async receive
    value = await channel.receive_async()
}
```

**Implementación:**
- `send_async()` → `Future<Result<(), ChannelError>>`
- `receive_async()` → `Future<Option<T>>`
- Usa Promise internally para await

### Thread Safety

**Todo es thread-safe:**

```python
# Internal implementation
class Channel[T]:
    def __init__(self, capacity):
        self._buffer = deque(maxlen=capacity)
        self._lock = threading.Lock()
        self._not_empty = threading.Condition(self._lock)
        self._not_full = threading.Condition(self._lock)
        self._closed = False
```

**Guarantees:**
- Thread-safe send/receive
- No race conditions
- No data corruption
- FIFO order preservation

### Error Handling

```vela
# Send errors
enum SendError {
    ChannelFull,    # try_send() cuando full
    ChannelClosed   # send() cuando closed
}

# Receive errors
enum ReceiveError {
    ChannelEmpty,   # try_receive() cuando empty
    ChannelClosed   # receive() cuando closed y empty
}

# Usage
result = channel.try_send(42)
match result {
    Ok(_) => print("Sent")
    Err(ChannelFull) => print("Full")
    Err(ChannelClosed) => print("Closed")
}
```

## Consecuencias

### Positivas

✅ **Type-safe communication:** No shared memory bugs  
✅ **Simple API:** send/receive is intuitive  
✅ **Natural backpressure:** Bounded channels prevent OOM  
✅ **MPSC support:** Multiple producers, single consumer  
✅ **Auto-close:** When all senders dropped  
✅ **Integration:** Works con Workers y async/await  
✅ **Thread-safe:** Locks + condition variables  

### Negativas

⚠️ **Blocking puede causar deadlocks:**
```vela
# DEADLOCK ejemplo
channel = Channel<T>(capacity=1)
channel.send(1)  # OK
channel.send(2)  # BLOCKS forever (nadie recibiendo)
```
**Mitigación:** Docs + examples, timeouts, try_send()

⚠️ **Unbounded channels pueden exhaust memory:**
```vela
channel = Channel<T>()  # Unbounded
for i in 0..10_000_000 {
    channel.send(i)  # No backpressure, puede OOM
}
```
**Mitigación:** Prefer bounded channels, docs warning

⚠️ **Select operation compleja (defer Phase 2):**
- Necesita polling simultáneo en multiple channels
- Defer para Sprint 20

⚠️ **Performance overhead:**
- Locks en cada send/receive
- Context switching para blocking
**Mitigación:** Benchmarks, optimization si necesario

## Alternativas Consideradas

### 1. Actor Model (Erlang-style)

```vela
actor Worker {
    fn receive() -> Message {
        # Actor mailbox
    }
}
```

**Pros:**
- Isolation completo
- Location transparency

**Cons:**
- Más complejo de implementar
- Overhead mayor
- No es Go/Rust-style

**Decisión:** Rechazado - Demasiado complejo para MVP

### 2. Shared Memory + Locks

```vela
# Manual locks
mutex = Mutex<Vec<T>>()
data = mutex.lock()
data.push(value)
mutex.unlock()
```

**Pros:**
- Lower-level control
- Mejor performance (sin channel overhead)

**Cons:**
- Error-prone (deadlocks, race conditions)
- No high-level abstraction
- Hard to use correctly

**Decisión:** Rechazado - Muy low-level, queremos high-level API

### 3. Single Channel Object (no Sender/Receiver split)

```vela
channel = Channel<T>()
channel.send(42)
channel.receive()
```

**Pros:**
- API más simple
- Menos objetos

**Cons:**
- Menos type-safe
- Ownership unclear
- No auto-close cuando senders dropped

**Decisión:** Rechazado - Preferimos type-safety de Rust model

### 4. Async-only Channels (no blocking)

```vela
# Solo async variants
await channel.send(42)
value = await channel.receive()
```

**Pros:**
- No blocking threads
- Better para async/await

**Cons:**
- Workers necesitan blocking variants
- No funciona en sync contexts

**Decisión:** Rechazado - Necesitamos blocking para Workers

## Implementación

### Components

```
Channel<T> (Factory)
    ↓
(Sender<T>, Receiver<T>)
    ↓
_ChannelState<T> (Shared State)
    ↓
- buffer: deque
- lock: Lock
- not_empty: Condition
- not_full: Condition
- closed: bool
- sender_count: int
```

### Files

```
src/runtime/channels/
├── __init__.py           # Exports
├── channel.py            # Channel factory
├── sender.py             # Sender<T>
├── receiver.py           # Receiver<T>
└── channel_state.py      # Shared state

tests/unit/runtime/channels/
├── __init__.py
├── test_channel_send_receive.py      # Basic operations
├── test_channel_blocking.py          # Blocking behavior
├── test_channel_close.py             # Close semantics
├── test_channel_buffering.py         # Bounded/unbounded
└── test_channel_integration.py       # Con Workers
```

### Performance Targets

| Metric | Target |
|--------|--------|
| Send latency (unbuffered) | < 1ms |
| Send latency (buffered, not full) | < 100μs |
| Receive latency (buffered, not empty) | < 100μs |
| Throughput (single thread) | > 1M msgs/sec |
| Throughput (multi-thread) | > 100K msgs/sec |
| Memory overhead per channel | < 1KB |

## Referencias

### Inspiración

- **Go channels:** https://go.dev/tour/concurrency/2
- **Rust std::sync::mpsc:** https://doc.rust-lang.org/std/sync/mpsc/
- **Kotlin channels:** https://kotlinlang.org/docs/channels.html
- **Python queue.Queue:** https://docs.python.org/3/library/queue.html

### Documentación

- **Jira:** VELA-580 (Sprint 19)
- **TASK-051:** 48h estimadas
- **ADR-013:** Worker API (prerequisite)

### Patrones

- **Producer/Consumer:** Classic channel pattern
- **Pipeline:** Chain de transformaciones
- **Fan-out/Fan-in:** Distribute work, collect results
- **Pub/Sub:** Multiple receivers (future)

## Métricas de Éxito

- [ ] Channel<T> implementado (bounded/unbounded)
- [ ] Sender<T>/Receiver<T> con ownership semantics
- [ ] Blocking send/receive funcionales
- [ ] Non-blocking try_send/try_receive
- [ ] Close semantics correctos (auto-close)
- [ ] Thread-safe (locks + condition variables)
- [ ] 40+ tests escritos (100% pasando)
- [ ] Integration con Worker API
- [ ] Performance targets cumplidos
- [ ] Documentación completa (API + examples)

---

**Última Actualización:** 2025-12-02  
**Estado:** Aceptado  
**Siguiente:** Implementar Channel runtime (TASK-051)
