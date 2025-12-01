"""
Batch API for Reactive System

Implementación de: VELA-574 - TASK-032
Historia: API Pública de Batching
Fecha: 2025-12-01

Descripción:
API pública ergonómica para batching manual de actualizaciones reactivas:
- batch() function y context manager
- Decorador @batch para funciones
- Helpers: start_batch(), end_batch(), flush_batch()
- Global batch scope management
- Nested batching support
"""

from typing import Any, Callable, Optional, TypeVar
from functools import wraps
from contextlib import contextmanager

from .graph import ReactiveGraph
from .scheduler import ReactiveScheduler, get_global_scheduler


# Type variable para decoradores
T = TypeVar('T')
F = TypeVar('F', bound=Callable[..., Any])


# Global state para batching
_global_graph: Optional[ReactiveGraph] = None
_batch_stack: list[tuple[ReactiveGraph, ReactiveScheduler]] = []


def set_global_graph(graph: ReactiveGraph) -> None:
    """
    Establece el grafo reactivo global.
    
    Args:
        graph: Grafo reactivo a usar como global
        
    Example:
        >>> graph = ReactiveGraph()
        >>> set_global_graph(graph)
        >>> batch(lambda: signal.set(10))  # Usa el grafo global
    """
    global _global_graph
    _global_graph = graph


def get_global_graph() -> ReactiveGraph:
    """
    Obtiene el grafo reactivo global.
    
    Returns:
        ReactiveGraph: Grafo global
        
    Raises:
        RuntimeError: Si no hay grafo global configurado
    """
    if _global_graph is None:
        raise RuntimeError(
            "No global graph configured. "
            "Call set_global_graph() first or pass graph explicitly."
        )
    return _global_graph


def batch(
    fn: Callable[[], T],
    graph: Optional[ReactiveGraph] = None,
) -> T:
    """
    Ejecuta una función en modo batch.
    
    Todos los updates se acumulan y ejecutan al final del batch.
    
    Args:
        fn: Función a ejecutar
        graph: Grafo opcional (usa global si no se provee)
        
    Returns:
        T: Resultado de la función
        
    Example:
        >>> batch(lambda: (
        ...     signal1.set(10),
        ...     signal2.set(20),
        ...     signal3.set(30)
        ... ))
        # Solo 1 propagación al final
        
        >>> # Con grafo explícito
        >>> batch(lambda: signal.set(10), graph=my_graph)
    """
    target_graph = graph or get_global_graph()
    return target_graph.batch(fn)


@contextmanager
def batching(graph: Optional[ReactiveGraph] = None):
    """
    Context manager para batching.
    
    Args:
        graph: Grafo opcional (usa global si no se provee)
        
    Yields:
        None
        
    Example:
        >>> with batching():
        ...     signal1.set(10)
        ...     signal2.set(20)
        # Propagación al salir del with
        
        >>> # Con grafo explícito
        >>> with batching(graph=my_graph):
        ...     signal.set(10)
    """
    target_graph = graph or get_global_graph()
    
    # Start batch
    target_graph._is_batching = True
    target_graph._scheduler._is_batching = True
    target_graph._scheduler._batch_depth += 1
    
    # Track en el stack
    _batch_stack.append((target_graph, target_graph._scheduler))
    
    try:
        yield
    finally:
        # End batch
        target_graph._scheduler._batch_depth -= 1
        
        # Solo flush cuando salimos del batch más externo
        if target_graph._scheduler._batch_depth == 0:
            target_graph._is_batching = False
            target_graph._scheduler._is_batching = False
            target_graph._scheduler.flush()
        
        # Pop del stack
        _batch_stack.pop()


def start_batch(graph: Optional[ReactiveGraph] = None) -> None:
    """
    Inicia un batch manualmente.
    
    IMPORTANTE: Debes llamar end_batch() para finalizar el batch.
    Preferir usar batch() o batching() context manager.
    
    Args:
        graph: Grafo opcional (usa global si no se provee)
        
    Example:
        >>> start_batch()
        >>> signal1.set(10)
        >>> signal2.set(20)
        >>> end_batch()  # Propagación aquí
    """
    target_graph = graph or get_global_graph()
    target_graph._is_batching = True
    target_graph._scheduler._is_batching = True
    target_graph._scheduler._batch_depth += 1
    _batch_stack.append((target_graph, target_graph._scheduler))


def end_batch(graph: Optional[ReactiveGraph] = None) -> None:
    """
    Finaliza un batch iniciado con start_batch().
    
    Args:
        graph: Grafo opcional (usa global si no se provee)
        
    Raises:
        RuntimeError: Si no hay batch activo
        
    Example:
        >>> start_batch()
        >>> try:
        ...     signal.set(10)
        ... finally:
        ...     end_batch()  # Asegurar cleanup
    """
    target_graph = graph or get_global_graph()
    
    if not _batch_stack:
        raise RuntimeError("No active batch to end. Call start_batch() first.")
    
    # Verificar que coincide el grafo
    stack_graph, _ = _batch_stack[-1]
    if stack_graph != target_graph:
        raise RuntimeError(
            "Batch graph mismatch. "
            "Ensure you're calling end_batch() with the same graph as start_batch()."
        )
    
    # Decrementar depth
    target_graph._scheduler._batch_depth -= 1
    
    # Flush si es el batch más externo
    if target_graph._scheduler._batch_depth == 0:
        target_graph._is_batching = False
        target_graph._scheduler._is_batching = False
        target_graph._scheduler.flush()
    
    # Pop del stack
    _batch_stack.pop()


def flush_batch(graph: Optional[ReactiveGraph] = None) -> None:
    """
    Flush manual de un batch sin finalizarlo.
    
    Útil para forzar propagación intermedia durante un batch largo.
    
    Args:
        graph: Grafo opcional (usa global si no se provee)
        
    Example:
        >>> with batching():
        ...     signal1.set(10)
        ...     signal2.set(20)
        ...     flush_batch()  # Propagación intermedia
        ...     signal3.set(30)
        # Propagación final al salir del with
    """
    target_graph = graph or get_global_graph()
    target_graph._scheduler.flush()


def is_batching(graph: Optional[ReactiveGraph] = None) -> bool:
    """
    Verifica si hay un batch activo.
    
    Args:
        graph: Grafo opcional (usa global si no se provee)
        
    Returns:
        bool: True si hay batch activo
        
    Example:
        >>> is_batching()  # False
        >>> with batching():
        ...     print(is_batching())  # True
    """
    target_graph = graph or get_global_graph()
    return target_graph._is_batching


def batch_decorator(
    graph: Optional[ReactiveGraph] = None,
) -> Callable[[F], F]:
    """
    Decorador para ejecutar funciones en batch automáticamente.
    
    Args:
        graph: Grafo opcional (usa global si no se provee)
        
    Returns:
        Callable: Decorador
        
    Example:
        >>> @batch_decorator()
        ... def update_signals():
        ...     signal1.set(10)
        ...     signal2.set(20)
        ...     signal3.set(30)
        >>> 
        >>> update_signals()  # Ejecuta en batch automáticamente
        
        >>> # Con grafo explícito
        >>> @batch_decorator(graph=my_graph)
        ... def update():
        ...     signal.set(10)
    """
    def decorator(fn: F) -> F:
        @wraps(fn)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            target_graph = graph or get_global_graph()
            return target_graph.batch(lambda: fn(*args, **kwargs))
        return wrapper  # type: ignore
    return decorator


# Alias más corto para el decorador
batch_fn = batch_decorator


class BatchScope:
    """
    Scope manager para batching con API fluent.
    
    Permite manejar múltiples batches de forma estructurada.
    
    Example:
        >>> scope = BatchScope()
        >>> scope.start()
        >>> signal1.set(10)
        >>> signal2.set(20)
        >>> scope.end()  # Propagación aquí
        
        >>> # O como context manager
        >>> with BatchScope():
        ...     signal.set(10)
    """
    
    def __init__(self, graph: Optional[ReactiveGraph] = None):
        """
        Inicializa un BatchScope.
        
        Args:
            graph: Grafo opcional (usa global si no se provee)
        """
        self.graph = graph or get_global_graph()
        self._active = False
    
    def start(self) -> 'BatchScope':
        """
        Inicia el batch scope.
        
        Returns:
            BatchScope: Self (para chaining)
        """
        if self._active:
            raise RuntimeError("BatchScope already active")
        
        start_batch(self.graph)
        self._active = True
        return self
    
    def end(self) -> None:
        """Finaliza el batch scope."""
        if not self._active:
            raise RuntimeError("BatchScope not active")
        
        end_batch(self.graph)
        self._active = False
    
    def flush(self) -> 'BatchScope':
        """
        Flush manual sin finalizar el scope.
        
        Returns:
            BatchScope: Self (para chaining)
        """
        if not self._active:
            raise RuntimeError("BatchScope not active")
        
        flush_batch(self.graph)
        return self
    
    def __enter__(self) -> 'BatchScope':
        """Context manager enter."""
        self.start()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Context manager exit."""
        if self._active:
            self.end()
    
    def __repr__(self) -> str:
        """String representation."""
        return f"BatchScope(active={self._active}, graph={self.graph})"


# Exports públicos
__all__ = [
    # Core API
    'batch',
    'batching',
    'start_batch',
    'end_batch',
    'flush_batch',
    'is_batching',
    
    # Decoradores
    'batch_decorator',
    'batch_fn',
    
    # Scope manager
    'BatchScope',
    
    # Global graph
    'set_global_graph',
    'get_global_graph',
]


if __name__ == "__main__":
    # Demo de la API
    print("Batch API - Demo\n")
    
    from .signal import Signal
    from .computed import Computed
    
    # Setup
    graph = ReactiveGraph()
    set_global_graph(graph)
    
    signal1 = Signal(0, graph=graph)
    signal2 = Signal(0, graph=graph)
    computed = Computed(
        lambda: signal1.get() + signal2.get(),
        graph=graph
    )
    
    print("1. batch() function:")
    batch(lambda: (
        signal1.set(10),
        signal2.set(20),
    ))
    print(f"   Result: {computed.get()}")
    
    print("\n2. batching() context manager:")
    with batching():
        signal1.set(100)
        signal2.set(200)
    print(f"   Result: {computed.get()}")
    
    print("\n3. @batch_decorator():")
    @batch_decorator()
    def update_both(a: int, b: int):
        signal1.set(a)
        signal2.set(b)
        return a + b
    
    result = update_both(50, 50)
    print(f"   Function returned: {result}")
    print(f"   Computed result: {computed.get()}")
    
    print("\n4. BatchScope:")
    with BatchScope() as scope:
        signal1.set(5)
        signal2.set(10)
    print(f"   Result: {computed.get()}")
    
    print("\n✅ Batch API demo completed!")
