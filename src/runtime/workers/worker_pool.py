"""
WorkerPool - Thread Pool for Executing Worker Tasks

Implementación de: VELA-580 (TASK-050)
Sprint 19 - Workers y Channels

WorkerPool gestiona un pool de threads OS para ejecutar workers:
- Fixed-size pool (default: CPU count)
- Queue-based task distribution
- Graceful shutdown support
- Thread reuse (no per-task thread creation)

Thread-safety: All operations are thread-safe.
Singleton pattern: Only one pool exists per process.
"""

from typing import Callable, Dict, List, Optional, TypeVar
import concurrent.futures
import threading
import os
from .worker_handle import WorkerHandle


T = TypeVar('T')


class WorkerPool:
    """
    Thread pool for executing worker tasks.
    
    WorkerPool manages a fixed-size pool of OS threads that execute
    worker functions. Tasks are queued if all threads are busy.
    
    Singleton pattern: Use Worker.get_pool() to access global instance.
    """
    
    # Global singleton instance
    _global_pool: Optional['WorkerPool'] = None
    _global_lock = threading.Lock()
    
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
    
    @classmethod
    def get_global(cls, max_workers: Optional[int] = None) -> 'WorkerPool':
        """
        Get global singleton worker pool.
        
        Args:
            max_workers: Max workers (only used on first call)
            
        Returns:
            Global WorkerPool instance
        """
        if cls._global_pool is None:
            with cls._global_lock:
                if cls._global_pool is None:
                    cls._global_pool = cls(max_workers)
        return cls._global_pool
    
    @classmethod
    def configure_global(cls, max_workers: int) -> None:
        """
        Configure global pool (must be called before first use).
        
        Args:
            max_workers: Maximum concurrent workers
            
        Raises:
            RuntimeError: If pool already initialized
        """
        with cls._global_lock:
            if cls._global_pool is not None:
                raise RuntimeError("WorkerPool already initialized. Call configure_global() before first Worker.spawn()")
            cls._global_pool = cls(max_workers)
    
    def get_next_worker_id(self) -> int:
        """
        Get next unique worker ID.
        
        Returns:
            Unique worker ID
            
        Thread-safe.
        """
        with self.lock:
            worker_id = self.next_worker_id
            self.next_worker_id += 1
            return worker_id
    
    def submit(self, func: Callable[[], T], handle: WorkerHandle) -> None:
        """
        Submit function to thread pool.
        
        Args:
            func: Function to execute
            handle: WorkerHandle to track execution
            
        Raises:
            RuntimeError: If pool is shutdown
            
        Thread-safe.
        """
        if self._shutdown:
            raise RuntimeError("WorkerPool is shutdown")
        
        # Add to active handles
        with self.lock:
            self.active_handles[handle.worker_id] = handle
        
        # Submit to thread pool
        # Note: ThreadPoolExecutor.submit() is thread-safe
        self.executor.submit(self._execute_with_handle, func, handle)
    
    def _execute_with_handle(self, func: Callable[[], T], handle: WorkerHandle) -> None:
        """
        Execute function and notify handle of result.
        
        This runs on worker thread.
        
        Args:
            func: Function to execute
            handle: WorkerHandle to notify
        """
        try:
            # Check timeout before execution
            if handle.is_timeout():
                raise TimeoutError(
                    f"Worker {handle.name} timeout after {handle.elapsed():.2f}s"
                )
            
            # Check if already cancelled
            if handle.is_cancelled():
                # Don't execute, already cancelled
                return
            
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
            # Fail with error (includes TimeoutError, user exceptions, etc.)
            handle.fail(e)
            
        finally:
            # Remove from active handles
            with self.lock:
                self.active_handles.pop(handle.worker_id, None)
    
    def get_active_count(self) -> int:
        """
        Get number of currently active workers.
        
        Returns:
            Number of active workers
            
        Thread-safe.
        """
        with self.lock:
            return len(self.active_handles)
    
    def get_active_handles(self) -> List[WorkerHandle]:
        """
        Get list of currently active worker handles.
        
        Returns:
            List of active WorkerHandle instances
            
        Thread-safe.
        """
        with self.lock:
            return list(self.active_handles.values())
    
    def cancel_all(self) -> int:
        """
        Cancel all active workers (best-effort).
        
        Returns:
            Number of workers cancelled
            
        Thread-safe.
        """
        with self.lock:
            handles = list(self.active_handles.values())
        
        count = 0
        for handle in handles:
            if handle.cancel():
                count += 1
        
        return count
    
    def shutdown(self, wait: bool = True, cancel_pending: bool = False) -> None:
        """
        Shutdown worker pool.
        
        Args:
            wait: If True, wait for all active workers to complete
            cancel_pending: If True, cancel pending workers (best-effort)
            
        Thread-safe.
        """
        self._shutdown = True
        
        if cancel_pending:
            self.cancel_all()
        
        # Shutdown thread pool executor
        self.executor.shutdown(wait=wait)
    
    def is_shutdown(self) -> bool:
        """
        Check if pool is shutdown.
        
        Returns:
            True if shutdown, False otherwise
        """
        return self._shutdown
    
    def __del__(self):
        """Ensure clean shutdown on garbage collection."""
        if not self._shutdown:
            self.shutdown(wait=False)
    
    def __repr__(self) -> str:
        """String representation for debugging."""
        return (
            f"WorkerPool(max_workers={self.max_workers}, "
            f"active={self.get_active_count()}, "
            f"shutdown={self._shutdown})"
        )
