"""
Stress Tests - Event System

Tests de stress del sistema de eventos:
- 10,000+ eventos emitidos consecutivos
- 1,000+ listeners registrados simultáneos
- Propagation tree de 100 niveles
- Memory stability después de 1M+ eventos
- Concurrent-like behavior simulation

Jira: VELA-576 (Sprint 14)
Task: TASK-035Q
Fecha: 2025-12-02
"""

import pytest
import time
import sys
import os

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../src')))

from runtime.events.event_bus import EventBus, Event, EventPhase


class EventTarget:
    """Mock EventTarget for stress tests."""
    
    def __init__(self, name: str, parent=None):
        self.name = name
        self.parent = parent
        self.bus = EventBus()
    
    def addEventListener(self, event_type: str, listener, use_capture: bool = False, priority: int = 0):
        return self.bus.on(event_type, listener, use_capture=use_capture, priority=priority)
    
    def dispatchEvent(self, event):
        return self.bus.dispatch_event(event, target=self)


class TestEventStress:
    """Stress tests del Event System."""
    
    # ==================== High Volume Events ====================
    
    def test_10k_events_single_listener(self):
        """10,000 eventos con 1 listener."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        bus.on("test", listener)
        
        start = time.perf_counter()
        
        # Emit 10K events
        for i in range(10000):
            bus.emit("test", {"iteration": i})
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 10000
        assert elapsed < 5.0  # Should complete in < 5 seconds
        
        print(f"\n[STRESS] 10K events: {elapsed:.2f}s ({10000/elapsed:.0f} events/sec)")
    
    def test_100k_events_single_listener(self):
        """100,000 eventos con 1 listener."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        bus.on("test", listener)
        
        start = time.perf_counter()
        
        # Emit 100K events
        for i in range(100000):
            bus.emit("test", {"iteration": i})
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 100000
        assert elapsed < 30.0  # Should complete in < 30 seconds
        
        print(f"\n[STRESS] 100K events: {elapsed:.2f}s ({100000/elapsed:.0f} events/sec)")
    
    def test_1million_events_single_listener(self):
        """1,000,000 eventos con 1 listener (ultimate stress)."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        bus.on("test", listener)
        
        start = time.perf_counter()
        
        # Emit 1M events
        for i in range(1000000):
            bus.emit("test", {"iteration": i})
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 1000000
        
        print(f"\n[STRESS] 1M events: {elapsed:.2f}s ({1000000/elapsed:.0f} events/sec)")
    
    # ==================== High Volume Listeners ====================
    
    def test_1000_listeners_single_event(self):
        """1,000 listeners con 1 evento."""
        bus = EventBus()
        
        counter = [0]
        
        def make_listener(listener_id):
            def listener(event):
                counter[0] += 1
            return listener
        
        # Register 1000 listeners
        for i in range(1000):
            bus.on("test", make_listener(i))
        
        start = time.perf_counter()
        
        bus.emit("test", {})
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 1000
        assert elapsed < 1.0  # Should complete in < 1 second
        
        print(f"\n[STRESS] 1000 listeners: {elapsed:.3f}s")
    
    def test_10k_listeners_single_event(self):
        """10,000 listeners con 1 evento (extreme stress)."""
        bus = EventBus()
        
        counter = [0]
        
        def make_listener(listener_id):
            def listener(event):
                counter[0] += 1
            return listener
        
        # Register 10K listeners
        for i in range(10000):
            bus.on("test", make_listener(i))
        
        start = time.perf_counter()
        
        bus.emit("test", {})
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 10000
        
        print(f"\n[STRESS] 10K listeners: {elapsed:.3f}s")
    
    def test_1000_listeners_100_events(self):
        """1,000 listeners con 100 eventos."""
        bus = EventBus()
        
        counter = [0]
        
        def make_listener(listener_id):
            def listener(event):
                counter[0] += 1
            return listener
        
        # Register 1000 listeners
        for i in range(1000):
            bus.on("test", make_listener(i))
        
        start = time.perf_counter()
        
        # Emit 100 events
        for i in range(100):
            bus.emit("test", {"iteration": i})
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 100000  # 1000 listeners * 100 events
        assert elapsed < 10.0  # Should complete in < 10 seconds
        
        print(f"\n[STRESS] 1000 listeners * 100 events: {elapsed:.2f}s")
    
    # ==================== Deep Propagation Trees ====================
    
    def test_100_level_propagation_tree(self):
        """Propagación en árbol de 100 niveles."""
        # Create 100-level hierarchy
        levels = []
        for i in range(100):
            parent = levels[-1] if levels else None
            level = EventTarget(f"level{i}", parent=parent)
            levels.append(level)
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Add listeners to all levels
        for level in levels:
            level.addEventListener("data", listener)
        
        start = time.perf_counter()
        
        # Dispatch from deepest level
        event = Event("data", {})
        levels[-1].dispatchEvent(event)
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 100
        assert elapsed < 1.0  # Should complete in < 1 second
        
        print(f"\n[STRESS] 100 level propagation: {elapsed:.3f}s")
    
    def test_50_level_tree_with_100_events(self):
        """50 niveles con 100 eventos."""
        # Create 50-level hierarchy
        levels = []
        for i in range(50):
            parent = levels[-1] if levels else None
            level = EventTarget(f"level{i}", parent=parent)
            levels.append(level)
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Add listeners to all levels
        for level in levels:
            level.addEventListener("data", listener)
        
        start = time.perf_counter()
        
        # Dispatch 100 events from deepest level
        for i in range(100):
            event = Event("data", {"iteration": i})
            levels[-1].dispatchEvent(event)
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 5000  # 50 levels * 100 events
        assert elapsed < 5.0  # Should complete in < 5 seconds
        
        print(f"\n[STRESS] 50 levels * 100 events: {elapsed:.2f}s")
    
    # ==================== Memory Stability ====================
    
    def test_memory_stability_after_100k_events(self):
        """Estabilidad de memoria después de 100K eventos."""
        import sys
        
        bus = EventBus()
        
        events_received = []
        
        def listener(event):
            # Store event type only (lightweight)
            events_received.append(event.type)
        
        bus.on("test", listener)
        
        # Baseline memory
        baseline = sys.getsizeof(events_received)
        
        # Emit 100K events
        for i in range(100000):
            bus.emit("test", {})
        
        # Final memory
        final = sys.getsizeof(events_received)
        
        assert len(events_received) == 100000
        
        # Memory should grow linearly, not exponentially
        growth_per_event = (final - baseline) / 100000
        assert growth_per_event < 100  # < 100 bytes per event
        
        print(f"\n[STRESS] Memory growth: {growth_per_event:.2f} bytes/event")
    
    def test_no_memory_leak_after_subscribe_unsubscribe_cycles(self):
        """Sin memory leaks después de ciclos subscribe/unsubscribe."""
        bus = EventBus()
        
        def listener(event):
            pass
        
        # Baseline
        initial_listeners = len(bus._listeners.get("test", []))
        
        # 1000 subscribe/unsubscribe cycles
        for i in range(1000):
            sub = bus.on("test", listener)
            sub.unsubscribe()
        
        # Final
        final_listeners = len(bus._listeners.get("test", []))
        
        # Should be back to initial state
        assert final_listeners == initial_listeners
        
        print(f"\n[STRESS] 1000 subscribe/unsubscribe cycles: No leaks")
    
    # ==================== Concurrent-Like Patterns ====================
    
    def test_interleaved_subscribe_emit_unsubscribe(self):
        """Pattern entrelazado: subscribe -> emit -> unsubscribe."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        # Interleaved pattern 1000 times
        for i in range(1000):
            sub = bus.on("test", listener)
            bus.emit("test", {})
            sub.unsubscribe()
        
        assert counter[0] == 1000
        
        print(f"\n[STRESS] 1000 interleaved cycles completed")
    
    def test_rapid_event_type_switching(self):
        """Cambio rápido entre tipos de eventos."""
        bus = EventBus()
        
        counters = {"event_a": 0, "event_b": 0, "event_c": 0}
        
        def make_listener(event_type):
            def listener(event):
                counters[event_type] += 1
            return listener
        
        bus.on("event_a", make_listener("event_a"))
        bus.on("event_b", make_listener("event_b"))
        bus.on("event_c", make_listener("event_c"))
        
        # Rapid switching 10K times
        for i in range(10000):
            bus.emit("event_a", {})
            bus.emit("event_b", {})
            bus.emit("event_c", {})
        
        assert counters["event_a"] == 10000
        assert counters["event_b"] == 10000
        assert counters["event_c"] == 10000
        
        print(f"\n[STRESS] 30K events across 3 types")
    
    def test_mixed_priorities_under_load(self):
        """Prioridades mixtas bajo carga."""
        bus = EventBus()
        
        execution_order = []
        
        def make_listener(name, priority):
            def listener(event):
                execution_order.append(name)
            return listener
        
        # Register 100 listeners con prioridades random
        import random
        for i in range(100):
            priority = random.randint(-50, 50)
            bus.on("test", make_listener(f"listener_{i}", priority), priority=priority)
        
        start = time.perf_counter()
        
        # Emit 100 events
        for i in range(100):
            bus.emit("test", {})
            execution_order.clear()  # Reset for next iteration
        
        elapsed = time.perf_counter() - start
        
        assert elapsed < 10.0  # Should handle sorting efficiently
        
        print(f"\n[STRESS] 100 listeners * 100 events with priorities: {elapsed:.2f}s")
    
    # ==================== Extreme Payload Sizes ====================
    
    def test_large_payload_repeated(self):
        """Payload grande repetido 1000 veces."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
            assert len(event.payload["data"]) == 100000
        
        bus.on("test", listener)
        
        large_payload = {"data": "x" * 100000}  # 100KB
        
        start = time.perf_counter()
        
        for i in range(1000):
            bus.emit("test", large_payload)
        
        elapsed = time.perf_counter() - start
        
        assert counter[0] == 1000
        
        print(f"\n[STRESS] 1000 large payloads (100KB each): {elapsed:.2f}s")
    
    def test_nested_payload_structure(self):
        """Payload con estructura anidada profunda."""
        bus = EventBus()
        
        counter = [0]
        
        def listener(event):
            counter[0] += 1
        
        bus.on("test", listener)
        
        # Create deeply nested payload
        def create_nested(depth):
            if depth == 0:
                return {"value": "leaf"}
            return {"nested": create_nested(depth - 1)}
        
        nested_payload = create_nested(100)
        
        for i in range(1000):
            bus.emit("test", nested_payload)
        
        assert counter[0] == 1000
        
        print(f"\n[STRESS] 1000 deeply nested payloads (100 levels)")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
