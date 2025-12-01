"""
Tests unitarios para Watch

Jira: VELA-573 - TASK-030
Historia: Sistema Reactivo
"""

import pytest

from src.reactive.watch import Watch, watch
from src.reactive.signal import Signal
from src.reactive.computed import Computed
from src.reactive.types import DisposedNodeError


class TestWatchBasics:
    """Tests básicos de Watch."""
    
    def test_watch_creation(self):
        """Test creación de watch."""
        count = Signal(0)
        callback_called = []
        
        w = Watch(count, lambda new, old: callback_called.append((new, old)))
        
        # No debería ejecutar inmediatamente (immediate=False por defecto)
        assert len(callback_called) == 0
        
        w.dispose()
    
    def test_watch_helper_function(self):
        """Test función helper watch()."""
        count = Signal(0)
        calls = []
        
        w = watch(count, lambda new, old: calls.append((new, old)))
        
        assert len(calls) == 0
        
        w.dispose()
    
    def test_watch_executes_on_change(self):
        """Test que watch ejecuta callback cuando signal cambia."""
        count = Signal(0)
        calls = []
        
        w = Watch(count, lambda new, old: calls.append((new, old)))
        
        count.set(5)
        
        assert len(calls) == 1
        assert calls[0] == (5, 0)
        
        w.dispose()
    
    def test_watch_with_immediate(self):
        """Test watch con immediate=True."""
        count = Signal(5)
        calls = []
        
        w = Watch(count, lambda new, old: calls.append((new, old)), immediate=True)
        
        # Debería ejecutar inmediatamente
        assert len(calls) == 1
        assert calls[0] == (5, 5)  # Same value (initial)
        
        count.set(10)
        assert len(calls) == 2
        assert calls[1] == (10, 5)
        
        w.dispose()


class TestWatchCallback:
    """Tests de callbacks."""
    
    def test_watch_callback_receives_new_and_old(self):
        """Test que callback recibe valores nuevo y anterior."""
        count = Signal(10)
        received = []
        
        def callback(new_val, old_val):
            received.append({"new": new_val, "old": old_val})
        
        w = Watch(count, callback)
        
        count.set(20)
        
        assert len(received) == 1
        assert received[0]["new"] == 20
        assert received[0]["old"] == 10
        
        count.set(30)
        
        assert len(received) == 2
        assert received[1]["new"] == 30
        assert received[1]["old"] == 20
        
        w.dispose()
    
    def test_watch_callback_error_handling(self):
        """Test que errores en callback no rompen el watch."""
        count = Signal(0)
        calls = []
        
        def callback(new_val, old_val):
            calls.append((new_val, old_val))
            if new_val == 5:
                raise Exception("Test error")
        
        w = Watch(count, callback)
        
        count.set(5)  # Lanza error pero no debería romper
        assert len(calls) == 1
        
        count.set(10)  # Debería seguir funcionando
        assert len(calls) == 2
        
        w.dispose()


class TestWatchMultipleSources:
    """Tests con múltiples sources."""
    
    def test_watch_multiple_signals(self):
        """Test watch con múltiples signals."""
        a = Signal(1)
        b = Signal(2)
        calls = []
        
        w = Watch([a, b], lambda new_vals, old_vals: calls.append((new_vals, old_vals)))
        
        a.set(10)
        
        assert len(calls) == 1
        assert calls[0] == ([10, 2], [1, 2])
        
        b.set(20)
        
        assert len(calls) == 2
        assert calls[1] == ([10, 20], [10, 2])
        
        w.dispose()
    
    def test_watch_signal_and_computed(self):
        """Test watch con signal y computed."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        calls = []
        
        w = Watch([count, doubled], lambda new_vals, old_vals: calls.append((new_vals, old_vals)))
        
        count.set(10)
        
        assert len(calls) == 1
        assert calls[0] == ([10, 20], [5, 10])
        
        w.dispose()
    
    def test_watch_multiple_with_immediate(self):
        """Test múltiples sources con immediate=True."""
        a = Signal(1)
        b = Signal(2)
        calls = []
        
        w = Watch([a, b], lambda new_vals, old_vals: calls.append(new_vals), immediate=True)
        
        assert len(calls) == 1
        assert calls[0] == [1, 2]
        
        w.dispose()


class TestWatchStopResume:
    """Tests de stop() y resume()."""
    
    def test_watch_stop_prevents_callback(self):
        """Test que stop() previene ejecución de callback."""
        count = Signal(0)
        calls = []
        
        w = Watch(count, lambda new, old: calls.append((new, old)))
        
        count.set(5)
        assert len(calls) == 1
        
        w.stop()
        
        count.set(10)
        assert len(calls) == 1  # NO ejecutó
        
        count.set(15)
        assert len(calls) == 1  # Todavía NO ejecutó
        
        w.dispose()
    
    def test_watch_resume_continues_watching(self):
        """Test que resume() reactiva el watch."""
        count = Signal(0)
        calls = []
        
        w = Watch(count, lambda new, old: calls.append((new, old)))
        
        count.set(5)
        assert len(calls) == 1
        
        w.stop()
        count.set(10)
        assert len(calls) == 1
        
        w.resume()
        
        # Resume NO ejecuta callback inmediatamente
        assert len(calls) == 1
        
        count.set(15)
        assert len(calls) == 2
        assert calls[1] == (15, 10)  # Usa el último valor antes de stop
        
        w.dispose()
    
    def test_watch_is_stopped_property(self):
        """Test propiedad is_stopped."""
        count = Signal(0)
        w = Watch(count, lambda new, old: None)
        
        assert not w.is_stopped
        
        w.stop()
        assert w.is_stopped
        
        w.resume()
        assert not w.is_stopped
        
        w.dispose()


class TestWatchDispose:
    """Tests de dispose()."""
    
    def test_watch_dispose(self):
        """Test que dispose() limpia el watch."""
        count = Signal(0)
        w = Watch(count, lambda new, old: None)
        
        w.dispose()
        assert w.is_disposed
    
    def test_watch_no_callback_after_dispose(self):
        """Test que no hay callback después de dispose()."""
        count = Signal(0)
        calls = []
        
        w = Watch(count, lambda new, old: calls.append((new, old)))
        
        count.set(5)
        assert len(calls) == 1
        
        w.dispose()
        
        count.set(10)
        assert len(calls) == 1  # NO ejecutó después de dispose
    
    def test_watch_cleanup_sources(self):
        """Test que dispose() limpia sources."""
        count = Signal(0)
        w = Watch(count, lambda new, old: None)
        
        assert len(w._sources) == 1
        
        w.dispose()
        
        assert len(w._sources) == 0


class TestWatchWithComputed:
    """Tests de watch con computed."""
    
    def test_watch_computed_dependency(self):
        """Test watch observando un computed."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        calls = []
        
        w = Watch(doubled, lambda new, old: calls.append((new, old)))
        
        count.set(10)
        
        assert len(calls) == 1
        assert calls[0] == (20, 10)
        
        w.dispose()
    
    def test_watch_computed_chain(self):
        """Test watch con chain de computed."""
        base = Signal(2)
        doubled = Computed(lambda: base.get() * 2)
        quadrupled = Computed(lambda: doubled.get() * 2)
        calls = []
        
        w = Watch(quadrupled, lambda new, old: calls.append((new, old)))
        
        base.set(3)
        
        assert len(calls) == 1
        assert calls[0] == (12, 8)  # 3*2*2 vs 2*2*2
        
        w.dispose()


class TestWatchRepresentation:
    """Tests de __repr__."""
    
    def test_watch_repr_active(self):
        """Test __repr__ cuando está activo."""
        count = Signal(0)
        w = Watch(count, lambda new, old: None, watch_id="test-watch")
        
        assert "test-watch" in repr(w)
        assert "1 sources" in repr(w)
        assert "active" in repr(w)
        
        w.dispose()
    
    def test_watch_repr_stopped(self):
        """Test __repr__ cuando está stopped."""
        count = Signal(0)
        w = Watch(count, lambda new, old: None, watch_id="test-watch")
        w.stop()
        
        assert "stopped" in repr(w)
        
        w.dispose()
    
    def test_watch_repr_disposed(self):
        """Test __repr__ cuando está disposed."""
        count = Signal(0)
        w = Watch(count, lambda new, old: None, watch_id="test-watch")
        w.dispose()
        
        assert "disposed" in repr(w)
    
    def test_watch_repr_multiple_sources(self):
        """Test __repr__ con múltiples sources."""
        a = Signal(1)
        b = Signal(2)
        c = Signal(3)
        w = Watch([a, b, c], lambda new, old: None)
        
        assert "3 sources" in repr(w)
        
        w.dispose()


class TestWatchIntegration:
    """Tests de integración complejos."""
    
    def test_watch_conditional_changes(self):
        """Test watch con cambios condicionales."""
        flag = Signal(True)
        value = Signal(10)
        calls = []
        
        w = Watch([flag, value], lambda new_vals, old_vals: calls.append(new_vals))
        
        value.set(20)
        assert len(calls) == 1
        assert calls[0] == [True, 20]
        
        flag.set(False)
        assert len(calls) == 2
        assert calls[1] == [False, 20]
        
        w.dispose()
    
    def test_multiple_watchers_same_signal(self):
        """Test múltiples watchers en el mismo signal."""
        count = Signal(0)
        calls1 = []
        calls2 = []
        calls3 = []
        
        w1 = Watch(count, lambda new, old: calls1.append(new))
        w2 = Watch(count, lambda new, old: calls2.append(new))
        w3 = Watch(count, lambda new, old: calls3.append(new))
        
        count.set(5)
        
        assert calls1 == [5]
        assert calls2 == [5]
        assert calls3 == [5]
        
        for w in [w1, w2, w3]:
            w.dispose()
    
    def test_watch_with_nested_updates(self):
        """Test watch con updates anidados."""
        a = Signal(1)
        b = Signal(2)
        calls = []
        
        def callback(new_val, old_val):
            calls.append((new_val, old_val))
            if new_val == 10:
                b.set(20)  # Update anidado
        
        w = Watch(a, callback)
        
        a.set(10)
        
        # Solo el cambio de 'a' debería triggear el callback
        assert len(calls) == 1
        assert calls[0] == (10, 1)
        
        w.dispose()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
