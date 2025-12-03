"""
Tests unitarios para Worker API - Error Handling

Implementación de: VELA-580 (TASK-050)
Sprint 19 - Workers y Channels

Tests:
- Worker que lanza excepción
- Worker con diferentes tipos de excepciones
- Error propagation via Future
- Worker con error en closure
"""

import pytest
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.workers import Worker, WorkerPool
from runtime.async_runtime import Executor


class TestWorkerErrors:
    """Tests for Worker error handling."""
    
    def setup_method(self):
        """Setup before each test."""
        WorkerPool._global_pool = None
        self.executor = Executor()
    
    def teardown_method(self):
        """Cleanup after each test."""
        pool = WorkerPool.get_global()
        pool.shutdown(wait=True)
    
    def test_worker_raises_exception(self):
        """Test worker that raises generic exception."""
        def failing_task():
            raise Exception("Something went wrong")
        
        future = Worker.spawn(failing_task)
        
        with pytest.raises(Exception, match="Something went wrong"):
            self.executor.run_until_complete(future)
    
    def test_worker_raises_value_error(self):
        """Test worker that raises ValueError."""
        def invalid_input():
            raise ValueError("Invalid input")
        
        future = Worker.spawn(invalid_input)
        
        with pytest.raises(ValueError, match="Invalid input"):
            self.executor.run_until_complete(future)
    
    def test_worker_raises_zero_division_error(self):
        """Test worker that raises ZeroDivisionError."""
        def divide_by_zero():
            return 10 / 0
        
        future = Worker.spawn(divide_by_zero)
        
        with pytest.raises(ZeroDivisionError):
            self.executor.run_until_complete(future)
    
    def test_worker_raises_key_error(self):
        """Test worker that raises KeyError."""
        def missing_key():
            d = {"a": 1}
            return d["b"]  # KeyError
        
        future = Worker.spawn(missing_key)
        
        with pytest.raises(KeyError):
            self.executor.run_until_complete(future)
    
    def test_worker_raises_index_error(self):
        """Test worker that raises IndexError."""
        def out_of_bounds():
            lst = [1, 2, 3]
            return lst[10]  # IndexError
        
        future = Worker.spawn(out_of_bounds)
        
        with pytest.raises(IndexError):
            self.executor.run_until_complete(future)
    
    def test_worker_raises_custom_exception(self):
        """Test worker that raises custom exception."""
        class CustomError(Exception):
            pass
        
        def custom_error():
            raise CustomError("Custom error message")
        
        future = Worker.spawn(custom_error)
        
        with pytest.raises(CustomError, match="Custom error message"):
            self.executor.run_until_complete(future)
    
    def test_multiple_workers_some_fail(self):
        """Test multiple workers where some fail and some succeed."""
        def task(n):
            if n == 2:
                raise ValueError(f"Error on {n}")
            return n * 2
        
        future1 = Worker.spawn(lambda: task(1))  # OK
        future2 = Worker.spawn(lambda: task(2))  # Error
        future3 = Worker.spawn(lambda: task(3))  # OK
        
        # Worker 1 should succeed
        result1 = self.executor.run_until_complete(future1)
        assert result1 == 2
        
        # Worker 2 should fail
        with pytest.raises(ValueError, match="Error on 2"):
            self.executor.run_until_complete(future2)
        
        # Worker 3 should succeed
        result3 = self.executor.run_until_complete(future3)
        assert result3 == 6
    
    def test_worker_error_in_closure_capture(self):
        """Test worker with error accessing closure variable."""
        # Note: This test verifies that closure capture works,
        # even if the captured value leads to error
        x = None
        
        def access_none():
            return x.some_method()  # AttributeError
        
        future = Worker.spawn(access_none)
        
        with pytest.raises(AttributeError):
            self.executor.run_until_complete(future)
    
    def test_worker_error_message_preserved(self):
        """Test that error message is preserved through Future."""
        def detailed_error():
            raise ValueError("Detailed error message with important context")
        
        future = Worker.spawn(detailed_error)
        
        with pytest.raises(ValueError) as exc_info:
            self.executor.run_until_complete(future)
        
        assert "Detailed error message with important context" in str(exc_info.value)
    
    def test_worker_spawn_all_first_error_fails_fast(self):
        """Test that spawn_all() fails on first error (fail-fast behavior)."""
        def task(n):
            if n == 1:
                raise ValueError("Error on worker 1")
            return n * 2
        
        funcs = [
            lambda: task(0),
            lambda: task(1),  # This will error
            lambda: task(2)
        ]
        
        future = Worker.spawn_all(funcs)
        
        # Should raise ValueError from worker 1
        with pytest.raises(ValueError, match="Error on worker 1"):
            self.executor.run_until_complete(future)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
