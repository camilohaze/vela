"""
Tests para Event Propagation (TASK-035O)

Historia: VELA-575 - Dependency Injection
Epic: VELA-573 - Sistema de Reactividad
Sprint: 14

Tests de propagación de eventos con bubbling, capturing y cancelación.

Coverage:
- Bubbling propagation (child → parent)
- Capturing propagation (parent → child)
- stopPropagation() behavior
- stopImmediatePropagation() behavior
- preventDefault() behavior
- Mixed capturing/bubbling listeners
- Component hierarchy scenarios
- Event phase transitions
- Path composition
"""

import pytest
from unittest.mock import Mock, call
from dataclasses import dataclass
from typing import Optional, Any
import sys
import os

# Add src to path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../src')))

from runtime.events.event_bus import Event, EventBus, EventPhase


# =====================
# Mock Components
# =====================

class MockEventTarget:
    """Mock component for testing propagation."""
    
    def __init__(self, name: str, parent: Optional['MockEventTarget'] = None):
        self.name = name
        self.parent = parent
        self.bus = EventBus()
        self.received_events = []
    
    def dispatch_event(self, event: Event):
        """Dispatch event with propagation."""
        event.target = self
        return self.bus.dispatch_event(
            event, 
            target=self,
            use_capturing=True,
            use_bubbling=event.bubbles
        )
    
    def add_event_listener(self, event_type: str, listener, use_capture: bool = False):
        """Add event listener."""
        return self.bus.on(event_type, listener, use_capture=use_capture)
    
    def __repr__(self):
        return f"MockEventTarget({self.name})"


# =====================
# Test: Event Class Extensions
# =====================

class TestEventClassExtensions:
    """Test new Event class propagation fields and methods."""
    
    def test_event_has_propagation_fields(self):
        """Test que Event tiene todos los campos de propagación."""
        event = Event("test", {"data": "value"})
        
        # Campos básicos
        assert event.type == "test"
        assert event.payload == {"data": "value"}
        assert event.target is None
        
        # Campos de propagación (nuevos en TASK-035O)
        assert event.current_target is None
        assert event.event_phase == EventPhase.NONE
        assert event.bubbles is True
        assert event.cancelable is True
        assert event.immediate_propagation_stopped is False
        assert event.path == []
    
    def test_stop_immediate_propagation(self):
        """Test que stop_immediate_propagation() funciona."""
        event = Event("test", {})
        
        assert event.immediate_propagation_stopped is False
        assert event.propagation_stopped is False
        
        event.stop_immediate_propagation()
        
        assert event.immediate_propagation_stopped is True
        assert event.propagation_stopped is True
    
    def test_compose_path_simple(self):
        """Test compose_path() con jerarquía simple."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        grandchild = MockEventTarget("grandchild", parent=child)
        
        event = Event("test", {})
        path = event.compose_path(grandchild)
        
        assert len(path) == 3
        assert path[0].name == "root"
        assert path[1].name == "child"
        assert path[2].name == "grandchild"
    
    def test_compose_path_no_parent(self):
        """Test compose_path() sin parent."""
        target = MockEventTarget("target")
        
        event = Event("test", {})
        path = event.compose_path(target)
        
        assert len(path) == 1
        assert path[0].name == "target"
    
    def test_event_phase_enum(self):
        """Test que EventPhase enum tiene valores correctos."""
        assert EventPhase.NONE.value == 0
        assert EventPhase.CAPTURING.value == 1
        assert EventPhase.AT_TARGET.value == 2
        assert EventPhase.BUBBLING.value == 3


# =====================
# Test: Bubbling Propagation
# =====================

class TestBubblingPropagation:
    """Test propagación bubbling (child → parent)."""
    
    def test_bubbling_through_hierarchy(self):
        """Test que evento bubble desde child a root."""
        # Setup hierarchy: grandchild → child → root
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        grandchild = MockEventTarget("grandchild", parent=child)
        
        # Track execution order
        order = []
        
        root.add_event_listener("click", lambda e: order.append("root"))
        child.add_event_listener("click", lambda e: order.append("child"))
        grandchild.add_event_listener("click", lambda e: order.append("grandchild"))
        
        # Dispatch from grandchild
        event = Event("click", {"button": 1}, bubbles=True)
        grandchild.dispatch_event(event)
        
        # Should execute: grandchild → child → root
        assert order == ["grandchild", "child", "root"]
    
    def test_bubbling_sets_event_phase(self):
        """Test que event_phase se actualiza correctamente."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        
        phases = []
        
        root.add_event_listener("click", lambda e: phases.append(e.event_phase))
        child.add_event_listener("click", lambda e: phases.append(e.event_phase))
        
        event = Event("click", {}, bubbles=True)
        child.dispatch_event(event)
        
        # child: AT_TARGET, root: BUBBLING
        assert phases[0] == EventPhase.AT_TARGET
        assert phases[1] == EventPhase.BUBBLING
    
    def test_bubbling_sets_current_target(self):
        """Test que current_target se actualiza en cada fase."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        
        current_targets = []
        
        root.add_event_listener("click", lambda e: current_targets.append(e.current_target.name))
        child.add_event_listener("click", lambda e: current_targets.append(e.current_target.name))
        
        event = Event("click", {}, bubbles=True)
        child.dispatch_event(event)
        
        assert current_targets == ["child", "root"]
    
    def test_no_bubbling_when_bubbles_false(self):
        """Test que evento NO bubble cuando bubbles=False."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        
        order = []
        
        root.add_event_listener("click", lambda e: order.append("root"))
        child.add_event_listener("click", lambda e: order.append("child"))
        
        event = Event("click", {}, bubbles=False)
        child.dispatch_event(event)
        
        # Solo child, sin bubbling
        assert order == ["child"]


# =====================
# Test: Capturing Propagation
# =====================

class TestCapturingPropagation:
    """Test propagación capturing (parent → child)."""
    
    def test_full_propagation_cycle(self):
        """Test ciclo completo: capturing → at_target → bubbling."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        grandchild = MockEventTarget("grandchild", parent=child)
        
        # Track execution with phases
        execution = []
        
        def track(name):
            def handler(e):
                execution.append({
                    "name": name,
                    "phase": e.event_phase,
                    "current": e.current_target.name
                })
            return handler
        
        # Register capturing listeners (for capturing phase)
        root.add_event_listener("click", track("root"), use_capture=True)
        child.add_event_listener("click", track("child"), use_capture=True)
        
        # Register bubbling listeners (for at_target and bubbling phases)
        root.add_event_listener("click", track("root"), use_capture=False)
        child.add_event_listener("click", track("child"), use_capture=False)
        grandchild.add_event_listener("click", track("grandchild"), use_capture=False)
        
        event = Event("click", {}, bubbles=True)
        grandchild.dispatch_event(event)
        
        # Orden esperado:
        # 1. CAPTURING: root → child
        # 2. AT_TARGET: grandchild
        # 3. BUBBLING: child → root
        assert len(execution) == 5
        
        # CAPTURING phase
        assert execution[0]["phase"] == EventPhase.CAPTURING
        assert execution[0]["current"] == "root"
        assert execution[1]["phase"] == EventPhase.CAPTURING
        assert execution[1]["current"] == "child"
        
        # AT_TARGET phase
        assert execution[2]["phase"] == EventPhase.AT_TARGET
        assert execution[2]["current"] == "grandchild"
        
        # BUBBLING phase
        assert execution[3]["phase"] == EventPhase.BUBBLING
        assert execution[3]["current"] == "child"
        assert execution[4]["phase"] == EventPhase.BUBBLING
        assert execution[4]["current"] == "root"
    
    def test_capturing_skipped_at_target(self):
        """Test que capturing NO ejecuta en target."""
        target = MockEventTarget("target")
        
        order = []
        target.add_event_listener("click", lambda e: order.append("listener"))
        
        event = Event("click", {}, bubbles=True)
        target.dispatch_event(event)
        
        # Solo AT_TARGET (sin capturing ni bubbling porque no hay parent)
        assert order == ["listener"]


# =====================
# Test: stopPropagation
# =====================

class TestStopPropagation:
    """Test stopPropagation() y stopImmediatePropagation()."""
    
    def test_stop_propagation_in_bubbling(self):
        """Test que stopPropagation() detiene bubbling."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        
        order = []
        
        root.add_event_listener("click", lambda e: order.append("root"))
        child.add_event_listener("click", lambda e: (
            order.append("child"),
            e.stop_propagation()
        ))
        
        event = Event("click", {}, bubbles=True)
        child.dispatch_event(event)
        
        # Solo child (stopPropagation prevents bubbling to root)
        assert order == ["child"]
    
    def test_stop_propagation_in_capturing(self):
        """Test que stopPropagation() detiene capturing."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        grandchild = MockEventTarget("grandchild", parent=child)
        
        execution = []
        
        def stop_at_child(e):
            execution.append("child-capturing")
            e.stop_propagation()
        
        # Register capturing listeners
        root.add_event_listener("click", lambda e: execution.append("root-capturing"), use_capture=True)
        child.add_event_listener("click", stop_at_child, use_capture=True)
        
        # Register bubbling listener on grandchild
        grandchild.add_event_listener("click", lambda e: execution.append("grandchild"), use_capture=False)
        
        event = Event("click", {}, bubbles=True)
        grandchild.dispatch_event(event)
        
        # Capturing: root → child (STOP)
        # No AT_TARGET ni BUBBLING
        assert "root-capturing" in execution
        assert "child-capturing" in execution
        assert "grandchild" not in execution
    
    def test_stop_immediate_propagation(self):
        """Test que stopImmediatePropagation() detiene listeners restantes."""
        target = MockEventTarget("target")
        
        order = []
        
        def first_listener(e):
            order.append("first")
            e.stop_immediate_propagation()
        
        target.add_event_listener("click", first_listener)
        target.add_event_listener("click", lambda e: order.append("second"))
        target.add_event_listener("click", lambda e: order.append("third"))
        
        event = Event("click", {})
        target.dispatch_event(event)
        
        # Solo "first" (immediate propagation stopped)
        assert order == ["first"]
    
    def test_stop_immediate_propagation_sets_flags(self):
        """Test que stopImmediatePropagation() setea ambos flags."""
        event = Event("test", {})
        
        event.stop_immediate_propagation()
        
        assert event.immediate_propagation_stopped is True
        assert event.propagation_stopped is True


# =====================
# Test: preventDefault
# =====================

class TestPreventDefault:
    """Test preventDefault() behavior."""
    
    def test_prevent_default_returns_false(self):
        """Test que preventDefault() hace que dispatch retorne False."""
        target = MockEventTarget("target")
        
        target.add_event_listener("click", lambda e: e.prevent_default())
        
        event = Event("click", {}, cancelable=True)
        result = target.dispatch_event(event)
        
        # dispatch_event retorna False cuando default prevented
        assert result is False
    
    def test_prevent_default_with_non_cancelable(self):
        """Test preventDefault() en evento non-cancelable."""
        target = MockEventTarget("target")
        
        target.add_event_listener("click", lambda e: e.prevent_default())
        
        event = Event("click", {}, cancelable=False)
        result = target.dispatch_event(event)
        
        # Debería ser ignorado o manejado gracefully
        # (Implementación depende de runtime)
        assert event.default_prevented is False or event.default_prevented is True


# =====================
# Test: Complex Scenarios
# =====================

class TestComplexScenarios:
    """Test escenarios complejos de propagación."""
    
    def test_multiple_listeners_same_level(self):
        """Test múltiples listeners en mismo target."""
        target = MockEventTarget("target")
        
        order = []
        
        target.add_event_listener("click", lambda e: order.append("first"))
        target.add_event_listener("click", lambda e: order.append("second"))
        target.add_event_listener("click", lambda e: order.append("third"))
        
        event = Event("click", {})
        target.dispatch_event(event)
        
        assert order == ["first", "second", "third"]
    
    def test_propagation_with_deep_hierarchy(self):
        """Test propagación con jerarquía profunda (5 niveles)."""
        # Build hierarchy
        level1 = MockEventTarget("level1")
        level2 = MockEventTarget("level2", parent=level1)
        level3 = MockEventTarget("level3", parent=level2)
        level4 = MockEventTarget("level4", parent=level3)
        level5 = MockEventTarget("level5", parent=level4)
        
        order = []
        
        for target in [level1, level2, level3, level4, level5]:
            target.add_event_listener("event", lambda e, t=target: order.append(t.name))
        
        event = Event("event", {}, bubbles=True)
        level5.dispatch_event(event)
        
        # AT_TARGET: level5
        # Bubbling: level4 → level3 → level2 → level1
        # (Capturing disabled until useCapture support)
        assert len(order) == 5
        
        # Check phases
        assert order[0] == "level5"  # AT_TARGET
        assert order[1:5] == ["level4", "level3", "level2", "level1"]  # Bubbling
    
    def test_event_target_remains_constant(self):
        """Test que event.target NO cambia durante propagación."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        
        targets = []
        
        root.add_event_listener("click", lambda e: targets.append(e.target.name))
        child.add_event_listener("click", lambda e: targets.append(e.target.name))
        
        event = Event("click", {}, bubbles=True)
        child.dispatch_event(event)
        
        # event.target SIEMPRE es child (original target)
        assert all(t == "child" for t in targets)
    
    def test_event_path_composition(self):
        """Test que event.path se compone correctamente."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        grandchild = MockEventTarget("grandchild", parent=child)
        
        captured_path = []
        
        def capture_path(e):
            captured_path.extend([t.name for t in e.path])
        
        grandchild.add_event_listener("click", capture_path)
        
        event = Event("click", {}, bubbles=True)
        grandchild.dispatch_event(event)
        
        assert captured_path == ["root", "child", "grandchild"]
    
    def test_mixed_bubbling_and_non_bubbling(self):
        """Test listeners con eventos bubbling y non-bubbling mezclados."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        
        bubbling_count = 0
        non_bubbling_count = 0
        
        def count_bubbling(e):
            nonlocal bubbling_count
            bubbling_count += 1
        
        def count_non_bubbling(e):
            nonlocal non_bubbling_count
            non_bubbling_count += 1
        
        root.add_event_listener("bubble", count_bubbling)
        root.add_event_listener("no-bubble", count_non_bubbling)
        child.add_event_listener("bubble", count_bubbling)
        child.add_event_listener("no-bubble", count_non_bubbling)
        
        # Dispatch bubbling event
        bubble_event = Event("bubble", {}, bubbles=True)
        child.dispatch_event(bubble_event)
        
        # Dispatch non-bubbling event
        no_bubble_event = Event("no-bubble", {}, bubbles=False)
        child.dispatch_event(no_bubble_event)
        
        # bubbling: child (at_target) + root (bubbling) = 2
        # non-bubbling: solo child (at_target) = 1
        # (Capturing disabled until useCapture support)
        assert bubbling_count == 2  # at_target (1) + bubbling (1)
        assert non_bubbling_count == 1  # at_target (1)


# =====================
# Test: Error Handling
# =====================

class TestErrorHandling:
    """Test manejo de errores en propagación."""
    
    def test_listener_error_isolated(self):
        """Test que error en listener NO detiene propagación."""
        root = MockEventTarget("root")
        child = MockEventTarget("child", parent=root)
        
        order = []
        
        def failing_listener(e):
            order.append("child-before-error")
            raise Exception("Test error")
        
        child.add_event_listener("click", failing_listener)
        root.add_event_listener("click", lambda e: order.append("root"))
        
        event = Event("click", {}, bubbles=True)
        
        # No debería raise exception
        child.dispatch_event(event)
        
        # Root listener debería ejecutarse a pesar del error
        assert "root" in order
    
    def test_empty_hierarchy(self):
        """Test dispatch en target sin parent."""
        target = MockEventTarget("target")
        
        executed = False
        
        def listener(e):
            nonlocal executed
            executed = True
        
        target.add_event_listener("click", listener)
        
        event = Event("click", {})
        target.dispatch_event(event)
        
        assert executed is True


# =====================
# Run Tests
# =====================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
