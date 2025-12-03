# TASK-047: Implementar Future<T> y Promise<T>

## 📋 Información General
- **Historia:** VELA-580 - Async/Await
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30
- **Estimación:** 48 horas

## 🎯 Objetivo
Implementar tipos de runtime `Future<T>` y `Promise<T>` para el sistema async/await de Vela, inspirados en Rust, JavaScript y Scala.

## 🔨 Implementación

### Componentes Desarrollados

#### 1. **Poll<T>** (125 líneas)
Estado de polling de un Future. Representa el resultado de una operación de polling.

**Archivo:** `src/runtime/async_runtime/poll.py`

**Tipos:**
- `PollState` enum:
  - `READY`: Valor disponible
  - `PENDING`: Aún en progreso

**API:**
```python
@dataclass
class Poll[T]:
    state: PollState
    value: Optional[T] = None
    
    @staticmethod
    def ready(value: T) -> Poll[T]
    
    @staticmethod
    def pending() -> Poll[T]
    
    def is_ready() -> bool
    def is_pending() -> bool
    def unwrap() -> T
    def unwrap_or(default: T) -> T
    def map(f: Callable[[T], U]) -> Poll[U]
```

**Ejemplo de uso:**
```python
# Crear poll ready
poll = Poll.ready(42)
assert poll.is_ready()
assert poll.unwrap() == 42

# Crear poll pending
poll = Poll.pending()
assert poll.is_pending()

# Transformar con map
poll = Poll.ready(10)
doubled = poll.map(lambda x: x * 2)
assert doubled.unwrap() == 20
```

#### 2. **Waker** (113 líneas)
Mecanismo de wake-up para notificar cuando un Future está listo para hacer progreso.

**Archivo:** `src/runtime/async_runtime/waker.py`

**API:**
```python
@dataclass
class Waker:
    callback: Callable[[], None]
    _woken: bool = False
    _lock: Lock = field(default_factory=Lock)
    
    def wake() -> None
    def wake_by_ref() -> None  # Rust compatibility
    def is_woken() -> bool
    def reset() -> None
    def clone() -> Waker
    
    @staticmethod
    def noop() -> Waker
```

**Características:**
- **Thread-safe**: Usa `threading.Lock`
- **Wake once**: Solo ejecuta callback una vez hasta reset
- **Cloneable**: Permite compartir waker entre futures
- **No-op**: Waker sin acción para casos simples

**Ejemplo de uso:**
```python
# Waker con callback
wake_count = 0
def on_wake():
    global wake_count
    wake_count += 1

waker = Waker(on_wake)
waker.wake()  # Ejecuta callback
assert wake_count == 1

waker.wake()  # No ejecuta (ya despertado)
assert wake_count == 1

waker.reset()  # Reset para reusar
waker.wake()
assert wake_count == 2

# No-op waker
waker = Waker.noop()
waker.wake()  # No hace nada
```

#### 3. **Future<T>** (415 líneas)
Trait abstracto para computaciones asíncronas lazy. Representa un valor que estará disponible en el futuro.

**Archivo:** `src/runtime/async_runtime/future.py`

**API Core:**
```python
class Future[T](ABC):
    @abstractmethod
    def poll(waker: Waker) -> Poll[T]
```

**Combinadores:**
```python
# Transformación
def map(f: Callable[[T], U]) -> Future[U]
def flat_map(f: Callable[[T], Future[U]]) -> Future[U]

# Chaining (Promise-style)
def then(callback: Callable[[T], U]) -> Future[U]
def catch(callback: Callable[[Exception], T]) -> Future[T]

# Combinación
def and_then(other: Future[U]) -> Future[Tuple[T, U]]
def or_else(other: Future[T]) -> Future[T]
```

**Factory Methods:**
```python
@staticmethod
def ready(value: T) -> Future[T]

@staticmethod
def pending() -> Future[T]

@staticmethod
def from_result(result: Result[T, Exception]) -> Future[T]

@staticmethod
def all(futures: List[Future[T]]) -> Future[List[T]]

@staticmethod
def race(futures: List[Future[T]]) -> Future[T]

@staticmethod
def timeout(future: Future[T], seconds: float) -> Future[T]
```

**Implementaciones Concretas (11 clases):**

1. **ReadyFuture<T>**: Future inmediatamente listo
2. **PendingFuture<T>**: Future que nunca completa
3. **ErrorFuture<T>**: Future fallido con error
4. **MapFuture<T, U>**: Transformación `f: T → U`
5. **FlatMapFuture<T, U>**: Transformación monádica `f: T → Future<U>`
6. **CatchFuture<T>**: Recuperación de errores
7. **AndFuture<T, U>**: Combinación paralela → `(T, U)`
8. **OrFuture<T>**: Race condition (primero en completar)
9. **AllFuture<T>**: Esperar todos → `[T]`
10. **RaceFuture<T>**: Primero de una lista
11. **TimeoutFuture<T>**: Con deadline

**Ejemplos de uso:**

```python
# Future ready
future = Future.ready(42)
waker = Waker.noop()
poll = future.poll(waker)
assert poll.is_ready()
assert poll.unwrap() == 42

# Map: transformación
future = Future.ready(10)
doubled = future.map(lambda x: x * 2)
poll = doubled.poll(waker)
assert poll.unwrap() == 20

# FlatMap: composición monádica
future = Future.ready(5)
chained = future.flat_map(lambda x: Future.ready(x * 3))
poll = chained.poll(waker)
assert poll.unwrap() == 15

# And: combinación paralela
f1 = Future.ready(1)
f2 = Future.ready(2)
combined = f1.and_then(f2)
poll = combined.poll(waker)
assert poll.unwrap() == (1, 2)

# All: esperar múltiples
futures = [Future.ready(1), Future.ready(2), Future.ready(3)]
all_future = Future.all(futures)
poll = all_future.poll(waker)
assert poll.unwrap() == [1, 2, 3]

# Race: primero en completar
f1 = Future.ready(10)
f2 = Future.pending()
race = Future.race([f1, f2])
poll = race.poll(waker)
assert poll.unwrap() == 10

# Timeout: con deadline
future = Future.pending()
with_timeout = Future.timeout(future, 1.0)
# Lanzará TimeoutError después de 1 segundo

# Catch: recuperación de errores
future = Future.from_result(Err(ValueError("error")))
recovered = future.catch(lambda e: "default")
poll = recovered.poll(waker)
assert poll.unwrap() == "default"

# Chaining: estilo Promise
future = (Future.ready(5)
    .map(lambda x: x * 2)      # 10
    .map(lambda x: x + 5)      # 15
    .then(lambda x: x / 3))    # 5.0
poll = future.poll(waker)
assert poll.unwrap() == 5.0
```

#### 4. **Promise<T>** (246 líneas)
Producer side de un Future. Permite resolver/rechazar un valor asincrónicamente.

**Archivo:** `src/runtime/async_runtime/promise.py`

**API:**
```python
class Promise[T]:
    def __init__()
    
    def future() -> Future[T]
    def resolve(value: T) -> None
    def reject(error: Exception) -> None
    def is_completed() -> bool
    
    @staticmethod
    def resolved(value: T) -> Promise[T]
    
    @staticmethod
    def rejected(error: Exception) -> Promise[T]
```

**Características:**
- **Thread-safe**: Usa `threading.Lock`
- **Resolve once**: Solo se puede resolver/rechazar una vez
- **Auto wake**: Notifica automáticamente al waker al resolver
- **Future asociado**: Crea `PromiseFuture` interno

**Helpers:**
```python
def promise_from_callback(f: Callable[[Callable[[T], None]], None]) -> Future[T]

def promise_from_error_callback(
    f: Callable[[Callable[[T], None], Callable[[Exception], None]], None]
) -> Future[T]
```

**Ejemplos de uso:**

```python
# Promise básico
promise = Promise[int]()
future = promise.future()

# Poll antes de resolver
waker = Waker.noop()
poll = future.poll(waker)
assert poll.is_pending()

# Resolver promise
promise.resolve(42)
assert promise.is_completed()

# Poll después de resolver
poll = future.poll(waker)
assert poll.is_ready()
assert poll.unwrap() == 42

# No se puede resolver dos veces
try:
    promise.resolve(43)
except RuntimeError:
    pass  # RuntimeError: Promise already completed

# Promise rechazado
promise = Promise[str]()
future = promise.future()
promise.reject(ValueError("error"))
assert promise.is_completed()

# Factories
p1 = Promise.resolved(100)
assert p1.is_completed()

p2 = Promise.rejected(ValueError("error"))
assert p2.is_completed()

# Integración con callbacks (estilo Node.js)
def fetch_data(callback):
    # Simula operación asíncrona
    callback("result")

future = promise_from_callback(fetch_data)
poll = future.poll(waker)
assert poll.is_ready()
assert poll.unwrap() == "result"

# Callbacks con manejo de errores
def risky_operation(on_success, on_error):
    try:
        result = do_work()
        on_success(result)
    except Exception as e:
        on_error(e)

future = promise_from_error_callback(risky_operation)
```

**Waker Notification:**
```python
# Promise notifica al waker cuando se resuelve
wake_count = 0
def on_wake():
    global wake_count
    wake_count += 1

promise = Promise[int]()
future = promise.future()
waker = Waker(on_wake)

# Registrar waker
poll = future.poll(waker)
assert poll.is_pending()

# Resolver promise → notifica waker
promise.resolve(42)
assert wake_count == 1  # Waker ejecutado
```

### Diseño Arquitectónico

#### Relación entre componentes

```
┌─────────────┐
│   Future<T> │  ← Trait abstracto
└──────┬──────┘
       │
       ├─── ReadyFuture<T>
       ├─── PendingFuture<T>
       ├─── ErrorFuture<T>
       ├─── MapFuture<T, U>
       ├─── FlatMapFuture<T, U>
       ├─── CatchFuture<T>
       ├─── AndFuture<T, U>
       ├─── OrFuture<T>
       ├─── AllFuture<T>
       ├─── RaceFuture<T>
       ├─── TimeoutFuture<T>
       └─── PromiseFuture<T>  ← Usado por Promise
                ▲
                │
         ┌──────┴──────┐
         │ Promise<T>  │  ← Producer
         └─────────────┘

┌─────────────┐
│   Poll<T>   │  ← Estado de polling
└─────────────┘
     ▲
     │ retorna
     │
   poll(waker)

┌─────────────┐
│    Waker    │  ← Wake-up mechanism
└─────────────┘
```

#### Inspiraciones

**Rust:**
- `Poll<T>` con `Ready` y `Pending`
- `Future` trait con método `poll()`
- `Waker` para notificaciones

**JavaScript:**
- `Promise<T>` con `resolve()` / `reject()`
- Métodos `.then()` y `.catch()`
- Factory `Promise.all()` y `Promise.race()`

**Scala:**
- Combinadores funcionales (`map`, `flatMap`)
- Composición monádica
- `Future` inmutable

### Decisiones de Diseño

#### 1. Lazy Evaluation (Estilo Rust)
Los Futures son **lazy**: no ejecutan hasta que se llama `poll()`.

```python
# Future no ejecuta hasta poll()
future = Future.ready(42)  # No evalúa
poll = future.poll(waker)  # Evalúa aquí
```

**Ventajas:**
- Control explícito de ejecución
- No desperdicia recursos
- Composición sin side effects

#### 2. Polling con Waker (Estilo Rust)
Cada `poll()` recibe un `Waker` para notificar cuando hacer re-poll.

```python
def poll(waker: Waker) -> Poll[T]:
    if not_ready:
        waker.wake()  # Notifica cuando esté listo
        return Poll.pending()
    return Poll.ready(value)
```

**Ventajas:**
- Eficiente (no busy-waiting)
- Cooperativo (no threads)
- Integrable con event loop

#### 3. Promise como Producer (Estilo JavaScript)
`Promise<T>` es mutable y se resuelve una vez.

```python
promise = Promise[int]()
future = promise.future()  # Consumer
promise.resolve(42)         # Producer
```

**Ventajas:**
- API familiar (JavaScript)
- Bridge para callbacks
- Integración con código existente

#### 4. Combinadores Funcionales (Estilo Scala)
Futures son inmutables y se componen con `map`, `flatMap`, etc.

```python
result = (future
    .map(lambda x: x * 2)
    .flat_map(lambda x: fetch(x))
    .catch(lambda e: default_value))
```

**Ventajas:**
- Composición declarativa
- No mutación
- Fácil de razonar

#### 5. Thread Safety
Todos los componentes son thread-safe con `threading.Lock`.

```python
with self._lock:
    if self._completed:
        raise RuntimeError("Promise already completed")
    self._completed = True
```

**Ventajas:**
- Seguro en multi-threading
- Previene race conditions
- Compatible con event loops multi-thread

### Testing

**Archivo:** `tests/unit/runtime/test_async.py` (419 líneas)

**Cobertura:** 38 tests (100% pasando)

**Suites:**

1. **TestPoll** (6 tests):
   - Creación de Ready y Pending
   - Unwrap y unwrap_or
   - Map transformation

2. **TestWaker** (5 tests):
   - Callback execution
   - Wake once behavior
   - Reset mechanism
   - Clone functionality
   - No-op waker

3. **TestFuture** (14 tests):
   - ReadyFuture y PendingFuture
   - Map y FlatMap
   - Then y Catch
   - AndThen y OrElse
   - All y Race
   - Pending propagation

4. **TestPromise** (9 tests):
   - Creación y polling
   - Resolve y Reject
   - Resolve/Reject once
   - Factories (resolved/rejected)
   - Waker notification

5. **TestCallbackIntegration** (1 test):
   - promise_from_callback

6. **TestEdgeCases** (3 tests):
   - Chaining multiple maps
   - FlatMap chain
   - Empty All/Race lists

**Resultados:**
```
38 passed in 0.14s
```

**Cobertura de casos:**
- ✅ Happy path (valores listos)
- ✅ Pending propagation (valores no listos)
- ✅ Error handling (excepciones)
- ✅ Edge cases (listas vacías, chaining)
- ✅ Thread safety (locks)
- ✅ Waker notification (callbacks)

## ✅ Criterios de Aceptación

- [x] Poll<T> implementado con Ready/Pending
- [x] Waker implementado con callbacks thread-safe
- [x] Future<T> trait abstracto con poll()
- [x] 11 implementaciones concretas de Future
- [x] Combinadores: map, flat_map, then, catch, and_then, or_else
- [x] Static factories: ready, pending, all, race, timeout
- [x] Promise<T> implementado como producer
- [x] Promise resolve/reject una vez
- [x] Promise notifica waker automáticamente
- [x] promise_from_callback para integración
- [x] 38 tests unitarios pasando (100%)
- [x] Thread safety con Lock
- [x] Documentación completa
- [x] Módulo renombrado (async → async_runtime)

## 📊 Métricas

- **Archivos creados:** 6
  - `src/runtime/async_runtime/__init__.py` (35 líneas)
  - `src/runtime/async_runtime/poll.py` (125 líneas)
  - `src/runtime/async_runtime/waker.py` (113 líneas)
  - `src/runtime/async_runtime/future.py` (415 líneas)
  - `src/runtime/async_runtime/promise.py` (246 líneas)
  - `tests/unit/runtime/test_async.py` (419 líneas)
- **Total líneas:** 1,353
- **Tests:** 38 (100% passing)
- **Tiempo de ejecución tests:** 0.14s
- **Cobertura:** 100% de funcionalidad core

## 🔗 Referencias

- **Jira:** [TASK-047](https://velalang.atlassian.net/browse/VELA-580)
- **Historia:** [VELA-580 - Async/Await](https://velalang.atlassian.net/browse/VELA-580)
- **Diseño:** `docs/features/VELA-580/TASK-045.md`
- **CPS Transform:** `docs/features/VELA-580/TASK-046.md`

**Inspiraciones:**
- Rust: [std::task::Poll](https://doc.rust-lang.org/std/task/enum.Poll.html)
- Rust: [std::future::Future](https://doc.rust-lang.org/std/future/trait.Future.html)
- JavaScript: [Promise](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
- Scala: [Future](https://docs.scala-lang.org/overviews/core/futures.html)

## 🚀 Próximos Pasos

**TASK-048:** Implementar Executor (event loop) para ejecutar Futures (56 horas)

**Funcionalidad requerida:**
- Event loop con polling
- Task scheduling
- Waker queues
- Spawn task
- Block on future
- Integration tests
