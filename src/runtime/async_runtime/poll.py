"""
Poll<T> - Estado de Polling

Implementación de: VELA-580 (TASK-047)
Sprint 18 - Async/Await

Representa el estado de un Future al ser polled:
- Ready(T): El Future ha completado con valor T
- Pending: El Future aún no está listo

Inspirado en Rust's std::task::Poll.
"""

from dataclasses import dataclass
from typing import TypeVar, Generic, Union
from enum import Enum, auto


T = TypeVar('T')


class PollState(Enum):
    """Estados posibles de un Poll"""
    READY = auto()   # Future completado
    PENDING = auto() # Future aún en progreso


@dataclass
class Poll(Generic[T]):
    """
    Resultado de polling un Future.
    
    Ejemplos:
    ```python
    # Future completado
    poll = Poll.ready(42)
    assert poll.state == PollState.READY
    assert poll.value == 42
    
    # Future pendiente
    poll = Poll.pending()
    assert poll.state == PollState.PENDING
    assert poll.value is None
    ```
    """
    
    state: PollState
    value: T = None
    
    @staticmethod
    def ready(value: T) -> 'Poll[T]':
        """
        Crea un Poll en estado Ready con un valor.
        
        Args:
            value: Valor del Future completado
            
        Returns:
            Poll[T] en estado READY
        """
        return Poll(state=PollState.READY, value=value)
    
    @staticmethod
    def pending() -> 'Poll[T]':
        """
        Crea un Poll en estado Pending.
        
        Returns:
            Poll[T] en estado PENDING
        """
        return Poll(state=PollState.PENDING)
    
    def is_ready(self) -> bool:
        """Verifica si el Poll está en estado Ready"""
        return self.state == PollState.READY
    
    def is_pending(self) -> bool:
        """Verifica si el Poll está en estado Pending"""
        return self.state == PollState.PENDING
    
    def unwrap(self) -> T:
        """
        Extrae el valor del Poll.
        
        Raises:
            ValueError: Si el Poll está en estado Pending
            
        Returns:
            El valor T del Poll
        """
        if self.is_pending():
            raise ValueError("Cannot unwrap a Pending poll")
        return self.value
    
    def unwrap_or(self, default: T) -> T:
        """
        Extrae el valor del Poll o retorna un default.
        
        Args:
            default: Valor por defecto si está Pending
            
        Returns:
            El valor T o default
        """
        return self.value if self.is_ready() else default
    
    def map(self, f):
        """
        Mapea el valor del Poll si está Ready.
        
        Args:
            f: Función de transformación (T) -> U
            
        Returns:
            Poll[U] transformado o Poll.pending()
        """
        if self.is_ready():
            return Poll.ready(f(self.value))
        return Poll.pending()
    
    def __repr__(self) -> str:
        if self.is_ready():
            return f"Poll::Ready({self.value})"
        return "Poll::Pending"
