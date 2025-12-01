"""
Tests unitarios para Computed<T>

Jira: US-06 - TASK-028
Historia: Sistema Reactivo
"""

import pytest

from src.reactive.computed import Computed, computed
from src.reactive.signal import Signal
from src.reactive.types import DisposedNodeError


class TestComputedBasics:
    """Tests básicos de Computed."""
    
    def test_computed_creation(self):
        """Test creación de computed."""
        c = Computed(lambda: 42)
        assert c.get() == 42
    
    def test_computed_helper_function(self):
        """Test función helper computed()."""
        c = computed(lambda: 100)
        assert c.get() == 100
    
    def test_computed_with_signal_dependency(self):
        """Test computed que depende de signal."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        
        assert doubled.get() == 10
    
    def test_computed_custom_id(self):
        """Test computed con ID personalizado."""
        c = Computed(lambda: 1, computed_id="my-computed")
        assert c._node.id == "my-computed"
    
    def test_computed_property_syntax(self):
        """Test que .value funciona como get()."""
        count = Signal(3)
        tripled = Computed(lambda: count.get() * 3)
        
        assert tripled.value == tripled.get()
        assert tripled.value == 9


class TestComputedLazyEval:
    """Tests de evaluación lazy."""
    
    def test_computed_lazy_initialization(self):
        """Test que computed NO se evalúa hasta el primer get()."""
        executed = []
        
        def compute_fn():
            executed.append(1)
            return 42
        
        c = Computed(compute_fn)
        assert len(executed) == 0  # No ejecutado aún
        
        value = c.get()
        assert value == 42
        assert len(executed) == 1  # Ejecutado ahora
    
    def test_computed_caching(self):
        """Test que computed cachea el resultado."""
        count = Signal(5)
        executions = []
        
        def compute_fn():
            executions.append(1)
            return count.get() * 2
        
        doubled = Computed(compute_fn)
        
        # Primera evaluación
        assert doubled.get() == 10
        assert len(executions) == 1
        
        # Segunda evaluación (cached)
        assert doubled.get() == 10
        assert len(executions) == 1  # NO se ejecutó de nuevo
    
    def test_computed_recompute_on_dependency_change(self):
        """Test que computed se recalcula cuando dependencia cambia."""
        count = Signal(5)
        executions = []
        
        def compute_fn():
            executions.append(1)
            return count.get() * 2
        
        doubled = Computed(compute_fn)
        
        assert doubled.get() == 10
        assert len(executions) == 1
        
        # Cambiar dependencia
        count.set(10)
        
        # Recompute
        assert doubled.get() == 20
        assert len(executions) == 2


class TestComputedTracking:
    """Tests de auto-tracking de dependencias."""
    
    def test_computed_tracks_single_signal(self):
        """Test que computed registra dependencia de signal."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        
        doubled.get()  # Trigger tracking
        
        # Verificar que doubled depende de count
        assert count._node in doubled._node.dependencies
    
    def test_computed_tracks_multiple_signals(self):
        """Test que computed registra múltiples signals."""
        a = Signal(2)
        b = Signal(3)
        sum_computed = Computed(lambda: a.get() + b.get())
        
        sum_computed.get()  # Trigger tracking
        
        assert a._node in sum_computed._node.dependencies
        assert b._node in sum_computed._node.dependencies
    
    def test_computed_propagates_changes(self):
        """Test que cambios se propagan a computed."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        
        assert doubled.get() == 10
        
        count.set(10)
        assert doubled.get() == 20
        
        count.set(15)
        assert doubled.get() == 30


class TestComputedChaining:
    """Tests de computed anidados."""
    
    def test_nested_computed(self):
        """Test computed que depende de otro computed."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        quadrupled = Computed(lambda: doubled.get() * 2)
        
        assert quadrupled.get() == 20
    
    def test_nested_computed_propagation(self):
        """Test que cambios se propagan en cadena."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        quadrupled = Computed(lambda: doubled.get() * 2)
        
        assert quadrupled.get() == 20
        
        count.set(10)
        assert quadrupled.get() == 40
    
    def test_diamond_dependency(self):
        """Test diamond dependency (A -> B, A -> C, B+C -> D)."""
        a = Signal(5)
        b = Computed(lambda: a.get() * 2)   # 10
        c = Computed(lambda: a.get() + 10)  # 15
        d = Computed(lambda: b.get() + c.get())  # 25
        
        assert d.get() == 25
        
        a.set(10)
        assert d.get() == 50  # (20 + 30)


class TestComputedPeek:
    """Tests de peek() sin tracking."""
    
    def test_computed_peek_returns_value(self):
        """Test que peek() retorna el valor."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        
        assert doubled.peek() == 10
    
    def test_computed_peek_initializes_if_needed(self):
        """Test que peek() inicializa el computed si es necesario."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        
        # peek() sin get() previo
        assert doubled.peek() == 10
        assert doubled._initialized


class TestComputedDispose:
    """Tests de dispose()."""
    
    def test_computed_dispose(self):
        """Test que dispose() limpia el computed."""
        c = Computed(lambda: 42)
        c.get()  # Inicializar
        
        c.dispose()
        assert c.is_disposed
    
    def test_computed_operations_after_dispose_fail(self):
        """Test que operaciones después de dispose() fallan."""
        c = Computed(lambda: 42)
        c.get()
        c.dispose()
        
        with pytest.raises(DisposedNodeError):
            c.get()
    
    def test_computed_cleanup_dependencies(self):
        """Test que dispose() limpia dependencias."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        doubled.get()  # Crear dependencias
        
        # Verificar que hay dependencia
        assert count._node in doubled._node.dependencies
        
        doubled.dispose()
        
        # Dependencias deberían estar limpias
        assert len(doubled._node.dependencies) == 0


class TestComputedState:
    """Tests de estado interno."""
    
    def test_computed_is_dirty_property(self):
        """Test propiedad is_dirty."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        
        assert doubled.is_dirty  # No inicializado
        
        doubled.get()
        assert not doubled.is_dirty  # Clean después de compute
        
        count.set(10)
        assert doubled.is_dirty  # Dirty después de cambio
    
    def test_computed_is_disposed_property(self):
        """Test propiedad is_disposed."""
        c = Computed(lambda: 42)
        assert not c.is_disposed
        
        c.dispose()
        assert c.is_disposed


class TestComputedRepresentation:
    """Tests de __repr__."""
    
    def test_computed_repr_before_init(self):
        """Test __repr__ antes de inicializar."""
        c = Computed(lambda: 42)
        assert repr(c) == "Computed(<not initialized>)"
    
    def test_computed_repr_after_init(self):
        """Test __repr__ después de inicializar."""
        c = Computed(lambda: 42)
        c.get()
        assert repr(c) == "Computed(42)"


class TestComputedIntegration:
    """Tests de integración complejos."""
    
    def test_computed_with_conditional_dependencies(self):
        """Test computed con dependencias condicionales."""
        flag = Signal(True)
        a = Signal(10)
        b = Signal(20)
        
        result = Computed(lambda: a.get() if flag.get() else b.get())
        
        assert result.get() == 10
        
        flag.set(False)
        assert result.get() == 20
    
    def test_computed_with_list_reduce(self):
        """Test computed que reduce una lista de signals."""
        numbers = [Signal(i) for i in range(5)]
        sum_computed = Computed(lambda: sum(s.get() for s in numbers))
        
        assert sum_computed.get() == 10  # 0+1+2+3+4
        
        numbers[0].set(10)
        assert sum_computed.get() == 20  # 10+1+2+3+4
    
    def test_computed_with_nested_calls(self):
        """Test computed con múltiples niveles de anidación."""
        base = Signal(2)
        level1 = Computed(lambda: base.get() * 2)      # 4
        level2 = Computed(lambda: level1.get() * 2)    # 8
        level3 = Computed(lambda: level2.get() * 2)    # 16
        
        assert level3.get() == 16
        
        base.set(3)
        assert level3.get() == 24  # 3*2*2*2
    
    def test_computed_multiple_reads_same_signal(self):
        """Test computed que lee el mismo signal múltiples veces."""
        count = Signal(5)
        result = Computed(lambda: count.get() + count.get() + count.get())
        
        assert result.get() == 15
        
        count.set(10)
        assert result.get() == 30


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
