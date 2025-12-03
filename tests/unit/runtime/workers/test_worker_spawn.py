"""
Tests unitarios para Worker API - Spawn Básico

Implementación de: VELA-580 (TASK-050)
Sprint 19 - Workers y Channels

Tests:
- Spawn worker simple
- Spawn worker con resultado
- Spawn múltiples workers
- Worker spawn con error
- Worker spawn con captura de closure
"""

import pytest
import time
import sys
import os

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.workers import Worker, WorkerPool
from runtime.async_runtime import Executor


class TestWorkerSpawn:
    """Tests for basic Worker.spawn() functionality."""
    
    def setup_method(self):
        """Setup before each test."""
        # Reset global pool
        WorkerPool._global_pool = None
        # Create executor for running async code
        self.executor = Executor()
    
    def teardown_method(self):
        """Cleanup after each test."""
        # Shutdown pool
        pool = WorkerPool.get_global()
        pool.shutdown(wait=True)
    
    def test_spawn_simple_worker(self):
        """Test spawning simple worker that returns value."""
        def compute():
            return 42
        
        # Spawn worker
        future = Worker.spawn(compute)
        
        # Await result
        result = self.executor.run_until_complete(future)
        
        assert result == 42
    
    def test_spawn_worker_with_computation(self):
        """Test spawning worker with actual computation."""
        def compute_sum():
            return sum(range(100))
        
        future = Worker.spawn(compute_sum)
        result = self.executor.run_until_complete(future)
        
        # sum(0..99) = 4950
        assert result == 4950
    
    def test_spawn_worker_with_sleep(self):
        """Test spawning worker that sleeps (simulates long-running task)."""
        def slow_task():
            time.sleep(0.1)  # 100ms
            return "done"
        
        start = time.time()
        future = Worker.spawn(slow_task)
        result = self.executor.run_until_complete(future)
        elapsed = time.time() - start
        
        assert result == "done"
        assert elapsed >= 0.1  # At least 100ms
    
    def test_spawn_multiple_workers_sequential(self):
        """Test spawning multiple workers sequentially."""
        def task(n):
            return n * 2
        
        # Spawn workers sequentially
        future1 = Worker.spawn(lambda: task(1))
        future2 = Worker.spawn(lambda: task(2))
        future3 = Worker.spawn(lambda: task(3))
        
        # Await results
        result1 = self.executor.run_until_complete(future1)
        result2 = self.executor.run_until_complete(future2)
        result3 = self.executor.run_until_complete(future3)
        
        assert result1 == 2
        assert result2 == 4
        assert result3 == 6
    
    def test_spawn_worker_with_closure_capture(self):
        """Test worker capturing variables from closure."""
        x = 10
        y = 20
        
        def compute():
            return x + y
        
        future = Worker.spawn(compute)
        result = self.executor.run_until_complete(future)
        
        assert result == 30
    
    def test_spawn_worker_with_complex_closure(self):
        """Test worker with complex closure capture."""
        data = [1, 2, 3, 4, 5]
        multiplier = 3
        
        def process():
            return sum(x * multiplier for x in data)
        
        future = Worker.spawn(process)
        result = self.executor.run_until_complete(future)
        
        # (1+2+3+4+5) * 3 = 45
        assert result == 45
    
    def test_spawn_worker_returns_none(self):
        """Test worker that returns None."""
        def no_return():
            pass  # Returns None implicitly
        
        future = Worker.spawn(no_return)
        result = self.executor.run_until_complete(future)
        
        assert result is None
    
    def test_spawn_worker_returns_tuple(self):
        """Test worker that returns tuple."""
        def compute_tuple():
            return (1, 2, 3)
        
        future = Worker.spawn(compute_tuple)
        result = self.executor.run_until_complete(future)
        
        assert result == (1, 2, 3)
    
    def test_spawn_worker_returns_dict(self):
        """Test worker that returns dict."""
        def compute_dict():
            return {"key": "value", "number": 42}
        
        future = Worker.spawn(compute_dict)
        result = self.executor.run_until_complete(future)
        
        assert result == {"key": "value", "number": 42}
    
    def test_spawn_worker_with_named_worker(self):
        """Test spawning worker with custom name."""
        def compute():
            return 100
        
        future = Worker.spawn(compute, name="my-worker")
        result = self.executor.run_until_complete(future)
        
        assert result == 100


class TestWorkerSpawnAll:
    """Tests for Worker.spawn_all() convenience method."""
    
    def setup_method(self):
        """Setup before each test."""
        WorkerPool._global_pool = None
        self.executor = Executor()
    
    def teardown_method(self):
        """Cleanup after each test."""
        pool = WorkerPool.get_global()
        pool.shutdown(wait=True)
    
    def test_spawn_all_empty_list(self):
        """Test spawn_all with empty list."""
        future = Worker.spawn_all([])
        result = self.executor.run_until_complete(future)
        
        assert result == []
    
    def test_spawn_all_single_worker(self):
        """Test spawn_all with single worker."""
        def task():
            return 42
        
        future = Worker.spawn_all([task])
        result = self.executor.run_until_complete(future)
        
        assert result == [42]
    
    def test_spawn_all_multiple_workers(self):
        """Test spawn_all with multiple workers."""
        def task(n):
            return n * 2
        
        funcs = [
            lambda: task(1),
            lambda: task(2),
            lambda: task(3),
            lambda: task(4)
        ]
        
        future = Worker.spawn_all(funcs)
        results = self.executor.run_until_complete(future)
        
        assert results == [2, 4, 6, 8]
    
    def test_spawn_all_parallel_execution(self):
        """Test that spawn_all executes workers in parallel."""
        def slow_task(n):
            time.sleep(0.1)  # 100ms per worker
            return n
        
        funcs = [lambda i=i: slow_task(i) for i in range(4)]
        
        start = time.time()
        future = Worker.spawn_all(funcs)
        results = self.executor.run_until_complete(future)
        elapsed = time.time() - start
        
        assert results == [0, 1, 2, 3]
        # If parallel, should take ~100ms (not 400ms sequential)
        # Allow some overhead, so check < 300ms
        assert elapsed < 0.3


class TestWorkerPool:
    """Tests for WorkerPool configuration and management."""
    
    def setup_method(self):
        """Setup before each test."""
        WorkerPool._global_pool = None
        self.executor = Executor()
    
    def teardown_method(self):
        """Cleanup after each test."""
        if WorkerPool._global_pool:
            WorkerPool._global_pool.shutdown(wait=True)
    
    def test_get_pool_singleton(self):
        """Test that get_pool() returns singleton."""
        pool1 = Worker.get_pool()
        pool2 = Worker.get_pool()
        
        assert pool1 is pool2
    
    def test_configure_pool_max_workers(self):
        """Test configuring pool with custom max_workers."""
        Worker.configure_pool(max_workers=2)
        
        pool = Worker.get_pool()
        assert pool.max_workers == 2
    
    def test_configure_pool_after_use_fails(self):
        """Test that configure_pool() fails after first spawn."""
        # Spawn worker (initializes pool)
        future = Worker.spawn(lambda: 42)
        self.executor.run_until_complete(future)
        
        # Try to configure pool (should fail)
        with pytest.raises(RuntimeError, match="already initialized"):
            Worker.configure_pool(max_workers=8)
    
    def test_pool_default_max_workers(self):
        """Test that pool defaults to CPU count."""
        pool = Worker.get_pool()
        
        # Should be os.cpu_count() or 4
        expected = os.cpu_count() or 4
        assert pool.max_workers == expected
    
    def test_pool_active_count(self):
        """Test tracking active worker count."""
        def slow_task():
            time.sleep(0.2)
            return "done"
        
        pool = Worker.get_pool()
        
        # No active workers initially
        assert pool.get_active_count() == 0
        
        # Spawn workers
        future1 = Worker.spawn(slow_task)
        future2 = Worker.spawn(slow_task)
        
        # Give workers time to start
        time.sleep(0.05)
        
        # Should have 2 active workers
        assert pool.get_active_count() == 2
        
        # Wait for completion
        self.executor.run_until_complete(future1)
        self.executor.run_until_complete(future2)
        
        # No active workers after completion
        assert pool.get_active_count() == 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
