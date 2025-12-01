"""
Tests para Reactive Scheduler

Jira: VELA-574 - TASK-031
Historia: Scheduler Reactivo Avanzado

Tests para:
- Batching de actualizaciones
- Priorización de updates
- Coalescing de cambios múltiples
- Context manager
- Performance benchmarks
"""

import pytest
import time
from src.reactive.scheduler import (
    ReactiveScheduler,
    SchedulerPriority,
    ScheduledUpdate,
)
from src.reactive.graph import ReactiveGraph, ReactiveNode
from src.reactive.types import NodeType, NodeState
from src.reactive.signal import Signal
from src.reactive.computed import Computed


class TestSchedulerPriority:
    """Tests para SchedulerPriority enum."""
    
    def test_priority_ordering(self):
        """Test que las prioridades están ordenadas correctamente."""
        assert SchedulerPriority.SYNC.value < SchedulerPriority.HIGH.value
        assert SchedulerPriority.HIGH.value < SchedulerPriority.NORMAL.value
        assert SchedulerPriority.NORMAL.value < SchedulerPriority.LOW.value
    
    def test_priority_names(self):
        """Test nombres de prioridades."""
        assert SchedulerPriority.SYNC.name == "SYNC"
        assert SchedulerPriority.HIGH.name == "HIGH"
        assert SchedulerPriority.NORMAL.name == "NORMAL"
        assert SchedulerPriority.LOW.name == "LOW"


class TestScheduledUpdate:
    """Tests para ScheduledUpdate."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        self.node = ReactiveNode(NodeType.SIGNAL, initial_value=0)
        self.graph.register_node(self.node)
    
    def test_initialization(self):
        """Test creación de ScheduledUpdate."""
        update = ScheduledUpdate(
            node=self.node,
            priority=SchedulerPriority.HIGH,
            timestamp=time.time(),
        )
        
        assert update.node == self.node
        assert update.priority == SchedulerPriority.HIGH
        assert isinstance(update.timestamp, float)
    
    def test_ordering_by_priority(self):
        """Test que updates se ordenan por prioridad."""
        update1 = ScheduledUpdate(self.node, SchedulerPriority.HIGH, 1.0)
        update2 = ScheduledUpdate(self.node, SchedulerPriority.LOW, 0.5)
        
        assert update1 < update2  # HIGH (1) < LOW (3)
    
    def test_ordering_by_timestamp(self):
        """Test que con misma prioridad, se usa timestamp."""
        update1 = ScheduledUpdate(self.node, SchedulerPriority.HIGH, 1.0)
        update2 = ScheduledUpdate(self.node, SchedulerPriority.HIGH, 2.0)
        
        assert update1 < update2  # 1.0 < 2.0
    
    def test_repr(self):
        """Test representación string."""
        update = ScheduledUpdate(self.node, SchedulerPriority.SYNC, 123.456789)
        repr_str = repr(update)
        
        assert "ScheduledUpdate" in repr_str
        assert "SYNC" in repr_str
        assert "123.456" in repr_str


class TestReactiveScheduler:
    """Tests para ReactiveScheduler."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.scheduler = ReactiveScheduler()
        self.graph = ReactiveGraph(scheduler=self.scheduler)
        
        # Crear signal
        self.signal = Signal(0, graph=self.graph)
    
    def test_initialization(self):
        """Test inicialización del scheduler."""
        assert not self.scheduler.is_flushing
        assert not self.scheduler.is_batching
        assert self.scheduler.metrics['total_updates'] == 0
    
    def test_schedule_update(self):
        """Test programar un update."""
        self.scheduler.schedule_update(
            self.signal._node,
            SchedulerPriority.HIGH
        )
        
        assert self.scheduler.metrics['total_updates'] == 1
    
    def test_coalescing_multiple_updates(self):
        """Test que updates al mismo nodo se coalescen."""
        # Entrar en modo batch manualmente
        self.scheduler._is_batching = True
        
        # Programar 3 updates al mismo nodo
        self.scheduler.schedule_update(self.signal._node)
        self.scheduler.schedule_update(self.signal._node)
        self.scheduler.schedule_update(self.signal._node)
        
        # 3 intentos de update, pero solo 1 real + 2 coalesced
        assert self.scheduler.metrics['total_updates'] == 3
        assert self.scheduler.metrics['coalesced_updates'] == 2
        
        # Salir del batch
        self.scheduler._is_batching = False
        self.scheduler.flush()
    
    def test_batch_mode(self):
        """Test modo batch."""
        updates_executed = []
        
        def record_update():
            self.signal.set(self.signal.get() + 1)
            updates_executed.append(self.signal.get())
        
        # Ejecutar en batch
        self.scheduler.batch(lambda: [
            record_update(),
            record_update(),
            record_update(),
        ])
        
        # 3 updates ejecutados
        assert len(updates_executed) == 3
        assert updates_executed == [1, 2, 3]
    
    def test_batch_returns_result(self):
        """Test que batch retorna el resultado de la función."""
        result = self.scheduler.batch(lambda: 42)
        assert result == 42
        
        result = self.scheduler.batch(lambda: "hello")
        assert result == "hello"
    
    def test_nested_batching(self):
        """Test batches anidados."""
        outer_result = []
        
        def inner_batch():
            self.signal.set(5)
            return "inner"
        
        def outer_batch():
            self.signal.set(10)
            result = self.scheduler.batch(inner_batch)
            outer_result.append(result)
            return "outer"
        
        result = self.scheduler.batch(outer_batch)
        
        assert result == "outer"
        assert outer_result == ["inner"]
        assert self.signal.get() == 5  # Inner batch ganó
    
    def test_priority_inference(self):
        """Test inferencia automática de prioridad."""
        # Signal → SYNC
        priority = self.scheduler._infer_priority(self.signal._node)
        assert priority == SchedulerPriority.SYNC
        
        # Computed → HIGH
        computed = Computed(lambda: self.signal.get() * 2, graph=self.graph)
        priority = self.scheduler._infer_priority(computed._node)
        assert priority == SchedulerPriority.HIGH
    
    def test_flush_empty_scheduler(self):
        """Test flush con scheduler vacío."""
        self.scheduler.flush()  # No debe fallar
        assert self.scheduler.metrics['flush_count'] == 1
    
    def test_flush_with_updates(self):
        """Test flush con updates pendientes."""
        self.signal.set(10)
        self.scheduler.flush()
        
        assert self.scheduler.metrics['flush_count'] >= 1
    
    def test_max_flush_depth(self):
        """Test prevención de loops infinitos."""
        # Este test verifica que existe el límite
        assert self.scheduler._max_flush_depth == 100
        
        # En práctica, los ciclos se detectan antes por el grafo
        # Este test solo verifica que el límite existe como safety net
    
    def test_clear(self):
        """Test limpieza del scheduler."""
        self.scheduler.schedule_update(self.signal._node)
        self.scheduler.clear()
        
        assert len(self.scheduler._scheduled_nodes) == 0
    
    def test_repr(self):
        """Test representación string."""
        repr_str = repr(self.scheduler)
        
        assert "ReactiveScheduler" in repr_str
        assert "pending=" in repr_str
        assert "flushing=" in repr_str


class TestSchedulerIntegration:
    """Tests de integración con ReactiveGraph."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.scheduler = ReactiveScheduler()
        self.graph = ReactiveGraph(scheduler=self.scheduler)
    
    def test_graph_uses_scheduler(self):
        """Test que el grafo usa el scheduler."""
        assert self.graph._scheduler == self.scheduler
    
    def test_batch_through_graph(self):
        """Test batching a través del grafo."""
        signal1 = Signal(0, graph=self.graph)
        signal2 = Signal(0, graph=self.graph)
        computed = Computed(
            lambda: signal1.get() + signal2.get(),
            graph=self.graph
        )
        
        # Batch a través del grafo
        self.graph.batch(lambda: (
            signal1.set(10),
            signal2.set(20),
        ))
        
        # Computed debe tener el valor actualizado
        assert computed.get() == 30
    
    def test_context_manager_batching(self):
        """Test batching con context manager."""
        signal = Signal(0, graph=self.graph)
        computed = Computed(lambda: signal.get() * 2, graph=self.graph)
        
        with self.graph.batching():
            signal.set(5)
            signal.set(10)
            signal.set(15)
        
        # Solo 1 update al final
        assert computed.get() == 30
    
    def test_multiple_signals_batch(self):
        """Test múltiples signals en batch."""
        signals = [Signal(i, graph=self.graph) for i in range(10)]
        computed = Computed(
            lambda: sum(s.get() for s in signals),
            graph=self.graph
        )
        
        # Actualizar todos en batch
        self.graph.batch(lambda: [s.set(s.get() + 1) for s in signals])
        
        # Computed debe reflejar cambios
        assert computed.get() == sum(range(1, 11))


class TestSchedulerPerformance:
    """Benchmarks de performance del scheduler."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.scheduler = ReactiveScheduler()
        self.graph = ReactiveGraph(scheduler=self.scheduler)
    
    def test_benchmark_batching_vs_individual(self):
        """Benchmark: batching vs updates individuales."""
        signals = [Signal(i, graph=self.graph) for i in range(100)]
        computed = Computed(
            lambda: sum(s.get() for s in signals),
            graph=self.graph
        )
        
        # Tiempo sin batch
        start = time.perf_counter()
        for s in signals:
            s.set(s.get() + 1)
        time_individual = time.perf_counter() - start
        
        # Resetear
        for i, s in enumerate(signals):
            s.set(i)
        
        # Tiempo con batch
        start = time.perf_counter()
        self.graph.batch(lambda: [s.set(s.get() + 1) for s in signals])
        time_batched = time.perf_counter() - start
        
        print(f"\n  Individual: {time_individual:.6f}s")
        print(f"  Batched:    {time_batched:.6f}s")
        print(f"  Speedup:    {time_individual / time_batched:.2f}x")
        
        # Batch debe ser más rápido (o al menos similar)
        assert time_batched <= time_individual * 1.5  # Tolerar 50% overhead
    
    def test_benchmark_coalescing(self):
        """Benchmark: coalescing de updates múltiples."""
        signal = Signal(0, graph=self.graph)
        _computed = Computed(lambda: signal.get() * 2, graph=self.graph)
        
        # 1000 updates al mismo signal
        start = time.perf_counter()
        self.graph.batch(lambda: [signal.set(i) for i in range(1000)])
        elapsed = time.perf_counter() - start
        
        print(f"\n  1000 coalesced updates: {elapsed:.6f}s")
        print(f"  Metrics: {self.scheduler.metrics}")
        
        # Debe completar en tiempo razonable
        assert elapsed < 1.0  # Menos de 1 segundo
        
        # Debe haber coalescing
        assert self.scheduler.metrics['coalesced_updates'] > 0
    
    def test_scheduler_overhead(self):
        """Test overhead del scheduler."""
        signal = Signal(0, graph=self.graph)
        
        # Medir overhead de scheduling
        iterations = 1000
        
        start = time.perf_counter()
        for _ in range(iterations):
            self.scheduler.schedule_update(signal._node)
        elapsed = time.perf_counter() - start
        
        per_update = (elapsed / iterations) * 1_000_000  # microsegundos
        
        print(f"\n  Scheduling overhead: {per_update:.2f} μs/update")
        
        # Debe ser muy rápido
        assert per_update < 100  # Menos de 100 μs por update


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
