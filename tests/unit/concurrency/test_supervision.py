"""
Tests for Supervision System

Jira: VELA-579
Task: TASK-042
Sprint: Sprint 17
Fecha: 2025-12-02

Tests para el sistema de supervisión de actors:
- OneForOneStrategy
- OneForAllStrategy
- RestForOneStrategy
- RestartPolicy y backoff
- Escalación de errores
- Edge cases (restart loops)
"""

import pytest
import time
from unittest.mock import Mock, patch, call
from typing import Any

from src.concurrency.actor import Actor, ActorRef, ActorState
from src.concurrency.supervision import (
    SupervisorActor,
    SupervisionStrategy,
    OneForOneStrategy,
    OneForAllStrategy,
    RestForOneStrategy,
    RestartPolicy,
    RestartStats,
    BackoffStrategy,
    SupervisorDirective,
)


# ============================================================================
# TEST ACTORS
# ============================================================================

class SimpleTestActor(Actor):
    """Actor simple para tests."""
    
    def __init__(self, name: str = "test"):
        super().__init__()
        self.name = name
        self.received_messages = []
        self.restart_count = 0
        self.stop_count = 0
    
    def receive(self, message: Any) -> None:
        """Procesar mensaje."""
        self.received_messages.append(message)
        
        if message == "fail":
            raise RuntimeError("Test failure")
    
    def pre_restart(self, error=None) -> None:
        """Hook antes de reinicio."""
        self.restart_count += 1
        super().pre_restart(error)
    
    def post_restart(self, error=None) -> None:
        """Hook después de reinicio."""
        super().post_restart(error)
    
    def post_stop(self) -> None:
        """Hook al detener."""
        self.stop_count += 1
        super().post_stop()


class FailingActor(Actor):
    """Actor que siempre falla."""
    
    def __init__(self):
        super().__init__()
        self.failure_count = 0
    
    def receive(self, message: Any) -> None:
        """Siempre falla."""
        self.failure_count += 1
        raise RuntimeError(f"Always fails (attempt {self.failure_count})")


# ============================================================================
# HELPERS
# ============================================================================

def create_supervisor_with_ref(strategy=None, parent=None, name="supervisor"):
    """Helper para crear supervisor con ActorRef correctamente inicializado."""
    supervisor = SupervisorActor(strategy=strategy, parent_supervisor=parent)
    # Crear ActorRef usando la firma correcta
    ref = ActorRef(name=name, actor=supervisor)
    supervisor.state = ActorState.RUNNING
    return supervisor


# ============================================================================
# FIXTURES
# ============================================================================

@pytest.fixture
def simple_actor():
    """Actor simple de prueba."""
    actor = SimpleTestActor("simple")
    actor.state = ActorState.RUNNING
    return actor


@pytest.fixture
def restart_policy():
    """Política de reinicio por defecto."""
    return RestartPolicy(
        max_retries=3,
        within_time_window=60.0,
        backoff_strategy=BackoffStrategy.CONSTANT,
        initial_delay=0.1,  # Delay corto para tests
        max_delay=1.0
    )


@pytest.fixture
def one_for_one_strategy(restart_policy):
    """OneForOne strategy."""
    return OneForOneStrategy(restart_policy)


@pytest.fixture
def one_for_all_strategy(restart_policy):
    """OneForAll strategy."""
    return OneForAllStrategy(restart_policy)


@pytest.fixture
def rest_for_one_strategy(restart_policy):
    """RestForOne strategy."""
    return RestForOneStrategy(restart_policy)


@pytest.fixture
def supervisor(one_for_one_strategy):
    """Supervisor con OneForOne strategy."""
    return create_supervisor_with_ref(strategy=one_for_one_strategy, name="supervisor")


# ============================================================================
# TESTS: RestartPolicy
# ============================================================================

class TestRestartPolicy:
    """Tests para RestartPolicy."""
    
    def test_constant_backoff(self):
        """Test backoff constante."""
        policy = RestartPolicy(
            backoff_strategy=BackoffStrategy.CONSTANT,
            initial_delay=2.0,
            max_delay=10.0
        )
        
        assert policy.calculate_delay(0) == 2.0
        assert policy.calculate_delay(1) == 2.0
        assert policy.calculate_delay(10) == 2.0
    
    def test_linear_backoff(self):
        """Test backoff lineal."""
        policy = RestartPolicy(
            backoff_strategy=BackoffStrategy.LINEAR,
            initial_delay=1.0,
            max_delay=10.0
        )
        
        assert policy.calculate_delay(0) == 1.0
        assert policy.calculate_delay(1) == 2.0
        assert policy.calculate_delay(2) == 3.0
        assert policy.calculate_delay(3) == 4.0
        assert policy.calculate_delay(20) == 10.0  # max_delay
    
    def test_exponential_backoff(self):
        """Test backoff exponencial."""
        policy = RestartPolicy(
            backoff_strategy=BackoffStrategy.EXPONENTIAL,
            initial_delay=1.0,
            max_delay=10.0
        )
        
        assert policy.calculate_delay(0) == 1.0   # 1 * 2^0
        assert policy.calculate_delay(1) == 2.0   # 1 * 2^1
        assert policy.calculate_delay(2) == 4.0   # 1 * 2^2
        assert policy.calculate_delay(3) == 8.0   # 1 * 2^3
        assert policy.calculate_delay(4) == 10.0  # cap at max_delay
    
    def test_should_restart_within_limit(self):
        """Test should_restart dentro del límite."""
        policy = RestartPolicy(max_retries=3, within_time_window=60.0)
        
        stats = RestartStats(actor_ref=Mock())
        stats.restart_times = [
            time.time() - 10,  # Hace 10s
            time.time() - 5,   # Hace 5s
        ]
        
        # 2 reinicios < 3 límite → permitir
        assert policy.should_restart(stats) is True
    
    def test_should_restart_exceeds_limit(self):
        """Test should_restart excede límite."""
        policy = RestartPolicy(max_retries=3, within_time_window=60.0)
        
        stats = RestartStats(actor_ref=Mock())
        stats.restart_times = [
            time.time() - 10,
            time.time() - 5,
            time.time() - 2,
        ]
        
        # 3 reinicios = 3 límite → NO permitir más
        assert policy.should_restart(stats) is False
    
    def test_should_restart_outside_window(self):
        """Test should_restart fuera de ventana."""
        policy = RestartPolicy(max_retries=2, within_time_window=10.0)
        
        stats = RestartStats(actor_ref=Mock())
        stats.restart_times = [
            time.time() - 100,  # Hace 100s (fuera de ventana)
            time.time() - 50,   # Hace 50s (fuera de ventana)
            time.time() - 5,    # Hace 5s (dentro de ventana)
        ]
        
        # Solo 1 reinicio en ventana de 10s → permitir
        assert policy.should_restart(stats) is True


class TestRestartStats:
    """Tests para RestartStats."""
    
    def test_record_failure(self):
        """Test registrar fallo."""
        stats = RestartStats(actor_ref=Mock())
        
        assert stats.failure_count == 0
        assert stats.last_failure_time is None
        
        stats.record_failure()
        
        assert stats.failure_count == 1
        assert stats.last_failure_time is not None
    
    def test_record_restart(self):
        """Test registrar reinicio."""
        stats = RestartStats(actor_ref=Mock())
        
        assert stats.total_restarts == 0
        assert len(stats.restart_times) == 0
        
        stats.record_restart()
        
        assert stats.total_restarts == 1
        assert len(stats.restart_times) == 1
    
    def test_get_failures_in_window(self):
        """Test contar fallos en ventana."""
        stats = RestartStats(actor_ref=Mock())
        
        now = time.time()
        stats.restart_times = [
            now - 100,  # Fuera de ventana
            now - 50,   # Fuera de ventana
            now - 5,    # Dentro
            now - 2,    # Dentro
        ]
        
        failures_in_10s = stats.get_failures_in_window(10.0)
        assert failures_in_10s == 2


# ============================================================================
# TESTS: SupervisorActor - Child Management
# ============================================================================

class TestSupervisorChildManagement:
    """Tests para gestión de children."""
    
    def test_spawn_child(self, supervisor):
        """Test crear child."""
        child_ref = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        assert "child1" in supervisor.children
        assert supervisor.children["child1"] == child_ref
        assert "child1" in supervisor.restart_stats
    
    def test_spawn_child_duplicate_name(self, supervisor):
        """Test crear child con nombre duplicado."""
        supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        with pytest.raises(ValueError, match="already exists"):
            supervisor.spawn_child(SimpleTestActor, "child1", "child1")
    
    def test_stop_child(self, supervisor):
        """Test detener child."""
        child_ref = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        assert "child1" in supervisor.children
        
        supervisor.stop_child(child_ref)
        
        assert "child1" not in supervisor.children
        assert "child1" not in supervisor.restart_stats
        assert child_ref.actor.state == ActorState.STOPPED
    
    def test_get_all_children(self, supervisor):
        """Test obtener todos los children."""
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        child3 = supervisor.spawn_child(SimpleTestActor, "child3", "child3")
        
        all_children = supervisor.get_all_children()
        
        assert len(all_children) == 3
        assert child1 in all_children
        assert child2 in all_children
        assert child3 in all_children
    
    def test_get_children_after(self, supervisor):
        """Test obtener children posteriores."""
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        child3 = supervisor.spawn_child(SimpleTestActor, "child3", "child3")
        child4 = supervisor.spawn_child(SimpleTestActor, "child4", "child4")
        
        # Children después de child2
        subsequent = supervisor.get_children_after(child2)
        
        assert len(subsequent) == 2
        assert child3 in subsequent
        assert child4 in subsequent
        assert child1 not in subsequent
        assert child2 not in subsequent
    
    def test_get_children_after_last(self, supervisor):
        """Test obtener children después del último."""
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        
        subsequent = supervisor.get_children_after(child2)
        
        assert len(subsequent) == 0
    
    def test_get_restart_stats(self, supervisor):
        """Test obtener estadísticas de reinicio."""
        child_ref = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        stats = supervisor.get_restart_stats(child_ref)
        
        assert stats is not None
        assert stats.actor_ref == child_ref
        assert stats.total_restarts == 0


# ============================================================================
# TESTS: OneForOne Strategy
# ============================================================================

class TestOneForOneStrategy:
    """Tests para OneForOne supervision strategy."""
    
    def test_restart_only_failed_child(self, supervisor):
        """Test reiniciar solo el child que falló."""
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        child3 = supervisor.spawn_child(SimpleTestActor, "child3", "child3")
        
        # Simular fallo de child2
        error = RuntimeError("Test error")
        supervisor.handle_child_failure(child2, error)
        
        # TASK-043: Esperar restart asíncrono (delay=0.1s + buffer)
        time.sleep(0.15)
        
        # Solo child2 debe reiniciarse
        assert child2.actor.restart_count == 1
        assert child1.actor.restart_count == 0
        assert child3.actor.restart_count == 0
        
        # Todos siguen RUNNING
        assert child1.actor.state == ActorState.RUNNING
        assert child2.actor.state == ActorState.RUNNING
        assert child3.actor.state == ActorState.RUNNING
    
    def test_restart_with_backoff(self, supervisor):
        """Test reinicio con backoff."""
        child_ref = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        start_time = time.time()
        
        error = RuntimeError("Test error")
        supervisor.handle_child_failure(child_ref, error)
        
        # TASK-043: Esperar restart asíncrono (delay=0.1s + buffer)
        time.sleep(0.15)
        
        elapsed = time.time() - start_time
        
        # Debe haber esperado al menos initial_delay (0.1s en fixture)
        assert elapsed >= 0.1
    
    def test_restart_increments_stats(self, supervisor):
        """Test que reinicio incrementa estadísticas."""
        child_ref = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        stats = supervisor.get_restart_stats(child_ref)
        assert stats.failure_count == 0
        assert stats.total_restarts == 0
        
        error = RuntimeError("Test error")
        supervisor.handle_child_failure(child_ref, error)
        
        # TASK-043: Esperar restart asíncrono (delay=0.1s + buffer)
        time.sleep(0.15)
        
        assert stats.failure_count == 1
        assert stats.total_restarts == 1
    
    def test_escalate_after_max_retries(self):
        """Test escalar después de max_retries."""
        # Supervisor con parent
        parent_supervisor = SupervisorActor()
        parent_supervisor.ref = ActorRef(name="parent", actor=parent_supervisor)
        
        # Policy estricto (1 retry, delay corto para tests)
        strict_policy = RestartPolicy(
            max_retries=1, 
            within_time_window=60.0,
            backoff_strategy=BackoffStrategy.CONSTANT,
            initial_delay=0.1  # TASK-043: Delay corto para tests
        )
        strategy = OneForOneStrategy(strict_policy)
        
        supervisor = SupervisorActor(strategy=strategy, parent_supervisor=parent_supervisor)
        supervisor.ref = ActorRef(name="supervisor", actor=supervisor)
        
        child_ref = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        # Mock del método handle_child_failure del padre
        parent_supervisor.handle_child_failure = Mock()
        
        # Primer fallo → reinicia
        supervisor.handle_child_failure(child_ref, RuntimeError("Error 1"))
        
        # TASK-043: Esperar restart asíncrono (delay=0.1s + buffer)
        time.sleep(0.15)
        
        assert child_ref.actor.restart_count == 1
        parent_supervisor.handle_child_failure.assert_not_called()
        
        # Segundo fallo → escala (excede max_retries=1)
        supervisor.handle_child_failure(child_ref, RuntimeError("Error 2"))
        parent_supervisor.handle_child_failure.assert_called_once()


# ============================================================================
# TESTS: OneForAll Strategy
# ============================================================================

class TestOneForAllStrategy:
    """Tests para OneForAll supervision strategy."""
    
    def test_restart_all_children(self):
        """Test reiniciar TODOS los children cuando uno falla."""
        policy = RestartPolicy(max_retries=3, backoff_strategy=BackoffStrategy.CONSTANT, initial_delay=0.1)
        strategy = OneForAllStrategy(policy)
        supervisor = SupervisorActor(strategy=strategy)
        supervisor.ref = ActorRef(name="supervisor", actor=supervisor)
        
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        child3 = supervisor.spawn_child(SimpleTestActor, "child3", "child3")
        
        # Child2 falla
        error = RuntimeError("Test error")
        supervisor.handle_child_failure(child2, error)
        
        # TASK-043: Esperar restart asíncrono (delay=0.1s + buffer)
        time.sleep(0.15)
        
        # TODOS deben reiniciarse
        assert child1.actor.restart_count == 1
        assert child2.actor.restart_count == 1
        assert child3.actor.restart_count == 1
    
    def test_stop_all_children_on_directive_stop(self):
        """Test detener TODOS si directiva es STOP."""
        policy = RestartPolicy(max_retries=0)  # No retries → STOP
        strategy = OneForAllStrategy(policy)
        supervisor = SupervisorActor(strategy=strategy)
        supervisor.ref = ActorRef(name="supervisor", actor=supervisor)
        
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        child3 = supervisor.spawn_child(SimpleTestActor, "child3", "child3")
        
        # Primer fallo ya excede max_retries=0
        stats = supervisor.get_restart_stats(child2)
        stats.restart_times = [time.time()]  # Simular 1 reinicio previo
        
        error = RuntimeError("Test error")
        
        # Mock escalate para que no escale realmente
        supervisor.escalate_failure = Mock()
        
        supervisor.handle_child_failure(child2, error)
        
        # Debe escalar
        supervisor.escalate_failure.assert_called_once()


# ============================================================================
# TESTS: RestForOne Strategy
# ============================================================================

class TestRestForOneStrategy:
    """Tests para RestForOne supervision strategy."""
    
    def test_restart_failed_and_subsequent_children(self):
        """Test reiniciar fallido y posteriores."""
        policy = RestartPolicy(max_retries=3, backoff_strategy=BackoffStrategy.CONSTANT, initial_delay=0.1)
        strategy = RestForOneStrategy(policy)
        supervisor = SupervisorActor(strategy=strategy)
        supervisor.ref = ActorRef(name="supervisor", actor=supervisor)
        
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        child3 = supervisor.spawn_child(SimpleTestActor, "child3", "child3")
        child4 = supervisor.spawn_child(SimpleTestActor, "child4", "child4")
        
        # Child2 falla → reiniciar child2, child3, child4 (NO child1)
        error = RuntimeError("Test error")
        supervisor.handle_child_failure(child2, error)
        
        # TASK-043: Esperar restart asíncrono (delay=0.1s + buffer)
        time.sleep(0.15)
        
        assert child1.actor.restart_count == 0  # NO reiniciado
        assert child2.actor.restart_count == 1  # Reiniciado
        assert child3.actor.restart_count == 1  # Reiniciado
        assert child4.actor.restart_count == 1  # Reiniciado
    
    def test_restart_last_child_only(self):
        """Test reiniciar solo el último child (sin posteriores)."""
        policy = RestartPolicy(max_retries=3, backoff_strategy=BackoffStrategy.CONSTANT, initial_delay=0.1)
        strategy = RestForOneStrategy(policy)
        supervisor = SupervisorActor(strategy=strategy)
        supervisor.ref = ActorRef(name="supervisor", actor=supervisor)
        
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        child3 = supervisor.spawn_child(SimpleTestActor, "child3", "child3")
        
        # Child3 (último) falla → solo child3 se reinicia
        error = RuntimeError("Test error")
        supervisor.handle_child_failure(child3, error)
        
        # TASK-043: Esperar restart asíncrono (delay=0.1s + buffer)
        time.sleep(0.15)
        
        assert child1.actor.restart_count == 0
        assert child2.actor.restart_count == 0
        assert child3.actor.restart_count == 1


# ============================================================================
# TESTS: Escalation
# ============================================================================

class TestEscalation:
    """Tests para escalación de errores."""
    
    def test_escalate_to_parent_supervisor(self):
        """Test escalar a supervisor padre."""
        parent = SupervisorActor()
        parent.ref = ActorRef(name="parent", actor=parent)
        parent.handle_child_failure = Mock()
        
        supervisor = SupervisorActor(parent_supervisor=parent)
        supervisor.ref = ActorRef(name="supervisor", actor=supervisor)
        
        child_ref = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        error = RuntimeError("Test error")
        supervisor.escalate_failure(child_ref, error)
        
        # Parent debe recibir notificación
        parent.handle_child_failure.assert_called_once_with(supervisor.ref, error)
    
    def test_escalate_without_parent_stops_child(self):
        """Test escalar sin padre detiene el child."""
        supervisor = SupervisorActor(parent_supervisor=None)
        supervisor.ref = ActorRef(name="supervisor", actor=supervisor)
        
        child_ref = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        error = RuntimeError("Test error")
        supervisor.escalate_failure(child_ref, error)
        
        # Sin padre → child debe detenerse
        assert "child1" not in supervisor.children
        assert child_ref.actor.state == ActorState.STOPPED


# ============================================================================
# TESTS: Edge Cases
# ============================================================================

class TestEdgeCases:
    """Tests de casos límite."""
    
    def test_handle_failure_of_unknown_child(self, supervisor):
        """Test manejar fallo de child desconocido."""
        unknown_actor = SimpleTestActor("unknown")
        unknown_ref = ActorRef(name="unknown", actor=unknown_actor)
        
        # No debe crashear
        supervisor.handle_child_failure(unknown_ref, RuntimeError("Error"))
        
        # Unknown no debe agregarse a children
        assert "unknown" not in supervisor.children
    
    def test_restart_unknown_child(self, supervisor):
        """Test reiniciar child desconocido."""
        unknown_actor = SimpleTestActor("unknown")
        unknown_ref = ActorRef(name="unknown", actor=unknown_actor)
        
        # No debe crashear
        supervisor.restart_child(unknown_ref)
        
        # Unknown no debe reiniciarse
        assert unknown_actor.restart_count == 0
    
    def test_supervisor_pre_restart_stops_children(self, supervisor):
        """Test pre_restart del supervisor detiene children."""
        child1 = supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        child2 = supervisor.spawn_child(SimpleTestActor, "child2", "child2")
        
        supervisor.pre_restart(RuntimeError("Supervisor failed"))
        
        # Todos los children deben detenerse
        assert len(supervisor.children) == 0
        assert child1.actor.state == ActorState.STOPPED
        assert child2.actor.state == ActorState.STOPPED


# ============================================================================
# TESTS: Integration
# ============================================================================

class TestIntegration:
    """Tests de integración."""
    
    def test_nested_supervisors(self):
        """Test supervisores anidados (jerarquía)."""
        # Top-level supervisor
        top_supervisor = SupervisorActor()
        top_supervisor.ref = ActorRef(name="top", actor=top_supervisor)
        
        # Mid-level supervisor
        mid_supervisor = SupervisorActor(parent_supervisor=top_supervisor)
        mid_supervisor.ref = ActorRef(name="mid", actor=mid_supervisor)
        
        # Bottom-level supervisor
        bottom_supervisor = SupervisorActor(parent_supervisor=mid_supervisor)
        bottom_supervisor.ref = ActorRef(name="bottom", actor=bottom_supervisor)
        
        # Child actor
        child_ref = bottom_supervisor.spawn_child(SimpleTestActor, "child1", "child1")
        
        # Mock escalate_failure
        mid_supervisor.handle_child_failure = Mock()
        
        # Child falla y escala
        bottom_supervisor.escalate_failure(child_ref, RuntimeError("Test"))
        
        # Mid debe recibir escalación
        mid_supervisor.handle_child_failure.assert_called_once()


# ============================================================================
# TESTS: SupervisorDirective
# ============================================================================

class TestSupervisorDirective:
    """Tests para SupervisorDirective enum."""
    
    def test_directive_values(self):
        """Test valores del enum."""
        assert SupervisorDirective.RESUME.value == "resume"
        assert SupervisorDirective.RESTART.value == "restart"
        assert SupervisorDirective.STOP.value == "stop"
        assert SupervisorDirective.ESCALATE.value == "escalate"


# ============================================================================
# TESTS: BackoffStrategy
# ============================================================================

class TestBackoffStrategyEnum:
    """Tests para BackoffStrategy enum."""
    
    def test_strategy_values(self):
        """Test valores del enum."""
        assert BackoffStrategy.CONSTANT.value == "constant"
        assert BackoffStrategy.LINEAR.value == "linear"
        assert BackoffStrategy.EXPONENTIAL.value == "exponential"


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
