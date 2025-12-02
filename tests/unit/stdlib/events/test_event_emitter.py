"""
Tests unitarios para EventEmitter interface

TASK-035N: Implementar EventEmitter interface
Historia: VELA-575 - Dependency Injection
Epic: VELA-573 - Sistema de Reactividad
Sprint: 14

Tests para:
- EventEmitter interface (API contract)
- EventEmitterBase class (default implementation)
- TypedEventEmitter (type-safe variant)
- EventEmitterMixin (composition pattern)
- Integration con EventBus runtime
"""

import pytest
import sys
from typing import Any
sys.path.append('../..')

from src.runtime.events.event_bus import (
    EventBus, Event, Subscription, EventListener
)


# ============================================================================
# MOCK IMPLEMENTATION: EventEmitterBase en Python
# ============================================================================

class EventEmitterBase:
    """
    Python implementation of EventEmitterBase (transpiled from Vela).
    Esta es la implementación que el compilador Vela generaría.
    """
    
    def __init__(self):
        """Initialize EventEmitterBase con internal EventBus."""
        self._bus = EventBus()
    
    def on(self, event_type: str, listener: EventListener) -> Subscription:
        """Subscribe to event type."""
        return self._bus.on(event_type, listener)
    
    def once(self, event_type: str, listener: EventListener) -> Subscription:
        """Subscribe for one-time notification."""
        return self._bus.once(event_type, listener)
    
    def emit(self, event_type: str, payload: Any) -> None:
        """Emit event to all listeners."""
        self._bus.emit(event_type, payload)
    
    def off(self, event_type: str, listener: EventListener) -> None:
        """Remove listener from event type."""
        self._bus.off(event_type, listener)
    
    def remove_all_listeners(self, event_type: str = None) -> None:
        """Remove all listeners for event type or all events."""
        self._bus.clear(event_type)
    
    def listener_count(self, event_type: str) -> int:
        """Get count of listeners for event type."""
        return self._bus.listener_count(event_type)
    
    def event_types(self) -> list:
        """Get list of all registered event types."""
        return self._bus.event_types()


class TypedEventEmitter(EventEmitterBase):
    """
    Python implementation of TypedEventEmitter.
    Type checking se hace en compile-time en Vela, aquí es runtime.
    """
    
    def __init__(self, expected_type=None):
        """
        Initialize with optional type checking.
        
        Args:
            expected_type: Expected type for payload (for runtime checking)
        """
        super().__init__()
        self._expected_type = expected_type
    
    def emit(self, event_type: str, payload: Any) -> None:
        """Emit typed event with optional runtime type checking."""
        if self._expected_type is not None:
            if not isinstance(payload, self._expected_type):
                raise TypeError(
                    f"Expected payload of type {self._expected_type}, "
                    f"got {type(payload)}"
                )
        super().emit(event_type, payload)


class EventEmitterMixin:
    """
    Python implementation of EventEmitterMixin.
    Composition-based event emitter for classes that can't extend.
    """
    
    def __init__(self):
        """Initialize mixin with internal EventEmitterBase."""
        self._emitter = EventEmitterBase()
    
    def on(self, event_type: str, listener: EventListener) -> Subscription:
        """Subscribe to event type."""
        return self._emitter.on(event_type, listener)
    
    def once(self, event_type: str, listener: EventListener) -> Subscription:
        """Subscribe for one-time notification."""
        return self._emitter.once(event_type, listener)
    
    def emit(self, event_type: str, payload: Any) -> None:
        """Emit event."""
        self._emitter.emit(event_type, payload)
    
    def off(self, event_type: str, listener: EventListener) -> None:
        """Remove listener."""
        self._emitter.off(event_type, listener)
    
    def remove_all_listeners(self, event_type: str = None) -> None:
        """Remove all listeners."""
        self._emitter.remove_all_listeners(event_type)
    
    def listener_count(self, event_type: str) -> int:
        """Get listener count."""
        return self._emitter.listener_count(event_type)


# ============================================================================
# TEST FIXTURES
# ============================================================================

@pytest.fixture
def emitter():
    """Create EventEmitterBase instance for testing."""
    return EventEmitterBase()


@pytest.fixture
def typed_emitter():
    """Create TypedEventEmitter instance for testing."""
    return TypedEventEmitter(expected_type=dict)


@pytest.fixture
def mixin():
    """Create EventEmitterMixin instance for testing."""
    return EventEmitterMixin()


# ============================================================================
# TEST SUITE: EventEmitterBase - Basic Functionality
# ============================================================================

class TestEventEmitterBasicFunctionality:
    """Tests básicos de EventEmitterBase."""
    
    def test_initialization(self, emitter):
        """Test que EventEmitterBase se inicializa correctamente."""
        assert emitter is not None
        assert hasattr(emitter, '_bus')
        assert isinstance(emitter._bus, EventBus)
    
    def test_on_returns_subscription(self, emitter):
        """Test que on() retorna Subscription object."""
        handler = lambda e: None
        subscription = emitter.on("test.event", handler)
        
        assert isinstance(subscription, Subscription)
        assert subscription.event_type == "test.event"
        assert subscription.listener == handler
    
    def test_emit_calls_listener(self, emitter):
        """Test que emit() llama al listener registrado."""
        received_events = []
        
        def handler(event: Event):
            received_events.append(event)
        
        emitter.on("test.event", handler)
        emitter.emit("test.event", {"data": "hello"})
        
        assert len(received_events) == 1
        assert received_events[0].type == "test.event"
        assert received_events[0].payload == {"data": "hello"}
    
    def test_multiple_listeners_same_event(self, emitter):
        """Test múltiples listeners para mismo evento."""
        call_count = []
        
        emitter.on("test", lambda e: call_count.append(1))
        emitter.on("test", lambda e: call_count.append(2))
        emitter.on("test", lambda e: call_count.append(3))
        
        emitter.emit("test", "payload")
        
        assert len(call_count) == 3
        assert call_count == [1, 2, 3]
    
    def test_off_removes_listener(self, emitter):
        """Test que off() remueve listener correctamente."""
        call_count = []
        handler = lambda e: call_count.append(1)
        
        emitter.on("test", handler)
        emitter.emit("test", "data")
        assert len(call_count) == 1
        
        emitter.off("test", handler)
        emitter.emit("test", "data")
        assert len(call_count) == 1  # No se llamó de nuevo
    
    def test_once_calls_only_once(self, emitter):
        """Test que once() solo llama una vez."""
        call_count = []
        
        emitter.once("test", lambda e: call_count.append(1))
        
        emitter.emit("test", "data1")
        emitter.emit("test", "data2")
        emitter.emit("test", "data3")
        
        assert len(call_count) == 1
    
    def test_listener_count(self, emitter):
        """Test que listener_count() retorna count correcto."""
        assert emitter.listener_count("test") == 0
        
        emitter.on("test", lambda e: None)
        assert emitter.listener_count("test") == 1
        
        emitter.on("test", lambda e: None)
        assert emitter.listener_count("test") == 2
        
        emitter.on("other", lambda e: None)
        assert emitter.listener_count("test") == 2
        assert emitter.listener_count("other") == 1
    
    def test_remove_all_listeners_specific_event(self, emitter):
        """Test removeAllListeners() para evento específico."""
        emitter.on("test", lambda e: None)
        emitter.on("test", lambda e: None)
        emitter.on("other", lambda e: None)
        
        assert emitter.listener_count("test") == 2
        assert emitter.listener_count("other") == 1
        
        emitter.remove_all_listeners("test")
        
        assert emitter.listener_count("test") == 0
        assert emitter.listener_count("other") == 1
    
    def test_remove_all_listeners_all_events(self, emitter):
        """Test removeAllListeners() sin args limpia todos."""
        emitter.on("test1", lambda e: None)
        emitter.on("test2", lambda e: None)
        emitter.on("test3", lambda e: None)
        
        assert len(emitter.event_types()) == 3
        
        emitter.remove_all_listeners()
        
        assert len(emitter.event_types()) == 0
    
    def test_event_types(self, emitter):
        """Test que event_types() retorna tipos registrados."""
        assert emitter.event_types() == []
        
        emitter.on("test1", lambda e: None)
        emitter.on("test2", lambda e: None)
        emitter.on("test1", lambda e: None)
        
        types = emitter.event_types()
        assert len(types) == 2
        assert "test1" in types
        assert "test2" in types


# ============================================================================
# TEST SUITE: EventEmitter - Advanced Features
# ============================================================================

class TestEventEmitterAdvancedFeatures:
    """Tests de features avanzadas de EventEmitter."""
    
    def test_subscription_unsubscribe(self, emitter):
        """Test que Subscription.unsubscribe() funciona."""
        call_count = []
        handler = lambda e: call_count.append(1)
        
        subscription = emitter.on("test", handler)
        emitter.emit("test", "data")
        assert len(call_count) == 1
        
        subscription.unsubscribe()
        emitter.emit("test", "data")
        assert len(call_count) == 1  # No se llamó
    
    def test_subscription_context_manager(self, emitter):
        """Test Subscription como context manager."""
        call_count = []
        handler = lambda e: call_count.append(1)
        
        with emitter.on("test", handler) as sub:
            emitter.emit("test", "data")
            assert len(call_count) == 1
        
        # Después del context, debe estar unsubscribed
        emitter.emit("test", "data")
        assert len(call_count) == 1
    
    def test_multiple_instances_isolated(self):
        """Test que múltiples instancias están aisladas."""
        emitter1 = EventEmitterBase()
        emitter2 = EventEmitterBase()
        
        count1 = []
        count2 = []
        
        emitter1.on("test", lambda e: count1.append(1))
        emitter2.on("test", lambda e: count2.append(1))
        
        emitter1.emit("test", "data")
        assert len(count1) == 1
        assert len(count2) == 0
        
        emitter2.emit("test", "data")
        assert len(count1) == 1
        assert len(count2) == 1
    
    def test_error_isolation(self, emitter):
        """Test que error en un listener no afecta otros."""
        call_order = []
        
        def good_handler1(e):
            call_order.append("good1")
        
        def bad_handler(e):
            call_order.append("bad")
            raise Exception("Handler error")
        
        def good_handler2(e):
            call_order.append("good2")
        
        emitter.on("test", good_handler1)
        emitter.on("test", bad_handler)
        emitter.on("test", good_handler2)
        
        # Emit no debe crashear
        emitter.emit("test", "data")
        
        # Todos los handlers deben haberse ejecutado
        assert call_order == ["good1", "bad", "good2"]
    
    def test_emit_with_different_payload_types(self, emitter):
        """Test que emit() acepta diferentes tipos de payload."""
        received_payloads = []
        
        emitter.on("test", lambda e: received_payloads.append(e.payload))
        
        emitter.emit("test", "string")
        emitter.emit("test", 42)
        emitter.emit("test", {"key": "value"})
        emitter.emit("test", [1, 2, 3])
        emitter.emit("test", None)
        
        assert received_payloads == [
            "string",
            42,
            {"key": "value"},
            [1, 2, 3],
            None
        ]


# ============================================================================
# TEST SUITE: TypedEventEmitter
# ============================================================================

class TestTypedEventEmitter:
    """Tests para TypedEventEmitter (type-safe variant)."""
    
    def test_initialization(self, typed_emitter):
        """Test que TypedEventEmitter se inicializa."""
        assert typed_emitter is not None
        assert typed_emitter._expected_type == dict
    
    def test_emit_correct_type(self, typed_emitter):
        """Test que emit con tipo correcto funciona."""
        received = []
        typed_emitter.on("test", lambda e: received.append(e.payload))
        
        # Debe aceptar dict
        typed_emitter.emit("test", {"key": "value"})
        assert len(received) == 1
        assert received[0] == {"key": "value"}
    
    def test_emit_wrong_type_raises_error(self, typed_emitter):
        """Test que emit con tipo incorrecto lanza error."""
        typed_emitter.on("test", lambda e: None)
        
        # Debe rechazar string (esperaba dict)
        with pytest.raises(TypeError) as exc_info:
            typed_emitter.emit("test", "wrong type")
        
        assert "Expected payload of type" in str(exc_info.value)
    
    def test_no_type_checking_when_none(self):
        """Test que sin tipo no hay validación."""
        emitter = TypedEventEmitter(expected_type=None)
        received = []
        
        emitter.on("test", lambda e: received.append(e.payload))
        
        # Debe aceptar cualquier tipo
        emitter.emit("test", "string")
        emitter.emit("test", 123)
        emitter.emit("test", {"dict": True})
        
        assert len(received) == 3


# ============================================================================
# TEST SUITE: EventEmitterMixin
# ============================================================================

class TestEventEmitterMixin:
    """Tests para EventEmitterMixin (composition pattern)."""
    
    def test_initialization(self, mixin):
        """Test que mixin se inicializa correctamente."""
        assert mixin is not None
        assert hasattr(mixin, '_emitter')
        assert isinstance(mixin._emitter, EventEmitterBase)
    
    def test_mixin_on_emit_work(self, mixin):
        """Test que on/emit funcionan en mixin."""
        received = []
        
        mixin.on("test", lambda e: received.append(e.payload))
        mixin.emit("test", "data")
        
        assert len(received) == 1
        assert received[0] == "data"
    
    def test_mixin_composition_pattern(self):
        """Test mixin usado en composition pattern."""
        
        class CustomComponent:
            """Component que usa mixin en lugar de inheritance."""
            
            def __init__(self):
                self._events = EventEmitterMixin()
            
            def on(self, event_type, listener):
                return self._events.on(event_type, listener)
            
            def emit(self, event_type, payload):
                self._events.emit(event_type, payload)
            
            def do_something(self):
                self.emit("action", "did something")
        
        component = CustomComponent()
        received = []
        
        component.on("action", lambda e: received.append(e.payload))
        component.do_something()
        
        assert len(received) == 1
        assert received[0] == "did something"


# ============================================================================
# TEST SUITE: Integration with EventBus
# ============================================================================

class TestEventEmitterIntegration:
    """Tests de integración con EventBus runtime."""
    
    def test_emitter_uses_eventbus_internally(self, emitter):
        """Test que EventEmitterBase usa EventBus internamente."""
        assert isinstance(emitter._bus, EventBus)
    
    def test_subscription_object_compatibility(self, emitter):
        """Test que Subscription es compatible con EventBus."""
        sub = emitter.on("test", lambda e: None)
        
        # Debe ser el mismo Subscription de EventBus
        assert isinstance(sub, Subscription)
        assert hasattr(sub, 'unsubscribe')
        assert hasattr(sub, 'event_type')
        assert hasattr(sub, 'listener')
    
    def test_event_object_structure(self, emitter):
        """Test que Event object tiene estructura esperada."""
        received_event = None
        
        def handler(event):
            nonlocal received_event
            received_event = event
        
        emitter.on("test", handler)
        emitter.emit("test", {"data": "value"})
        
        assert received_event is not None
        assert isinstance(received_event, Event)
        assert received_event.type == "test"
        assert received_event.payload == {"data": "value"}
        assert hasattr(received_event, 'timestamp')
        assert hasattr(received_event, 'target')


# ============================================================================
# TEST SUITE: Real-World Usage Patterns
# ============================================================================

class TestRealWorldUsagePatterns:
    """Tests de patrones de uso del mundo real."""
    
    def test_user_service_pattern(self):
        """Test patrón de servicio que emite eventos."""
        
        class User:
            def __init__(self, name):
                self.name = name
        
        class UserService(EventEmitterBase):
            """Service que emite eventos de lifecycle."""
            
            def create_user(self, name):
                user = User(name)
                self.emit("user.created", user)
                return user
            
            def delete_user(self, user):
                self.emit("user.deleted", user)
        
        service = UserService()
        created_users = []
        deleted_users = []
        
        service.on("user.created", lambda e: created_users.append(e.payload))
        service.on("user.deleted", lambda e: deleted_users.append(e.payload))
        
        user1 = service.create_user("Alice")
        user2 = service.create_user("Bob")
        service.delete_user(user1)
        
        assert len(created_users) == 2
        assert created_users[0].name == "Alice"
        assert created_users[1].name == "Bob"
        
        assert len(deleted_users) == 1
        assert deleted_users[0].name == "Alice"
    
    def test_component_lifecycle_pattern(self):
        """Test patrón de lifecycle de componente."""
        
        class Component(EventEmitterBase):
            """Component con lifecycle events."""
            
            def __init__(self):
                super().__init__()
                self.mounted = False
            
            def mount(self):
                self.mounted = True
                self.emit("mount", {"component": self})
            
            def unmount(self):
                self.mounted = False
                self.emit("unmount", {"component": self})
                self.remove_all_listeners()
        
        component = Component()
        lifecycle_events = []
        
        component.on("mount", lambda e: lifecycle_events.append("mounted"))
        component.on("unmount", lambda e: lifecycle_events.append("unmounted"))
        
        component.mount()
        component.unmount()
        
        assert lifecycle_events == ["mounted", "unmounted"]
        assert component.listener_count("mount") == 0
        assert component.listener_count("unmount") == 0
    
    def test_observer_pattern(self):
        """Test patrón observer usando EventEmitter."""
        
        class DataStore(EventEmitterBase):
            """Store que notifica cambios."""
            
            def __init__(self):
                super().__init__()
                self._data = {}
            
            def set(self, key, value):
                old_value = self._data.get(key)
                self._data[key] = value
                self.emit("change", {
                    "key": key,
                    "value": value,
                    "old_value": old_value
                })
            
            def get(self, key):
                return self._data.get(key)
        
        store = DataStore()
        changes = []
        
        store.on("change", lambda e: changes.append(e.payload))
        
        store.set("name", "Alice")
        store.set("age", 30)
        store.set("name", "Bob")
        
        assert len(changes) == 3
        assert changes[0]["key"] == "name"
        assert changes[0]["value"] == "Alice"
        assert changes[2]["old_value"] == "Alice"


# ============================================================================
# TEST SUMMARY
# ============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
