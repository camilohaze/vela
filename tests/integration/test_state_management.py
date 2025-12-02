"""
Tests de Integración para State Management System

Historia: VELA-577
Task: TASK-035AA
Sprint: Sprint 15

Tests que validan la integración completa del sistema de state management:
- Store + Actions + Reducers + @connect + @select + @persistent + middleware
"""

import pytest
import time
import json
from typing import Any, Dict, List
from dataclasses import dataclass


# =====================================================
# MOCK CLASSES (Store, Action, Reducer, etc.)
# =====================================================

class Action:
    """Base class for actions"""
    def __init__(self, action_type: str, payload: Any = None):
        self.type = action_type
        self.payload = payload


class Store:
    """Store implementation"""
    def __init__(self, reducer, initial_state=None, middlewares=None):
        self.state = initial_state or {}
        self.reducer = reducer
        self.middlewares = middlewares or []
        self.subscribers = []
        
        # Apply middleware to dispatch
        if self.middlewares:
            original_dispatch = self._dispatch
            for middleware in reversed(self.middlewares):
                original_dispatch = self._wrap_dispatch_with_middleware(original_dispatch, middleware)
            self._final_dispatch = original_dispatch
        else:
            self._final_dispatch = self._dispatch
    
    def _wrap_dispatch_with_middleware(self, next_dispatch, middleware):
        """Wrap dispatch with middleware"""
        def wrapped(action):
            context = MiddlewareContext(self, self.get_state, next_dispatch)
            middleware.handle(context, next_dispatch, action)
        return wrapped
    
    def _dispatch(self, action):
        """Internal dispatch without middleware"""
        self.state = self.reducer(self.state, action)
        self._notify_subscribers()
    
    def dispatch(self, action):
        """Public dispatch (goes through middleware)"""
        self._final_dispatch(action)
    
    def subscribe(self, listener):
        """Subscribe to state changes"""
        self.subscribers.append(listener)
        return lambda: self.subscribers.remove(listener)
    
    def get_state(self):
        """Get current state"""
        return self.state
    
    def _notify_subscribers(self):
        """Notify all subscribers"""
        for listener in self.subscribers:
            listener()


@dataclass
class MiddlewareContext:
    """Context passed to middleware"""
    store: Store
    get_state: callable
    dispatch: callable


class Middleware:
    """Base middleware class"""
    def handle(self, context: MiddlewareContext, next_func, action: Action):
        next_func(action)


# =====================================================
# MOCK WIDGETS & DECORATORS
# =====================================================

class Widget:
    """Base widget class"""
    def __init__(self):
        self.rendered_count = 0
        self.props = {}
    
    def render(self):
        self.rendered_count += 1
        return f"Widget rendered {self.rendered_count} times"


def connect(store, map_state_to_props=None, map_dispatch_to_props=None):
    """@connect decorator mock"""
    def decorator(widget_class):
        class ConnectedWidget(widget_class):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                self._store = store
                self._unsubscribe = None
                self._mount()
            
            def _mount(self):
                """Subscribe to store"""
                self._unsubscribe = self._store.subscribe(self._on_state_change)
                self._update_props()
            
            def _on_state_change(self):
                """Handle state change"""
                old_props = self.props.copy()
                self._update_props()
                if old_props != self.props:
                    self.render()
            
            def _update_props(self):
                """Update props from store"""
                state = self._store.get_state()
                if map_state_to_props:
                    self.props.update(map_state_to_props(state))
                if map_dispatch_to_props:
                    self.props.update(map_dispatch_to_props(self._store.dispatch))
            
            def unmount(self):
                """Unsubscribe from store"""
                if self._unsubscribe:
                    self._unsubscribe()
        
        return ConnectedWidget
    return decorator


def select(selector_func):
    """@select decorator mock with memoization"""
    cache = {"prev_state": None, "prev_result": None}
    
    def decorator(method):
        def wrapper(self, state):
            # Check cache
            if cache["prev_state"] == state:
                return cache["prev_result"]
            
            # Compute new result
            result = selector_func(state)
            cache["prev_state"] = state
            cache["prev_result"] = result
            return result
        
        return wrapper
    return decorator


def persistent(options):
    """@persistent decorator mock"""
    def decorator(store_class):
        class PersistentStore(store_class):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                self._persist_key = options.get("key", "vela-store")
                self._storage = options.get("storage", {})
                self._load_persisted_state()
                
                # Subscribe to save on changes
                self.subscribe(self._save_state)
            
            def _load_persisted_state(self):
                """Load persisted state"""
                data = self._storage.get(self._persist_key)
                if data:
                    self.state.update(json.loads(data))
            
            def _save_state(self):
                """Save current state"""
                self._storage[self._persist_key] = json.dumps(self.state)
            
            def clear_persisted_state(self):
                """Clear persisted state"""
                if self._persist_key in self._storage:
                    del self._storage[self._persist_key]
        
        return PersistentStore
    return decorator


# =====================================================
# MIDDLEWARES
# =====================================================

class LoggerMiddleware(Middleware):
    """Logs all actions"""
    def __init__(self):
        self.logs = []
    
    def handle(self, context, next_func, action):
        self.logs.append({
            "action": action.type,
            "payload": action.payload,
            "timestamp": time.time()
        })
        next_func(action)


class ErrorHandlerMiddleware(Middleware):
    """Catches errors in reducers"""
    def __init__(self):
        self.errors = []
    
    def handle(self, context, next_func, action):
        try:
            next_func(action)
        except Exception as e:
            self.errors.append({"action": action.type, "error": str(e)})
            context.dispatch(Action("ERROR", {"message": str(e)}))


class ThrottleMiddleware(Middleware):
    """Throttles actions"""
    def __init__(self, delay=0.1):
        self.delay = delay
        self.last_dispatch = {}
    
    def handle(self, context, next_func, action):
        now = time.time()
        last = self.last_dispatch.get(action.type, 0)
        
        if now - last >= self.delay:
            self.last_dispatch[action.type] = now
            next_func(action)


# =====================================================
# TODO APP (E2E Test Scenario)
# =====================================================

@dataclass
class Todo:
    """Todo item"""
    id: int
    text: str
    completed: bool = False


# Actions
class AddTodoAction(Action):
    def __init__(self, text: str):
        super().__init__("ADD_TODO", {"text": text})


class ToggleTodoAction(Action):
    def __init__(self, todo_id: int):
        super().__init__("TOGGLE_TODO", {"id": todo_id})


class RemoveTodoAction(Action):
    def __init__(self, todo_id: int):
        super().__init__("REMOVE_TODO", {"id": todo_id})


# Reducer
def todo_reducer(state: Dict[str, Any], action: Action) -> Dict[str, Any]:
    """Reducer for todo app"""
    if action.type == "ADD_TODO":
        todos = state.get("todos", [])
        new_todo = {
            "id": len(todos) + 1,
            "text": action.payload["text"],
            "completed": False
        }
        return {**state, "todos": todos + [new_todo]}
    
    elif action.type == "TOGGLE_TODO":
        todos = state.get("todos", [])
        new_todos = [
            {**t, "completed": not t["completed"]} if t["id"] == action.payload["id"] else t
            for t in todos
        ]
        return {**state, "todos": new_todos}
    
    elif action.type == "REMOVE_TODO":
        todos = state.get("todos", [])
        new_todos = [t for t in todos if t["id"] != action.payload["id"]]
        return {**state, "todos": new_todos}
    
    elif action.type == "ERROR":
        return {**state, "error": action.payload["message"]}
    
    return state


# Selectors
def select_todos(state: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Select all todos"""
    return state.get("todos", [])


def select_active_todos(state: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Select active todos"""
    return [t for t in state.get("todos", []) if not t["completed"]]


def select_completed_todos(state: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Select completed todos"""
    return [t for t in state.get("todos", []) if t["completed"]]


def select_todo_count(state: Dict[str, Any]) -> Dict[str, int]:
    """Select todo counts"""
    todos = state.get("todos", [])
    return {
        "total": len(todos),
        "active": len([t for t in todos if not t["completed"]]),
        "completed": len([t for t in todos if t["completed"]])
    }


# =====================================================
# INTEGRATION TESTS
# =====================================================

class TestStoreIntegration:
    """Tests de integración del Store básico"""
    
    def test_store_dispatch_updates_state(self):
        """Test que dispatch actualiza el estado"""
        def counter_reducer(state, action):
            if action.type == "INCREMENT":
                return {**state, "count": state.get("count", 0) + 1}
            return state
        
        store = Store(counter_reducer, {"count": 0})
        store.dispatch(Action("INCREMENT"))
        
        assert store.get_state()["count"] == 1
    
    def test_store_subscribe_notifies_listeners(self):
        """Test que subscribe notifica a listeners"""
        def counter_reducer(state, action):
            if action.type == "INCREMENT":
                return {**state, "count": state.get("count", 0) + 1}
            return state
        
        store = Store(counter_reducer, {"count": 0})
        notifications = []
        
        store.subscribe(lambda: notifications.append(store.get_state()["count"]))
        store.dispatch(Action("INCREMENT"))
        store.dispatch(Action("INCREMENT"))
        
        assert notifications == [1, 2]
    
    def test_store_unsubscribe_stops_notifications(self):
        """Test que unsubscribe detiene notificaciones"""
        def counter_reducer(state, action):
            if action.type == "INCREMENT":
                return {**state, "count": state.get("count", 0) + 1}
            return state
        
        store = Store(counter_reducer, {"count": 0})
        notifications = []
        
        unsubscribe = store.subscribe(lambda: notifications.append(store.get_state()["count"]))
        store.dispatch(Action("INCREMENT"))
        
        unsubscribe()
        store.dispatch(Action("INCREMENT"))
        
        assert notifications == [1]  # Solo primera notificación


class TestConnectIntegration:
    """Tests de integración del @connect decorator"""
    
    def test_connect_injects_state_as_props(self):
        """Test que @connect inyecta estado como props"""
        def reducer(state, action):
            if action.type == "SET_NAME":
                return {**state, "name": action.payload}
            return state
        
        store = Store(reducer, {"name": "Vela"})
        
        @connect(store, map_state_to_props=lambda state: {"name": state["name"]})
        class NameWidget(Widget):
            pass
        
        widget = NameWidget()
        assert widget.props["name"] == "Vela"
    
    def test_connect_triggers_render_on_state_change(self):
        """Test que @connect re-renderiza al cambiar estado"""
        def reducer(state, action):
            if action.type == "INCREMENT":
                return {**state, "count": state.get("count", 0) + 1}
            return state
        
        store = Store(reducer, {"count": 0})
        
        @connect(store, map_state_to_props=lambda state: {"count": state["count"]})
        class CounterWidget(Widget):
            pass
        
        widget = CounterWidget()
        initial_renders = widget.rendered_count
        
        store.dispatch(Action("INCREMENT"))
        
        assert widget.rendered_count == initial_renders + 1
        assert widget.props["count"] == 1
    
    def test_connect_does_not_render_if_props_unchanged(self):
        """Test que @connect NO re-renderiza si props no cambian"""
        def reducer(state, action):
            if action.type == "SET_OTHER":
                return {**state, "other": action.payload}
            return state
        
        store = Store(reducer, {"count": 0, "other": "value"})
        
        @connect(store, map_state_to_props=lambda state: {"count": state["count"]})
        class CounterWidget(Widget):
            pass
        
        widget = CounterWidget()
        initial_renders = widget.rendered_count
        
        # Cambiar estado que NO afecta props
        store.dispatch(Action("SET_OTHER", "new_value"))
        
        assert widget.rendered_count == initial_renders
    
    def test_connect_unmount_stops_updates(self):
        """Test que unmount detiene actualizaciones"""
        def reducer(state, action):
            if action.type == "INCREMENT":
                return {**state, "count": state.get("count", 0) + 1}
            return state
        
        store = Store(reducer, {"count": 0})
        
        @connect(store, map_state_to_props=lambda state: {"count": state["count"]})
        class CounterWidget(Widget):
            pass
        
        widget = CounterWidget()
        widget.unmount()
        
        initial_renders = widget.rendered_count
        store.dispatch(Action("INCREMENT"))
        
        assert widget.rendered_count == initial_renders


class TestSelectIntegration:
    """Tests de integración del @select decorator"""
    
    def test_select_memoizes_results(self):
        """Test que @select cachea resultados"""
        computation_count = [0]
        
        def expensive_selector(state):
            computation_count[0] += 1
            return state.get("todos", [])
        
        class TodoSelector:
            @select(expensive_selector)
            def get_todos(self, state):
                pass
        
        selector = TodoSelector()
        state = {"todos": [{"id": 1, "text": "Task 1"}]}
        
        # Primera llamada: debe computar
        result1 = selector.get_todos(state)
        
        # Segunda llamada con mismo estado: debe usar cache
        result2 = selector.get_todos(state)
        
        assert computation_count[0] == 1
        assert result1 == result2
    
    def test_select_recomputes_on_state_change(self):
        """Test que @select recomputa cuando cambia estado"""
        computation_count = [0]
        
        def counter_selector(state):
            computation_count[0] += 1
            return state.get("count", 0)
        
        class CounterSelector:
            @select(counter_selector)
            def get_count(self, state):
                pass
        
        selector = CounterSelector()
        state1 = {"count": 0}
        state2 = {"count": 1}
        
        result1 = selector.get_count(state1)
        result2 = selector.get_count(state2)
        
        assert computation_count[0] == 2
        assert result1 == 0
        assert result2 == 1


class TestPersistentIntegration:
    """Tests de integración del @persistent decorator"""
    
    def test_persistent_saves_state_on_change(self):
        """Test que @persistent guarda estado al cambiar"""
        storage = {}
        
        def counter_reducer(state, action):
            if action.type == "INCREMENT":
                return {**state, "count": state.get("count", 0) + 1}
            return state
        
        @persistent({"key": "counter-store", "storage": storage})
        class CounterStore(Store):
            pass
        
        store = CounterStore(counter_reducer, {"count": 0})
        store.dispatch(Action("INCREMENT"))
        
        assert "counter-store" in storage
        saved_state = json.loads(storage["counter-store"])
        assert saved_state["count"] == 1
    
    def test_persistent_restores_state_on_init(self):
        """Test que @persistent restaura estado al inicializar"""
        storage = {"counter-store": json.dumps({"count": 42})}
        
        def counter_reducer(state, action):
            return state
        
        @persistent({"key": "counter-store", "storage": storage})
        class CounterStore(Store):
            pass
        
        store = CounterStore(counter_reducer, {"count": 0})
        
        assert store.get_state()["count"] == 42
    
    def test_persistent_clear_removes_saved_state(self):
        """Test que clear_persisted_state elimina estado guardado"""
        storage = {"counter-store": json.dumps({"count": 42})}
        
        def counter_reducer(state, action):
            return state
        
        @persistent({"key": "counter-store", "storage": storage})
        class CounterStore(Store):
            pass
        
        store = CounterStore(counter_reducer, {"count": 0})
        store.clear_persisted_state()
        
        assert "counter-store" not in storage


class TestMiddlewareIntegration:
    """Tests de integración del sistema de middleware"""
    
    def test_middleware_intercepts_actions(self):
        """Test que middleware intercepta acciones"""
        logger = LoggerMiddleware()
        
        def counter_reducer(state, action):
            if action.type == "INCREMENT":
                return {**state, "count": state.get("count", 0) + 1}
            return state
        
        store = Store(counter_reducer, {"count": 0}, middlewares=[logger])
        store.dispatch(Action("INCREMENT"))
        
        assert len(logger.logs) == 1
        assert logger.logs[0]["action"] == "INCREMENT"
    
    def test_middleware_chain_executes_in_order(self):
        """Test que middleware chain se ejecuta en orden"""
        execution_order = []
        
        class FirstMiddleware(Middleware):
            def handle(self, context, next_func, action):
                execution_order.append("first")
                next_func(action)
        
        class SecondMiddleware(Middleware):
            def handle(self, context, next_func, action):
                execution_order.append("second")
                next_func(action)
        
        def reducer(state, action):
            execution_order.append("reducer")
            return state
        
        store = Store(reducer, {}, middlewares=[FirstMiddleware(), SecondMiddleware()])
        store.dispatch(Action("TEST"))
        
        assert execution_order == ["first", "second", "reducer"]
    
    def test_error_handler_middleware_catches_exceptions(self):
        """Test que ErrorHandlerMiddleware captura excepciones"""
        error_handler = ErrorHandlerMiddleware()
        
        def buggy_reducer(state, action):
            if action.type == "BUGGY":
                raise ValueError("Reducer error")
            return state
        
        store = Store(buggy_reducer, {}, middlewares=[error_handler])
        store.dispatch(Action("BUGGY"))
        
        assert len(error_handler.errors) == 1
        assert error_handler.errors[0]["action"] == "BUGGY"
    
    def test_throttle_middleware_limits_dispatch_rate(self):
        """Test que ThrottleMiddleware limita tasa de dispatch"""
        throttle = ThrottleMiddleware(delay=0.1)
        logger = LoggerMiddleware()
        
        def reducer(state, action):
            return state
        
        store = Store(reducer, {}, middlewares=[throttle, logger])
        
        # Dispatch rápido (3 acciones)
        store.dispatch(Action("TEST"))
        store.dispatch(Action("TEST"))
        store.dispatch(Action("TEST"))
        
        # Solo primera debe pasar
        assert len(logger.logs) == 1


class TestFullStackIntegration:
    """Tests de integración con stack completo (Store + @connect + @select + @persistent + middleware)"""
    
    def test_todo_app_full_flow(self):
        """Test del flujo completo de TodoApp"""
        storage = {}
        logger = LoggerMiddleware()
        
        @persistent({"key": "todos", "storage": storage})
        class TodoStore(Store):
            pass
        
        store = TodoStore(todo_reducer, {"todos": []}, middlewares=[logger])
        
        # Widget conectado
        @connect(store, map_state_to_props=lambda state: {
            "todos": select_todos(state),
            "count": select_todo_count(state)
        })
        class TodoListWidget(Widget):
            pass
        
        widget = TodoListWidget()
        
        # Agregar todo
        store.dispatch(AddTodoAction("Aprender Vela"))
        assert len(widget.props["todos"]) == 1
        assert widget.props["count"]["total"] == 1
        assert widget.props["count"]["active"] == 1
        
        # Toggle todo
        store.dispatch(ToggleTodoAction(1))
        assert widget.props["todos"][0]["completed"] == True
        assert widget.props["count"]["completed"] == 1
        
        # Verificar persistencia
        assert "todos" in storage
        saved_state = json.loads(storage["todos"])
        assert len(saved_state["todos"]) == 1
        
        # Verificar logging
        assert len(logger.logs) == 2
        assert logger.logs[0]["action"] == "ADD_TODO"
        assert logger.logs[1]["action"] == "TOGGLE_TODO"
    
    def test_multiple_widgets_share_state(self):
        """Test que múltiples widgets comparten el mismo estado"""
        def counter_reducer(state, action):
            if action.type == "INCREMENT":
                return {**state, "count": state.get("count", 0) + 1}
            return state
        
        store = Store(counter_reducer, {"count": 0})
        
        @connect(store, map_state_to_props=lambda state: {"count": state["count"]})
        class CounterWidget(Widget):
            pass
        
        widget1 = CounterWidget()
        widget2 = CounterWidget()
        widget3 = CounterWidget()
        
        store.dispatch(Action("INCREMENT"))
        
        assert widget1.props["count"] == 1
        assert widget2.props["count"] == 1
        assert widget3.props["count"] == 1
    
    def test_state_immutability_preserved(self):
        """Test que la inmutabilidad del estado se preserva"""
        def reducer(state, action):
            if action.type == "UPDATE":
                return {**state, "value": action.payload}
            return state
        
        store = Store(reducer, {"value": "original"})
        original_state = store.get_state()
        
        store.dispatch(Action("UPDATE", "modified"))
        new_state = store.get_state()
        
        assert original_state["value"] == "original"
        assert new_state["value"] == "modified"
        assert id(original_state) != id(new_state)


# =====================================================
# EJECUTAR TESTS
# =====================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
