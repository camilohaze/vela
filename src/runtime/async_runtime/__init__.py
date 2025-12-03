"""
Vela Async Runtime

Implementación de Future<T> y Promise<T> para async/await.

Implementación de: VELA-580 (TASK-047)
Sprint 18 - Async/Await
Fecha: 2025-12-02

Este módulo implementa los tipos fundamentales para programación asíncrona en Vela:
- Future<T>: Consumidor (leer valor futuro)
- Promise<T>: Productor (escribir valor futuro)
- Poll<T>: Estado de polling
- Waker: Mecanismo de wake-up

Inspirado en:
- Rust: Future trait, Waker
- JavaScript: Promise API
- Swift: Task/async/await
"""

from .poll import Poll, PollState
from .waker import Waker
from .future import Future
from .promise import Promise

__all__ = [
    'Poll',
    'PollState',
    'Waker',
    'Future',
    'Promise',
]
