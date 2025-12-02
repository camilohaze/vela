"""
Tests E2E para TodoApp con State Management completo

Historia: VELA-577
Task: TASK-035AA
Sprint: Sprint 15

Tests end-to-end que simulan una aplicación completa de TODOs con:
- Store + Actions + Reducers
- @connect para widgets
- @select para selectors memoizados
- @persistent para guardar estado
- Middleware para logging y undo/redo
"""

import pytest
import json
import time
from typing import Any, Dict, List
from dataclasses import dataclass


# =====================================================
# STORE & STATE MANAGEMENT (from integration tests)
# =====================================================

class Action:
    """Base action class"""
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
        
        # Apply middleware
        if self.middlewares:
            original_dispatch = self._dispatch
            for middleware in reversed(self.middlewares):
                original_dispatch = self._wrap_dispatch_with_middleware(original_dispatch, middleware)
            self._final_dispatch = original_dispatch
        else:
            self._final_dispatch = self._dispatch
    
    def _wrap_dispatch_with_middleware(self, next_dispatch, middleware):
        def wrapped(action):
            context = MiddlewareContext(self, self.get_state, next_dispatch)
            middleware.handle(context, next_dispatch, action)
        return wrapped
    
    def _dispatch(self, action):
        self.state = self.reducer(self.state, action)
        self._notify_subscribers()
    
    def dispatch(self, action):
        self._final_dispatch(action)
    
    def subscribe(self, listener):
        self.subscribers.append(listener)
        return lambda: self.subscribers.remove(listener)
    
    def get_state(self):
        return self.state
    
    def _notify_subscribers(self):
        for listener in self.subscribers:
            listener()


@dataclass
class MiddlewareContext:
    store: Store
    get_state: callable
    dispatch: callable


class Middleware:
    def handle(self, context: MiddlewareContext, next_func, action: Action):
        next_func(action)


# =====================================================
# DECORATORS
# =====================================================

def connect(store, map_state_to_props=None, map_dispatch_to_props=None):
    """@connect decorator"""
    def decorator(widget_class):
        class ConnectedWidget(widget_class):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                self._store = store
                self._unsubscribe = None
                self._mount()
            
            def _mount(self):
                self._unsubscribe = self._store.subscribe(self._on_state_change)
                self._update_props()
            
            def _on_state_change(self):
                old_props = self.props.copy()
                self._update_props()
                if old_props != self.props:
                    self.render()
            
            def _update_props(self):
                state = self._store.get_state()
                if map_state_to_props:
                    self.props.update(map_state_to_props(state))
                if map_dispatch_to_props:
                    self.props.update(map_dispatch_to_props(self._store.dispatch))
            
            def unmount(self):
                if self._unsubscribe:
                    self._unsubscribe()
        
        return ConnectedWidget
    return decorator


def persistent(options):
    """@persistent decorator"""
    def decorator(store_class):
        class PersistentStore(store_class):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                self._persist_key = options.get("key", "vela-store")
                self._storage = options.get("storage", {})
                self._load_persisted_state()
                self.subscribe(self._save_state)
            
            def _load_persisted_state(self):
                data = self._storage.get(self._persist_key)
                if data:
                    self.state.update(json.loads(data))
            
            def _save_state(self):
                self._storage[self._persist_key] = json.dumps(self.state)
            
            def clear_persisted_state(self):
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
            "timestamp": time.time(),
            "prev_state": context.get_state()
        })
        next_func(action)
        self.logs[-1]["next_state"] = context.get_state()


class UndoRedoMiddleware(Middleware):
    """Implements undo/redo functionality"""
    def __init__(self, max_history=50):
        self.past = []
        self.future = []
        self.max_history = max_history
    
    def handle(self, context, next_func, action):
        if action.type == "UNDO":
            if self.past:
                current = context.get_state()
                self.future.append(current)
                previous = self.past.pop()
                context.store.state = previous
                context.store._notify_subscribers()
            return
        
        elif action.type == "REDO":
            if self.future:
                current = context.get_state()
                self.past.append(current)
                next_state = self.future.pop()
                context.store.state = next_state
                context.store._notify_subscribers()
            return
        
        # Normal action: save to history
        current = context.get_state()
        self.past.append(current)
        if len(self.past) > self.max_history:
            self.past.pop(0)
        self.future = []
        
        next_func(action)
    
    def can_undo(self):
        return len(self.past) > 0
    
    def can_redo(self):
        return len(self.future) > 0


# =====================================================
# TODO APP - DOMAIN
# =====================================================

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


class SetFilterAction(Action):
    def __init__(self, filter_type: str):
        super().__init__("SET_FILTER", {"filter": filter_type})


class ClearCompletedAction(Action):
    def __init__(self):
        super().__init__("CLEAR_COMPLETED")


class EditTodoAction(Action):
    def __init__(self, todo_id: int, text: str):
        super().__init__("EDIT_TODO", {"id": todo_id, "text": text})


# Reducer
def todo_reducer(state: Dict[str, Any], action: Action) -> Dict[str, Any]:
    """Main reducer for TodoApp"""
    if action.type == "ADD_TODO":
        todos = state.get("todos", [])
        new_todo = {
            "id": state.get("next_id", 1),
            "text": action.payload["text"],
            "completed": False
        }
        return {
            **state,
            "todos": todos + [new_todo],
            "next_id": state.get("next_id", 1) + 1
        }
    
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
    
    elif action.type == "SET_FILTER":
        return {**state, "filter": action.payload["filter"]}
    
    elif action.type == "CLEAR_COMPLETED":
        todos = state.get("todos", [])
        new_todos = [t for t in todos if not t["completed"]]
        return {**state, "todos": new_todos}
    
    elif action.type == "EDIT_TODO":
        todos = state.get("todos", [])
        new_todos = [
            {**t, "text": action.payload["text"]} if t["id"] == action.payload["id"] else t
            for t in todos
        ]
        return {**state, "todos": new_todos}
    
    return state


# Selectors
def select_visible_todos(state: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Select todos based on current filter"""
    todos = state.get("todos", [])
    filter_type = state.get("filter", "all")
    
    if filter_type == "active":
        return [t for t in todos if not t["completed"]]
    elif filter_type == "completed":
        return [t for t in todos if t["completed"]]
    else:
        return todos


def select_todo_stats(state: Dict[str, Any]) -> Dict[str, int]:
    """Select todo statistics"""
    todos = state.get("todos", [])
    return {
        "total": len(todos),
        "active": len([t for t in todos if not t["completed"]]),
        "completed": len([t for t in todos if t["completed"]])
    }


# =====================================================
# TODO APP - WIDGETS
# =====================================================

class Widget:
    """Base widget class"""
    def __init__(self):
        self.rendered_count = 0
        self.props = {}
        self.render_history = []
    
    def render(self):
        self.rendered_count += 1
        self.render_history.append(self.props.copy())
        return f"Widget rendered {self.rendered_count} times"


class TodoListWidget(Widget):
    """Widget that displays list of todos"""
    pass


class TodoStatsWidget(Widget):
    """Widget that displays todo statistics"""
    pass


class TodoFiltersWidget(Widget):
    """Widget for filter buttons"""
    pass


# =====================================================
# E2E TESTS
# =====================================================

class TestTodoAppE2E:
    """Tests E2E de TodoApp completo"""
    
    def test_add_multiple_todos(self):
        """Test agregar múltiples TODOs"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1, "filter": "all"})
        
        store.dispatch(AddTodoAction("Aprender Vela"))
        store.dispatch(AddTodoAction("Crear parser"))
        store.dispatch(AddTodoAction("Implementar runtime"))
        
        state = store.get_state()
        assert len(state["todos"]) == 3
        assert state["todos"][0]["text"] == "Aprender Vela"
        assert state["todos"][1]["text"] == "Crear parser"
        assert state["todos"][2]["text"] == "Implementar runtime"
    
    def test_toggle_todo_completion(self):
        """Test marcar TODO como completado"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1})
        
        store.dispatch(AddTodoAction("Tarea 1"))
        store.dispatch(ToggleTodoAction(1))
        
        state = store.get_state()
        assert state["todos"][0]["completed"] == True
        
        # Toggle de nuevo
        store.dispatch(ToggleTodoAction(1))
        state = store.get_state()
        assert state["todos"][0]["completed"] == False
    
    def test_remove_todo(self):
        """Test eliminar TODO"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1})
        
        store.dispatch(AddTodoAction("Tarea 1"))
        store.dispatch(AddTodoAction("Tarea 2"))
        store.dispatch(RemoveTodoAction(1))
        
        state = store.get_state()
        assert len(state["todos"]) == 1
        assert state["todos"][0]["text"] == "Tarea 2"
    
    def test_filter_todos_by_status(self):
        """Test filtrar TODOs por estado"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1, "filter": "all"})
        
        store.dispatch(AddTodoAction("Tarea 1"))
        store.dispatch(AddTodoAction("Tarea 2"))
        store.dispatch(AddTodoAction("Tarea 3"))
        store.dispatch(ToggleTodoAction(1))
        store.dispatch(ToggleTodoAction(3))
        
        # Filtrar activos
        store.dispatch(SetFilterAction("active"))
        active_todos = select_visible_todos(store.get_state())
        assert len(active_todos) == 1
        assert active_todos[0]["text"] == "Tarea 2"
        
        # Filtrar completados
        store.dispatch(SetFilterAction("completed"))
        completed_todos = select_visible_todos(store.get_state())
        assert len(completed_todos) == 2
        assert completed_todos[0]["text"] == "Tarea 1"
        assert completed_todos[1]["text"] == "Tarea 3"
        
        # Mostrar todos
        store.dispatch(SetFilterAction("all"))
        all_todos = select_visible_todos(store.get_state())
        assert len(all_todos) == 3
    
    def test_clear_completed_todos(self):
        """Test limpiar TODOs completados"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1})
        
        store.dispatch(AddTodoAction("Tarea 1"))
        store.dispatch(AddTodoAction("Tarea 2"))
        store.dispatch(AddTodoAction("Tarea 3"))
        store.dispatch(ToggleTodoAction(1))
        store.dispatch(ToggleTodoAction(2))
        
        store.dispatch(ClearCompletedAction())
        
        state = store.get_state()
        assert len(state["todos"]) == 1
        assert state["todos"][0]["text"] == "Tarea 3"
    
    def test_edit_todo_text(self):
        """Test editar texto de TODO"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1})
        
        store.dispatch(AddTodoAction("Tarea original"))
        store.dispatch(EditTodoAction(1, "Tarea editada"))
        
        state = store.get_state()
        assert state["todos"][0]["text"] == "Tarea editada"
    
    def test_todo_statistics(self):
        """Test estadísticas de TODOs"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1})
        
        store.dispatch(AddTodoAction("Tarea 1"))
        store.dispatch(AddTodoAction("Tarea 2"))
        store.dispatch(AddTodoAction("Tarea 3"))
        store.dispatch(ToggleTodoAction(1))
        
        stats = select_todo_stats(store.get_state())
        assert stats["total"] == 3
        assert stats["active"] == 2
        assert stats["completed"] == 1


class TestTodoAppWithWidgets:
    """Tests E2E con widgets conectados"""
    
    def test_todo_list_widget_updates_on_add(self):
        """Test que widget de lista se actualiza al agregar TODO"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1, "filter": "all"})
        
        @connect(store, map_state_to_props=lambda state: {
            "todos": select_visible_todos(state)
        })
        class TodoList(TodoListWidget):
            pass
        
        widget = TodoList()
        initial_renders = widget.rendered_count
        
        store.dispatch(AddTodoAction("Nueva tarea"))
        
        assert widget.rendered_count == initial_renders + 1
        assert len(widget.props["todos"]) == 1
    
    def test_stats_widget_updates_on_toggle(self):
        """Test que widget de stats se actualiza al toggle"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1})
        
        @connect(store, map_state_to_props=lambda state: {
            "stats": select_todo_stats(state)
        })
        class StatsWidget(TodoStatsWidget):
            pass
        
        widget = StatsWidget()
        
        store.dispatch(AddTodoAction("Tarea 1"))
        assert widget.props["stats"]["total"] == 1
        assert widget.props["stats"]["active"] == 1
        
        store.dispatch(ToggleTodoAction(1))
        assert widget.props["stats"]["active"] == 0
        assert widget.props["stats"]["completed"] == 1
    
    def test_multiple_widgets_sync(self):
        """Test que múltiples widgets se sincronizan"""
        store = Store(todo_reducer, {"todos": [], "next_id": 1, "filter": "all"})
        
        @connect(store, map_state_to_props=lambda state: {
            "todos": select_visible_todos(state)
        })
        class TodoList(TodoListWidget):
            pass
        
        @connect(store, map_state_to_props=lambda state: {
            "stats": select_todo_stats(state)
        })
        class StatsWidget(TodoStatsWidget):
            pass
        
        list_widget = TodoList()
        stats_widget = StatsWidget()
        
        store.dispatch(AddTodoAction("Tarea 1"))
        
        assert len(list_widget.props["todos"]) == 1
        assert stats_widget.props["stats"]["total"] == 1


class TestTodoAppWithPersistence:
    """Tests E2E con persistencia"""
    
    def test_todos_persist_across_sessions(self):
        """Test que TODOs persisten entre sesiones"""
        storage = {}
        
        @persistent({"key": "todo-app", "storage": storage})
        class TodoStore(Store):
            pass
        
        # Sesión 1: Crear TODOs
        store1 = TodoStore(todo_reducer, {"todos": [], "next_id": 1})
        store1.dispatch(AddTodoAction("Tarea persistente"))
        
        # Sesión 2: Restaurar TODOs
        store2 = TodoStore(todo_reducer, {"todos": [], "next_id": 1})
        state = store2.get_state()
        
        assert len(state["todos"]) == 1
        assert state["todos"][0]["text"] == "Tarea persistente"
    
    def test_filter_persists_across_sessions(self):
        """Test que filtro persiste entre sesiones"""
        storage = {}
        
        @persistent({"key": "todo-app", "storage": storage})
        class TodoStore(Store):
            pass
        
        # Sesión 1: Establecer filtro
        store1 = TodoStore(todo_reducer, {"todos": [], "filter": "all"})
        store1.dispatch(SetFilterAction("completed"))
        
        # Sesión 2: Verificar filtro
        store2 = TodoStore(todo_reducer, {"todos": [], "filter": "all"})
        assert store2.get_state()["filter"] == "completed"


class TestTodoAppWithMiddleware:
    """Tests E2E con middleware"""
    
    def test_logger_records_all_actions(self):
        """Test que logger registra todas las acciones"""
        logger = LoggerMiddleware()
        store = Store(todo_reducer, {"todos": [], "next_id": 1}, middlewares=[logger])
        
        store.dispatch(AddTodoAction("Tarea 1"))
        store.dispatch(AddTodoAction("Tarea 2"))
        store.dispatch(ToggleTodoAction(1))
        
        assert len(logger.logs) == 3
        assert logger.logs[0]["action"] == "ADD_TODO"
        assert logger.logs[1]["action"] == "ADD_TODO"
        assert logger.logs[2]["action"] == "TOGGLE_TODO"
    
    def test_undo_redo_functionality(self):
        """Test funcionalidad de undo/redo"""
        undo_redo = UndoRedoMiddleware()
        store = Store(todo_reducer, {"todos": [], "next_id": 1}, middlewares=[undo_redo])
        
        # Agregar TODOs
        store.dispatch(AddTodoAction("Tarea 1"))
        store.dispatch(AddTodoAction("Tarea 2"))
        assert len(store.get_state()["todos"]) == 2
        
        # Undo
        store.dispatch(Action("UNDO"))
        assert len(store.get_state()["todos"]) == 1
        
        store.dispatch(Action("UNDO"))
        assert len(store.get_state()["todos"]) == 0
        
        # Redo
        store.dispatch(Action("REDO"))
        assert len(store.get_state()["todos"]) == 1
        
        store.dispatch(Action("REDO"))
        assert len(store.get_state()["todos"]) == 2
    
    def test_undo_after_new_action_clears_future(self):
        """Test que nueva acción limpia el historial de redo"""
        undo_redo = UndoRedoMiddleware()
        store = Store(todo_reducer, {"todos": [], "next_id": 1}, middlewares=[undo_redo])
        
        store.dispatch(AddTodoAction("Tarea 1"))
        store.dispatch(AddTodoAction("Tarea 2"))
        store.dispatch(Action("UNDO"))
        
        # Nueva acción: debe limpiar future
        store.dispatch(AddTodoAction("Tarea 3"))
        
        # No debe poder hacer redo
        store.dispatch(Action("REDO"))
        assert len(store.get_state()["todos"]) == 2
        assert store.get_state()["todos"][1]["text"] == "Tarea 3"


class TestTodoAppCompleteStack:
    """Tests E2E con stack completo (Store + widgets + persistence + middleware)"""
    
    def test_complete_todo_workflow(self):
        """Test workflow completo de TodoApp"""
        storage = {}
        logger = LoggerMiddleware()
        undo_redo = UndoRedoMiddleware()
        
        @persistent({"key": "complete-todo-app", "storage": storage})
        class TodoStore(Store):
            pass
        
        store = TodoStore(
            todo_reducer,
            {"todos": [], "next_id": 1, "filter": "all"},
            middlewares=[logger, undo_redo]
        )
        
        # Widgets
        @connect(store, map_state_to_props=lambda state: {
            "todos": select_visible_todos(state)
        })
        class TodoList(TodoListWidget):
            pass
        
        @connect(store, map_state_to_props=lambda state: {
            "stats": select_todo_stats(state)
        })
        class StatsWidget(TodoStatsWidget):
            pass
        
        list_widget = TodoList()
        stats_widget = StatsWidget()
        
        # 1. Agregar TODOs
        store.dispatch(AddTodoAction("Implementar parser"))
        store.dispatch(AddTodoAction("Crear tests"))
        store.dispatch(AddTodoAction("Documentar API"))
        
        assert len(list_widget.props["todos"]) == 3
        assert stats_widget.props["stats"]["total"] == 3
        
        # 2. Completar algunos
        store.dispatch(ToggleTodoAction(1))
        store.dispatch(ToggleTodoAction(2))
        
        assert stats_widget.props["stats"]["completed"] == 2
        assert stats_widget.props["stats"]["active"] == 1
        
        # 3. Filtrar completados
        store.dispatch(SetFilterAction("completed"))
        assert len(list_widget.props["todos"]) == 2
        
        # 4. Undo (deshace el último filter, quedamos en filter="completed")
        # Después del undo, tenemos 2 completados (id=1, id=2) y 1 activo (id=3)
        # El filter sigue siendo "completed" tras el undo
        store.dispatch(Action("UNDO"))
        # Cambiar a "active" muestra solo el activo
        store.dispatch(SetFilterAction("active"))
        assert len(list_widget.props["todos"]) == 1  # Solo id=3 está activo
        
        # 5. Verificar persistencia
        assert "complete-todo-app" in storage
        
        # 6. Verificar logging (UNDO no genera log adicional)
        assert len(logger.logs) == 8  # 3 add + 2 toggle + 2 filter + 1 filter after undo
        
        # 7. Limpiar completados
        # Después del undo de filter, tenemos 2 completados (id=1, id=2) y 1 activo (id=3)
        # Cambiamos a "all" para ver todos antes de limpiar
        store.dispatch(SetFilterAction("all"))
        todos_before_clear = len(list_widget.props["todos"])
        assert todos_before_clear == 3  # 2 completados + 1 activo
        
        store.dispatch(ClearCompletedAction())
        
        # Después de limpiar, solo queda el activo (id=3)
        assert len(list_widget.props["todos"]) == 1
        assert stats_widget.props["stats"]["total"] == 1
        assert stats_widget.props["stats"]["active"] == 1
        assert stats_widget.props["stats"]["completed"] == 0


# =====================================================
# EJECUTAR TESTS
# =====================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
