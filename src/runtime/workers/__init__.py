"""
Vela Worker API

Implementación de: VELA-580 (TASK-050)
Sprint 19 - Workers y Channels
Fecha: 2025-01-28

Este módulo implementa Worker API para computación paralela:
- Worker: API estática para spawn de workers
- WorkerHandle: Handle interno para tracking de workers
- WorkerPool: Thread pool para ejecutar workers

Inspirado en:
- Rust: tokio::task::spawn_blocking, rayon::spawn
- Go: goroutines
- Swift: Task.detached
- Java: ExecutorService
"""

from .worker_handle import WorkerHandle
from .worker_pool import WorkerPool
from .worker import Worker

__all__ = [
    'Worker',
    'WorkerHandle',
    'WorkerPool',
]
