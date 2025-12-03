"""
Integration Tests: Workers + Channels

Tests VELA-581 (TASK-052)
Sprint 19 - Workers y Channels

Tests de integración completos que verifican Workers y Channels
trabajando juntos en escenarios realistas.
"""

import pytest
import sys
import os
import time
import threading

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../src')))

from runtime.channels import Channel
from runtime.workers import Worker, WorkerPool
from runtime.async_runtime import Executor


class TestProducerConsumerPatterns:
    """Tests para patrones Producer-Consumer."""
    
    def test_single_producer_single_consumer(self):
        """Test básico 1 producer → 1 consumer."""
        sender, receiver = Channel.new()
        
        def producer():
            for i in range(100):
                sender.send(i)
            sender.close()
        
        def consumer():
            total = 0
            for value in receiver:
                total += value
            return total
        
        # Spawn workers
        executor = Executor()
        
        p = Worker.spawn(lambda: producer())
        c = Worker.spawn(lambda: consumer())
        
        # Wait for producer
        executor.run_until_complete(p)
        
        # Get consumer result
        result = executor.run_until_complete(c)
        
        # Sum(0..99) = 4950
        assert result == 4950
    
    def test_multiple_producers_single_consumer(self):
        """Test MPSC: 5 producers → 1 consumer."""
        sender, receiver = Channel.new()
        
        def producer(id, count):
            for i in range(count):
                sender.send((id, i))
        
        def consumer():
            counts = {}
            for (id, _) in receiver:
                counts[id] = counts.get(id, 0) + 1
            return counts
        
        # Spawn 5 producers
        executor = Executor()
        
        producers = []
        for id in range(5):
            s = sender.clone()
            def make_producer(pid, psend):
                try:
                    for i in range(20):
                        psend.send((pid, i))
                finally:
                    psend.close()  # Explicit close
            p = Worker.spawn(lambda i=id, s=s: make_producer(i, s))
            producers.append(p)
        
        # Drop original sender
        sender.close()
        
        # Spawn consumer
        c = Worker.spawn(lambda: consumer())
        
        # Wait for all producers
        for p in producers:
            executor.run_until_complete(p)
        
        # Get result
        result = executor.run_until_complete(c)
        
        # Each producer should have sent 20 messages
        assert result == {0: 20, 1: 20, 2: 20, 3: 20, 4: 20}


class TestPipelinePatterns:
    """Tests para patrones de pipeline."""
    
    def test_three_stage_pipeline(self):
        """Test pipeline de 3 etapas: Generate → Transform → Aggregate."""
        s1, r1 = Channel.new()
        s2, r2 = Channel.new()
        
        def stage1_generate():
            """Generate numbers 1-50."""
            for i in range(1, 51):
                s1.send(i)
            s1.close()
        
        def stage2_transform():
            """Square each number."""
            for value in r1:
                s2.send(value ** 2)
            s2.close()
        
        def stage3_aggregate():
            """Sum all squared numbers."""
            total = 0
            for value in r2:
                total += value
            return total
        
        # Spawn pipeline stages
        executor = Executor()
        
        w1 = Worker.spawn(lambda: stage1_generate())
        w2 = Worker.spawn(lambda: stage2_transform())
        w3 = Worker.spawn(lambda: stage3_aggregate())
        
        # Wait for all stages
        executor.run_until_complete(w1)
        executor.run_until_complete(w2)
        result = executor.run_until_complete(w3)
        
        # Sum of squares: 1² + 2² + ... + 50² = 42925
        assert result == 42925
    
    def test_fan_out_fan_in(self):
        """Test Fan-out/Fan-in: 1 → 3 → 1."""
        input_s, input_r = Channel.new()
        output_s, output_r = Channel.new()
        
        def distributor():
            """Distribute work to 10 items."""
            for i in range(10):
                input_s.send(i)
            input_s.close()
        
        def worker(id, out_sender):
            """Process items (multiply by 10)."""
            try:
                while True:
                    value = input_r.try_receive()
                    if value is None:
                        if input_r.is_closed():
                            break
                        time.sleep(0.001)
                        continue
                    result = value * 10
                    out_sender.send((id, result))
            finally:
                out_sender.close()  # Explicit close
        
        def collector():
            """Collect all results."""
            results = []
            for (id, value) in output_r:
                results.append(value)
            return sorted(results)
        
        # Spawn distributor
        executor = Executor()
        
        d = Worker.spawn(lambda: distributor())
        
        # Spawn 3 workers
        workers = []
        for id in range(3):
            s = output_s.clone()
            w = Worker.spawn(lambda i=id, s=s: worker(i, s))
            workers.append(w)
        
        # Drop original output sender
        output_s.close()
        
        # Spawn collector
        c = Worker.spawn(lambda: collector())
        
        # Wait for distributor
        executor.run_until_complete(d)
        
        # Wait for workers
        for w in workers:
            executor.run_until_complete(w)
        
        # Get results
        result = executor.run_until_complete(c)
        
        # Should have all values * 10
        assert result == [0, 10, 20, 30, 40, 50, 60, 70, 80, 90]


class TestBackpressureAndBuffering:
    """Tests para backpressure y buffering."""
    
    def test_bounded_channel_backpressure(self):
        """Test que bounded channel aplica backpressure correctamente."""
        sender, receiver = Channel.new(capacity=5)
        
        send_times = []
        
        def fast_producer():
            """Producer rápido que envía 20 items."""
            for i in range(20):
                start = time.time()
                sender.send(i)
                elapsed = time.time() - start
                send_times.append(elapsed)
            sender.close()
        
        def slow_consumer():
            """Consumer lento que procesa con delay."""
            results = []
            for value in receiver:
                time.sleep(0.01)  # 10ms por item
                results.append(value)
            return results
        
        # Spawn workers
        executor = Executor()
        
        p = Worker.spawn(lambda: fast_producer())
        c = Worker.spawn(lambda: slow_consumer())
        
        # Wait for both
        executor.run_until_complete(p)
        result = executor.run_until_complete(c)
        
        # Should receive all 20 items
        assert result == list(range(20))
        
        # Some sends should have blocked (taken more time)
        blocked_sends = [t for t in send_times if t > 0.005]
        assert len(blocked_sends) > 0, "Expected some sends to block"
    
    def test_unbounded_no_blocking(self):
        """Test que unbounded channel nunca bloquea."""
        sender, receiver = Channel.new()  # Unbounded
        
        def producer():
            """Producer envía 1000 items rápidamente."""
            for i in range(1000):
                sender.send(i)
            sender.close()
        
        def consumer():
            """Consumer lee después."""
            time.sleep(0.1)  # Wait for producer to fill buffer
            count = 0
            for _ in receiver:
                count += 1
            return count
        
        # Spawn workers
        executor = Executor()
        
        start = time.time()
        p = Worker.spawn(lambda: producer())
        c = Worker.spawn(lambda: consumer())
        
        # Producer should complete quickly
        executor.run_until_complete(p)
        producer_time = time.time() - start
        
        # Should be fast (< 100ms)
        assert producer_time < 0.1
        
        # Consumer gets all items
        count = executor.run_until_complete(c)
        assert count == 1000


class TestErrorHandling:
    """Tests para manejo de errores."""
    
    def test_worker_error_propagation(self):
        """Test que errores en workers se propagan correctamente."""
        sender, receiver = Channel.new()
        
        def failing_producer():
            sender.send(1)
            sender.send(2)
            raise ValueError("Producer error")
        
        def consumer():
            results = []
            for value in receiver:
                results.append(value)
            return results
        
        # Spawn workers
        executor = Executor()
        
        p = Worker.spawn(lambda: failing_producer())
        c = Worker.spawn(lambda: consumer())
        
        # Producer should error
        with pytest.raises(ValueError, match="Producer error"):
            executor.run_until_complete(p)
        
        # Close sender so consumer finishes
        sender.close()
        
        # Consumer should receive partial data
        result = executor.run_until_complete(c)
        assert result == [1, 2]
    
    def test_channel_closed_during_send(self):
        """Test comportamiento cuando channel se cierra durante send."""
        from runtime.channels import ChannelClosedError
        
        sender, receiver = Channel.new(capacity=2)
        
        def producer():
            try:
                for i in range(10):
                    sender.send(i)
            except ChannelClosedError:
                return "closed"
            return "completed"
        
        def closer():
            """Close channel after short delay."""
            time.sleep(0.05)
            receiver.close()
        
        # Spawn workers
        executor = Executor()
        
        p = Worker.spawn(lambda: producer())
        t = threading.Thread(target=closer)
        t.start()
        
        # Producer should encounter closed channel
        result = executor.run_until_complete(p)
        t.join()
        
        assert result == "closed"


class TestConcurrencyStress:
    """Stress tests para concurrencia."""
    
    def test_many_concurrent_workers(self):
        """Test con muchos workers concurrentes (50 workers)."""
        sender, receiver = Channel.new()
        
        def worker(id, s):
            """Cada worker envía 10 mensajes."""
            try:
                for i in range(10):
                    s.send((id, i))
            finally:
                s.close()  # Explicit close
        
        def collector():
            """Collect todos los mensajes."""
            counts = {}
            for (id, _) in receiver:
                counts[id] = counts.get(id, 0) + 1
            return counts
        
        # Spawn 50 workers
        executor = Executor()
        
        workers = []
        for id in range(50):
            s = sender.clone()
            w = Worker.spawn(lambda i=id, s=s: worker(i, s))
            workers.append(w)
        
        # Drop original sender
        sender.close()
        
        # Spawn collector
        c = Worker.spawn(lambda: collector())
        
        # Wait for all workers
        for w in workers:
            executor.run_until_complete(w)
        
        # Get results
        result = executor.run_until_complete(c)
        
        # Each worker should have sent 10 messages
        assert len(result) == 50
        assert all(count == 10 for count in result.values())
    
    def test_high_throughput(self):
        """Test throughput con grandes volúmenes de datos."""
        sender, receiver = Channel.new()
        
        MESSAGE_COUNT = 10000
        
        def producer():
            for i in range(MESSAGE_COUNT):
                sender.send(i)
            sender.close()
        
        def consumer():
            count = 0
            for _ in receiver:
                count += 1
            return count
        
        # Spawn workers
        executor = Executor()
        
        start = time.time()
        
        p = Worker.spawn(lambda: producer())
        c = Worker.spawn(lambda: consumer())
        
        # Wait for completion
        executor.run_until_complete(p)
        result = executor.run_until_complete(c)
        
        elapsed = time.time() - start
        
        # Should receive all messages
        assert result == MESSAGE_COUNT
        
        # Calculate throughput
        throughput = MESSAGE_COUNT / elapsed
        
        # Should achieve > 10K msgs/sec
        assert throughput > 10000, f"Throughput too low: {throughput:.0f} msgs/sec"


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
