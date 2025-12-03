# Worker API Technical Specification

**Version**: 1.0  
**Status**: Draft  
**Authors**: Vela Core Team  
**Date**: 2025-01-28

---

## 1. Overview

Worker API provides **CPU-parallel computation** for Vela via thread pool with async/await integration.

**Key Features**:
- Spawn functions on worker threads
- Non-blocking await via `Future<T>`
- Timeout support
- Best-effort cancellation
- Error propagation
- Resource limits (thread pool)

---

## 2. API Reference

### 2.1 Worker (Static Class)

```python
class Worker:
    """
    Static API for spawning worker tasks on thread pool.
    
    Workers execute functions on separate OS threads, enabling
    CPU parallelism for compute-intensive operations.
    """
    
    @staticmethod
    def spawn(
        func: Callable[[], T],
        *,
        timeout: Optional[float] = None,
        name: Optional[str] = None
    ) -> Future[T]:
        """
        Spawn function on worker thread.
        
        Args:
            func: Zero-argument function to execute on worker.
                  Use closures to capture required data.
            timeout: Optional timeout in seconds. Worker fails with
                     TimeoutError if exceeds this duration.
            name: Optional worker name for debugging.
            
        Returns:
            Future[T] that resolves when worker completes.
            
        Raises:
            TimeoutError: If worker exceeds timeout (via Future rejection)
            Exception: If func() raises exception (via Future rejection)
            
        Example:
            >>> def compute() -> int:
            ...     return sum(range(1000000))
            >>> future = Worker.spawn(compute, timeout=5.0)
            >>> result = await future  # 499999500000
        """
        pass
    
    @staticmethod
    def spawn_all(
        funcs: List[Callable[[], T]],
        *,
        timeout: Optional[float] = None
    ) -> Future[List[T]]:
        """
        Spawn multiple workers and wait for all to complete.
        
        Convenience method equivalent to:
            await Future.all([Worker.spawn(f) for f in funcs])
        
        Args:
            funcs: List of functions to spawn
            timeout: Per-worker timeout (not total timeout)
            
        Returns:
            Future[List[T]] with all results in order
            
        Example:
            >>> def task(n: int) -> int:
            ...     return n * 2
            >>> futures = Worker.spawn_all([
            ...     lambda: task(1),
            ...     lambda: task(2),
            ...     lambda: task(3)
            ... ])
            >>> results = await futures  # [2, 4, 6]
        """
        pass
    
    @staticmethod
    def get_pool() -> 'WorkerPool':
        """Get global worker pool (singleton)."""
        pass
    
    @staticmethod
    def configure_pool(max_workers: Optional[int] = None) -> None:
        """
        Configure worker pool (must be called before first spawn).
        
        Args:
            max_workers: Maximum concurrent workers.
                         Default: os.cpu_count() or 4
        """
        pass
```

---

### 2.2 WorkerHandle (Internal)

```python
class WorkerHandle:
    """
    Internal handle to track running worker state.
    
    Users don't interact with WorkerHandle directly; it's used
    internally by Worker and WorkerPool.
    """
    
    def __init__(
        self,
        worker_id: int,
        promise: Promise[T],
        timeout: Optional[float] = None,
        name: Optional[str] = None
    ):
        """
        Initialize worker handle.
        
        Args:
            worker_id: Unique worker ID
            promise: Promise to resolve when worker completes
            timeout: Optional timeout in seconds
            name: Optional worker name for debugging
        """
        self.worker_id = worker_id
        self.promise = promise
        self.timeout = timeout
        self.name = name or f"worker-{worker_id}"
        self.start_time = time.time()
        self.cancelled = False
        self.completed = False
    
    def complete(self, result: T) -> None:
        """
        Mark worker as successfully completed.
        
        Args:
            result: Result value from worker function
        """
        if not self.cancelled and not self.completed:
            self.completed = True
            self.promise.resolve(result)
    
    def fail(self, error: Exception) -> None:
        """
        Mark worker as failed with error.
        
        Args:
            error: Exception raised by worker function
        """
        if not self.cancelled and not self.completed:
            self.completed = True
            self.promise.reject(error)
    
    def cancel(self) -> bool:
        """
        Cancel worker (best-effort).
        
        Note: Python threads cannot be killed. This marks the handle
        as cancelled, preventing Promise resolution, but the underlying
        thread may continue executing.
        
        Returns:
            True if cancellation was successful (handle marked cancelled)
        """
        if not self.completed:
            self.cancelled = True
            self.promise.reject(CancellationError(f"Worker {self.name} cancelled"))
            return True
        return False
    
    def is_timeout(self) -> bool:
        """Check if worker has exceeded timeout."""
        if self.timeout is None:
            return False
        elapsed = time.time() - self.start_time
        return elapsed > self.timeout
    
    def elapsed(self) -> float:
        """Get elapsed time since worker started."""
        return time.time() - self.start_time
```

---

### 2.3 WorkerPool (Singleton)

```python
class WorkerPool:
    """
    Thread pool for executing worker tasks.
    
    WorkerPool manages a fixed-size pool of OS threads that execute
    worker functions. Tasks are queued if all threads are busy.
    
    Singleton pattern: Only one pool exists per process.
    """
    
    def __init__(self, max_workers: Optional[int] = None):
        """
        Initialize worker pool.
        
        Args:
            max_workers: Maximum concurrent workers.
                         Default: os.cpu_count() or 4
        """
        self.max_workers = max_workers or os.cpu_count() or 4
        self.executor = concurrent.futures.ThreadPoolExecutor(
            max_workers=self.max_workers,
            thread_name_prefix="vela-worker"
        )
        self.active_handles: Dict[int, WorkerHandle] = {}
        self.next_worker_id = 0
        self.lock = threading.Lock()
        self._shutdown = False
    
    def submit(self, func: Callable[[], T], handle: WorkerHandle) -> None:
        """
        Submit function to thread pool.
        
        Args:
            func: Function to execute
            handle: WorkerHandle to track execution
            
        Raises:
            RuntimeError: If pool is shutdown
        """
        if self._shutdown:
            raise RuntimeError("WorkerPool is shutdown")
        
        with self.lock:
            self.active_handles[handle.worker_id] = handle
        
        # Submit to thread pool
        future = self.executor.submit(self._execute_with_handle, func, handle)
    
    def _execute_with_handle(self, func: Callable[[], T], handle: WorkerHandle) -> None:
        """
        Execute function and notify handle of result.
        
        This runs on worker thread.
        """
        try:
            # Check timeout before execution
            if handle.is_timeout():
                raise TimeoutError(
                    f"Worker {handle.name} timeout after {handle.elapsed():.2f}s"
                )
            
            # Execute function
            result = func()
            
            # Check timeout after execution
            if handle.is_timeout():
                raise TimeoutError(
                    f"Worker {handle.name} timeout after {handle.elapsed():.2f}s"
                )
            
            # Complete successfully
            handle.complete(result)
            
        except Exception as e:
            # Fail with error
            handle.fail(e)
            
        finally:
            # Remove from active handles
            with self.lock:
                self.active_handles.pop(handle.worker_id, None)
    
    def get_active_count(self) -> int:
        """Get number of currently active workers."""
        with self.lock:
            return len(self.active_handles)
    
    def get_active_handles(self) -> List[WorkerHandle]:
        """Get list of currently active worker handles."""
        with self.lock:
            return list(self.active_handles.values())
    
    def shutdown(self, wait: bool = True, cancel_pending: bool = False) -> None:
        """
        Shutdown worker pool.
        
        Args:
            wait: If True, wait for all active workers to complete
            cancel_pending: If True, cancel pending workers (best-effort)
        """
        self._shutdown = True
        
        if cancel_pending:
            with self.lock:
                for handle in list(self.active_handles.values()):
                    handle.cancel()
        
        self.executor.shutdown(wait=wait)
    
    def __del__(self):
        """Ensure clean shutdown on garbage collection."""
        if not self._shutdown:
            self.shutdown(wait=False)
```

---

## 3. Usage Examples

### 3.1 Basic Worker Spawn

```vela
# Vela code
async fn main() -> void {
  # Spawn worker for CPU-intensive task
  future: Future<Number> = Worker.spawn(() => {
    # Heavy computation on worker thread
    sum: Number = 0
    (0..1000000).forEach(i => {
      sum = sum + i
    })
    return sum
  })
  
  # Await result (non-blocking on main thread)
  result: Number = await future
  print("Result: ${result}")
}
```

```python
# Python implementation
import asyncio
from src.runtime.workers import Worker
from src.runtime.async_runtime import Executor

async def main():
    # Spawn worker
    def compute():
        return sum(range(1000000))
    
    future = Worker.spawn(compute)
    
    # Await result
    result = await future.to_awaitable()  # 499999500000
    print(f"Result: {result}")

# Run
executor = Executor.get_global()
executor.block_on(main())
```

### 3.2 Parallel Workers with Future.all

```vela
async fn process_images(paths: List<String>) -> List<Image> {
  # Spawn worker per image
  futures = paths.map(path => 
    Worker.spawn(() => Image.decode(path))
  )
  
  # Wait for all workers to complete
  images = await Future.all(futures)
  return images
}
```

### 3.3 Worker with Timeout

```vela
async fn fetch_with_timeout(url: String) -> Result<Data> {
  future = Worker.spawn(
    () => http_fetch(url),
    timeout=5.0
  )
  
  try {
    data = await future
    return Ok(data)
  } catch (e: TimeoutError) {
    return Err(Error("Request timeout"))
  }
}
```

### 3.4 Worker Cancellation

```vela
async fn cancellable_task() -> Option<Result> {
  future = Worker.spawn(() => very_slow_computation())
  
  # User cancels task
  if user_cancelled {
    future.cancel()
    return None
  }
  
  try {
    result = await future
    return Some(result)
  } catch (e: CancellationError) {
    return None
  }
}
```

### 3.5 Error Handling

```vela
async fn risky_worker() -> Result<Data> {
  future = Worker.spawn(() => {
    if random() < 0.5 {
      throw Error("Random failure")
    }
    return Data { value: 42 }
  })
  
  match await future {
    Ok(data) => {
      print("Success: ${data.value}")
      return Ok(data)
    }
    Err(error) => {
      print("Worker failed: ${error}")
      return Err(error)
    }
  }
}
```

---

## 4. Threading Model

### 4.1 Thread Pool Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      WorkerPool                          │
│  ┌────────────────────────────────────────────────────┐ │
│  │  ThreadPoolExecutor (max_workers=CPU_COUNT)        │ │
│  │                                                      │ │
│  │  Thread 1  Thread 2  Thread 3  Thread 4            │ │
│  │     ↓         ↓         ↓         ↓                 │ │
│  │  [Task A] [Task B] [Task C] [Task D]              │ │
│  │                                                      │ │
│  │  Queue: [Task E, Task F, Task G, ...]             │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
         ↓                    ↓                    ↓
    [Future A]           [Future B]           [Future C]
         ↓                    ↓                    ↓
   (await result)       (await result)       (await result)
```

### 4.2 Execution Flow

```
1. User calls Worker.spawn(func)
   ↓
2. Worker creates Promise and WorkerHandle
   ↓
3. Worker submits func + handle to WorkerPool
   ↓
4. WorkerPool queues task (if pool full) or assigns to free thread
   ↓
5. Worker thread executes func()
   ↓
6. On completion: handle.complete(result) → promise.resolve(result)
   On error:      handle.fail(error)    → promise.reject(error)
   ↓
7. Future resolves/rejects
   ↓
8. User receives result via await
```

### 4.3 Timeout Handling

```python
def _execute_with_handle(func, handle):
    try:
        # Check timeout BEFORE execution
        if handle.is_timeout():
            raise TimeoutError(...)
        
        # Execute
        result = func()
        
        # Check timeout AFTER execution
        if handle.is_timeout():
            raise TimeoutError(...)
        
        handle.complete(result)
    except Exception as e:
        handle.fail(e)
```

⚠️ **Limitation**: Timeout is checked before/after execution, NOT during. Long-running functions must check timeout internally.

### 4.4 Cancellation Flow

```
1. User calls future.cancel()
   ↓
2. Future finds WorkerHandle (stored during spawn)
   ↓
3. handle.cancel() marks handle as cancelled
   ↓
4. promise.reject(CancellationError)
   ↓
5. Worker thread continues executing (can't be killed)
   BUT handle.complete/fail checks cancelled flag and ignores result
```

---

## 5. Performance Characteristics

### 5.1 Overhead Analysis

| Operation | Cost | Notes |
|-----------|------|-------|
| Worker.spawn() | ~0.5-1ms | Promise creation + pool submission |
| Thread start | ~3-5ms | OS thread startup (amortized via pool) |
| Thread context switch | ~1-10μs | OS scheduler overhead |
| Future await | ~0.1ms | Poll + waker registration |
| Cancellation | ~0.05ms | Handle flag + Promise rejection |

### 5.2 Scalability

**Thread Pool Size**:
- Default: `os.cpu_count()` (optimal for CPU-bound)
- Max recommended: 2-4x CPU count (for mixed CPU/IO)
- Memory per thread: ~8MB (OS stack + Python interpreter)

**Queue Depth**:
- Unlimited (Python's ThreadPoolExecutor uses unbounded queue)
- Risk: Memory exhaustion if spawn rate >> completion rate

**Throughput**:
- Target: > 100 workers/sec spawn rate
- Bottleneck: Thread availability, not spawn overhead

### 5.3 Python GIL Impact

⚠️ **Critical Limitation**: Python's Global Interpreter Lock (GIL) limits true parallelism for pure Python code.

**Workarounds**:
1. **CPU-bound with C extensions**: NumPy, image processing, crypto (release GIL)
2. **IO-bound**: Network, disk (GIL released during IO)
3. **Future**: Cython workers, PyPy, or native Rust backend

**Measurement**:
```python
# Pure Python (GIL-bound)
def compute():
    return sum(range(1000000))

# Parallelism: ~1.2x (not 4x with 4 cores)

# NumPy (GIL-free)
import numpy as np
def compute_np():
    return np.sum(np.arange(1000000))

# Parallelism: ~3.8x (near-linear scaling)
```

---

## 6. Error Handling

### 6.1 Exception Types

```python
# TimeoutError: Worker exceeded timeout
future = Worker.spawn(slow_func, timeout=1.0)
try:
    result = await future
except TimeoutError as e:
    print(f"Timeout: {e}")

# CancellationError: Worker was cancelled
future = Worker.spawn(long_func)
future.cancel()
try:
    result = await future
except CancellationError as e:
    print(f"Cancelled: {e}")

# RuntimeError: Pool shutdown
Worker.get_pool().shutdown()
future = Worker.spawn(func)  # RuntimeError: WorkerPool is shutdown

# Custom exceptions: Propagated from worker
def risky_func():
    raise ValueError("Invalid input")

future = Worker.spawn(risky_func)
try:
    result = await future
except ValueError as e:
    print(f"Worker error: {e}")
```

### 6.2 Error Propagation

```
Worker thread:
  func() raises Exception
       ↓
  handle.fail(exception)
       ↓
  promise.reject(exception)
       ↓
Main thread:
  future polling detects rejection
       ↓
  await raises exception
```

### 6.3 Partial Failure in Future.all

```vela
# Spawn 3 workers, 1 fails
futures = [
  Worker.spawn(() => compute(1)),  # OK
  Worker.spawn(() => compute(2)),  # Throws error
  Worker.spawn(() => compute(3))   # OK
]

try {
  results = await Future.all(futures)
} catch (error) {
  # Future.all fails fast on first error
  # Other workers may still be running
}
```

**Behavior**:
- `Future.all` rejects on first failure
- Other workers continue executing (can't be killed)
- Use `Future.allSettled` to wait for all regardless of errors

---

## 7. Best Practices

### 7.1 When to Use Workers

✅ **Use Workers For**:
- CPU-intensive computation (image processing, compression, crypto)
- Blocking operations (legacy sync APIs)
- Parallel data processing (map over large datasets)

❌ **Don't Use Workers For**:
- IO-bound operations (use async/await instead)
- Lightweight tasks (spawn overhead > task time)
- Shared state (workers can't share memory safely)

### 7.2 Closure Capture

Workers execute on separate threads, so **capture all needed data in closure**:

```vela
# ✅ GOOD: Capture value in closure
data: String = load_data()
future = Worker.spawn(() => process(data))

# ❌ BAD: Reference external mutable state
state count: Number = 0
future = Worker.spawn(() => {
  count = count + 1  # Race condition!
})
```

### 7.3 Resource Management

```vela
# Configure pool before first spawn
Worker.configure_pool(max_workers=8)

# Cleanup on shutdown
Worker.get_pool().shutdown(wait=True)
```

### 7.4 Timeout Guidelines

```vela
# Conservative timeout for critical paths
future = Worker.spawn(func, timeout=30.0)

# Aggressive timeout for best-effort
future = Worker.spawn(func, timeout=1.0)
```

### 7.5 Error Handling Patterns

```vela
# Pattern 1: Match on Result
match await Worker.spawn(risky_func) {
  Ok(value) => handle_success(value)
  Err(error) => handle_error(error)
}

# Pattern 2: Try-catch
try {
  result = await Worker.spawn(risky_func)
  handle_success(result)
} catch (error) {
  handle_error(error)
}

# Pattern 3: Fallback
result = await Worker.spawn(func).unwrapOr(default_value)
```

---

## 8. Implementation Notes

### 8.1 Thread Safety

**Thread-safe components**:
- `WorkerPool.active_handles` (protected by lock)
- `Promise.resolve/reject` (atomic state transition)
- `WorkerHandle.cancel` (atomic cancelled flag)

**Non-thread-safe components**:
- User-provided closures (user responsibility)

### 8.2 Memory Management

**Automatic cleanup**:
- Worker threads reused (pool)
- WorkerHandle removed from active_handles on completion
- Promise/Future GC'd when no references

**Potential leaks**:
- Long-running workers hold references
- Cancelled workers continue executing (Python limitation)

### 8.3 Testing Strategy

**Unit tests**:
- Spawn single worker, verify result
- Spawn with timeout, verify TimeoutError
- Cancel worker, verify CancellationError
- Worker error, verify exception propagation
- Pool limits, verify queueing

**Integration tests**:
- Future.all with multiple workers
- Workers communicating via channels (TASK-051)
- Real-world workloads (image processing, etc.)

**Performance tests**:
- Spawn throughput (workers/sec)
- Latency (spawn to start)
- Scalability (1 to N workers)
- GIL contention measurement

---

## 9. Future Work

### 9.1 Planned Enhancements

**TASK-051: Channels**
- Workers communicate via `Channel<T>`
- Enables producer/consumer patterns

**TASK-037-041: Actor Model**
- Workers evolve into actors with mailboxes
- Supervision hierarchies
- Location transparency

**Native Backend**:
- Rust/C++ backend for GIL-free workers
- True parallelism for pure computation

**Work-Stealing Scheduler**:
- Better load balancing across threads
- Reduce idle time

### 9.2 Open Questions

1. **Thread pool sizing**: Auto-tune based on workload?
2. **Priority scheduling**: High-priority workers first?
3. **Affinity**: Pin workers to specific CPU cores?
4. **Monitoring**: Metrics for worker utilization, queue depth, latency?

---

## 10. Conclusion

Worker API provides **simple, effective CPU parallelism** for Vela:

✅ **Strengths**:
- Minimal API surface (`Worker.spawn()`)
- Native async/await integration
- Resource control via thread pool
- Robust error handling

⚠️ **Limitations**:
- Python GIL limits parallelism
- No true thread cancellation
- Thread-per-worker model (not green threads)

**Next**: Implement `src/runtime/workers/` with full test coverage.
