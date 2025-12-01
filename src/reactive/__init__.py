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

__all__ = [
    'ReactiveGraph',
    'ReactiveNode',
    'TrackingContext',
    'track',
    'untrack',
    'Signal',
    'signal',
    'Computed',
    'computed',
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
