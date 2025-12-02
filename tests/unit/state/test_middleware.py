"""
Tests unitarios para Middleware System

Jira: VELA-577 - TASK-035Y
Historia: State Management
"""

import pytest
import time
from dataclasses import dataclass
from typing import Any, List

from src.reactive.middleware import (
    Middleware,
    MiddlewareContext,
    LoggerMiddleware,
    AsyncMiddleware,
    ThrottleMiddleware,
    DebounceMiddleware,
    ErrorHandlerMiddleware,
    CacheMiddleware,
    compose_middleware,
    apply_middleware,
    create_middleware
)
from src.reactive.action import SimpleAction


# Mock Classes

@dataclass
class AppState:
    """Estado mock de la aplicación."""
    counter: int = 0
    logs: List[str] = None
    error: str = None
    
    def __post_init__(self):
        if self.logs is None:
            self.logs = []


class MockStore:
    """Mock de Store para testing."""
    
    def __init__(self, initial_state: AppState = None):
        self.state = initial_state or AppState()
        self.actions_received: List[Any] = []
    
    def get_state(self) -> AppState:
        """Obtiene estado actual."""
        return self.state
    
    def dispatch(self, action: Any) -> None:
        """Dispatch básico."""
        self.actions_received.append(action)
        
        # Simular reducer simple
        if hasattr(action, 'type'):
            if action.type == "INCREMENT":
                self.state = AppState(
                    counter=self.state.counter + 1,
                    logs=self.state.logs
                )
            elif action.type == "ADD_LOG":
                self.state = AppState(
                    counter=self.state.counter,
                    logs=self.state.logs + [action.payload]
                )
            elif action.type == "ERROR":
                self.state = AppState(
                    counter=self.state.counter,
                    logs=self.state.logs,
                    error=action.payload.get("error", "")
                )


# Tests de Middleware Base

class TestMiddlewareBase:
    """Tests para clase base Middleware."""
    
    def test_middleware_default_behavior(self):
        """Test que middleware por defecto pasa al siguiente."""
        middleware = Middleware()
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        called = []
        
        def next_fn(action):
            called.append(action)
        
        action = SimpleAction("TEST")
        middleware.handle(context, next_fn, action)
        
        assert len(called) == 1
        assert called[0] == action
    
    def test_middleware_callable(self):
        """Test que middleware es callable."""
        middleware = Middleware()
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        called = []
        def next_fn(action):
            called.append(action)
        
        action = SimpleAction("TEST")
        middleware(context, next_fn, action)
        
        assert len(called) == 1


# Tests de LoggerMiddleware

class TestLoggerMiddleware:
    """Tests para LoggerMiddleware."""
    
    def test_logger_logs_actions(self, capsys):
        """Test que logger registra acciones."""
        middleware = LoggerMiddleware(log_actions=True, log_state=False)
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        action = SimpleAction("INCREMENT")
        middleware.handle(context, lambda a: None, action)
        
        captured = capsys.readouterr()
        assert "action INCREMENT" in captured.out
    
    def test_logger_logs_state(self, capsys):
        """Test que logger registra cambios de estado."""
        middleware = LoggerMiddleware(log_actions=False, log_state=True)
        store = MockStore(AppState(counter=5))
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        action = SimpleAction("INCREMENT")
        
        def next_fn(a):
            store.state = AppState(counter=6)
        
        middleware.handle(context, next_fn, action)
        
        captured = capsys.readouterr()
        assert "prev state" in captured.out
        assert "next state" in captured.out


# Tests de AsyncMiddleware

class TestAsyncMiddleware:
    """Tests para AsyncMiddleware."""
    
    def test_async_middleware_handles_functions(self):
        """Test que async middleware maneja funciones."""
        middleware = AsyncMiddleware()
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        dispatched = []
        
        def thunk(dispatch, getState):
            state = getState()
            dispatch(SimpleAction("INCREMENT"))
            dispatched.append("executed")
        
        middleware.handle(context, lambda a: None, thunk)
        
        assert len(dispatched) == 1
        assert "executed" in dispatched
    
    def test_async_middleware_passes_regular_actions(self):
        """Test que async middleware pasa acciones normales."""
        middleware = AsyncMiddleware()
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        called = []
        
        def next_fn(action):
            called.append(action)
        
        action = SimpleAction("INCREMENT")
        middleware.handle(context, next_fn, action)
        
        assert len(called) == 1
        assert called[0] == action


# Tests de ThrottleMiddleware

class TestThrottleMiddleware:
    """Tests para ThrottleMiddleware."""
    
    def test_throttle_allows_first_action(self):
        """Test que throttle permite primera acción."""
        middleware = ThrottleMiddleware(delay=1000)
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        called = []
        
        def next_fn(action):
            called.append(action)
        
        action = SimpleAction("INCREMENT")
        middleware.handle(context, next_fn, action)
        
        assert len(called) == 1
    
    def test_throttle_blocks_rapid_actions(self, capsys):
        """Test que throttle bloquea acciones rápidas."""
        middleware = ThrottleMiddleware(delay=100)
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        called = []
        
        def next_fn(action):
            called.append(action)
        
        action = SimpleAction("INCREMENT")
        
        # Primera acción: pasa
        middleware.handle(context, next_fn, action)
        assert len(called) == 1
        
        # Segunda acción inmediata: bloqueada
        middleware.handle(context, next_fn, action)
        assert len(called) == 1  # No cambió
        
        captured = capsys.readouterr()
        assert "Throttled" in captured.out
    
    def test_throttle_allows_after_delay(self):
        """Test que throttle permite después del delay."""
        middleware = ThrottleMiddleware(delay=50)
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        called = []
        
        def next_fn(action):
            called.append(action)
        
        action = SimpleAction("INCREMENT")
        
        # Primera acción
        middleware.handle(context, next_fn, action)
        assert len(called) == 1
        
        # Esperar delay
        time.sleep(0.06)  # 60ms > 50ms
        
        # Segunda acción: pasa
        middleware.handle(context, next_fn, action)
        assert len(called) == 2


# Tests de DebounceMiddleware

class TestDebounceMiddleware:
    """Tests para DebounceMiddleware."""
    
    def test_debounce_delays_action(self):
        """Test que debounce retrasa acciones."""
        middleware = DebounceMiddleware(delay=50)
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        called = []
        
        def next_fn(action):
            called.append(action)
        
        action = SimpleAction("INCREMENT")
        middleware.handle(context, next_fn, action)
        
        # Inmediatamente: no se ejecutó
        assert len(called) == 0
        
        # Después del delay: se ejecutó
        time.sleep(0.06)  # 60ms > 50ms
        assert len(called) == 1


# Tests de ErrorHandlerMiddleware

class TestErrorHandlerMiddleware:
    """Tests para ErrorHandlerMiddleware."""
    
    def test_error_handler_catches_exceptions(self, capsys):
        """Test que error handler captura excepciones."""
        middleware = ErrorHandlerMiddleware()
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        def next_fn(action):
            raise ValueError("Test error")
        
        action = SimpleAction("INCREMENT")
        
        # No debe lanzar excepción
        middleware.handle(context, next_fn, action)
        
        captured = capsys.readouterr()
        assert "[Error]" in captured.out
        assert "Test error" in captured.out
    
    def test_error_handler_calls_callback(self):
        """Test que error handler llama callback custom."""
        errors_captured = []
        
        def on_error(error, action):
            errors_captured.append((error, action))
        
        middleware = ErrorHandlerMiddleware(on_error=on_error)
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        def next_fn(action):
            raise ValueError("Test error")
        
        action = SimpleAction("INCREMENT")
        middleware.handle(context, next_fn, action)
        
        assert len(errors_captured) == 1
        assert isinstance(errors_captured[0][0], ValueError)
        assert errors_captured[0][1] == action


# Tests de CacheMiddleware

class TestCacheMiddleware:
    """Tests para CacheMiddleware."""
    
    def test_cache_stores_results(self):
        """Test que cache guarda resultados."""
        middleware = CacheMiddleware(max_size=10)
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        executions = []
        
        def next_fn(action):
            executions.append(action)
            store.state = AppState(counter=store.state.counter + 1)
        
        action = SimpleAction("INCREMENT")
        
        # Primera ejecución
        middleware.handle(context, next_fn, action)
        assert len(executions) == 1
        assert store.state.counter == 1
        
        # Segunda ejecución (cacheada): no debería ejecutar
        middleware.handle(context, next_fn, action)
        assert len(executions) == 1  # No cambió
        assert store.state.counter == 1  # No cambió
    
    def test_cache_evicts_oldest(self):
        """Test que cache elimina el más viejo."""
        middleware = CacheMiddleware(max_size=2)
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        def next_fn(action):
            store.state = AppState(counter=store.state.counter + 1)
        
        # Llenar cache
        middleware.handle(context, next_fn, SimpleAction("ACTION1"))
        middleware.handle(context, next_fn, SimpleAction("ACTION2"))
        middleware.handle(context, next_fn, SimpleAction("ACTION3"))
        
        # Cache debe tener solo 2 elementos (max_size)
        assert len(middleware._cache) == 2
        assert len(middleware._cache_keys) == 2


# Tests de Helpers

class TestHelperFunctions:
    """Tests para funciones helper."""
    
    def test_compose_middleware(self):
        """Test de compose_middleware."""
        call_order = []
        
        class Middleware1(Middleware):
            def handle(self, context, next, action):
                call_order.append("before1")
                next(action)
                call_order.append("after1")
        
        class Middleware2(Middleware):
            def handle(self, context, next, action):
                call_order.append("before2")
                next(action)
                call_order.append("after2")
        
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        composed = compose_middleware(Middleware1(), Middleware2())
        
        def final_next(action):
            call_order.append("final")
        
        action = SimpleAction("TEST")
        composed(context, final_next, action)
        
        assert call_order == ["before1", "before2", "final", "after2", "after1"]
    
    def test_apply_middleware(self):
        """Test de apply_middleware."""
        store = MockStore()
        
        call_log = []
        
        class LoggingMiddleware(Middleware):
            def handle(self, context, next, action):
                call_log.append("middleware")
                next(action)
        
        apply_middleware(store, LoggingMiddleware())
        
        # Dispatch con middleware aplicado
        store.dispatch(SimpleAction("INCREMENT"))
        
        assert "middleware" in call_log
        assert len(store.actions_received) == 1
    
    def test_create_middleware(self):
        """Test de create_middleware."""
        call_log = []
        
        def my_handler(context, next, action):
            call_log.append("custom")
            next(action)
        
        middleware = create_middleware(my_handler)
        
        store = MockStore()
        context = MiddlewareContext(
            store=store,
            get_state=store.get_state,
            dispatch=store.dispatch
        )
        
        middleware.handle(context, lambda a: None, SimpleAction("TEST"))
        
        assert "custom" in call_log


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
