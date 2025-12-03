"""
WorkerHandle - Internal Handle for Worker State Tracking

Implementación de: VELA-580 (TASK-050)
Sprint 19 - Workers y Channels

WorkerHandle rastrea el estado de un worker en ejecución:
- worker_id: ID único del worker
- promise: Promise<T> para resolver cuando worker termina
- timeout: Timeout opcional
- cancelled: Flag de cancelación
- start_time: Timestamp de inicio
"""

from typing import TypeVar, Optional
import time
import threading


T = TypeVar('T')


class CancellationError(Exception):
    """Exception raised when worker is cancelled."""
    pass


class WorkerHandle:
    """
    Internal handle to track running worker state.
    
    Users don't interact with WorkerHandle directly; it's used
    internally by Worker and WorkerPool.
    
    Thread-safe: All state mutations protected by lock.
    """
    
    def __init__(
        self,
        worker_id: int,
        promise: 'Promise[T]',  # type: ignore
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
        self._lock = threading.Lock()
    
    def complete(self, result: T) -> None:
        """
        Mark worker as successfully completed.
        
        Args:
            result: Result value from worker function
            
        Thread-safe: Can be called from worker thread.
        """
        with self._lock:
            if not self.cancelled and not self.completed:
                self.completed = True
                self.promise.resolve(result)
    
    def fail(self, error: Exception) -> None:
        """
        Mark worker as failed with error.
        
        Args:
            error: Exception raised by worker function
            
        Thread-safe: Can be called from worker thread.
        """
        with self._lock:
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
            
        Thread-safe: Can be called from any thread.
        """
        with self._lock:
            if not self.completed:
                self.cancelled = True
                self.promise.reject(CancellationError(f"Worker {self.name} cancelled"))
                return True
            return False
    
    def is_timeout(self) -> bool:
        """
        Check if worker has exceeded timeout.
        
        Returns:
            True if worker exceeded timeout, False otherwise
        """
        if self.timeout is None:
            return False
        elapsed = time.time() - self.start_time
        return elapsed > self.timeout
    
    def elapsed(self) -> float:
        """
        Get elapsed time since worker started.
        
        Returns:
            Elapsed time in seconds
        """
        return time.time() - self.start_time
    
    def is_cancelled(self) -> bool:
        """
        Check if worker is cancelled.
        
        Returns:
            True if cancelled, False otherwise
        """
        with self._lock:
            return self.cancelled
    
    def is_completed(self) -> bool:
        """
        Check if worker is completed.
        
        Returns:
            True if completed (success or failure), False otherwise
        """
        with self._lock:
            return self.completed
    
    def __repr__(self) -> str:
        """String representation for debugging."""
        state = "completed" if self.completed else "cancelled" if self.cancelled else "running"
        elapsed = self.elapsed()
        return f"WorkerHandle(id={self.worker_id}, name='{self.name}', state={state}, elapsed={elapsed:.3f}s)"
