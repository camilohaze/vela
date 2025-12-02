"""
Tests unitarios para Reducer types

Implementación de: VELA-577 (TASK-035S)
Sprint: Sprint 15
Fecha: 2025-12-02
"""

import pytest
from src.reactive.reducer import (
    Reducer,
    CombinedReducer,
    ReducerBuilder,
    create_reducer,
    identity_reducer
)
from src.reactive.action import SimpleAction, PayloadAction


class TestBasicReducer:
    """Tests para reducers básicos."""
    
    def test_counter_reducer(self):
        """Test reducer simple de contador."""
        def counter_reducer(state: int, action) -> int:
            if action.get_type() == "INCREMENT":
                return state + 1
            elif action.get_type() == "DECREMENT":
                return state - 1
            return state
        
        state = 0
        state = counter_reducer(state, SimpleAction("INCREMENT"))
        assert state == 1
        
        state = counter_reducer(state, SimpleAction("INCREMENT"))
        assert state == 2
        
        state = counter_reducer(state, SimpleAction("DECREMENT"))
        assert state == 1
    
    def test_reducer_purity(self):
        """Test que reducers son puros (no mutan estado)."""
        def reducer(state: dict, action) -> dict:
            if action.get_type() == "UPDATE":
                return {**state, "value": action.payload}
            return state
        
        original_state = {"value": 0}
        new_state = reducer(original_state, PayloadAction("UPDATE", payload=10))
        
        # Estado original no debe cambiar
        assert original_state["value"] == 0
        assert new_state["value"] == 10
    
    def test_reducer_with_payload(self):
        """Test reducer con actions con payload."""
        def reducer(state: int, action) -> int:
            if action.get_type() == "SET":
                return action.payload
            elif action.get_type() == "ADD":
                return state + action.payload
            return state
        
        state = 5
        state = reducer(state, PayloadAction("SET", payload=10))
        assert state == 10
        
        state = reducer(state, PayloadAction("ADD", payload=3))
        assert state == 13


class TestCombinedReducer:
    """Tests para CombinedReducer."""
    
    def test_combined_reducer_basic(self):
        """Test combinar múltiples reducers."""
        def counter_reducer(state: int, action) -> int:
            if state is None:
                state = 0
            if action.get_type() == "INCREMENT":
                return state + 1
            return state
        
        def todos_reducer(state: list, action) -> list:
            if state is None:
                state = []
            if action.get_type() == "ADD_TODO":
                return [*state, action.payload]
            return state
        
        combined = CombinedReducer({
            "counter": counter_reducer,
            "todos": todos_reducer
        })
        
        state = {"counter": 0, "todos": []}
        
        # Dispatch INCREMENT
        new_state = combined.reduce(state, SimpleAction("INCREMENT"))
        assert new_state["counter"] == 1
        assert new_state["todos"] == []
        
        # Dispatch ADD_TODO
        new_state = combined.reduce(new_state, PayloadAction("ADD_TODO", payload="Learn Vela"))
        assert new_state["counter"] == 1
        assert len(new_state["todos"]) == 1
    
    def test_combined_reducer_isolation(self):
        """Test que cada reducer solo modifica su parte del estado."""
        def reducer_a(state, action):
            if action.get_type() == "UPDATE_A":
                return action.payload
            return state
        
        def reducer_b(state, action):
            if action.get_type() == "UPDATE_B":
                return action.payload
            return state
        
        combined = CombinedReducer({
            "a": reducer_a,
            "b": reducer_b
        })
        
        state = {"a": 1, "b": 2}
        
        # Solo debe cambiar "a"
        new_state = combined.reduce(state, PayloadAction("UPDATE_A", payload=10))
        assert new_state["a"] == 10
        assert new_state["b"] == 2
        
        # Solo debe cambiar "b"
        new_state = combined.reduce(new_state, PayloadAction("UPDATE_B", payload=20))
        assert new_state["a"] == 10
        assert new_state["b"] == 20
    
    def test_combined_reducer_returns_same_object_if_no_change(self):
        """Test optimización: retornar mismo objeto si nada cambió."""
        def noop_reducer(state, action):
            return state
        
        combined = CombinedReducer({
            "value": noop_reducer
        })
        
        state = {"value": 42}
        new_state = combined.reduce(state, SimpleAction("NOOP"))
        
        # Debe ser el mismo objeto (identity comparison)
        assert new_state is state


class TestReducerBuilder:
    """Tests para ReducerBuilder."""
    
    def test_builder_basic(self):
        """Test builder básico."""
        reducer = (
            ReducerBuilder(initial_state=0)
            .case("INCREMENT", lambda state, action: state + 1)
            .case("DECREMENT", lambda state, action: state - 1)
            .build()
        )
        
        state = 0
        state = reducer(state, SimpleAction("INCREMENT"))
        assert state == 1
        
        state = reducer(state, SimpleAction("DECREMENT"))
        assert state == 0
    
    def test_builder_with_payload(self):
        """Test builder con payload."""
        reducer = (
            ReducerBuilder()
            .case("SET", lambda state, action: action.payload)
            .case("ADD", lambda state, action: state + action.payload)
            .build()
        )
        
        state = 5
        state = reducer(state, PayloadAction("SET", payload=10))
        assert state == 10
        
        state = reducer(state, PayloadAction("ADD", payload=3))
        assert state == 13
    
    def test_builder_with_default_handler(self):
        """Test builder con handler por defecto."""
        reducer = (
            ReducerBuilder(initial_state=0)
            .case("INCREMENT", lambda state, action: state + 1)
            .default(lambda state, action: state)
            .build()
        )
        
        state = 5
        state = reducer(state, SimpleAction("INCREMENT"))
        assert state == 6
        
        # Action desconocida usa default handler
        state = reducer(state, SimpleAction("UNKNOWN"))
        assert state == 6
    
    def test_builder_chaining(self):
        """Test encadenamiento de métodos."""
        reducer = (
            ReducerBuilder()
            .case("A", lambda s, a: s + 1)
            .case("B", lambda s, a: s + 2)
            .case("C", lambda s, a: s + 3)
            .build()
        )
        
        state = 0
        state = reducer(state, SimpleAction("A"))
        state = reducer(state, SimpleAction("B"))
        state = reducer(state, SimpleAction("C"))
        
        assert state == 6


class TestCreateReducer:
    """Tests para create_reducer helper."""
    
    def test_create_reducer_basic(self):
        """Test crear reducer con dict de handlers."""
        reducer = create_reducer({
            "INCREMENT": lambda state, action: state + 1,
            "DECREMENT": lambda state, action: state - 1,
            "RESET": lambda state, action: 0
        })
        
        state = 0
        state = reducer(state, SimpleAction("INCREMENT"))
        assert state == 1
        
        state = reducer(state, SimpleAction("INCREMENT"))
        assert state == 2
        
        state = reducer(state, SimpleAction("RESET"))
        assert state == 0
    
    def test_create_reducer_with_default(self):
        """Test create_reducer con handler por defecto."""
        reducer = create_reducer(
            handlers={
                "SET": lambda state, action: action.payload
            },
            default=lambda state, action: state
        )
        
        state = 5
        state = reducer(state, PayloadAction("SET", payload=10))
        assert state == 10
        
        # Action desconocida usa default
        state = reducer(state, SimpleAction("UNKNOWN"))
        assert state == 10


class TestIdentityReducer:
    """Tests para identity_reducer."""
    
    def test_identity_reducer(self):
        """Test que identity reducer no cambia el estado."""
        state = {"value": 42}
        new_state = identity_reducer(state, SimpleAction("ANY"))
        
        assert new_state is state
        assert new_state == {"value": 42}
    
    def test_identity_reducer_with_different_types(self):
        """Test identity reducer con diferentes tipos."""
        assert identity_reducer(0, SimpleAction("A")) == 0
        assert identity_reducer("test", SimpleAction("A")) == "test"
        assert identity_reducer([1, 2, 3], SimpleAction("A")) == [1, 2, 3]


class TestComplexReducers:
    """Tests para casos complejos."""
    
    def test_todo_app_reducer(self):
        """Test reducer complejo de Todo App."""
        def todos_reducer(state: list, action) -> list:
            if state is None:
                state = []
            
            action_type = action.get_type()
            
            if action_type == "ADD_TODO":
                new_todo = {
                    "id": len(state) + 1,
                    "text": action.payload,
                    "completed": False
                }
                return [*state, new_todo]
            
            elif action_type == "TOGGLE_TODO":
                return [
                    {**todo, "completed": not todo["completed"]} 
                    if todo["id"] == action.payload 
                    else todo
                    for todo in state
                ]
            
            elif action_type == "REMOVE_TODO":
                return [todo for todo in state if todo["id"] != action.payload]
            
            elif action_type == "CLEAR_COMPLETED":
                return [todo for todo in state if not todo["completed"]]
            
            return state
        
        # Test flujo completo
        state = []
        
        # Agregar todos
        state = todos_reducer(state, PayloadAction("ADD_TODO", payload="Learn Vela"))
        state = todos_reducer(state, PayloadAction("ADD_TODO", payload="Build app"))
        assert len(state) == 2
        
        # Toggle
        state = todos_reducer(state, PayloadAction("TOGGLE_TODO", payload=1))
        assert state[0]["completed"] == True
        assert state[1]["completed"] == False
        
        # Remove
        state = todos_reducer(state, PayloadAction("REMOVE_TODO", payload=2))
        assert len(state) == 1
        
        # Clear completed
        state = todos_reducer(state, SimpleAction("CLEAR_COMPLETED"))
        assert len(state) == 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
