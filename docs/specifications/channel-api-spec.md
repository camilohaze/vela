# Channel<T> API Specification

**Version:** 1.0.0  
**Status:** Design Phase  
**Sprint:** 19 - Workers y Channels  
**Task:** VELA-580 (TASK-051)

---

## Table of Contents

1. [Overview](#overview)
2. [API Reference](#api-reference)
3. [Usage Examples](#usage-examples)
4. [Threading Model](#threading-model)
5. [Performance Characteristics](#performance-characteristics)
6. [Error Handling](#error-handling)
7. [Best Practices](#best-practices)

---

## Overview

**Channel<T>** es el mecanismo de comunicación inter-thread thread-safe en Vela.

### Key Features

- ✅ **Type-safe:** Channel<T> es genérico y type-safe
- ✅ **Thread-safe:** Locks + condition variables
- ✅ **MPSC:** Multiple Producer Single Consumer
- ✅ **Bounded/Unbounded:** Control de backpressure
- ✅ **Auto-close:** Cuando todos los senders se dropean
- ✅ **Blocking/Non-blocking:** send/try_send, receive/try_receive
- ✅ **Async integration:** send_async/receive_async

### Design Principles

1. **"Share memory by communicating, don't communicate by sharing memory"** (Go philosophy)
2. **Type-safe ownership** (Rust model: Sender/Receiver split)
3. **Simple API** (send/receive is intuitive)
4. **Natural backpressure** (bounded channels)

---

## API Reference

### Channel Factory

#### `Channel<T>.new(capacity)`

Create new channel, returns `(Sender<T>, Receiver<T>)` tuple.

```vela
# Bounded channel (capacity=10)
(sender, receiver) = Channel<Number>.new(capacity=10)

# Unbounded channel
(sender, receiver) = Channel<String>.new()  # capacity=None
```

**Parameters:**
- `capacity: Optional<Number>` - Max buffer size. `None` = unbounded.

**Returns:**
- `(Sender<T>, Receiver<T>)` - Tuple of sender and receiver.

**Thread Safety:** ✅ Thread-safe

---

### Sender<T>

Producer side of channel. Can send values to channel.

#### `sender.send(value: T) -> Result<void, SendError>`

Send value to channel. **Blocks** if channel is full (bounded).

```vela
sender.send(42)  # Blocks if full
```

**Parameters:**
- `value: T` - Value to send

**Returns:**
- `Ok(void)` - Successfully sent
- `Err(ChannelClosed)` - Channel is closed

**Blocking:** ✅ Blocks if full (bounded channels)  
**Thread Safety:** ✅ Thread-safe

#### `sender.try_send(value: T) -> Result<void, SendError>`

Non-blocking send. Returns immediately.

```vela
result = sender.try_send(42)
match result {
    Ok(_) => print("Sent")
    Err(ChannelFull) => print("Full, retry later")
    Err(ChannelClosed) => print("Closed")
}
```

**Parameters:**
- `value: T` - Value to send

**Returns:**
- `Ok(void)` - Successfully sent
- `Err(ChannelFull)` - Channel buffer is full
- `Err(ChannelClosed)` - Channel is closed

**Blocking:** ❌ Never blocks  
**Thread Safety:** ✅ Thread-safe

#### `sender.send_async(value: T) -> Future<Result<void, SendError>>`

Async send (integrates with async/await runtime).

```vela
await sender.send_async(42)
```

**Parameters:**
- `value: T` - Value to send

**Returns:**
- `Future<Result<void, SendError>>`

**Blocking:** ❌ Returns Future immediately  
**Thread Safety:** ✅ Thread-safe

#### `sender.clone() -> Sender<T>`

Clone sender (enables MPSC - Multiple Producer Single Consumer).

```vela
sender2 = sender.clone()
sender3 = sender.clone()
# Now 3 senders can send to same channel
```

**Returns:**
- `Sender<T>` - New sender reference

**Thread Safety:** ✅ Thread-safe

#### `sender.close()`

Close channel from sender side.

```vela
sender.close()
# Further sends will fail
```

**Thread Safety:** ✅ Thread-safe  
**Idempotent:** ✅ Multiple closes OK

---

### Receiver<T>

Consumer side of channel. Can receive values from channel.

#### `receiver.receive() -> Option<T>`

Receive value from channel. **Blocks** if channel is empty.

```vela
value = receiver.receive()  # Blocks if empty
match value {
    Some(v) => print(f"Got {v}")
    None => print("Channel closed")
}
```

**Returns:**
- `Some(value)` - Received value
- `None` - Channel is closed and empty

**Blocking:** ✅ Blocks if empty  
**Thread Safety:** ✅ Thread-safe

#### `receiver.try_receive() -> Option<T>`

Non-blocking receive. Returns immediately.

```vela
result = receiver.try_receive()
match result {
    Some(value) => print(f"Got {value}")
    None => print("Empty or closed")
}
```

**Returns:**
- `Some(value)` - Received value
- `None` - Channel is empty or closed

**Blocking:** ❌ Never blocks  
**Thread Safety:** ✅ Thread-safe

#### `receiver.receive_async() -> Future<Option<T>>`

Async receive (integrates with async/await runtime).

```vela
value = await receiver.receive_async()
```

**Returns:**
- `Future<Option<T>>`

**Blocking:** ❌ Returns Future immediately  
**Thread Safety:** ✅ Thread-safe

#### `receiver.receive_timeout(timeout: Float) -> Option<T>`

Receive with timeout.

```vela
value = receiver.receive_timeout(1.0)  # Wait max 1 second
match value {
    Some(v) => print(f"Got {v}")
    None => print("Timeout or closed")
}
```

**Parameters:**
- `timeout: Float` - Timeout in seconds

**Returns:**
- `Some(value)` - Received value
- `None` - Timeout reached or channel closed

**Blocking:** ✅ Blocks up to timeout  
**Thread Safety:** ✅ Thread-safe

#### `receiver.close()`

Close channel from receiver side.

```vela
receiver.close()
# Further sends will fail
```

**Thread Safety:** ✅ Thread-safe  
**Idempotent:** ✅ Multiple closes OK

#### `receiver.len() -> Number`

Get current number of buffered values.

```vela
count = receiver.len()  # e.g., 5
```

**Returns:**
- `Number` - Count of buffered values

**Thread Safety:** ✅ Thread-safe

#### `receiver.is_empty() -> Bool`

Check if channel buffer is empty.

```vela
if receiver.is_empty() {
    print("No messages")
}
```

**Returns:**
- `Bool` - True if empty

**Thread Safety:** ✅ Thread-safe

#### `receiver.is_closed() -> Bool`

Check if channel is closed.

```vela
if receiver.is_closed() {
    print("Channel closed")
}
```

**Returns:**
- `Bool` - True if closed

**Thread Safety:** ✅ Thread-safe

---

## Usage Examples

### Example 1: Basic Send/Receive

```vela
# Create channel
(sender, receiver) = Channel<String>.new(capacity=10)

# Send value
sender.send("Hello, World!")

# Receive value
message = receiver.receive()
print(message)  # "Hello, World!"

# Close channel
sender.close()
```

### Example 2: Producer/Consumer with Workers

```vela
# Create channel
(sender, receiver) = Channel<Number>.new(capacity=100)

# Producer worker
producer = Worker.spawn(() => {
    for i in 0..1000 {
        sender.send(i)
        print(f"Produced: {i}")
    }
    sender.close()  # Signal completion
})

# Consumer worker
consumer = Worker.spawn(() => {
    while let Some(value) = receiver.receive() {
        result = process(value)
        print(f"Consumed: {result}")
    }
})

# Wait for completion
await producer
await consumer
print("Done!")
```

### Example 3: Multiple Producers (MPSC)

```vela
(sender, receiver) = Channel<String>.new(capacity=50)

# Clone sender for multiple producers
sender1 = sender
sender2 = sender.clone()
sender3 = sender.clone()

# Producer 1
producer1 = Worker.spawn(() => {
    for i in 0..100 {
        sender1.send(f"P1-{i}")
    }
})

# Producer 2
producer2 = Worker.spawn(() => {
    for i in 0..100 {
        sender2.send(f"P2-{i}")
    }
})

# Producer 3
producer3 = Worker.spawn(() => {
    for i in 0..100 {
        sender3.send(f"P3-{i}")
    }
})

# Consumer
consumer = Worker.spawn(() => {
    count = 0
    while let Some(msg) = receiver.receive() {
        print(msg)
        count = count + 1
    }
    print(f"Total received: {count}")  # 300
})

await producer1
await producer2
await producer3
sender.close()  # Close original sender
await consumer
```

### Example 4: Pipeline Pattern

```vela
# Stage 1: Load data
(sender1, receiver1) = Channel<String>.new(capacity=10)

stage1 = Worker.spawn(() => {
    for line in file.readLines() {
        sender1.send(line)
    }
    sender1.close()
})

# Stage 2: Parse data
(sender2, receiver2) = Channel<Data>.new(capacity=10)

stage2 = Worker.spawn(() => {
    while let Some(line) = receiver1.receive() {
        data = parse(line)
        sender2.send(data)
    }
    sender2.close()
})

# Stage 3: Process data
(sender3, receiver3) = Channel<Result>.new(capacity=10)

stage3 = Worker.spawn(() => {
    while let Some(data) = receiver2.receive() {
        result = process(data)
        sender3.send(result)
    }
    sender3.close()
})

# Stage 4: Save results
stage4 = Worker.spawn(() => {
    while let Some(result) = receiver3.receive() {
        saveToDatabase(result)
    }
})

# Wait pipeline completion
await stage1
await stage2
await stage3
await stage4
```

### Example 5: Non-blocking Send/Receive

```vela
(sender, receiver) = Channel<Number>.new(capacity=5)

# Fill channel
for i in 0..5 {
    sender.send(i)  # OK
}

# Try send (will fail - full)
result = sender.try_send(10)
match result {
    Ok(_) => print("Sent")
    Err(ChannelFull) => print("Channel full!")  # This branch
    Err(ChannelClosed) => print("Closed")
}

# Try receive
value = receiver.try_receive()
match value {
    Some(v) => print(f"Got {v}")  # Got 0
    None => print("Empty")
}
```

### Example 6: Receive with Timeout

```vela
(sender, receiver) = Channel<String>.new(capacity=10)

# Worker that sends after 2 seconds
worker = Worker.spawn(() => {
    time.sleep(2.0)
    sender.send("Hello")
})

# Timeout after 1 second (will fail)
value = receiver.receive_timeout(1.0)
match value {
    Some(v) => print(f"Got {v}")
    None => print("Timeout!")  # This branch
}

# Timeout after 5 seconds (will succeed)
value = receiver.receive_timeout(5.0)
match value {
    Some(v) => print(f"Got {v}")  # "Hello"
    None => print("Timeout!")
}

await worker
```

### Example 7: Async Integration

```vela
async fn processAsync() -> Result {
    (sender, receiver) = Channel<Data>.new(capacity=10)
    
    # Spawn worker to send data
    worker = Worker.spawn(() => {
        for i in 0..100 {
            data = fetchData(i)
            sender.send(data)
        }
        sender.close()
    })
    
    # Async receive loop
    results = []
    while let Some(data) = await receiver.receive_async() {
        # Process data asynchronously
        result = await processDataAsync(data)
        results.push(result)
    }
    
    await worker
    return results
}
```

---

## Threading Model

### Internal Implementation

```python
class _ChannelState[T]:
    """Shared state between Sender and Receiver"""
    
    def __init__(self, capacity: Optional[int]):
        self.buffer = deque(maxlen=capacity) if capacity else deque()
        self.capacity = capacity
        self.closed = False
        self.sender_count = 1  # Reference counting
        
        # Thread synchronization
        self.lock = threading.Lock()
        self.not_empty = threading.Condition(self.lock)
        self.not_full = threading.Condition(self.lock)
```

### Blocking Behavior

#### Send Blocking (Bounded Channel)

```python
def send(self, value: T):
    with self.lock:
        # Wait until not full
        while len(self.buffer) >= self.capacity and not self.closed:
            self.not_full.wait()  # BLOCKS HERE
        
        if self.closed:
            raise ChannelClosedError()
        
        self.buffer.append(value)
        self.not_empty.notify()  # Wake receiver
```

#### Receive Blocking

```python
def receive(self) -> Optional[T]:
    with self.lock:
        # Wait until not empty
        while len(self.buffer) == 0 and not self.closed:
            self.not_empty.wait()  # BLOCKS HERE
        
        if len(self.buffer) == 0:
            return None  # Closed and empty
        
        value = self.buffer.popleft()
        self.not_full.notify()  # Wake sender
        return value
```

### Thread Safety Guarantees

1. **Mutual Exclusion:** Only one thread modifies buffer at a time (via `self.lock`)
2. **FIFO Order:** deque preserves order
3. **No Lost Wakeups:** Condition variables ensure proper wakeup
4. **No Spurious Wakeups Handled:** while loops handle spurious wakeups

---

## Performance Characteristics

### Latency

| Operation | Latency | Notes |
|-----------|---------|-------|
| `send()` (not full) | < 100μs | Lock + buffer.append() |
| `send()` (full) | Blocks | Until space available |
| `receive()` (not empty) | < 100μs | Lock + buffer.popleft() |
| `receive()` (empty) | Blocks | Until message arrives |
| `try_send()` | < 50μs | Never blocks |
| `try_receive()` | < 50μs | Never blocks |

### Throughput

| Scenario | Throughput | Notes |
|----------|------------|-------|
| Single thread (no contention) | > 1M msgs/sec | Minimal lock overhead |
| Multi-thread (high contention) | > 100K msgs/sec | Lock contention |
| Bounded channel (full) | Backpressure | Sender blocks |
| Unbounded channel | No backpressure | Can exhaust memory |

### Memory

| Metric | Value |
|--------|-------|
| Channel overhead | ~1KB | State + locks |
| Per-message overhead | ~80 bytes | deque node |
| Bounded channel memory | `capacity * sizeof(T) + overhead` |
| Unbounded channel memory | Unbounded (can OOM) |

---

## Error Handling

### SendError

```vela
enum SendError {
    ChannelFull,    # try_send() when full
    ChannelClosed   # send() when closed
}
```

### ReceiveError

```vela
# receive() returns Option<T>
# None means closed and empty
```

### Example Error Handling

```vela
# Send with error handling
result = sender.try_send(42)
match result {
    Ok(_) => print("Success")
    Err(ChannelFull) => {
        print("Channel full, waiting...")
        sender.send(42)  # Block until space
    }
    Err(ChannelClosed) => {
        print("Channel closed, cannot send")
    }
}

# Receive with error handling
value = receiver.receive()
match value {
    Some(v) => process(v)
    None => print("Channel closed and empty")
}
```

---

## Best Practices

### 1. Prefer Bounded Channels

```vela
# ✅ GOOD: Bounded (memory bounded)
(sender, receiver) = Channel<T>.new(capacity=100)

# ⚠️ CAUTION: Unbounded (can OOM)
(sender, receiver) = Channel<T>.new()
```

**Rationale:** Bounded channels provide natural backpressure and prevent OOM.

### 2. Close Channels to Signal Completion

```vela
# ✅ GOOD: Close when done
sender.send(data)
sender.close()  # Receiver knows no more data

# ❌ BAD: Never close
sender.send(data)
# Receiver will block forever
```

### 3. Use try_send/try_receive for Non-blocking

```vela
# ✅ GOOD: Non-blocking
while let Some(msg) = receiver.try_receive() {
    process(msg)
}

# ⚠️ CAUTION: May block forever
while True {
    msg = receiver.receive()  # Blocks if empty
    process(msg)
}
```

### 4. Use MPSC for Multiple Producers

```vela
# ✅ GOOD: Clone sender
sender2 = sender.clone()
worker1 = Worker.spawn(() => sender.send(data1))
worker2 = Worker.spawn(() => sender2.send(data2))
```

### 5. Avoid Deadlocks

```vela
# ❌ BAD: Deadlock
channel = Channel<T>.new(capacity=1)
channel.send(1)  # OK
channel.send(2)  # DEADLOCK (nobody receiving)

# ✅ GOOD: Separate sender/receiver threads
(sender, receiver) = Channel<T>.new(capacity=1)
worker = Worker.spawn(() => {
    receiver.receive()
    receiver.receive()
})
sender.send(1)
sender.send(2)
await worker
```

### 6. Choose Right Capacity

```vela
# Small capacity: Low memory, more blocking
(sender, receiver) = Channel<T>.new(capacity=10)

# Large capacity: High memory, less blocking
(sender, receiver) = Channel<T>.new(capacity=10000)

# Rule of thumb: capacity = producer_rate * latency
# If producing 100/sec, latency 0.1s → capacity=10
```

### 7. Use Timeout for Robustness

```vela
# ✅ GOOD: Timeout prevents infinite blocking
value = receiver.receive_timeout(5.0)
match value {
    Some(v) => process(v)
    None => print("Timeout, retry or fail")
}

# ❌ BAD: Can block forever
value = receiver.receive()
```

---

## Summary

**Channel<T>** provides:
- ✅ Thread-safe message passing
- ✅ Type-safe communication
- ✅ MPSC support
- ✅ Bounded/unbounded variants
- ✅ Blocking/non-blocking operations
- ✅ Auto-close semantics
- ✅ Async integration

**Use cases:**
- Producer/Consumer patterns
- Pipeline processing
- Fan-out/Fan-in
- Inter-worker communication

**Next Steps:**
- Implement Channel runtime (TASK-051)
- Write comprehensive tests
- Performance benchmarks
- Integration with Workers

---

**Version:** 1.0.0  
**Last Updated:** 2025-12-02  
**References:** ADR-014, VELA-580
