# TASK-000H: Modelo de Concurrencia Formal de Vela

## 📋 Información General
- **Historia:** VELA-561 (Formal Specifications - Phase 0)
- **Epic:** EPIC-00B: Formal Specifications
- **Sprint:** 1
- **Estado:** Pendiente ⏳
- **Prioridad:** P0 (Crítica)
- **Estimación:** 64 horas
- **Dependencias:** TASK-000F

---

## 🎯 Objetivo

Documentar formalmente el modelo de concurrencia de Vela, incluyendo:

- **Actor message passing semantics** (semántica de paso de mensajes)
- **Signal propagation order guarantees** (orden de propagación de signals)
- **Memory visibility guarantees** (garantías de visibilidad de memoria)
- **Race condition prevention mechanisms** (prevención de race conditions)
- **Deadlock prevention guarantees** (prevención de deadlocks)

---

## 📐 Modelo de Concurrencia Formal

### 1. Actor Message Passing Semantics

#### 1.1 Modelo de Actores

**Definición formal:**
```
Actor = (State, Mailbox, Behavior)

donde:
- State: estado local privado (no compartido)
- Mailbox: cola FIFO de mensajes entrantes
- Behavior: función que procesa mensajes
```

**Propiedades:**
```
P1: Aislamiento
    ∀ actores a₁, a₂, state(a₁) ∩ state(a₂) = ∅
    (no shared mutable state)

P2: Procesamiento Secuencial
    ∀ actor a, |processing_messages(a)| ≤ 1
    (un mensaje a la vez)

P3: Orden FIFO
    send(a, m₁) before send(a, m₂) ⟹ 
    process(a, m₁) before process(a, m₂)

P4: At-Most-Once Delivery
    ∀ mensaje m, processed(m) ≤ 1
```

#### 1.2 Sintaxis en Vela

```vela
actor Counter {
  # Estado privado (no accesible desde fuera)
  state count: Number = 0
  
  # Handlers de mensajes
  fn increment() -> void {
    this.count = this.count + 1
  }
  
  fn getCount() -> Number {
    return this.count
  }
  
  fn reset() -> void {
    this.count = 0
  }
}

# Uso
counter = Counter()
counter.increment()  # Envía mensaje "increment"
counter.increment()  # Envía mensaje "increment"
result = counter.getCount()  # Envía mensaje "getCount", retorna 2
```

#### 1.3 Semántica Operacional

```
Estado de Actor:
  ActorState = (id, state, mailbox, behavior)

Envío de Mensaje:
  send(actor_id, message) →
    mailbox(actor_id).enqueue(message)

Procesamiento:
  process(actor) →
    if mailbox(actor).isEmpty() then wait
    else
      message = mailbox(actor).dequeue()
      new_state = behavior(actor.state, message)
      actor.state = new_state
```

**Reglas formales:**

```
         mailbox(a) = [m | rest]
         behavior(state(a), m) = state'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  (Actor-Process)
  ⟨a, mailbox: [m | rest], state⟩ 
    → 
  ⟨a, mailbox: rest, state'⟩


  actor_exists(a)
  message_valid(m)
━━━━━━━━━━━━━━━━━━━━━━━  (Actor-Send)
  send(a, m) → mailbox(a).append(m)
```

---

### 2. Signal Propagation Order Guarantees

#### 2.1 Reactive Dependency Graph

Vela mantiene un **DAG (Directed Acyclic Graph)** de dependencias:

```vela
state a: Number = 1
state b: Number = 2

computed c: Number {
  return a + b
}

computed d: Number {
  return c * 2
}

computed e: Number {
  return a + d
}
```

**Grafo de dependencias:**
```
    a ----+----> c -----> d -----> e
          |               ^        ^
    b ----+               |        |
                          +--------+
```

#### 2.2 Orden de Propagación (Topological Sort)

**Algoritmo:**
```
1. Cuando signal s cambia, marcar s como "dirty"
2. Propagar "dirty" a todos los dependientes (DFS)
3. Ordenar dependientes topológicamente
4. Recomputar en orden topológico
```

**Garantías:**
```
G1: Computación Mínima
    ∀ signal s, s recomputado ⟺ ∃ dependencia dirty

G2: Orden Consistente
    s₁ depends_on s₂ ⟹ recompute(s₂) before recompute(s₁)

G3: Una Vez Por Tick
    ∀ signal s, ∀ tick t, recomputed(s, t) ≤ 1
    (no glitches: valores intermedios inconsistentes)

G4: Detección de Ciclos
    ∃ ciclo en dependency_graph ⟹ compile_error
```

#### 2.3 Ejemplo de Propagación

```vela
state x: Number = 1
computed y: Number { return x * 2 }
computed z: Number { return y + 1 }

effect {
  print("z = ${z}")
}

# Frame 1:
x = 5
# Propagación:
# 1. x dirty
# 2. y dirty (depends on x)
# 3. z dirty (depends on y)
# 4. Recompute: x(5) → y(10) → z(11) → effect("z = 11")
```

**Timeline:**
```
t0: x=1, y=2, z=3
t1: x=5 (write)
t2: y=10 (recompute)
t3: z=11 (recompute)
t4: effect ejecutado

Garantía: nunca se ve y=2 con x=5 (no glitch)
```

#### 2.4 Batching de Updates

```vela
state a: Number = 0
state b: Number = 0

computed sum: Number {
  return a + b
}

effect {
  print("sum = ${sum}")
}

# Múltiples updates en mismo tick
a = 10
b = 20
a = 15

# Batch: solo una recomputación
# sum = 15 + 20 = 35
# effect ejecutado UNA VEZ: "sum = 35"
```

**Regla:**
```
∀ signal s, ∀ tick t,
  updates(s, t) = n ⟹ recompute_dependents(s, t) = 1
```

---

### 3. Memory Visibility Guarantees

#### 3.1 Sincronización en Actors

**Garantía:**
```
∀ mensajes m₁, m₂ enviados a actor a,
  send(a, m₁) happens-before send(a, m₂)
    ⟹
  efectos de m₁ visibles durante procesamiento de m₂
```

**Ejemplo:**
```vela
actor Database {
  state data: Map<String, String> = {}
  
  fn write(key: String, value: String) -> void {
    this.data[key] = value
  }
  
  fn read(key: String) -> String {
    return this.data[key]
  }
}

db = Database()
db.write("name", "Alice")  # Mensaje 1
result = db.read("name")   # Mensaje 2
# Garantía: result == "Alice" (write visible en read)
```

#### 3.2 Sincronización en Signals

**Garantía:**
```
∀ signals s₁, s₂ donde s₂ depends_on s₁,
  write(s₁) happens-before recompute(s₂)
    ⟹
  nuevo valor de s₁ visible en recomputación de s₂
```

**Ejemplo:**
```vela
state x: Number = 0

computed y: Number {
  return x + 1  # Lee x
}

x = 10  # Escribe x

# Garantía: y recomputado ve x=10, no x=0
```

#### 3.3 Memory Barriers

Vela inserta **memory barriers automáticos**:

```
Actor message processing:
  - Acquire barrier al inicio (ve todos los writes previos)
  - Release barrier al final (todos los writes visibles después)

Signal recomputation:
  - Acquire barrier antes de leer dependencias
  - Release barrier después de escribir resultado
```

---

### 4. Race Condition Prevention Mechanisms

#### 4.1 No Shared Mutable State

**Regla fundamental:**
```
∀ threads t₁, t₂,
∀ variable mutable v,
  (t₁ accede v) ∧ (t₂ accede v) ⟹ compile_error
```

**Enforcement:**

1. **Actors:** Estado privado, acceso solo vía mensajes
```vela
actor Counter {
  state count: Number = 0  # Privado, no compartible
  
  fn increment() -> void {
    this.count += 1  # OK: solo este actor accede
  }
}
```

2. **Signals:** Estado reactivo, updates automáticamente sincronizados
```vela
state counter: Number = 0  # Sincronizado automáticamente

effect {
  counter += 1  # OK: sistema reactivo garantiza no race
}
```

3. **Immutables:** Compartibles libremente
```vela
config: Config = loadConfig()  # Inmutable

async fn worker1() -> void {
  print(config.host)  # OK: inmutable, safe
}

async fn worker2() -> void {
  print(config.port)  # OK: inmutable, safe
}
```

#### 4.2 Atomics para Low-Level

```vela
counter: Atomic<Number> = Atomic(0)

async fn increment() -> void {
  counter.fetchAdd(1)  # Operación atómica
}

# Spawn 100 workers
for i in 0..100 {
  spawn(increment)
}

# Garantía: counter == 100 (no race conditions)
```

**Operaciones atómicas:**
```
fetchAdd(n)    : atomically add n
fetchSub(n)    : atomically subtract n
compareExchange(expected, new) : CAS
load(ordering) : atomic load
store(ordering): atomic store
```

---

### 5. Deadlock Prevention Guarantees

#### 5.1 No Locks en Modelo de Actores

**Propiedad fundamental:**
```
Actors no usan locks ⟹ no deadlocks posibles
```

**Razón:**
- Actores procesan mensajes secuencialmente
- No esperan locks de otros actores
- Solo esperan mensajes en mailbox

#### 5.2 Async/Await sin Bloqueo

```vela
async fn fetchUser(id: Number) -> Result<User> {
  response = await httpClient.get("/users/${id}")
  return Ok(response.body)
}

async fn fetchPosts(userId: Number) -> Result<List<Post>> {
  response = await httpClient.get("/posts?user=${userId}")
  return Ok(response.body)
}

async fn loadUserData(id: Number) -> void {
  user = await fetchUser(id)
  posts = await fetchPosts(user.id)
  
  # NO DEADLOCK: awaits no bloquean threads, solo suspend
}
```

**Garantía:**
```
await NO bloquea OS thread ⟹ no thread starvation
await solo suspend tarea actual ⟹ otros tasks continúan
```

#### 5.3 Actor Ordering

Para prevenir deadlocks circulares en requests entre actores:

**Regla:**
```
Si actor a₁ envía mensaje a a₂,
  a₂ NO DEBE enviar mensaje síncrono a a₁
  (previene ciclos de espera)
```

**Patrón recomendado: Reply Messages**
```vela
actor ServiceA {
  fn requestData(replyTo: Actor) -> void {
    data = this.computeData()
    replyTo.send("response", data)  # Async reply
  }
}

actor ServiceB {
  fn needsData() -> void {
    serviceA.requestData(this)  # Async request
    # No espera, continúa procesando
  }
  
  fn onResponse(data: Data) -> void {
    # Handler para respuesta
  }
}
```

---

### 6. Formal Verification

#### 6.1 Propiedades Verificables

**Safety Properties:**
```
S1: No data races
    ∀ memory location m, ∀ time t,
      writers(m, t) ≤ 1 ∨ readers(m, t) = 0

S2: No use-after-free
    ∀ object o, ∀ reference r a o,
      alive(o) ∨ compile_error(r)

S3: No null pointer dereference
    ∀ Option<T> opt,
      deref(opt) solo si opt = Some(_)
```

**Liveness Properties:**
```
L1: Message delivery
    send(actor, msg) ∧ actor_alive 
      ⟹ eventually process(actor, msg)

L2: Signal propagation
    write(signal_a) ∧ signal_b depends_on signal_a
      ⟹ eventually recompute(signal_b)

L3: No starvation
    ∀ actor a, ∃ mensaje m en mailbox(a)
      ⟹ eventually process(a, m)
```

#### 6.2 Model Checking

Vela permite model checking con TLA+:

```tla
--------------------- MODULE VelaActors ---------------------
VARIABLES actors, mailboxes, state

Send(actor, msg) ==
  mailboxes' = [mailboxes EXCEPT ![actor] = Append(@, msg)]

Process(actor) ==
  /\ mailboxes[actor] # <<>>
  /\ LET msg == Head(mailboxes[actor])
     IN /\ mailboxes' = [mailboxes EXCEPT ![actor] = Tail(@)]
        /\ state' = Behavior(state[actor], msg)

NoDataRace ==
  \A m \in MemoryLocations:
    Cardinality({a \in Actors: Writes(a, m)}) <= 1

Spec == Init /\ [][Next]_vars /\ Liveness
=============================================================
```

---

## 📊 Comparación con Otros Modelos

| Característica | Vela (Actors+Signals) | Erlang/Elixir | Akka | Go | Rust |
|----------------|----------------------|---------------|------|-----|------|
| **Modelo base** | Actor + Reactive | Actor | Actor | CSP | Ownership |
| **No shared state** | ✅ | ✅ | ✅ | ❌ | ✅ |
| **No data races** | ✅ (compile-time) | ✅ (runtime) | ✅ (runtime) | ❌ (runtime) | ✅ (compile-time) |
| **No deadlocks** | ✅ (no locks) | ✅ (no locks) | ⚠️ (posible) | ⚠️ (posible) | ⚠️ (posible) |
| **Reactive built-in** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Performance** | High | Medium | High | High | Very High |

---

## ✅ Criterios de Aceptación

- [x] Actor message passing semantics formalizada
- [x] Signal propagation order garantizado
- [x] Memory visibility guarantees especificadas
- [x] Race condition prevention documentado
- [x] Deadlock prevention garantizado
- [x] Propiedades formales verificables definidas
- [x] Comparación con otros modelos incluida

---

## 🔗 Referencias

### Papers Académicos
- **Actor Model:** [A Universal Modular ACTOR Formalism (Agha, 1986)](https://apps.dtic.mil/sti/citations/ADA157917)
- **Reactive Programming:** [A Survey on Reactive Programming (Bainomugisha et al., 2013)](https://dl.acm.org/doi/10.1145/2501654.2501666)
- **Memory Models:** [Foundations of the C++ Concurrency Memory Model (Boehm & Adve, 2008)](https://dl.acm.org/doi/10.1145/1375581.1375591)
- **Deadlock Prevention:** [Deadlock-Free Separation Logic (Gotsman et al., 2013)](https://www.cs.tau.ac.il/~orilahav/papers/esop13.pdf)

### Implementaciones de Referencia
- [Erlang/OTP Actor System](https://www.erlang.org/doc/design_principles/des_princ.html)
- [Akka Actors (Scala/Java)](https://doc.akka.io/docs/akka/current/typed/actors.html)
- [Swift Actors](https://docs.swift.org/swift-book/LanguageGuide/Concurrency.html#ID645)
- [Solid.js Reactivity](https://www.solidjs.com/docs/latest/api#createeffect)

---

**Estado:** ⏳ Pendiente de implementación  
**Prioridad:** P0 - Crítico para concurrency safety  
**Siguiente paso:** TASK-000I (Contratos formales de stdlib)
