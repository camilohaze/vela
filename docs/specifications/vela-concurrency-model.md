# Vela Concurrency Model Specification (Formal)

**Version:** 0.1.0-draft  
**Status:** Work in Progress  
**Last Updated:** 2025-11-30

---

## Table of Contents

1. [Actor Message Passing Semantics](#1-actor-message-passing-semantics)
2. [Signal Propagation Order Guarantees](#2-signal-propagation-order-guarantees)
3. [Memory Visibility Guarantees](#3-memory-visibility-guarantees)
4. [Race Condition Prevention](#4-race-condition-prevention)
5. [Deadlock Prevention](#5-deadlock-prevention)

---

## 1. Actor Message Passing Semantics

### 1.1 Actor Model Basics

**Definition:** An actor is an isolated unit of computation that:
- Has private state (no shared memory)
- Communicates only via asynchronous messages
- Processes one message at a time (sequential inbox processing)

```vela
actor Counter {
    var count: Int = 0;
    
    fn increment() {
        count += 1;
    }
    
    fn get() -> Int {
        return count;
    }
}
```

### 1.2 Message Sending Semantics

**Rule:** Message sends are asynchronous and non-blocking.

```vela
let counter = spawn Counter();
counter.send(Increment);  // Returns immediately
counter.send(Get);        // Queued after Increment
```

**Formal semantics:**

```
⟨actor.send(msg), σ⟩ → ⟨(), σ[mailbox(actor) ← mailbox(actor) ++ [msg]]⟩
```

**Guarantees:**
1. Messages are delivered in FIFO order per sender
2. No global ordering guarantee across different senders
3. At-most-once delivery (no automatic retries)

### 1.3 Message Processing

**Rule:** Actors process messages sequentially from their mailbox.

```
while mailbox.has_messages():
    msg = mailbox.dequeue()
    handle(msg)
```

**Guarantee:** No data races within actor state (single-threaded execution per actor).

### 1.4 Actor Lifecycle

```
Created → Running → Suspended → Terminated
```

**States:**
- **Created**: Actor spawned but not started
- **Running**: Processing messages
- **Suspended**: Waiting for messages
- **Terminated**: No longer accepts messages

---

## 2. Signal Propagation Order Guarantees

### 2.1 Signal Model

**Definition:** Signals are reactive state containers that automatically notify dependents on change.

```vela
let count = signal(0);
let doubled = computed(() => count.get() * 2);

count.set(5);  // Automatically updates `doubled` to 10
```

### 2.2 Propagation Order

**Guarantee:** Signals propagate changes in topological order (dependencies before dependents).

**Example:**
```vela
let a = signal(1);
let b = computed(() => a.get() + 1);  // b depends on a
let c = computed(() => b.get() + 1);  // c depends on b

a.set(10);
// Propagation order: a → b → c
```

**Formal guarantee:**

```
If c depends on b and b depends on a, then:
    update(a) happens-before update(b) happens-before update(c)
```

### 2.3 Batching

**Rule:** Multiple signal updates in same microtask are batched.

```vela
let a = signal(1);
let b = signal(2);
let sum = computed(() => a.get() + b.get());

a.set(10);
b.set(20);  // `sum` recomputes only once with both changes
```

**Guarantee:** Effects run after all signal updates in current batch complete.

---

## 3. Memory Visibility Guarantees

### 3.1 Cross-Actor Visibility

**Rule:** Message reception establishes happens-before relationship.

```vela
// Actor A
actor.state = 42;
other_actor.send(Notify);  // (Release)

// Actor B (other_actor)
fn handle(Notify) {
    // Guaranteed to see actor.state = 42 if accessed via message data
}  // (Acquire)
```

**Guarantee:** Sender's memory writes are visible to receiver after message reception.

### 3.2 Signal Visibility

**Rule:** Signal updates use SeqCst ordering.

```vela
signal.set(value)  // SeqCst write
let x = signal.get()  // SeqCst read
```

**Guarantee:** All threads see signal updates in same global order.

---

## 4. Race Condition Prevention

### 4.1 Type System Enforcement

**Rule:** Data accessible from multiple threads must be `Send + Sync`.

```vela
actor MyActor {
    var data: Vec<Int>;  // OK: Vec<Int> is Send
    // var data: Rc<Int>;  // ERROR: Rc is not Send
}
```

### 4.2 Actor Isolation

**Guarantee:** Actor state is never directly accessible from other actors.

```vela
actor Counter {
    var count: Int = 0;  // Private, inaccessible from outside
}

let counter = spawn Counter();
// counter.count = 10;  // ERROR: cannot access actor state directly
counter.send(SetCount(10));  // OK: via message
```

### 4.3 Immutable Sharing

**Rule:** Immutable data can be safely shared across actors.

```vela
let shared_data = [1, 2, 3];  // Immutable array

actor A { /* can read shared_data */ }
actor B { /* can read shared_data */ }
```

**Guarantee:** No data races on immutable data.

---

## 5. Deadlock Prevention

### 5.1 Deadlock Scenarios

**Common causes:**
1. Circular wait (A waits for B, B waits for A)
2. Synchronous request-response in actors
3. Holding locks while waiting

### 5.2 Prevention Strategies

#### 5.2.1 Asynchronous-Only Communication

**Rule:** Actors never block waiting for responses.

```vela
// BAD (hypothetical blocking):
// let result = actor.call_sync(Compute);  // Would deadlock if actor calls back

// GOOD (async):
actor.send(Compute);
actor.send(WithResponse(callback));
```

#### 5.2.2 No Nested Locks

**Rule:** Vela's actor model doesn't expose locks to user code.

**Guarantee:** Deadlocks from lock ordering are impossible.

#### 5.2.3 Supervision Trees

**Rule:** Parent actors supervise child actors and can break deadlocks by restarting.

```vela
supervisor {
    strategy: RestartOne,
    children: [
        actor_a,
        actor_b,
    ]
}
```

**Guarantee:** If actor deadlocks, supervisor can detect timeout and restart.

### 5.3 Liveness Guarantee

**Theorem:** Vela's actor system is livelock-free under these conditions:

1. Message sends don't block
2. Actors don't busy-wait
3. Finite message processing time

**Proof sketch:**
- Each actor makes progress processing inbox
- No circular blocking (async communication)
- Fairness: scheduler gives each actor CPU time

---

## Appendix A: Actor Patterns

### A.1 Request-Response

```vela
actor Server {
    fn handle(Request(data, reply_to)) {
        let result = process(data);
        reply_to.send(Response(result));
    }
}

let server = spawn Server();
let (sender, receiver) = channel();
server.send(Request(data, sender));
let result = receiver.recv();  // Blocking wait OK here (not in actor)
```

### A.2 Pub-Sub

```vela
actor PubSub {
    var subscribers: List<Actor> = [];
    
    fn subscribe(actor: Actor) {
        subscribers.push(actor);
    }
    
    fn publish(event: Event) {
        for sub in subscribers {
            sub.send(event);
        }
    }
}
```

### A.3 Worker Pool

```vela
supervisor WorkerPool {
    strategy: RestartOne,
    children: (1..10).map(|_| spawn Worker()),
    
    fn dispatch(task: Task) {
        let worker = children.next_available();
        worker.send(task);
    }
}
```

---

## Appendix B: Formal Semantics

### B.1 Actor State Machine

```
State := (mailbox: Queue<Msg>, state: S, status: Status)

Status := Created | Running | Suspended | Terminated

Transitions:
    Created --spawn--> Running
    Running --no_messages--> Suspended
    Suspended --message_arrives--> Running
    Running --terminate--> Terminated
```

### B.2 Message Delivery

```
⟨send(actor, msg), σ⟩ → ⟨(), σ'⟩
where σ' = σ[actor.mailbox ← σ(actor.mailbox) ++ [msg]]

⟨receive(actor), σ⟩ → ⟨msg, σ'⟩
where σ(actor.mailbox) = [msg | rest]
      σ' = σ[actor.mailbox ← rest]
```

### B.3 Signal Propagation

```
⟨signal.set(v), σ⟩ → ⟨(), σ'⟩
where σ' = σ[signal.value ← v]
           [notify_dependents(signal)]

notify_dependents(signal):
    for dep in signal.dependents (topological order):
        dep.recompute()
```

---

**References:**
- Hewitt Actor Model: https://en.wikipedia.org/wiki/Actor_model
- Erlang Actor System: https://www.erlang.org/doc/reference_manual/processes.html
- Akka Documentation: https://doc.akka.io/docs/akka/current/typed/actors.html

---

*Document generated for Sprint 1 (TASK-000H)*  
*Historia: VELA-561 (US-00B)*  
*Last updated: 2025-11-30*
