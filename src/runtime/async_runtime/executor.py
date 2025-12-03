"""
Executor - Event loop para ejecutar Futures

Implementación de:
- Executor: Event loop principal con task scheduling
- spawn(): Crear y registrar task
- run_until_complete(): Ejecutar hasta completar future
- run(): Ejecutar event loop indefinidamente
- step(): Ejecutar un paso del event loop

Jira: VELA-580
Historia: TASK-048
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import TypeVar, Generic, Optional, List, Dict, Callable
from collections import deque
from threading import Lock
import time

from .future import Future
from .poll import Poll
from .waker import Waker
from .task import Task, TaskHandle, TaskId, TaskState


T = TypeVar('T')


@dataclass
class Executor:
    """
    Executor - Event loop para ejecutar Futures
    
    Maneja:
    - Ready queue: Tareas listas para polling
    - Waiting tasks: Tareas esperando wake-up
    - Wakers: Despertadores por task
    
    Flujo:
    1. Pop task de ready_queue
    2. Poll task con waker
    3. Si Ready → Completar
    4. Si Pending → Mover a waiting, registrar waker
    5. Esperar wake-ups
    6. Waker despierta → Mover de waiting a ready_queue
    7. Repetir
    
    Ejemplo:
        executor = Executor()
        
        # Spawn task
        future = Future.ready(42)
        handle = executor.spawn(future)
        
        # Run hasta completar
        result = executor.run_until_complete(future)
        
        # Run indefinidamente
        executor.run()
    """
    
    # Ready queue: tareas listas para polling
    ready_queue: deque[Task] = field(default_factory=deque)
    
    # Waiting tasks: tareas esperando wake-up
    waiting: Dict[TaskId, Task] = field(default_factory=dict)
    
    # Wakers registrados por task
    wakers: Dict[TaskId, Waker] = field(default_factory=dict)
    
    # Lock para thread safety
    _lock: Lock = field(default_factory=Lock)
    
    # Flag para detener event loop
    _running: bool = False
    
    def spawn(self, future: Future[T]) -> TaskHandle[T]:
        """
        Spawn un nuevo task
        
        Args:
            future: Future a ejecutar
            
        Returns:
            TaskHandle para controlar el task
        """
        task = Task(future)
        
        with self._lock:
            # Agregar a ready queue
            self.ready_queue.append(task)
        
        return TaskHandle(task)
    
    def run_until_complete(self, future: Future[T], timeout: Optional[float] = None) -> T:
        """
        Ejecuta event loop hasta que future complete
        
        Args:
            future: Future a esperar
            timeout: Timeout en segundos (None = sin timeout)
            
        Returns:
            Resultado del future
            
        Raises:
            Exception: Si future falla
            TimeoutError: Si timeout alcanzado
        """
        # Spawn task
        handle = self.spawn(future)
        
        start_time = time.time()
        max_idle_iterations = 1000  # Máximo de iteraciones sin progreso
        idle_count = 0
        
        # Run event loop hasta que complete o falle
        while handle.is_pending():
            # Check timeout
            if timeout and (time.time() - start_time) > timeout:
                handle.cancel()
                raise TimeoutError(f"Future did not complete within {timeout}s")
            
            if not self.step():
                # No hay tareas, esperar un poco
                idle_count += 1
                if idle_count > max_idle_iterations:
                    # Si no hay progreso después de muchas iteraciones, asumir que está colgado
                    if handle.is_pending():
                        raise RuntimeError("Future appears to be stuck (no progress after 1000 idle iterations)")
                time.sleep(0.001)
            else:
                idle_count = 0  # Reset counter
        
        # Retornar resultado (lanza excepción si falló)
        return handle.result()
    
    def run(self, max_iterations: Optional[int] = None) -> None:
        """
        Ejecuta event loop indefinidamente
        
        Args:
            max_iterations: Máximo de iteraciones (None = infinito)
        """
        self._running = True
        iterations = 0
        
        while self._running:
            if max_iterations and iterations >= max_iterations:
                break
            
            if not self.step():
                # No hay tareas, esperar
                time.sleep(0.001)
            
            iterations += 1
    
    def step(self) -> bool:
        """
        Ejecuta un paso del event loop
        
        Returns:
            True si procesó algún task, False si ready_queue vacío
        """
        # Pop task de ready queue
        task = self._pop_ready_task()
        if not task:
            return False
        
        # Poll task
        self._poll_task(task)
        return True
    
    def stop(self) -> None:
        """Detiene el event loop"""
        self._running = False
    
    def _pop_ready_task(self) -> Optional[Task]:
        """Pop task de ready queue"""
        with self._lock:
            if not self.ready_queue:
                return None
            return self.ready_queue.popleft()
    
    def _poll_task(self, task: Task) -> None:
        """
        Poll un task
        
        Args:
            task: Task a pollear
        """
        # Crear waker que mueve task de waiting a ready
        def on_wake():
            with self._lock:
                # Mover de waiting a ready
                if task.task_id in self.waiting:
                    del self.waiting[task.task_id]
                    self.ready_queue.append(task)
        
        waker = Waker(on_wake)
        
        try:
            # Poll task
            poll = task.poll(waker)
            
            if poll.is_pending():
                # Aún pendiente, mover a waiting
                with self._lock:
                    self.waiting[task.task_id] = task
                    self.wakers[task.task_id] = waker
            else:
                # Completado, limpiar waker
                with self._lock:
                    if task.task_id in self.wakers:
                        del self.wakers[task.task_id]
                        
        except Exception as e:
            # Error durante poll, marcar como failed
            # (ya manejado en Task.poll)
            with self._lock:
                if task.task_id in self.wakers:
                    del self.wakers[task.task_id]
    
    def active_tasks(self) -> int:
        """Número de tareas activas (ready + waiting)"""
        with self._lock:
            return len(self.ready_queue) + len(self.waiting)
    
    def waiting_tasks(self) -> int:
        """Número de tareas esperando"""
        with self._lock:
            return len(self.waiting)
    
    def ready_tasks(self) -> int:
        """Número de tareas listas"""
        with self._lock:
            return len(self.ready_queue)
    
    def __repr__(self) -> str:
        return f"Executor<ready={self.ready_tasks()}, waiting={self.waiting_tasks()}>"


def block_on(future: Future[T]) -> T:
    """
    Ejecuta un future hasta completar (blocking)
    
    Crea executor temporal y ejecuta hasta que future complete.
    
    Args:
        future: Future a ejecutar
        
    Returns:
        Resultado del future
        
    Ejemplo:
        result = block_on(Future.ready(42))
        assert result == 42
    """
    executor = Executor()
    return executor.run_until_complete(future)


@dataclass
class Runtime:
    """
    Runtime global para async/await
    
    Singleton que maneja el executor principal.
    
    Ejemplo:
        runtime = Runtime.get()
        handle = runtime.spawn(future)
        result = runtime.block_on(future)
    """
    
    executor: Executor = field(default_factory=Executor)


# Singleton pattern (variables globales)
_runtime_instance: Optional[Runtime] = None
_runtime_lock = Lock()


def get_runtime() -> Runtime:
    """Obtiene instancia singleton del runtime"""
    global _runtime_instance
    if _runtime_instance is None:
        with _runtime_lock:
            if _runtime_instance is None:
                _runtime_instance = Runtime()
    return _runtime_instance


# Attach como método de clase
Runtime.get = staticmethod(get_runtime)


# Métodos de instancia de Runtime
def _runtime_spawn(self, future: Future[T]) -> TaskHandle[T]:
    """Spawn task en executor"""
    return self.executor.spawn(future)


def _runtime_block_on(self, future: Future[T]) -> T:
    """Ejecuta future hasta completar (blocking)"""
    return self.executor.run_until_complete(future)


def _runtime_run(self, max_iterations: Optional[int] = None) -> None:
    """Ejecuta event loop"""
    self.executor.run(max_iterations)


def _runtime_stop(self) -> None:
    """Detiene event loop"""
    self.executor.stop()


def _runtime_active_tasks(self) -> int:
    """Número de tareas activas"""
    return self.executor.active_tasks()


def _runtime_repr(self) -> str:
    return f"Runtime<{self.executor}>"


Runtime.spawn = _runtime_spawn
Runtime.block_on = _runtime_block_on
Runtime.run = _runtime_run
Runtime.stop = _runtime_stop
Runtime.active_tasks = _runtime_active_tasks
Runtime.__repr__ = _runtime_repr
