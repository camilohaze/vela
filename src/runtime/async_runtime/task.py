"""
Task - Unidad de trabajo asíncrono

Implementación de:
- Task<T>: Wrapper de Future con estado y lifecycle
- TaskState: Estados del task (Pending, Running, Completed, Failed, Cancelled)
- TaskId: Identificador único de task

Jira: VELA-580
Historia: TASK-048
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import TypeVar, Generic, Optional, Any
from threading import Lock
import uuid

from .future import Future
from .poll import Poll
from .waker import Waker


T = TypeVar('T')


class TaskState(Enum):
    """Estados posibles de un Task"""
    PENDING = auto()     # No iniciado
    RUNNING = auto()     # En ejecución
    COMPLETED = auto()   # Completado exitosamente
    FAILED = auto()      # Falló con error
    CANCELLED = auto()   # Cancelado


@dataclass
class TaskId:
    """Identificador único de task"""
    value: str = field(default_factory=lambda: str(uuid.uuid4()))
    
    def __hash__(self) -> int:
        return hash(self.value)
    
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, TaskId):
            return False
        return self.value == other.value
    
    def __repr__(self) -> str:
        return f"TaskId({self.value[:8]}...)"


@dataclass
class Task(Generic[T]):
    """
    Task<T> - Unidad de trabajo asíncrono
    
    Wrapper de Future<T> con estado, lifecycle y manejo de errores.
    
    Estados:
    - PENDING: No iniciado
    - RUNNING: En ejecución (polleando)
    - COMPLETED: Completado con valor
    - FAILED: Falló con error
    - CANCELLED: Cancelado por usuario
    
    Ejemplo:
        future = Future.ready(42)
        task = Task(future)
        
        waker = Waker.noop()
        poll = task.poll(waker)
        
        if poll.is_ready():
            result = task.result()
    """
    
    future: Future[T]
    task_id: TaskId = field(default_factory=TaskId)
    state: TaskState = TaskState.PENDING
    _result: Optional[T] = None
    _error: Optional[Exception] = None
    _lock: Lock = field(default_factory=Lock)
    
    def poll(self, waker: Waker) -> Poll[T]:
        """
        Poll el future subyacente
        
        Args:
            waker: Waker para notificar cuando esté listo
            
        Returns:
            Poll<T> con resultado o Pending
            
        Raises:
            RuntimeError: Si task está cancelado
        """
        with self._lock:
            if self.state == TaskState.CANCELLED:
                raise RuntimeError(f"Task {self.task_id} was cancelled")
            
            if self.state == TaskState.COMPLETED:
                return Poll.ready(self._result)
            
            if self.state == TaskState.FAILED:
                raise self._error
            
            # Marcar como running
            self.state = TaskState.RUNNING
        
        try:
            # Poll el future
            poll = self.future.poll(waker)
            
            with self._lock:
                if poll.is_ready():
                    # Completado exitosamente
                    self._result = poll.unwrap()
                    self.state = TaskState.COMPLETED
                    return Poll.ready(self._result)
                else:
                    # Aún pendiente
                    self.state = TaskState.PENDING
                    return Poll.pending()
                    
        except Exception as e:
            # Error durante poll
            with self._lock:
                self._error = e
                self.state = TaskState.FAILED
            raise
    
    def is_completed(self) -> bool:
        """Verifica si task está completado"""
        with self._lock:
            return self.state == TaskState.COMPLETED
    
    def is_failed(self) -> bool:
        """Verifica si task falló"""
        with self._lock:
            return self.state == TaskState.FAILED
    
    def is_cancelled(self) -> bool:
        """Verifica si task fue cancelado"""
        with self._lock:
            return self.state == TaskState.CANCELLED
    
    def is_pending(self) -> bool:
        """Verifica si task está pendiente"""
        with self._lock:
            return self.state in (TaskState.PENDING, TaskState.RUNNING)
    
    def result(self) -> T:
        """
        Obtiene el resultado del task
        
        Returns:
            Valor T si completado
            
        Raises:
            RuntimeError: Si no está completado o fue cancelado
            Exception: Si falló con error
        """
        with self._lock:
            if self.state == TaskState.COMPLETED:
                return self._result
            elif self.state == TaskState.FAILED:
                raise self._error
            elif self.state == TaskState.CANCELLED:
                raise RuntimeError(f"Task {self.task_id} was cancelled")
            else:
                raise RuntimeError(f"Task {self.task_id} is not completed yet")
    
    def error(self) -> Optional[Exception]:
        """Obtiene el error si falló"""
        with self._lock:
            return self._error
    
    def cancel(self) -> bool:
        """
        Cancela el task
        
        Returns:
            True si fue cancelado, False si ya estaba completado/failed
        """
        with self._lock:
            if self.state in (TaskState.COMPLETED, TaskState.FAILED):
                return False  # Ya terminó, no se puede cancelar
            
            self.state = TaskState.CANCELLED
            return True
    
    def __repr__(self) -> str:
        return f"Task<{self.task_id}, {self.state.name}>"


@dataclass
class TaskHandle(Generic[T]):
    """
    TaskHandle<T> - Handle para controlar y consultar un Task
    
    Permite:
    - Consultar estado del task
    - Obtener resultado cuando complete
    - Cancelar task
    
    Ejemplo:
        handle = executor.spawn(future)
        
        # Consultar estado
        if handle.is_completed():
            result = handle.result()
        
        # Cancelar
        handle.cancel()
    """
    
    task: Task[T]
    
    def task_id(self) -> TaskId:
        """Obtiene el TaskId"""
        return self.task.task_id
    
    def is_completed(self) -> bool:
        """Verifica si task completó"""
        return self.task.is_completed()
    
    def is_failed(self) -> bool:
        """Verifica si task falló"""
        return self.task.is_failed()
    
    def is_cancelled(self) -> bool:
        """Verifica si task fue cancelado"""
        return self.task.is_cancelled()
    
    def is_pending(self) -> bool:
        """Verifica si task está pendiente"""
        return self.task.is_pending()
    
    def result(self) -> T:
        """
        Obtiene el resultado (blocking)
        
        Returns:
            Valor T si completado
            
        Raises:
            RuntimeError: Si no completado o cancelado
            Exception: Si falló
        """
        return self.task.result()
    
    def error(self) -> Optional[Exception]:
        """Obtiene el error si falló"""
        return self.task.error()
    
    def cancel(self) -> bool:
        """
        Cancela el task
        
        Returns:
            True si cancelado, False si ya terminó
        """
        return self.task.cancel()
    
    def __repr__(self) -> str:
        return f"TaskHandle<{self.task.task_id}, {self.task.state.name}>"
