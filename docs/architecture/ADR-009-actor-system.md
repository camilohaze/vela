# ADR-009: Actor System Architecture

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

Vela necesita un modelo de concurrencia robusto para aplicaciones modernas. Los desafíos de concurrencia incluyen:

1. **Shared Memory Problems**: Race conditions, deadlocks, data races
2. **Complex Synchronization**: Locks, mutexes, semaphores difíciles de razonar
3. **Scalability**: Threads no escalan bien (C10K problem)
4. **Error Handling**: Difícil propagar errores entre threads
5. **Location Transparency**: No hay abstracción para distribuir trabajo

**Requisitos:**
- Concurrencia segura sin shared memory
- Escalabilidad (miles de actores concurrentes)
- Fault tolerance (manejo de errores robusto)
- Location transparency (local y remoto igual)
- Integración con sistema reactivo de Vela

---

## Decisión

Implementar un **Actor System** basado en el Actor Model (Hewitt, 1973) con inspiración de Akka/Erlang pero adaptado a Vela.

### Principios del Actor Model

1. **Encapsulation**: Estado privado, no compartido
2. **Message Passing**: Comunicación solo via mensajes inmutables
3. **Asynchronous**: Send and forget (no blocking)
4. **Location Transparency**: Mismo código para local y remoto
5. **Fault Tolerance**: Supervision hierarchies

---

## Arquitectura Propuesta

### 1. Actor Instances

**Responsabilidad:** Encapsular estado y comportamiento.

```vela
actor Counter {
  // Estado privado (NUNCA accesible desde fuera)
  state count: Number = 0
  
  // Lifecycle hooks
  fn preStart() -> void {
    print("Actor starting...")
  }
  
  fn postStop() -> void {
    print("Actor stopped")
  }
  
  // Message handler (ÚNICO método público)
  fn receive(message: Message) -> void {
    match message {
      Increment => {
        this.count = this.count + 1
      }
      Decrement => {
        this.count = this.count - 1
      }
      GetCount(sender: ActorRef) => {
        sender.send(CountResult(this.count))
      }
    }
  }
}
```

**Características:**
- Estado privado (no accesible directamente)
- Single-threaded processing (un mensaje a la vez)
- No blocking operations en `receive()`
- Lifecycle hooks: `preStart()`, `postStop()`, `preRestart()`, `postRestart()`

---

### 2. Mailbox System

**Responsabilidad:** Cola de mensajes para cada actor.

**Tipos de Mailbox:**

#### A. UnboundedMailbox (Default)
```python
# Sin límite de mensajes
# Riesgo: OutOfMemory si producer >> consumer
mailbox = UnboundedMailbox()
```

#### B. BoundedMailbox
```python
# Límite fijo de mensajes
# Backpressure: rechaza mensajes cuando está lleno
mailbox = BoundedMailbox(capacity=1000)
```

#### C. PriorityMailbox
```python
# Prioriza mensajes según criterio
# System messages > High priority > Normal > Low
mailbox = PriorityMailbox(
  priority_fn = (message) => {
    match message {
      SystemMessage => 0    # Highest priority
      ImportantTask => 1
      _ => 10               # Lowest priority
    }
  }
)
```

**Garantías:**
- **FIFO ordering** (mismo sender → mismo receiver)
- **At-most-once delivery** (sin duplicados)
- **Non-blocking send** (fire and forget)

---

### 3. Message Processing Loop

**Responsabilidad:** Procesar mensajes secuencialmente.

**Algoritmo:**
```python
fn process_loop(actor: Actor):
  while actor.is_alive:
    # 1. Fetch message (blocking si vacío)
    message = actor.mailbox.receive()
    
    # 2. Process message
    try:
      actor.receive(message)
    except error:
      # 3. Handle error (notify supervisor)
      actor.supervisor.notify_error(actor, error)
    
    # 4. Check if more messages
    if actor.mailbox.is_empty():
      # Release thread (work stealing)
      yield_thread()
```

**Características:**
- Un mensaje a la vez (serialización automática)
- Error handling integrado
- Yield cuando idle (eficiencia)

---

### 4. Thread Pool Executor

**Responsabilidad:** Pool de threads para ejecutar actores.

**Diseño: Work Stealing**

```
┌─────────────────────────────────────────┐
│       Thread Pool Executor              │
│                                         │
│  Thread 1    Thread 2    Thread 3      │
│  ┌──────┐    ┌──────┐    ┌──────┐      │
│  │Queue1│    │Queue2│    │Queue3│      │
│  │ A1  │    │ A3  │    │ A5  │      │
│  │ A2  │    │ A4  │    │      │      │
│  └──────┘    └──────┘    └──────┘      │
│      │            ▲           │         │
│      │            │ steal     │         │
│      └────────────┘           │         │
│                                         │
└─────────────────────────────────────────┘
```

**Estrategia:**
1. Cada thread tiene su **propia queue** (local)
2. Si queue local vacía, **roba** trabajo de otro thread
3. Robo desde el final (LIFO) para minimizar contención
4. Dynamic sizing: grow cuando saturado, shrink cuando idle

**Configuración:**
```vela
executor = ThreadPoolExecutor(
  min_threads: 2,              # Mínimo de threads
  max_threads: 100,            # Máximo de threads
  keep_alive: Duration(60s),   # Tiempo antes de shrink
  queue_capacity: 1000         # Capacidad de cada queue
)
```

---

### 5. Actor Scheduler

**Responsabilidad:** Decidir QUÉ actor ejecutar y CUÁNDO.

**Estrategias de Scheduling:**

#### A. Fair Scheduling (Default)
```python
# Round-robin entre actores con mensajes pendientes
# Evita starvation (todos reciben tiempo)
scheduler = FairScheduler()
```

#### B. Priority Scheduling
```python
# Actores con prioridad alta primero
# Riesgo: starvation de low-priority actors
scheduler = PriorityScheduler()
```

#### C. Work-Conserving Scheduling
```python
# SIEMPRE hay un actor ejecutándose si hay trabajo
# Maximiza throughput
scheduler = WorkConservingScheduler()
```

**Algoritmo (Fair Scheduling):**
```python
fn schedule():
  ready_actors = get_actors_with_messages()
  
  # Round-robin
  for actor in ready_actors:
    # Process N messages (fairness quantum)
    process_batch(actor, quantum=10)
    
    # Yield si otros esperan
    if has_waiting_actors():
      yield_thread()
```

**Garantías:**
- **Fairness**: Todos los actores progresan
- **Starvation-free**: No hay actores bloqueados indefinidamente
- **Throughput**: Maximizar mensajes procesados/segundo

---

### 6. Actor References (ActorRef)

**Responsabilidad:** Handle para comunicarse con actores.

```vela
# ActorRef: Referencia opaca a un actor
ref: ActorRef<Counter> = spawn Counter()

# Send message (asíncrono, non-blocking)
ref.send(Increment)

# Ask pattern (request-response)
result: Future<Number> = ref.ask(GetCount)
count = await result  # Wait for response
```

**Características:**
- **Location transparency**: Mismo código para local y remoto
- **Type-safe**: `ActorRef<T>` tipado con actor type
- **Serializable**: Puede pasar por red
- **Equality**: Dos refs al mismo actor son iguales

---

### 7. Actor System Runtime

**Responsabilidad:** Gestionar lifecycle de todo el sistema.

```vela
# Create actor system
system = ActorSystem(
  name: "MySystem",
  config: ActorSystemConfig(
    executor: ThreadPoolExecutor(min_threads: 4, max_threads: 100),
    scheduler: FairScheduler(),
    default_mailbox: UnboundedMailbox()
  )
)

# Spawn actors
counter = system.spawn(Counter, name: "counter1")
logger = system.spawn(Logger, name: "logger")

# Use actors
counter.send(Increment)

# Shutdown system (graceful)
system.shutdown(timeout: Duration(10s))
```

**Responsabilidades:**
- Actor lifecycle management (spawn, stop, restart)
- Thread pool management
- Scheduler execution
- Guardian actors (supervision root)
- Configuration management

---

## Comparación con Alternativas

### Opción 1: Shared Memory + Locks (Threads tradicionales)

**Pros:**
- ✅ Familiar para desarrolladores
- ✅ Buen soporte en OS

**Contras:**
- ❌ Race conditions
- ❌ Deadlocks
- ❌ Difícil de razonar
- ❌ No escala bien

**Decisión:** ❌ Rechazado - Demasiado propenso a errores

---

### Opción 2: CSP (Communicating Sequential Processes) - Go style

**Pros:**
- ✅ Channels type-safe
- ✅ Más simple que actors
- ✅ Buen para pipelines

**Contras:**
- ❌ No location transparency
- ❌ No fault tolerance built-in
- ❌ Channels pueden causar deadlocks
- ❌ No encapsulation de estado

**Decisión:** ❌ Rechazado - Menos robusto que actors

---

### Opción 3: Async/Await + Promises (JavaScript style)

**Pros:**
- ✅ Sintaxis familiar
- ✅ Buen para I/O-bound
- ✅ Event loop eficiente

**Contras:**
- ❌ Callback hell (sin async/await)
- ❌ No fault tolerance
- ❌ No location transparency
- ❌ Single-threaded (no paralelismo real)

**Decisión:** ⚠️ Complementario - Implementar en Sprint 18 para I/O

---

### Opción 4: Actor Model (Erlang/Akka style)

**Pros:**
- ✅ Concurrency segura (no shared memory)
- ✅ Location transparency (distribuido)
- ✅ Fault tolerance (supervision)
- ✅ Scalability (millones de actores)
- ✅ Encapsulation (estado privado)

**Contras:**
- ⚠️ Curva de aprendizaje
- ⚠️ Overhead de mensajes
- ⚠️ Debugging más complejo

**Decisión:** ✅ **ELEGIDO** - Mejor balance para concurrencia moderna

---

## Consecuencias

### Positivas

1. **Concurrency Safety**: No race conditions, no deadlocks
2. **Scalability**: Miles de actores en paralelo
3. **Fault Tolerance**: Errores aislados por actor
4. **Location Transparency**: Mismo código local/remoto (futuro)
5. **Maintainability**: Código más fácil de razonar
6. **Testability**: Actores testables aisladamente

### Negativas

1. **Learning Curve**: Paradigma nuevo para muchos devs
2. **Message Overhead**: Copy de mensajes inmutables
3. **Debugging**: Stack traces menos claros
4. **Memory**: Mailboxes consumen memoria
5. **Latency**: Comunicación asíncrona añade latency

### Mitigaciones

1. **Documentación exhaustiva** con ejemplos
2. **Message pooling** para reducir allocations
3. **DevTools** para visualizar mensajes (Sprint 17)
4. **Bounded mailboxes** para evitar OOM
5. **Optimización de latency** con batching

---

## Decisiones de Diseño Clave

### 1. Single-Threaded Actor Processing

**Decisión:** Cada actor procesa UN mensaje a la vez.

**Razón:** 
- Simplifica razonamiento (no locks en actor)
- Evita race conditions dentro del actor
- Serialización automática de acceso a estado

**Trade-off:** Throughput limitado por actor (mitigado con más actores)

---

### 2. Unbounded Mailbox por Default

**Decisión:** Mailboxes sin límite por defecto.

**Razón:**
- Más simple para comenzar
- No rechaza mensajes inesperadamente
- Developer puede cambiar a bounded si necesita

**Trade-off:** Riesgo de OOM si producer >> consumer

---

### 3. Work Stealing en Thread Pool

**Decisión:** Threads roban trabajo de otros cuando están idle.

**Razón:**
- Mejor balance de carga
- Maximiza utilización de threads
- Reduce contención (robo desde final)

**Trade-off:** Complejidad de implementación

---

### 4. Fair Scheduling por Default

**Decisión:** Round-robin entre actores.

**Razón:**
- Evita starvation
- Predecible
- Bueno para mayoría de casos

**Trade-off:** No óptimo para workloads con prioridades

---

### 5. Location Transparency (Futuro)

**Decisión:** API diseñada para soportar actores remotos.

**Razón:**
- Misma API para local y remoto
- Facilita distribución posterior
- ActorRef serializable

**Trade-off:** Complejidad extra en diseño inicial

---

## Plan de Implementación

### Sprint 16 (Actor System Core)
- ✅ TASK-036: ADR-009 (este documento)
- ⏳ TASK-037: Actor instances (estado privado, receive)
- ⏳ TASK-038: Mailbox system (unbounded, bounded, priority)
- ⏳ TASK-039: Message processing loop
- ⏳ TASK-040: Thread pool executor (work stealing)
- ⏳ TASK-041: Actor scheduling (fair scheduling)

### Sprint 17 (Supervision)
- TASK-042: Supervision strategies (OneForOne, OneForAll)
- TASK-043: Restart logic (backoff, max retries)
- TASK-044: Tests de actor system

### Sprint 18 (Async/Await)
- TASK-045: Async/await semantics
- TASK-046: Async transform (CPS)
- TASK-047: Future<T> y Promise<T>
- TASK-048: Executor para futures

### Sprint 19 (Workers)
- TASK-050: Worker API (spawn, await)
- TASK-051: Channel<T> (comunicación entre threads)
- TASK-052: Tests de workers

---

## Ejemplos de Uso

### Ejemplo 1: Counter Actor (Simple)

```vela
actor Counter {
  state count: Number = 0
  
  fn receive(message: Message) -> void {
    match message {
      Increment => this.count = this.count + 1
      Decrement => this.count = this.count - 1
      GetCount(sender) => sender.send(this.count)
      Reset => this.count = 0
    }
  }
}

# Usage
counter = spawn Counter()
counter.send(Increment)
counter.send(Increment)
counter.send(GetCount(self))  # Receive: 2
```

---

### Ejemplo 2: Chat Room (Multiple Actors)

```vela
actor ChatRoom {
  state users: List<ActorRef<User>> = []
  
  fn receive(message: Message) -> void {
    match message {
      Join(user) => {
        this.users.push(user)
        broadcast(UserJoined(user.name))
      }
      
      Leave(user) => {
        this.users.remove(user)
        broadcast(UserLeft(user.name))
      }
      
      Message(text, sender) => {
        broadcast(ChatMessage(sender.name, text))
      }
    }
  }
  
  fn broadcast(message: Message) -> void {
    this.users.forEach(user => user.send(message))
  }
}

actor User {
  name: String
  
  fn receive(message: Message) -> void {
    match message {
      ChatMessage(sender, text) => {
        print("${sender}: ${text}")
      }
      UserJoined(name) => {
        print("${name} joined")
      }
      UserLeft(name) => {
        print("${name} left")
      }
    }
  }
}

# Usage
room = spawn ChatRoom()
alice = spawn User(name: "Alice")
bob = spawn User(name: "Bob")

room.send(Join(alice))
room.send(Join(bob))
room.send(Message("Hello!", alice))
```

---

### Ejemplo 3: Pipeline (Producer-Consumer)

```vela
actor Producer {
  consumer: ActorRef<Consumer>
  
  fn receive(message: Message) -> void {
    match message {
      Start => {
        (1..1000).forEach(i => {
          this.consumer.send(Work(i))
        })
        this.consumer.send(Done)
      }
    }
  }
}

actor Consumer {
  state processed: Number = 0
  
  fn receive(message: Message) -> void {
    match message {
      Work(data) => {
        # Process data
        result = process(data)
        this.processed = this.processed + 1
      }
      
      Done => {
        print("Processed ${this.processed} items")
      }
    }
  }
}

# Usage
consumer = spawn Consumer()
producer = spawn Producer(consumer: consumer)
producer.send(Start)
```

---

## Referencias

- **Hewitt, Carl (1973)**: "A Universal Modular Actor Formalism for Artificial Intelligence"
- **Erlang OTP**: https://www.erlang.org/doc/design_principles/des_princ.html
- **Akka Documentation**: https://doc.akka.io/docs/akka/current/typed/guide/introduction.html
- **Orleans (.NET)**: https://learn.microsoft.com/en-us/dotnet/orleans/
- **Ray (Python)**: https://docs.ray.io/en/latest/ray-core/actors.html

---

## Métricas de Éxito

1. **Throughput**: >= 1M mensajes/segundo (single machine)
2. **Latency**: < 1ms promedio (end-to-end)
3. **Scalability**: 10,000+ actores concurrentes
4. **Memory**: < 1KB overhead por actor
5. **Tests**: 100% cobertura de actor system

---

**Estado:** ✅ Aceptado  
**Fecha:** 2025-12-02  
**Sprint:** Sprint 16 (VELA-578)  
**Autor:** Vela Core Team
