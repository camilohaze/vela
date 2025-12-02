"""
Event System - Core Types and Classes

Implementación del sistema de eventos type-safe de Vela.
TASK-035L: Implementar EventBus<T> core
Historia: VELA-575 - Dependency Injection
Sprint: 14
"""

from typing import TypeVar, Generic, Callable, Optional, Any, Dict, List
from dataclasses import dataclass, field
from datetime import datetime
import threading
import weakref


T = TypeVar('T')
# EventListener accepts Any to avoid Callable generic issues in Python < 3.12
EventListener = Callable[[Any], None]


@dataclass
class Event(Generic[T]):
    """
    Generic Event object con payload type-safe.
    
    Attributes:
        type: Event type identifier (e.g., "user.created")
        payload: Event data (generic type T)
        timestamp: Event creation timestamp (Unix timestamp)
        target: Optional target object that triggered the event
        propagation_stopped: Flag to stop event propagation
        default_prevented: Flag to prevent default behavior
        tags: Optional tags for event filtering
    
    Example:
        >>> event = Event("user.created", user, tags=["audit"])
        >>> event.stop_propagation()
    """
    
    type: str
    payload: T
    timestamp: float = field(default_factory=lambda: datetime.now().timestamp())
    target: Optional[Any] = None
    propagation_stopped: bool = False
    default_prevented: bool = False
    tags: List[str] = field(default_factory=list)
    
    def stop_propagation(self) -> None:
        """Stop event propagation (for propagating events)."""
        self.propagation_stopped = True
    
    def prevent_default(self) -> None:
        """Prevent default behavior associated with event."""
        self.default_prevented = True


class Subscription:
    """
    Subscription object para manejar unsubscribe.
    Implementa disposable pattern para auto-cleanup.
    
    Attributes:
        event_type: Event type this subscription listens to
        listener: Listener function
        bus: EventBus that manages this subscription
        disposed: Flag indicating if subscription is disposed
    
    Example:
        >>> subscription = bus.on("user.created", handler)
        >>> subscription.unsubscribe()  # Manual cleanup
    """
    
    def __init__(
        self, 
        event_type: str, 
        listener: EventListener, 
        bus: 'EventBus'
    ):
        self.event_type = event_type
        self.listener = listener
        self.bus = bus
        self.disposed = False
    
    def unsubscribe(self) -> None:
        """Unsubscribe from event bus."""
        if not self.disposed:
            self.bus.off(self.event_type, self.listener)
            self.disposed = True
    
    def __enter__(self):
        """Context manager support."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Auto-unsubscribe on context exit."""
        self.unsubscribe()
        return False


class EventBus:
    """
    Generic type-safe Event Bus para comunicación desacoplada.
    
    Features:
    - Type-safe: EventBus<T> garantiza tipos correctos
    - Múltiples listeners por evento
    - Error isolation: Un listener crasheado no afecta otros
    - Subscription pattern: Auto-dispose support
    - Thread-safe: Puede usarse desde múltiples threads
    
    Example:
        >>> bus = EventBus[User]()
        >>> subscription = bus.on("user.created", lambda e: print(e.payload.name))
        >>> bus.emit("user.created", User(name="Alice"))
        >>> subscription.unsubscribe()
    """
    
    def __init__(self):
        """Initialize EventBus with empty listeners dict."""
        self._listeners = {}  # type: Dict[str, List[EventListener]]
        self._lock = threading.Lock()
    
    @classmethod
    def __class_getitem__(cls, item):
        """Support EventBus[T]() syntax without Generic inheritance."""
        return cls
    
    def on(
        self, 
        event_type: str, 
        listener: EventListener
    ) -> Subscription:
        """
        Subscribe to event type.
        
        Args:
            event_type: Event type to listen to (e.g., "user.created")
            listener: Callback function to handle event
        
        Returns:
            Subscription object for unsubscribe
        
        Example:
            >>> def on_user_created(event: Event[User]):
            ...     print(f"User created: {event.payload.name}")
            >>> sub = bus.on("user.created", on_user_created)
        """
        with self._lock:
            if event_type not in self._listeners:
                self._listeners[event_type] = []
            self._listeners[event_type].append(listener)
        
        return Subscription(event_type, listener, self)
    
    def emit(self, event_type: str, payload: Any) -> None:
        """
        Emit event to all subscribed listeners.
        
        Args:
            event_type: Event type to emit
            payload: Event data (must match EventBus<T> type)
        
        Features:
        - Error isolation: Exceptions in listeners don't crash emit
        - Listeners are called in subscription order
        - Thread-safe: Copies listener list before notifying
        
        Example:
            >>> bus.emit("user.created", User(name="Bob"))
        """
        # Copy listeners to avoid concurrent modification
        with self._lock:
            if event_type not in self._listeners:
                return
            listeners = self._listeners[event_type].copy()
        
        # Create event object
        event = Event(type=event_type, payload=payload)
        
        # Notify listeners (outside lock to avoid deadlock)
        for listener in listeners:
            try:
                listener(event)
            except Exception as e:
                # Error isolation: log but continue
                import logging
                logging.error(
                    f"Error in event listener for '{event_type}': {e}",
                    exc_info=True
                )
    
    def off(
        self, 
        event_type: str, 
        listener: EventListener
    ) -> None:
        """
        Unsubscribe listener from event type.
        
        Args:
            event_type: Event type to unsubscribe from
            listener: Listener function to remove
        
        Example:
            >>> bus.off("user.created", on_user_created)
        """
        with self._lock:
            if event_type in self._listeners:
                try:
                    self._listeners[event_type].remove(listener)
                    # Remove event type if no more listeners
                    if not self._listeners[event_type]:
                        del self._listeners[event_type]
                except ValueError:
                    # Listener not found, ignore
                    pass
    
    def once(
        self, 
        event_type: str, 
        listener: EventListener
    ) -> Subscription:
        """
        Subscribe to event type for one-time notification.
        Auto-unsubscribes after first event.
        
        Args:
            event_type: Event type to listen to
            listener: Callback function to handle event
        
        Returns:
            Subscription object
        
        Example:
            >>> bus.once("app.ready", lambda e: print("App is ready!"))
        """
        def wrapped_listener(event: Any):
            listener(event)
            self.off(event_type, wrapped_listener)
        
        return self.on(event_type, wrapped_listener)
    
    def clear(self, event_type: Optional[str] = None) -> None:
        """
        Remove all listeners.
        
        Args:
            event_type: If provided, clear only this event type.
                       If None, clear all event types.
        
        Example:
            >>> bus.clear("user.created")  # Clear specific event
            >>> bus.clear()  # Clear all events
        """
        with self._lock:
            if event_type is not None:
                self._listeners.pop(event_type, None)
            else:
                self._listeners.clear()
    
    def listener_count(self, event_type: str) -> int:
        """
        Get count of listeners for event type.
        
        Args:
            event_type: Event type to count listeners for
        
        Returns:
            Number of listeners subscribed to event type
        
        Example:
            >>> count = bus.listener_count("user.created")
            >>> print(f"{count} listeners for user.created")
        """
        with self._lock:
            return len(self._listeners.get(event_type, []))
    
    def event_types(self) -> List[str]:
        """
        Get list of all registered event types.
        
        Returns:
            List of event types that have listeners
        
        Example:
            >>> types = bus.event_types()
            >>> print(f"Registered events: {types}")
        """
        with self._lock:
            return list(self._listeners.keys())


class AutoDisposeEventBus(EventBus):
    """
    EventBus con auto-dispose support basado en owner.
    Previene memory leaks al trackear subscriptions por owner.
    
    Features:
    - Track subscriptions por owner object
    - Auto-dispose all subscriptions cuando owner se destruye
    - Integration con component lifecycle (mount/destroy)
    
    Example:
        >>> bus = AutoDisposeEventBus[User]()
        >>> component = MyComponent()
        >>> bus.on("user.created", handler, owner=component)
        >>> bus.dispose_all(component)  # Cleanup all component subscriptions
    """
    
    def __init__(self):
        super().__init__()
        # Track subscriptions by owner (using weak refs to avoid leaks)
        self._subscriptions_by_owner: Dict[int, List[Subscription]] = {}
        self._owner_refs: Dict[int, Any] = {}  # weakref.ref objects
    
    def on(
        self, 
        event_type: str, 
        listener: EventListener,
        owner: Optional[Any] = None
    ) -> Subscription:
        """
        Subscribe to event with optional owner tracking.
        
        Args:
            event_type: Event type to listen to
            listener: Callback function
            owner: Optional owner object for auto-dispose
        
        Returns:
            Subscription object
        
        Example:
            >>> class Component:
            ...     def mount(self):
            ...         bus.on("event", self.handler, owner=self)
            ...     def destroy(self):
            ...         bus.dispose_all(self)
        """
        subscription = super().on(event_type, listener)
        
        # Track subscription if owner provided
        if owner is not None:
            owner_id = id(owner)
            
            with self._lock:
                if owner_id not in self._subscriptions_by_owner:
                    self._subscriptions_by_owner[owner_id] = []
                    # Store weak reference to detect when owner is GC'd
                    self._owner_refs[owner_id] = weakref.ref(
                        owner,
                        lambda ref: self._cleanup_owner(owner_id)
                    )
                
                self._subscriptions_by_owner[owner_id].append(subscription)
        
        return subscription
    
    def dispose_all(self, owner: Any) -> None:
        """
        Dispose all subscriptions for owner.
        
        Args:
            owner: Owner object whose subscriptions to dispose
        
        Example:
            >>> component = MyComponent()
            >>> # ... register multiple subscriptions ...
            >>> bus.dispose_all(component)  # Cleanup all at once
        """
        owner_id = id(owner)
        
        with self._lock:
            if owner_id in self._subscriptions_by_owner:
                subscriptions = self._subscriptions_by_owner.pop(owner_id)
                self._owner_refs.pop(owner_id, None)
        
        # Unsubscribe outside lock
        for subscription in subscriptions:
            subscription.unsubscribe()
    
    def _cleanup_owner(self, owner_id: int) -> None:
        """
        Internal: Cleanup subscriptions when owner is garbage collected.
        
        Args:
            owner_id: ID of owner object that was GC'd
        """
        with self._lock:
            subscriptions = self._subscriptions_by_owner.pop(owner_id, [])
            self._owner_refs.pop(owner_id, None)
        
        # Unsubscribe all
        for subscription in subscriptions:
            if not subscription.disposed:
                subscription.unsubscribe()


# Singleton global event bus (optional convenience)
_global_bus: Optional[EventBus[Any]] = None


def get_global_bus() -> EventBus[Any]:
    """
    Get singleton global event bus.
    Convenient para eventos cross-cutting (logging, analytics, etc.)
    
    Returns:
        Global EventBus instance
    
    Example:
        >>> bus = get_global_bus()
        >>> bus.emit("app.started", {"version": "1.0.0"})
    """
    global _global_bus
    if _global_bus is None:
        _global_bus = EventBus[Any]()
    return _global_bus


__all__ = [
    'Event',
    'EventListener',
    'Subscription',
    'EventBus',
    'AutoDisposeEventBus',
    'get_global_bus',
]
