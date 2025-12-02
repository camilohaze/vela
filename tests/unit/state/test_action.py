"""
Tests unitarios para Action types

Implementación de: VELA-577 (TASK-035S)
Sprint: Sprint 15
Fecha: 2025-12-02
"""

import pytest
from src.reactive.action import (
    Action,
    SimpleAction,
    PayloadAction,
    ActionCreator,
    InitAction,
    ResetAction
)


class TestSimpleAction:
    """Tests para SimpleAction."""
    
    def test_create_simple_action(self):
        """Test creación de action simple."""
        action = SimpleAction("INCREMENT")
        
        assert action.get_type() == "INCREMENT"
        assert action.type == "INCREMENT"
    
    def test_simple_action_immutable(self):
        """Test que SimpleAction es inmutable (frozen dataclass)."""
        action = SimpleAction("INCREMENT")
        
        with pytest.raises(Exception):
            action.type = "DECREMENT"
    
    def test_simple_action_to_dict(self):
        """Test conversión a dict."""
        action = SimpleAction("INCREMENT")
        
        result = action.to_dict()
        
        assert result == {"type": "INCREMENT"}
    
    def test_simple_action_repr(self):
        """Test representación string."""
        action = SimpleAction("INCREMENT")
        
        assert "SimpleAction" in repr(action)


class TestPayloadAction:
    """Tests para PayloadAction."""
    
    def test_create_payload_action(self):
        """Test creación de action con payload."""
        action = PayloadAction("ADD_TODO", payload="Learn Vela")
        
        assert action.get_type() == "ADD_TODO"
        assert action.payload == "Learn Vela"
    
    def test_payload_action_with_number(self):
        """Test action con payload numérico."""
        action = PayloadAction("SET_COUNT", payload=42)
        
        assert action.payload == 42
    
    def test_payload_action_with_dict(self):
        """Test action con payload dict."""
        payload = {"id": 1, "text": "Test", "completed": False}
        action = PayloadAction("ADD_TODO", payload=payload)
        
        assert action.payload == payload
        assert action.payload["id"] == 1
    
    def test_payload_action_immutable(self):
        """Test que PayloadAction es inmutable."""
        action = PayloadAction("SET", payload=10)
        
        with pytest.raises(Exception):
            action.payload = 20
    
    def test_payload_action_to_dict(self):
        """Test conversión a dict."""
        action = PayloadAction("ADD_TODO", payload="Learn Vela")
        
        result = action.to_dict()
        
        assert result["type"] == "ADD_TODO"
        assert result["payload"] == "Learn Vela"


class TestActionCreator:
    """Tests para ActionCreator factory."""
    
    def test_create_simple_action_creator(self):
        """Test crear action creator simple."""
        increment = ActionCreator.simple("INCREMENT")
        
        action = increment()
        
        assert action.get_type() == "INCREMENT"
        assert isinstance(action, SimpleAction)
    
    def test_create_payload_action_creator(self):
        """Test crear action creator con payload."""
        add_todo = ActionCreator.payload("ADD_TODO")
        
        action = add_todo("Learn Vela")
        
        assert action.get_type() == "ADD_TODO"
        assert action.payload == "Learn Vela"
        assert isinstance(action, PayloadAction)
    
    def test_action_creators_reusable(self):
        """Test que los creators son reutilizables."""
        increment = ActionCreator.simple("INCREMENT")
        
        action1 = increment()
        action2 = increment()
        
        # Son distintos objetos
        assert action1 is not action2
        # Pero del mismo tipo
        assert action1.get_type() == action2.get_type()
    
    def test_payload_creator_with_different_payloads(self):
        """Test creator con diferentes payloads."""
        add_todo = ActionCreator.payload("ADD_TODO")
        
        action1 = add_todo("Task 1")
        action2 = add_todo("Task 2")
        
        assert action1.payload == "Task 1"
        assert action2.payload == "Task 2"


class TestSpecialActions:
    """Tests para acciones especiales predefinidas."""
    
    def test_init_action(self):
        """Test InitAction."""
        action = InitAction()
        
        assert action.get_type() == "@@INIT"
        assert isinstance(action, SimpleAction)
    
    def test_reset_action(self):
        """Test ResetAction."""
        action = ResetAction()
        
        assert action.get_type() == "@@RESET"
        assert isinstance(action, SimpleAction)


class TestCustomAction:
    """Tests para acciones custom."""
    
    def test_custom_action_class(self):
        """Test crear clase custom de Action."""
        class IncrementAction(Action):
            def __init__(self):
                self.type = "INCREMENT"
            
            def get_type(self):
                return self.type
        
        action = IncrementAction()
        
        assert action.get_type() == "INCREMENT"
    
    def test_custom_action_with_data(self):
        """Test custom action con datos."""
        class AddTodoAction(Action):
            def __init__(self, text: str, priority: int):
                self.type = "ADD_TODO"
                self.text = text
                self.priority = priority
            
            def get_type(self):
                return self.type
        
        action = AddTodoAction("Learn Vela", priority=1)
        
        assert action.get_type() == "ADD_TODO"
        assert action.text == "Learn Vela"
        assert action.priority == 1
    
    def test_custom_action_to_dict(self):
        """Test custom action serialization."""
        class AddTodoAction(Action):
            def __init__(self, text: str):
                self.type = "ADD_TODO"
                self.text = text
            
            def get_type(self):
                return self.type
        
        action = AddTodoAction("Learn Vela")
        result = action.to_dict()
        
        assert result["type"] == "ADD_TODO"
        assert result["text"] == "Learn Vela"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
