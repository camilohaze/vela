"""
Sistema Reactivo de Vela

Implementación de: US-06
Sprint: 11
Fecha: 2025-12-01

Descripción:
Sistema reactivo completo con signals, computed values, effects y watchers.
Inspirado en Vue 3, SolidJS y Svelte 5.

Exports públicos:
- ReactiveGraph: Grafo de dependencias
- ReactiveNode: Nodo base del grafo
- TrackingContext: Contexto de auto-tracking
- Signal: Valor reactivo mutable
- Computed: Valor derivado (lazy + cached)
- Effect: Side effect reactivo
- Watch: Observer explícito
"""

from .graph import ReactiveGraph, ReactiveNode
from .tracking import TrackingContext, track, untrack
from .signal import Signal, signal
from .computed import Computed, computed
from .effect import Effect, effect
from .watch import Watch, watch
from .scheduler import ReactiveScheduler, SchedulerPriority
from .batch import (
    batch,
    batching,
    start_batch,
    end_batch,
    flush_batch,
    is_batching,
    batch_decorator,
    batch_fn,
    BatchScope,
    set_global_graph,
    get_global_graph as get_batch_global_graph,
)
from .memoization import (
    MemoCache,
    MemoizationManager,
    get_memo_manager,
    compute_cache_key,
    memoize,
)

__all__ = [
    # Core
    'ReactiveGraph',
    'ReactiveNode',
    'TrackingContext',
    'track',
    'untrack',
    
    # Primitives
    'Signal',
    'signal',
    'Computed',
    'computed',
    'Effect',
    'effect',
    'Watch',
    'watch',
    
    # Scheduler (VELA-574 - TASK-031)
    'ReactiveScheduler',
    'SchedulerPriority',
    
    # Batch API (VELA-574 - TASK-032)
    'batch',
    'batching',
    'start_batch',
    'end_batch',
    'flush_batch',
    'is_batching',
    'batch_decorator',
    'batch_fn',
    'BatchScope',
    'set_global_graph',
    
    # Memoization (VELA-574 - TASK-033)
    'MemoCache',
    'MemoizationManager',
    'get_memo_manager',
    'compute_cache_key',
    'memoize',
]

# Instancia global del grafo reactivo
global_graph = ReactiveGraph()


def get_global_graph() -> ReactiveGraph:
    """
    Obtiene la instancia global del grafo reactivo.
    
    Returns:
        ReactiveGraph: Instancia global
    """
    return global_graph


def reset_global_graph() -> None:
    """
    Resetea el grafo global (útil para testing).
    """
    global global_graph
    global_graph = ReactiveGraph()
