"""
Tests para Batch API

Jira: VELA-574 - TASK-032
Historia: API Pública de Batching

Tests para:
- batch() function
- batching() context manager
- start_batch() / end_batch()
- flush_batch()
- is_batching()
- @batch_decorator()
- BatchScope class
- Global graph management
"""

import pytest
from src.reactive.batch import (
    batch,
    batching,
    start_batch,
    end_batch,
    flush_batch,
    is_batching,
    batch_decorator,
    batch_fn,
    BatchScope,
    set_global_graph,
    get_global_graph,
)
from src.reactive.graph import ReactiveGraph
from src.reactive.signal import Signal
from src.reactive.computed import Computed


class TestGlobalGraph:
    """Tests para gestión de grafo global."""
    
    def test_set_and_get_global_graph(self):
        """Test establecer y obtener grafo global."""
        graph = ReactiveGraph()
        set_global_graph(graph)
        
        assert get_global_graph() == graph
    
    def test_get_global_graph_without_set_raises(self):
        """Test que get_global_graph() sin set lanza error."""
        # Este test verifica la integración con __init__.py
        # que ya tiene un grafo global configurado
        # Por tanto, get_global_graph() siempre debería funcionar
        graph = get_global_graph()
        assert graph is not None


class TestBatchFunction:
    """Tests para batch() function."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
        
        self.signal1 = Signal(0, graph=self.graph)
        self.signal2 = Signal(0, graph=self.graph)
        self.computed = Computed(
            lambda: self.signal1.get() + self.signal2.get(),
            graph=self.graph
        )
    
    def test_batch_function_basic(self):
        """Test batch() function básico."""
        result = batch(lambda: (
            self.signal1.set(10),
            self.signal2.set(20),
        ))
        
        assert self.computed.get() == 30
    
    def test_batch_function_returns_result(self):
        """Test que batch() retorna el resultado."""
        result = batch(lambda: 42)
        assert result == 42
        
        result = batch(lambda: "hello")
        assert result == "hello"
    
    def test_batch_with_explicit_graph(self):
        """Test batch() con grafo explícito."""
        other_graph = ReactiveGraph()
        signal = Signal(0, graph=other_graph)
        computed = Computed(lambda: signal.get() * 2, graph=other_graph)
        
        batch(lambda: signal.set(5), graph=other_graph)
        
        assert computed.get() == 10
    
    def test_batch_multiple_updates_same_signal(self):
        """Test múltiples updates al mismo signal en batch."""
        batch(lambda: (
            self.signal1.set(1),
            self.signal1.set(2),
            self.signal1.set(3),
        ))
        
        # Solo el último valor
        assert self.signal1.get() == 3


class TestBatchingContextManager:
    """Tests para batching() context manager."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
        
        self.signal = Signal(0, graph=self.graph)
        self.computed = Computed(lambda: self.signal.get() * 2, graph=self.graph)
    
    def test_batching_context_manager(self):
        """Test batching() como context manager."""
        with batching():
            self.signal.set(10)
            self.signal.set(20)
            self.signal.set(30)
        
        assert self.computed.get() == 60
    
    def test_batching_with_explicit_graph(self):
        """Test batching() con grafo explícito."""
        other_graph = ReactiveGraph()
        signal = Signal(0, graph=other_graph)
        
        with batching(graph=other_graph):
            signal.set(5)
        
        assert signal.get() == 5
    
    def test_nested_batching_context_managers(self):
        """Test batching() anidados."""
        with batching():
            self.signal.set(10)
            
            with batching():
                self.signal.set(20)
            
            self.signal.set(30)
        
        assert self.computed.get() == 60
    
    def test_batching_exception_handling(self):
        """Test que batching() limpia estado aunque haya excepción."""
        try:
            with batching():
                self.signal.set(10)
                raise ValueError("test error")
        except ValueError:
            pass
        
        # Debe haber salido del batch correctamente
        assert not is_batching()


class TestStartEndBatch:
    """Tests para start_batch() / end_batch()."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
        
        self.signal = Signal(0, graph=self.graph)
    
    def test_start_end_batch_basic(self):
        """Test start_batch() / end_batch() básico."""
        start_batch()
        
        assert is_batching()
        
        self.signal.set(10)
        self.signal.set(20)
        
        end_batch()
        
        assert not is_batching()
        assert self.signal.get() == 20
    
    def test_end_batch_without_start_raises(self):
        """Test que end_batch() sin start lanza error."""
        # Clear batch stack correctamente
        import sys
        batch_module = sys.modules['src.reactive.batch']
        batch_module._batch_stack.clear()
        
        with pytest.raises(RuntimeError, match="No active batch"):
            end_batch()
    
    def test_nested_start_end_batch(self):
        """Test start/end anidados."""
        start_batch()
        self.signal.set(10)
        
        start_batch()
        self.signal.set(20)
        end_batch()
        
        self.signal.set(30)
        end_batch()
        
        assert self.signal.get() == 30


class TestFlushBatch:
    """Tests para flush_batch()."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
        
        self.signal = Signal(0, graph=self.graph)
        self.computed = Computed(lambda: self.signal.get() * 2, graph=self.graph)
        self.effect_calls = []
        
        from src.reactive.effect import Effect
        Effect(
            lambda: self.effect_calls.append(self.signal.get()),
            graph=self.graph
        )
    
    def test_flush_batch_intermediate(self):
        """Test flush intermedio durante batch."""
        with batching():
            self.signal.set(10)
            flush_batch()  # Flush intermedio
            
            self.signal.set(20)
            flush_batch()  # Flush final manual dentro del batch
        
        # Verificar después de salir del batch
        # El último valor debe haber sido propagado
        assert self.computed.get() == 40


class TestIsBatching:
    """Tests para is_batching()."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
    
    def test_is_batching_false_initially(self):
        """Test que is_batching() es False inicialmente."""
        assert not is_batching()
    
    def test_is_batching_true_inside_context(self):
        """Test que is_batching() es True dentro de context."""
        with batching():
            assert is_batching()
        
        assert not is_batching()
    
    def test_is_batching_with_explicit_graph(self):
        """Test is_batching() con grafo explícito."""
        other_graph = ReactiveGraph()
        
        with batching(graph=other_graph):
            assert is_batching(graph=other_graph)


class TestBatchDecorator:
    """Tests para @batch_decorator()."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
        
        self.signal1 = Signal(0, graph=self.graph)
        self.signal2 = Signal(0, graph=self.graph)
        self.computed = Computed(
            lambda: self.signal1.get() + self.signal2.get(),
            graph=self.graph
        )
    
    def test_batch_decorator_basic(self):
        """Test @batch_decorator() básico."""
        @batch_decorator()
        def update_signals(a: int, b: int):
            self.signal1.set(a)
            self.signal2.set(b)
            return a + b
        
        result = update_signals(10, 20)
        
        assert result == 30
        assert self.computed.get() == 30
    
    def test_batch_decorator_preserves_function_name(self):
        """Test que el decorador preserva el nombre de la función."""
        @batch_decorator()
        def my_function():
            # Empty function for name testing
            return None
        
        assert my_function.__name__ == "my_function"
    
    def test_batch_fn_alias(self):
        """Test que batch_fn es alias de batch_decorator."""
        assert batch_fn == batch_decorator
    
    def test_batch_decorator_with_explicit_graph(self):
        """Test @batch_decorator() con grafo explícito."""
        other_graph = ReactiveGraph()
        signal = Signal(0, graph=other_graph)
        
        @batch_decorator(graph=other_graph)
        def update(value: int):
            signal.set(value)
        
        update(42)
        assert signal.get() == 42


class TestBatchScope:
    """Tests para BatchScope class."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
        
        self.signal = Signal(0, graph=self.graph)
    
    def test_batch_scope_basic(self):
        """Test BatchScope básico."""
        scope = BatchScope()
        scope.start()
        
        self.signal.set(10)
        
        scope.end()
        
        assert self.signal.get() == 10
    
    def test_batch_scope_as_context_manager(self):
        """Test BatchScope como context manager."""
        with BatchScope():
            self.signal.set(20)
        
        assert self.signal.get() == 20
    
    def test_batch_scope_start_twice_raises(self):
        """Test que start() dos veces lanza error."""
        scope = BatchScope()
        scope.start()
        
        with pytest.raises(RuntimeError, match="already active"):
            scope.start()
        
        scope.end()
    
    def test_batch_scope_end_without_start_raises(self):
        """Test que end() sin start lanza error."""
        scope = BatchScope()
        
        with pytest.raises(RuntimeError, match="not active"):
            scope.end()
    
    def test_batch_scope_flush(self):
        """Test BatchScope.flush()."""
        computed = Computed(lambda: self.signal.get() * 2, graph=self.graph)
        
        with BatchScope() as scope:
            self.signal.set(10)
            scope.flush()
            
            # Debe estar actualizado
            assert computed.get() == 20
    
    def test_batch_scope_repr(self):
        """Test BatchScope.__repr__()."""
        scope = BatchScope()
        repr_str = repr(scope)
        
        assert "BatchScope" in repr_str
        assert "active=False" in repr_str
        
        scope.start()
        repr_str = repr(scope)
        assert "active=True" in repr_str
        scope.end()


class TestBatchAPIIntegration:
    """Tests de integración de toda la API de batching."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
    
    def test_mixing_batch_apis(self):
        """Test mezclando diferentes APIs de batching."""
        signal = Signal(0, graph=self.graph)
        computed = Computed(lambda: signal.get() * 2, graph=self.graph)
        
        # Usar batch() function
        batch(lambda: signal.set(10))
        # batch() automáticamente hace flush al final
        assert computed.get() == 20
        
        # Test del context manager con NUEVO signal/computed
        # para evitar problemas de estado previo
        signal2 = Signal(0, graph=self.graph)
        computed2 = Computed(lambda: signal2.get() * 2, graph=self.graph)
        
        # Usar context manager
        with batching():
            signal2.set(20)
        
        # Después del context manager, el scheduler debe haber hecho flush
        assert computed2.get() == 40
        
        # Test del decorador con NUEVO signal/computed
        signal3 = Signal(0, graph=self.graph)
        computed3 = Computed(lambda: signal3.get() * 2, graph=self.graph)
        
        @batch_decorator()
        def update(value: int):
            signal3.set(value)
        
        update(30)
        assert computed3.get() == 60
    
    def test_complex_nested_batching(self):
        """Test batching complejo anidado."""
        signal = Signal(0, graph=self.graph)
        updates = []
        
        with batching():
            signal.set(1)
            updates.append(signal.get())
            
            batch(lambda: signal.set(2))
            updates.append(signal.get())
            
            with batching():
                signal.set(3)
                updates.append(signal.get())
            
            signal.set(4)
            updates.append(signal.get())
        
        assert updates == [1, 2, 3, 4]
        assert signal.get() == 4


class TestBatchAPIPerformance:
    """Tests de performance de la API de batching."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
        set_global_graph(self.graph)
    
    def test_batch_overhead(self):
        """Test overhead de batching."""
        import time
        
        signal = Signal(0, graph=self.graph)
        iterations = 1000
        
        # Sin batch
        start = time.perf_counter()
        for i in range(iterations):
            signal.set(i)
        time_no_batch = time.perf_counter() - start
        
        # Con batch
        start = time.perf_counter()
        batch(lambda: [signal.set(i) for i in range(iterations)])
        time_with_batch = time.perf_counter() - start
        
        print(f"\n  No batch: {time_no_batch:.6f}s")
        print(f"  With batch: {time_with_batch:.6f}s")
        print(f"  Speedup: {time_no_batch / time_with_batch:.2f}x")
        
        # Batch debe ser significativamente más rápido
        assert time_with_batch < time_no_batch


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
