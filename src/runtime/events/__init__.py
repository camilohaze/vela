"""
Event System Runtime - Sistema de eventos type-safe de Vela

Módulo que implementa el sistema de eventos genérico para comunicación
desacoplada entre componentes.

TASK-035L: Implementar EventBus<T> core
Sprint: 14
Epic: EPIC-03C - Event System

Exports:
    - Event: Generic event object
    - EventBus: Type-safe event bus
    - AutoDisposeEventBus: Event bus con auto-dispose
    - Subscription: Subscription object para unsubscribe
    - get_global_bus: Get global event bus singleton
"""

from .event_bus import (
    Event,
    EventListener,
    Subscription,
    EventBus,
    AutoDisposeEventBus,
    get_global_bus,
)

__all__ = [
    'Event',
    'EventListener',
    'Subscription',
    'EventBus',
    'AutoDisposeEventBus',
    'get_global_bus',
]

__version__ = '1.0.0'
