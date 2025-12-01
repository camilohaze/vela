"""
Reactive Scheduler

Implementación de: VELA-574 - TASK-031
Historia: Scheduler Reactivo Avanzado
Fecha: 2025-12-01

Descripción:
Scheduler avanzado para el sistema reactivo con:
- Batching automático de actualizaciones
- Priorización de updates (signals > computed > effects)
- Scheduling inteligente (microtask queue)
- Coalescing de múltiples updates al mismo nodo
- Performance optimizations con memoization
"""

from typing import Any, Callable, Dict, List, Optional, Set, Tuple, TYPE_CHECKING
from collections import deque
from enum import Enum
import time

from .types import (
    IReactiveNode,
    NodeType,
    NodeState,
    CyclicDependencyError,
)

# Evitar import circular
if TYPE_CHECKING:
    from .graph import ReactiveNode


class SchedulerPriority(Enum):
    """
    Prioridades de scheduling.
    
    SYNC: Ejecución inmediata (signals)
    HIGH: Alta prioridad (computed)
    NORMAL: Prioridad normal (effects)
    LOW: Baja prioridad (cleanup, GC)
    """
    SYNC = 0      # Inmediato
    HIGH = 1      # Computed
    NORMAL = 2    # Effects
    LOW = 3       # Cleanup


class ScheduledUpdate:
    """
    Representa un update programado.
    
    Encapsula un nodo y su prioridad para scheduling.
    """
    
    def __init__(
        self,
        node: 'ReactiveNode',
        priority: SchedulerPriority,
        timestamp: float,
    ):
        """
        Inicializa un update programado.
        
        Args:
            node: Nodo a actualizar
            priority: Prioridad del update
            timestamp: Timestamp del scheduling
        """
        self.node = node
        self.priority = priority
        self.timestamp = timestamp
    
    def __lt__(self, other: 'ScheduledUpdate') -> bool:
        """Comparación para ordenamiento por prioridad."""
        if self.priority.value != other.priority.value:
            return self.priority.value < other.priority.value
        return self.timestamp < other.timestamp
    
    def __repr__(self) -> str:
        return (
            f"ScheduledUpdate(node={self.node.id}, "
            f"priority={self.priority.name}, ts={self.timestamp:.6f})"
        )


class ReactiveScheduler:
    """
    Scheduler reactivo avanzado.
    
    Gestiona la propagación eficiente de cambios con:
    - Batching automático
    - Priorización de updates
    - Coalescing de múltiples cambios
    - Microtask queue para async execution
    """
    
    def __init__(self):
        """Inicializa el scheduler."""
        # Queues por prioridad (usar Any para evitar import circular)
        self._sync_queue: deque = deque()
        self._high_queue: deque = deque()
        self._normal_queue: deque = deque()
        self._low_queue: deque = deque()
        
        # Tracking
        self._scheduled_nodes: Set[str] = set()  # IDs de nodos ya scheduled
        self._is_flushing: bool = False
        self._flush_depth: int = 0
        self._max_flush_depth: int = 100  # Prevenir loops infinitos
        
        # Batching
        self._is_batching: bool = False
        self._batch_depth: int = 0
        
        # Metrics
        self._metrics = {
            'total_updates': 0,
            'batched_updates': 0,
            'coalesced_updates': 0,
            'flush_count': 0,
        }
    
    @property
    def is_flushing(self) -> bool:
        """Si el scheduler está ejecutando updates."""
        return self._is_flushing
    
    @property
    def is_batching(self) -> bool:
        """Si el scheduler está en modo batch."""
        return self._is_batching
    
    @property
    def metrics(self) -> Dict[str, int]:
        """Métricas del scheduler."""
        return self._metrics.copy()
    
    def schedule_update(
        self,
        node: 'ReactiveNode',
        priority: Optional[SchedulerPriority] = None,
    ) -> None:
        """
        Programa un update para un nodo.
        
        Args:
            node: Nodo a actualizar
            priority: Prioridad (opcional, se infiere del tipo si no se provee)
        """
        # Incrementar total siempre
        self._metrics['total_updates'] += 1
        
        # Coalescing: si ya está scheduled, skip
        if node.id in self._scheduled_nodes:
            self._metrics['coalesced_updates'] += 1
            return
        
        # Inferir prioridad si no se provee
        if priority is None:
            priority = self._infer_priority(node)
        
        # Agregar a la queue correspondiente
        queue = self._get_queue(priority)
        queue.append(node)
        self._scheduled_nodes.add(node.id)
        
        # Si estamos en batch, no flush automático
        if self._is_batching:
            self._metrics['batched_updates'] += 1
            return
        
        # Flush automático solo para SYNC
        if priority == SchedulerPriority.SYNC:
            self.flush()
    
    def flush(self) -> None:
        """
        Ejecuta todos los updates pendientes.
        
        Procesa las queues en orden de prioridad:
        1. SYNC (signals)
        2. HIGH (computed)
        3. NORMAL (effects)
        4. LOW (cleanup)
        """
        if self._is_flushing:
            return  # Ya estamos flushing (evitar re-entrancy)
        
        self._is_flushing = True
        self._flush_depth = 0
        self._metrics['flush_count'] += 1
        
        try:
            while self._has_pending_updates():
                if self._flush_depth >= self._max_flush_depth:
                    raise RuntimeError(
                        f"Max flush depth ({self._max_flush_depth}) exceeded. "
                        "Possible infinite update loop."
                    )
                
                self._flush_depth += 1
                
                # Procesar en orden de prioridad
                self._flush_queue(self._sync_queue)
                self._flush_queue(self._high_queue)
                self._flush_queue(self._normal_queue)
                self._flush_queue(self._low_queue)
        finally:
            self._is_flushing = False
            self._flush_depth = 0
            self._scheduled_nodes.clear()
    
    def batch(self, fn: Callable[[], Any]) -> Any:
        """
        Ejecuta una función en modo batch.
        
        Todos los updates se acumulan y ejecutan al final del batch.
        
        Args:
            fn: Función a ejecutar
            
        Returns:
            Any: Resultado de la función
        
        Example:
            >>> scheduler.batch(lambda: (
            ...     count.set(1),
            ...     count.set(2),
            ...     count.set(3)
            ... ))
            # Solo 1 update al final
        """
        self._is_batching = True
        self._batch_depth += 1
        
        try:
            result = fn()
            return result
        finally:
            self._batch_depth -= 1
            
            # Solo flush cuando salimos del batch más externo
            if self._batch_depth == 0:
                self._is_batching = False
                self.flush()
    
    def _infer_priority(self, node: 'ReactiveNode') -> SchedulerPriority:
        """
        Infiere la prioridad de un nodo según su tipo.
        
        Args:
            node: Nodo
            
        Returns:
            SchedulerPriority: Prioridad inferida
        """
        type_priority_map = {
            NodeType.SIGNAL: SchedulerPriority.SYNC,
            NodeType.COMPUTED: SchedulerPriority.HIGH,
            NodeType.EFFECT: SchedulerPriority.NORMAL,
            NodeType.WATCH: SchedulerPriority.NORMAL,
        }
        return type_priority_map.get(node.node_type, SchedulerPriority.NORMAL)
    
    def _get_queue(self, priority: SchedulerPriority) -> deque:
        """
        Obtiene la queue para una prioridad.
        
        Args:
            priority: Prioridad
            
        Returns:
            deque: Queue correspondiente
        """
        queue_map = {
            SchedulerPriority.SYNC: self._sync_queue,
            SchedulerPriority.HIGH: self._high_queue,
            SchedulerPriority.NORMAL: self._normal_queue,
            SchedulerPriority.LOW: self._low_queue,
        }
        return queue_map[priority]
    
    def _has_pending_updates(self) -> bool:
        """Verifica si hay updates pendientes."""
        return bool(
            self._sync_queue or
            self._high_queue or
            self._normal_queue or
            self._low_queue
        )
    
    def _flush_queue(self, queue: deque) -> None:
        """
        Flush de una queue específica con ordenamiento topológico.
        
        Los nodos se procesan en orden topológico para garantizar
        que las dependencies se calculen antes que sus dependientes.
        
        Args:
            queue: Queue a procesar
        """
        # Agrupar todos los nodos a procesar
        nodes_to_process = []
        while queue:
            node = queue.popleft()
            if node.state != NodeState.CLEAN:
                nodes_to_process.append(node)
        
        if not nodes_to_process:
            return
        
        # Ordenar topológicamente usando in-degree
        # Esto garantiza que dependencies se procesen antes que dependientes
        in_degree = {}
        for node in nodes_to_process:
            # Contar cuántas dependencies del nodo están en nodes_to_process
            in_degree[node] = sum(
                1 for dep in node._dependencies 
                if dep in nodes_to_process and dep.state != NodeState.CLEAN
            )
        
        # Kahn's algorithm: empezar con nodos sin dependencies
        ready = deque([node for node in nodes_to_process if in_degree[node] == 0])
        sorted_nodes = []
        
        while ready:
            node = ready.popleft()
            sorted_nodes.append(node)
            
            # Decrementar in-degree de dependientes
            for dependent in node._dependents:
                if dependent in in_degree:
                    in_degree[dependent] -= 1
                    if in_degree[dependent] == 0:
                        ready.append(dependent)
        
        # Si no se procesaron todos, hay ciclo (usar orden original)
        if len(sorted_nodes) < len(nodes_to_process):
            sorted_nodes = nodes_to_process
        
        # Procesar en orden topológico
        for node in sorted_nodes:
            node.mark_dirty()
            
            try:
                if node.state == NodeState.DIRTY:
                    node.recompute()
            except Exception as e:
                print(f"Error updating {node.id}: {e}")
                node._last_error = e
    
    def clear(self) -> None:
        """Limpia todas las queues."""
        self._sync_queue.clear()
        self._high_queue.clear()
        self._normal_queue.clear()
        self._low_queue.clear()
        self._scheduled_nodes.clear()
    
    def __repr__(self) -> str:
        """Representación string del scheduler."""
        return (
            f"ReactiveScheduler("
            f"pending={len(self._scheduled_nodes)}, "
            f"flushing={self._is_flushing}, "
            f"batching={self._is_batching}"
            f")"
        )


# Global scheduler instance
_global_scheduler: Optional[ReactiveScheduler] = None


def get_global_scheduler() -> ReactiveScheduler:
    """
    Obtiene la instancia global del scheduler.
    
    Returns:
        ReactiveScheduler: Scheduler global
    """
    global _global_scheduler
    if _global_scheduler is None:
        _global_scheduler = ReactiveScheduler()
    return _global_scheduler


def set_global_scheduler(scheduler: ReactiveScheduler) -> None:
    """
    Establece el scheduler global.
    
    Args:
        scheduler: Nuevo scheduler
    """
    global _global_scheduler
    _global_scheduler = scheduler


if __name__ == "__main__":
    # Ejemplo de uso
    print("Reactive Scheduler - Demo\n")
    
    scheduler = ReactiveScheduler()
    
    # Simular updates
    from .signal import Signal
    
    count = Signal(0)
    
    print("1. Single update:")
    scheduler.schedule_update(count._node, SchedulerPriority.SYNC)
    print(f"   Metrics: {scheduler.metrics}\n")
    
    print("2. Batch updates:")
    def batch_updates():
        count.set(1)
        count.set(2)
        count.set(3)
    
    scheduler.batch(batch_updates)
    print(f"   Metrics: {scheduler.metrics}\n")
    
    print("3. Coalescing:")
    scheduler.schedule_update(count._node)
    scheduler.schedule_update(count._node)  # Coalesced
    scheduler.schedule_update(count._node)  # Coalesced
    scheduler.flush()
    print(f"   Metrics: {scheduler.metrics}")
