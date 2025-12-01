"""
Tests unitarios para el Grafo Reactivo

Implementación de: US-06 - TASK-025
Historia: Sistema Reactivo
Fecha: 2025-12-01

Tests:
- Creación de nodos
- Agregar/remover dependencias
- Propagación de cambios
- Detección de ciclos
- Batching de actualizaciones
- Topological sort
- Garbage collection
"""

import pytest
from typing import List

import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../src')))

from reactive.graph import ReactiveGraph, ReactiveNode
from reactive.types import NodeType, NodeState, CyclicDependencyError, DisposedNodeError


class TestReactiveNode:
    """Tests para ReactiveNode."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
    
    def test_node_creation(self):
        """Test de creación de nodo."""
        node = ReactiveNode(NodeType.SIGNAL, initial_value=42)
        
        assert node.id is not None
        assert node.node_type == NodeType.SIGNAL
        assert node.state == NodeState.CLEAN
        assert node.value == 42
        assert len(node.dependencies) == 0
        assert len(node.dependents) == 0
    
    def test_node_with_custom_id(self):
        """Test de nodo con ID personalizado."""
        node = ReactiveNode(NodeType.SIGNAL, initial_value=10, node_id="my_signal")
        
        assert node.id == "my_signal"
    
    def test_add_dependency(self):
        """Test de agregar dependencia."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        computed = ReactiveNode(NodeType.COMPUTED)
        
        computed.add_dependency(signal)
        
        assert signal in computed.dependencies
        assert computed in signal.dependents
    
    def test_remove_dependency(self):
        """Test de remover dependencia."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        computed = ReactiveNode(NodeType.COMPUTED)
        
        computed.add_dependency(signal)
        computed.remove_dependency(signal)
        
        assert signal not in computed.dependencies
        assert computed not in signal.dependents
    
    def test_clear_dependencies(self):
        """Test de limpiar todas las dependencias."""
        signal1 = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        signal2 = ReactiveNode(NodeType.SIGNAL, initial_value=20)
        computed = ReactiveNode(NodeType.COMPUTED)
        
        computed.add_dependency(signal1)
        computed.add_dependency(signal2)
        computed.clear_dependencies()
        
        assert len(computed.dependencies) == 0
        assert computed not in signal1.dependents
        assert computed not in signal2.dependents
    
    def test_mark_dirty(self):
        """Test de marcar como dirty."""
        node = ReactiveNode(NodeType.COMPUTED, initial_value=10)
        assert node.state == NodeState.CLEAN
        
        node.mark_dirty()
        assert node.state == NodeState.DIRTY
    
    def test_recompute_signal(self):
        """Test de recompute en signal (no-op)."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        signal._value = 20
        signal.mark_dirty()
        
        result = signal.recompute()
        
        assert result == 20
        assert signal.state == NodeState.CLEAN
    
    def test_recompute_computed(self):
        """Test de recompute en computed."""
        counter = [0]
        
        def compute():
            counter[0] += 1
            return counter[0] * 10
        
        computed = ReactiveNode(NodeType.COMPUTED, compute_fn=compute)
        computed.mark_dirty()
        
        result = computed.recompute()
        
        assert result == 10
        assert computed.value == 10
        assert computed.state == NodeState.CLEAN
    
    def test_dispose_node(self):
        """Test de dispose de nodo."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        computed = ReactiveNode(NodeType.COMPUTED)
        computed.add_dependency(signal)
        
        computed.dispose()
        
        assert computed.state == NodeState.DISPOSED
        assert len(computed.dependencies) == 0
        assert computed not in signal.dependents
    
    def test_disposed_node_operations_fail(self):
        """Test que operaciones en nodo disposed fallan."""
        node = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        node.dispose()
        
        with pytest.raises(DisposedNodeError):
            another = ReactiveNode(NodeType.COMPUTED)
            node.add_dependency(another)
        
        with pytest.raises(DisposedNodeError):
            node.recompute()


class TestReactiveGraph:
    """Tests para ReactiveGraph."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
    
    def test_graph_creation(self):
        """Test de creación de grafo."""
        assert self.graph.node_count == 0
        assert not self.graph.is_tracking
        assert self.graph.current_computation is None
    
    def test_register_node(self):
        """Test de registrar nodo."""
        node = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        self.graph.register_node(node)
        
        assert self.graph.node_count == 1
        assert self.graph.get_node(node.id) == node
    
    def test_unregister_node(self):
        """Test de desregistrar nodo."""
        node = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        self.graph.register_node(node)
        self.graph.unregister_node(node)
        
        assert self.graph.node_count == 0
        assert self.graph.get_node(node.id) is None
    
    def test_tracking_simple(self):
        """Test de tracking simple."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        computed = ReactiveNode(NodeType.COMPUTED)
        
        self.graph.register_node(signal)
        self.graph.register_node(computed)
        
        def compute():
            self.graph.record_dependency(signal)
            return signal.value * 2
        
        result = self.graph.track(computed, compute)
        
        assert result == 20
        assert signal in computed.dependencies
        assert computed in signal.dependents
    
    def test_tracking_nested(self):
        """Test de tracking anidado."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        computed1 = ReactiveNode(NodeType.COMPUTED)
        computed2 = ReactiveNode(NodeType.COMPUTED)
        
        self.graph.register_node(signal)
        self.graph.register_node(computed1)
        self.graph.register_node(computed2)
        
        def compute1():
            self.graph.record_dependency(signal)
            return signal.value * 2
        
        def compute2():
            self.graph.record_dependency(computed1)
            return self.graph.track(computed1, compute1) + 5
        
        result = self.graph.track(computed2, compute2)
        
        assert result == 25
        assert signal in computed1.dependencies
        assert computed1 in computed2.dependencies
    
    def test_propagate_change_simple(self):
        """Test de propagación simple."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        
        computation_count = [0]
        
        def compute():
            self.graph.record_dependency(signal)
            computation_count[0] += 1
            return signal.value * 2
        
        computed = ReactiveNode(NodeType.COMPUTED, compute_fn=compute)
        
        self.graph.register_node(signal)
        self.graph.register_node(computed)
        
        # Setup inicial
        self.graph.track(computed, compute)
        assert computation_count[0] == 1
        
        # Cambiar signal
        signal._value = 20
        self.graph.propagate_change(signal)
        
        assert computation_count[0] == 2
        assert computed.value == 40
    
    def test_propagate_change_chain(self):
        """Test de propagación en cadena."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        
        def compute1():
            self.graph.record_dependency(signal)
            return signal.value * 2
        
        def compute2():
            self.graph.record_dependency(computed1)
            return computed1.value + 5
        
        computed1 = ReactiveNode(NodeType.COMPUTED, compute_fn=compute1)
        computed2 = ReactiveNode(NodeType.COMPUTED, compute_fn=compute2)
        
        self.graph.register_node(signal)
        self.graph.register_node(computed1)
        self.graph.register_node(computed2)
        
        # Setup
        self.graph.track(computed1, compute1)
        self.graph.track(computed2, compute2)
        
        assert computed1.value == 20
        assert computed2.value == 25
        
        # Cambiar signal
        signal._value = 15
        self.graph.propagate_change(signal)
        
        assert computed1.value == 30
        assert computed2.value == 35
    
    def test_topological_sort(self):
        """Test de ordenamiento topológico."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        c1 = ReactiveNode(NodeType.COMPUTED)
        c2 = ReactiveNode(NodeType.COMPUTED)
        c3 = ReactiveNode(NodeType.COMPUTED)
        
        # signal -> c1 -> c2 -> c3
        c1.add_dependency(signal)
        c2.add_dependency(c1)
        c3.add_dependency(c2)
        
        nodes = {signal, c1, c2, c3}
        sorted_nodes = self.graph._topological_sort(nodes)
        
        # Verificar orden
        assert sorted_nodes.index(signal) < sorted_nodes.index(c1)
        assert sorted_nodes.index(c1) < sorted_nodes.index(c2)
        assert sorted_nodes.index(c2) < sorted_nodes.index(c3)
    
    def test_detect_cycle_direct(self):
        """Test de detección de ciclo directo."""
        c1 = ReactiveNode(NodeType.COMPUTED)
        c2 = ReactiveNode(NodeType.COMPUTED)
        
        # c1 -> c2 -> c1 (ciclo)
        c1.add_dependency(c2)
        c2.add_dependency(c1)
        
        nodes = {c1, c2}
        
        with pytest.raises(CyclicDependencyError) as exc_info:
            self.graph._detect_cycles(nodes)
        
        assert "Ciclo de dependencias detectado" in str(exc_info.value)
    
    def test_detect_cycle_indirect(self):
        """Test de detección de ciclo indirecto."""
        c1 = ReactiveNode(NodeType.COMPUTED)
        c2 = ReactiveNode(NodeType.COMPUTED)
        c3 = ReactiveNode(NodeType.COMPUTED)
        
        # c1 -> c2 -> c3 -> c1 (ciclo)
        c1.add_dependency(c2)
        c2.add_dependency(c3)
        c3.add_dependency(c1)
        
        nodes = {c1, c2, c3}
        
        with pytest.raises(CyclicDependencyError):
            self.graph._detect_cycles(nodes)
    
    def test_batch_updates(self):
        """Test de batching de actualizaciones."""
        signal1 = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        signal2 = ReactiveNode(NodeType.SIGNAL, initial_value=20)
        
        computation_count = [0]
        
        def compute():
            self.graph.record_dependency(signal1)
            self.graph.record_dependency(signal2)
            computation_count[0] += 1
            return signal1.value + signal2.value
        
        computed = ReactiveNode(NodeType.COMPUTED, compute_fn=compute)
        
        self.graph.register_node(signal1)
        self.graph.register_node(signal2)
        self.graph.register_node(computed)
        
        # Setup
        self.graph.track(computed, compute)
        assert computation_count[0] == 1
        
        # Batch updates
        def batch_fn():
            signal1._value = 15
            self.graph.propagate_change(signal1)
            
            signal2._value = 25
            self.graph.propagate_change(signal2)
        
        self.graph.batch(batch_fn)
        
        # Solo 1 recomputación adicional (batched)
        assert computation_count[0] == 2
        assert computed.value == 40
    
    def test_dispose_all(self):
        """Test de dispose de todos los nodos."""
        nodes = [
            ReactiveNode(NodeType.SIGNAL, initial_value=i)
            for i in range(5)
        ]
        
        for node in nodes:
            self.graph.register_node(node)
        
        assert self.graph.node_count == 5
        
        self.graph.dispose_all()
        
        assert self.graph.node_count == 0
        for node in nodes:
            assert node.state == NodeState.DISPOSED
    
    def test_debug_info(self):
        """Test de debug info."""
        self.graph.register_node(ReactiveNode(NodeType.SIGNAL, initial_value=10))
        self.graph.register_node(ReactiveNode(NodeType.COMPUTED))
        self.graph.register_node(ReactiveNode(NodeType.EFFECT))
        
        info = self.graph.debug_info()
        
        assert info['total_nodes'] == 3
        assert info['nodes_by_type']['SIGNAL'] == 1
        assert info['nodes_by_type']['COMPUTED'] == 1
        assert info['nodes_by_type']['EFFECT'] == 1
        assert not info['is_tracking']
        assert not info['is_batching']


class TestComplexScenarios:
    """Tests de escenarios complejos."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.graph = ReactiveGraph()
    
    def test_diamond_dependency(self):
        """Test de dependencia en diamante (diamond problem)."""
        signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
        
        def compute_a():
            self.graph.record_dependency(signal)
            return signal.value * 2
        
        def compute_b():
            self.graph.record_dependency(signal)
            return signal.value + 5
        
        def compute_c():
            self.graph.record_dependency(computed_a)
            self.graph.record_dependency(computed_b)
            return computed_a.value + computed_b.value
        
        computed_a = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_a)
        computed_b = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_b)
        computed_c = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_c)
        
        self.graph.register_node(signal)
        self.graph.register_node(computed_a)
        self.graph.register_node(computed_b)
        self.graph.register_node(computed_c)
        
        # Setup
        self.graph.track(computed_a, compute_a)
        self.graph.track(computed_b, compute_b)
        self.graph.track(computed_c, compute_c)
        
        assert computed_a.value == 20
        assert computed_b.value == 15
        assert computed_c.value == 35
        
        # Cambiar signal
        signal._value = 20
        self.graph.propagate_change(signal)
        
        assert computed_a.value == 40
        assert computed_b.value == 25
        assert computed_c.value == 65
    
    def test_multiple_signals(self):
        """Test con múltiples signals."""
        signals = [
            ReactiveNode(NodeType.SIGNAL, initial_value=i)
            for i in range(1, 6)
        ]
        
        def compute_sum():
            total = 0
            for s in signals:
                self.graph.record_dependency(s)
                total += s.value
            return total
        
        computed = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_sum)
        
        for s in signals:
            self.graph.register_node(s)
        self.graph.register_node(computed)
        
        # Setup
        self.graph.track(computed, compute_sum)
        assert computed.value == 15  # 1+2+3+4+5
        
        # Cambiar un signal
        signals[0]._value = 10
        self.graph.propagate_change(signals[0])
        
        assert computed.value == 24  # 10+2+3+4+5
    
    def test_cleanup_on_recompute(self):
        """Test de cleanup en recomputación."""
        cleanup_called = [False]
        
        def effect_fn():
            def cleanup():
                cleanup_called[0] = True
            return cleanup
        
        effect = ReactiveNode(NodeType.EFFECT, compute_fn=effect_fn)
        
        # Primera computación
        effect.recompute()
        assert not cleanup_called[0]
        
        # Segunda computación (debería llamar cleanup)
        effect.recompute()
        assert cleanup_called[0]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
