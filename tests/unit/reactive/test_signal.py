"""
Tests unitarios para Signal<T>

Implementación de: US-06 - TASK-026
Historia: Sistema Reactivo
Fecha: 2025-12-01

Tests:
- Creación de signals
- Get/Set de valores
- Auto-tracking de dependencias
- Subscribers
- Comparación personalizada
- Update funcional
- Peek sin tracking
- Dispose y cleanup
"""

import pytest
from typing import List

import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../src')))

from reactive.signal import Signal, signal
from reactive import reset_global_graph, get_global_graph
from reactive.types import NodeType, DisposedNodeError


class TestSignalBasics:
    """Tests básicos de Signal."""
    
    def setup_method(self):
        """Setup para cada test."""
        reset_global_graph()
        self.graph = get_global_graph()
    
    def test_signal_creation(self):
        """Test de creación de signal."""
        s = Signal(42)
        
        assert s.get() == 42
        assert s.value == 42
        assert not s.is_disposed
    
    def test_signal_creation_with_different_types(self):
        """Test de signals con diferentes tipos."""
        int_signal = Signal(42)
        str_signal = Signal("hello")
        list_signal = Signal([1, 2, 3])
        dict_signal = Signal({"a": 1})
        
        assert int_signal.get() == 42
        assert str_signal.get() == "hello"
        assert list_signal.get() == [1, 2, 3]
        assert dict_signal.get() == {"a": 1}
    
    def test_signal_helper_function(self):
        """Test de función helper signal()."""
        s = signal(100)
        
        assert s.get() == 100
        assert isinstance(s, Signal)
    
    def test_signal_with_custom_id(self):
        """Test de signal con ID personalizado."""
        s = Signal(10, signal_id="my_signal")
        
        assert s._node.id == "my_signal"
    
    def test_signal_set(self):
        """Test de setear valor."""
        s = Signal(0)
        
        s.set(10)
        assert s.get() == 10
        
        s.set(20)
        assert s.get() == 20
    
    def test_signal_property_syntax(self):
        """Test de property syntax (get/set)."""
        s = Signal(0)
        
        # Get usando property
        assert s.value == 0
        
        # Set usando property
        s.value = 15
        assert s.value == 15
    
    def test_signal_update_functional(self):
        """Test de update funcional."""
        s = Signal(10)
        
        s.update(lambda x: x + 5)
        assert s.get() == 15
        
        s.update(lambda x: x * 2)
        assert s.get() == 30
    
    def test_signal_peek_no_tracking(self):
        """Test de peek sin tracking."""
        s = Signal(42)
        
        # Peek no debe registrar dependencia
        value = s.peek()
        assert value == 42
        
        # Verificar que no hay tracking activo
        assert not self.graph.is_tracking


class TestSignalTracking:
    """Tests de auto-tracking."""
    
    def setup_method(self):
        """Setup para cada test."""
        reset_global_graph()
        self.graph = get_global_graph()
    
    def test_signal_get_registers_dependency(self):
        """Test que get() registra dependencia."""
        from reactive.graph import ReactiveNode
        from reactive.types import NodeType
        
        s = Signal(10)
        computed = ReactiveNode(NodeType.COMPUTED)
        self.graph.register_node(computed)
        
        def compute():
            return s.get() * 2
        
        result = self.graph.track(computed, compute)
        
        assert result == 20
        assert s._node in computed.dependencies
    
    def test_signal_peek_no_dependency(self):
        """Test que peek() NO registra dependencia."""
        from reactive.graph import ReactiveNode
        from reactive.types import NodeType
        
        s = Signal(10)
        computed = ReactiveNode(NodeType.COMPUTED)
        self.graph.register_node(computed)
        
        def compute():
            return s.peek() * 2  # peek, no get
        
        result = self.graph.track(computed, compute)
        
        assert result == 20
        assert s._node not in computed.dependencies


class TestSignalPropagation:
    """Tests de propagación de cambios."""
    
    def setup_method(self):
        """Setup para cada test."""
        reset_global_graph()
        self.graph = get_global_graph()
    
    def test_signal_change_propagates(self):
        """Test que cambios se propagan."""
        from reactive.graph import ReactiveNode
        from reactive.types import NodeType
        
        s = Signal(10)
        
        computation_count = [0]
        
        def compute():
            computation_count[0] += 1
            return s.get() * 2
        
        computed = ReactiveNode(NodeType.COMPUTED, compute_fn=compute)
        self.graph.register_node(computed)
        
        # Computación inicial
        self.graph.track(computed, compute)
        assert computation_count[0] == 1
        assert computed.value == 20
        
        # Cambiar signal
        s.set(15)
        assert computation_count[0] == 2
        assert computed.value == 30
    
    def test_signal_no_propagation_if_equal(self):
        """Test que no propaga si el valor es igual."""
        from reactive.graph import ReactiveNode
        from reactive.types import NodeType
        
        s = Signal(10)
        
        computation_count = [0]
        
        def compute():
            computation_count[0] += 1
            return s.get() * 2
        
        computed = ReactiveNode(NodeType.COMPUTED, compute_fn=compute)
        self.graph.register_node(computed)
        
        # Computación inicial
        self.graph.track(computed, compute)
        assert computation_count[0] == 1
        
        # Set con mismo valor (NO debe propagar)
        s.set(10)
        assert computation_count[0] == 1  # Sin cambio
    
    def test_signal_custom_equals(self):
        """Test de comparación personalizada."""
        # Comparación que ignora case
        def case_insensitive_equals(a: str, b: str) -> bool:
            return a.lower() == b.lower()
        
        s = Signal("Hello", equals=case_insensitive_equals)
        
        computation_count = [0]
        
        def compute():
            computation_count[0] += 1
            return s.get().upper()
        
        from reactive.graph import ReactiveNode
        from reactive.types import NodeType
        computed = ReactiveNode(NodeType.COMPUTED, compute_fn=compute)
        self.graph.register_node(computed)
        
        # Setup
        self.graph.track(computed, compute)
        assert computation_count[0] == 1
        
        # Set con mismo valor (case-insensitive)
        s.set("HELLO")  # Misma palabra, diferente case
        assert computation_count[0] == 1  # NO debe propagar


class TestSignalSubscribers:
    """Tests de subscribers."""
    
    def setup_method(self):
        """Setup para cada test."""
        reset_global_graph()
    
    def test_signal_subscribe(self):
        """Test de subscribe a signal."""
        s = Signal(10)
        
        changes = []
        
        def on_change(new, old):
            changes.append((new, old))
        
        unsubscribe = s.subscribe(on_change)
        
        s.set(20)
        s.set(30)
        
        assert changes == [(20, 10), (30, 20)]
        
        # Unsubscribe
        unsubscribe()
        s.set(40)
        
        # No debe agregar más cambios
        assert changes == [(20, 10), (30, 20)]
    
    def test_signal_multiple_subscribers(self):
        """Test de múltiples subscribers."""
        s = Signal(0)
        
        calls_1 = [0]
        calls_2 = [0]
        
        def subscriber_1(new, old):
            calls_1[0] += 1
        
        def subscriber_2(new, old):
            calls_2[0] += 1
        
        s.subscribe(subscriber_1)
        s.subscribe(subscriber_2)
        
        s.set(10)
        
        assert calls_1[0] == 1
        assert calls_2[0] == 1
    
    def test_signal_subscriber_error_handling(self):
        """Test que errores en subscribers no rompen el sistema."""
        s = Signal(10)
        
        def bad_subscriber(new, old):
            raise ValueError("Bad subscriber")
        
        def good_subscriber(new, old):
            pass  # OK
        
        s.subscribe(bad_subscriber)
        s.subscribe(good_subscriber)
        
        # No debe lanzar excepción
        s.set(20)


class TestSignalComparisons:
    """Tests de comparaciones."""
    
    def setup_method(self):
        """Setup para cada test."""
        reset_global_graph()
    
    def test_signal_equality_with_value(self):
        """Test de igualdad con valor."""
        s = Signal(42)
        
        assert s == 42
        assert not (s == 10)
    
    def test_signal_equality_with_signal(self):
        """Test de igualdad entre signals."""
        s1 = Signal(42)
        s2 = Signal(42)
        s3 = Signal(10)
        
        assert s1 == s2
        assert not (s1 == s3)
    
    def test_signal_hash(self):
        """Test de hash (para usar en sets/dicts)."""
        s1 = Signal(10)
        s2 = Signal(10)
        
        # Diferentes signals tienen diferentes hashes (basado en ID)
        assert hash(s1) != hash(s2)
        
        # Puede usar en set
        signal_set = {s1, s2}
        assert len(signal_set) == 2


class TestSignalDispose:
    """Tests de dispose."""
    
    def setup_method(self):
        """Setup para cada test."""
        reset_global_graph()
        self.graph = get_global_graph()
    
    def test_signal_dispose(self):
        """Test de dispose."""
        s = Signal(10)
        
        assert not s.is_disposed
        assert self.graph.node_count == 1
        
        s.dispose()
        
        assert s.is_disposed
        assert self.graph.node_count == 0
    
    def test_signal_operations_after_dispose_fail(self):
        """Test que operaciones después de dispose fallan."""
        s = Signal(10)
        s.dispose()
        
        with pytest.raises(DisposedNodeError):
            s.get()
        
        with pytest.raises(DisposedNodeError):
            s.set(20)
    
    def test_signal_dispose_clears_subscribers(self):
        """Test que dispose limpia subscribers."""
        s = Signal(10)
        
        called = [False]
        
        def on_change(new, old):
            called[0] = True
        
        s.subscribe(on_change)
        s.dispose()
        
        # No debe lanzar excepción
        # (subscribers fueron limpiados)


class TestSignalRepresentation:
    """Tests de representación string."""
    
    def test_signal_repr(self):
        """Test de __repr__."""
        s = Signal(42)
        assert repr(s) == "Signal(42)"
    
    def test_signal_str(self):
        """Test de __str__."""
        s = Signal(42)
        assert str(s) == "42"
        
        s2 = Signal("hello")
        assert str(s2) == "hello"


class TestSignalIntegration:
    """Tests de integración completos."""
    
    def setup_method(self):
        """Setup para cada test."""
        reset_global_graph()
        self.graph = get_global_graph()
    
    def test_signal_with_computed(self):
        """Test de signal con computed."""
        from reactive.graph import ReactiveNode
        from reactive.types import NodeType
        
        count = Signal(0)
        
        def compute_doubled():
            return count.get() * 2
        
        doubled = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_doubled)
        self.graph.register_node(doubled)
        
        # Setup
        self.graph.track(doubled, compute_doubled)
        assert doubled.value == 0
        
        # Cambiar signal
        count.set(5)
        assert doubled.value == 10
        
        count.set(10)
        assert doubled.value == 20
    
    def test_signal_chain(self):
        """Test de cadena de signals y computed."""
        from reactive.graph import ReactiveNode
        from reactive.types import NodeType
        
        a = Signal(1)
        b = Signal(2)
        
        def compute_sum():
            return a.get() + b.get()
        
        def compute_product():
            return sum_node.value * 10
        
        sum_node = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_sum)
        product_node = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_product)
        
        self.graph.register_node(sum_node)
        self.graph.register_node(product_node)
        
        # Setup
        self.graph.track(sum_node, compute_sum)
        self.graph.track(product_node, lambda: (
            self.graph.record_dependency(sum_node),
            compute_product()
        )[1])
        
        assert sum_node.value == 3
        assert product_node.value == 30
        
        # Cambiar signal a
        a.set(5)
        assert sum_node.value == 7
        assert product_node.value == 70
    
    def test_signal_with_list_updates(self):
        """Test de signal con listas (inmutabilidad)."""
        items = Signal([1, 2, 3])
        
        # Update inmutable
        items.update(lambda lst: lst + [4])
        assert items.get() == [1, 2, 3, 4]
        
        # Update con map
        items.update(lambda lst: [x * 2 for x in lst])
        assert items.get() == [2, 4, 6, 8]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
