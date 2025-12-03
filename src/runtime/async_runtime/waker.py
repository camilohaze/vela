"""
Waker - Mecanismo de Wake-up

Implementación de: VELA-580 (TASK-047)
Sprint 18 - Async/Await

El Waker es usado para despertar (wake) una tarea cuando un Future está listo.
Permite implementar polling eficiente sin busy-waiting.

Inspirado en:
- Rust: std::task::Waker
- Java: LockSupport.park/unpark
"""

from typing import Callable, Optional
from dataclasses import dataclass
import threading


@dataclass
class Waker:
    """
    Waker para despertar tareas asíncronas.
    
    Un Waker es pasado al método poll() de un Future. Cuando el Future
    está listo, llama a waker.wake() para notificar al executor que debe
    volver a pollear el Future.
    
    Ejemplos:
    ```python
    # Crear waker con callback
    def on_wake():
        print("Task is ready!")
    
    waker = Waker(callback=on_wake)
    
    # Despertar la tarea
    waker.wake()
    ```
    """
    
    callback: Optional[Callable[[], None]] = None
    _woken: bool = False
    _lock: threading.Lock = None
    
    def __post_init__(self):
        """Inicializar lock después de dataclass init"""
        if self._lock is None:
            object.__setattr__(self, '_lock', threading.Lock())
    
    def wake(self) -> None:
        """
        Despertar la tarea asociada.
        
        Este método puede ser llamado múltiples veces, pero el callback
        solo se ejecuta una vez hasta el próximo reset.
        """
        with self._lock:
            if not self._woken:
                object.__setattr__(self, '_woken', True)
                if self.callback:
                    self.callback()
    
    def wake_by_ref(self) -> None:
        """
        Alias para wake() (compatibilidad con Rust).
        
        En Rust, wake_by_ref() toma &Waker en lugar de Waker.
        En Python no hay diferencia.
        """
        self.wake()
    
    def is_woken(self) -> bool:
        """
        Verifica si el Waker ha sido despertado.
        
        Returns:
            True si wake() fue llamado
        """
        with self._lock:
            return self._woken
    
    def reset(self) -> None:
        """
        Resetea el estado del Waker para reutilizarlo.
        """
        with self._lock:
            object.__setattr__(self, '_woken', False)
    
    def clone(self) -> 'Waker':
        """
        Clona el Waker (mismo callback).
        
        Returns:
            Nuevo Waker con el mismo callback
        """
        return Waker(callback=self.callback)
    
    @staticmethod
    def noop() -> 'Waker':
        """
        Crea un Waker no-op (sin callback).
        
        Útil para testing o cuando no se necesita notificación.
        
        Returns:
            Waker sin callback
        """
        return Waker(callback=None)
    
    def __repr__(self) -> str:
        callback_name = self.callback.__name__ if self.callback else "None"
        return f"Waker(callback={callback_name}, woken={self._woken})"
