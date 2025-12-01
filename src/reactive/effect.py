"""
Effect - Side Effects Reactivos

Implementación de: VELA-573 - TASK-029
Historia: Sistema Reactivo
Fecha: 2025-12-01

Descripción:
Effect ejecuta automáticamente side effects cuando sus
dependencias reactivas cambian. Soporta cleanup functions
para limpiar efectos previos.
"""

from typing import Callable, Optional

from .graph import ReactiveGraph, ReactiveNode
from .types import NodeType, NodeState, DisposedNodeError


def get_global_graph():
    """Import helper to avoid circular dependency."""
    from . import global_graph
    return global_graph


class Effect:
    """
    Effect - Side effect reactivo.
    
    Un Effect ejecuta automáticamente side effects cuando sus
    dependencias reactivas cambian. Se vuelve a ejecutar cada vez
    que cualquier señal/computed que lea durante su ejecución cambie.
    
    Features:
    - Auto-tracking: Dependencias se registran automáticamente
    - Cleanup: Función de cleanup se ejecuta antes de re-run
    - Stop/Resume: Puede pausarse y resumirse
    - Dispose: Limpieza completa al destruir
    
    Example:
        count = Signal(0)
        
        def log_count():
            print(f"Count: {count.get()}")
            return lambda: print("Cleanup")
        
        eff = Effect(log_count)  # Ejecuta inmediatamente
        
        count.set(5)  # Ejecuta de nuevo (con cleanup)
        
        eff.stop()  # Pausar
        count.set(10)  # NO ejecuta
        
        eff.dispose()  # Cleanup final
    """
    
    def __init__(
        self,
        effect_fn: Callable[[], Optional[Callable[[], None]]],
        *,
        graph: Optional[ReactiveGraph] = None,
        effect_id: Optional[str] = None,
    ):
        """
        Crea un nuevo Effect.
        
        Args:
            effect_fn: Función de effect () => cleanup? 
                      Puede retornar una función de cleanup (opcional)
            graph: Grafo reactivo (usa el global si no se provee)
            effect_id: ID personalizado (opcional)
        """
        self._graph = graph or get_global_graph()
        self._effect_fn = effect_fn
        self._cleanup_fn: Optional[Callable[[], None]] = None
        self._stopped = False
        
        # Crear nodo reactivo
        self._node = ReactiveNode(
            NodeType.EFFECT,
            compute_fn=self._run_effect,
            node_id=effect_id,
        )
        
        # Registrar en el grafo
        self._graph.register_node(self._node)
        
        # Ejecutar inmediatamente
        self.run()
    
    def _run_effect(self) -> None:
        """Función interna que ejecuta el effect con cleanup."""
        if self._stopped:
            return
        
        # Ejecutar cleanup anterior (si existe)
        if self._cleanup_fn is not None:
            try:
                self._cleanup_fn()
            except Exception:
                pass  # Ignorar errores en cleanup
            finally:
                self._cleanup_fn = None
        
        # Ejecutar effect
        result = self._effect_fn()
        
        # Guardar nueva función de cleanup (si existe)
        if callable(result):
            self._cleanup_fn = result
    
    def run(self) -> None:
        """
        Ejecuta el effect manualmente.
        
        Normalmente los effects se ejecutan automáticamente cuando
        sus dependencias cambian, pero este método permite
        forzar ejecución manual.
        
        Raises:
            DisposedNodeError: Si el effect fue destruido
        """
        if self._node.state == NodeState.DISPOSED:
            raise DisposedNodeError(f"Effect {self._node.id} is disposed")
        
        if self._stopped:
            return
        
        # Track dependencies y ejecutar
        self._graph.track(self._node, self._run_effect)
        self._node._state = NodeState.CLEAN
    
    def stop(self) -> None:
        """
        Pausa el effect (no se ejecutará en cambios).
        
        Puede resumirse con resume().
        """
        self._stopped = True
    
    def resume(self) -> None:
        """
        Resume un effect pausado.
        
        Ejecuta el effect inmediatamente después de resumir.
        """
        if self._stopped:
            self._stopped = False
            self.run()
    
    def dispose(self) -> None:
        """
        Destruye el effect y limpia recursos.
        
        Ejecuta el cleanup final y desregistra del grafo.
        """
        # Ejecutar cleanup final
        if self._cleanup_fn is not None:
            try:
                self._cleanup_fn()
            except Exception:
                pass
            finally:
                self._cleanup_fn = None
        
        # Limpiar nodo
        self._node.dispose()
        self._graph.unregister_node(self._node)
        self._stopped = True
    
    @property
    def is_disposed(self) -> bool:
        """Si el effect fue destruido."""
        return self._node.state == NodeState.DISPOSED
    
    @property
    def is_stopped(self) -> bool:
        """Si el effect está pausado."""
        return self._stopped
    
    def __repr__(self) -> str:
        """Representación string del effect."""
        if self.is_disposed:
            status = "disposed"
        elif self.is_stopped:
            status = "stopped"
        else:
            status = "active"
        return f"Effect({self._node.id}, {status})"


def effect(
    effect_fn: Callable[[], Optional[Callable[[], None]]],
    **kwargs
) -> Effect:
    """
    Función helper para crear effects.
    
    Args:
        effect_fn: Función de effect
        **kwargs: Argumentos adicionales para Effect()
        
    Returns:
        Effect: Nuevo effect
        
    Example:
        count = signal(0)
        eff = effect(lambda: print(f"Count: {count.get()}"))
    """
    return Effect(effect_fn, **kwargs)


if __name__ == "__main__":
    from .signal import Signal
    
    # Ejemplo básico
    count = Signal(0)
    
    def log_count():
        print(f"Count: {count.get()}")
    
    eff = Effect(log_count)
    # Output: Count: 0
    
    count.set(5)
    # Output: Count: 5
    
    count.set(10)
    # Output: Count: 10
    
    # Con cleanup
    iterations = []
    
    def with_cleanup():
        current = count.get()
        iterations.append(current)
        print(f"Effect running, count: {current}")
        
        def cleanup():
            print(f"Cleaning up iteration {current}")
        
        return cleanup
    
    eff2 = Effect(with_cleanup)
    # Output: Effect running, count: 10
    
    count.set(15)
    # Output: Cleaning up iteration 10
    # Output: Effect running, count: 15
    
    eff2.dispose()
    # Output: Cleaning up iteration 15
    
    print("\nEffect examples completed")
