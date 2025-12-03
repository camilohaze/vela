"""
Benchmarks for Workers + Channels Integration

Tests performance characteristics:
- Channel throughput (bounded/unbounded)
- Worker spawn latency
- MPSC scalability (1, 10, 50, 100 producers)
- Pipeline efficiency (2, 3, 4, 5 stages)
- Memory usage under load

Jira: TASK-052 (VELA-581)
Sprint 19
"""

import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')))

import time
import pytest
from src.runtime.workers.worker import Worker
from src.runtime.async_runtime.executor import Executor
from src.runtime.channels.channel import Channel


class TestChannelThroughput:
    """Benchmark channel throughput with different configurations."""
    
    def test_benchmark_unbounded_channel_throughput(self):
        """Benchmark: Unbounded channel throughput."""
        sender, receiver = Channel.unbounded()
        executor = Executor()
        
        message_count = 100_000
        
        def producer():
            start = time.perf_counter()
            for i in range(message_count):
                sender.send(i)
            duration = time.perf_counter() - start
            return duration
        
        def consumer():
            count = 0
            for _ in receiver:
                count += 1
                if count == message_count:
                    break
            return count
        
        p = Worker.spawn(producer)
        c = Worker.spawn(consumer)
        
        producer_time = executor.run_until_complete(p)
        sender.close()
        
        consumer_count = executor.run_until_complete(c)
        
        throughput = message_count / producer_time
        
        print(f"\n=== Unbounded Channel Throughput ===")
        print(f"Messages: {message_count:,}")
        print(f"Producer time: {producer_time:.3f}s")
        print(f"Throughput: {throughput:,.0f} msgs/sec")
        
        assert consumer_count == message_count
        assert throughput > 100_000  # Target: > 100K msgs/sec
    
    def test_benchmark_bounded_channel_throughput(self):
        """Benchmark: Bounded channel throughput."""
        sender, receiver = Channel.new(capacity=1000)
        executor = Executor()
        
        message_count = 100_000
        
        def producer():
            start = time.perf_counter()
            for i in range(message_count):
                sender.send(i)
            duration = time.perf_counter() - start
            return duration
        
        def consumer():
            count = 0
            start = time.perf_counter()
            for _ in receiver:
                count += 1
                if count == message_count:
                    break
            duration = time.perf_counter() - start
            return (count, duration)
        
        c = Worker.spawn(consumer)
        p = Worker.spawn(producer)
        
        producer_time = executor.run_until_complete(p)
        sender.close()
        
        consumer_count, consumer_time = executor.run_until_complete(c)
        
        throughput = message_count / max(producer_time, consumer_time)
        
        print(f"\n=== Bounded Channel Throughput (capacity=1000) ===")
        print(f"Messages: {message_count:,}")
        print(f"Producer time: {producer_time:.3f}s")
        print(f"Consumer time: {consumer_time:.3f}s")
        print(f"Throughput: {throughput:,.0f} msgs/sec")
        
        assert consumer_count == message_count
        assert throughput > 50_000  # Target: > 50K msgs/sec (bounded slower due to blocking)


class TestWorkerSpawnLatency:
    """Benchmark worker spawn latency."""
    
    def test_benchmark_worker_spawn_latency(self):
        """Benchmark: Worker spawn latency."""
        executor = Executor()
        
        iterations = 1000
        
        def noop():
            return 42
        
        start = time.perf_counter()
        workers = [Worker.spawn(noop) for _ in range(iterations)]
        spawn_time = time.perf_counter() - start
        
        # Wait for all workers
        for w in workers:
            executor.run_until_complete(w)
        
        total_time = time.perf_counter() - start
        
        avg_spawn_latency = (spawn_time / iterations) * 1000  # ms
        avg_total_latency = (total_time / iterations) * 1000  # ms
        
        print(f"\n=== Worker Spawn Latency ===")
        print(f"Workers: {iterations:,}")
        print(f"Total spawn time: {spawn_time:.3f}s")
        print(f"Avg spawn latency: {avg_spawn_latency:.3f}ms")
        print(f"Avg total latency: {avg_total_latency:.3f}ms")
        
        assert avg_spawn_latency < 1.0  # Target: < 1ms spawn latency


class TestMPSCScalability:
    """Benchmark MPSC scalability with varying producer counts."""
    
    @pytest.mark.parametrize("producer_count", [1, 10, 50, 100])
    def test_benchmark_mpsc_scalability(self, producer_count):
        """Benchmark: MPSC scalability."""
        sender, receiver = Channel.unbounded()
        executor = Executor()
        
        messages_per_producer = 1000
        total_messages = producer_count * messages_per_producer
        
        def producer(id, s):
            try:
                for i in range(messages_per_producer):
                    s.send((id, i))
            finally:
                s.close()
        
        def consumer():
            count = 0
            start = time.perf_counter()
            for _ in receiver:
                count += 1
                if count == total_messages:
                    break
            duration = time.perf_counter() - start
            return (count, duration)
        
        # Spawn consumer
        c = Worker.spawn(consumer)
        
        # Spawn producers
        start = time.perf_counter()
        producers = []
        for id in range(producer_count):
            s = sender.clone()
            p = Worker.spawn(lambda i=id, s=s: producer(i, s))
            producers.append(p)
        
        # Drop original sender
        sender.close()
        
        # Wait for all producers
        for p in producers:
            executor.run_until_complete(p)
        
        producer_time = time.perf_counter() - start
        
        # Wait for consumer
        count, consumer_time = executor.run_until_complete(c)
        
        throughput = total_messages / consumer_time
        
        print(f"\n=== MPSC Scalability ({producer_count} producers) ===")
        print(f"Total messages: {total_messages:,}")
        print(f"Producer time: {producer_time:.3f}s")
        print(f"Consumer time: {consumer_time:.3f}s")
        print(f"Throughput: {throughput:,.0f} msgs/sec")
        
        assert count == total_messages
        assert throughput > 10_000  # Target: > 10K msgs/sec


class TestPipelineEfficiency:
    """Benchmark pipeline efficiency with varying stage counts."""
    
    @pytest.mark.parametrize("stage_count", [2, 3, 4, 5])
    def test_benchmark_pipeline_efficiency(self, stage_count):
        """Benchmark: Pipeline efficiency."""
        executor = Executor()
        
        message_count = 10_000
        
        # Create channels for pipeline
        channels = [Channel.unbounded() for _ in range(stage_count)]
        
        def generator(output_s):
            try:
                for i in range(message_count):
                    output_s.send(i)
            finally:
                output_s.close()
        
        def stage(input_r, output_s):
            try:
                for value in input_r:
                    # Simple transformation (x * 2)
                    output_s.send(value * 2)
            finally:
                output_s.close()
        
        def aggregator(input_r):
            count = 0
            for _ in input_r:
                count += 1
            return count
        
        # Spawn generator
        start = time.perf_counter()
        g = Worker.spawn(lambda: generator(channels[0][0]))
        
        # Spawn intermediate stages
        workers = [g]
        for i in range(stage_count - 1):
            s_in, r_in = channels[i]
            s_out, r_out = channels[i + 1]
            w = Worker.spawn(lambda r=r_in, s=s_out: stage(r, s))
            workers.append(w)
        
        # Spawn aggregator
        _, r_final = channels[-1]
        agg = Worker.spawn(lambda: aggregator(r_final))
        workers.append(agg)
        
        # Wait for all workers
        for w in workers[:-1]:
            executor.run_until_complete(w)
        
        count = executor.run_until_complete(agg)
        
        total_time = time.perf_counter() - start
        throughput = message_count / total_time
        
        print(f"\n=== Pipeline Efficiency ({stage_count} stages) ===")
        print(f"Messages: {message_count:,}")
        print(f"Stages: {stage_count}")
        print(f"Total time: {total_time:.3f}s")
        print(f"Throughput: {throughput:,.0f} msgs/sec")
        
        assert count == message_count
        assert throughput > 5_000  # Target: > 5K msgs/sec


class TestMemoryUsage:
    """Benchmark memory usage under load."""
    
    def test_benchmark_memory_usage_bounded(self):
        """Benchmark: Memory usage with bounded channel."""
        import tracemalloc
        
        tracemalloc.start()
        
        sender, receiver = Channel.new(capacity=100)
        executor = Executor()
        
        message_count = 10_000
        
        def producer():
            for i in range(message_count):
                sender.send(i)
        
        def consumer():
            count = 0
            for _ in receiver:
                count += 1
                if count == message_count:
                    break
            return count
        
        snapshot_before = tracemalloc.take_snapshot()
        
        p = Worker.spawn(producer)
        c = Worker.spawn(consumer)
        
        executor.run_until_complete(p)
        sender.close()
        
        count = executor.run_until_complete(c)
        
        snapshot_after = tracemalloc.take_snapshot()
        
        stats = snapshot_after.compare_to(snapshot_before, 'lineno')
        total_allocated = sum(stat.size_diff for stat in stats if stat.size_diff > 0)
        
        tracemalloc.stop()
        
        memory_per_message = total_allocated / message_count
        
        print(f"\n=== Memory Usage (bounded, capacity=100) ===")
        print(f"Messages: {message_count:,}")
        print(f"Total allocated: {total_allocated / 1024:.2f} KB")
        print(f"Memory per message: {memory_per_message:.2f} bytes")
        
        assert count == message_count
        assert memory_per_message < 1000  # Target: < 1KB per message
    
    def test_benchmark_memory_usage_unbounded(self):
        """Benchmark: Memory usage with unbounded channel."""
        import tracemalloc
        
        tracemalloc.start()
        
        sender, receiver = Channel.unbounded()
        executor = Executor()
        
        message_count = 10_000
        
        def producer():
            for i in range(message_count):
                sender.send(i)
        
        def consumer():
            count = 0
            for _ in receiver:
                count += 1
                if count == message_count:
                    break
            return count
        
        snapshot_before = tracemalloc.take_snapshot()
        
        p = Worker.spawn(producer)
        c = Worker.spawn(consumer)
        
        executor.run_until_complete(p)
        sender.close()
        
        count = executor.run_until_complete(c)
        
        snapshot_after = tracemalloc.take_snapshot()
        
        stats = snapshot_after.compare_to(snapshot_before, 'lineno')
        total_allocated = sum(stat.size_diff for stat in stats if stat.size_diff > 0)
        
        tracemalloc.stop()
        
        memory_per_message = total_allocated / message_count
        
        print(f"\n=== Memory Usage (unbounded) ===")
        print(f"Messages: {message_count:,}")
        print(f"Total allocated: {total_allocated / 1024:.2f} KB")
        print(f"Memory per message: {memory_per_message:.2f} bytes")
        
        assert count == message_count
        assert memory_per_message < 1000  # Target: < 1KB per message


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
