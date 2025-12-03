# Especificación Formal: Async/Await en Vela

**Versión**: 1.0.0  
**Fecha**: 2025-12-02  
**Estado**: ✅ Aceptado  
**Autor**: Language Design Team

---

## 1. Introducción

Esta especificación define formalmente el comportamiento de async/await en Vela, incluyendo:

1. **Sintaxis**: Gramática formal de `async fn` y `await`
2. **Semántica Operacional**: Reglas de ejecución step-by-step
3. **Sistema de Tipos**: Type rules para `Future<T>`, `Promise<T>`
4. **Transformación**: Desugaring de async/await a state machines (CPS)
5. **Executor**: Modelo de ejecución (event loop)
6. **Interacción con Features**: Integración con actors, signals, error handling

---

## 2. Sintaxis Formal (EBNF)

### 2.1. Async Function Declaration

```ebnf
AsyncFunction ::= "async" "fn" Identifier "(" Parameters? ")" "->" Type Block

Parameters ::= Parameter ("," Parameter)*
Parameter ::= Identifier ":" Type

Block ::= "{" Statement* "}"
```

**Ejemplos**:
```vela
async fn fetchData() -> Future<String> { ... }
async fn processUser(id: Number) -> Result<User, Error> { ... }
```

### 2.2. Await Expression

```ebnf
AwaitExpr ::= "await" Expression

Expression ::= AwaitExpr
             | FunctionCall
             | BinaryOp
             | Identifier
             | Literal
             | ...
```

**Ejemplos**:
```vela
data = await fetchData()
user = await getUser(123).map(u => u.name)
result = (await compute()) + 10
```

### 2.3. Future Type

```ebnf
FutureType ::= "Future" "<" Type ">"
PromiseType ::= "Promise" "<" Type ">"

Type ::= FutureType
       | PromiseType
       | "Result" "<" Type "," Type ">"
       | ...
```

---

## 3. Semántica Operacional

### 3.1. Typing Rules (Type System)

#### Rule 1: Async Function Type
```
Γ ⊢ body : T
────────────────────────────────
Γ ⊢ async fn f() -> T : Future<T>
```

**Interpretación**: Si el cuerpo de la función retorna `T`, entonces la función async retorna `Future<T>`.

#### Rule 2: Await Expression Type
```
Γ ⊢ expr : Future<T>
────────────────────────
Γ ⊢ await expr : T
```

**Interpretación**: Si `expr` es de tipo `Future<T>`, entonces `await expr` retorna `T`.

#### Rule 3: Await Only in Async Context
```
Γ ⊢ expr : Future<T>    (current_context ≠ async)
──────────────────────────────────────────────────
Γ ⊢ await expr : ⊥  (Type Error)
```

**Interpretación**: `await` solo puede usarse dentro de `async fn` (error de compilación si no).

### 3.2. Execution Semantics (Operational Semantics)

#### State Machine Representation

Un `async fn` se transforma en un **state machine** con estados:

```
State ::= Start                          # Estado inicial
        | Await(Future<T>, Continuation) # Esperando Future
        | Done(T)                        # Completado con valor T
        | Error(E)                       # Error
```

#### Evaluation Rules

**Rule 1: Start State**
```
⟨Start, async fn f() { await e1; s2 }⟩ → ⟨Await(e1, λx.s2), poll⟩
```

**Interpretación**: Al iniciar, evalúa el primer `await` y entra en estado `Await`.

**Rule 2: Await State (Future Ready)**
```
⟨Await(Future<T>, cont), poll⟩ → ⟨cont(value), poll⟩
  if Future.poll() = Ready(value)
```

**Interpretación**: Si el Future está listo, continúa con el valor.

**Rule 3: Await State (Future Pending)**
```
⟨Await(Future<T>, cont), poll⟩ → ⟨Await(Future<T>, cont), suspend⟩
  if Future.poll() = Pending
```

**Interpretación**: Si el Future no está listo, suspende la ejecución (registra waker).

**Rule 4: Done State**
```
⟨Done(value), _⟩ → value
```

**Interpretación**: Estado final, retorna el valor.

### 3.3. Example: Step-by-Step Execution

**Código Original**:
```vela
async fn example() -> Number {
  x = await fetchNumber()  # await 1
  y = await fetchNumber()  # await 2
  return x + y
}
```

**Ejecución**:
```
Step 1: ⟨Start, example()⟩
  → Evalúa fetchNumber()
  → ⟨Await(Future1, λx. { y = await fetchNumber(); return x + y }), poll⟩

Step 2: Poll Future1
  → Future1.poll() = Pending
  → ⟨Await(Future1, cont), suspend⟩
  → Executor registra waker y pasa a otra tarea

Step 3: (Después de tiempo) Waker despierta
  → Future1.poll() = Ready(10)
  → ⟨cont(10), poll⟩  # x = 10
  → Evalúa fetchNumber()
  → ⟨Await(Future2, λy. return 10 + y), poll⟩

Step 4: Poll Future2
  → Future2.poll() = Pending
  → ⟨Await(Future2, cont), suspend⟩

Step 5: (Después de tiempo) Waker despierta
  → Future2.poll() = Ready(5)
  → ⟨cont(5), poll⟩  # y = 5
  → return 10 + 5
  → ⟨Done(15), _⟩

Step 6: Completado
  → 15
```

---

## 4. Type System Details

### 4.1. Future<T> Interface

```vela
interface Future<T> {
  # Poll the future (usado por executor)
  fn poll(self, waker: Waker) -> Poll<T>
  
  # Map transformation
  fn map<U>(self, f: (T) -> U) -> Future<U>
  
  # FlatMap (monadic bind)
  fn flatMap<U>(self, f: (T) -> Future<U>) -> Future<U>
  
  # Then combinator
  fn then<U>(self, callback: (T) -> U) -> Future<U>
  
  # Catch error
  fn catch(self, callback: (Error) -> T) -> Future<T>
  
  # Combine futures
  fn and<U>(self, other: Future<U>) -> Future<(T, U)>
  
  # Alternative future
  fn or(self, other: Future<T>) -> Future<T>
}
```

**Leyes (Functor)**:
```
# Identity
future.map(x => x) = future

# Composition
future.map(f).map(g) = future.map(x => g(f(x)))
```

**Leyes (Monad)**:
```
# Left identity
return(a).flatMap(f) = f(a)

# Right identity
m.flatMap(return) = m

# Associativity
m.flatMap(f).flatMap(g) = m.flatMap(x => f(x).flatMap(g))
```

### 4.2. Promise<T> API

```vela
class Promise<T> {
  state: PromiseState<T>  # Pending | Resolved(T) | Rejected(Error)
  wakers: List<Waker>     # Wakers esperando
  
  # Constructor
  constructor() {
    this.state = PromiseState::Pending
    this.wakers = []
  }
  
  # Obtener Future
  fn future(self) -> Future<T> {
    return Future::fromPromise(this)
  }
  
  # Resolver (solo una vez)
  fn resolve(self, value: T) -> void {
    if this.state != Pending {
      throw Error("Promise already settled")
    }
    this.state = Resolved(value)
    this.wakers.forEach(w => w.wake())
  }
  
  # Rechazar (solo una vez)
  fn reject(self, error: Error) -> void {
    if this.state != Pending {
      throw Error("Promise already settled")
    }
    this.state = Rejected(error)
    this.wakers.forEach(w => w.wake())
  }
}

enum PromiseState<T> {
  Pending
  Resolved(T)
  Rejected(Error)
}
```

---

## 5. Transformación CPS (Continuation Passing Style)

### 5.1. Desugaring Simple

**Código Original**:
```vela
async fn simple() -> Number {
  x = await fetchNumber()
  return x + 1
}
```

**Desugaring (Pseudo-código interno)**:
```vela
fn simple() -> Future<Number> {
  promise = Promise<Number>()
  
  # State machine con 2 estados
  state = State::Start
  
  fn resume() -> void {
    match state {
      State::Start => {
        future = fetchNumber()
        state = State::Await1(future)
        
        future.then(x => {
          state.x = x
          resume()
        })
      }
      
      State::Await1(x) => {
        result = x + 1
        promise.resolve(result)
      }
    }
  }
  
  resume()
  return promise.future()
}
```

### 5.2. Desugaring con Múltiples Awaits

**Código Original**:
```vela
async fn multiple() -> Number {
  a = await fetch1()
  b = await fetch2()
  c = await fetch3()
  return a + b + c
}
```

**Desugaring**:
```vela
fn multiple() -> Future<Number> {
  promise = Promise<Number>()
  
  enum LocalState {
    Start
    Await1(Future<Number>)
    Await2(a: Number, Future<Number>)
    Await3(a: Number, b: Number, Future<Number>)
  }
  
  state = LocalState::Start
  
  fn resume() -> void {
    match state {
      LocalState::Start => {
        future_a = fetch1()
        state = LocalState::Await1(future_a)
        future_a.then(a => resume())
      }
      
      LocalState::Await1(a) => {
        future_b = fetch2()
        state = LocalState::Await2(a, future_b)
        future_b.then(b => resume())
      }
      
      LocalState::Await2(a, b) => {
        future_c = fetch3()
        state = LocalState::Await3(a, b, future_c)
        future_c.then(c => resume())
      }
      
      LocalState::Await3(a, b, c) => {
        result = a + b + c
        promise.resolve(result)
      }
    }
  }
  
  resume()
  return promise.future()
}
```

### 5.3. Desugaring con Control Flow

**Código Original (if statement)**:
```vela
async fn conditional(flag: Bool) -> String {
  if flag {
    x = await fetchA()
    return x
  } else {
    y = await fetchB()
    return y
  }
}
```

**Desugaring**:
```vela
fn conditional(flag: Bool) -> Future<String> {
  promise = Promise<String>()
  
  enum LocalState {
    Start
    IfTrue(Future<String>)
    IfFalse(Future<String>)
  }
  
  state = LocalState::Start
  
  fn resume() -> void {
    match state {
      LocalState::Start => {
        if flag {
          future = fetchA()
          state = LocalState::IfTrue(future)
          future.then(x => resume())
        } else {
          future = fetchB()
          state = LocalState::IfFalse(future)
          future.then(y => resume())
        }
      }
      
      LocalState::IfTrue(x) => {
        promise.resolve(x)
      }
      
      LocalState::IfFalse(y) => {
        promise.resolve(y)
      }
    }
  }
  
  resume()
  return promise.future()
}
```

---

## 6. Executor Model

### 6.1. Core Executor API

```vela
class Executor {
  # Cola de tareas listas para ejecutar
  ready_queue: Queue<Task>
  
  # Tareas esperando (con wakers)
  waiting: Map<TaskId, Task>
  
  # Waker registry
  wakers: Map<TaskId, Waker>
  
  # Spawn nueva tarea async
  fn spawn<T>(self, future: Future<T>) -> TaskHandle<T> {
    task = Task::new(future)
    this.ready_queue.push(task)
    return TaskHandle::new(task.id)
  }
  
  # Event loop principal
  fn run(self) -> void {
    while !this.ready_queue.isEmpty() || !this.waiting.isEmpty() {
      # Procesar tareas listas
      if let Some(task) = this.ready_queue.pop() {
        this.pollTask(task)
      }
      
      # Esperar eventos I/O si no hay tareas listas
      if this.ready_queue.isEmpty() {
        this.waitForIO()
      }
    }
  }
  
  # Poll una tarea
  fn pollTask(self, task: Task) -> void {
    waker = Waker::new(task.id, this)
    
    match task.future.poll(waker) {
      Poll::Ready(value) => {
        task.complete(value)
        this.waiting.remove(task.id)
      }
      Poll::Pending => {
        this.waiting.set(task.id, task)
        this.wakers.set(task.id, waker)
      }
    }
  }
  
  # Esperar eventos I/O (epoll/kqueue/IOCP)
  fn waitForIO(self) -> void {
    events = this.ioPoller.wait(timeout: 100)
    
    events.forEach(event => {
      taskId = event.taskId
      if let Some(task) = this.waiting.get(taskId) {
        this.ready_queue.push(task)
      }
    })
  }
}
```

### 6.2. Waker API

```vela
class Waker {
  taskId: TaskId
  executor: Executor
  
  # Despertar tarea (llamado por I/O o timers)
  fn wake(self) -> void {
    if let Some(task) = this.executor.waiting.get(this.taskId) {
      this.executor.ready_queue.push(task)
      this.executor.waiting.remove(this.taskId)
    }
  }
}
```

### 6.3. Task Representation

```vela
class Task {
  id: TaskId
  future: Future<_>
  state: TaskState
  
  enum TaskState {
    Ready      # Listo para poll
    Waiting    # Esperando I/O
    Completed  # Terminado
  }
}
```

---

## 7. Combinadores de Futures

### 7.1. Future.all() - Ejecutar en Paralelo

```vela
# Implementación
fn all<T>(futures: List<Future<T>>) -> Future<List<T>> {
  promise = Promise<List<T>>()
  results: List<Option<T>> = futures.map(_ => None)
  completed = 0
  
  futures.forEachIndexed((i, future) => {
    future.then(value => {
      results[i] = Some(value)
      completed = completed + 1
      
      if completed == futures.length {
        promise.resolve(results.map(opt => opt.unwrap()))
      }
    })
  })
  
  return promise.future()
}

# Uso
async fn example() -> List<String> {
  results = await Future.all([
    fetch("url1"),
    fetch("url2"),
    fetch("url3")
  ])
  return results
}
```

### 7.2. Future.race() - Primero en Completar

```vela
# Implementación
fn race<T>(futures: List<Future<T>>) -> Future<T> {
  promise = Promise<T>()
  settled = false
  
  futures.forEach(future => {
    future.then(value => {
      if !settled {
        settled = true
        promise.resolve(value)
      }
    })
  })
  
  return promise.future()
}

# Uso
async fn example() -> String {
  fastest = await Future.race([
    fetchFromServer1(),
    fetchFromServer2()
  ])
  return fastest
}
```

### 7.3. Future.timeout() - Con Timeout

```vela
# Implementación
fn timeout<T>(future: Future<T>, ms: Number) -> Future<Result<T, TimeoutError>> {
  promise = Promise<Result<T, TimeoutError>>()
  timedOut = false
  
  # Registrar timeout
  timer = setTimeout(() => {
    if !timedOut {
      timedOut = true
      promise.resolve(Err(TimeoutError("Timeout after ${ms}ms")))
    }
  }, ms)
  
  # Ejecutar future
  future.then(value => {
    if !timedOut {
      timedOut = true
      clearTimeout(timer)
      promise.resolve(Ok(value))
    }
  })
  
  return promise.future()
}

# Uso
async fn example() -> Result<String, TimeoutError> {
  data = await Future.timeout(
    fetchData(),
    5000  # 5 segundos
  )
  return data
}
```

---

## 8. Integración con Actors

### 8.1. Async Message Handlers

```vela
actor AsyncWorker {
  # Handler asíncrono
  async fn handleMessage(self, msg: Message) -> void {
    match msg {
      FetchData(url) => {
        data = await httpClient.get(url)
        this.send(self, DataReceived(data))
      }
      
      ProcessData(data) => {
        result = await heavyComputation(data)
        this.send(self, Processed(result))
      }
    }
  }
}
```

### 8.2. Mailbox NO Bloqueante

**Garantías**:
1. ✅ Si un handler suspende (await), el mailbox NO se bloquea
2. ✅ Actor procesa siguiente mensaje mientras espera Future
3. ✅ Orden de mensajes al mismo actor se preserva

**Implementación (pseudo-código)**:
```vela
class ActorMailbox {
  queue: Queue<Message> = []
  activeHandlers: Set<TaskId> = {}
  
  fn enqueue(self, msg: Message) -> void {
    this.queue.push(msg)
    this.processNext()
  }
  
  fn processNext(self) -> void {
    if let Some(msg) = this.queue.pop() {
      # Spawn async handler (NO bloqueante)
      taskId = executor.spawn(this.actor.handleMessage(msg))
      this.activeHandlers.add(taskId)
      
      # Handler completado
      taskId.onComplete(() => {
        this.activeHandlers.remove(taskId)
      })
    }
  }
}
```

---

## 9. Manejo de Errores

### 9.1. Try/Catch en Async

```vela
async fn safeFetch() -> Result<String, Error> {
  try {
    data = await fetchData()  # Puede fallar
    return Ok(data)
  } catch (e: NetworkError) {
    return Err(e)
  }
}
```

### 9.2. Operador ? (Propagación)

```vela
async fn propagate() -> Result<String, Error> {
  user = await fetchUser(123)?     # Propaga error si Err
  orders = await fetchOrders(user.id)?
  return Ok(process(orders))
}
```

**Desugaring del `?`**:
```vela
# Original: x = await fetch()?
# Desugaring:
result = await fetch()
match result {
  Ok(value) => x = value
  Err(e) => return Err(e)
}
```

---

## 10. Performance y Optimizaciones

### 10.1. Zero-Cost Abstraction

**Objetivo**: `async/await` debe tener overhead **mínimo** comparado con callbacks manuales.

**Técnicas**:
1. **Inline Small Futures**: Futures pequeños se inline (evitar indirección)
2. **Stack Allocation**: State machine en stack cuando sea posible
3. **Lazy Futures**: No inician hasta primer `poll()`
4. **Waker Caching**: Reusar wakers para reducir allocations

### 10.2. Benchmarks Esperados

| Operación | Overhead Máximo |
|-----------|-----------------|
| `await` ready future | < 5ns |
| Crear Future | < 20ns |
| Poll future | < 10ns |
| Wake task | < 15ns |

---

## 11. Limitaciones v1.0

### No Soportado en v1.0

1. ❌ **Cancellation**: No hay API para cancelar Futures
2. ❌ **Structured Concurrency**: No hay scopes automáticos
3. ❌ **Async Streams**: No hay `AsyncIterator`
4. ❌ **Async Drop**: No hay destructores asíncronos

### Features Futuras (v2.0+)

```vela
# v2.0: Cancellation
async fn cancellable() -> String {
  handle = spawn(longRunningTask())
  handle.cancel()  # Cancelar
}

# v2.0: Structured Concurrency
async fn structured() -> void {
  scope {
    spawn(task1())
    spawn(task2())
  }  # Auto-cancel si scope termina
}

# v2.0: Async Streams
async fn streamData() -> AsyncIterator<String> {
  for await item in dataStream {
    yield item
  }
}
```

---

## 12. Ejemplos Completos

### 12.1. HTTP Server Async

```vela
import 'system:http'

async fn handleRequest(req: Request) -> Response {
  match req.path {
    "/users/:id" => {
      userId = req.params.get("id")?
      user = await fetchUser(userId)?
      return Response.json(user)
    }
    
    "/orders" => {
      orders = await fetchOrders()?
      return Response.json(orders)
    }
    
    _ => {
      return Response.notFound()
    }
  }
}

async fn main() -> void {
  server = HttpServer::new("127.0.0.1:8080")
  server.onRequest(handleRequest)
  
  print("Server running on http://localhost:8080")
  await server.listen()
}
```

### 12.2. Actor System con Async

```vela
actor DatabaseWorker {
  db: DatabaseConnection
  
  async fn handleMessage(self, msg: Message) -> void {
    match msg {
      Query(sql, sender) => {
        result = await this.db.query(sql)
        sender.send(QueryResult(result))
      }
      
      Insert(data, sender) => {
        id = await this.db.insert(data)
        sender.send(InsertSuccess(id))
      }
    }
  }
}

async fn main() -> void {
  db = spawn(DatabaseWorker::new())
  
  # Enviar query asíncrono
  promise = Promise<QueryResult>()
  db.send(Query("SELECT * FROM users", promise))
  
  result = await promise.future()
  print("Result: ${result}")
}
```

### 12.3. Parallel Data Processing

```vela
async fn processData() -> List<Result> {
  # Cargar datos
  rawData = await loadData()
  
  # Procesar en paralelo (chunks de 100)
  chunks = rawData.chunk(100)
  
  futures = chunks.map(chunk => processChunk(chunk))
  results = await Future.all(futures)
  
  return results.flatten()
}

async fn processChunk(chunk: List<Data>) -> List<Result> {
  # Procesamiento pesado
  results = chunk.map(item => heavyProcess(item))
  return results
}
```

---

## 13. Testing

### 13.1. Unit Tests

```vela
@test
async fn testAsyncFunction() -> void {
  result = await fetchData()
  assert(result.length > 0, "Data should not be empty")
}

@test
async fn testErrorHandling() -> void {
  result = await fetchInvalidData()
  match result {
    Ok(_) => panic("Should have failed")
    Err(e) => assert(e.message == "Not found")
  }
}
```

### 13.2. Integration Tests

```vela
@test
async fn testActorAsync() -> void {
  actor = spawn(AsyncWorker::new())
  
  promise = Promise<String>()
  actor.send(FetchData("http://test.com", promise))
  
  result = await promise.future()
  assert(result.contains("test"))
}
```

---

## 14. Conclusión

Esta especificación define completamente el comportamiento de async/await en Vela, incluyendo:

- ✅ **Sintaxis formal** (EBNF)
- ✅ **Reglas de tipado** (type system)
- ✅ **Semántica operacional** (step-by-step execution)
- ✅ **Transformación CPS** (desugaring a state machines)
- ✅ **Executor model** (event loop)
- ✅ **Combinadores** (Future.all, race, timeout)
- ✅ **Integración actors** (async handlers)
- ✅ **Manejo de errores** (Result<T, E>, try/catch, ?)

**Próximos Pasos**:
1. **TASK-046**: Implementar transformación CPS en compilador (80h)
2. **TASK-047**: Implementar Future<T> y Promise<T> en runtime (48h)
3. **TASK-048**: Implementar Executor (event loop) (56h)
4. **TASK-049**: Tests completos (40h)

---

**Referencias**:
- ADR-012: Async/Await Semantics
- VELA-580: Sprint 18 - Async/Await
- Rust Async Book: https://rust-lang.github.io/async-book/
