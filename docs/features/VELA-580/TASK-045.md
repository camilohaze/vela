# TASK-045: Diseñar async/await semantics

## 📋 Información General
- **Historia:** VELA-580 - Async/Await
- **Sprint:** 18
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Estimación:** 16 horas
- **Equipo:** Language Design

## 🎯 Objetivo

Definir formalmente el comportamiento de funciones asíncronas (`async fn`) y expresiones `await` en Vela, incluyendo:

1. **Sintaxis**: Gramática EBNF completa de async/await
2. **Semántica**: Reglas de ejecución (operational semantics)
3. **Sistema de Tipos**: Type rules para `Future<T>`, `Promise<T>`
4. **Transformación**: Desugaring a state machines (CPS - Continuation Passing Style)
5. **Modelo de Ejecución**: Executor (event loop) para polling futures
6. **Integración**: Con actors, signals, error handling (Result<T, E>)

## 🔨 Implementación

### Decisiones Clave

#### 1. Sintaxis Elegida: `async fn` / `await`

**Inspirada en**: Rust, JavaScript, Swift, Kotlin

```vela
# Función asíncrona
async fn fetchData() -> Future<String> {
  response = await httpClient.get("https://api.example.com")
  return response.body
}

# Con manejo de errores
async fn fetchUser(id: Number) -> Result<User, Error> {
  try {
    user = await getUser(id)
    return Ok(user)
  } catch (e) {
    return Err(e)
  }
}

# Arrow function async
fetchAsync = async () => {
  data = await loadData()
  return data
}
```

**Razones**:
- ✅ Sintaxis familiar para desarrolladores (Rust/JS/Swift)
- ✅ Declarativa y legible
- ✅ Type-safe (`Future<T>` explícito)
- ❌ Alternativa rechazada: Callbacks (callback hell)
- ❌ Alternativa rechazada: Green threads (overhead de runtime)

#### 2. Tipos Fundamentales

##### Future<T>
```vela
interface Future<T> {
  # Polling (usado por executor)
  fn poll(self, waker: Waker) -> Poll<T>
  
  # Combinadores funcionales
  fn map<U>(self, f: (T) -> U) -> Future<U>
  fn flatMap<U>(self, f: (T) -> Future<U>) -> Future<U>
  fn then<U>(self, callback: (T) -> U) -> Future<U>
  fn catch(self, callback: (Error) -> T) -> Future<T>
  
  # Combinaciones
  fn and<U>(self, other: Future<U>) -> Future<(T, U)>
  fn or(self, other: Future<T>) -> Future<T>
}

enum Poll<T> {
  Ready(T)     # Future completado
  Pending      # Aún no listo
}
```

##### Promise<T>
```vela
class Promise<T> {
  # Obtener Future asociado
  fn future(self) -> Future<T>
  
  # Resolver (una sola vez)
  fn resolve(self, value: T) -> void
  
  # Rechazar (una sola vez)
  fn reject(self, error: Error) -> void
}
```

**Relación**: `Promise` es el **productor** (escribe), `Future` es el **consumidor** (lee).

#### 3. Transformación CPS (Continuation Passing Style)

**Código Original**:
```vela
async fn example() -> Number {
  x = await fetch1()
  y = await fetch2()
  return x + y
}
```

**Desugaring Interno** (pseudo-código generado por compilador):
```vela
fn example() -> Future<Number> {
  promise = Promise<Number>()
  
  enum State {
    Start
    Await1(Future<Number>)
    Await2(x: Number, Future<Number>)
  }
  
  state = State::Start
  
  fn resume() -> void {
    match state {
      State::Start => {
        future1 = fetch1()
        state = State::Await1(future1)
        future1.then(x => resume())
      }
      
      State::Await1(x) => {
        future2 = fetch2()
        state = State::Await2(x, future2)
        future2.then(y => resume())
      }
      
      State::Await2(x, y) => {
        result = x + y
        promise.resolve(result)
      }
    }
  }
  
  resume()
  return promise.future()
}
```

**Ventajas de CPS**:
- ✅ **Zero-cost**: No requiere heap para stack frames
- ✅ **Compatible con LLVM/WASM**: Compila eficientemente
- ✅ **No GC overhead**: No genera presión en garbage collector
- ✅ **Análisis estático**: Compilador puede optimizar fácilmente

#### 4. Executor (Event Loop)

```vela
class Executor {
  ready_queue: Queue<Task>    # Tareas listas
  waiting: Map<TaskId, Task>  # Esperando I/O
  wakers: Map<TaskId, Waker>  # Despertadores
  
  fn spawn<T>(self, future: Future<T>) -> TaskHandle<T>
  fn run(self) -> void
  fn pollTask(self, task: Task) -> void
  fn waitForIO(self) -> void
}
```

**Event Loop Flow**:
1. Pop tarea de `ready_queue`
2. Poll tarea con `future.poll(waker)`
3. Si `Poll::Ready(value)` → Completar tarea
4. Si `Poll::Pending` → Mover a `waiting`, registrar waker
5. Esperar eventos I/O (epoll/kqueue/IOCP)
6. Waker despierta → Mover de `waiting` a `ready_queue`
7. Repetir

#### 5. Combinadores de Futures

##### Future.all() - Paralelo
```vela
async fn parallel() -> List<String> {
  results = await Future.all([
    fetch("url1"),
    fetch("url2"),
    fetch("url3")
  ])
  return results
}
```

##### Future.race() - Primero que completa
```vela
async fn fastest() -> String {
  result = await Future.race([
    fetchServer1(),
    fetchServer2()
  ])
  return result
}
```

##### Future.timeout() - Con timeout
```vela
async fn withTimeout() -> Result<String, TimeoutError> {
  result = await Future.timeout(
    fetchData(),
    5000  # 5 segundos
  )
  return result
}
```

#### 6. Integración con Actors

```vela
actor AsyncWorker {
  # Handler asíncrono
  async fn handleMessage(self, msg: Message) -> void {
    match msg {
      FetchData(url) => {
        data = await httpClient.get(url)
        this.send(self, DataReceived(data))
      }
    }
  }
}
```

**Garantías**:
- ✅ Mailbox NO se bloquea durante `await`
- ✅ Actor procesa siguiente mensaje si handler suspende
- ✅ Orden de mensajes preservado

#### 7. Manejo de Errores

##### Try/Catch
```vela
async fn safeFetch() -> Result<String, Error> {
  try {
    data = await fetchData()
    return Ok(data)
  } catch (e: NetworkError) {
    return Err(e)
  }
}
```

##### Operador ? (Propagación)
```vela
async fn propagate() -> Result<String, Error> {
  user = await fetchUser(123)?     # Propaga Err si falla
  orders = await fetchOrders(user.id)?
  return Ok(process(orders))
}
```

### Archivos Generados

1. **ADR-012**: `docs/architecture/ADR-012-async-await-semantics.md`
   - Decisión arquitectónica completa
   - Comparación con Rust, JavaScript, Swift, Kotlin
   - Alternativas consideradas (green threads, callbacks, generators)
   - Consecuencias positivas y negativas

2. **Especificación Formal**: `docs/specifications/async-await-spec.md`
   - Gramática EBNF (sintaxis formal)
   - Type rules (reglas de tipado)
   - Operational semantics (semántica de ejecución)
   - Transformación CPS (ejemplos detallados)
   - Executor model (event loop)
   - Combinadores (Future.all, race, timeout)
   - Integración con actors
   - Testing examples

3. **Documentación Tarea**: `docs/features/VELA-580/TASK-045.md` (este archivo)

### Semántica Operacional (Formal)

#### Type Rules

**Rule 1**: Async Function Type
```
Γ ⊢ body : T
────────────────────────────────
Γ ⊢ async fn f() -> T : Future<T>
```

**Rule 2**: Await Expression Type
```
Γ ⊢ expr : Future<T>
────────────────────────
Γ ⊢ await expr : T
```

**Rule 3**: Await Only in Async Context
```
Γ ⊢ expr : Future<T>    (current_context ≠ async)
──────────────────────────────────────────────────
Γ ⊢ await expr : ⊥  (Type Error)
```

#### Execution Semantics

**State Machine**:
```
State ::= Start                          # Inicial
        | Await(Future<T>, Continuation) # Esperando
        | Done(T)                        # Completado
        | Error(E)                       # Error
```

**Evaluation Steps**:
```
⟨Start, async fn f() { await e1; s2 }⟩ → ⟨Await(e1, λx.s2), poll⟩

⟨Await(Future<T>, cont), poll⟩ → ⟨cont(value), poll⟩
  if Future.poll() = Ready(value)

⟨Await(Future<T>, cont), poll⟩ → ⟨Await(Future<T>, cont), suspend⟩
  if Future.poll() = Pending

⟨Done(value), _⟩ → value
```

## ✅ Criterios de Aceptación

- [x] ✅ Sintaxis formal definida (EBNF)
- [x] ✅ Type rules especificadas (Hindley-Milner style)
- [x] ✅ Semántica operacional documentada (step-by-step)
- [x] ✅ Transformación CPS explicada con ejemplos
- [x] ✅ Executor model diseñado (event loop)
- [x] ✅ Future<T> y Promise<T> APIs definidos
- [x] ✅ Combinadores especificados (all, race, timeout)
- [x] ✅ Integración con actors diseñada
- [x] ✅ Manejo de errores definido (Result<T, E>, try/catch, ?)
- [x] ✅ ADR creado (decisión arquitectónica)
- [x] ✅ Especificación formal completa
- [x] ✅ Documentación de tarea generada

## 📊 Comparación con Otros Lenguajes

| Feature | Vela | Rust | JavaScript | Swift | Kotlin |
|---------|------|------|------------|-------|--------|
| **Syntax** | `async fn` / `await` | `async fn` / `.await` | `async function` / `await` | `async func` / `await` | `suspend fun` |
| **Future Type** | `Future<T>` | `impl Future` | `Promise<T>` | `Task<T>` | `Deferred<T>` |
| **Error Handling** | `Result<T, E>` | `Result<T, E>` | `try/catch` | `try/throws` | `try/catch` |
| **Executor** | Built-in | `tokio`/`async-std` | Node.js/Browser | Swift runtime | Dispatcher |
| **Cancellation** | v2.0 (futuro) | `Drop` | `AbortController` | `Task.cancel()` | `Job.cancel()` |
| **Zero-Cost** | ✅ CPS | ✅ Poll-based | ❌ Runtime | ⚠️ Runtime | ❌ Runtime |

**Conclusión**: Vela combina lo mejor de Rust (zero-cost, type-safe) con la ergonomía de JS/Swift.

## 🔄 Próximos Pasos (Sprint 18)

### TASK-046: Implementar async transform (CPS) - 80h
- Análisis de control flow en AST
- Generación de state machines
- Transformación de `await` expressions
- Preservación de tipos en transformación

### TASK-047: Implementar Future<T> y Promise<T> - 48h
- Implementación en runtime (Rust)
- Future trait con poll()
- Promise class con resolve/reject
- Waker system

### TASK-048: Implementar executor - 56h
- Event loop principal
- Task scheduling
- I/O polling (epoll/kqueue/IOCP)
- Waker registry

### TASK-049: Tests de async/await - 40h
- Unit tests (transformación CPS)
- Integration tests (Future combinadores)
- Performance benchmarks
- Edge cases (errors, cancellation, nested awaits)

## 📚 Referencias

- **Jira**: [TASK-045](https://velalang.atlassian.net/browse/VELA-580)
- **Historia**: [VELA-580 - Async/Await](https://velalang.atlassian.net/browse/VELA-580)
- **ADR**: [ADR-012 - Async/Await Semantics](../../architecture/ADR-012-async-await-semantics.md)
- **Especificación**: [Async/Await Spec](../../specifications/async-await-spec.md)

### Referencias Externas

- **Rust Async Book**: https://rust-lang.github.io/async-book/
- **JavaScript Promises/A+**: https://promisesaplus.com/
- **Swift Concurrency**: https://docs.swift.org/swift-book/LanguageGuide/Concurrency.html
- **Kotlin Coroutines**: https://kotlinlang.org/docs/coroutines-overview.html
- **CPS Transformation**: https://en.wikipedia.org/wiki/Continuation-passing_style

## 💡 Lecciones Aprendidas

1. **CPS vs Green Threads**: CPS es más complejo en el compilador, pero genera código más eficiente y compatible con LLVM/WASM.

2. **Promise/Future Split**: Separar "productor" (Promise) y "consumidor" (Future) hace el sistema más type-safe que JS Promises.

3. **Waker System**: Crucial para eficiencia - evita polling activo (busy-wait), solo despierta cuando hay progreso.

4. **Integración Actors**: Async handlers en actors requiere cuidado - mailbox NO debe bloquearse durante await.

5. **Error Handling**: `Result<T, E>` + operador `?` es más explícito que try/catch (Rust-style).

## 🎉 Conclusión

TASK-045 completada exitosamente. Se ha definido completamente la semántica de async/await en Vela, incluyendo:

- ✅ **Sintaxis formal** (EBNF)
- ✅ **Type system** (type rules)
- ✅ **Semántica operacional** (execution model)
- ✅ **Transformación CPS** (desugaring a state machines)
- ✅ **Executor model** (event loop con waker system)
- ✅ **Future<T> y Promise<T>** (APIs completos)
- ✅ **Combinadores** (all, race, timeout)
- ✅ **Integración actors** (async message handlers)
- ✅ **Manejo de errores** (Result<T, E>, try/catch, ?)

**Resultado**: Especificación lista para implementación en TASK-046 (transformación CPS en compilador).

**Impacto**: Vela tendrá async/await moderno, type-safe y zero-cost, comparable a Rust pero con sintaxis más ergonómica.
