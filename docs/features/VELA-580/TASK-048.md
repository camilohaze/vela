# TASK-048: Implementar Executor (Event Loop)

## 📋 Información General
- **Historia:** VELA-580 - Async/Await
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Estimación:** 56 horas

## 🎯 Objetivo
Implementar Executor (event loop) para ejecutar Futures y manejar task scheduling con sistema de waker queues.

## 🔨 Implementación

### Componentes Desarrollados

#### 1. **Task<T>** (216 líneas)
Unidad de trabajo asíncrono que wrappea un Future con estado y lifecycle.

**Archivo:** `src/runtime/async_runtime/task.py`

**Estados (TaskState enum)**:
- `PENDING`: No iniciado
- `RUNNING`: En ejecución (polleando)
- `COMPLETED`: Completado exitosamente
- `FAILED`: Falló con error
- `CANCELLED`: Cancelado por usuario

**Clases**:

```python
@dataclass
class TaskId:
    """Identificador único de task (UUID)"""
    value: str
    
    def __hash__() -> int
    def __eq__(other) -> bool

@dataclass
class Task[T]:
    """Task<T> - Wrapper de Future con estado"""
    future: Future[T]
    task_id: TaskId
    state: TaskState
    _result: Optional[T]
    _error: Optional[Exception]
    _lock: Lock
    
    def poll(waker: Waker) -> Poll[T]
    def is_completed() -> bool
    def is_failed() -> bool
    def is_cancelled() -> bool
    def is_pending() -> bool
    def result() -> T
    def error() -> Optional[Exception]
    def cancel() -> bool
```

**Lifecycle Flow**:
```
PENDING → RUNNING → COMPLETED
              ↓
           FAILED
              ↓
          CANCELLED
```

**Ejemplos de uso**:

```python
# Crear task
future = Future.ready(42)
task = Task(future)

# Poll task
waker = Waker.noop()
poll = task.poll(waker)

if poll.is_ready():
    result = task.result()  # 42
    assert task.is_completed()

# Cancelar task
pending_task = Task(Future.pending())
cancelled = pending_task.cancel()
assert cancelled
assert pending_task.is_cancelled()

# Error handling
try:
    task.poll(waker)
except Exception as e:
    assert task.is_failed()
    assert task.error() == e
```

**Thread Safety**:
- Usa `threading.Lock` para sincronización
- Safe para acceso concurrente desde múltiples threads
- Estado protegido por lock

#### 2. **TaskHandle<T>** (80 líneas)
Handle para controlar y consultar un Task desde el exterior.

**API**:
```python
@dataclass
class TaskHandle[T]:
    """TaskHandle<T> - Handle para controlar task"""
    task: Task[T]
    
    def task_id() -> TaskId
    def is_completed() -> bool
    def is_failed() -> bool
    def is_cancelled() -> bool
    def is_pending() -> bool
    def result() -> T
    def error() -> Optional[Exception]
    def cancel() -> bool
```

**Ejemplo de uso**:
```python
# Obtener handle de executor
handle = executor.spawn(future)

# Consultar estado
if handle.is_completed():
    result = handle.result()

# Cancelar
handle.cancel()
```

#### 3. **Executor** (180 líneas)
Event loop principal que maneja task scheduling y polling.

**Archivo:** `src/runtime/async_runtime/executor.py`

**Estructura**:
```python
@dataclass
class Executor:
    """Executor - Event loop para ejecutar Futures"""
    ready_queue: deque[Task]           # Tareas listas para polling
    waiting: Dict[TaskId, Task]        # Tareas esperando wake-up
    wakers: Dict[TaskId, Waker]        # Despertadores por task
    _lock: Lock
    _running: bool
    
    def spawn(future: Future[T]) -> TaskHandle[T]
    def run_until_complete(future: Future[T], timeout: Optional[float]) -> T
    def run(max_iterations: Optional[int]) -> None
    def step() -> bool
    def stop() -> None
    
    # Métricas
    def active_tasks() -> int
    def waiting_tasks() -> int
    def ready_tasks() -> int
```

**Event Loop Flow**:

```
┌─────────────────────────────────────┐
│         Event Loop Cycle            │
├─────────────────────────────────────┤
│                                     │
│  1. Pop task de ready_queue         │
│           ↓                         │
│  2. Poll task con waker             │
│           ↓                         │
│  3. Check resultado:                │
│      • Poll::Ready?                 │
│        → Completar task             │
│        → Eliminar de wakers         │
│        → Retornar resultado         │
│      • Poll::Pending?               │
│        → Mover a waiting            │
│        → Registrar waker            │
│        → Waker callback mueve a     │
│          ready cuando despierte     │
│           ↓                         │
│  4. Repetir (run) o retornar (step) │
│                                     │
└─────────────────────────────────────┘
```

**Waker Integration**:

Cuando se poll un task, se crea un waker que mueve el task de `waiting` a `ready_queue` cuando se despierte:

```python
def on_wake():
    with self._lock:
        # Mover de waiting a ready
        if task.task_id in self.waiting:
            del self.waiting[task.task_id]
            self.ready_queue.append(task)

waker = Waker(on_wake)
poll = task.poll(waker)

if poll.is_pending():
    # Mover a waiting y registrar waker
    self.waiting[task.task_id] = task
    self.wakers[task.task_id] = waker
```

**Ejemplos de uso**:

```python
# Crear executor
executor = Executor()

# Spawn task
future = Future.ready(42)
handle = executor.spawn(future)

# Ejecutar un paso
processed = executor.step()
if processed:
    print(f"Task completado: {handle.result()}")

# Ejecutar hasta completar
future = Future.ready(100)
result = executor.run_until_complete(future)
assert result == 100

# Run con max_iterations
executor.spawn(Future.ready(1))
executor.spawn(Future.ready(2))
executor.run(max_iterations=10)

# Metrics
print(f"Active tasks: {executor.active_tasks()}")
print(f"Waiting tasks: {executor.waiting_tasks()}")
print(f"Ready tasks: {executor.ready_tasks()}")
```

**Timeout Support**:

```python
# Timeout en run_until_complete
try:
    result = executor.run_until_complete(future, timeout=5.0)
except TimeoutError:
    print("Future did not complete within 5s")
```

**Protection contra Infinite Loops**:

Si no hay progreso después de 1000 iteraciones idle, lanza `RuntimeError`:

```python
# Si future está stuck (never completes)
future = Future.pending()
try:
    executor.run_until_complete(future)
except RuntimeError as e:
    print("Future appears to be stuck")
```

#### 4. **Runtime** (50 líneas)
Runtime global singleton que maneja el executor principal.

**API**:
```python
@dataclass
class Runtime:
    """Runtime global para async/await"""
    executor: Executor
    
    @staticmethod
    def get() -> Runtime  # Singleton
    
    def spawn(future: Future[T]) -> TaskHandle[T]
    def block_on(future: Future[T]) -> T
    def run(max_iterations: Optional[int]) -> None
    def stop() -> None
    def active_tasks() -> int
```

**Singleton Pattern**:
```python
# Variables globales
_runtime_instance: Optional[Runtime] = None
_runtime_lock = Lock()

def get_runtime() -> Runtime:
    """Thread-safe singleton"""
    global _runtime_instance
    if _runtime_instance is None:
        with _runtime_lock:
            if _runtime_instance is None:
                _runtime_instance = Runtime()
    return _runtime_instance

Runtime.get = staticmethod(get_runtime)
```

**Ejemplo de uso**:
```python
# Obtener runtime global
runtime = Runtime.get()

# Spawn task
handle = runtime.spawn(Future.ready(42))

# Block on future
result = runtime.block_on(Future.ready(100))

# Run event loop
runtime.run(max_iterations=100)
```

#### 5. **block_on()** Helper (5 líneas)
Helper function para ejecutar future sincrónicamente.

```python
def block_on(future: Future[T]) -> T:
    """
    Ejecuta un future hasta completar (blocking)
    
    Crea executor temporal y ejecuta hasta que future complete.
    """
    executor = Executor()
    return executor.run_until_complete(future)
```

**Ejemplo de uso**:
```python
# Ejecutar future sincrónicamente
result = block_on(Future.ready(42))
assert result == 42

# Con promise
promise = Promise[int]()
future = promise.future()
promise.resolve(100)
result = block_on(future)
assert result == 100
```

### Diseño Arquitectónico

#### Relación entre componentes

```
┌──────────────────────────────────────┐
│           Runtime (Singleton)        │
│  ┌─────────────────────────────────┐ │
│  │         Executor                │ │
│  │  ┌──────────────────────────┐  │ │
│  │  │   Ready Queue            │  │ │
│  │  │   [Task1, Task2, ...]    │  │ │
│  │  └──────────────────────────┘  │ │
│  │  ┌──────────────────────────┐  │ │
│  │  │   Waiting Map            │  │ │
│  │  │   {TaskId: Task}         │  │ │
│  │  └──────────────────────────┘  │ │
│  │  ┌──────────────────────────┐  │ │
│  │  │   Wakers Map             │  │ │
│  │  │   {TaskId: Waker}        │  │ │
│  │  └──────────────────────────┘  │ │
│  └─────────────────────────────────┘ │
└──────────────────────────────────────┘
         │              │
         │ spawn        │ poll
         ↓              ↓
    TaskHandle       Task<T>
         │              │
         │ control      │ wraps
         ↓              ↓
      Task<T>       Future<T>
```

#### Task Scheduling Algorithm

**Spawn**:
```
spawn(future) {
  task = Task(future)
  ready_queue.append(task)
  return TaskHandle(task)
}
```

**Step (polling)**:
```
step() {
  task = ready_queue.pop()
  if task == None:
    return False
  
  waker = Waker(on_wake = {
    waiting.remove(task.id)
    ready_queue.append(task)
  })
  
  poll = task.poll(waker)
  
  if poll.is_pending():
    waiting[task.id] = task
    wakers[task.id] = waker
  else:
    wakers.remove(task.id)
  
  return True
}
```

**Wake Flow**:
```
Promise.resolve(value) {
  waker.wake()  # Ejecuta callback
  # ↓
  # on_wake() {
  #   waiting.remove(task.id)
  #   ready_queue.append(task)
  # }
  # ↓
  # Task ahora en ready_queue
  # ↓
  # Próximo step() lo procesa
}
```

### Integración con Future/Promise

**Con Promise resolution**:
```python
executor = Executor()
promise = Promise[int]()
future = promise.future()

# Spawn
handle = executor.spawn(future)

# Step (queda pending)
executor.step()
assert executor.waiting_tasks() == 1

# Resolver promise (despierta waker)
promise.resolve(42)  # Waker mueve task a ready_queue

# Step (ahora completa)
executor.step()
assert handle.result() == 42
```

**Con Future combinators**:
```python
executor = Executor()

# Chained future
future = (Future.ready(5)
    .map(lambda x: x * 2)      # 10
    .map(lambda x: x + 5)      # 15
    .map(lambda x: x / 3))     # 5.0

result = executor.run_until_complete(future)
assert result == 5.0
```

**Con Future.all**:
```python
executor = Executor()

futures = [
    Future.ready(1),
    Future.ready(2),
    Future.ready(3)
]

all_future = Future.all(futures)
result = executor.run_until_complete(all_future)
assert result == [1, 2, 3]
```

**Con Future.race**:
```python
executor = Executor()

futures = [
    Future.ready(10),
    Future.pending(),
    Future.pending()
]

race_future = Future.race(futures)
result = executor.run_until_complete(race_future)
assert result == 10  # Primero en completar
```

### Testing

**Archivo:** `tests/unit/runtime/test_executor.py` (507 líneas)

**Cobertura:** 35 tests (100% pasando)

**Suites:**

1. **TestTask** (9 tests):
   - Creación y estados
   - Poll (ready, pending, error)
   - Result antes de completar (raises)
   - Cancelación
   - TaskId unique

2. **TestTaskHandle** (3 tests):
   - Creación de handle
   - Obtener resultado via handle
   - Cancelar via handle

3. **TestExecutor** (9 tests):
   - Creación de executor
   - Spawn task
   - Step (ready, pending)
   - run_until_complete
   - run con promise
   - Múltiples tasks
   - run con max_iterations
   - stop executor

4. **TestBlockOn** (3 tests):
   - block_on con ready
   - block_on con promise
   - block_on con map

5. **TestRuntime** (3 tests):
   - Runtime singleton
   - Spawn via Runtime
   - block_on via Runtime

6. **TestIntegration** (5 tests):
   - Chained futures
   - Future.all con executor
   - Future.race con executor
   - Promise resolution flow
   - Error propagation

7. **TestEdgeCases** (3 tests):
   - Empty executor step
   - Task double poll
   - Cancel then get result

**Resultados**:
```
35 passed in 0.09s
```

**Combined (all async runtime tests)**:
```
114 passed in 0.38s
  - 38 tests: Future/Promise (TASK-047)
  - 35 tests: Executor/Task (TASK-048)
  - 41 tests: Event system (previo)
```

## ✅ Criterios de Aceptación

- [x] Task<T> implementado con estados (PENDING, RUNNING, COMPLETED, FAILED, CANCELLED)
- [x] TaskHandle<T> para control de tasks
- [x] TaskId unique con UUID
- [x] Executor con ready_queue y waiting map
- [x] Waker registry por TaskId
- [x] spawn() para crear tasks
- [x] step() para ejecutar un paso del event loop
- [x] run() para ejecutar event loop indefinidamente
- [x] run_until_complete() con timeout support
- [x] Waker integration (wake mueve task de waiting a ready)
- [x] Runtime singleton thread-safe
- [x] block_on() helper
- [x] Task cancellation
- [x] Error handling (propagación de excepciones)
- [x] Protection contra infinite loops
- [x] Thread safety con Lock
- [x] 35 tests unitarios pasando (100%)
- [x] Integration tests con Future/Promise
- [x] Documentación completa

## 📊 Métricas

- **Archivos creados:** 3
  - `src/runtime/async_runtime/task.py` (216 líneas)
  - `src/runtime/async_runtime/executor.py` (318 líneas)
  - `tests/unit/runtime/test_executor.py` (507 líneas)
- **Archivos modificados:** 1
  - `src/runtime/async_runtime/__init__.py` (exports actualizados)
- **Total líneas:** 1,041
- **Tests:** 35 (100% passing)
- **Tiempo de ejecución tests:** 0.09s
- **Combined tests:** 73 (async runtime completo)

## 🔗 Referencias

- **Jira:** [TASK-048](https://velalang.atlassian.net/browse/VELA-580)
- **Historia:** [VELA-580 - Async/Await](https://velalang.atlassian.net/browse/VELA-580)
- **Diseño:** `docs/features/VELA-580/TASK-045.md`
- **Future/Promise:** `docs/features/VELA-580/TASK-047.md`

**Inspiraciones:**
- Tokio (Rust): Task scheduling, waker system
- async-std (Rust): block_on pattern
- JavaScript Event Loop: Ready queue, waiting queue
- Go scheduler: Task states, cancellation

## 🚀 Próximos Pasos

**TASK-049:** Tests completos de async/await (40 horas)

**Funcionalidad requerida:**
- Tests end-to-end de async/await
- Performance benchmarks
- Stress tests (1000s de tasks)
- Edge cases completos
- Integration con actors (futuro)
- Memory leak tests
- Deadlock detection tests
