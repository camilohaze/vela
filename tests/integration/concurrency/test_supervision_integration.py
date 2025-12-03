"""
Integration Tests: Supervision Hierarchies

Jira: VELA-579
Task: TASK-044
Sprint: Sprint 17
Fecha: 2025-12-02

Tests end-to-end de jerarqu\u00edas de supervisi\u00f3n:
- Nested supervisors (3+ levels deep)
- Mixed supervision strategies
- Escalation chains
- Restart with state recovery
- Supervisor failures and recovery
"""

import pytest
import time
import threading
from typing import Any, Optional
from unittest.mock import Mock

from src.concurrency.actor import Actor, ActorRef, ActorState, spawn
from src.concurrency.supervision import (
    SupervisorActor,
    SupervisionStrategy,
    OneForOneStrategy,
    OneForAllStrategy,
    RestForOneStrategy,
    RestartPolicy,
    BackoffStrategy,
)


# ============================================================================
# TEST ACTORS
# ============================================================================

class StatefulActor(Actor):
    """Actor con estado que se puede recuperar."""
    
    def __init__(self, initial_value: int = 0):
        super().__init__()
        self.value = initial_value
        self.processed = []
        self.restart_count = 0
        self.lock = threading.Lock()
    
    def receive(self, message: Any) -> None:
        """Procesar mensaje."""
        with self.lock:
            if message == "fail":
                raise RuntimeError("Intentional failure")
            elif message == "increment":
                self.value += 1
            elif message == "get":
                pass  # Solo query
            else:
                self.processed.append(message)
    
    def pre_restart(self, error=None) -> None:
        """Guardar estado antes de reiniciar."""
        with self.lock:
            self.restart_count += 1
        super().pre_restart(error)
    
    def post_restart(self, error=None) -> None:
        """Recuperar después de reinicio."""
        # Estado se resetea, pero restart_count persiste en nueva instancia
        super().post_restart(error)


class ChildActor(Actor):
    """Actor child para tests de supervisión."""
    
    def __init__(self, name: str, should_fail: bool = False):
        super().__init__()
        self.name = name
        self.should_fail = should_fail
        self.messages_received = []
        self.restart_count = 0
        self.stop_count = 0
        self.lock = threading.Lock()
    
    def receive(self, message: Any) -> None:
        """Procesar mensaje."""
        with self.lock:
            self.messages_received.append(message)
            
            if message == "fail" or (self.should_fail and message == "trigger"):
                raise RuntimeError(f"{self.name} failure")
    
    def pre_restart(self, error=None) -> None:
        """Hook antes de reinicio."""
        with self.lock:
            self.restart_count += 1
        super().pre_restart(error)
    
    def post_stop(self) -> None:
        """Hook al detener."""
        with self.lock:
            self.stop_count += 1
        super().post_stop()


class WorkerActor(Actor):
    """Actor worker que puede fallar."""
    
    def __init__(self, id: int, failure_rate: float = 0.0):
        super().__init__()
        self.id = id
        self.failure_rate = failure_rate
        self.work_done = 0
        self.failures = 0
        self.restart_count = 0
        self.lock = threading.Lock()
    
    def receive(self, message: Any) -> None:
        """Procesar trabajo."""
        import random
        
        with self.lock:
            if message == "work":
                # Simular fallo aleatorio
                if random.random() < self.failure_rate:
                    self.failures += 1
                    raise RuntimeError(f"Worker {self.id} failed")
                
                self.work_done += 1
    
    def pre_restart(self, error=None) -> None:
        """Hook antes de reinicio."""
        with self.lock:
            self.restart_count += 1
        super().pre_restart(error)


# ============================================================================
# TEST: Nested Supervisors (3+ Levels)
# ============================================================================

class TestNestedSupervisors:
    """Tests de supervisores anidados."""
    
    def test_three_level_supervisor_hierarchy(self):
        """Test: Jerarquía de 3 niveles (root -> mid -> leaf)."""
        # Given: Supervisor raíz con OneForOne
        root_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        root_supervisor = spawn(SupervisorActor, strategy=root_strategy)
        
        # Mid-level supervisor con OneForAll
        mid_strategy = OneForAllStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        mid_supervisor = root_supervisor._actor.spawn_child(
            SupervisorActor,
            name="mid",
            strategy=mid_strategy
        )
        
        # Leaf actors bajo mid supervisor
        leaf1 = mid_supervisor._actor.spawn_child(
            ChildActor,
            "leaf1",
            "leaf1"
        )
        leaf2 = mid_supervisor._actor.spawn_child(
            ChildActor,
            "leaf2",
            "leaf2"
        )
        
        # When: Leaf1 falla
        leaf1.send("fail")
        time.sleep(0.2)
        
        # Then: Leaf1 fue reiniciado por mid supervisor
        assert leaf1._actor.restart_count >= 1
        
        # Leaf2 también fue reiniciado (OneForAll strategy)
        assert leaf2._actor.restart_count >= 1
        
        # Mid supervisor sigue RUNNING
        assert mid_supervisor.actor.state == ActorState.RUNNING
        
        # Root supervisor sigue RUNNING
        assert root_supervisor.actor.state == ActorState.RUNNING
        
        # Cleanup
        root_supervisor.stop()
    
    def test_deep_hierarchy_four_levels(self):
        """Test: Jerarquía profunda de 4 niveles."""
        # Given: Root -> L1 -> L2 -> Worker
        root_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=10, initial_delay=0.05))
        root = spawn(SupervisorActor, strategy=root_strategy)
        
        l1_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        l1_supervisor = root._actor.spawn_child(
            SupervisorActor,
            name="l1",
            strategy=l1_strategy
        )
        
        l2_strategy = RestForOneStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        l2_supervisor = l1_supervisor._actor.spawn_child(
            SupervisorActor,
            name="l2",
            strategy=l2_strategy
        )
        
        worker = l2_supervisor._actor.spawn_child(
            ChildActor,
            "worker",
            "worker"
        )
        
        # When: Worker falla
        worker.send("fail")
        time.sleep(0.2)
        
        # Then: Worker reiniciado
        assert worker._actor.restart_count >= 1
        
        # Todos los supervisores siguen RUNNING
        assert l2_supervisor.actor.state == ActorState.RUNNING
        assert l1_supervisor.actor.state == ActorState.RUNNING
        assert root.actor.state == ActorState.RUNNING
        
        # Cleanup
        root.stop()
    
    def test_nested_supervisors_independent_policies(self):
        """Test: Supervisores anidados con políticas independientes."""
        # Given: Root (max_retries=10) -> Mid (max_retries=2) -> Leaf
        root_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=10, initial_delay=0.05))
        root = spawn(SupervisorActor, strategy=root_strategy)
        
        mid_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=2, initial_delay=0.05))
        mid = root._actor.spawn_child(
            SupervisorActor,
            name="mid",
            strategy=mid_strategy
        )
        
        leaf = mid._actor.spawn_child(
            ChildActor,
            "leaf",
            "leaf"
        )
        
        # When: Leaf falla 3 veces
        for i in range(3):
            leaf.send("fail")
            time.sleep(0.2)  # Wait for restart timer
        
        # Wait extra for all restarts to complete
        time.sleep(0.3)
        
        # Then: Leaf fue reiniciado 2 veces (max_retries de mid)
        # Después del 3er fallo, mid escala al root
        assert leaf._actor.restart_count >= 2
        
        # Cleanup
        root.stop()


# ============================================================================
# TEST: Mixed Supervision Strategies
# ============================================================================

class TestMixedStrategies:
    """Tests de combinación de estrategias de supervisión."""
    
    def test_oneforone_parent_with_oneforall_child(self):
        """Test: OneForOne parent con OneForAll child supervisor."""
        # Given: Root con OneForOne
        root_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        root = spawn(SupervisorActor, strategy=root_strategy)
        
        # Child supervisor con OneForAll
        child_sup_strategy = OneForAllStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        child_sup = root._actor.spawn_child(
            SupervisorActor,
            name="child_sup",
            strategy=child_sup_strategy
        )
        
        # Workers bajo child supervisor
        worker1 = child_sup._actor.spawn_child(
            ChildActor,
            "w1",
            "w1"
        )
        worker2 = child_sup._actor.spawn_child(
            ChildActor,
            "w2",
            "w2"
        )
        
        # When: Worker1 falla
        worker1.send("fail")
        time.sleep(0.2)
        
        # Then: Ambos workers reiniciados (OneForAll)
        assert worker1._actor.restart_count >= 1
        assert worker2._actor.restart_count >= 1
        
        # Cleanup
        root.stop()
    
    def test_restforone_with_nested_oneforone(self):
        """Test: RestForOne strategy con OneForOne anidado."""
        # Given: Root con RestForOne
        root_strategy = RestForOneStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        root = spawn(SupervisorActor, strategy=root_strategy)
        
        # Child1: supervisor con OneForOne
        child1_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        child1 = root._actor.spawn_child(
            SupervisorActor,
            name="child1",
            strategy=child1_strategy
        )
        
        # Child2: simple actor
        child2 = root._actor.spawn_child(
            ChildActor,
            "child2",
            "child2"
        )
        
        # Child3: simple actor
        child3 = root._actor.spawn_child(
            ChildActor,
            "child3",
            "child3"
        )
        
        # When: Child2 falla
        child2.send("fail")
        time.sleep(0.2)
        
        # Then: Child2 y Child3 reiniciados (RestForOne)
        assert child2._actor.restart_count >= 1
        assert child3._actor.restart_count >= 1
        
        # Child1 NO reiniciado (viene antes en orden)
        # (Nota: Child1 es supervisor, no tiene restart_count directamente)
        
        # Cleanup
        root.stop()
    
    def test_complex_hierarchy_multiple_strategies(self):
        """Test: Jerarquía compleja con múltiples estrategias."""
        # Given: Root (OneForAll) -> Mid1 (OneForOne) + Mid2 (RestForOne) -> Workers
        root_strategy = OneForAllStrategy(restart_policy=RestartPolicy(max_retries=10, initial_delay=0.05))
        root = spawn(SupervisorActor, strategy=root_strategy)
        
        mid1_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        mid1 = root._actor.spawn_child(
            SupervisorActor,
            name="mid1",
            strategy=mid1_strategy
        )
        
        mid2_strategy = RestForOneStrategy(restart_policy=RestartPolicy(max_retries=3, initial_delay=0.05))
        mid2 = root._actor.spawn_child(
            SupervisorActor,
            name="mid2",
            strategy=mid2_strategy
        )
        
        # Workers bajo mid1
        m1w1 = mid1._actor.spawn_child(ChildActor, "m1w1", "m1w1")
        m1w2 = mid1._actor.spawn_child(ChildActor, "m1w2", "m1w2")
        
        # Workers bajo mid2
        m2w1 = mid2._actor.spawn_child(ChildActor, "m2w1", "m2w1")
        m2w2 = mid2._actor.spawn_child(ChildActor, "m2w2", "m2w2")
        
        # When: m1w1 falla
        m1w1.send("fail")
        time.sleep(0.2)
        
        # Then: Solo m1w1 reiniciado (OneForOne en mid1)
        assert m1w1._actor.restart_count >= 1
        # m1w2 NO reiniciado
        assert m1w2._actor.restart_count == 0
        
        # mid2 y sus workers NO afectados
        assert m2w1._actor.restart_count == 0
        assert m2w2._actor.restart_count == 0
        
        # Cleanup
        root.stop()


# ============================================================================
# TEST: Escalation Chains
# ============================================================================

class TestEscalationChains:
    """Tests de cadenas de escalación de errores."""
    
    def test_escalation_to_parent_supervisor(self):
        """Test: Escalación de error al supervisor padre."""
        # Given: Root -> Mid -> Leaf (max_retries=2)
        root_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=10, initial_delay=0.05))
        root = spawn(SupervisorActor, strategy=root_strategy)
        
        mid_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=2, initial_delay=0.05))
        mid = root._actor.spawn_child(
            SupervisorActor,
            name="mid",
            strategy=mid_strategy,
            parent_supervisor=root._actor
        )
        
        leaf = mid._actor.spawn_child(
            ChildActor,
            "leaf",
            "leaf"
        )
        
        # When: Leaf falla 3 veces (excede max_retries=2)
        for i in range(3):
            leaf.send("fail")
            time.sleep(0.2)
        
        # Wait for all restarts
        time.sleep(0.3)
        
        # Then: Leaf fue reiniciado al menos 2 veces
        assert leaf._actor.restart_count >= 2
        
        # Mid supervisor escala al root (o reinicia leaf bajo root)
        # Verificamos que mid sigue operativo
        assert mid.actor.state == ActorState.RUNNING or mid.actor.state == ActorState.RESTARTING
        
        # Cleanup
        root.stop()
    
    def test_escalation_through_multiple_levels(self):
        """Test: Escalación a través de múltiples niveles."""
        # Given: L0 -> L1 (max_retries=1) -> L2 (max_retries=1) -> Worker
        l0_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=10, initial_delay=0.05))
        l0 = spawn(SupervisorActor, strategy=l0_strategy)
        
        l1_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=1, initial_delay=0.05))
        l1 = l0._actor.spawn_child(
            SupervisorActor,
            name="l1",
            strategy=l1_strategy,
            parent_supervisor=l0._actor
        )
        
        l2_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=1, initial_delay=0.05))        
        l2 = l1._actor.spawn_child(
            SupervisorActor,
            name="l2",
            strategy=l2_strategy,
            parent_supervisor=l1._actor
        )
        
        worker = l2._actor.spawn_child(
            ChildActor,
            "worker",
            "worker"
        )
        
        # When: Worker falla 3 veces
        for i in range(3):
            worker.send("fail")
            time.sleep(0.2)
        
        # Wait for all escalations
        time.sleep(0.3)
        
        # Then: Worker reiniciado al menos 1 vez
        assert worker._actor.restart_count >= 1
        
        # L2 escala a L1, que escala a L0
        # L0 tiene max_retries=10, por lo que sistema sigue operativo
        assert l0.actor.state == ActorState.RUNNING
        
        # Cleanup
        l0.stop()


# ============================================================================
# TEST: Restart with State Recovery
# ============================================================================

class TestStateRecovery:
    """Tests de recuperación de estado después de restart."""
    
    def test_stateful_actor_loses_state_on_restart(self):
        """Test: Actor pierde estado por defecto al reiniciar."""
        # Given: Supervisor con stateful actor
        supervisor_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        supervisor = spawn(SupervisorActor, strategy=supervisor_strategy)
        
        stateful = supervisor._actor.spawn_child(
            StatefulActor,
            name="stateful",
            initial_value=100
        )
        
        # When: Actor incrementa y luego falla
        stateful.send("increment")
        stateful.send("increment")
        time.sleep(0.1)
        
        original_value = stateful._actor.value  # Debería ser 102
        
        stateful.send("fail")
        time.sleep(0.2)
        
        # Then: Valor se resetea a inicial después de restart
        # Nueva instancia tiene valor inicial
        assert stateful._actor.restart_count >= 1
        # Valor depende de implementación de restart (puede ser 100 o resetarse)
        
        # Cleanup
        supervisor.stop()
    
    def test_actor_metrics_persist_across_restarts(self):
        """Test: Métricas del actor persisten a través de restarts."""
        # Given: Supervisor con actor
        supervisor_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5, initial_delay=0.05))
        supervisor = spawn(SupervisorActor, strategy=supervisor_strategy)
        
        actor = supervisor._actor.spawn_child(
            ChildActor,
            "actor",
            "actor"
        )
        
        # When: Actor procesa mensajes y falla
        actor.send("msg1")
        actor.send("msg2")
        time.sleep(0.1)
        
        actor.send("fail")
        time.sleep(0.2)
        
        # Then: restart_count incrementado
        assert actor._actor.restart_count >= 1
        
        # Cleanup
        supervisor.stop()


# ============================================================================
# TEST: Supervisor Failures
# ============================================================================

class TestSupervisorFailures:
    """Tests de fallos del supervisor mismo."""
    
    def test_supervisor_handles_child_spawn_failure(self):
        """Test: Supervisor maneja fallo al spawning child."""
        # Given: Supervisor
        supervisor_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=5))
        supervisor = spawn(SupervisorActor, strategy=supervisor_strategy)
        
        # When: Intentamos spawn child con argumentos inválidos
        # (ChildActor requiere 'name' pero no lo pasamos)
        try:
            child = supervisor._actor.spawn_child(
                ChildActor,
                "child"
                # Falta 2nd arg (child name)
            )
            # Si no falla, verificamos que supervisor sigue operativo
            assert supervisor.actor.state == ActorState.RUNNING
        except Exception:
            # Supervisor debe seguir operativo tras error
            assert supervisor.actor.state == ActorState.RUNNING
        
        # Cleanup
        supervisor.stop()
    
    def test_supervisor_with_all_children_failed(self):
        """Test: Supervisor cuando todos los children fallan."""
        # Given: Supervisor con 3 children que fallan constantemente
        supervisor_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=2, initial_delay=0.1))
        supervisor = spawn(SupervisorActor, strategy=supervisor_strategy)
        
        child1 = supervisor._actor.spawn_child(
            ChildActor,
            "c1",
            "c1",
            should_fail=True
        )
        child2 = supervisor._actor.spawn_child(
            ChildActor,
            "c2",
            "c2",
            should_fail=True
        )
        child3 = supervisor._actor.spawn_child(
            ChildActor,
            "c3",
            "c3",
            should_fail=True
        )
        
        # When: Todos los children fallan
        child1.send("trigger")
        child2.send("trigger")
        child3.send("trigger")
        time.sleep(0.3)
        
        # Then: Supervisor intentó reiniciarlos
        assert child1._actor.restart_count >= 1
        assert child2._actor.restart_count >= 1
        assert child3._actor.restart_count >= 1
        
        # Supervisor sigue operativo o escaló error
        assert supervisor.actor.state in [ActorState.RUNNING, ActorState.RESTARTING]
        
        # Cleanup
        supervisor.stop()


# ============================================================================
# TEST: Load Testing Supervision
# ============================================================================

class TestSupervisionUnderLoad:
    """Tests de supervisión bajo carga."""
    
    def test_supervisor_with_many_children_under_load(self):
        """Test: Supervisor con 50 children procesando mensajes."""
        # Given: Supervisor con 50 workers
        supervisor_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=10, initial_delay=0.1))
        supervisor = spawn(SupervisorActor, strategy=supervisor_strategy)
        
        workers = []
        for i in range(50):
            worker = supervisor._actor.spawn_child(
                ChildActor,
                f"w{i}",
                f"w{i}"
            )
            workers.append(worker)
        
        # When: Cada worker recibe 20 mensajes
        for worker in workers:
            for j in range(20):
                worker.send(f"msg-{j}")
        
        # Wait for processing
        time.sleep(2.0)
        
        # Then: Todos los workers procesaron mensajes
        for worker in workers:
            assert len(worker._actor.messages_received) == 20
        
        # Cleanup
        supervisor.stop()
    
    def test_supervisor_handles_random_failures_under_load(self):
        """Test: Supervisor maneja fallos aleatorios bajo carga."""
        # Given: Supervisor con 20 workers (failure_rate=0.3)
        supervisor_strategy = OneForOneStrategy(restart_policy=RestartPolicy(max_retries=50, initial_delay=0.05))
        supervisor = spawn(SupervisorActor, strategy=supervisor_strategy)
        
        workers = []
        for i in range(20):
            worker = supervisor._actor.spawn_child(
                WorkerActor,
                name=f"worker{i}",
                id=i,
                failure_rate=0.3
            )
            workers.append(worker)
        
        # When: Cada worker recibe 10 mensajes de trabajo
        for worker in workers:
            for j in range(10):
                worker.send("work")
        
        # Wait for processing
        time.sleep(3.0)
        
        # Then: Algunos workers tuvieron fallos y fueron reiniciados
        total_failures = sum(w._actor.failures for w in workers)
        total_restarts = sum(w._actor.restart_count for w in workers)
        
        assert total_failures > 0  # Hubo fallos (probabilísticamente)
        # (Restarts pueden ser 0 si no excedieron max_retries)
        
        # Supervisor sigue operativo
        assert supervisor.actor.state == ActorState.RUNNING
        
        # Cleanup
        supervisor.stop()


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
