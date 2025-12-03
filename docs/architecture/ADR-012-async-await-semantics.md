# ADR-012: Async/Await Semantics

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

Vela necesita un sistema de programación asíncrona robusto y moderno para:

1. **I/O No Bloqueante**: Operaciones de red, filesystem, base de datos
2. **Concurrencia Ligera**: Miles de tareas asíncronas concurrentes sin overhead de threads
3. **Integración con Actors**: Los actores necesitan async/await para operaciones internas
4. **APIs Modernas**: HTTP clients, WebSockets, timers, etc.
5. **Ergonomía**: Sintaxis clara y familiar (inspirada en Rust, JS/TS, Swift, Kotlin)

**Desafíos Principales**:
- Vela es **funcional puro** → No hay mutabilidad implícita
- Integración con sistema de actores (message passing)
- Manejo de errores robusto (`Result<T, E>`)
- Compatibilidad con reactividad (`signal`, `computed`, `effect`)

## Decisión

### 1. Sintaxis de Async/Await

#### 1.1. Funciones Asíncronas

```vela
# Función async básica
async fn fetchData() -> Future<String> {
  response = await httpClient.get("https://api.example.com/data")
  return response.body
}

# Función async con manejo de errores
async fn fetchUser(id: Number) -> Result<User, Error> {
  try {
    response = await httpClient.get("/users/${id}")
    user = await response.json()
    return Ok(user)
  } catch (e: NetworkError) {
    return Err(e)
  }
}

# Arrow function async
fetchAsync = async () => {
  data = await loadData()
  return data.process()
}
```

**Reglas**:
- ✅ `async fn` declara función asíncrona
- ✅ Retorna `Future<T>` automáticamente (inferido)
- ✅ `await` solo válido dentro de `async fn`
- ✅ `await` desugara a operaciones de Future

#### 1.2. Await Expression

```vela
# Await simple
result = await someAsyncOperation()

# Await con chaining
user = await fetchUser(123).map(u => u.name)

# Await en expresiones
total = (await fetchPrice()) * quantity

# Await con match
match await fetchData() {
  Ok(data) => processData(data)
  Err(e) => handleError(e)
}
```

**Reglas**:
- ✅ `await` suspende ejecución hasta que Future se resuelva
- ✅ `await` retorna el valor `T` de `Future<T>`
- ✅ Si Future falla, propaga error (ver manejo de errores)
- ❌ NO se puede usar `await` fuera de `async fn`

### 2. Tipos Fundamentales

#### 2.1. Future<T>

```vela
# Future es un valor que eventualmente se resolverá
interface Future<T> {
  # Registra callback para cuando Future se resuelva
  fn then<U>(self, callback: (T) -> U) -> Future<U>
  
  # Registra callback para error
  fn catch(self, callback: (Error) -> T) -> Future<T>
  
  # Combina con otro Future
  fn and<U>(self, other: Future<U>) -> Future<(T, U)>
  
  # Alternativa si falla
  fn or(self, other: Future<T>) -> Future<T>
  
  # Mapea el valor
  fn map<U>(self, f: (T) -> U) -> Future<U>
  
  # FlatMap (evita Future<Future<T>>)
  fn flatMap<U>(self, f: (T) -> Future<U>) -> Future<U>
  
  # Polling (usado internamente por executor)
  fn poll(self, waker: Waker) -> Poll<T>
}

# Estado de polling
enum Poll<T> {
  Ready(T)         # Future completado
  Pending          # Future aún no listo
}
```

**Diseño Inspirado En**:
- **Rust**: `Future` trait con `poll()`, `Waker` para despertador
- **JavaScript**: `Promise` con `.then()`, `.catch()`
- **Swift**: `async/await` con `Task` y `continuation`

#### 2.2. Promise<T>

```vela
# Promise es la "escritora" de un Future (una sola vez)
class Promise<T> {
  # Obtener el Future correspondiente
  fn future(self) -> Future<T>
  
  # Resolver la Promise con un valor
  fn resolve(self, value: T) -> void
  
  # Rechazar la Promise con error
  fn reject(self, error: Error) -> void
}

# Uso típico
fn createDelayedValue() -> Future<String> {
  promise = Promise<String>()
  
  # Simular async operation
  setTimeout(() => {
    promise.resolve("Hello after 1s")
  }, 1000)
  
  return promise.future()
}
```

**Relación Promise ↔ Future**:
- `Promise<T>` es el **productor** (escribe una vez)
- `Future<T>` es el **consumidor** (lee cuando esté listo)
- Similar a `Promise`/`Future` en Scala, `Promise`/`Deferred` en Kotlin

### 3. Modelo de Ejecución

#### 3.1. Transformation a State Machine (Continuation Passing Style)

**Código Original (async/await)**:
```vela
async fn fetchAndProcess() -> Result<String> {
  user = await fetchUser(123)         # Punto de suspensión 1
  orders = await fetchOrders(user.id) # Punto de suspensión 2
  return Ok(processOrders(orders))
}
```

**Desugaring Interno (CPS - Continuation Passing Style)**:
```vela
# El compilador transforma a state machine con 3 estados

fn fetchAndProcess() -> Future<Result<String>> {
  promise = Promise<Result<String>>()
  state = State::Start
  
  # State machine
  fn resumeStateMachine() -> void {
    match state {
      State::Start => {
        # Suspensión 1: esperar fetchUser
        futureUser = fetchUser(123)
        state = State::AwaitUser(futureUser)
        
        futureUser.then(user => {
          state.user = user  # Guardar resultado
          resumeStateMachine()
        })
      }
      
      State::AwaitUser(user) => {
        # Suspensión 2: esperar fetchOrders
        futureOrders = fetchOrders(user.id)
        state = State::AwaitOrders(user, futureOrders)
        
        futureOrders.then(orders => {
          state.orders = orders
          resumeStateMachine()
        })
      }
      
      State::AwaitOrders(user, orders) => {
        # Completar
        result = processOrders(orders)
        promise.resolve(Ok(result))
      }
    }
  }
  
  resumeStateMachine()
  return promise.future()
}
```

**Ventajas del Approach CPS**:
- ✅ No requiere runtime complejo (no necesita heap para stack frames)
- ✅ Compatible con compilación a LLVM/WASM
- ✅ Zero-cost abstraction (overhead mínimo)
- ✅ Similar a Rust (`async fn` → `impl Future`)

#### 3.2. Executor (Event Loop)

```vela
# Event loop ejecuta Futures hasta completarse
class Executor {
  tasks: Queue<Future<_>> = []
  wakers: Map<FutureId, Waker> = {}
  
  fn spawn<T>(self, future: Future<T>) -> void {
    this.tasks.push(future)
  }
  
  fn run(self) -> void {
    while !this.tasks.isEmpty() {
      future = this.tasks.pop()
      waker = Waker(futureId => this.tasks.push(futureId))
      
      match future.poll(waker) {
        Poll::Ready(value) => {
          # Future completado
          continue
        }
        Poll::Pending => {
          # Future aún no listo, registrar waker
          this.wakers.set(future.id, waker)
        }
      }
    }
  }
}

# Uso
executor = Executor()
executor.spawn(fetchAndProcess())
executor.run()
```

**Modelo Basado En**:
- **Rust**: `tokio` executor con `spawn()` y `block_on()`
- **JavaScript**: Event loop con microtask queue
- **Python**: `asyncio` event loop

### 4. Combinadores de Futures

```vela
# Ejecutar múltiples Futures en paralelo
async fn fetchAll() -> Result<(User, Orders)> {
  results = await Future.all([
    fetchUser(123),
    fetchOrders(456)
  ])
  return Ok(results)
}

# Ejecutar el primero que complete
async fn fetchFastest() -> Result<String> {
  result = await Future.race([
    fetchFromServer1(),
    fetchFromServer2()
  ])
  return result
}

# Ejecutar secuencialmente (equivalente a múltiples awaits)
async fn fetchSequential() -> Result<String> {
  user = await fetchUser(123)
  orders = await fetchOrders(user.id)
  return Ok(process(orders))
}

# Timeout
async fn fetchWithTimeout() -> Result<String> {
  result = await Future.timeout(
    fetchData(),
    duration: 5000  # 5 segundos
  )
  return result
}
```

### 5. Manejo de Errores

#### 5.1. Try/Catch en Async

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

#### 5.2. Propagación Automática con Result<T, E>

```vela
# Operador ? para propagación (como Rust)
async fn fetchUserOrders(userId: Number) -> Result<Orders, Error> {
  user = await fetchUser(userId)?     # Si Err, retorna inmediatamente
  orders = await fetchOrders(user.id)?
  return Ok(orders)
}
```

#### 5.3. Future.catch() Combinator

```vela
# Alternativa funcional
async fn fetchWithFallback() -> String {
  data = await fetchPrimary()
    .catch(e => fetchSecondary())
    .catch(e => "default value")
  
  return data
}
```

### 6. Integración con Actors

```vela
# Actores pueden usar async/await internamente
actor AsyncWorker {
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

# Spawn actor con async handler
worker = spawn(AsyncWorker)
worker.send(FetchData("https://api.example.com"))
```

**Reglas de Integración**:
- ✅ Los handlers de mensajes pueden ser `async fn`
- ✅ `await` solo dentro de handlers asíncronos
- ✅ Mailbox NO se bloquea (actor procesa siguiente mensaje si handler suspende)
- ✅ Garantía de orden: Mensajes al mismo actor se procesan en orden

### 7. Comparación con Otros Lenguajes

| Feature | Vela | Rust | JavaScript | Swift | Kotlin |
|---------|------|------|------------|-------|--------|
| **Syntax** | `async fn` / `await` | `async fn` / `.await` | `async function` / `await` | `async func` / `await` | `suspend fun` / coroutines |
| **Future Type** | `Future<T>` | `impl Future<Output=T>` | `Promise<T>` | `Task<T>` | `Deferred<T>` |
| **Error Handling** | `Result<T, E>` | `Result<T, E>` | `try/catch` | `try/throws` | `try/catch` |
| **Executor** | Built-in event loop | `tokio`/`async-std` | Browser/Node.js | Swift concurrency runtime | Kotlin coroutines dispatcher |
| **Cancellation** | Not in v1.0 | `Drop` trait | `AbortController` | `Task.cancel()` | `Job.cancel()` |
| **Structured Concurrency** | Not in v1.0 | Scoped tasks (nightly) | Not built-in | Task groups | `coroutineScope` |

## Consecuencias

### Positivas
- ✅ **Ergonomía Moderna**: Sintaxis familiar para desarrolladores de Rust/JS/Swift
- ✅ **Zero-Cost**: CPS transformation sin overhead de runtime
- ✅ **Integración Actor System**: Async handlers en actores
- ✅ **Type Safety**: `Future<T>` y `Result<T, E>` previenen race conditions y errores
- ✅ **Composabilidad**: Combinadores funcionales (`.then()`, `.map()`, `.flatMap()`)

### Negativas
- ⚠️ **Complejidad del Compilador**: Transformación CPS requiere análisis de control flow
- ⚠️ **Debugging**: Stack traces de async code son complejas
- ⚠️ **Cancellation**: No se implementa en v1.0 (feature futura)
- ⚠️ **Structured Concurrency**: No se implementa en v1.0 (feature futura)

### Trade-offs
- **CPS vs Green Threads**: Elegimos CPS porque no requiere runtime scheduler pesado
- **Promise/Future Split**: Más verboso que JS, pero más explícito y type-safe
- **No Cancellation**: Simplifica v1.0, pero limita casos de uso avanzados

## Alternativas Consideradas

### 1. Green Threads (Erlang/Go Style)
```vela
# Alternativa: green threads
fn fetchData() -> String {
  spawn(() => heavyWork())  # Lanza thread ligero
  return "done"
}
```

**Rechazada porque**:
- ❌ Requiere runtime scheduler complejo (overhead de memoria)
- ❌ No se integra bien con LLVM/WASM
- ❌ Dificulta control de recursos (cuántos threads activos?)

### 2. Callbacks (Node.js Old Style)
```vela
# Alternativa: callbacks
fn fetchData(callback: (String) -> void) -> void {
  httpClient.get("/data", (response) => {
    callback(response.body)
  })
}
```

**Rechazada porque**:
- ❌ Callback hell (difícil de leer/mantener)
- ❌ No composable (no hay `.then()`, `.map()`, etc.)
- ❌ Error handling complicado

### 3. Generator/Yield (Python/ES6 Style)
```vela
# Alternativa: generators
fn* fetchData() -> Generator<String> {
  response = yield httpClient.get("/data")
  return response.body
}
```

**Rechazada porque**:
- ❌ Menos intuitivo que `async/await`
- ❌ Requiere `yield` keyword adicional
- ❌ Mezcla generadores con async (diferentes propósitos)

## Referencias

- **Jira**: [TASK-045](https://velalang.atlassian.net/browse/VELA-580)
- **Historia**: [VELA-580](https://velalang.atlassian.net/browse/VELA-580)
- **Rust Async Book**: https://rust-lang.github.io/async-book/
- **JavaScript Promises**: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
- **Swift Concurrency**: https://docs.swift.org/swift-book/LanguageGuide/Concurrency.html
- **Kotlin Coroutines**: https://kotlinlang.org/docs/coroutines-overview.html

## Implementación

- **ADR**: `docs/architecture/ADR-012-async-await-semantics.md` (este archivo)
- **Especificación Técnica**: `docs/specifications/async-await-spec.md`
- **Documentación Tarea**: `docs/features/VELA-580/TASK-045.md`
- **Próximas Tareas**:
  - TASK-046: Implementar async transform (CPS) en compilador
  - TASK-047: Implementar `Future<T>` y `Promise<T>` en runtime
  - TASK-048: Implementar Executor (event loop)
  - TASK-049: Tests de async/await

## Actualización de Roadmap

Esta decisión arquitectónica define las bases para Sprint 18 (Async/Await):
- ✅ TASK-045: Diseño completo de semántica async/await
- ⏳ TASK-046: Implementación de transformación CPS (80h)
- ⏳ TASK-047: Implementación de tipos Future/Promise (48h)
- ⏳ TASK-048: Implementación de Executor (56h)
- ⏳ TASK-049: Suite de tests (40h)

**Total Sprint 18**: 240 horas de implementación
