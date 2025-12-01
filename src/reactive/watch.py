"""
Watch - Explicit Watchers

Implementación de: VELA-573 - TASK-030
Historia: Sistema Reactivo
Fecha: 2025-12-01

Descripción:
Watch permite observar explícitamente cambios en signals o computed
values y ejecutar callbacks cuando cambian. Soporta modos immediate
y deep watching.
"""

from typing import Callable, Optional, Union, List, Any

from .graph import ReactiveGraph, ReactiveNode
from .signal import Signal
from .computed import Computed
from .types import NodeType, NodeState, DisposedNodeError


def get_global_graph():
    """Import helper to avoid circular dependency."""
    from . import global_graph
    return global_graph


# Type alias para sources
WatchSource = Union[Signal, Computed]
WatchCallback = Callable[[Any, Any], None]


class Watch:
    """
    Watch - Observador explícito de cambios.
    
    Watch permite observar cambios en uno o más signals/computed
    y ejecutar un callback cuando cambian. A diferencia de Effect,
    Watch es explícito sobre qué observar.
    
    Features:
    - Explicit sources: Especificas qué signals/computed observar
    - Callback con old/new values: Recibe valores anterior y nuevo
    - Immediate mode: Ejecuta callback inmediatamente o espera cambio
    - Multiple sources: Puede observar múltiples signals a la vez
    - Deep watching: Observa cambios profundos en objetos (futuro)
    
    Example:
        count = Signal(0)
        
        def on_change(new_val, old_val):
            print(f"Changed: {old_val} -> {new_val}")
        
        w = Watch(count, on_change)
        
        count.set(5)
        # Output: Changed: 0 -> 5
        
        w.stop()  # Pausar
        w.dispose()  # Limpiar
    """
    
    def __init__(
        self,
        source: Union[WatchSource, List[WatchSource]],
        callback: WatchCallback,
        *,
        immediate: bool = False,
        deep: bool = False,
        graph: Optional[ReactiveGraph] = None,
        watch_id: Optional[str] = None,
    ):
        """
        Crea un nuevo Watch.
        
        Args:
            source: Signal/Computed a observar, o lista de ellos
            callback: Función (new_value, old_value) => void
            immediate: Si True, ejecuta callback inmediatamente
            deep: Si True, observa cambios profundos (no implementado aún)
            graph: Grafo reactivo (usa el global si no se provee)
            watch_id: ID personalizado (opcional)
        """
        self._graph = graph or get_global_graph()
        self._callback = callback
        self._immediate = immediate
        self._deep = deep  # Para futuro soporte
        self._stopped = False
        
        # Convertir source a lista
        if not isinstance(source, list):
            self._sources = [source]
        else:
            self._sources = source
        
        # Valores anteriores para comparación
        self._old_values = [s.peek() if hasattr(s, 'peek') else s.get() for s in self._sources]
        
        # Crear effect interno que observa las sources
        self._node = ReactiveNode(
            NodeType.WATCH,
            compute_fn=self._watch_effect,
            node_id=watch_id,
        )
        
        # Registrar en el grafo
        self._graph.register_node(self._node)
        
        # Ejecutar inmediatamente si immediate=True
        if immediate:
            self._execute_callback()
        
        # Establecer tracking inicial
        self._setup_tracking()
    
    def _setup_tracking(self) -> None:
        """Establece el tracking de las sources."""
        # Track dependencies
        self._graph.track(self._node, self._watch_effect)
        self._node._state = NodeState.CLEAN
    
    def _watch_effect(self) -> None:
        """Función interna del effect que detecta cambios."""
        if self._stopped:
            return
        
        # Leer valores actuales (esto registra dependencias)
        current_values = []
        for source in self._sources:
            if isinstance(source, (Signal, Computed)):
                current_values.append(source.get())
            else:
                current_values.append(source)
        
        # Comparar con valores anteriores
        changed = False
        for i, (current, old) in enumerate(zip(current_values, self._old_values)):
            if current != old:
                changed = True
                break
        
        # Si cambió, ejecutar callback
        if changed:
            self._execute_callback()
            # Actualizar valores anteriores
            self._old_values = current_values
    
    def _execute_callback(self) -> None:
        """Ejecuta el callback con valores actuales y anteriores."""
        if self._stopped:
            return
        
        # Para múltiples sources, pasamos listas
        if len(self._sources) == 1:
            current = self._sources[0].peek() if hasattr(self._sources[0], 'peek') else self._sources[0].get()
            old = self._old_values[0]
            try:
                self._callback(current, old)
            except Exception:
                pass  # Ignorar errores en callback
        else:
            current_values = [s.peek() if hasattr(s, 'peek') else s.get() for s in self._sources]
            try:
                self._callback(current_values, self._old_values)
            except Exception:
                pass
    
    def stop(self) -> None:
        """
        Pausa el watcher (no ejecutará callback en cambios).
        
        Puede resumirse con resume().
        """
        self._stopped = True
    
    def resume(self) -> None:
        """
        Resume un watcher pausado.
        
        No ejecuta callback inmediatamente, solo reactiva el watching.
        Actualiza valores actuales sin ejecutar callback.
        """
        if self._stopped:
            self._stopped = False
            # Actualizar valores actuales SIN ejecutar callback
            # Esto evita que resume() dispare el callback inmediatamente
            for i, source in enumerate(self._sources):
                if isinstance(source, (Signal, Computed)):
                    self._old_values[i] = source.peek()
                else:
                    self._old_values[i] = source
            # Re-establecer tracking
            self._setup_tracking()
    
    def dispose(self) -> None:
        """
        Destruye el watcher y limpia recursos.
        """
        self._node.dispose()
        self._graph.unregister_node(self._node)
        self._stopped = True
        self._sources = []
        self._old_values = []
    
    @property
    def is_disposed(self) -> bool:
        """Si el watcher fue destruido."""
        return self._node.state == NodeState.DISPOSED
    
    @property
    def is_stopped(self) -> bool:
        """Si el watcher está pausado."""
        return self._stopped
    
    def __repr__(self) -> str:
        """Representación string del watcher."""
        if self.is_disposed:
            status = "disposed"
        elif self.is_stopped:
            status = "stopped"
        else:
            status = "active"
        
        source_count = len(self._sources)
        return f"Watch({self._node.id}, {source_count} sources, {status})"


def watch(
    source: Union[WatchSource, List[WatchSource]],
    callback: WatchCallback,
    **kwargs
) -> Watch:
    """
    Función helper para crear watchers.
    
    Args:
        source: Signal/Computed a observar, o lista
        callback: Función (new, old) => void
        **kwargs: Argumentos adicionales para Watch()
        
    Returns:
        Watch: Nuevo watcher
        
    Example:
        count = signal(0)
        w = watch(count, lambda new, old: print(f"{old} -> {new}"))
    """
    return Watch(source, callback, **kwargs)


if __name__ == "__main__":
    from .signal import Signal
    from .computed import Computed
    
    # Ejemplo básico
    count = Signal(0)
    
    def on_count_change(new_val, old_val):
        print(f"Count changed: {old_val} -> {new_val}")
    
    w = Watch(count, on_count_change)
    
    count.set(5)
    # Output: Count changed: 0 -> 5
    
    count.set(10)
    # Output: Count changed: 5 -> 10
    
    # Con immediate=True
    name = Signal("Alice")
    
    def on_name_change(new_val, old_val):
        print(f"Name: {old_val} -> {new_val}")
    
    w2 = Watch(name, on_name_change, immediate=True)
    # Output: Name: Alice -> Alice (ejecuta inmediatamente)
    
    name.set("Bob")
    # Output: Name: Alice -> Bob
    
    # Múltiples sources
    a = Signal(1)
    b = Signal(2)
    
    def on_multiple_change(new_vals, old_vals):
        print(f"Changed: {old_vals} -> {new_vals}")
    
    w3 = Watch([a, b], on_multiple_change)
    
    a.set(10)
    # Output: Changed: [1, 2] -> [10, 2]
    
    b.set(20)
    # Output: Changed: [10, 2] -> [10, 20]
    
    # Cleanup
    w.dispose()
    w2.dispose()
    w3.dispose()
    
    print("\nWatch examples completed")
