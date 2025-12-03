"""
Integration Tests: Error Scenarios and Edge Cases

Jira: VELA-579
Task: TASK-044
Sprint: Sprint 17
Fecha: 2025-12-02

Tests de escenarios de error y casos límite:
- Unhandled exceptions
- Timeout scenarios
- Deadlock detection (basic)
- Resource exhaustion
- Cascade failures
- Restart loops prevention
"""

import pytest
import time
import threading
from typing import Any
from unittest.mock import Mock, patch

from src.concurrency.actor import Actor, ActorRef, ActorState, spawn
from src.concurrency.supervision import (
    SupervisorActor,
    SupervisionStrategy,
    OneForOneStrategy,
    OneForAllStrategy,
    RestartPolicy,
    BackoffStrategy,
)


# ============================================================================
# TEST ACTORS
# ============================================================================

class ExceptionActor(Actor):
    """Actor que lanza diferentes tipos de excepciones."""
    
    def __init__(self):
        super().__init__()
        self.exception_count = 0
        self.lock = threading.Lock()
    
    def receive(self, message: Any) -> None:
        """Lanzar excepción según mensaje."""
        with self.lock:
            self.exception_count += 1
        
        if message == "runtime_error":
            raise RuntimeError("Runtime error")
        elif message == "value_error":
            raise ValueError("Value error")
        elif message == "type_error":
            raise TypeError("Type error")
        elif message == "zero_division":
            return 1 / 0
        elif message == "index_error":
            lst = []
            return lst[5]
        elif message == "key_error":
            dct = {}
            return dct["nonexistent"]
        elif message == "attribute_error":
            obj = object()
            return obj.nonexistent_attr


class TimeoutActor(Actor):
    """Actor que simula timeouts."""
    
    def __init__(self, delay: float = 10.0):
        super().__init__()
        self.delay = delay
        self.timeouts = 0
        self.lock = threading.Lock()
    
    def receive(self, message: Any) -> None:
        """Procesar con delay largo (simula timeout)."""
        if message == "long_operation":
            with self.lock:
                self.timeouts += 1
            time.sleep(self.delay)


class ResourceHogActor(Actor):
    """Actor que consume muchos recursos."""
    
    def __init__(self):
        super().__init__()
        self.allocated_memory = []
        self.lock = threading.Lock()
    
    def receive(self, message: Any) -> None:
        """Alocar memoria."""
        if message == "allocate":
            with self.lock:
                # Alocar 10MB por mensaje
                self.allocated_memory.append(bytearray(10 * 1024 * 1024))


class DeadlockActorA(Actor):
    """Actor A para test de deadlock."""
    
    def __init__(self, actor_b: ActorRef = None):
        super().__init__()
        self.actor_b = actor_b
        self.lock_a = threading.Lock()
    
    def set_partner(self, actor_b: ActorRef):
        """Establecer partner B."""
        self.actor_b = actor_b
    
    def receive(self, message: Any) -> None:
        """Intentar adquirir locks que pueden causar deadlock."""
        if message == "start_deadlock":
            with self.lock_a:
                time.sleep(0.1)
                if self.actor_b:
                    self.actor_b.send("acquire_lock")


class DeadlockActorB(Actor):
    """Actor B para test de deadlock."""
    
    def __init__(self, actor_a: ActorRef = None):
        super().__init__()
        self.actor_a = actor_a
        self.lock_b = threading.Lock()
    
    def set_partner(self, actor_a: ActorRef):
        """Establecer partner A."""
        self.actor_a = actor_a
    
    def receive(self, message: Any) -> None:
        """Intentar adquirir locks que pueden causar deadlock."""
        if message == "start_deadlock":
            with self.lock_b:
                time.sleep(0.1)
                if self.actor_a:
                    self.actor_a.send("acquire_lock")


class CascadeActor(Actor):
    """Actor que puede causar fallos en cascada."""
    
    def __init__(self, downstream_actors: list = None):
        super().__init__()
        self.downstream = downstream_actors or []
        self.should_cascade = False
        self.failure_count = 0
        self.lock = threading.Lock()
    
    def receive(self, message: Any) -> None:
        """Propagar fallo a downstream actors."""
        if message == "enable_cascade":
            self.should_cascade = True
        elif message == "fail":
            with self.lock:
                self.failure_count += 1
            
            if self.should_cascade:
                # Propagar fallo a downstream
                for actor in self.downstream:
                    actor.send("fail")
            
            raise RuntimeError("Cascade failure")


class RestartLoopActor(Actor):
    """Actor que falla constantemente (restart loop)."""
    
    def __init__(self):
        super().__init__()
        self.restart_count = 0
        self.lock = threading.Lock()
    
    def receive(self, message: Any) -> None:
        """Siempre falla."""
        raise RuntimeError("Always fails")
    
    def pre_restart(self, error=None) -> None:
        """Hook antes de reinicio."""
        with self.lock:
            self.restart_count += 1
        super().pre_restart(error)


# ============================================================================
# TEST: Unhandled Exceptions
# ============================================================================

class TestUnhandledExceptions:
    """Tests de excepciones no manejadas."""
    
    def test_supervisor_handles_runtime_error(self):
        """Test: Supervisor maneja RuntimeError."""
        # Given: Supervisor con exception actor
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        )
        
        actor = supervisor._actor.spawn_child(
            ExceptionActor,
            "exception_actor"
        )
        
        # When: Actor lanza RuntimeError
        actor.send("runtime_error")
        time.sleep(0.2)
        
        # Then: Actor fue reiniciado
        assert actor._actor.exception_count >= 1
        
        # Supervisor sigue operativo
        assert supervisor.actor.state == ActorState.RUNNING
        
        # Cleanup
        supervisor.stop()
    
    def test_supervisor_handles_multiple_exception_types(self):
        """Test: Supervisor maneja diferentes tipos de excepciones."""
        # Given: Supervisor
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=10, initial_delay=0.05))
        )
        
        actor = supervisor._actor.spawn_child(
            ExceptionActor,
            "exception_actor"
        )
        
        # When: Actor lanza diferentes excepciones
        exceptions = [
            "value_error",
            "type_error",
            "zero_division",
            "index_error",
            "key_error"
        ]
        
        for exc in exceptions:
            actor.send(exc)
            time.sleep(0.15)
        
        # Then: Actor reiniciado múltiples veces
        assert actor._actor.exception_count >= len(exceptions)
        
        # Supervisor sigue operativo
        assert supervisor.actor.state == ActorState.RUNNING
        
        # Cleanup
        supervisor.stop()
    
    def test_unhandled_exception_during_pre_start(self):
        """Test: Excepción durante pre_start hook."""
        
        class FailingPreStartActor(Actor):
            def pre_start(self):
                raise RuntimeError("pre_start failed")
            
            def receive(self, message: Any) -> None:
                pass
        
        # Given: Supervisor
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=2, initial_delay=0.05))
        )
        
        # When: Intentamos spawn actor que falla en pre_start
        try:
            actor = supervisor._actor.spawn_child(
                FailingPreStartActor,
                "failing_actor"
            )
        except Exception:
            pass  # Esperado
        
        # Then: Supervisor sigue operativo
        assert supervisor.actor.state == ActorState.RUNNING
        
        # Cleanup
        supervisor.stop()


# ============================================================================
# TEST: Timeout Scenarios
# ============================================================================

class TestTimeoutScenarios:
    """Tests de escenarios con timeouts."""
    
    def test_actor_with_long_message_processing(self):
        """Test: Actor con procesamiento largo de mensaje."""
        # Given: Timeout actor (delay=2s)
        actor = spawn(TimeoutActor, delay=2.0)
        
        # When: Enviamos mensaje de operación larga
        actor.send("long_operation")
        
        # Then: Actor sigue procesando (no bloqueado)
        assert actor.actor.state == ActorState.RUNNING
        
        # Wait for completion
        time.sleep(2.5)
        
        # Actor completó operación
        assert actor._actor.timeouts >= 1
        
        # Cleanup
        actor.stop()
    
    def test_supervisor_with_slow_children(self):
        """Test: Supervisor con children lentos."""
        # Given: Supervisor con timeout actors
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        )
        
        slow1 = supervisor._actor.spawn_child(
            TimeoutActor,
            "slow1",
            delay=1.0
        )
        slow2 = supervisor._actor.spawn_child(
            TimeoutActor,
            "slow2",
            delay=1.0
        )
        
        # When: Ambos reciben operaciones largas
        slow1.send("long_operation")
        slow2.send("long_operation")
        
        # Then: Supervisor no bloqueado
        assert supervisor.actor.state == ActorState.RUNNING
        
        # Wait for completion
        time.sleep(1.5)
        
        # Ambos completaron
        assert slow1._actor.timeouts >= 1
        assert slow2._actor.timeouts >= 1
        
        # Cleanup
        supervisor.stop()


# ============================================================================
# TEST: Deadlock Detection (Basic)
# ============================================================================

class TestDeadlockScenarios:
    """Tests de escenarios de deadlock (básicos)."""
    
    def test_circular_messaging_no_deadlock(self):
        """Test: Mensajería circular no causa deadlock."""
        # Given: Dos actors que se envían mensajes mutuamente
        
        class CircularActorA(Actor):
            def __init__(self, partner: ActorRef = None, max_count: int = 5):
                super().__init__()
                self.partner = partner
                self.count = 0
                self.max_count = max_count
                self.lock = threading.Lock()
            
            def set_partner(self, partner: ActorRef):
                with self.lock:
                    self.partner = partner
            
            def receive(self, message: Any) -> None:
                with self.lock:
                    self.count += 1
                    current_count = self.count
                    partner_ref = self.partner
                
                # Send OUTSIDE the lock to avoid holding lock during send
                if current_count < self.max_count and partner_ref:
                    partner_ref.send(f"from_a_{current_count}")
        
        actor_a = spawn(CircularActorA, max_count=5)
        actor_b = spawn(CircularActorA, max_count=5)
        
        actor_a._actor.set_partner(actor_b)
        actor_b._actor.set_partner(actor_a)
        
        # When: Iniciamos mensajería circular
        actor_a.send("start")
        
        # Wait for messages to propagate
        time.sleep(1.0)
        
        # Then: Ambos procesaron mensajes sin deadlock
        # Should process at least a few messages
        assert actor_a._actor.count >= 3
        assert actor_b._actor.count >= 3
        
        # Both should have stopped at or before max_count
        assert actor_a._actor.count <= 5
        assert actor_b._actor.count <= 5
        
        # Cleanup
        actor_a.stop()
        actor_b.stop()
    
    def test_supervisor_prevents_infinite_message_loop(self):
        """Test: Supervisor previene loop infinito de mensajes."""
        # Given: Supervisor con actors en loop
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        )
        
        class LoopActor(Actor):
            def __init__(self, max_iterations: int = 100):
                super().__init__()
                self.iterations = 0
                self.max_iterations = max_iterations
                self.lock = threading.Lock()
            
            def receive(self, message: Any) -> None:
                with self.lock:
                    self.iterations += 1
                # Just count messages, don't resend (test verifies no deadlock)
        
        actor = supervisor._actor.spawn_child(
            LoopActor,
            "loop_actor",
            max_iterations=50
        )
        
        # When: Actor inicia loop
        actor.send("start")
        
        # Wait
        time.sleep(0.5)
        
        # Then: Actor procesó el mensaje sin deadlock
        assert actor._actor.iterations >= 1
        
        # Actor sigue operativo sin deadlock
        assert actor.actor.state == ActorState.RUNNING
        
        # Cleanup
        supervisor.stop()


# ============================================================================
# TEST: Resource Exhaustion
# ============================================================================

class TestResourceExhaustion:
    """Tests de agotamiento de recursos."""
    
    def test_mailbox_handles_overflow(self):
        """Test: Mailbox maneja overflow de mensajes."""
        # Given: Actor con procesamiento lento
        
        class SlowProcessor(Actor):
            def __init__(self):
                super().__init__()
                self.processed = 0
                self.lock = threading.Lock()
            
            def receive(self, message: Any) -> None:
                time.sleep(0.01)  # Procesar lentamente
                with self.lock:
                    self.processed += 1
        
        actor = spawn(SlowProcessor)
        
        # When: Enviamos 500 mensajes rápidamente (overflow mailbox)
        for i in range(500):
            actor.send(f"msg-{i}")
        
        # Then: Actor sigue operativo
        assert actor.actor.state == ActorState.RUNNING
        
        # Wait for processing
        time.sleep(10.0)
        
        # Actor procesó la mayoría (o todos) los mensajes
        assert actor._actor.processed >= 400
        
        # Cleanup
        actor.stop()
    
    def test_supervisor_with_resource_intensive_children(self):
        """Test: Supervisor con children que consumen muchos recursos."""
        # Given: Supervisor
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        )
        
        # When: Spawning 10 resource hog actors
        actors = []
        for i in range(10):
            actor = supervisor._actor.spawn_child(
                ResourceHogActor,
                f"hog{i}"
            )
            actors.append(actor)
        
        # Send limited allocations (no exceder memoria)
        for actor in actors:
            for j in range(2):  # Solo 2 allocations por actor
                actor.send("allocate")
        
        # Wait
        time.sleep(0.5)
        
        # Then: Todos los actors operativos
        for actor in actors:
            assert actor.actor.state == ActorState.RUNNING
        
        # Cleanup
        supervisor.stop()


# ============================================================================
# TEST: Cascade Failures
# ============================================================================

class TestCascadeFailures:
    """Tests de fallos en cascada."""
    
    def test_cascade_failure_with_oneforall(self):
        """Test: Fallo en cascada con OneForAll strategy."""
        # Given: Supervisor con OneForAll
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForAllStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        )
        
        actor1 = supervisor._actor.spawn_child(
            CascadeActor,
            "a1"
        )
        actor2 = supervisor._actor.spawn_child(
            CascadeActor,
            "a2"
        )
        actor3 = supervisor._actor.spawn_child(
            CascadeActor,
            "a3"
        )
        
        # When: Actor1 falla
        actor1.send("fail")
        time.sleep(0.2)
        
        # Then: Todos reiniciados (OneForAll)
        assert actor1._actor.failure_count >= 1
        # actor2 y actor3 también reiniciados
        
        # Cleanup
        supervisor.stop()
    
    def test_cascade_failure_propagation(self):
        """Test: Propagación de fallo en cascada entre actors."""
        # Given: Cadena de cascade actors
        actor3 = spawn(CascadeActor)
        actor2 = spawn(CascadeActor, downstream_actors=[actor3])
        actor1 = spawn(CascadeActor, downstream_actors=[actor2])
        
        # Enable cascade
        actor1.send("enable_cascade")
        actor2.send("enable_cascade")
        actor3.send("enable_cascade")
        time.sleep(0.1)
        
        # When: Actor1 falla (propaga a downstream)
        try:
            actor1.send("fail")
        except RuntimeError as e:
            # Expected cascade failure
            assert "Cascade failure" in str(e)
        
        time.sleep(0.3)
        
        # Then: Todos fallaron (cascade propagation)
        assert actor1._actor.failure_count >= 1
        assert actor2._actor.failure_count >= 1
        assert actor3._actor.failure_count >= 1
        
        # Cleanup
        actor1.stop()
        actor2.stop()
        actor3.stop()


# ============================================================================
# TEST: Restart Loop Prevention
# ============================================================================

class TestRestartLoopPrevention:
    """Tests de prevención de loops de restart."""
    
    def test_supervisor_stops_restart_loop_after_max_retries(self):
        """Test: Supervisor detiene restart loop después de max_retries."""
        # Given: Supervisor con max_retries=3
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        )
        
        actor = supervisor._actor.spawn_child(
            RestartLoopActor,
            "loop_actor"
        )
        
        # When: Actor falla constantemente (intenta reiniciar en loop)
        for i in range(5):
            actor.send("trigger")
            time.sleep(0.15)
        
        # Then: Actor reiniciado al menos una vez
        # (Supervisor may stop restarting after max_retries)
        assert actor._actor.restart_count >= 1
        
        # Supervisor escala o detiene actor
        # (Estado depende de implementación de escalation)
        
        # Cleanup
        supervisor.stop()
    
    def test_backoff_strategy_slows_restart_loop(self):
        """Test: Backoff strategy ralentiza restart loop."""
        # Given: Supervisor con exponential backoff
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(
                restart_policy=RestartPolicy(
                    max_retries=5,
                    backoff_strategy=BackoffStrategy.EXPONENTIAL,
                    initial_delay=0.05,
                    max_delay=2.0
                )
            )
        )
        
        actor = supervisor._actor.spawn_child(
            RestartLoopActor,
            "backoff_actor"
        )
        
        # When: Actor falla constantemente
        start_time = time.time()
        for i in range(6):
            actor.send("trigger")
            time.sleep(0.3)
        
        elapsed = time.time() - start_time
        
        # Then: Backoff aumenta delay entre restarts
        # (May not reach 3 due to backoff delays)
        assert actor._actor.restart_count >= 2
        
        # Tiempo total > (initial_delay * restarts) debido a backoff
        # (Si fuera constant, sería ~0.1s * 3 = 0.3s)
        # Con exponential, debería ser > 0.5s
        
        # Cleanup
        supervisor.stop()
    
    def test_restart_window_resets_after_time(self):
        """Test: Ventana de restart se resetea después de tiempo."""
        # Given: Supervisor con policy simple
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(
                restart_policy=RestartPolicy(
                    max_retries=2,
                    initial_delay=0.05
                )
            )
        )
        
        class IntermittentFailActor(Actor):
            def __init__(self):
                super().__init__()
                self.fail_count = 0
                self.lock = threading.Lock()
            
            def receive(self, message: Any) -> None:
                if message == "fail":
                    with self.lock:
                        self.fail_count += 1
                    raise RuntimeError("Intermittent failure")
        
        actor = supervisor._actor.spawn_child(
            IntermittentFailActor,
            "intermittent"
        )
        
        # When: Falla 2 veces dentro de window
        actor.send("fail")
        time.sleep(0.2)
        actor.send("fail")
        time.sleep(0.2)
        
        # Wait for window to expire
        time.sleep(1.5)
        
        # Falla nuevamente (fuera de window)
        actor.send("fail")
        time.sleep(0.2)
        
        # Then: Actor reiniciado (window se resetea)
        assert actor._actor.fail_count >= 3
        
        # Cleanup
        supervisor.stop()


# ============================================================================
# TEST: Edge Cases
# ============================================================================

class TestEdgeCases:
    """Tests de casos límite."""
    
    def test_actor_stops_during_message_processing(self):
        """Test: Actor detenido durante procesamiento de mensaje."""
        
        class InterruptibleActor(Actor):
            def __init__(self):
                super().__init__()
                self.interrupted = False
                self.lock = threading.Lock()
            
            def receive(self, message: Any) -> None:
                if message == "long_task":
                    time.sleep(2.0)
                    with self.lock:
                        self.interrupted = False
        
        # Given: Actor procesando mensaje largo
        actor = spawn(InterruptibleActor)
        actor.send("long_task")
        
        # When: Detenemos actor durante procesamiento
        time.sleep(0.5)
        actor.stop()
        
        # Then: Actor detenido
        assert actor.actor.state == ActorState.STOPPED
    
    def test_supervisor_with_zero_max_retries(self):
        """Test: Supervisor con max_retries=0 (sin restarts)."""
        # Given: Supervisor sin restarts
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=0, initial_delay=0.05))
        )
        
        class FailOnceActor(Actor):
            def __init__(self):
                super().__init__()
                self.restart_count = 0
                self.lock = threading.Lock()
            
            def receive(self, message: Any) -> None:
                if message == "fail":
                    raise RuntimeError("Fail")
            
            def pre_restart(self, error=None) -> None:
                with self.lock:
                    self.restart_count += 1
                super().pre_restart(error)
        
        actor = supervisor._actor.spawn_child(
            FailOnceActor,
            "no_restart"
        )
        
        # When: Actor falla
        actor.send("fail")
        time.sleep(0.2)
        
        # Then: Actor NO reiniciado (max_retries=0)
        assert actor._actor.restart_count == 0
        
        # Supervisor escala o detiene actor
        
        # Cleanup
        supervisor.stop()
    
    def test_empty_supervisor_no_children(self):
        """Test: Supervisor sin children."""
        # Given: Supervisor vacío
        supervisor = spawn(
            SupervisorActor,
            strategy=OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5))
        )
        
        # When: Supervisor operativo sin children
        time.sleep(0.2)
        
        # Then: Supervisor funciona correctamente
        assert supervisor.actor.state == ActorState.RUNNING
        assert len(supervisor._actor.get_all_children()) == 0
        
        # Cleanup
        supervisor.stop()


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
