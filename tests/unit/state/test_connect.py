"""
Tests unitarios para @connect decorator

Jira: VELA-577
Historia: Sprint 15 - State Management
Subtask: TASK-035V
"""

import pytest
import sys
sys.path.append('..')

from src.reactive.connect import (
    connect,
    ConnectOptions,
    shallow_equal,
    map_state_to_props,
    create_selector,
    connect_to_store,
    connect_with_dispatch
)
from src.reactive.store import Store
from src.reactive.action import Action
from src.reactive.reducer import Reducer


# ===================================================================
# FIXTURES
# ===================================================================

class CounterState:
    """Estado de prueba."""
    def __init__(self, count: int = 0):
        self.count = count


class IncrementAction(Action):
    """Acción de prueba."""
    def get_type(self) -> str:
        return "INCREMENT"


def counter_reducer(state: CounterState, action: Action) -> CounterState:
    """Reducer de prueba."""
    if isinstance(action, IncrementAction):
        return CounterState(state.count + 1)
    return state


@pytest.fixture
def counter_store():
    """Store de prueba."""
    return Store(
        initial_state=CounterState(0),
        reducer=counter_reducer
    )


class MockWidget:
    """Widget mock para testing."""
    def __init__(self):
        self.mount_called = False
        self.update_called = False
        self.destroy_called = False
        self.render_count = 0
    
    def mount(self):
        self.mount_called = True
    
    def update(self):
        self.update_called = True
        self.render_count += 1
    
    def destroy(self):
        self.destroy_called = True


# ===================================================================
# TESTS - shallow_equal
# ===================================================================

class TestShallowEqual:
    """Tests de shallow_equal function."""
    
    def test_same_object(self):
        """Test que mismo objeto es igual"""
        obj = {"a": 1}
        assert shallow_equal(obj, obj) == True
    
    def test_equal_dicts(self):
        """Test que dicts iguales son iguales"""
        assert shallow_equal({"a": 1, "b": 2}, {"a": 1, "b": 2}) == True
    
    def test_different_dicts(self):
        """Test que dicts diferentes NO son iguales"""
        assert shallow_equal({"a": 1}, {"a": 2}) == False
    
    def test_different_keys(self):
        """Test que dicts con keys diferentes NO son iguales"""
        assert shallow_equal({"a": 1}, {"b": 1}) == False
    
    def test_equal_lists(self):
        """Test que listas iguales son iguales"""
        assert shallow_equal([1, 2, 3], [1, 2, 3]) == True
    
    def test_different_lists(self):
        """Test que listas diferentes NO son iguales"""
        assert shallow_equal([1, 2], [1, 3]) == False
    
    def test_different_lengths(self):
        """Test que listas con longitudes diferentes NO son iguales"""
        assert shallow_equal([1, 2], [1, 2, 3]) == False
    
    def test_different_types(self):
        """Test que tipos diferentes NO son iguales"""
        assert shallow_equal({"a": 1}, [1]) == False
    
    def test_primitives(self):
        """Test comparación de primitivos"""
        assert shallow_equal(1, 1) == True
        assert shallow_equal("hello", "hello") == True
        assert shallow_equal(True, True) == True
        assert shallow_equal(1, 2) == False


# ===================================================================
# TESTS - @connect decorator
# ===================================================================

class TestConnectDecorator:
    """Tests del decorador @connect."""
    
    def test_connect_decorator_exists(self):
        """Test que el decorator existe"""
        assert connect is not None
        assert callable(connect)
    
    def test_connect_injects_props(self, counter_store):
        """Test que @connect inyecta props del selector"""
        
        @connect(ConnectOptions(
            store=counter_store,
            selector=lambda state: {"count": state.count}
        ))
        class TestWidget(MockWidget):
            pass
        
        widget = TestWidget()
        widget.mount()
        
        # Verificar que count fue inyectado
        assert hasattr(widget, 'count')
        assert widget.count == 0
    
    def test_connect_injects_dispatch(self, counter_store):
        """Test que @connect inyecta dispatch function"""
        
        @connect(ConnectOptions(
            store=counter_store,
            selector=lambda state: {"count": state.count}
        ))
        class TestWidget(MockWidget):
            pass
        
        widget = TestWidget()
        widget.mount()
        
        # Verificar que dispatch fue inyectado
        assert hasattr(widget, 'dispatch')
        assert callable(widget.dispatch)
    
    def test_connect_auto_subscribes(self, counter_store):
        """Test que @connect auto-subscribe al store"""
        
        @connect(ConnectOptions(
            store=counter_store,
            selector=lambda state: {"count": state.count}
        ))
        class TestWidget(MockWidget):
            pass
        
        widget = TestWidget()
        widget.mount()
        
        # Dispatch action
        counter_store.dispatch(IncrementAction())
        
        # Widget debería tener count actualizado
        assert widget.count == 1
    
    def test_connect_triggers_update_on_change(self, counter_store):
        """Test que @connect trigger update cuando props cambian"""
        
        @connect(ConnectOptions(
            store=counter_store,
            selector=lambda state: {"count": state.count}
        ))
        class TestWidget(MockWidget):
            pass
        
        widget = TestWidget()
        widget.mount()
        
        initial_render_count = widget.render_count
        
        # Dispatch action
        counter_store.dispatch(IncrementAction())
        
        # update() debería haberse llamado
        assert widget.render_count > initial_render_count
    
    def test_connect_no_update_if_props_equal(self, counter_store):
        """Test que @connect NO trigger update si props NO cambian"""
        
        # Selector que ignora el count (siempre retorna mismo valor)
        @connect(ConnectOptions(
            store=counter_store,
            selector=lambda state: {"static": "value"}
        ))
        class TestWidget(MockWidget):
            pass
        
        widget = TestWidget()
        widget.mount()
        
        initial_render_count = widget.render_count
        
        # Dispatch action (pero selector no cambia)
        counter_store.dispatch(IncrementAction())
        
        # update() NO debería haberse llamado
        assert widget.render_count == initial_render_count
    
    def test_connect_auto_unsubscribes(self, counter_store):
        """Test que @connect auto-unsubscribe al destruir"""
        
        @connect(ConnectOptions(
            store=counter_store,
            selector=lambda state: {"count": state.count}
        ))
        class TestWidget(MockWidget):
            pass
        
        widget = TestWidget()
        widget.mount()
        
        # Obtener listeners actuales
        initial_count = len(counter_store._listeners)
        
        # Destruir widget
        widget.destroy()
        
        # Debería haber un listener menos
        assert len(counter_store._listeners) < initial_count
    
    def test_connect_preserves_original_mount(self, counter_store):
        """Test que @connect preserva mount original"""
        
        mount_called = [False]
        
        @connect(ConnectOptions(
            store=counter_store,
            selector=lambda state: {"count": state.count}
        ))
        class TestWidget(MockWidget):
            def mount(self):
                super().mount()
                mount_called[0] = True
        
        widget = TestWidget()
        widget.mount()
        
        # mount original debería haberse llamado
        assert mount_called[0] == True
        assert widget.mount_called == True


# ===================================================================
# TESTS - create_selector
# ===================================================================

class TestCreateSelector:
    """Tests de create_selector (memoization)."""
    
    def test_create_selector_memoizes(self):
        """Test que create_selector cachea resultados"""
        
        call_count = [0]
        
        def expensive_computation(todos):
            call_count[0] += 1
            return [t for t in todos if t.get('completed')]
        
        selector = create_selector(
            lambda state: state.get('todos'),
            result_func=expensive_computation
        )
        
        state = {'todos': [{'id': 1, 'completed': True}]}
        
        # Primera llamada
        result1 = selector(state)
        assert call_count[0] == 1
        
        # Segunda llamada con mismo state
        result2 = selector(state)
        assert call_count[0] == 1  # NO recomputó
        assert result1 == result2
    
    def test_create_selector_recomputes_on_change(self):
        """Test que create_selector recomputa cuando inputs cambian"""
        
        call_count = [0]
        
        def compute(todos):
            call_count[0] += 1
            return len(todos)
        
        selector = create_selector(
            lambda state: state.get('todos'),
            result_func=compute
        )
        
        state1 = {'todos': [1, 2, 3]}
        state2 = {'todos': [1, 2, 3, 4]}
        
        # Primera llamada
        result1 = selector(state1)
        assert result1 == 3
        assert call_count[0] == 1
        
        # Segunda llamada con state diferente
        result2 = selector(state2)
        assert result2 == 4
        assert call_count[0] == 2  # Recomputó


# ===================================================================
# TESTS - API HELPERS
# ===================================================================

class TestConnectHelpers:
    """Tests de APIs helper."""
    
    def test_connect_to_store(self, counter_store):
        """Test de connect_to_store API simplificada"""
        
        @connect_to_store(counter_store, lambda state: {"count": state.count})
        class TestWidget(MockWidget):
            pass
        
        widget = TestWidget()
        widget.mount()
        
        assert hasattr(widget, 'count')
        assert widget.count == 0
    
    def test_connect_with_dispatch(self, counter_store):
        """Test de connect_with_dispatch"""
        
        @connect_with_dispatch(counter_store, lambda state: {"count": state.count})
        class TestWidget(MockWidget):
            pass
        
        widget = TestWidget()
        widget.mount()
        
        assert hasattr(widget, 'count')
        assert hasattr(widget, 'dispatch')
        
        # Usar dispatch
        widget.dispatch(IncrementAction())
        assert widget.count == 1
    
    def test_map_state_to_props(self):
        """Test de map_state_to_props (identity function)"""
        
        selector = map_state_to_props(lambda state: {"count": state.count})
        
        # Es solo una identity function para legibilidad
        assert callable(selector)


# ===================================================================
# TESTS - METADATA
# ===================================================================

class TestConnectMetadata:
    """Tests de metadata del decorator."""
    
    def test_connect_adds_metadata(self, counter_store):
        """Test que @connect agrega metadata a la clase"""
        
        @connect(ConnectOptions(
            store=counter_store,
            selector=lambda state: {"count": state.count}
        ))
        class TestWidget(MockWidget):
            pass
        
        # Verificar metadata
        assert hasattr(TestWidget, '__connected__')
        assert TestWidget.__connected__ == True
        
        assert hasattr(TestWidget, '__connect_options__')
        assert TestWidget.__connect_options__.store == counter_store


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
