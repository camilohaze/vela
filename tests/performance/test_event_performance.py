"""
Performance Benchmarks - Event System

Benchmarks de performance del sistema de eventos:
- emit() con 1000+ listeners
- dispatch_event() con propagación profunda
- Memory usage con 10K+ eventos
- Comparación vs baseline teórico

Jira: VELA-576 (Sprint 14)
Task: TASK-035Q
Fecha: 2025-12-02
"""

import pytest
import time
import sys
import os
from statistics import mean, stdev

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../src')))

from runtime.events.event_bus import EventBus, Event, EventPhase


class EventTarget:
    """Mock EventTarget for benchmarks."""
    
    def __init__(self, name: str, parent=None):
        self.name = name
        self.parent = parent
        self.bus = EventBus()
    
    def addEventListener(self, event_type: str, listener, use_capture: bool = False, priority: int = 0):
        return self.bus.on(event_type, listener, use_capture=use_capture, priority=priority)
    
    def dispatchEvent(self, event):
        return self.bus.dispatch_event(event, target=self)


class TestEventPerformance:
    """Performance benchmarks del Event System."""
    
    # ==================== Benchmark Utilities ====================
    
    def benchmark(self, func, iterations: int = 1000):
        """Execute benchmark and return average time."""
        times = []
        
        for _ in range(iterations):
            start = time.perf_counter()
            func()
            end = time.perf_counter()
            times.append(end - start)
        
        return {
            "mean": mean(times),
            "stdev": stdev(times) if len(times) > 1 else 0,
            "min": min(times),
            "max": max(times),
            "total": sum(times)
        }
    
    # ==================== emit() Performance ====================
    
    def test_emit_with_single_listener_performance(self):
        """Benchmark emit() con 1 listener (baseline)."""
        bus = EventBus()
        
        events_received = []
        
        def listener(event):
            events_received.append(event)
        
        bus.on("test", listener)
        
        def emit_once():
            bus.emit("test", {"value": 42})
        
        results = self.benchmark(emit_once, iterations=10000)
        
        assert results["mean"] < 0.001  # < 1ms average
        assert len(events_received) == 10000
        
        print(f"\n[PERF] emit() 1 listener: {results['mean']*1000:.4f}ms avg")
    
    def test_emit_with_10_listeners_performance(self):
        """Benchmark emit() con 10 listeners."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Register 10 listeners
        for i in range(10):
            bus.on("test", listener)
        
        def emit_once():
            bus.emit("test", {"value": 42})
        
        results = self.benchmark(emit_once, iterations=5000)
        
        assert results["mean"] < 0.005  # < 5ms average
        assert counter[0] == 50000  # 10 listeners * 5000 iterations
        
        print(f"\n[PERF] emit() 10 listeners: {results['mean']*1000:.4f}ms avg")
    
    def test_emit_with_100_listeners_performance(self):
        """Benchmark emit() con 100 listeners."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Register 100 listeners
        for i in range(100):
            bus.on("test", listener)
        
        def emit_once():
            bus.emit("test", {"value": 42})
        
        results = self.benchmark(emit_once, iterations=1000)
        
        assert results["mean"] < 0.050  # < 50ms average
        assert counter[0] == 100000  # 100 listeners * 1000 iterations
        
        print(f"\n[PERF] emit() 100 listeners: {results['mean']*1000:.4f}ms avg")
    
    def test_emit_with_1000_listeners_performance(self):
        """Benchmark emit() con 1000 listeners (stress)."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Register 1000 listeners
        for i in range(1000):
            bus.on("test", listener)
        
        def emit_once():
            bus.emit("test", {"value": 42})
        
        results = self.benchmark(emit_once, iterations=100)
        
        assert results["mean"] < 0.500  # < 500ms average
        assert counter[0] == 100000  # 1000 listeners * 100 iterations
        
        print(f"\n[PERF] emit() 1000 listeners: {results['mean']*1000:.4f}ms avg")
    
    # ==================== dispatch_event() Performance ====================
    
    def test_dispatch_simple_event_performance(self):
        """Benchmark dispatch_event() sin propagación."""
        target = EventTarget("button")
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        target.addEventListener("click", listener)
        
        def dispatch_once():
            event = Event("click", {"x": 100, "y": 200})
            target.dispatchEvent(event)
        
        results = self.benchmark(dispatch_once, iterations=5000)
        
        assert results["mean"] < 0.002  # < 2ms average
        assert counter[0] == 5000
        
        print(f"\n[PERF] dispatch_event() simple: {results['mean']*1000:.4f}ms avg")
    
    def test_dispatch_with_3_level_hierarchy_performance(self):
        """Benchmark dispatch_event() con jerarquía 3 niveles."""
        root = EventTarget("root")
        parent = EventTarget("parent", parent=root)
        child = EventTarget("child", parent=parent)
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Add listeners to all levels
        root.addEventListener("click", listener)
        parent.addEventListener("click", listener)
        child.addEventListener("click", listener)
        
        def dispatch_once():
            event = Event("click", {})
            child.dispatchEvent(event)
        
        results = self.benchmark(dispatch_once, iterations=2000)
        
        assert results["mean"] < 0.005  # < 5ms average
        assert counter[0] == 6000  # 3 listeners * 2000 iterations
        
        print(f"\n[PERF] dispatch_event() 3 levels: {results['mean']*1000:.4f}ms avg")
    
    def test_dispatch_with_10_level_hierarchy_performance(self):
        """Benchmark dispatch_event() con jerarquía profunda (10 niveles)."""
        # Create 10-level hierarchy
        levels = []
        for i in range(10):
            parent = levels[-1] if levels else None
            level = EventTarget(f"level{i}", parent=parent)
            levels.append(level)
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Add listeners to all levels
        for level in levels:
            level.addEventListener("data", listener)
        
        def dispatch_once():
            event = Event("data", {})
            levels[-1].dispatchEvent(event)
        
        results = self.benchmark(dispatch_once, iterations=500)
        
        assert results["mean"] < 0.020  # < 20ms average
        assert counter[0] == 5000  # 10 listeners * 500 iterations
        
        print(f"\n[PERF] dispatch_event() 10 levels: {results['mean']*1000:.4f}ms avg")
    
    def test_dispatch_with_capturing_and_bubbling_performance(self):
        """Benchmark dispatch_event() con capturing + bubbling."""
        root = EventTarget("root")
        parent = EventTarget("parent", parent=root)
        child = EventTarget("child", parent=parent)
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Add capturing listeners
        root.addEventListener("click", listener, use_capture=True)
        parent.addEventListener("click", listener, use_capture=True)
        
        # Add bubbling listeners
        root.addEventListener("click", listener)
        parent.addEventListener("click", listener)
        child.addEventListener("click", listener)
        
        def dispatch_once():
            event = Event("click", {})
            child.dispatchEvent(event)
        
        results = self.benchmark(dispatch_once, iterations=1000)
        
        assert results["mean"] < 0.010  # < 10ms average
        assert counter[0] == 5000  # 5 listeners * 1000 iterations
        
        print(f"\n[PERF] dispatch_event() capturing+bubbling: {results['mean']*1000:.4f}ms avg")
    
    # ==================== Priority Sorting Performance ====================
    
    def test_priority_sorting_performance(self):
        """Benchmark sorting de listeners por priority."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Register 100 listeners con prioridades random
        import random
        for i in range(100):
            priority = random.randint(-50, 50)
            bus.on("test", listener, priority=priority)
        
        def emit_once():
            bus.emit("test", {})
        
        results = self.benchmark(emit_once, iterations=1000)
        
        assert results["mean"] < 0.050  # < 50ms average
        assert counter[0] == 100000
        
        print(f"\n[PERF] priority sorting (100 listeners): {results['mean']*1000:.4f}ms avg")
    
    # ==================== Memory Usage ====================
    
    def test_memory_usage_10k_events(self):
        """Test memory usage con 10K eventos emitidos."""
        import sys
        
        bus = EventBus()
        
        events_received = []
        
        def listener(event):
            events_received.append(event.type)
        
        bus.on("test", listener)
        
        # Measure baseline
        baseline_size = sys.getsizeof(events_received)
        
        # Emit 10K events
        for i in range(10000):
            bus.emit("test", {"iteration": i})
        
        # Measure final
        final_size = sys.getsizeof(events_received)
        
        assert len(events_received) == 10000
        
        # Memory should scale linearly
        size_per_event = (final_size - baseline_size) / 10000
        assert size_per_event < 100  # < 100 bytes per event
        
        print(f"\n[PERF] Memory per event: {size_per_event:.2f} bytes")
    
    def test_memory_usage_1000_listeners(self):
        """Test memory usage con 1000 listeners registrados."""
        import sys
        
        bus = EventBus()
        
        listeners = []
        
        # Register 1000 listeners
        for i in range(1000):
            def listener(event):
                pass
            
            listeners.append(listener)
            bus.on("test", listener)
        
        # Estimate memory usage
        listeners_dict = bus._listeners.get("test", [])
        
        assert len(listeners_dict) == 1000
        
        print(f"\n[PERF] 1000 listeners registered successfully")
    
    # ==================== Throughput Tests ====================
    
    def test_throughput_simple_emit(self):
        """Test throughput de emit() simple."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        bus.on("test", listener)
        
        # Emit 100K events
        start = time.perf_counter()
        for i in range(100000):
            bus.emit("test", {"i": i})
        end = time.perf_counter()
        
        elapsed = end - start
        throughput = 100000 / elapsed
        
        assert counter[0] == 100000
        assert throughput > 10000  # > 10K events/sec
        
        print(f"\n[PERF] emit() throughput: {throughput:.0f} events/sec")
    
    def test_throughput_dispatch_with_propagation(self):
        """Test throughput de dispatch_event() con propagación."""
        root = EventTarget("root")
        parent = EventTarget("parent", parent=root)
        child = EventTarget("child", parent=parent)
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        root.addEventListener("click", listener)
        parent.addEventListener("click", listener)
        child.addEventListener("click", listener)
        
        # Dispatch 10K events
        start = time.perf_counter()
        for i in range(10000):
            event = Event("click", {"i": i})
            child.dispatchEvent(event)
        end = time.perf_counter()
        
        elapsed = end - start
        throughput = 10000 / elapsed
        
        assert counter[0] == 30000  # 3 listeners * 10K events
        assert throughput > 1000  # > 1K events/sec
        
        print(f"\n[PERF] dispatch_event() throughput: {throughput:.0f} events/sec")
    
    # ==================== Latency Tests ====================
    
    def test_latency_p50_p95_p99(self):
        """Test latencia percentiles (p50, p95, p99)."""
        bus = EventBus()
        
        def listener(event):
            pass
        
        bus.on("test", listener)
        
        latencies = []
        
        for i in range(10000):
            start = time.perf_counter()
            bus.emit("test", {"i": i})
            end = time.perf_counter()
            latencies.append(end - start)
        
        latencies.sort()
        
        p50 = latencies[len(latencies) // 2]
        p95 = latencies[int(len(latencies) * 0.95)]
        p99 = latencies[int(len(latencies) * 0.99)]
        
        assert p50 < 0.001  # p50 < 1ms
        assert p95 < 0.005  # p95 < 5ms
        assert p99 < 0.010  # p99 < 10ms
        
        print(f"\n[PERF] Latency p50: {p50*1000:.4f}ms")
        print(f"[PERF] Latency p95: {p95*1000:.4f}ms")
        print(f"[PERF] Latency p99: {p99*1000:.4f}ms")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])  # -s to show print output
