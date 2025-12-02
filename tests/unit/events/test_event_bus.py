"""
Tests unitarios para EventBus<T> core

TASK-035L: Implementar EventBus<T> core
Sprint: 14
Historia: VELA-575
"""

import pytest
import threading
import time
from dataclasses import dataclass
from typing import List

from src.runtime.events import (
    Event,
    EventBus,
    AutoDisposeEventBus,
    Subscription,
    get_global_bus,
)


# Test data classes
@dataclass
class User:
    name: str
    email: str


@dataclass
class Message:
    content: str
    sender: str


class TestEvent:
    """Tests para Event class."""
    
    def test_event_creation(self):
        """Test crear event con payload."""
        user = User("Alice", "alice@example.com")
        event = Event("user.created", user)
        
        assert event.type == "user.created"
        assert event.payload == user
        assert event.payload.name == "Alice"
        assert event.timestamp > 0
        assert event.target is None
        assert not event.propagation_stopped
        assert not event.default_prevented
    
    def test_event_with_tags(self):
        """Test event con tags para filtering."""
        user = User("Bob", "bob@example.com")
        event = Event("user.created", user, tags=["audit", "important"])
        
        assert "audit" in event.tags
        assert "important" in event.tags
        assert len(event.tags) == 2
    
    def test_stop_propagation(self):
        """Test stop_propagation."""
        event = Event("test", "data")
        assert not event.propagation_stopped
        
        event.stop_propagation()
        assert event.propagation_stopped
    
    def test_prevent_default(self):
        """Test prevent_default."""
        event = Event("test", "data")
        assert not event.default_prevented
        
        event.prevent_default()
        assert event.default_prevented


class TestSubscription:
    """Tests para Subscription class."""
    
    def test_subscription_creation(self):
        """Test crear subscription."""
        bus = EventBus[str]()
        handler = lambda e: None
        
        subscription = Subscription("test", handler, bus)
        
        assert subscription.event_type == "test"
        assert subscription.listener == handler
        assert subscription.bus == bus
        assert not subscription.disposed
    
    def test_subscription_unsubscribe(self):
        """Test unsubscribe manual."""
        bus = EventBus[str]()
        called = []
        
        def handler(event):
            called.append(event.payload)
        
        subscription = bus.on("test", handler)
        
        bus.emit("test", "first")
        assert len(called) == 1
        
        subscription.unsubscribe()
        assert subscription.disposed
        
        bus.emit("test", "second")
        assert len(called) == 1  # No llamado después de unsubscribe
    
    def test_subscription_context_manager(self):
        """Test subscription como context manager."""
        bus = EventBus[str]()
        called = []
        
        def handler(event):
            called.append(event.payload)
        
        with bus.on("test", handler) as subscription:
            bus.emit("test", "inside")
            assert len(called) == 1
        
        # Auto-unsubscribed después de context exit
        bus.emit("test", "outside")
        assert len(called) == 1


class TestEventBus:
    """Tests para EventBus core."""
    
    def test_eventbus_creation(self):
        """Test crear EventBus vacío."""
        bus = EventBus[User]()
        
        assert bus.listener_count("user.created") == 0
        assert bus.event_types() == []
    
    def test_on_adds_listener(self):
        """Test on() agrega listener."""
        bus = EventBus[User]()
        handler = lambda e: None
        
        subscription = bus.on("user.created", handler)
        
        assert isinstance(subscription, Subscription)
        assert bus.listener_count("user.created") == 1
        assert "user.created" in bus.event_types()
    
    def test_emit_calls_listener(self):
        """Test emit() llama listener."""
        bus = EventBus[User]()
        called = []
        
        def handler(event: Event[User]):
            called.append(event.payload)
        
        bus.on("user.created", handler)
        user = User("Alice", "alice@example.com")
        
        bus.emit("user.created", user)
        
        assert len(called) == 1
        assert called[0] == user
        assert called[0].name == "Alice"
    
    def test_emit_no_listeners(self):
        """Test emit sin listeners no crashea."""
        bus = EventBus[str]()
        
        # No debe crashear
        bus.emit("nonexistent", "data")
    
    def test_multiple_listeners(self):
        """Test múltiples listeners para mismo evento."""
        bus = EventBus[str]()
        calls = []
        
        bus.on("test", lambda e: calls.append("first"))
        bus.on("test", lambda e: calls.append("second"))
        bus.on("test", lambda e: calls.append("third"))
        
        assert bus.listener_count("test") == 3
        
        bus.emit("test", "data")
        
        assert calls == ["first", "second", "third"]
    
    def test_off_removes_listener(self):
        """Test off() remueve listener."""
        bus = EventBus[str]()
        called = []
        
        def handler(event):
            called.append(event.payload)
        
        bus.on("test", handler)
        bus.emit("test", "first")
        assert len(called) == 1
        
        bus.off("test", handler)
        assert bus.listener_count("test") == 0
        
        bus.emit("test", "second")
        assert len(called) == 1  # No llamado después de off
    
    def test_off_nonexistent_listener(self):
        """Test off() con listener inexistente no crashea."""
        bus = EventBus[str]()
        
        # No debe crashear
        bus.off("test", lambda e: None)
    
    def test_once_auto_unsubscribes(self):
        """Test once() se auto-desuscribe después de primer evento."""
        bus = EventBus[str]()
        called = []
        
        def handler(event):
            called.append(event.payload)
        
        bus.once("test", handler)
        
        bus.emit("test", "first")
        assert len(called) == 1
        
        bus.emit("test", "second")
        assert len(called) == 1  # No llamado segunda vez
        
        # Listener fue removido
        assert bus.listener_count("test") == 0
    
    def test_clear_specific_event(self):
        """Test clear() remueve listeners de evento específico."""
        bus = EventBus[str]()
        
        bus.on("event1", lambda e: None)
        bus.on("event1", lambda e: None)
        bus.on("event2", lambda e: None)
        
        assert bus.listener_count("event1") == 2
        assert bus.listener_count("event2") == 1
        
        bus.clear("event1")
        
        assert bus.listener_count("event1") == 0
        assert bus.listener_count("event2") == 1  # No afectado
    
    def test_clear_all_events(self):
        """Test clear() sin argumentos remueve todos los listeners."""
        bus = EventBus[str]()
        
        bus.on("event1", lambda e: None)
        bus.on("event2", lambda e: None)
        bus.on("event3", lambda e: None)
        
        assert len(bus.event_types()) == 3
        
        bus.clear()
        
        assert len(bus.event_types()) == 0
        assert bus.listener_count("event1") == 0
    
    def test_error_isolation(self):
        """Test error en un listener no afecta otros."""
        bus = EventBus[str]()
        called = []
        
        def failing_handler(event):
            raise ValueError("Test error")
        
        def working_handler(event):
            called.append(event.payload)
        
        bus.on("test", failing_handler)
        bus.on("test", working_handler)
        
        # No debe crashear
        bus.emit("test", "data")
        
        # working_handler debe haber sido llamado
        assert len(called) == 1
        assert called[0] == "data"
    
    def test_type_safety(self):
        """Test type safety con generic EventBus."""
        # EventBus<User> solo acepta User payloads
        bus = EventBus[User]()
        received_users: List[User] = []
        
        def handler(event: Event[User]):
            # Type checker garantiza que event.payload es User
            received_users.append(event.payload)
            assert hasattr(event.payload, 'name')
            assert hasattr(event.payload, 'email')
        
        bus.on("user.created", handler)
        
        user = User("Alice", "alice@example.com")
        bus.emit("user.created", user)
        
        assert len(received_users) == 1
        assert received_users[0].name == "Alice"


class TestThreadSafety:
    """Tests de thread safety."""
    
    def test_concurrent_emit(self):
        """Test emit desde múltiples threads."""
        bus = EventBus[int]()
        results = []
        lock = threading.Lock()
        
        def handler(event: Event[int]):
            with lock:
                results.append(event.payload)
        
        bus.on("number", handler)
        
        # Emit desde múltiples threads
        threads = []
        for i in range(10):
            thread = threading.Thread(target=lambda n=i: bus.emit("number", n))
            threads.append(thread)
            thread.start()
        
        for thread in threads:
            thread.join()
        
        # Todos los eventos deben ser recibidos
        assert len(results) == 10
        assert set(results) == set(range(10))
    
    def test_concurrent_subscribe(self):
        """Test subscribe desde múltiples threads."""
        bus = EventBus[str]()
        subscriptions = []
        
        def subscribe_worker():
            sub = bus.on("test", lambda e: None)
            subscriptions.append(sub)
        
        threads = []
        for _ in range(10):
            thread = threading.Thread(target=subscribe_worker)
            threads.append(thread)
            thread.start()
        
        for thread in threads:
            thread.join()
        
        assert bus.listener_count("test") == 10
        assert len(subscriptions) == 10


class TestAutoDisposeEventBus:
    """Tests para AutoDisposeEventBus."""
    
    def test_autodispose_creation(self):
        """Test crear AutoDisposeEventBus."""
        bus = AutoDisposeEventBus[str]()
        
        assert isinstance(bus, EventBus)
        assert bus.listener_count("test") == 0
    
    def test_on_with_owner(self):
        """Test on() con owner tracking."""
        bus = AutoDisposeEventBus[str]()
        
        class Component:
            pass
        
        component = Component()
        called = []
        
        bus.on("test", lambda e: called.append(e.payload), owner=component)
        
        bus.emit("test", "first")
        assert len(called) == 1
    
    def test_dispose_all(self):
        """Test dispose_all() remueve todas las subscriptions del owner."""
        bus = AutoDisposeEventBus[str]()
        
        class Component:
            pass
        
        component = Component()
        called = []
        
        # Registrar múltiples subscriptions
        bus.on("event1", lambda e: called.append("e1"), owner=component)
        bus.on("event2", lambda e: called.append("e2"), owner=component)
        bus.on("event3", lambda e: called.append("e3"), owner=component)
        
        # Emitir antes de dispose
        bus.emit("event1", "")
        bus.emit("event2", "")
        bus.emit("event3", "")
        assert len(called) == 3
        
        # Dispose all
        bus.dispose_all(component)
        
        # Emitir después de dispose
        bus.emit("event1", "")
        bus.emit("event2", "")
        bus.emit("event3", "")
        assert len(called) == 3  # No new calls
    
    def test_multiple_owners(self):
        """Test múltiples owners independientes."""
        bus = AutoDisposeEventBus[str]()
        
        class ComponentA:
            pass
        
        class ComponentB:
            pass
        
        comp_a = ComponentA()
        comp_b = ComponentB()
        
        calls_a = []
        calls_b = []
        
        bus.on("test", lambda e: calls_a.append(e.payload), owner=comp_a)
        bus.on("test", lambda e: calls_b.append(e.payload), owner=comp_b)
        
        bus.emit("test", "first")
        assert len(calls_a) == 1
        assert len(calls_b) == 1
        
        # Dispose solo comp_a
        bus.dispose_all(comp_a)
        
        bus.emit("test", "second")
        assert len(calls_a) == 1  # No más llamados
        assert len(calls_b) == 2  # Todavía recibe eventos


class TestGlobalBus:
    """Tests para global event bus."""
    
    def test_get_global_bus(self):
        """Test get_global_bus() retorna singleton."""
        bus1 = get_global_bus()
        bus2 = get_global_bus()
        
        assert bus1 is bus2  # Same instance
    
    def test_global_bus_works(self):
        """Test global bus funciona correctamente."""
        bus = get_global_bus()
        called = []
        
        # Limpiar listeners previos
        bus.clear()
        
        bus.on("app.started", lambda e: called.append(e.payload))
        bus.emit("app.started", {"version": "1.0.0"})
        
        assert len(called) == 1
        assert called[0]["version"] == "1.0.0"
        
        # Cleanup
        bus.clear()


class TestEdgeCases:
    """Tests de edge cases."""
    
    def test_emit_during_emit(self):
        """Test emit evento durante handling de otro evento."""
        bus = EventBus[str]()
        calls = []
        
        def handler1(event):
            calls.append("handler1")
            # Emit otro evento durante handling
            bus.emit("event2", "nested")
        
        def handler2(event):
            calls.append("handler2")
        
        bus.on("event1", handler1)
        bus.on("event2", handler2)
        
        bus.emit("event1", "trigger")
        
        assert calls == ["handler1", "handler2"]
    
    def test_unsubscribe_during_emit(self):
        """Test unsubscribe durante emit."""
        bus = EventBus[str]()
        calls = []
        subscription = None
        
        def handler(event):
            calls.append("called")
            if subscription:
                subscription.unsubscribe()
        
        subscription = bus.on("test", handler)
        
        bus.emit("test", "first")
        assert len(calls) == 1
        
        bus.emit("test", "second")
        assert len(calls) == 1  # No llamado segunda vez
    
    def test_many_listeners(self):
        """Test performance con muchos listeners."""
        bus = EventBus[int]()
        count = 1000
        calls = []
        
        for i in range(count):
            bus.on("test", lambda e, n=i: calls.append(n))
        
        assert bus.listener_count("test") == count
        
        start = time.time()
        bus.emit("test", 42)
        duration = time.time() - start
        
        assert len(calls) == count
        assert duration < 1.0  # Debe ser rápido (<1s para 1000 listeners)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
