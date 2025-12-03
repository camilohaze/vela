"""
Tests for Channel Integration with Workers

Tests VELA-580 (TASK-051)
Sprint 19 - Workers y Channels
"""

import pytest
import sys
import os

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.channels import Channel
from runtime.workers import Worker
from runtime.async_runtime import Executor


class TestWorkerChannelIntegration:
    """Tests for Workers communicating via Channels."""
    
    def test_worker_sends_to_channel(self):
        """Test worker sending to channel."""
        sender, receiver = Channel.new()
        
        def worker_task():
            sender.send("from worker")
        
        # Spawn worker
        executor = Executor()
        future = Worker.spawn(lambda: worker_task())
        
        # Wait for completion
        result = executor.run_until_complete(future)
        assert result is None  # Task returns None
        
        # Receive from channel
        value = receiver.receive()
        assert value == "from worker"
    
    def test_worker_receives_from_channel(self):
        """Test worker receiving from channel."""
        sender, receiver = Channel.new()
        
        def worker_task():
            value = receiver.receive()
            return value * 2
        
        # Send value
        sender.send(21)
        
        # Spawn worker
        executor = Executor()
        future = Worker.spawn(lambda: worker_task())
        
        # Wait for result
        result = executor.run_until_complete(future)
        assert result == 42
    
    def test_worker_pipeline(self):
        """Test pipeline of workers connected by channels."""
        # Stage 1 -> Stage 2 -> Stage 3
        s1_send, s1_recv = Channel.new()
        s2_send, s2_recv = Channel.new()
        
        def stage1():
            """Generate numbers 1-5."""
            for i in range(1, 6):
                s1_send.send(i)
            s1_send.close()
        
        def stage2():
            """Multiply by 2."""
            for value in s1_recv:
                s2_send.send(value * 2)
            s2_send.close()
        
        def stage3():
            """Sum all."""
            total = 0
            for value in s2_recv:
                total += value
            return total
        
        # Spawn workers
        executor = Executor()
        
        w1 = Worker.spawn(lambda: stage1())
        w2 = Worker.spawn(lambda: stage2())
        w3 = Worker.spawn(lambda: stage3())
        
        # Wait for completion
        executor.run_until_complete(w1)
        executor.run_until_complete(w2)
        result = executor.run_until_complete(w3)
        
        # Result: (1+2+3+4+5) * 2 = 30
        assert result == 30
    
    @pytest.mark.skip(reason="WorkerPool API needs review - different signature")
    def test_worker_pool_with_channel(self):
        """Test worker pool processing channel messages."""
        sender, receiver = Channel.new()
        
        # Send work items
        for i in range(10):
            sender.send(i)
        
        sender.close()
        
        # Process with worker pool
        from runtime.workers import WorkerPool
        
        pool = WorkerPool(max_workers=3)
        
        results = []
        for value in receiver:
            future = pool.submit(lambda v=value: v ** 2)
            results.append(future)
        
        # Wait for all results
        executor = Executor()
        final_results = []
        for f in results:
            result = executor.run_until_complete(f)
            final_results.append(result)
        
        pool.shutdown()
        
        # Check results
        assert sorted(final_results) == [i**2 for i in range(10)]
    
    def test_worker_error_with_channel(self):
        """Test worker error handling with channel."""
        sender, receiver = Channel.new()
        
        def failing_worker():
            sender.send("before error")
            raise ValueError("Worker error")
        
        # Spawn worker
        executor = Executor()
        future = Worker.spawn(lambda: failing_worker())
        
        # Receive value before error
        value = receiver.receive()
        assert value == "before error"
        
        # Worker should have errored
        with pytest.raises(ValueError, match="Worker error"):
            executor.run_until_complete(future)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
