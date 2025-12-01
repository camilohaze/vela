"""
Signal<T> - Valor Reactivo Mutable

Implementación de: US-06 - TASK-026
Historia: Sistema Reactivo
Fecha: 2025-12-01

Descripción:
Implementa Signal<T>, la primitiva base del sistema reactivo.
Un Signal es un valor mutable que notifica automáticamente a sus
dependientes cuando cambia.

Inspirado en:
- Vue 3 ref()
- SolidJS createSignal()
- Preact signals
- Svelte 5 $state
"""

from typing import Any, Callable, Generic, Optional, Set, TypeVar
import weakref

from .graph import ReactiveGraph, ReactiveNode
from .types import NodeType, NodeState, DisposedNodeError

def get_global_graph():
    """Import helper to avoid circular dependency."""
    from . import global_graph
    return global_graph


T = TypeVar('T')


class Signal(Generic[T]):
    """
    Signal<T> - Valor reactivo mutable.
    
    Un Signal es la primitiva base del sistema reactivo. Almacena un valor
    mutable y notifica automáticamente a todos sus dependientes (computed,
    effects, watchers) cuando cambia.
    
    Features:
    - Auto-tracking: Computed/effects registran dependencia al leer
    - Change notification: Propaga cambios automáticamente
    - Type-safe: Generic type T para type checking
    - Weak references: No memory leaks
    
    Example:
        count = Signal(0)
        doubled = Computed(lambda: count.get() * 2)
        
        count.set(5)  # doubled se actualiza automáticamente a 10
    """
    
    def __init__(
        self,
        initial_value: T,
        *,
        graph: Optional[ReactiveGraph] = None,
        signal_id: Optional[str] = None,
        equals: Optional[Callable[[T, T], bool]] = None,
    ):
        """
        Crea un nuevo Signal.
        
        Args:
            initial_value: Valor inicial del signal
            graph: Grafo reactivo (usa el global si no se provee)
            signal_id: ID personalizado (opcional)
            equals: Función de comparación personalizada (default: ==)
        """
        self._graph = graph or get_global_graph()
        self._equals = equals or self._default_equals
        
        # Crear nodo reactivo
        self._node = ReactiveNode(
            NodeType.SIGNAL,
            initial_value=initial_value,
            node_id=signal_id,
        )
        
        # Registrar en el grafo
        self._graph.register_node(self._node)
        
        # Metadata
        self._subscribers: Set[Callable[[T, T], None]] = set()
    
    @staticmethod
    def _default_equals(a: T, b: T) -> bool:
        """Comparación por defecto (==)."""
        return a == b
    
    @property
    def value(self) -> T:
        """
        Obtiene el valor actual (readonly property).
        
        Alias de get() para compatibilidad con property syntax.
        
        Returns:
            T: Valor actual
        """
        return self.get()
    
    @value.setter
    def value(self, new_value: T) -> None:
        """
        Establece un nuevo valor (property setter).
        
        Alias de set() para compatibilidad con property syntax.
        
        Args:
            new_value: Nuevo valor
        """
        self.set(new_value)
    
    def get(self) -> T:
        """
        Obtiene el valor actual del signal.
        
        Si se llama dentro de un computed, effect o watch,
        registra automáticamente la dependencia.
        
        Returns:
            T: Valor actual
            
        Raises:
            DisposedNodeError: Si el signal fue destruido
        """
        if self._node.state == NodeState.DISPOSED:
            raise DisposedNodeError(f"Signal {self._node.id} is disposed")
        
        # Auto-tracking: registrar dependencia en computación activa
        self._graph.record_dependency(self._node)
        
        return self._node.value
    
    def set(self, new_value: T) -> None:
        """
        Establece un nuevo valor y propaga cambios.
        
        Si el nuevo valor es igual al anterior (según self._equals),
        no se propagan cambios (optimización).
        
        Args:
            new_value: Nuevo valor
            
        Raises:
            DisposedNodeError: Si el signal fue destruido
        """
        if self._node.state == NodeState.DISPOSED:
            raise DisposedNodeError(f"Signal {self._node.id} is disposed")
        
        old_value = self._node.value
        
        # Comparar valores (skip si son iguales)
        if self._equals(old_value, new_value):
            return
        
        # Actualizar valor
        self._node._value = new_value
        
        # Notificar subscribers
        self._notify_subscribers(new_value, old_value)
        
        # Propagar cambio en el grafo
        self._graph.propagate_change(self._node)
    
    def update(self, updater_fn: Callable[[T], T]) -> None:
        """
        Actualiza el valor usando una función.
        
        Útil para actualizaciones funcionales inmutables.
        
        Args:
            updater_fn: Función que recibe el valor actual y retorna el nuevo
            
        Example:
            count = Signal(0)
            count.update(lambda x: x + 1)  # Incrementa en 1
        """
        self.set(updater_fn(self._node.value))
    
    def peek(self) -> T:
        """
        Obtiene el valor sin tracking de dependencias.
        
        Útil cuando se quiere leer el valor dentro de un computed/effect
        sin crear una dependencia.
        
        Returns:
            T: Valor actual
            
        Example:
            computed = Computed(lambda: {
                tracked = signal.get()      # Crea dependencia
                untracked = signal.peek()   # NO crea dependencia
                return tracked + untracked
            })
        """
        return self._node.value
    
    def subscribe(self, callback: Callable[[T, T], None]) -> Callable[[], None]:
        """
        Suscribe un callback para notificaciones directas.
        
        El callback se ejecuta cuando el valor cambia, independientemente
        del sistema reactivo. Útil para integraciones con librerías externas.
        
        Args:
            callback: Función (new_value, old_value) => void
            
        Returns:
            Callable: Función de unsubscribe
            
        Example:
            count = Signal(0)
            
            def on_change(new, old):
                print(f"Changed from {old} to {new}")
            
            unsubscribe = count.subscribe(on_change)
            count.set(5)  # "Changed from 0 to 5"
            unsubscribe()  # Desuscribir
        """
        self._subscribers.add(callback)
        
        # Retornar función de unsubscribe
        def unsubscribe():
            self._subscribers.discard(callback)
        
        return unsubscribe
    
    def _notify_subscribers(self, new_value: T, old_value: T) -> None:
        """
        Notifica a todos los subscribers.
        
        Args:
            new_value: Nuevo valor
            old_value: Valor anterior
        """
        for callback in self._subscribers.copy():
            try:
                callback(new_value, old_value)
            except Exception:
                # Ignorar errores en callbacks
                pass
    
    def dispose(self) -> None:
        """Destruye el signal y limpia recursos."""
        self._node.dispose()
        self._graph.unregister_node(self._node)
        self._subscribers.clear()
    
    @property
    def is_disposed(self) -> bool:
        """Si el signal fue destruido."""
        return self._node.state == NodeState.DISPOSED
    
    def __repr__(self) -> str:
        """Representación string del signal."""
        return f"Signal({self._node.value})"
    
    def __str__(self) -> str:
        """String representation."""
        return str(self._node.value)
    
    def __eq__(self, other) -> bool:
        """
        Compara signals por valor.
        
        Args:
            other: Otro signal o valor
            
        Returns:
            bool: True si los valores son iguales
        """
        if isinstance(other, Signal):
            return self._equals(self._node.value, other._node.value)
        return self._equals(self._node.value, other)
    
    def __hash__(self) -> int:
        """Hash basado en ID del nodo (para usar en sets/dicts)."""
        return hash(self._node.id)


# Type annotation helper
def signal(initial_value: T, **kwargs) -> Signal[T]:
    """
    Función helper para crear signals con inferencia de tipos.
    
    Args:
        initial_value: Valor inicial
        **kwargs: Argumentos adicionales para Signal()
        
    Returns:
        Signal[T]: Nuevo signal
        
    Example:
        count = signal(0)           # Signal[int]
        name = signal("Alice")      # Signal[str]
        items = signal([1, 2, 3])   # Signal[List[int]]
    """
    return Signal(initial_value, **kwargs)


if __name__ == "__main__":
    # Ejemplos de uso
    from .types import NodeState
    
    # Crear signal
    count = Signal(0)
    print(f"Initial: {count.get()}")  # 0
    
    # Actualizar valor
    count.set(5)
    print(f"After set: {count.get()}")  # 5
    
    # Update funcional
    count.update(lambda x: x + 1)
    print(f"After update: {count.get()}")  # 6
    
    # Property syntax
    count.value = 10
    print(f"After property: {count.value}")  # 10
    
    # Subscribe
    def on_change(new, old):
        print(f"Changed: {old} -> {new}")
    
    unsubscribe = count.subscribe(on_change)
    count.set(15)  # "Changed: 10 -> 15"
    
    unsubscribe()
    count.set(20)  # No output (unsubscribed)
    
    # Peek (sin tracking)
    peeked = count.peek()
    print(f"Peeked: {peeked}")  # 20
    
    # Dispose
    count.dispose()
    print(f"Disposed: {count.is_disposed}")  # True
    
    print("\nSignal examples completed")
