"""
Tests unitarios para Store<T>

Implementación de: VELA-577 (TASK-035S)
Sprint: Sprint 15
Fecha: 2025-12-02
"""

import pytest
import time
from src.reactive.store import Store
from src.reactive.action import SimpleAction, PayloadAction, InitAction, ResetAction


class TestStoreBasics:
    """Tests básicos del Store."""
    
    def test_store_creation(self):
        """Test crear un store."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            return state
        
        store = Store(initial_state, reducer)
        
        assert store.get_state() == {"count": 0}
        assert store.get_action_count() >= 1  # InitAction
    
    def test_store_get_state(self):
        """Test obtener estado."""
        initial_state = {"value": 42}
        
        def reducer(state, action):
            return state
        
        store = Store(initial_state, reducer)
        state = store.get_state()
        
        assert state["value"] == 42
    
    def test_store_dispatch(self):
        """Test dispatch de acciones."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        store.dispatch(SimpleAction("INCREMENT"))
        assert store.get_state()["count"] == 1
        
        store.dispatch(SimpleAction("INCREMENT"))
        assert store.get_state()["count"] == 2


class TestStoreSubscribe:
    """Tests para subscription."""
    
    def test_subscribe_basic(self):
        """Test suscribirse a cambios."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        # Track cambios
        calls = []
        
        def listener(state):
            calls.append(state["count"])
        
        store.subscribe(listener)
        
        store.dispatch(SimpleAction("INCREMENT"))
        store.dispatch(SimpleAction("INCREMENT"))
        
        # Listener fue llamado 2 veces
        assert len(calls) == 2
        assert calls == [1, 2]
    
    def test_unsubscribe(self):
        """Test desuscribirse."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        calls = []
        
        def listener(state):
            calls.append(state["count"])
        
        unsubscribe = store.subscribe(listener)
        
        store.dispatch(SimpleAction("INCREMENT"))
        assert len(calls) == 1
        
        # Unsubscribe
        unsubscribe()
        
        store.dispatch(SimpleAction("INCREMENT"))
        # No debe llamarse más
        assert len(calls) == 1
    
    def test_multiple_subscribers(self):
        """Test múltiples subscribers."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        calls1 = []
        calls2 = []
        
        store.subscribe(lambda state: calls1.append(state["count"]))
        store.subscribe(lambda state: calls2.append(state["count"]))
        
        store.dispatch(SimpleAction("INCREMENT"))
        
        assert len(calls1) == 1
        assert len(calls2) == 1


class TestStoreSelectors:
    """Tests para selectors."""
    
    def test_select_basic(self):
        """Test crear selector."""
        initial_state = {"count": 10, "name": "Vela"}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        # Selector para count
        count_selector = store.select(lambda state: state["count"])
        
        assert count_selector.get() == 10
        
        # Dispatch
        store.dispatch(SimpleAction("INCREMENT"))
        
        assert count_selector.get() == 11
    
    def test_select_computed(self):
        """Test selector con computación."""
        initial_state = {"count": 5}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        # Selector con transformación
        doubled = store.select(lambda state: state["count"] * 2)
        
        assert doubled.get() == 10
        
        store.dispatch(SimpleAction("INCREMENT"))
        assert doubled.get() == 12


class TestStoreReset:
    """Tests para reset."""
    
    def test_reset_store(self):
        """Test resetear store al estado inicial."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        # Cambiar estado
        store.dispatch(SimpleAction("INCREMENT"))
        store.dispatch(SimpleAction("INCREMENT"))
        assert store.get_state()["count"] == 2
        
        # Reset
        store.reset()
        assert store.get_state()["count"] == 0


class TestStoreMiddleware:
    """Tests para middleware system."""
    
    def test_middleware_basic(self):
        """Test middleware básico."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        # Middleware que logea
        logs = []
        
        def logger_middleware(store, next, action):
            logs.append(f"Before: {action.get_type()}")
            next(action)
            logs.append(f"After: {action.get_type()}")
        
        store = Store(initial_state, reducer, middlewares=[logger_middleware])
        
        store.dispatch(SimpleAction("INCREMENT"))
        
        # Middleware fue ejecutado
        assert "Before: INCREMENT" in logs
        assert "After: INCREMENT" in logs
    
    def test_middleware_chain(self):
        """Test cadena de middlewares."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            return state
        
        order = []
        
        def middleware1(store, next, action):
            order.append("m1-before")
            next(action)
            order.append("m1-after")
        
        def middleware2(store, next, action):
            order.append("m2-before")
            next(action)
            order.append("m2-after")
        
        store = Store(initial_state, reducer, middlewares=[middleware1, middleware2])
        
        # Limpiar orden después de InitAction
        order.clear()
        
        store.dispatch(SimpleAction("TEST"))
        
        # Orden correcto
        assert order == ["m1-before", "m2-before", "m2-after", "m1-after"]


class TestStoreDevTools:
    """Tests para DevTools integration."""
    
    def test_history_enabled(self):
        """Test historial habilitado."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer, enable_devtools=True)
        
        store.dispatch(SimpleAction("INCREMENT"))
        store.dispatch(SimpleAction("INCREMENT"))
        
        history = store.get_history()
        
        # InitAction + 2 INCREMENTs
        assert len(history) >= 3
    
    def test_time_travel(self):
        """Test time-travel debugging."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer, enable_devtools=True)
        
        # Dispatch varias acciones
        store.dispatch(SimpleAction("INCREMENT"))  # count = 1
        store.dispatch(SimpleAction("INCREMENT"))  # count = 2
        store.dispatch(SimpleAction("INCREMENT"))  # count = 3
        
        assert store.get_state()["count"] == 3
        
        # Time-travel al índice 1 (después de InitAction)
        store.time_travel(1)
        
        # Estado debe ser 1
        assert store.get_state()["count"] == 1
    
    def test_undo_redo(self):
        """Test undo/redo."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer, enable_devtools=True)
        
        store.dispatch(SimpleAction("INCREMENT"))  # 1
        store.dispatch(SimpleAction("INCREMENT"))  # 2
        
        assert store.get_state()["count"] == 2
        
        # Undo
        store.undo()
        assert store.get_state()["count"] == 1
        
        # Undo again
        store.undo()
        assert store.get_state()["count"] == 0
        
        # Redo
        store.redo()
        assert store.get_state()["count"] == 1
        
        # Redo again
        store.redo()
        assert store.get_state()["count"] == 2


class TestStoreSerialization:
    """Tests para serialización."""
    
    def test_to_json(self):
        """Test serializar estado a JSON."""
        initial_state = {"count": 42, "name": "Vela"}
        
        def reducer(state, action):
            return state
        
        store = Store(initial_state, reducer)
        
        json_str = store.to_json()
        
        assert '"count": 42' in json_str
        assert '"name": "Vela"' in json_str
    
    def test_from_json(self):
        """Test restaurar desde JSON."""
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        json_str = '{"count": 10, "name": "Vela"}'
        
        store = Store.from_json(json_str, reducer)
        
        assert store.get_state()["count"] == 10
        assert store.get_state()["name"] == "Vela"
        
        # Debe ser funcional
        store.dispatch(SimpleAction("INCREMENT"))
        assert store.get_state()["count"] == 11


class TestStorePerformance:
    """Tests de performance."""
    
    def test_many_dispatches(self):
        """Test performance con muchas acciones."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        start = time.perf_counter()
        
        # 1000 dispatches
        for _ in range(1000):
            store.dispatch(SimpleAction("INCREMENT"))
        
        elapsed = time.perf_counter() - start
        
        assert store.get_state()["count"] == 1000
        assert elapsed < 1.0  # Debe ser < 1 segundo
    
    def test_many_subscribers(self):
        """Test performance con muchos subscribers."""
        initial_state = {"count": 0}
        
        def reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {**state, "count": state["count"] + 1}
            return state
        
        store = Store(initial_state, reducer)
        
        # 100 subscribers
        for _ in range(100):
            store.subscribe(lambda state: None)
        
        start = time.perf_counter()
        
        # 100 dispatches
        for _ in range(100):
            store.dispatch(SimpleAction("INCREMENT"))
        
        elapsed = time.perf_counter() - start
        
        assert elapsed < 1.0  # Debe ser < 1 segundo


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
