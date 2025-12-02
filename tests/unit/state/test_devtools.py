"""
Tests for DevTools Extension

Test coverage:
- DevToolsExtension initialization and configuration
- State history tracking
- Action recording with timestamps
- Time-travel debugging (jump to action)
- Action skipping and recomputation
- State diff calculation
- Export/import state
- Event system (subscribe/notify)
- Integration with Store and middleware
- Edge cases and error handling
"""

import pytest
import time
from src.reactive.devtools import (
    DevToolsExtension,
    DevToolsConfig,
    ActionRecord,
    StateSnapshot,
    create_devtools_middleware,
    connect_devtools,
    devtools
)


# =====================================================
# MOCK STORE
# =====================================================

class MockStore:
    """Mock Store for testing"""
    
    def __init__(self, initial_state=None, reducer=None):
        self.state = initial_state or {}
        self.reducer = reducer or (lambda s, a: s)
        self.subscribers = []
        self.middlewares = []
        self._initial_state = self.state.copy()
    
    def get_state(self):
        """Get current state"""
        return self.state.copy()
    
    def dispatch(self, action):
        """Dispatch an action"""
        self.state = self.reducer(self.state, action)
        self._notify_subscribers()
    
    def subscribe(self, callback):
        """Subscribe to state changes"""
        self.subscribers.append(callback)
        return lambda: self.subscribers.remove(callback)
    
    def _notify_subscribers(self):
        """Notify all subscribers"""
        for subscriber in self.subscribers:
            subscriber()


class MockAction:
    """Mock Action"""
    def __init__(self, action_type, payload=None):
        self.type = action_type
        self.payload = payload


# =====================================================
# TEST: DEVTOOLS EXTENSION INITIALIZATION
# =====================================================

class TestDevToolsExtensionInitialization:
    """Test DevTools initialization and configuration"""
    
    def test_default_initialization(self):
        """Test default initialization"""
        devtools = DevToolsExtension()
        
        assert devtools.config.name == "Vela Store"
        assert devtools.config.max_history == 50
        assert devtools.is_enabled == True
        assert devtools.is_time_traveling == False
        assert devtools.store is None
        assert len(devtools.history) == 0
        assert devtools.current_index == -1
    
    def test_custom_config(self):
        """Test custom configuration"""
        config = DevToolsConfig(
            name="MyStore",
            max_history=100,
            latency=10
        )
        devtools = DevToolsExtension(config)
        
        assert devtools.config.name == "MyStore"
        assert devtools.config.max_history == 100
        assert devtools.config.latency == 10
    
    def test_features_configuration(self):
        """Test features configuration"""
        config = DevToolsConfig(
            features={
                "jump": True,
                "skip": False,
                "import": True,
                "export": False
            }
        )
        devtools = DevToolsExtension(config)
        
        assert devtools.config.features["jump"] == True
        assert devtools.config.features["skip"] == False
        assert devtools.config.features["import"] == True
        assert devtools.config.features["export"] == False


# =====================================================
# TEST: STORE CONNECTION
# =====================================================

class TestStoreConnection:
    """Test connecting and disconnecting from store"""
    
    def test_connect_to_store(self):
        """Test connecting to a store"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        
        devtools.connect(store)
        
        assert devtools.store is store
    
    def test_disconnect_from_store(self):
        """Test disconnecting from store"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        
        devtools.connect(store)
        devtools.record_action("INCREMENT", None, {"count": 0}, {"count": 1})
        
        assert len(devtools.history) == 1
        
        devtools.disconnect()
        
        assert devtools.store is None
        assert len(devtools.history) == 0
        assert devtools.current_index == -1
    
    def test_connect_sends_init_event(self):
        """Test connect sends INIT event"""
        store = MockStore({"count": 5})
        devtools = DevToolsExtension()
        
        events = []
        devtools.subscribe(lambda event_type, data: events.append((event_type, data)))
        
        devtools.connect(store)
        
        assert len(events) == 1
        assert events[0][0] == "INIT"
        assert events[0][1]["state"] == {"count": 5}


# =====================================================
# TEST: ACTION RECORDING
# =====================================================

class TestActionRecording:
    """Test action recording in history"""
    
    def test_record_simple_action(self):
        """Test recording a simple action"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action(
            action_type="INCREMENT",
            payload=None,
            state_before={"count": 0},
            state_after={"count": 1}
        )
        
        assert len(devtools.history) == 1
        assert devtools.history[0].action_type == "INCREMENT"
        assert devtools.history[0].state_before == {"count": 0}
        assert devtools.history[0].state_after == {"count": 1}
        assert devtools.current_index == 0
    
    def test_record_action_with_payload(self):
        """Test recording action with payload"""
        store = MockStore({"items": []})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action(
            action_type="ADD_ITEM",
            payload={"text": "Todo 1"},
            state_before={"items": []},
            state_after={"items": [{"text": "Todo 1"}]}
        )
        
        assert len(devtools.history) == 1
        assert devtools.history[0].payload == {"text": "Todo 1"}
    
    def test_record_multiple_actions(self):
        """Test recording multiple actions"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        for i in range(5):
            devtools.record_action(
                action_type="INCREMENT",
                payload=None,
                state_before={"count": i},
                state_after={"count": i + 1}
            )
        
        assert len(devtools.history) == 5
        assert devtools.current_index == 4
    
    def test_record_respects_max_history(self):
        """Test recording respects max_history limit"""
        config = DevToolsConfig(max_history=3)
        store = MockStore({"count": 0})
        devtools = DevToolsExtension(config)
        devtools.connect(store)
        
        # Record 5 actions (exceeds max_history of 3)
        for i in range(5):
            devtools.record_action(
                action_type="INCREMENT",
                payload=None,
                state_before={"count": i},
                state_after={"count": i + 1}
            )
        
        # Only last 3 should remain
        assert len(devtools.history) == 3
        assert devtools.history[0].state_after == {"count": 3}
        assert devtools.history[-1].state_after == {"count": 5}
    
    def test_record_action_assigns_unique_ids(self):
        """Test each action gets a unique ID"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        for i in range(3):
            devtools.record_action(
                action_type="INCREMENT",
                payload=None,
                state_before={"count": i},
                state_after={"count": i + 1}
            )
        
        ids = [record.id for record in devtools.history]
        assert len(ids) == len(set(ids))  # All IDs are unique
        assert ids == [1, 2, 3]
    
    def test_record_action_sends_event(self):
        """Test recording action sends ACTION event"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)  # INIT event sent here, before subscribe
        
        events = []
        devtools.subscribe(lambda event_type, data: events.append((event_type, data)))
        
        devtools.record_action(
            action_type="INCREMENT",
            payload={"amount": 1},
            state_before={"count": 0},
            state_after={"count": 1}
        )
        
        # Should have only ACTION event (INIT was before subscribe)
        assert len(events) == 1
        assert events[0][0] == "ACTION"
        assert events[0][1]["type"] == "INCREMENT"
        assert events[0][1]["payload"] == {"amount": 1}
        assert events[0][1]["state"] == {"count": 1}
    
    def test_record_when_disabled(self):
        """Test recording when DevTools is disabled"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.disable()
        
        devtools.record_action(
            action_type="INCREMENT",
            payload=None,
            state_before={"count": 0},
            state_after={"count": 1}
        )
        
        # Should not record
        assert len(devtools.history) == 0
    
    def test_record_when_time_traveling(self):
        """Test recording when time-traveling (should skip)"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        # Record initial action
        devtools.record_action("INCREMENT", None, {"count": 0}, {"count": 1})
        
        # Simulate time-traveling
        devtools.is_time_traveling = True
        
        # Try to record (should be ignored)
        devtools.record_action("INCREMENT", None, {"count": 1}, {"count": 2})
        
        # Should only have 1 action
        assert len(devtools.history) == 1


# =====================================================
# TEST: TIME-TRAVEL DEBUGGING
# =====================================================

class TestTimeTravelDebugging:
    """Test time-travel debugging (jump to action)"""
    
    def test_jump_to_action(self):
        """Test jumping to a specific action"""
        def reducer(state, action):
            if action.type == "INCREMENT":
                return {"count": state["count"] + 1}
            return state
        
        store = MockStore({"count": 0}, reducer)
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        # Record 5 actions
        for i in range(5):
            devtools.record_action(
                action_type="INCREMENT",
                payload=None,
                state_before={"count": i},
                state_after={"count": i + 1}
            )
        
        # Jump to action 3 (state should be {"count": 3})
        action_id = devtools.history[2].id  # Index 2 = action 3
        success = devtools.jump_to_action(action_id)
        
        assert success == True
        assert store.state == {"count": 3}
        assert devtools.current_index == 2
    
    def test_jump_to_first_action(self):
        """Test jumping to first action"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        for i in range(3):
            devtools.record_action(
                action_type="INCREMENT",
                payload=None,
                state_before={"count": i},
                state_after={"count": i + 1}
            )
        
        action_id = devtools.history[0].id
        success = devtools.jump_to_action(action_id)
        
        assert success == True
        assert store.state == {"count": 1}
        assert devtools.current_index == 0
    
    def test_jump_to_last_action(self):
        """Test jumping to last action"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        for i in range(3):
            devtools.record_action(
                action_type="INCREMENT",
                payload=None,
                state_before={"count": i},
                state_after={"count": i + 1}
            )
        
        action_id = devtools.history[-1].id
        success = devtools.jump_to_action(action_id)
        
        assert success == True
        assert store.state == {"count": 3}
    
    def test_jump_to_invalid_action(self):
        """Test jumping to invalid action ID"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action("INCREMENT", None, {"count": 0}, {"count": 1})
        
        # Try to jump to non-existent action
        success = devtools.jump_to_action(9999)
        
        assert success == False
    
    def test_jump_without_store(self):
        """Test jumping without connected store"""
        devtools = DevToolsExtension()
        
        success = devtools.jump_to_action(1)
        
        assert success == False
    
    def test_jump_sends_event(self):
        """Test jump sends JUMP event"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action("INCREMENT", None, {"count": 0}, {"count": 1})
        devtools.record_action("INCREMENT", None, {"count": 1}, {"count": 2})
        
        events = []
        devtools.subscribe(lambda event_type, data: events.append((event_type, data)))
        
        action_id = devtools.history[0].id
        devtools.jump_to_action(action_id)
        
        jump_events = [e for e in events if e[0] == "JUMP"]
        assert len(jump_events) == 1
        assert jump_events[0][1]["actionId"] == action_id
        assert jump_events[0][1]["state"] == {"count": 1}
    
    def test_jump_notifies_subscribers(self):
        """Test jump notifies store subscribers"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        notifications = []
        store.subscribe(lambda: notifications.append(store.state.copy()))
        
        devtools.record_action("INCREMENT", None, {"count": 0}, {"count": 1})
        devtools.record_action("INCREMENT", None, {"count": 1}, {"count": 2})
        
        notifications.clear()
        
        action_id = devtools.history[0].id
        devtools.jump_to_action(action_id)
        
        # Should notify once
        assert len(notifications) == 1
        assert notifications[0] == {"count": 1}


# =====================================================
# TEST: ACTION SKIPPING
# =====================================================

class TestActionSkipping:
    """Test skipping actions and recomputation"""
    
    def test_skip_action(self):
        """Test skipping an action"""
        def reducer(state, action):
            if action.type == "ADD":
                return {"count": state["count"] + action.payload}
            return state
        
        store = MockStore({"count": 0}, reducer)
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        # Record: 0 -> 5 -> 10 -> 15
        devtools.record_action("ADD", 5, {"count": 0}, {"count": 5})
        devtools.record_action("ADD", 5, {"count": 5}, {"count": 10})
        devtools.record_action("ADD", 5, {"count": 10}, {"count": 15})
        
        # Skip middle action (5 -> 10)
        action_id = devtools.history[1].id
        success = devtools.skip_action(action_id)
        
        # Should succeed
        assert success == True
        
        # History should have 2 actions now
        assert len(devtools.history) == 2
    
    def test_skip_action_when_disabled(self):
        """Test skipping when feature is disabled"""
        config = DevToolsConfig(features={"skip": False})
        store = MockStore({"count": 0})
        devtools = DevToolsExtension(config)
        devtools.connect(store)
        
        devtools.record_action("INCREMENT", None, {"count": 0}, {"count": 1})
        
        action_id = devtools.history[0].id
        success = devtools.skip_action(action_id)
        
        assert success == False
        assert len(devtools.history) == 1
    
    def test_skip_invalid_action(self):
        """Test skipping invalid action ID"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action("INCREMENT", None, {"count": 0}, {"count": 1})
        
        success = devtools.skip_action(9999)
        
        assert success == False
    
    def test_skip_sends_event(self):
        """Test skip sends SKIP event"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action("INCREMENT", None, {"count": 0}, {"count": 1})
        
        events = []
        devtools.subscribe(lambda event_type, data: events.append((event_type, data)))
        
        action_id = devtools.history[0].id
        devtools.skip_action(action_id)
        
        skip_events = [e for e in events if e[0] == "SKIP"]
        assert len(skip_events) == 1


# =====================================================
# TEST: RESET
# =====================================================

class TestReset:
    """Test reset functionality"""
    
    def test_reset_clears_history(self):
        """Test reset clears history"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        for i in range(3):
            devtools.record_action("INCREMENT", None, {"count": i}, {"count": i + 1})
        
        devtools.reset()
        
        assert len(devtools.history) == 0
        assert devtools.current_index == -1
        assert devtools.next_action_id == 1
    
    def test_reset_restores_initial_state(self):
        """Test reset restores initial state"""
        store = MockStore({"count": 0})
        store._initial_state = {"count": 0}
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        store.state = {"count": 10}
        
        devtools.reset()
        
        assert store.state == {"count": 0}
    
    def test_reset_sends_event(self):
        """Test reset sends RESET event"""
        store = MockStore({"count": 5})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        events = []
        devtools.subscribe(lambda event_type, data: events.append((event_type, data)))
        
        devtools.reset()
        
        reset_events = [e for e in events if e[0] == "RESET"]
        assert len(reset_events) == 1


# =====================================================
# TEST: EXPORT/IMPORT
# =====================================================

class TestExportImport:
    """Test export and import functionality"""
    
    def test_export_state(self):
        """Test exporting state"""
        store = MockStore({"count": 5})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action("INCREMENT", None, {"count": 5}, {"count": 6})
        
        exported = devtools.export_state()
        
        assert exported != "{}"
        assert "count" in exported
    
    def test_import_state(self):
        """Test importing state"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        # Import state
        data = '{"state": {"count": 10}, "history": [], "currentIndex": -1}'
        success = devtools.import_state(data)
        
        assert success == True
        assert store.state == {"count": 10}
    
    def test_import_invalid_data(self):
        """Test importing invalid data"""
        store = MockStore({"count": 0})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        success = devtools.import_state("invalid json")
        
        assert success == False
    
    def test_export_when_disabled(self):
        """Test export when feature is disabled"""
        config = DevToolsConfig(features={"export": False})
        store = MockStore({"count": 0})
        devtools = DevToolsExtension(config)
        devtools.connect(store)
        
        exported = devtools.export_state()
        
        assert exported == "{}"
    
    def test_import_when_disabled(self):
        """Test import when feature is disabled"""
        config = DevToolsConfig(features={"import": False})
        store = MockStore({"count": 0})
        devtools = DevToolsExtension(config)
        devtools.connect(store)
        
        success = devtools.import_state('{"state": {"count": 10}}')
        
        assert success == False


# =====================================================
# TEST: STATE DIFF
# =====================================================

class TestStateDiff:
    """Test state diff calculation"""
    
    def test_diff_with_added_keys(self):
        """Test diff with added keys"""
        store = MockStore({})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action("ADD", None, {}, {"count": 1})
        devtools.record_action("ADD", None, {"count": 1}, {"count": 1, "name": "Test"})
        
        diff = devtools.get_state_diff(
            devtools.history[0].id,
            devtools.history[1].id
        )
        
        assert "name" in diff["added"]
        assert diff["added"]["name"] == "Test"
    
    def test_diff_with_removed_keys(self):
        """Test diff with removed keys"""
        store = MockStore({})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        # Record: state with 2 keys -> state with 1 key -> empty state
        devtools.record_action("ADD", None, {}, {"count": 1, "name": "Test"})
        devtools.record_action("REMOVE", None, {"count": 1, "name": "Test"}, {"count": 1})
        
        # Diff from action 1 to action 2 should show "name" removed
        diff = devtools.get_state_diff(
            devtools.history[0].id,
            devtools.history[1].id
        )
        
        assert "name" in diff["removed"]
        assert diff["removed"]["name"] == "Test"
    
    def test_diff_with_modified_keys(self):
        """Test diff with modified keys"""
        store = MockStore({})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        devtools.record_action("SET", None, {"count": 1}, {"count": 5})
        devtools.record_action("SET", None, {"count": 5}, {"count": 10})
        
        diff = devtools.get_state_diff(
            devtools.history[0].id,
            devtools.history[1].id
        )
        
        assert "count" in diff["modified"]
        assert diff["modified"]["count"]["from"] == 5
        assert diff["modified"]["count"]["to"] == 10
    
    def test_diff_with_invalid_actions(self):
        """Test diff with invalid action IDs"""
        store = MockStore({})
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        diff = devtools.get_state_diff(9999, 8888)
        
        assert diff is None


# =====================================================
# TEST: EVENT SYSTEM
# =====================================================

class TestEventSystem:
    """Test event subscription and notification"""
    
    def test_subscribe_to_events(self):
        """Test subscribing to events"""
        devtools = DevToolsExtension()
        
        events = []
        unsubscribe = devtools.subscribe(lambda event_type, data: events.append(event_type))
        
        devtools._send_event("TEST", {})
        
        assert len(events) == 1
        assert events[0] == "TEST"
        
        unsubscribe()
    
    def test_unsubscribe_from_events(self):
        """Test unsubscribing from events"""
        devtools = DevToolsExtension()
        
        events = []
        unsubscribe = devtools.subscribe(lambda event_type, data: events.append(event_type))
        
        devtools._send_event("TEST1", {})
        unsubscribe()
        devtools._send_event("TEST2", {})
        
        # Should only receive TEST1
        assert len(events) == 1
        assert events[0] == "TEST1"
    
    def test_multiple_subscribers(self):
        """Test multiple subscribers receive events"""
        devtools = DevToolsExtension()
        
        events1 = []
        events2 = []
        
        devtools.subscribe(lambda event_type, data: events1.append(event_type))
        devtools.subscribe(lambda event_type, data: events2.append(event_type))
        
        devtools._send_event("TEST", {})
        
        assert len(events1) == 1
        assert len(events2) == 1


# =====================================================
# TEST: ENABLE/DISABLE
# =====================================================

class TestEnableDisable:
    """Test enabling and disabling DevTools"""
    
    def test_disable_devtools(self):
        """Test disabling DevTools"""
        devtools = DevToolsExtension()
        
        events = []
        devtools.subscribe(lambda event_type, data: events.append(event_type))
        
        devtools.disable()
        
        assert devtools.is_enabled == False
        disable_events = [e for e in events if e == "DISABLE"]
        assert len(disable_events) == 1
    
    def test_enable_devtools(self):
        """Test enabling DevTools"""
        devtools = DevToolsExtension()
        devtools.is_enabled = False
        
        events = []
        devtools.subscribe(lambda event_type, data: events.append(event_type))
        
        devtools.enable()
        
        assert devtools.is_enabled == True
        enable_events = [e for e in events if e == "ENABLE"]
        assert len(enable_events) == 1


# =====================================================
# TEST: MIDDLEWARE INTEGRATION
# =====================================================

class TestMiddlewareIntegration:
    """Test DevTools middleware integration"""
    
    def test_create_devtools_middleware(self):
        """Test creating DevTools middleware"""
        devtools = DevToolsExtension()
        middleware = create_devtools_middleware(devtools)
        
        assert middleware is not None
        assert hasattr(middleware, 'handle')
    
    def test_middleware_records_actions(self):
        """Test middleware records actions"""
        def reducer(state, action):
            if action.type == "INCREMENT":
                return {"count": state["count"] + 1}
            return state
        
        store = MockStore({"count": 0}, reducer)
        devtools = DevToolsExtension()
        devtools.connect(store)
        
        middleware = create_devtools_middleware(devtools)
        store.middlewares.append(middleware)
        
        # Simulate middleware chain
        action = MockAction("INCREMENT")
        context = store
        next_func = lambda a: store.dispatch(a)
        
        middleware.handle(context, next_func, action)
        
        # Should record action
        assert len(devtools.history) >= 1


# =====================================================
# TEST: HELPER FUNCTIONS
# =====================================================

class TestHelperFunctions:
    """Test helper functions"""
    
    def test_connect_devtools(self):
        """Test connect_devtools helper"""
        store = MockStore({"count": 0})
        
        devtools = connect_devtools(store)
        
        assert devtools is not None
        assert devtools.store is store
        assert hasattr(store, '_initial_state')
    
    def test_connect_devtools_with_config(self):
        """Test connect_devtools with custom config"""
        store = MockStore({"count": 0})
        config = DevToolsConfig(name="TestStore", max_history=100)
        
        devtools = connect_devtools(store, config)
        
        assert devtools.config.name == "TestStore"
        assert devtools.config.max_history == 100


# =====================================================
# RUN TESTS
# =====================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
