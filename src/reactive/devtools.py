"""
DevTools Integration for State Management

Provides integration with browser DevTools extensions (Redux DevTools Protocol)
for state inspection, time-travel debugging, and action replay.

Features:
- State history tracking
- Action history with timestamps
- Time-travel debugging (jump to any state)
- Action replay
- State diff visualization
- Export/import state snapshots
- WebSocket connection to browser extension
"""

import json
import time
from typing import Any, Dict, List, Optional, Callable
from dataclasses import dataclass, field
from datetime import datetime


# =====================================================
# DATA STRUCTURES
# =====================================================

@dataclass
class ActionRecord:
    """Record of a dispatched action"""
    action_type: str
    payload: Any
    timestamp: float
    state_before: Dict[str, Any]
    state_after: Dict[str, Any]
    id: int
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization"""
        return {
            "type": action_type,
            "payload": self.payload,
            "timestamp": self.timestamp,
            "id": self.id
        }


@dataclass
class StateSnapshot:
    """Snapshot of state at a point in time"""
    state: Dict[str, Any]
    action_id: int
    timestamp: float
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary"""
        return {
            "state": self.state,
            "actionId": self.action_id,
            "timestamp": self.timestamp
        }


@dataclass
class DevToolsConfig:
    """Configuration for DevTools"""
    name: str = "Vela Store"
    max_history: int = 50
    serialize: Callable[[Any], str] = lambda x: json.dumps(x, default=str)
    deserialize: Callable[[str], Any] = json.loads
    latency: int = 0  # Simulated network latency (ms)
    features: Dict[str, bool] = field(default_factory=lambda: {
        "jump": True,        # Jump to any action
        "skip": True,        # Skip actions
        "reorder": False,    # Reorder actions (not implemented)
        "import": True,      # Import state
        "export": True,      # Export state
        "persist": False     # Persist across sessions (not implemented)
    })


# =====================================================
# DEVTOOLS EXTENSION
# =====================================================

class DevToolsExtension:
    """
    DevTools extension for state management debugging.
    
    Compatible with Redux DevTools Protocol.
    """
    
    def __init__(self, config: Optional[DevToolsConfig] = None):
        """
        Initialize DevTools extension.
        
        Args:
            config: DevTools configuration
        """
        self.config = config or DevToolsConfig()
        self.history: List[ActionRecord] = []
        self.current_index: int = -1
        self.is_enabled: bool = True
        self.is_time_traveling: bool = False
        self.store: Optional[Any] = None
        self.next_action_id: int = 1
        
        # Listeners
        self.listeners: List[Callable[[str, Any], None]] = []
    
    def connect(self, store: Any) -> None:
        """
        Connect to a store.
        
        Args:
            store: Store instance to monitor
        """
        self.store = store
        self._send_event("INIT", {"state": store.get_state()})
    
    def disconnect(self) -> None:
        """Disconnect from store"""
        self.store = None
        self.history.clear()
        self.current_index = -1
    
    def record_action(
        self,
        action_type: str,
        payload: Any,
        state_before: Dict[str, Any],
        state_after: Dict[str, Any]
    ) -> None:
        """
        Record an action dispatch.
        
        Args:
            action_type: Type of action
            payload: Action payload
            state_before: State before action
            state_after: State after action
        """
        if not self.is_enabled or self.is_time_traveling:
            return
        
        # Create record
        record = ActionRecord(
            action_type=action_type,
            payload=payload,
            timestamp=time.time(),
            state_before=state_before,
            state_after=state_after,
            id=self.next_action_id
        )
        self.next_action_id += 1
        
        # Add to history
        self.history.append(record)
        
        # Trim history if needed
        if len(self.history) > self.config.max_history:
            self.history.pop(0)
        
        # Update current index
        self.current_index = len(self.history) - 1
        
        # Send to extension
        self._send_event("ACTION", {
            "type": action_type,
            "payload": payload,
            "state": state_after,
            "id": record.id
        })
    
    def jump_to_action(self, action_id: int) -> bool:
        """
        Jump to a specific action (time-travel).
        
        Args:
            action_id: ID of action to jump to
            
        Returns:
            True if jump was successful
        """
        if not self.store:
            return False
        
        # Find action index
        index = -1
        for i, record in enumerate(self.history):
            if record.id == action_id:
                index = i
                break
        
        if index == -1:
            return False
        
        # Set time-traveling flag
        self.is_time_traveling = True
        
        # Get state at that point
        target_state = self.history[index].state_after
        
        # Update store state directly
        self.store.state = target_state.copy()
        
        # Notify subscribers
        self.store._notify_subscribers()
        
        # Update current index
        self.current_index = index
        
        # Reset flag
        self.is_time_traveling = False
        
        # Send event
        self._send_event("JUMP", {
            "actionId": action_id,
            "state": target_state
        })
        
        return True
    
    def skip_action(self, action_id: int) -> bool:
        """
        Skip an action (remove from history and recompute).
        
        Args:
            action_id: ID of action to skip
            
        Returns:
            True if skip was successful
        """
        if not self.store or not self.config.features.get("skip"):
            return False
        
        # Find action
        index = -1
        for i, record in enumerate(self.history):
            if record.id == action_id:
                index = i
                break
        
        if index == -1:
            return False
        
        # Remove from history
        skipped = self.history.pop(index)
        
        # Recompute state from beginning
        self._recompute_from_index(0)
        
        # Send event
        self._send_event("SKIP", {
            "actionId": action_id,
            "state": self.store.get_state()
        })
        
        return True
    
    def reset(self) -> None:
        """Reset to initial state"""
        if not self.store:
            return
        
        self.history.clear()
        self.current_index = -1
        self.next_action_id = 1
        
        # Reset store to initial state
        if hasattr(self.store, '_initial_state'):
            self.store.state = self.store._initial_state.copy()
            self.store._notify_subscribers()
        
        self._send_event("RESET", {"state": self.store.get_state()})
    
    def export_state(self) -> str:
        """
        Export current state and history.
        
        Returns:
            JSON string with state and history
        """
        if not self.config.features.get("export"):
            return "{}"
        
        data = {
            "state": self.store.get_state() if self.store else {},
            "history": [
                {
                    "type": r.action_type,
                    "payload": r.payload,
                    "id": r.id,
                    "timestamp": r.timestamp
                }
                for r in self.history
            ],
            "currentIndex": self.current_index,
            "timestamp": time.time()
        }
        
        return self.config.serialize(data)
    
    def import_state(self, data: str) -> bool:
        """
        Import state and history.
        
        Args:
            data: JSON string with state and history
            
        Returns:
            True if import was successful
        """
        if not self.store or not self.config.features.get("import"):
            return False
        
        try:
            imported = self.config.deserialize(data)
            
            # Restore state
            self.store.state = imported["state"]
            self.store._notify_subscribers()
            
            # Restore history (simplified - without full ActionRecords)
            self.history.clear()
            self.current_index = imported.get("currentIndex", -1)
            
            self._send_event("IMPORT", {"state": imported["state"]})
            
            return True
        except Exception:
            return False
    
    def get_state_diff(self, action_id1: int, action_id2: int) -> Optional[Dict[str, Any]]:
        """
        Get diff between two states.
        
        Args:
            action_id1: First action ID
            action_id2: Second action ID
            
        Returns:
            Dictionary with added, removed, and modified keys
        """
        # Find actions
        state1, state2 = None, None
        
        for record in self.history:
            if record.id == action_id1:
                state1 = record.state_after
            if record.id == action_id2:
                state2 = record.state_after
        
        if not state1 or not state2:
            return None
        
        # Compute diff
        diff = {
            "added": {},
            "removed": {},
            "modified": {}
        }
        
        # Find added and modified
        for key, value in state2.items():
            if key not in state1:
                diff["added"][key] = value
            elif state1[key] != value:
                diff["modified"][key] = {
                    "from": state1[key],
                    "to": value
                }
        
        # Find removed
        for key in state1:
            if key not in state2:
                diff["removed"][key] = state1[key]
        
        return diff
    
    def subscribe(self, listener: Callable[[str, Any], None]) -> Callable[[], None]:
        """
        Subscribe to DevTools events.
        
        Args:
            listener: Function called with (event_type, data)
            
        Returns:
            Unsubscribe function
        """
        self.listeners.append(listener)
        return lambda: self.listeners.remove(listener)
    
    def enable(self) -> None:
        """Enable DevTools recording"""
        self.is_enabled = True
        self._send_event("ENABLE", {})
    
    def disable(self) -> None:
        """Disable DevTools recording"""
        self.is_enabled = False
        self._send_event("DISABLE", {})
    
    def _recompute_from_index(self, start_index: int) -> None:
        """
        Recompute state from a specific index.
        
        Args:
            start_index: Index to start recomputing from
        """
        if not self.store or start_index >= len(self.history):
            return
        
        # Get initial state
        if start_index == 0:
            if hasattr(self.store, '_initial_state'):
                current_state = self.store._initial_state.copy()
            else:
                current_state = {}
        else:
            current_state = self.history[start_index - 1].state_after.copy()
        
        # Replay actions from start_index
        self.is_time_traveling = True
        
        for i in range(start_index, len(self.history)):
            record = self.history[i]
            
            # Create action object
            class Action:
                def __init__(self, action_type, payload):
                    self.type = action_type
                    self.payload = payload
            
            action = Action(record.action_type, record.payload)
            
            # Apply reducer
            current_state = self.store.reducer(current_state, action)
            
            # Update record
            record.state_after = current_state
        
        # Update store
        self.store.state = current_state
        self.store._notify_subscribers()
        
        self.is_time_traveling = False
    
    def _send_event(self, event_type: str, data: Any) -> None:
        """
        Send event to listeners.
        
        Args:
            event_type: Type of event
            data: Event data
        """
        for listener in self.listeners:
            try:
                listener(event_type, data)
            except Exception:
                pass  # Ignore listener errors


# =====================================================
# HELPER FUNCTIONS
# =====================================================

def create_devtools_middleware(devtools: DevToolsExtension):
    """
    Create middleware that records actions in DevTools.
    
    Args:
        devtools: DevTools extension instance
        
    Returns:
        Middleware instance
    """
    class DevToolsMiddleware:
        def __init__(self):
            self.devtools = devtools
        
        def handle(self, context, next_func, action):
            """Handle action"""
            state_before = context.get_state()
            
            # Let action pass through
            next_func(action)
            
            state_after = context.get_state()
            
            # Record in DevTools
            self.devtools.record_action(
                action_type=action.type,
                payload=action.payload if hasattr(action, 'payload') else None,
                state_before=state_before,
                state_after=state_after
            )
    
    return DevToolsMiddleware()


def connect_devtools(store: Any, config: Optional[DevToolsConfig] = None) -> DevToolsExtension:
    """
    Connect DevTools to a store.
    
    Args:
        store: Store instance
        config: DevTools configuration
        
    Returns:
        DevTools extension instance
    """
    devtools = DevToolsExtension(config)
    devtools.connect(store)
    
    # Store initial state
    store._initial_state = store.get_state().copy()
    
    return devtools


# =====================================================
# DECORATOR
# =====================================================

def devtools(config: Optional[DevToolsConfig] = None):
    """
    Decorator to enable DevTools on a Store.
    
    Usage:
        @devtools()
        class MyStore(Store):
            pass
    """
    def decorator(store_class):
        class DevToolsStore(store_class):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                
                # Create DevTools extension
                self._devtools = DevToolsExtension(config)
                self._devtools.connect(self)
                
                # Store initial state
                self._initial_state = self.get_state().copy()
                
                # Add DevTools middleware
                devtools_middleware = create_devtools_middleware(self._devtools)
                self.middlewares.insert(0, devtools_middleware)
            
            def get_devtools(self) -> DevToolsExtension:
                """Get DevTools extension"""
                return self._devtools
        
        return DevToolsStore
    return decorator
