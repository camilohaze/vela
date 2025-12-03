"""
Tests unitarios para Worker API - Cancellation

Implementación de: VELA-580 (TASK-050)
Sprint 19 - Workers y Channels

Tests:
- Cancel worker antes de completar
- Cancel worker ya completado (no-op)
- Cancel múltiples workers
- Cancellation es best-effort (thread continúa)
"""

import pytest
import time
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.workers import Worker, WorkerPool
from runtime.workers.worker_handle import CancellationError
from runtime.async_runtime import Executor


class TestWorkerCancellation:
    """Tests for Worker cancellation."""
    
    def setup_method(self):
        """Setup before each test."""
        WorkerPool._global_pool = None
        self.executor = Executor()
    
    def teardown_method(self):
        """Cleanup after each test."""
        pool = WorkerPool.get_global()
        pool.shutdown(wait=True)
    
    def test_cancel_worker_before_completion(self):
        """Test cancelling worker before it completes."""
        def slow_task():
            time.sleep(0.5)  # 500ms
            return "should not see this"
        
        future = Worker.spawn(slow_task)
        
        # Cancel immediately
        cancelled = future.cancel()
        
        assert cancelled is True
        
        # Future should reject with CancellationError
        with pytest.raises(CancellationError, match="cancelled"):
            self.executor.run_until_complete(future)
    
    def test_cancel_already_completed_worker(self):
        """Test cancelling worker that already completed (no-op)."""
        def fast_task():
            return 42
        
        future = Worker.spawn(fast_task)
        
        # Wait for completion
        result = self.executor.run_until_complete(future)
        assert result == 42
        
        # Try to cancel (should be no-op)
        cancelled = future.cancel()
        
        # Cancellation should fail (already completed)
        assert cancelled is False
    
    def test_cancel_multiple_workers(self):
        """Test cancelling multiple workers."""
        def slow_task(n):
            time.sleep(0.5)
            return n
        
        # Spawn multiple slow workers
        futures = [
            Worker.spawn(lambda i=i: slow_task(i))
            for i in range(3)
        ]
        
        # Cancel all
        for future in futures:
            future.cancel()
        
        # All should raise CancellationError
        for future in futures:
            with pytest.raises(CancellationError):
                self.executor.run_until_complete(future)
    
    def test_cancel_some_workers_not_others(self):
        """Test cancelling some workers but not others."""
        def task(n):
            time.sleep(0.1)
            return n * 2
        
        future1 = Worker.spawn(lambda: task(1))
        future2 = Worker.spawn(lambda: task(2))
        future3 = Worker.spawn(lambda: task(3))
        
        # Cancel only future2
        future2.cancel()
        
        # future1 and future3 should complete normally
        result1 = self.executor.run_until_complete(future1)
        result3 = self.executor.run_until_complete(future3)
        
        assert result1 == 2
        assert result3 == 6
        
        # future2 should be cancelled
        with pytest.raises(CancellationError):
            self.executor.run_until_complete(future2)
    
    def test_cancellation_error_message(self):
        """Test that cancellation error contains useful information."""
        def task():
            time.sleep(0.5)
            return "done"
        
        future = Worker.spawn(task, name="my-worker")
        future.cancel()
        
        with pytest.raises(CancellationError) as exc_info:
            self.executor.run_until_complete(future)
        
        error_msg = str(exc_info.value)
        assert "cancel" in error_msg.lower()
    
    def test_pool_cancel_all(self):
        """Test WorkerPool.cancel_all() method."""
        def slow_task():
            time.sleep(0.5)
            return "done"
        
        pool = Worker.get_pool()
        
        # Spawn multiple workers
        futures = [Worker.spawn(slow_task) for _ in range(3)]
        
        # Give workers time to start
        time.sleep(0.05)
        
        # Cancel all via pool
        count = pool.cancel_all()
        
        # Should have cancelled 3 workers
        assert count == 3
        
        # All futures should raise CancellationError
        for future in futures:
            with pytest.raises(CancellationError):
                self.executor.run_until_complete(future)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
