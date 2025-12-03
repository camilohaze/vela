"""
Future<T> - Valor Futuro

Implementación de: VELA-580 (TASK-047)
Sprint 18 - Async/Await

Future<T> representa un valor que estará disponible en el futuro.
Es el tipo fundamental para async/await en Vela.

Características:
- Lazy evaluation (solo computa cuando se pollea)
- Composición funcional (map, flatMap, then, catch)
- Combinadores (all, race, timeout)
- Zero-cost abstractions (sin overhead innecesario)

Inspirado en:
- Rust: std::future::Future trait
- JavaScript: Promise API
- Scala: Future
"""

from abc import ABC, abstractmethod
from typing import TypeVar, Generic, Callable, List, Optional, Union
from .poll import Poll
from .waker import Waker
import time


T = TypeVar('T')
U = TypeVar('U')
E = TypeVar('E')


class Future(ABC, Generic[T]):
    """
    Trait abstracto para Future<T>.
    
    Los Futures en Vela son lazy: solo se computan cuando se pollea.
    Un Future puede ser polleado múltiples veces hasta que retorne Poll.ready().
    
    Ejemplos:
    ```vela
    async fn fetchData() -> Future<String> {
      response = await httpClient.get("https://api.example.com")
      return response.body
    }
    
    # Composición
    userFuture = fetchData()
      .map(data => parseJson(data))
      .flatMap(json => saveToDb(json))
      .catch(error => handleError(error))
    ```
    """
    
    @abstractmethod
    def poll(self, waker: Waker) -> Poll[T]:
        """
        Pollead el Future para ver si está listo.
        
        Este método DEBE ser implementado por subclases.
        
        Args:
            waker: Waker para despertar cuando esté listo
            
        Returns:
            Poll.ready(value) si completado
            Poll.pending() si aún en progreso
        """
        pass
    
    def map(self, f: Callable[[T], U]) -> 'Future[U]':
        """
        Transforma el valor del Future cuando esté listo.
        
        Args:
            f: Función (T) -> U
            
        Returns:
            Future[U] transformado
            
        Ejemplos:
        ```python
        future = ReadyFuture(42)
        mapped = future.map(lambda x: x * 2)
        # mapped es Future[84]
        ```
        """
        return MapFuture(self, f)
    
    def flat_map(self, f: Callable[[T], 'Future[U]']) -> 'Future[U]':
        """
        Transforma y aplana el Future (monadic bind).
        
        Args:
            f: Función (T) -> Future[U]
            
        Returns:
            Future[U] aplanado
            
        Ejemplos:
        ```python
        future = ReadyFuture(42)
        flat_mapped = future.flat_map(lambda x: ReadyFuture(x * 2))
        ```
        """
        return FlatMapFuture(self, f)
    
    def then(self, callback: Callable[[T], U]) -> 'Future[U]':
        """
        Alias de map() (compatibilidad con JavaScript Promise).
        
        Args:
            callback: Función de transformación
            
        Returns:
            Future[U] transformado
        """
        return self.map(callback)
    
    def catch(self, callback: Callable[[Exception], T]) -> 'Future[T]':
        """
        Maneja errores en el Future.
        
        Args:
            callback: Función (Exception) -> T
            
        Returns:
            Future[T] con manejo de errores
        """
        return CatchFuture(self, callback)
    
    def and_then(self, other: 'Future[U]') -> 'Future[tuple[T, U]]':
        """
        Combina dos Futures en paralelo.
        
        Args:
            other: Otro Future[U]
            
        Returns:
            Future[(T, U)] con ambos resultados
        """
        return AndFuture(self, other)
    
    def or_else(self, other: 'Future[T]') -> 'Future[T]':
        """
        Retorna el primer Future que complete (race).
        
        Args:
            other: Future alternativo
            
        Returns:
            Future[T] del primero que complete
        """
        return OrFuture(self, other)
    
    @staticmethod
    def ready(value: T) -> 'Future[T]':
        """
        Crea un Future ya completado con un valor.
        
        Args:
            value: Valor del Future
            
        Returns:
            Future[T] inmediatamente listo
        """
        return ReadyFuture(value)
    
    @staticmethod
    def pending() -> 'Future[T]':
        """
        Crea un Future que nunca completa (útil para testing).
        
        Returns:
            Future[T] perpetuamente pendiente
        """
        return PendingFuture()
    
    @staticmethod
    def from_result(result: Union[T, Exception]) -> 'Future[T]':
        """
        Crea un Future desde un resultado o excepción.
        
        Args:
            result: Valor o Exception
            
        Returns:
            Future[T] listo o con error
        """
        if isinstance(result, Exception):
            return ErrorFuture(result)
        return ReadyFuture(result)
    
    @staticmethod
    def all(futures: List['Future']) -> 'Future[List]':
        """
        Espera a que todos los Futures completen.
        
        Args:
            futures: Lista de Futures
            
        Returns:
            Future[List] con todos los resultados
        """
        return AllFuture(futures)
    
    @staticmethod
    def race(futures: List['Future[T]']) -> 'Future[T]':
        """
        Retorna el primer Future que complete.
        
        Args:
            futures: Lista de Futures
            
        Returns:
            Future[T] del primero que complete
        """
        return RaceFuture(futures)
    
    @staticmethod
    def timeout(future: 'Future[T]', seconds: float) -> 'Future[T]':
        """
        Aplica un timeout a un Future.
        
        Args:
            future: Future a limitar
            seconds: Timeout en segundos
            
        Returns:
            Future[T] que falla si excede timeout
        """
        return TimeoutFuture(future, seconds)
    
    def __repr__(self) -> str:
        return f"Future<{type(self).__name__}>"


# ===================================================================
# IMPLEMENTACIONES CONCRETAS
# ===================================================================


class ReadyFuture(Future[T]):
    """Future ya completado con un valor"""
    
    def __init__(self, value: T):
        self.value = value
    
    def poll(self, waker: Waker) -> Poll[T]:
        return Poll.ready(self.value)
    
    def __repr__(self) -> str:
        return f"ReadyFuture({self.value})"


class PendingFuture(Future[T]):
    """Future que nunca completa"""
    
    def poll(self, waker: Waker) -> Poll[T]:
        return Poll.pending()
    
    def __repr__(self) -> str:
        return "PendingFuture()"


class ErrorFuture(Future[T]):
    """Future que falla con una excepción"""
    
    def __init__(self, error: Exception):
        self.error = error
    
    def poll(self, waker: Waker) -> Poll[T]:
        # En una implementación real, esto lanzaría la excepción
        # Por ahora, retornamos Poll.pending() (nunca completa)
        return Poll.pending()
    
    def __repr__(self) -> str:
        return f"ErrorFuture({self.error})"


class MapFuture(Future[U]):
    """Future que transforma el resultado de otro Future"""
    
    def __init__(self, inner: Future[T], f: Callable[[T], U]):
        self.inner = inner
        self.f = f
    
    def poll(self, waker: Waker) -> Poll[U]:
        inner_poll = self.inner.poll(waker)
        if inner_poll.is_ready():
            transformed = self.f(inner_poll.value)
            return Poll.ready(transformed)
        return Poll.pending()


class FlatMapFuture(Future[U]):
    """Future que aplana otro Future anidado"""
    
    def __init__(self, inner: Future[T], f: Callable[[T], Future[U]]):
        self.inner = inner
        self.f = f
        self.outer_future: Optional[Future[U]] = None
    
    def poll(self, waker: Waker) -> Poll[U]:
        # Primero, pollead el Future interno
        if self.outer_future is None:
            inner_poll = self.inner.poll(waker)
            if inner_poll.is_pending():
                return Poll.pending()
            # Inner está listo, crear outer Future
            self.outer_future = self.f(inner_poll.value)
        
        # Pollead el outer Future
        return self.outer_future.poll(waker)


class CatchFuture(Future[T]):
    """Future con manejo de errores"""
    
    def __init__(self, inner: Future[T], callback: Callable[[Exception], T]):
        self.inner = inner
        self.callback = callback
    
    def poll(self, waker: Waker) -> Poll[T]:
        try:
            return self.inner.poll(waker)
        except Exception as e:
            recovered = self.callback(e)
            return Poll.ready(recovered)


class AndFuture(Future[tuple[T, U]]):
    """Combina dos Futures en paralelo"""
    
    def __init__(self, future1: Future[T], future2: Future[U]):
        self.future1 = future1
        self.future2 = future2
    
    def poll(self, waker: Waker) -> Poll[tuple[T, U]]:
        poll1 = self.future1.poll(waker)
        poll2 = self.future2.poll(waker)
        
        if poll1.is_ready() and poll2.is_ready():
            return Poll.ready((poll1.value, poll2.value))
        return Poll.pending()


class OrFuture(Future[T]):
    """Retorna el primer Future que complete"""
    
    def __init__(self, future1: Future[T], future2: Future[T]):
        self.future1 = future1
        self.future2 = future2
    
    def poll(self, waker: Waker) -> Poll[T]:
        poll1 = self.future1.poll(waker)
        if poll1.is_ready():
            return poll1
        
        poll2 = self.future2.poll(waker)
        if poll2.is_ready():
            return poll2
        
        return Poll.pending()


class AllFuture(Future[List]):
    """Espera a que todos los Futures completen"""
    
    def __init__(self, futures: List[Future]):
        self.futures = futures
    
    def poll(self, waker: Waker) -> Poll[List]:
        results = []
        all_ready = True
        
        for future in self.futures:
            poll = future.poll(waker)
            if poll.is_pending():
                all_ready = False
                break
            results.append(poll.value)
        
        if all_ready:
            return Poll.ready(results)
        return Poll.pending()


class RaceFuture(Future[T]):
    """Retorna el primer Future que complete"""
    
    def __init__(self, futures: List[Future[T]]):
        self.futures = futures
    
    def poll(self, waker: Waker) -> Poll[T]:
        for future in self.futures:
            poll = future.poll(waker)
            if poll.is_ready():
                return poll
        return Poll.pending()


class TimeoutFuture(Future[T]):
    """Future con timeout"""
    
    def __init__(self, inner: Future[T], seconds: float):
        self.inner = inner
        self.deadline = time.time() + seconds
    
    def poll(self, waker: Waker) -> Poll[T]:
        if time.time() > self.deadline:
            raise TimeoutError(f"Future timed out")
        return self.inner.poll(waker)
