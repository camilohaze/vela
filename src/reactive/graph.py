"""
Grafo de Dependencias Reactivo

Implementación de: US-06 - TASK-025
Historia: Sistema Reactivo
Fecha: 2025-12-01

Actualizado: VELA-574 - US-07 - TASK-031
Historia: Scheduler Reactivo Avanzado
Fecha: 2025-12-01

Descripción:
Implementa el grafo de dependencias reactivo con:
- Auto-tracking de dependencias
- Propagación eficiente de cambios (push-based)
- Detección de ciclos
- Batching avanzado con scheduler priorizado
- Garbage collection automático
"""

from typing import Any, Callable, Dict, List, Optional, Set
from collections import deque
from contextlib import contextmanager
import uuid

from .types import (
    IReactiveNode,
    NodeType,
    NodeState,
    CyclicDependencyError,
    DisposedNodeError,
    InvalidStateError,
)
from .scheduler import ReactiveScheduler, SchedulerPriority


class ReactiveNode:
    """
    Nodo base del grafo reactivo.
    
    Representa un valor reactivo (signal, computed, effect, watch) en el grafo.
    Mantiene referencias a sus dependencias y dependientes para propagación eficiente.
    """
    
    def __init__(
        self,
        node_type: NodeType,
        compute_fn: Optional[Callable[[], Any]] = None,
        initial_value: Any = None,
        node_id: Optional[str] = None,
    ):
        """
        Inicializa un nodo reactivo.
        
        Args:
            node_type: Tipo del nodo (SIGNAL, COMPUTED, EFFECT, WATCH)
            compute_fn: Función de computación (para computed/effect/watch)
            initial_value: Valor inicial (para signals)
            node_id: ID personalizado (opcional, se genera UUID si no se provee)
        """
        self._id = node_id or f"{node_type.name.lower()}_{uuid.uuid4().hex[:8]}"
        self._node_type = node_type
        self._compute_fn = compute_fn
        self._value = initial_value
        self._state = NodeState.CLEAN
        
        # Grafo de dependencias
        self._dependencies: Set[ReactiveNode] = set()
        self._dependents: Set[ReactiveNode] = set()
        
        # Metadata
        self._cleanup_fn: Optional[Callable[[], None]] = None
        self._last_error: Optional[Exception] = None
    
    @property
    def id(self) -> str:
        """ID único del nodo."""
        return self._id
    
    @property
    def node_type(self) -> NodeType:
        """Tipo del nodo."""
        return self._node_type
    
    @property
    def state(self) -> NodeState:
        """Estado actual del nodo."""
        return self._state
    
    @property
    def value(self) -> Any:
        """Valor actual (cached)."""
        return self._value
    
    @property
    def dependencies(self) -> Set['ReactiveNode']:
        """Nodos de los que depende (inmutable view)."""
        return self._dependencies.copy()
    
    @property
    def dependents(self) -> Set['ReactiveNode']:
        """Nodos que dependen de este (inmutable view)."""
        return self._dependents.copy()
    
    def add_dependency(self, dependency: 'ReactiveNode') -> None:
        """
        Agrega una dependencia a este nodo.
        
        Args:
            dependency: Nodo del que depende
        """
        if self._state == NodeState.DISPOSED:
            raise DisposedNodeError(f"Node {self.id} is disposed")
        
        self._dependencies.add(dependency)
        dependency._dependents.add(self)
    
    def remove_dependency(self, dependency: 'ReactiveNode') -> None:
        """
        Remueve una dependencia de este nodo.
        
        Args:
            dependency: Nodo a remover
        """
        self._dependencies.discard(dependency)
        dependency._dependents.discard(self)
    
    def clear_dependencies(self) -> None:
        """Limpia todas las dependencias."""
        for dependency in self._dependencies.copy():
            self.remove_dependency(dependency)
    
    def mark_dirty(self) -> None:
        """Marca el nodo como dirty (necesita recalcularse)."""
        if self._state == NodeState.DISPOSED:
            return
        
        if self._state == NodeState.CLEAN:
            self._state = NodeState.DIRTY
    
    def recompute(self) -> Any:
        """
        Recalcula el valor del nodo.
        
        Returns:
            Any: Nuevo valor calculado
            
        Raises:
            InvalidStateError: Si el nodo está en estado inválido
        """
        if self._state == NodeState.DISPOSED:
            raise DisposedNodeError(f"Node {self.id} is disposed")
        
        if self._state == NodeState.COMPUTING:
            # Ya estamos computando (posible ciclo)
            return self._value
        
        if self._compute_fn is None:
            # Signals no se recomputan (su valor se setea directamente)
            self._state = NodeState.CLEAN
            return self._value
        
        # Marcar como computing
        self._state = NodeState.COMPUTING
        
        try:
            # Ejecutar cleanup previo (para effects)
            if self._cleanup_fn is not None:
                self._cleanup_fn()
                self._cleanup_fn = None
            
            # Computar nuevo valor
            new_value = self._compute_fn()
            
            # Guardar cleanup (para effects)
            if callable(new_value) and self._node_type == NodeType.EFFECT:
                self._cleanup_fn = new_value
                new_value = None
            
            # Actualizar valor
            self._value = new_value
            self._state = NodeState.CLEAN
            self._last_error = None
            
            return new_value
            
        except Exception as e:
            self._state = NodeState.DIRTY
            self._last_error = e
            raise
    
    def dispose(self) -> None:
        """Limpia el nodo del grafo."""
        if self._state == NodeState.DISPOSED:
            return
        
        # Ejecutar cleanup final
        if self._cleanup_fn is not None:
            try:
                self._cleanup_fn()
            except Exception:
                pass  # Ignorar errores en cleanup
        
        # Limpiar dependencias
        self.clear_dependencies()
        
        # Remover de dependientes
        for dependent in self._dependents.copy():
            dependent._dependencies.discard(self)
        self._dependents.clear()
        
        # Marcar como disposed
        self._state = NodeState.DISPOSED
        self._value = None
        self._compute_fn = None
        self._cleanup_fn = None
    
    def __repr__(self) -> str:
        """Representación string del nodo."""
        return (
            f"ReactiveNode(id={self.id}, type={self.node_type.name}, "
            f"state={self.state.name}, deps={len(self._dependencies)}, "
            f"dependents={len(self._dependents)})"
        )


class ReactiveGraph:
    """
    Grafo de dependencias reactivo.
    
    Gestiona el grafo completo de nodos reactivos y coordina:
    - Tracking automático de dependencias
    - Propagación de cambios
    - Detección de ciclos
    - Batching de actualizaciones
    """
    
    def __init__(self, scheduler: Optional[ReactiveScheduler] = None):
        """
        Inicializa un grafo reactivo vacío.
        
        Args:
            scheduler: Scheduler opcional (si no se provee, usa uno nuevo)
        """
        self._nodes: Dict[str, ReactiveNode] = {}
        self._active_computations: List[ReactiveNode] = []
        self._batch_queue: Set[ReactiveNode] = set()
        self._is_batching: bool = False
        
        # Scheduler avanzado (VELA-574 - TASK-031)
        self._scheduler: ReactiveScheduler = scheduler or ReactiveScheduler()
    
    @property
    def node_count(self) -> int:
        """Número de nodos en el grafo."""
        return len(self._nodes)
    
    @property
    def is_tracking(self) -> bool:
        """Si hay una computación activa (tracking enabled)."""
        return len(self._active_computations) > 0
    
    @property
    def current_computation(self) -> Optional[ReactiveNode]:
        """Computación activa actual (o None)."""
        return self._active_computations[-1] if self._active_computations else None
    
    def register_node(self, node: ReactiveNode) -> None:
        """
        Registra un nodo en el grafo.
        
        Args:
            node: Nodo a registrar
        """
        self._nodes[node.id] = node
    
    def unregister_node(self, node: ReactiveNode) -> None:
        """
        Remueve un nodo del grafo.
        
        Args:
            node: Nodo a remover
        """
        self._nodes.pop(node.id, None)
    
    def get_node(self, node_id: str) -> Optional[ReactiveNode]:
        """
        Obtiene un nodo por su ID.
        
        Args:
            node_id: ID del nodo
            
        Returns:
            Optional[ReactiveNode]: Nodo o None si no existe
        """
        return self._nodes.get(node_id)
    
    def track(self, node: ReactiveNode, compute_fn: Callable[[], Any]) -> Any:
        """
        Ejecuta una función con tracking automático de dependencias.
        
        Args:
            node: Nodo que está computando
            compute_fn: Función a ejecutar
            
        Returns:
            Any: Resultado de la computación
        """
        # Limpiar dependencias previas
        node.clear_dependencies()
        
        # Push al stack de computaciones
        self._active_computations.append(node)
        
        try:
            # Ejecutar computación (auto-tracking)
            result = compute_fn()
            return result
        finally:
            # Pop del stack
            self._active_computations.pop()
    
    def record_dependency(self, dependency: ReactiveNode) -> None:
        """
        Registra una dependencia en la computación activa.
        
        Se llama automáticamente cuando se lee un signal dentro de
        un computed, effect o watch.
        
        Args:
            dependency: Nodo del que depende la computación actual
        """
        if not self._active_computations:
            return  # No hay tracking activo
        
        current = self._active_computations[-1]
        
        if current == dependency:
            return  # No auto-dependencia
        
        # Agregar dependencia
        current.add_dependency(dependency)
    
    def propagate_change(self, changed_node: ReactiveNode) -> None:
        """
        Propaga un cambio desde un nodo modificado usando el scheduler.
        
        El scheduler maneja:
        - Batching automático
        - Priorización por tipo de nodo
        - Coalescing de updates múltiples
        
        Args:
            changed_node: Nodo que cambió
        """
        if self._is_batching:
            # En modo batch, solo acumular
            self._batch_queue.add(changed_node)
            return
        
        # Usar scheduler para programar update
        # El scheduler inferirá la prioridad según el tipo de nodo
        self._scheduler.schedule_update(changed_node)
    
    def _propagate_immediate(self, changed_node: ReactiveNode) -> None:
        """
        Propagación inmediata sin scheduler (para testing).
        
        Usa BFS para marcar todos los dependientes como dirty,
        luego recalcula en orden topológico.
        
        Args:
            changed_node: Nodo que cambió
        """
        # Marcar como dirty
        changed_node.mark_dirty()
        
        # BFS para marcar dependientes
        to_update = self._mark_dirty_dependents(changed_node)
        
        # Detectar ciclos
        self._detect_cycles(to_update)
        
        # Ordenar topológicamente
        sorted_nodes = self._topological_sort(to_update)
        
        # Recalcular en orden
        for node in sorted_nodes:
            if node.state == NodeState.DIRTY:
                node.recompute()
    
    def _mark_dirty_dependents(self, changed_node: ReactiveNode) -> Set[ReactiveNode]:
        """
        Marca todos los dependientes como dirty usando BFS.
        
        Args:
            changed_node: Nodo que cambió
            
        Returns:
            Set[ReactiveNode]: Conjunto de nodos a actualizar
        """
        to_update: Set[ReactiveNode] = {changed_node}
        queue: deque[ReactiveNode] = deque([changed_node])
        visited: Set[ReactiveNode] = set()
        
        while queue:
            node = queue.popleft()
            
            if node in visited:
                continue
            visited.add(node)
            
            # Marcar dependientes como dirty
            for dependent in node._dependents:
                if dependent.state != NodeState.DIRTY:
                    dependent.mark_dirty()
                    to_update.add(dependent)
                    queue.append(dependent)
        
        return to_update
    
    def _topological_sort(self, nodes: Set[ReactiveNode]) -> List[ReactiveNode]:
        """
        Ordena nodos topológicamente para recalculación.
        
        Args:
            nodes: Nodos a ordenar
            
        Returns:
            List[ReactiveNode]: Nodos ordenados
        """
        # Calcular in-degree
        in_degree: Dict[ReactiveNode, int] = {}
        for node in nodes:
            in_degree[node] = sum(
                1 for dep in node._dependencies if dep in nodes
            )
        
        # Kahn's algorithm
        queue = deque([node for node in nodes if in_degree[node] == 0])
        sorted_nodes: List[ReactiveNode] = []
        
        while queue:
            node = queue.popleft()
            sorted_nodes.append(node)
            
            for dependent in node._dependents:
                if dependent not in nodes:
                    continue
                
                in_degree[dependent] -= 1
                if in_degree[dependent] == 0:
                    queue.append(dependent)
        
        return sorted_nodes
    
    def _detect_cycles(self, nodes: Set[ReactiveNode]) -> None:
        """
        Detecta ciclos en el subgrafo.
        
        Args:
            nodes: Nodos a revisar
            
        Raises:
            CyclicDependencyError: Si se detecta un ciclo
        """
        visited: Set[ReactiveNode] = set()
        rec_stack: Set[ReactiveNode] = set()
        
        def dfs(node: ReactiveNode, path: List[str]) -> None:
            if node in rec_stack:
                # Ciclo detectado
                cycle_start = path.index(node.id)
                cycle_path = path[cycle_start:] + [node.id]
                raise CyclicDependencyError(cycle_path)
            
            if node in visited:
                return
            
            visited.add(node)
            rec_stack.add(node)
            path.append(node.id)
            
            for dependency in node._dependencies:
                if dependency in nodes:
                    dfs(dependency, path.copy())
            
            rec_stack.remove(node)
        
        for node in nodes:
            if node not in visited:
                dfs(node, [])
    
    def batch(self, fn: Callable[[], Any]) -> Any:
        """
        Ejecuta una función en modo batch con scheduler avanzado.
        
        Acumula todos los cambios y los propaga al final usando el scheduler.
        Soporta batches anidados.
        
        Args:
            fn: Función a ejecutar
            
        Returns:
            Any: Resultado de la función
            
        Example:
            >>> graph.batch(lambda: (
            ...     signal1.set(10),
            ...     signal2.set(20),
            ...     signal3.set(30)
            ... ))
            # Solo 1 propagación al final
        """
        # Usar el scheduler para manejar batching
        return self._scheduler.batch(fn)
    
    @contextmanager
    def batching(self):
        """
        Context manager para batching.
        
        Example:
            >>> with graph.batching():
            ...     signal1.set(10)
            ...     signal2.set(20)
            # Propagación al salir del with
        """
        self._is_batching = True
        try:
            yield
        finally:
            self._is_batching = False
            self._flush_batch()
    
    def _flush_batch(self) -> None:
        """Propaga todos los cambios acumulados en batch."""
        if not self._batch_queue:
            return
        
        # Marcar todos como dirty
        all_dirty: Set[ReactiveNode] = set()
        for node in self._batch_queue:
            node.mark_dirty()
            all_dirty.update(self._mark_dirty_dependents(node))
        
        # Detectar ciclos
        self._detect_cycles(all_dirty)
        
        # Ordenar y recalcular
        sorted_nodes = self._topological_sort(all_dirty)
        
        for node in sorted_nodes:
            if node.state == NodeState.DIRTY:
                node.recompute()
        
        self._batch_queue.clear()
    
    def dispose_all(self) -> None:
        """Destruye todos los nodos del grafo."""
        for node in list(self._nodes.values()):
            node.dispose()
        
        self._nodes.clear()
        self._active_computations.clear()
        self._batch_queue.clear()
    
    def debug_info(self) -> Dict[str, Any]:
        """
        Información de debugging del grafo.
        
        Returns:
            Dict: Información del grafo
        """
        nodes_by_type = {}
        for node_type in NodeType:
            nodes_by_type[node_type.name] = sum(
                1 for n in self._nodes.values() if n.node_type == node_type
            )
        
        return {
            'total_nodes': self.node_count,
            'nodes_by_type': nodes_by_type,
            'is_tracking': self.is_tracking,
            'is_batching': self._is_batching,
            'batch_queue_size': len(self._batch_queue),
            'active_computations': len(self._active_computations),
        }


if __name__ == "__main__":
    # Ejemplo de uso
    graph = ReactiveGraph()
    
    # Crear signal
    signal_node = ReactiveNode(NodeType.SIGNAL, initial_value=10)
    graph.register_node(signal_node)
    
    # Crear computed
    def compute_doubled():
        # Simular lectura del signal
        graph.record_dependency(signal_node)
        return signal_node.value * 2
    
    computed_node = ReactiveNode(NodeType.COMPUTED, compute_fn=compute_doubled)
    graph.register_node(computed_node)
    
    # Tracking y computación inicial
    result = graph.track(computed_node, compute_doubled)
    print(f"Computed result: {result}")  # 20
    print(f"Dependencies: {len(computed_node.dependencies)}")  # 1
    
    # Cambiar signal
    signal_node._value = 15
    graph.propagate_change(signal_node)
    
    print(f"New computed result: {computed_node.value}")  # 30
    
    # Debug info
    print(f"Debug info: {graph.debug_info()}")
