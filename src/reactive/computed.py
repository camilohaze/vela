"""
Computed<T> - Valor Derivado Reactivo

Implementación de: US-06 - TASK-028
Historia: Sistema Reactivo
Fecha: 2025-12-01

Descripción:
Computed<T> es un valor derivado que se calcula automáticamente
basado en signals u otros computed values. Usa lazy evaluation
y caching para optimizar performance.
"""

from typing import Callable, Generic, Optional, TypeVar

from .graph import ReactiveGraph, ReactiveNode
from .types import NodeType, NodeState, DisposedNodeError

T = TypeVar('T')


def get_global_graph():
    """Import helper to avoid circular dependency."""
    from . import global_graph
    return global_graph


class Computed(Generic[T]):
    """
    Computed<T> - Valor derivado reactivo.
    
    Un Computed es un valor que se calcula automáticamente basado en
    signals u otros computed values. Se recalcula solo cuando sus
    dependencias cambian (lazy + cached).
    
    Features:
    - Lazy evaluation: Solo se calcula cuando se lee
    - Caching: Resultado se cachea hasta que dependencias cambien
    - Auto-tracking: Dependencias se registran automáticamente
    - Composable: Computed puede depender de otros computed
    
    Example:
        count = Signal(0)
        doubled = Computed(lambda: count.get() * 2)
        
        print(doubled.get())  # 0 (calcula)
        print(doubled.get())  # 0 (cached)
        
        count.set(5)
        print(doubled.get())  # 10 (recalcula)
    """
    
    def __init__(
        self,
        compute_fn: Callable[[], T],
        *,
        graph: Optional[ReactiveGraph] = None,
        computed_id: Optional[str] = None,
    ):
        """
        Crea un nuevo Computed.
        
        Args:
            compute_fn: Función de computación () => T
            graph: Grafo reactivo (usa el global si no se provee)
            computed_id: ID personalizado (opcional)
        """
        self._graph = graph or get_global_graph()
        self._compute_fn = compute_fn
        
        # Crear nodo reactivo
        self._node = ReactiveNode(
            NodeType.COMPUTED,
            compute_fn=self._compute,
            node_id=computed_id,
        )
        
        # Registrar en el grafo
        self._graph.register_node(self._node)
        
        # Computación inicial (lazy - se hace en primer get)
        self._initialized = False
    
    def _compute(self) -> T:
        """Función interna de computación con tracking."""
        return self._compute_fn()
    
    @property
    def value(self) -> T:
        """
        Obtiene el valor actual (readonly property).
        
        Alias de get() para compatibilidad con property syntax.
        """
        return self.get()
    
    def get(self) -> T:
        """
        Obtiene el valor actual del computed.
        
        Si es la primera vez o está dirty, ejecuta la función de
        computación con auto-tracking. Si está clean, retorna el
        valor cacheado.
        
        Returns:
            T: Valor computado
            
        Raises:
            DisposedNodeError: Si el computed fue destruido
        """
        if self._node.state == NodeState.DISPOSED:
            raise DisposedNodeError(f"Computed {self._node.id} is disposed")
        
        # Si no está inicializado o está dirty, recompute
        if not self._initialized or self._node.state == NodeState.DIRTY:
            # Track dependencies y compute
            result = self._graph.track(self._node, self._compute)
            self._node._value = result
            self._node._state = NodeState.CLEAN
            self._initialized = True
        
        # Registrar dependencia en computación activa (nested computed/effects)
        self._graph.record_dependency(self._node)
        
        return self._node.value
    
    def peek(self) -> T:
        """
        Obtiene el valor sin tracking de dependencias.
        
        Returns:
            T: Valor actual (o None si no inicializado)
        """
        if not self._initialized:
            # Force compute sin tracking
            result = self._graph.track(self._node, self._compute)
            self._node._value = result
            self._node._state = NodeState.CLEAN
            self._initialized = True
        
        return self._node.value
    
    def dispose(self) -> None:
        """Destruye el computed y limpia recursos."""
        self._node.dispose()
        self._graph.unregister_node(self._node)
        self._initialized = False
    
    @property
    def is_disposed(self) -> bool:
        """Si el computed fue destruido."""
        return self._node.state == NodeState.DISPOSED
    
    @property
    def is_dirty(self) -> bool:
        """Si necesita recalcularse."""
        return self._node.state == NodeState.DIRTY
    
    def __repr__(self) -> str:
        """Representación string del computed."""
        if not self._initialized:
            return "Computed(<not initialized>)"
        return f"Computed({self._node.value})"


def computed(compute_fn: Callable[[], T], **kwargs) -> Computed[T]:
    """
    Función helper para crear computed values.
    
    Args:
        compute_fn: Función de computación
        **kwargs: Argumentos adicionales para Computed()
        
    Returns:
        Computed[T]: Nuevo computed
        
    Example:
        count = signal(0)
        doubled = computed(lambda: count.get() * 2)
    """
    return Computed(compute_fn, **kwargs)


if __name__ == "__main__":
    from .signal import Signal
    
    # Ejemplo básico
    count = Signal(0)
    doubled = Computed(lambda: count.get() * 2)
    
    print(f"Initial: {doubled.get()}")  # 0
    
    count.set(5)
    print(f"After set: {doubled.get()}")  # 10
    
    # Cached
    print(f"Cached: {doubled.get()}")  # 10 (no recompute)
    
    # Nested computed
    quadrupled = Computed(lambda: doubled.get() * 2)
    print(f"Quadrupled: {quadrupled.get()}")  # 20
    
    count.set(10)
    print(f"After change: {quadrupled.get()}")  # 40
    
    print("\nComputed examples completed")
