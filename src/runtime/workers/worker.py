"""
Worker - Static API for Spawning Workers

Implementación de: VELA-580 (TASK-050)
Sprint 19 - Workers y Channels

Worker proporciona API estática para spawn de workers en thread pool:
- Worker.spawn(func): Spawn función en worker thread
- Worker.spawn_all(funcs): Spawn múltiples workers en paralelo
- Worker.get_pool(): Acceder al pool global
- Worker.configure_pool(max_workers): Configurar pool

Inspirado en:
- Rust: tokio::task::spawn_blocking, rayon::spawn
- Go: go func() { ... }
- Swift: Task.detached { ... }
"""

from typing import TypeVar, Callable, List, Optional
from ..async_runtime import Future, Promise
from .worker_handle import WorkerHandle
from .worker_pool import WorkerPool


T = TypeVar('T')


class Worker:
    """
    Static API for spawning worker tasks on thread pool.
    
    Workers execute functions on separate OS threads, enabling
    CPU parallelism for compute-intensive operations.
    
    Example:
        >>> def compute() -> int:
        ...     return sum(range(1000000))
        >>> future = Worker.spawn(compute)
        >>> result = await future  # 499999500000
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
            RuntimeError: If pool is shutdown
            
        Example:
            >>> def compute() -> int:
            ...     return sum(range(1000000))
            >>> future = Worker.spawn(compute, timeout=5.0)
            >>> result = await future
        """
        # Get global pool
        pool = WorkerPool.get_global()
        
        # Create Promise + Future
        promise = Promise[T]()
        future = promise.future()  # FIX: Use future() instead of get_future()
        
        # Create WorkerHandle
        worker_id = pool.get_next_worker_id()
        handle = WorkerHandle(
            worker_id=worker_id,
            promise=promise,
            timeout=timeout,
            name=name
        )
        
        # Attach handle to future for cancellation support
        # This allows tests/users to cancel via future.cancel()
        future._worker_handle = handle
        
        # Submit to pool
        pool.submit(func, handle)
        
        return future
    
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
        # Spawn all workers
        futures = [
            Worker.spawn(func, timeout=timeout)
            for func in funcs
        ]
        
        # Return Future.all
        return Future.all(futures)
    
    @staticmethod
    def get_pool() -> WorkerPool:
        """
        Get global worker pool (singleton).
        
        Returns:
            Global WorkerPool instance
        """
        return WorkerPool.get_global()
    
    @staticmethod
    def configure_pool(max_workers: int) -> None:
        """
        Configure worker pool (must be called before first spawn).
        
        Args:
            max_workers: Maximum concurrent workers.
                         
        Raises:
            RuntimeError: If pool already initialized
            
        Example:
            >>> Worker.configure_pool(max_workers=8)
            >>> # Now all Worker.spawn() calls use 8-worker pool
        """
        WorkerPool.configure_global(max_workers)
