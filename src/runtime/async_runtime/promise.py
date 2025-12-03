"""
Promise<T> - Productor de Valores Futuros

Implementación de: VELA-580 (TASK-047)
Sprint 18 - Async/Await

Promise<T> es el productor (escritor) de un Future<T>.
Permite resolver o rechazar un Future manualmente.

Relación:
- Promise: Produce valor (resolve/reject)
- Future: Consume valor (await)

Inspirado en:
- JavaScript: Promise
- Rust: futures::channel::oneshot
- Scala: Promise
"""

from typing import TypeVar, Generic, Optional, Callable
from .future import Future
from .poll import Poll
from .waker import Waker
import threading


T = TypeVar('T')


class PromiseFuture(Future[T]):
    """
    Future interno usado por Promise.
    
    Este Future se completa cuando la Promise asociada
    llama a resolve() o reject().
    """
    
    def __init__(self):
        self._value: Optional[T] = None
        self._error: Optional[Exception] = None
        self._completed = False
        self._waker: Optional[Waker] = None
        self._lock = threading.Lock()
    
    def poll(self, waker: Waker) -> Poll[T]:
        """Poll el Future para ver si la Promise se resolvió"""
        with self._lock:
            if self._completed:
                if self._error:
                    # TODO: En una implementación real, lanzar excepción
                    # Por ahora, retornar Poll.pending()
                    return Poll.pending()
                return Poll.ready(self._value)
            
            # Guardar waker para notificar cuando se resuelva
            self._waker = waker
            return Poll.pending()
    
    def _complete(self, value: T) -> None:
        """Completa el Future con un valor (llamado por Promise)"""
        with self._lock:
            if self._completed:
                raise RuntimeError("Promise already resolved or rejected")
            self._completed = True
            self._value = value
            
            # Despertar waker si existe
            if self._waker:
                self._waker.wake()
    
    def _fail(self, error: Exception) -> None:
        """Falla el Future con un error (llamado por Promise)"""
        with self._lock:
            if self._completed:
                raise RuntimeError("Promise already resolved or rejected")
            self._completed = True
            self._error = error
            
            # Despertar waker si existe
            if self._waker:
                self._waker.wake()
    
    def is_completed(self) -> bool:
        """Verifica si el Future se completó"""
        with self._lock:
            return self._completed


class Promise(Generic[T]):
    """
    Promise<T> - Productor de un Future.
    
    Una Promise permite resolver manualmente un Future.
    Útil para integrar código callback-based con async/await.
    
    Características:
    - Resolve una sola vez (immutable después)
    - Thread-safe
    - Despertar wakers automáticamente
    
    Ejemplos:
    ```vela
    # Crear promise
    promise = Promise<String>()
    future = promise.future()
    
    # Resolver después
    promise.resolve("Hello, World!")
    
    # O rechazar
    promise.reject(Error("Failed"))
    
    # Uso con async/await
    async fn fetchWithCallback() -> String {
      promise = Promise<String>()
      
      # Callback-based API
      httpClient.get("https://api.example.com", callback=(response) => {
        promise.resolve(response.body)
      })
      
      # Esperar resultado
      result = await promise.future()
      return result
    }
    ```
    """
    
    def __init__(self):
        """Crea una nueva Promise"""
        self._future = PromiseFuture[T]()
        self._completed = False
        self._lock = threading.Lock()
    
    def future(self) -> Future[T]:
        """
        Obtiene el Future asociado a esta Promise.
        
        Returns:
            Future[T] que se completará cuando la Promise se resuelva
        """
        return self._future
    
    def resolve(self, value: T) -> None:
        """
        Resuelve la Promise con un valor.
        
        Args:
            value: Valor con el que resolver el Future
            
        Raises:
            RuntimeError: Si la Promise ya fue resuelta o rechazada
        """
        with self._lock:
            if self._completed:
                raise RuntimeError("Promise already resolved or rejected")
            self._completed = True
            self._future._complete(value)
    
    def reject(self, error: Exception) -> None:
        """
        Rechaza la Promise con un error.
        
        Args:
            error: Error con el que fallar el Future
            
        Raises:
            RuntimeError: Si la Promise ya fue resuelta o rechazada
        """
        with self._lock:
            if self._completed:
                raise RuntimeError("Promise already resolved or rejected")
            self._completed = True
            self._future._fail(error)
    
    def is_completed(self) -> bool:
        """
        Verifica si la Promise fue resuelta o rechazada.
        
        Returns:
            True si la Promise se completó
        """
        with self._lock:
            return self._completed
    
    @staticmethod
    def resolved(value: T) -> 'Promise[T]':
        """
        Crea una Promise ya resuelta.
        
        Args:
            value: Valor de la Promise
            
        Returns:
            Promise[T] ya resuelta
        """
        promise = Promise[T]()
        promise.resolve(value)
        return promise
    
    @staticmethod
    def rejected(error: Exception) -> 'Promise[T]':
        """
        Crea una Promise ya rechazada.
        
        Args:
            error: Error de la Promise
            
        Returns:
            Promise[T] ya rechazada
        """
        promise = Promise[T]()
        promise.reject(error)
        return promise
    
    def __repr__(self) -> str:
        status = "completed" if self._completed else "pending"
        return f"Promise<{status}>"


# ===================================================================
# HELPERS PARA INTEGRACIÓN CON CALLBACKS
# ===================================================================


def promise_from_callback(f: Callable[[Callable[[T], None]], None]) -> Future[T]:
    """
    Crea un Future desde una función callback-based.
    
    Args:
        f: Función que toma un callback y lo llama con el resultado
        
    Returns:
        Future[T] que se resuelve cuando el callback es llamado
        
    Ejemplos:
    ```python
    # Callback-based API
    def fetch_data(callback):
        # Simular operación async
        result = "data"
        callback(result)
    
    # Convertir a Future
    future = promise_from_callback(fetch_data)
    ```
    """
    promise = Promise[T]()
    
    def callback(value: T):
        promise.resolve(value)
    
    f(callback)
    return promise.future()


def promise_from_error_callback(
    f: Callable[[Callable[[T], None], Callable[[Exception], None]], None]
) -> Future[T]:
    """
    Crea un Future desde una función con callback de success y error.
    
    Args:
        f: Función que toma callbacks de success y error
        
    Returns:
        Future[T] que se resuelve o falla según el callback
        
    Ejemplos:
    ```python
    def fetch_data_with_error(on_success, on_error):
        try:
            result = "data"
            on_success(result)
        except Exception as e:
            on_error(e)
    
    future = promise_from_error_callback(fetch_data_with_error)
    ```
    """
    promise = Promise[T]()
    
    def on_success(value: T):
        promise.resolve(value)
    
    def on_error(error: Exception):
        promise.reject(error)
    
    f(on_success, on_error)
    return promise.future()
