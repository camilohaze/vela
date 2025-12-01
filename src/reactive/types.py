"""
Tipos base para el sistema reactivo de Vela

Implementación de: US-06 - TASK-025
Historia: Sistema Reactivo
Fecha: 2025-12-01

Descripción:
Define los tipos base y protocolos para el sistema reactivo.
"""

from typing import Any, Callable, Generic, Optional, Protocol, Set, TypeVar
from enum import Enum, auto


T = TypeVar('T')
R = TypeVar('R')


class NodeType(Enum):
    """Tipos de nodos reactivos."""
    SIGNAL = auto()      # Valor mutable reactivo
    COMPUTED = auto()    # Valor derivado (lazy + cached)
    EFFECT = auto()      # Side effect automático
    WATCH = auto()       # Observer explícito


class NodeState(Enum):
    """Estados de un nodo reactivo."""
    CLEAN = auto()       # Valor actualizado
    DIRTY = auto()       # Necesita recalcularse
    COMPUTING = auto()   # En proceso de cálculo
    DISPOSED = auto()    # Nodo destruido


class IReactiveNode(Protocol):
    """Protocolo para nodos reactivos."""
    
    @property
    def id(self) -> str:
        """ID único del nodo."""
        ...
    
    @property
    def node_type(self) -> NodeType:
        """Tipo del nodo."""
        ...
    
    @property
    def state(self) -> NodeState:
        """Estado actual del nodo."""
        ...
    
    @property
    def dependencies(self) -> Set['IReactiveNode']:
        """Nodos de los que depende."""
        ...
    
    @property
    def dependents(self) -> Set['IReactiveNode']:
        """Nodos que dependen de este."""
        ...
    
    def mark_dirty(self) -> None:
        """Marca el nodo como dirty."""
        ...
    
    def recompute(self) -> Any:
        """Recalcula el valor del nodo."""
        ...
    
    def dispose(self) -> None:
        """Limpia el nodo del grafo."""
        ...


class ComputationFn(Protocol, Generic[R]):
    """Función de computación genérica."""
    
    def __call__(self) -> R:
        """Ejecuta la computación."""
        ...


class EffectFn(Protocol):
    """Función de side effect."""
    
    def __call__(self) -> Optional[Callable[[], None]]:
        """
        Ejecuta el effect.
        
        Returns:
            Optional[Callable]: Cleanup function (si existe)
        """
        ...


class WatchCallback(Protocol, Generic[T]):
    """Callback para watchers."""
    
    def __call__(self, new_value: T, old_value: T) -> None:
        """
        Se ejecuta cuando el valor observado cambia.
        
        Args:
            new_value: Nuevo valor
            old_value: Valor anterior
        """
        ...


class CyclicDependencyError(Exception):
    """Error cuando se detecta un ciclo de dependencias."""
    
    def __init__(self, cycle_path: list[str]):
        self.cycle_path = cycle_path
        cycle_str = ' -> '.join(cycle_path)
        super().__init__(f"Ciclo de dependencias detectado: {cycle_str}")


class ReactiveError(Exception):
    """Error base para el sistema reactivo."""
    pass


class DisposedNodeError(ReactiveError):
    """Error al intentar usar un nodo ya destruido."""
    pass


class InvalidStateError(ReactiveError):
    """Error de estado inválido."""
    pass
