"""
Tests unitarios para Worker API - Timeout

Implementación de: VELA-580 (TASK-050)
Sprint 19 - Workers y Channels

Tests:
- Worker con timeout que completa a tiempo
- Worker que excede timeout
- Worker con timeout muy corto
- Múltiples workers con diferentes timeouts
"""

import pytest
import time
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.workers import Worker, WorkerPool
from runtime.async_runtime import Executor


class TestWorkerTimeout:
    """Tests for Worker timeout functionality."""
    
    def setup_method(self):
        """Setup before each test."""
        WorkerPool._global_pool = None
        self.executor = Executor()
    
    def teardown_method(self):
        """Cleanup after each test."""
        pool = WorkerPool.get_global()
        pool.shutdown(wait=True)
    
    def test_worker_completes_within_timeout(self):
        """Test worker that completes before timeout."""
        def fast_task():
            time.sleep(0.05)  # 50ms
            return "done"
        
        future = Worker.spawn(fast_task, timeout=1.0)  # 1 second timeout
        result = self.executor.run_until_complete(future)
        
        assert result == "done"
    
    def test_worker_exceeds_timeout(self):
        """Test worker that exceeds timeout."""
        def slow_task():
            time.sleep(0.3)  # 300ms
            return "should not see this"
        
        future = Worker.spawn(slow_task, timeout=0.1)  # 100ms timeout
        
        with pytest.raises(TimeoutError, match="timeout"):
            self.executor.run_until_complete(future)
    
    def test_worker_timeout_very_short(self):
        """Test worker with very short timeout."""
        def task():
            # Even minimal work may exceed very short timeout
            time.sleep(0.01)  # 10ms
            return "done"
        
        future = Worker.spawn(task, timeout=0.001)  # 1ms timeout
        
        # Should timeout (can't complete in 1ms)
        with pytest.raises(TimeoutError):
            self.executor.run_until_complete(future)
    
    def test_worker_timeout_immediate_return(self):
        """Test worker that returns immediately with timeout."""
        def instant_task():
            return 42
        
        future = Worker.spawn(instant_task, timeout=0.1)
        result = self.executor.run_until_complete(future)
        
        assert result == 42
    
    def test_multiple_workers_different_timeouts(self):
        """Test multiple workers with different timeouts."""
        def task(sleep_time):
            time.sleep(sleep_time)
            return f"slept {sleep_time}s"
        
        # Worker 1: Fast, long timeout (OK)
        future1 = Worker.spawn(lambda: task(0.05), timeout=1.0)
        
        # Worker 2: Slow, short timeout (Timeout)
        future2 = Worker.spawn(lambda: task(0.3), timeout=0.1)
        
        # Worker 3: Medium, medium timeout (OK)
        future3 = Worker.spawn(lambda: task(0.1), timeout=0.5)
        
        # Worker 1 should succeed
        result1 = self.executor.run_until_complete(future1)
        assert "slept 0.05s" in result1
        
        # Worker 2 should timeout
        with pytest.raises(TimeoutError):
            self.executor.run_until_complete(future2)
        
        # Worker 3 should succeed
        result3 = self.executor.run_until_complete(future3)
        assert "slept 0.1s" in result3
    
    def test_worker_no_timeout(self):
        """Test worker with no timeout (timeout=None)."""
        def slow_task():
            time.sleep(0.2)  # 200ms
            return "completed"
        
        # No timeout specified
        future = Worker.spawn(slow_task)
        result = self.executor.run_until_complete(future)
        
        assert result == "completed"
    
    def test_timeout_error_message(self):
        """Test that timeout error contains useful information."""
        def slow_task():
            time.sleep(0.2)
            return "done"
        
        future = Worker.spawn(slow_task, timeout=0.05, name="my-slow-worker")
        
        with pytest.raises(TimeoutError) as exc_info:
            self.executor.run_until_complete(future)
        
        error_msg = str(exc_info.value)
        assert "timeout" in error_msg.lower()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
