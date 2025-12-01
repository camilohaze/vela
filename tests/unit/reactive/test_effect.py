"""
Tests unitarios para Effect

Jira: VELA-573 - TASK-029
Historia: Sistema Reactivo
"""

import pytest

from src.reactive.effect import Effect, effect
from src.reactive.signal import Signal
from src.reactive.computed import Computed
from src.reactive.types import DisposedNodeError


class TestEffectBasics:
    """Tests básicos de Effect."""
    
    def test_effect_creation(self):
        """Test creación de effect."""
        executed = []
        
        def fn():
            executed.append(1)
        
        eff = Effect(fn)
        assert len(executed) == 1  # Ejecuta inmediatamente
        
        eff.dispose()
    
    def test_effect_helper_function(self):
        """Test función helper effect()."""
        executed = []
        
        eff = effect(lambda: executed.append(1))
        assert len(executed) == 1
        
        eff.dispose()
    
    def test_effect_custom_id(self):
        """Test effect con ID personalizado."""
        eff = Effect(lambda: None, effect_id="my-effect")
        assert eff._node.id == "my-effect"
        
        eff.dispose()
    
    def test_effect_executes_immediately(self):
        """Test que effect ejecuta inmediatamente al crear."""
        executed = []
        
        count = Signal(5)
        eff = Effect(lambda: executed.append(count.get()))
        
        assert len(executed) == 1
        assert executed[0] == 5
        
        eff.dispose()


class TestEffectAutoTracking:
    """Tests de auto-tracking."""
    
    def test_effect_tracks_signal_dependency(self):
        """Test que effect registra dependencia de signal."""
        count = Signal(5)
        executed = []
        
        def fn():
            executed.append(count.get())
        
        eff = Effect(fn)
        
        # Verificar que effect depende de count
        assert count._node in eff._node.dependencies
        
        eff.dispose()
    
    def test_effect_re_executes_on_signal_change(self):
        """Test que effect se re-ejecuta cuando signal cambia."""
        count = Signal(5)
        executions = []
        
        def fn():
            executions.append(count.get())
        
        eff = Effect(fn)
        
        assert len(executions) == 1
        assert executions[0] == 5
        
        count.set(10)
        
        assert len(executions) == 2
        assert executions[1] == 10
        
        eff.dispose()
    
    def test_effect_tracks_multiple_signals(self):
        """Test que effect rastrea múltiples signals."""
        a = Signal(2)
        b = Signal(3)
        results = []
        
        def fn():
            results.append(a.get() + b.get())
        
        eff = Effect(fn)
        
        assert results == [5]
        
        a.set(10)
        assert results == [5, 13]
        
        b.set(20)
        assert results == [5, 13, 30]
        
        eff.dispose()
    
    def test_effect_with_computed_dependency(self):
        """Test effect que depende de computed."""
        count = Signal(5)
        doubled = Computed(lambda: count.get() * 2)
        results = []
        
        def fn():
            results.append(doubled.get())
        
        eff = Effect(fn)
        
        assert results == [10]
        
        count.set(10)
        assert results == [10, 20]
        
        eff.dispose()


class TestEffectCleanup:
    """Tests de cleanup functions."""
    
    def test_effect_cleanup_on_re_run(self):
        """Test que cleanup se ejecuta antes de re-run."""
        count = Signal(5)
        cleanups = []
        executions = []
        
        def fn():
            value = count.get()
            executions.append(value)
            
            def cleanup():
                cleanups.append(value)
            
            return cleanup
        
        eff = Effect(fn)
        
        assert executions == [5]
        assert cleanups == []
        
        count.set(10)
        
        assert executions == [5, 10]
        assert cleanups == [5]  # Cleanup del valor anterior
        
        eff.dispose()
    
    def test_effect_cleanup_on_dispose(self):
        """Test que cleanup se ejecuta en dispose()."""
        cleanups = []
        
        def fn():
            return lambda: cleanups.append(1)
        
        eff = Effect(fn)
        
        assert len(cleanups) == 0
        
        eff.dispose()
        
        assert len(cleanups) == 1
    
    def test_effect_no_cleanup_if_not_returned(self):
        """Test que no hay error si no se retorna cleanup."""
        count = Signal(5)
        executions = []
        
        def fn():
            executions.append(count.get())
            # No retorna cleanup
        
        eff = Effect(fn)
        
        count.set(10)  # No debería fallar
        
        assert len(executions) == 2
        
        eff.dispose()


class TestEffectStopResume:
    """Tests de stop() y resume()."""
    
    def test_effect_stop_prevents_execution(self):
        """Test que stop() previene ejecución en cambios."""
        count = Signal(5)
        executions = []
        
        def fn():
            executions.append(count.get())
        
        eff = Effect(fn)
        
        assert len(executions) == 1
        
        eff.stop()
        
        count.set(10)
        assert len(executions) == 1  # NO se ejecutó
        
        count.set(15)
        assert len(executions) == 1  # Todavía NO se ejecutó
        
        eff.dispose()
    
    def test_effect_resume_continues_execution(self):
        """Test que resume() reactiva el effect."""
        count = Signal(5)
        executions = []
        
        def fn():
            executions.append(count.get())
        
        eff = Effect(fn)
        
        eff.stop()
        count.set(10)
        
        assert len(executions) == 1  # Solo ejecución inicial
        
        eff.resume()  # Resume ejecuta inmediatamente
        
        assert len(executions) == 2
        assert executions[1] == 10
        
        count.set(15)
        assert len(executions) == 3  # Ahora responde a cambios
        
        eff.dispose()
    
    def test_effect_is_stopped_property(self):
        """Test propiedad is_stopped."""
        eff = Effect(lambda: None)
        
        assert not eff.is_stopped
        
        eff.stop()
        assert eff.is_stopped
        
        eff.resume()
        assert not eff.is_stopped
        
        eff.dispose()


class TestEffectDispose:
    """Tests de dispose()."""
    
    def test_effect_dispose(self):
        """Test que dispose() limpia el effect."""
        eff = Effect(lambda: None)
        
        eff.dispose()
        assert eff.is_disposed
    
    def test_effect_run_after_dispose_fails(self):
        """Test que run() después de dispose() falla."""
        eff = Effect(lambda: None)
        eff.dispose()
        
        with pytest.raises(DisposedNodeError):
            eff.run()
    
    def test_effect_cleanup_dependencies_on_dispose(self):
        """Test que dispose() limpia dependencias."""
        count = Signal(5)
        eff = Effect(lambda: count.get())
        
        # Verificar que hay dependencia
        assert count._node in eff._node.dependencies
        
        eff.dispose()
        
        # Dependencias deberían estar limpias
        assert len(eff._node.dependencies) == 0


class TestEffectManualRun:
    """Tests de run() manual."""
    
    def test_effect_manual_run(self):
        """Test que run() ejecuta manualmente."""
        executions = []
        
        eff = Effect(lambda: executions.append(1))
        
        assert len(executions) == 1
        
        eff.run()
        assert len(executions) == 2
        
        eff.run()
        assert len(executions) == 3
        
        eff.dispose()
    
    def test_effect_manual_run_when_stopped(self):
        """Test que run() no ejecuta cuando está stopped."""
        executions = []
        
        eff = Effect(lambda: executions.append(1))
        
        eff.stop()
        
        eff.run()  # No debería ejecutar
        assert len(executions) == 1  # Solo ejecución inicial
        
        eff.dispose()


class TestEffectRepresentation:
    """Tests de __repr__."""
    
    def test_effect_repr_active(self):
        """Test __repr__ cuando está activo."""
        eff = Effect(lambda: None, effect_id="test-effect")
        assert "test-effect" in repr(eff)
        assert "active" in repr(eff)
        
        eff.dispose()
    
    def test_effect_repr_stopped(self):
        """Test __repr__ cuando está stopped."""
        eff = Effect(lambda: None, effect_id="test-effect")
        eff.stop()
        
        assert "stopped" in repr(eff)
        
        eff.dispose()
    
    def test_effect_repr_disposed(self):
        """Test __repr__ cuando está disposed."""
        eff = Effect(lambda: None, effect_id="test-effect")
        eff.dispose()
        
        assert "disposed" in repr(eff)


class TestEffectIntegration:
    """Tests de integración complejos."""
    
    def test_effect_with_conditional_dependencies(self):
        """Test effect con dependencias condicionales."""
        flag = Signal(True)
        a = Signal(10)
        b = Signal(20)
        results = []
        
        def fn():
            value = a.get() if flag.get() else b.get()
            results.append(value)
        
        eff = Effect(fn)
        
        assert results == [10]
        
        a.set(15)
        assert results == [10, 15]
        
        flag.set(False)
        assert results == [10, 15, 20]  # Ahora lee b
        
        b.set(25)
        assert results == [10, 15, 20, 25]
        
        a.set(30)
        # NO debería ejecutar porque ahora lee b, no a
        assert results == [10, 15, 20, 25]
        
        eff.dispose()
    
    def test_effect_chain_reaction(self):
        """Test múltiples effects en cadena."""
        count = Signal(0)
        log = []
        
        eff1 = Effect(lambda: log.append(f"eff1: {count.get()}"))
        eff2 = Effect(lambda: log.append(f"eff2: {count.get()}"))
        eff3 = Effect(lambda: log.append(f"eff3: {count.get()}"))
        
        assert len(log) == 3  # Todos ejecutan inmediatamente
        
        count.set(5)
        
        assert len(log) == 6  # Todos se re-ejecutan
        
        for eff in [eff1, eff2, eff3]:
            eff.dispose()
    
    def test_effect_with_nested_signals(self):
        """Test effect con múltiples niveles de signals."""
        a = Signal(2)
        b = Signal(3)
        c = Signal(4)
        results = []
        
        def fn():
            results.append(a.get() * b.get() + c.get())
        
        eff = Effect(fn)
        
        assert results == [10]  # 2*3+4
        
        a.set(5)
        assert results == [10, 19]  # 5*3+4
        
        b.set(2)
        assert results == [10, 19, 14]  # 5*2+4
        
        c.set(10)
        assert results == [10, 19, 14, 20]  # 5*2+10
        
        eff.dispose()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
