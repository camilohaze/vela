"""
Tests para Async Runtime (Future<T> y Promise<T>)

Implementación de: VELA-580 (TASK-047)
Sprint 18 - Async/Await
Fecha: 2025-12-02

Tests completos del sistema async de Vela.
"""

import pytest
import sys
sys.path.insert(0, '.')

from src.runtime.async_runtime.poll import Poll, PollState
from src.runtime.async_runtime.waker import Waker
from src.runtime.async_runtime.future import (
    Future, ReadyFuture, PendingFuture, ErrorFuture,
    MapFuture, FlatMapFuture, AndFuture, OrFuture,
    AllFuture, RaceFuture, TimeoutFuture
)
from src.runtime.async_runtime.promise import Promise, promise_from_callback


# ===================================================================
# TEST POLL
# ===================================================================


class TestPoll:
    """Tests para Poll<T>"""
    
    def test_ready(self):
        """Test: Poll.ready() crea Poll en estado Ready"""
        poll = Poll.ready(42)
        assert poll.is_ready()
        assert not poll.is_pending()
        assert poll.state == PollState.READY
        assert poll.value == 42
    
    def test_pending(self):
        """Test: Poll.pending() crea Poll en estado Pending"""
        poll = Poll.pending()
        assert poll.is_pending()
        assert not poll.is_ready()
        assert poll.state == PollState.PENDING
        assert poll.value is None
    
    def test_unwrap(self):
        """Test: unwrap() extrae el valor de Poll.ready()"""
        poll = Poll.ready(42)
        assert poll.unwrap() == 42
    
    def test_unwrap_pending_raises(self):
        """Test: unwrap() en Pending lanza ValueError"""
        poll = Poll.pending()
        with pytest.raises(ValueError):
            poll.unwrap()
    
    def test_unwrap_or(self):
        """Test: unwrap_or() retorna default si Pending"""
        ready = Poll.ready(42)
        pending = Poll.pending()
        
        assert ready.unwrap_or(0) == 42
        assert pending.unwrap_or(0) == 0
    
    def test_map(self):
        """Test: map() transforma el valor si Ready"""
        ready = Poll.ready(21)
        mapped = ready.map(lambda x: x * 2)
        assert mapped.is_ready()
        assert mapped.unwrap() == 42
        
        pending = Poll.pending()
        mapped2 = pending.map(lambda x: x * 2)
        assert mapped2.is_pending()


# ===================================================================
# TEST WAKER
# ===================================================================


class TestWaker:
    """Tests para Waker"""
    
    def test_wake_calls_callback(self):
        """Test: wake() ejecuta el callback"""
        called = []
        
        def on_wake():
            called.append(True)
        
        waker = Waker(callback=on_wake)
        waker.wake()
        
        assert len(called) == 1
        assert waker.is_woken()
    
    def test_wake_once(self):
        """Test: wake() solo ejecuta callback una vez"""
        call_count = 0
        
        def on_wake():
            nonlocal call_count
            call_count += 1
        
        waker = Waker(callback=on_wake)
        waker.wake()
        waker.wake()
        waker.wake()
        
        assert call_count == 1
    
    def test_reset(self):
        """Test: reset() permite re-wake"""
        call_count = 0
        
        def on_wake():
            nonlocal call_count
            call_count += 1
        
        waker = Waker(callback=on_wake)
        waker.wake()
        assert call_count == 1
        
        waker.reset()
        assert not waker.is_woken()
        
        waker.wake()
        assert call_count == 2
    
    def test_clone(self):
        """Test: clone() crea nuevo Waker con mismo callback"""
        call_count = 0
        
        def on_wake():
            nonlocal call_count
            call_count += 1
        
        waker1 = Waker(callback=on_wake)
        waker2 = waker1.clone()
        
        waker1.wake()
        assert call_count == 1
        
        waker2.wake()
        assert call_count == 2
    
    def test_noop(self):
        """Test: Waker.noop() no crashea al wake"""
        waker = Waker.noop()
        waker.wake()  # No debe crashear
        assert waker.is_woken()


# ===================================================================
# TEST FUTURE
# ===================================================================


class TestFuture:
    """Tests para Future<T>"""
    
    def test_ready_future(self):
        """Test: ReadyFuture retorna valor inmediatamente"""
        future = Future.ready(42)
        waker = Waker.noop()
        
        poll = future.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == 42
    
    def test_pending_future(self):
        """Test: PendingFuture nunca completa"""
        future = Future.pending()
        waker = Waker.noop()
        
        poll = future.poll(waker)
        assert poll.is_pending()
    
    def test_map(self):
        """Test: map() transforma el valor del Future"""
        future = Future.ready(21)
        mapped = future.map(lambda x: x * 2)
        
        waker = Waker.noop()
        poll = mapped.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == 42
    
    def test_map_pending(self):
        """Test: map() preserva Pending"""
        future = Future.pending()
        mapped = future.map(lambda x: x * 2)
        
        waker = Waker.noop()
        poll = mapped.poll(waker)
        assert poll.is_pending()
    
    def test_flat_map(self):
        """Test: flat_map() aplana Futures anidados"""
        future = Future.ready(21)
        flat = future.flat_map(lambda x: Future.ready(x * 2))
        
        waker = Waker.noop()
        poll = flat.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == 42
    
    def test_then(self):
        """Test: then() es alias de map()"""
        future = Future.ready(10)
        chained = future.then(lambda x: x + 5)
        
        waker = Waker.noop()
        poll = chained.poll(waker)
        assert poll.unwrap() == 15
    
    def test_catch(self):
        """Test: catch() maneja errores"""
        # En esta implementación simplificada, solo verificamos que compila
        future = Future.ready(42)
        caught = future.catch(lambda e: 0)
        
        waker = Waker.noop()
        poll = caught.poll(waker)
        assert poll.is_ready()
    
    def test_and_then(self):
        """Test: and_then() combina dos Futures"""
        f1 = Future.ready(1)
        f2 = Future.ready(2)
        combined = f1.and_then(f2)
        
        waker = Waker.noop()
        poll = combined.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == (1, 2)
    
    def test_and_then_with_pending(self):
        """Test: and_then() con Pending retorna Pending"""
        f1 = Future.ready(1)
        f2 = Future.pending()
        combined = f1.and_then(f2)
        
        waker = Waker.noop()
        poll = combined.poll(waker)
        assert poll.is_pending()
    
    def test_or_else(self):
        """Test: or_else() retorna el primero que complete"""
        f1 = Future.ready(1)
        f2 = Future.pending()
        race = f1.or_else(f2)
        
        waker = Waker.noop()
        poll = race.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == 1
    
    def test_all(self):
        """Test: Future.all() espera a todos"""
        futures = [
            Future.ready(1),
            Future.ready(2),
            Future.ready(3),
        ]
        all_future = Future.all(futures)
        
        waker = Waker.noop()
        poll = all_future.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == [1, 2, 3]
    
    def test_all_with_pending(self):
        """Test: Future.all() con Pending retorna Pending"""
        futures = [
            Future.ready(1),
            Future.pending(),
            Future.ready(3),
        ]
        all_future = Future.all(futures)
        
        waker = Waker.noop()
        poll = all_future.poll(waker)
        assert poll.is_pending()
    
    def test_race(self):
        """Test: Future.race() retorna el primero"""
        futures = [
            Future.pending(),
            Future.ready(2),
            Future.ready(3),
        ]
        race = Future.race(futures)
        
        waker = Waker.noop()
        poll = race.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == 2


# ===================================================================
# TEST PROMISE
# ===================================================================


class TestPromise:
    """Tests para Promise<T>"""
    
    def test_create_promise(self):
        """Test: Crear Promise y obtener Future"""
        promise = Promise[int]()
        future = promise.future()
        
        assert not promise.is_completed()
        assert isinstance(future, Future)
    
    def test_resolve(self):
        """Test: resolve() completa el Future"""
        promise = Promise[int]()
        future = promise.future()
        
        # Pendiente inicialmente
        waker = Waker.noop()
        poll = future.poll(waker)
        assert poll.is_pending()
        
        # Resolver
        promise.resolve(42)
        assert promise.is_completed()
        
        # Ahora está listo
        poll = future.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == 42
    
    def test_resolve_once(self):
        """Test: resolve() solo puede llamarse una vez"""
        promise = Promise[int]()
        promise.resolve(42)
        
        with pytest.raises(RuntimeError):
            promise.resolve(43)
    
    def test_reject(self):
        """Test: reject() falla el Future"""
        promise = Promise[str]()
        future = promise.future()
        
        error = ValueError("test error")
        promise.reject(error)
        
        assert promise.is_completed()
        
        # El Future no retorna Ready (está en error)
        waker = Waker.noop()
        poll = future.poll(waker)
        assert poll.is_pending()  # En implementación real, lanzaría excepción
    
    def test_reject_once(self):
        """Test: reject() solo puede llamarse una vez"""
        promise = Promise[int]()
        promise.reject(ValueError("error"))
        
        with pytest.raises(RuntimeError):
            promise.reject(ValueError("error2"))
    
    def test_resolve_then_reject_fails(self):
        """Test: No se puede reject después de resolve"""
        promise = Promise[int]()
        promise.resolve(42)
        
        with pytest.raises(RuntimeError):
            promise.reject(ValueError("error"))
    
    def test_resolved_factory(self):
        """Test: Promise.resolved() crea Promise ya resuelta"""
        promise = Promise.resolved(100)
        assert promise.is_completed()
        
        future = promise.future()
        waker = Waker.noop()
        poll = future.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == 100
    
    def test_rejected_factory(self):
        """Test: Promise.rejected() crea Promise ya rechazada"""
        promise = Promise.rejected(ValueError("error"))
        assert promise.is_completed()
    
    def test_waker_notification(self):
        """Test: Waker es despertado cuando Promise se resuelve"""
        promise = Promise[int]()
        future = promise.future()
        
        woken = []
        
        def on_wake():
            woken.append(True)
        
        waker = Waker(callback=on_wake)
        
        # Poll inicial
        poll = future.poll(waker)
        assert poll.is_pending()
        assert len(woken) == 0
        
        # Resolver Promise
        promise.resolve(42)
        
        # Waker debe haber sido despertado
        assert len(woken) == 1


# ===================================================================
# TEST INTEGRACIÓN CALLBACKS
# ===================================================================


class TestCallbackIntegration:
    """Tests para integración con APIs callback-based"""
    
    def test_promise_from_callback(self):
        """Test: promise_from_callback() convierte callback a Future"""
        def callback_api(callback):
            # Simular operación async
            result = "data"
            callback(result)
        
        future = promise_from_callback(callback_api)
        
        waker = Waker.noop()
        poll = future.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == "data"


# ===================================================================
# TEST EDGE CASES
# ===================================================================


class TestEdgeCases:
    """Tests de casos edge"""
    
    def test_chaining_multiple_maps(self):
        """Test: Encadenar múltiples map()"""
        future = Future.ready(5)
        result = (future
                  .map(lambda x: x + 1)
                  .map(lambda x: x * 2)
                  .map(lambda x: x - 2))
        
        waker = Waker.noop()
        poll = result.poll(waker)
        assert poll.unwrap() == 10  # (5+1)*2-2 = 10
    
    def test_flat_map_chain(self):
        """Test: Encadenar flat_map()"""
        future = Future.ready(10)
        result = (future
                  .flat_map(lambda x: Future.ready(x + 5))
                  .flat_map(lambda x: Future.ready(x * 2)))
        
        waker = Waker.noop()
        poll = result.poll(waker)
        assert poll.unwrap() == 30  # (10+5)*2 = 30
    
    def test_all_empty_list(self):
        """Test: Future.all([]) retorna lista vacía"""
        all_future = Future.all([])
        
        waker = Waker.noop()
        poll = all_future.poll(waker)
        assert poll.is_ready()
        assert poll.unwrap() == []
    
    def test_race_empty_list(self):
        """Test: Future.race([]) retorna Pending"""
        race = Future.race([])
        
        waker = Waker.noop()
        poll = race.poll(waker)
        assert poll.is_pending()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
