"""
Sistema de Auto-Tracking de Dependencias

Implementación de: US-06 - TASK-025
Historia: Sistema Reactivo
Fecha: 2025-12-01

Descripción:
Implementa el contexto de tracking automático de dependencias.
Permite que computed values, effects y watches registren sus
dependencias automáticamente al leer signals.
"""

from typing import Any, Callable, Optional, TypeVar
from contextlib import contextmanager

from .graph import ReactiveGraph, ReactiveNode


T = TypeVar('T')


class TrackingContext:
    """
    Contexto de tracking de dependencias.
    
    Gestiona el stack de computaciones activas para auto-tracking.
    """
    
    def __init__(self, graph: ReactiveGraph):
        """
        Inicializa el contexto de tracking.
        
        Args:
            graph: Grafo reactivo asociado
        """
        self._graph = graph
        self._paused = False
    
    @property
    def is_tracking(self) -> bool:
        """Si el tracking está activo."""
        return not self._paused and self._graph.is_tracking
    
    @property
    def current_computation(self) -> Optional[ReactiveNode]:
        """Computación activa actual."""
        return self._graph.current_computation if not self._paused else None
    
    def track(self, node: ReactiveNode, compute_fn: Callable[[], T]) -> T:
        """
        Ejecuta una función con tracking automático.
        
        Args:
            node: Nodo que está computando
            compute_fn: Función a ejecutar
            
        Returns:
            T: Resultado de la computación
        """
        if self._paused:
            # Tracking pausado, ejecutar sin tracking
            return compute_fn()
        
        return self._graph.track(node, compute_fn)
    
    def record_dependency(self, dependency: ReactiveNode) -> None:
        """
        Registra una dependencia en la computación activa.
        
        Args:
            dependency: Nodo del que depende
        """
        if self._paused:
            return
        
        self._graph.record_dependency(dependency)
    
    @contextmanager
    def pause(self):
        """
        Context manager para pausar el tracking temporalmente.
        
        Útil cuando se quiere leer signals sin crear dependencias.
        
        Ejemplo:
            with tracking_context.pause():
                value = signal.get()  # No crea dependencia
        """
        old_paused = self._paused
        self._paused = True
        
        try:
            yield
        finally:
            self._paused = old_paused
    
    @contextmanager
    def untrack(self):
        """Alias de pause() para compatibilidad."""
        with self.pause():
            yield


def track(compute_fn: Callable[[], T]) -> Callable[[], T]:
    """
    Decorator para funciones que deben hacer tracking automático.
    
    Args:
        compute_fn: Función a decorar
        
    Returns:
        Callable: Función decorada
        
    Ejemplo:
        @track
        def my_computed():
            return signal_a.get() + signal_b.get()
    """
    def wrapper(*args, **kwargs) -> T:
        # Obtener grafo global
        from . import get_global_graph
        graph = get_global_graph()
        
        # Si ya hay tracking activo, ejecutar directamente
        if graph.is_tracking:
            return compute_fn(*args, **kwargs)
        
        # Crear nodo temporal para tracking
        from .types import NodeType
        temp_node = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_fn)
        graph.register_node(temp_node)
        
        try:
            return graph.track(temp_node, lambda: compute_fn(*args, **kwargs))
        finally:
            graph.unregister_node(temp_node)
    
    return wrapper


def untrack(compute_fn: Callable[[], T]) -> T:
    """
    Ejecuta una función sin tracking de dependencias.
    
    Útil para leer signals dentro de un computed/effect sin crear
    dependencias.
    
    Args:
        compute_fn: Función a ejecutar
        
    Returns:
        T: Resultado de la computación
        
    Ejemplo:
        computed = Computed(lambda: {
            a = signal_a.get()  # Tracked
            b = untrack(lambda: signal_b.get())  # Not tracked
            return a + b
        })
    """
    from . import get_global_graph
    graph = get_global_graph()
    
    # Guardar computación actual
    current = graph.current_computation
    
    # Temporalmente vaciar el stack
    saved_stack = graph._active_computations.copy()
    graph._active_computations.clear()
    
    try:
        return compute_fn()
    finally:
        # Restaurar stack
        graph._active_computations = saved_stack


@contextmanager
def batch_updates():
    """
    Context manager para batching de actualizaciones.
    
    Todas las modificaciones a signals dentro del contexto
    se propagan en batch al final.
    
    Ejemplo:
        with batch_updates():
            signal_a.set(10)
            signal_b.set(20)
            signal_c.set(30)
        # Solo aquí se propagan los cambios
    """
    from . import get_global_graph
    graph = get_global_graph()
    
    if graph._is_batching:
        # Ya estamos en batch, no hacer nada
        yield
        return
    
    graph._is_batching = True
    graph._batch_queue.clear()
    
    try:
        yield
    finally:
        graph._is_batching = False
        graph._flush_batch()


def with_tracking(node: ReactiveNode, compute_fn: Callable[[], T]) -> T:
    """
    Ejecuta una función con tracking para un nodo específico.
    
    Args:
        node: Nodo que está computando
        compute_fn: Función a ejecutar
        
    Returns:
        T: Resultado de la computación
    """
    from . import get_global_graph
    graph = get_global_graph()
    return graph.track(node, compute_fn)


def record_dependency(dependency: ReactiveNode) -> None:
    """
    Registra una dependencia manualmente.
    
    Se llama automáticamente cuando se lee un signal,
    pero puede llamarse manualmente si es necesario.
    
    Args:
        dependency: Nodo del que depende
    """
    from . import get_global_graph
    graph = get_global_graph()
    graph.record_dependency(dependency)


if __name__ == "__main__":
    # Ejemplo de uso
    from .graph import ReactiveGraph, ReactiveNode
    from .types import NodeType
    
    graph = ReactiveGraph()
    
    # Signal
    signal = ReactiveNode(NodeType.SIGNAL, initial_value=10)
    graph.register_node(signal)
    
    # Computed con tracking automático
    @track
    def compute_doubled():
        record_dependency(signal)
        return signal.value * 2
    
    result = compute_doubled()
    print(f"Result: {result}")  # 20
    
    # Untrack example
    def compute_with_untrack():
        tracked = signal.value  # Tracked
        untracked = untrack(lambda: signal.value)  # Not tracked
        return tracked + untracked
    
    result2 = compute_with_untrack()
    print(f"Result with untrack: {result2}")  # 20
    
    # Batch updates
    with batch_updates():
        signal._value = 15
        signal._value = 20
        # Solo aquí se propagaría
    
    print("Tracking examples completed")
