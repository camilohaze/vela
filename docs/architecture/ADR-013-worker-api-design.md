# ADR-013: Worker API para Computación Paralela

## Estado
✅ Aceptado

## Fecha
2025-01-28

## Contexto

**Problema**: Vela necesita un sistema de workers para ejecutar tareas pesadas en paralelo sin bloquear el thread principal. El sistema async/await (Sprint 18) maneja asincronía cooperativa con I/O, pero no aprovecha múltiples CPU cores para computación intensiva.

**Necesidades**:
1. **Spawn workers**: Ejecutar funciones en threads separados
2. **Await completion**: Esperar resultados sin bloquear
3. **Cancellation**: Cancelar workers en ejecución
4. **Error propagation**: Manejar errores desde workers
5. **Resource limits**: Limitar workers concurrentes (thread pool)
6. **Integración con async/await**: Workers deben retornar `Future<T>`

**Casos de Uso**:
- CPU-intensive: procesamiento de imágenes, compresión, criptografía
- Parallel processing: procesar N items independientes en paralelo
- Offloading: mover trabajo pesado fuera del thread principal

---

## Decisión

Implementamos un **Worker API** basado en thread pool con integración nativa al sistema async/await existente.

### 1. API Surface

```vela
# Spawnar worker (thread separado)
async fn process_image(path: String) -> Result<Image> {
  # Heavy computation
  return Image.decode(path)
}

# Spawn retorna Future<T> que resuelve cuando el worker termina
future: Future<Result<Image>> = Worker.spawn(() => process_image("input.png"))

# Await resultado (sin bloquear thread principal)
result: Result<Image> = await future

# Spawn con múltiples workers (paralelo)
futures = [
  Worker.spawn(() => process_image("img1.png")),
  Worker.spawn(() => process_image("img2.png")),
  Worker.spawn(() => process_image("img3.png"))
]
results = await Future.all(futures)
```

### 2. Componentes Principales

#### **Worker** (static class)
```python
class Worker:
    """Static API for spawning workers."""
    
    @staticmethod
    def spawn(func: Callable[[], T], *, timeout: Optional[float] = None) -> Future[T]:
        """
        Spawn function on worker thread from pool.
        
        Args:
            func: Function to run on worker (no args, capture via closure)
            timeout: Optional timeout in seconds (None = no timeout)
            
        Returns:
            Future[T] that resolves with func() result or error
        """
        pass
```

#### **WorkerHandle** (internal)
```python
class WorkerHandle:
    """
    Handle to running worker task.
    
    Responsibilities:
    - Track worker state (running, done, cancelled)
    - Store result/error from worker
    - Notify Promise when worker completes
    """
    
    def __init__(self, worker_id: int, promise: Promise, timeout: Optional[float]):
        self.worker_id = worker_id
        self.promise = promise
        self.timeout = timeout
        self.start_time = time.time()
        self.cancelled = False
    
    def complete(self, result: Any) -> None:
        """Worker completed successfully."""
        if not self.cancelled:
            self.promise.resolve(result)
    
    def fail(self, error: Exception) -> None:
        """Worker failed with error."""
        if not self.cancelled:
            self.promise.reject(error)
    
    def cancel(self) -> bool:
        """Cancel worker (best-effort, thread can't be killed)."""
        self.cancelled = True
        return True
```

#### **WorkerPool** (singleton)
```python
class WorkerPool:
    """
    Thread pool for executing worker tasks.
    
    Design:
    - Fixed-size pool (default: CPU count)
    - Queue-based task distribution
    - Graceful shutdown support
    - Thread reuse (no per-task thread creation)
    """
    
    def __init__(self, max_workers: Optional[int] = None):
        self.max_workers = max_workers or os.cpu_count() or 4
        self.executor = concurrent.futures.ThreadPoolExecutor(
            max_workers=self.max_workers,
            thread_name_prefix="vela-worker"
        )
        self.active_handles: Dict[int, WorkerHandle] = {}
        self.next_worker_id = 0
        self.lock = threading.Lock()
    
    def submit(self, func: Callable, handle: WorkerHandle) -> None:
        """Submit function to thread pool."""
        future = self.executor.submit(self._execute_with_handle, func, handle)
        with self.lock:
            self.active_handles[handle.worker_id] = handle
    
    def _execute_with_handle(self, func: Callable, handle: WorkerHandle) -> None:
        """Execute func and notify handle of result."""
        try:
            if handle.timeout:
                # Check timeout
                elapsed = time.time() - handle.start_time
                if elapsed > handle.timeout:
                    raise TimeoutError(f"Worker timeout after {elapsed:.2f}s")
            
            result = func()
            handle.complete(result)
        except Exception as e:
            handle.fail(e)
        finally:
            with self.lock:
                self.active_handles.pop(handle.worker_id, None)
    
    def shutdown(self, wait: bool = True) -> None:
        """Shutdown thread pool."""
        self.executor.shutdown(wait=wait)
```

### 3. Integración con Async/Await

**Workers retornan `Future<T>`**, permitiendo:

```vela
# Sequential await
result1 = await Worker.spawn(() => heavy_compute_1())
result2 = await Worker.spawn(() => heavy_compute_2())

# Parallel await (usar Future.all)
results = await Future.all([
  Worker.spawn(() => heavy_compute_1()),
  Worker.spawn(() => heavy_compute_2())
])

# Race (primer worker que termine)
result = await Future.race([
  Worker.spawn(() => method_1()),
  Worker.spawn(() => method_2())
])
```

### 4. Timeout Support

```vela
# Worker con timeout de 5 segundos
future = Worker.spawn(() => slow_computation(), timeout=5.0)

try {
  result = await future
} catch (e: TimeoutError) {
  print("Worker timeout!")
}
```

### 5. Cancellation

**Limitación**: Python threads NO se pueden matar de forma segura. Cancellation es **best-effort**:

```vela
# Spawn worker
future = Worker.spawn(() => long_task())

# Cancel future (marca handle como cancelled)
future.cancel()

# Future rechaza con CancellationError
try {
  result = await future
} catch (e: CancellationError) {
  print("Worker cancelled")
}
```

⚠️ **IMPORTANTE**: El thread subyacente puede seguir ejecutándose. Worker API marca el `WorkerHandle` como cancelado, pero no mata el thread OS.

### 6. Error Handling

```vela
async fn risky_worker() -> Result<Data> {
  if random() < 0.5 {
    throw Error("Random failure")
  }
  return Data { value: 42 }
}

future = Worker.spawn(risky_worker)

match await future {
  Ok(data) => print("Success: ${data.value}")
  Err(error) => print("Worker failed: ${error}")
}
```

---

## Consecuencias

### ✅ Positivas

1. **CPU parallelism**: Aprovecha múltiples cores para computación intensiva
2. **Non-blocking**: Workers retornan `Future<T>`, compatible con async/await
3. **Resource control**: Thread pool limita workers concurrentes
4. **Simple API**: `Worker.spawn()` fácil de usar, similar a Rust/Go
5. **Error handling**: Excepciones propagadas via Promise rejection
6. **Timeout support**: Protección contra workers que nunca terminan
7. **Thread reuse**: Pool evita overhead de crear/destruir threads

### ❌ Negativas

1. **No true cancellation**: Threads Python no se pueden matar (limitación del lenguaje)
2. **GIL contention**: Python GIL limita paralelismo real (mitigado con threads C/Cython)
3. **Memory overhead**: Cada thread consume ~8MB RAM
4. **No shared state**: Workers deben comunicarse via channels (TASK-051)
5. **Closure capture**: Func pasada a `spawn()` debe capturar todo lo necesario

### ⚠️ Trade-offs

**Thread pool size**: Default = CPU count
- Muy pocos workers → underutilization
- Demasiados workers → context switching overhead, memoria
- Solución: Configurable, default razonable

**Timeout granularity**: Check timeout antes de ejecutar func
- No interrumpe ejecución en progreso
- Solución: Para timeout preciso, workers deben checkear periódicamente

---

## Alternativas Consideradas

### 1. **Process Pool (multiprocessing)** - Rechazada
- **Pro**: Bypasses Python GIL, true parallelism
- **Con**: High overhead (serialization, IPC), no shared memory
- **Razón**: Overhead prohibitivo para workers ligeros

### 2. **Async Workers (asyncio)** - Rechazada
- **Pro**: Lightweight, no threads
- **Con**: NO parallelism (single-threaded), GIL bound
- **Razón**: No aprovecha múltiples cores

### 3. **Actor Model (sin thread pool)** - Pospuesta
- **Pro**: Better isolation, supervision
- **Con**: Más complejo, requiere mailbox/message passing
- **Razón**: Workers son fundación, actors vienen después (TASK-037-041)

### 4. **Green Threads (gevent)** - Rechazada
- **Pro**: Lightweight concurrency
- **Con**: Monkey-patching, incompatible con stdlib
- **Razón**: Intrusivo, breaks ecosystem

---

## Referencias

### Inspiración de Otros Lenguajes

**Rust**:
```rust
// tokio::spawn (async task on runtime)
let handle = tokio::spawn(async move {
    expensive_computation()
});
let result = handle.await?;

// rayon::spawn (parallel task on thread pool)
rayon::spawn(move || expensive_computation());
```

**Go**:
```go
// Goroutine (green thread)
go func() {
    result := expensiveComputation()
    ch <- result
}()
result := <-ch
```

**JavaScript (Web Workers)**:
```javascript
// Web Worker (separate thread-like context)
const worker = new Worker('worker.js');
worker.postMessage({ data: input });
worker.onmessage = (e) => console.log(e.data);
```

**Swift**:
```swift
// Task.detached (unstructured concurrency)
let task = Task.detached {
    return await expensiveComputation()
}
let result = try await task.value
```

**Java**:
```java
// ExecutorService (thread pool)
ExecutorService executor = Executors.newFixedThreadPool(4);
Future<Result> future = executor.submit(() -> expensiveComputation());
Result result = future.get(); // blocking wait
```

---

## Implementación

### Archivos a Crear

1. **src/runtime/workers/__init__.py**
   - Exports: `Worker`, `WorkerPool`
   
2. **src/runtime/workers/worker.py**
   - `Worker` static class con `spawn()` method

3. **src/runtime/workers/worker_handle.py**
   - `WorkerHandle` para tracking de worker state

4. **src/runtime/workers/worker_pool.py**
   - `WorkerPool` singleton con thread pool

### Tests (tests/unit/runtime/workers/)

1. **test_worker_spawn.py**
   - Spawn worker, await result
   - Spawn múltiples workers
   - Worker con error

2. **test_worker_timeout.py**
   - Worker con timeout
   - Worker slow (timeout trigger)

3. **test_worker_cancellation.py**
   - Cancel worker antes de start
   - Cancel worker en progreso

4. **test_worker_pool.py**
   - Thread pool limits
   - Worker queue when pool full
   - Shutdown gracefully

---

## Métricas de Éxito

### Performance Targets

| Métrica | Target | Razón |
|---------|--------|-------|
| Spawn overhead | < 1ms | Spawning debe ser barato |
| Worker startup | < 5ms | Time to thread execution start |
| Throughput | > 100 workers/sec | Para paralelismo masivo |
| Pool utilization | > 80% | Thread pool efficiency |
| Cancellation latency | < 50ms | Fast cancellation response |

### Quality Targets

- **Test coverage**: ≥ 90%
- **Edge cases covered**: Timeout, cancellation, errors, pool limits
- **No deadlocks**: All tests pass under race detector
- **No memory leaks**: Workers cleanup properly

---

## Roadmap

### Sprint 19 (TASK-050)
- ✅ ADR-013 (este documento)
- 🔄 Implementar Worker, WorkerHandle, WorkerPool
- 🔄 Tests unitarios (spawn, timeout, cancel, errors)
- 🔄 Documentación API

### Sprint 19 (TASK-051)
- Channel<T> para comunicación entre workers

### Sprint 19 (TASK-052)
- Integration tests (workers + channels)
- Performance benchmarks
- Stress tests

### Future Sprints
- Actor model (TASK-037-041)
- Supervision hierarchies
- Distributed workers (network)

---

## Conclusión

**Worker API** proporciona computación paralela simple y efectiva en Vela:
- ✅ API minimalista: `Worker.spawn(func)`
- ✅ Integración nativa con async/await
- ✅ Thread pool para control de recursos
- ✅ Error handling y timeout

**Limitación conocida**: Cancellation no mata threads (Python limitation).

**Next**: Implementar `Worker`, `WorkerHandle`, `WorkerPool` en `src/runtime/workers/`.
