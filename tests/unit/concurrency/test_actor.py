"""
Tests for Actor Instance Implementation

Jira: VELA-578
Task: TASK-037
Sprint: Sprint 16

Tests que validan la implementación de Actor instances.
"""

import pytest
from src.concurrency.actor import (
    Actor, ActorRef, ActorState, CounterActor, spawn
)


class TestActorState:
    """Tests para ActorState enum."""
    
    def test_actor_state_values(self):
        """Test que ActorState tiene todos los valores esperados."""
        assert ActorState.UNINITIALIZED.value == "uninitialized"
        assert ActorState.STARTING.value == "starting"
        assert ActorState.RUNNING.value == "running"
        assert ActorState.STOPPING.value == "stopping"
        assert ActorState.STOPPED.value == "stopped"
        assert ActorState.RESTARTING.value == "restarting"


class SimpleActor(Actor):
    """Actor simple para testing."""
    
    def __init__(self):
        super().__init__()
        self.messages_received = []
        self.pre_start_called = False
        self.post_stop_called = False
        self.pre_restart_called = False
        self.post_restart_called = False
    
    def pre_start(self):
        self.pre_start_called = True
    
    def post_stop(self):
        self.post_stop_called = True
    
    def pre_restart(self, error):
        self.pre_restart_called = True
    
    def post_restart(self, error):
        self.post_restart_called = True
    
    def receive(self, message):
        self.messages_received.append(message)


class TestActorBase:
    """Tests para clase base Actor."""
    
    def test_actor_initialization(self):
        """Test que Actor se inicializa correctamente."""
        actor = SimpleActor()
        
        assert actor._actor_state == ActorState.UNINITIALIZED
        assert actor._actor_ref is None
        assert actor._message_count == 0
        assert actor._error_count == 0
    
    def test_actor_must_implement_receive(self):
        """Test que Actor es abstracta (debe implementar receive)."""
        
        class InvalidActor(Actor):
            pass
        
        # No se puede instanciar sin implementar receive
        with pytest.raises(TypeError):
            InvalidActor()
    
    def test_actor_has_lifecycle_hooks(self):
        """Test que Actor tiene todos los lifecycle hooks."""
        actor = SimpleActor()
        
        assert hasattr(actor, 'pre_start')
        assert hasattr(actor, 'post_stop')
        assert hasattr(actor, 'pre_restart')
        assert hasattr(actor, 'post_restart')
    
    def test_actor_get_state(self):
        """Test que get_state retorna estado actual."""
        actor = SimpleActor()
        assert actor.get_state() == ActorState.UNINITIALIZED
    
    def test_actor_set_state(self):
        """Test que _set_state cambia estado."""
        actor = SimpleActor()
        actor._set_state(ActorState.RUNNING)
        assert actor.get_state() == ActorState.RUNNING
    
    def test_actor_metrics(self):
        """Test que métricas están disponibles."""
        actor = SimpleActor()
        
        assert actor.get_message_count() == 0
        assert actor.get_error_count() == 0
        
        actor._increment_message_count()
        assert actor.get_message_count() == 1
        
        actor._increment_error_count()
        assert actor.get_error_count() == 1


class TestActorRef:
    """Tests para ActorRef."""
    
    def test_actor_ref_initialization(self):
        """Test que ActorRef se inicializa correctamente."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        assert ref.name == "test-actor"
        assert ref._actor == actor
        assert ref._stopped == False
        assert actor._actor_ref == ref
    
    def test_actor_ref_path(self):
        """Test que path es correcto."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        assert ref.path == "local://test-actor"
    
    def test_actor_ref_send(self):
        """Test que send envía mensaje al actor."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        ref.send("Hello")
        ref.send("World")
        
        assert actor.messages_received == ["Hello", "World"]
        assert actor.get_message_count() == 2
    
    def test_actor_ref_tell_alias(self):
        """Test que tell es alias de send."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        ref.tell("Message")
        
        assert actor.messages_received == ["Message"]
    
    def test_actor_ref_send_when_stopped_raises_error(self):
        """Test que send lanza error si actor está stopped."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        ref.stop()
        
        with pytest.raises(RuntimeError, match="is stopped"):
            ref.send("Message")
    
    def test_actor_ref_stop(self):
        """Test que stop detiene el actor."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        ref.stop()
        
        assert ref.is_stopped() == True
        assert actor.post_stop_called == True
        assert actor.get_state() == ActorState.STOPPED
    
    def test_actor_ref_stop_idempotent(self):
        """Test que stop es idempotente (puede llamarse múltiples veces)."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        ref.stop()
        ref.stop()  # No debe lanzar error
        
        assert ref.is_stopped() == True
    
    def test_actor_ref_equality(self):
        """Test que dos refs con mismo nombre son iguales."""
        actor1 = SimpleActor()
        actor2 = SimpleActor()
        
        ref1 = ActorRef("same-name", actor1)
        ref2 = ActorRef("same-name", actor2)
        
        assert ref1 == ref2
    
    def test_actor_ref_inequality(self):
        """Test que refs con diferentes nombres son distintos."""
        actor = SimpleActor()
        
        ref1 = ActorRef("actor1", actor)
        ref2 = ActorRef("actor2", actor)
        
        assert ref1 != ref2
    
    def test_actor_ref_hash(self):
        """Test que ActorRef es hashable."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        # Debe poder usarse en set/dict
        actor_set = {ref}
        assert ref in actor_set
        
        actor_dict = {ref: "value"}
        assert actor_dict[ref] == "value"
    
    def test_actor_ref_repr(self):
        """Test representación string de ActorRef."""
        actor = SimpleActor()
        ref = ActorRef("test-actor", actor)
        
        assert repr(ref) == "ActorRef(test-actor, running)"
        
        ref.stop()
        assert repr(ref) == "ActorRef(test-actor, stopped)"


class TestSpawnFunction:
    """Tests para función spawn."""
    
    def test_spawn_creates_actor_ref(self):
        """Test que spawn crea ActorRef correctamente."""
        ref = spawn(SimpleActor, name="test1")
        
        assert isinstance(ref, ActorRef)
        assert ref.name == "test1"
    
    def test_spawn_calls_pre_start(self):
        """Test que spawn llama pre_start lifecycle hook."""
        ref = spawn(SimpleActor, name="test2")
        
        assert ref._actor.pre_start_called == True
    
    def test_spawn_sets_state_to_running(self):
        """Test que spawn deja actor en estado RUNNING."""
        ref = spawn(SimpleActor, name="test3")
        
        assert ref._actor.get_state() == ActorState.RUNNING
    
    def test_spawn_auto_generates_name(self):
        """Test que spawn genera nombre si no se provee."""
        ref1 = spawn(SimpleActor)
        ref2 = spawn(SimpleActor)
        
        assert ref1.name.startswith("SimpleActor-")
        assert ref2.name.startswith("SimpleActor-")
        assert ref1.name != ref2.name
    
    def test_spawn_passes_kwargs_to_constructor(self):
        """Test que spawn pasa kwargs al constructor."""
        
        class ParamActor(Actor):
            def __init__(self, value: int, user_name: str):
                super().__init__()
                self.value = value
                self.user_name = user_name
            
            def receive(self, message):
                pass
        
        ref = spawn(ParamActor, name="param1", value=42, user_name="Alice")
        
        assert ref._actor.value == 42
        assert ref._actor.user_name == "Alice"
    
    def test_spawn_validates_actor_type(self):
        """Test que spawn valida que es subclase de Actor."""
        
        class NotAnActor:
            pass
        
        with pytest.raises(TypeError, match="must inherit from Actor"):
            spawn(NotAnActor)


class TestCounterActor:
    """Tests para CounterActor example."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.counter = spawn(CounterActor, name="counter-test")
    
    def test_counter_initialization(self):
        """Test que counter se inicializa en 0."""
        assert self.counter._actor.get_count() == 0
    
    def test_counter_increment(self, capsys):
        """Test incremento del contador."""
        self.counter.send("Increment")
        
        assert self.counter._actor.get_count() == 1
    
    def test_counter_decrement(self):
        """Test decremento del contador."""
        self.counter.send("Increment")
        self.counter.send("Increment")
        self.counter.send("Decrement")
        
        assert self.counter._actor.get_count() == 1
    
    def test_counter_reset(self):
        """Test reset del contador."""
        self.counter.send("Increment")
        self.counter.send("Increment")
        self.counter.send("Reset")
        
        assert self.counter._actor.get_count() == 0
    
    def test_counter_get_count_message(self):
        """Test mensaje GetCount con sender."""
        # Crear otro actor para recibir respuesta
        receiver = spawn(SimpleActor, name="receiver")
        
        self.counter.send("Increment")
        self.counter.send("Increment")
        self.counter.send(("GetCount", receiver))
        
        # Verificar que receiver recibió respuesta
        assert len(receiver._actor.messages_received) == 1
        assert receiver._actor.messages_received[0] == ("CountResult", 2)
    
    def test_counter_multiple_operations(self):
        """Test múltiples operaciones secuenciales."""
        self.counter.send("Increment")
        self.counter.send("Increment")
        self.counter.send("Increment")
        assert self.counter._actor.get_count() == 3
        
        self.counter.send("Decrement")
        assert self.counter._actor.get_count() == 2
        
        self.counter.send("Reset")
        assert self.counter._actor.get_count() == 0
    
    def test_counter_lifecycle_hooks(self, capsys):
        """Test que lifecycle hooks son llamados."""
        counter = spawn(CounterActor, name="lifecycle-test")
        
        captured = capsys.readouterr()
        assert "CounterActor starting" in captured.out
        
        counter.stop()
        
        captured = capsys.readouterr()
        assert "CounterActor stopped" in captured.out


class TestActorSelfReference:
    """Tests para self() method."""
    
    def test_actor_self_returns_actor_ref(self):
        """Test que self() retorna ActorRef."""
        ref = spawn(SimpleActor, name="self-test")
        
        self_ref = ref._actor.self()
        
        assert isinstance(self_ref, ActorRef)
        assert self_ref == ref
    
    def test_actor_self_before_initialization_raises_error(self):
        """Test que self() lanza error si actor no inicializado."""
        actor = SimpleActor()
        
        with pytest.raises(RuntimeError, match="not initialized"):
            actor.self()


class TestActorMessaging:
    """Tests de comunicación entre actores."""
    
    def test_actor_can_send_to_itself(self):
        """Test que actor puede enviarse mensajes a sí mismo."""
        
        class SelfSendingActor(Actor):
            def __init__(self):
                super().__init__()
                self.count = 0
            
            def receive(self, message):
                if message == "Start":
                    self.count += 1
                    if self.count < 5:
                        self.self().send("Start")
        
        ref = spawn(SelfSendingActor, name="self-sender")
        ref.send("Start")
        
        assert ref._actor.count == 5
    
    def test_actor_can_send_to_other_actor(self):
        """Test que actor puede enviar a otro actor."""
        
        class ForwarderActor(Actor):
            def __init__(self, target):
                super().__init__()
                self.target = target
            
            def receive(self, message):
                self.target.send(f"Forwarded: {message}")
        
        receiver = spawn(SimpleActor, name="receiver")
        forwarder = spawn(ForwarderActor, name="forwarder", target=receiver)
        
        forwarder.send("Hello")
        
        assert receiver._actor.messages_received == ["Forwarded: Hello"]


class TestActorErrorHandling:
    """Tests de manejo de errores en actores."""
    
    def test_actor_error_increments_counter(self):
        """Test que errores incrementan contador."""
        
        class ErrorActor(Actor):
            def receive(self, message):
                if message == "Fail":
                    self._increment_error_count()
        
        ref = spawn(ErrorActor, name="error-test")
        
        ref.send("Fail")
        ref.send("Fail")
        
        assert ref._actor.get_error_count() == 2
    
    def test_actor_pre_restart_hook(self):
        """Test que pre_restart es llamado."""
        actor = SimpleActor()
        error = ValueError("test error")
        
        actor.pre_restart(error)
        
        assert actor.pre_restart_called == True
    
    def test_actor_post_restart_hook(self):
        """Test que post_restart es llamado."""
        actor = SimpleActor()
        error = ValueError("test error")
        
        actor.post_restart(error)
        
        assert actor.post_restart_called == True


class TestActorMetrics:
    """Tests de métricas de actores."""
    
    def test_message_count_increases(self):
        """Test que contador de mensajes aumenta."""
        ref = spawn(SimpleActor, name="metrics-test")
        
        assert ref._actor.get_message_count() == 0
        
        ref.send("Msg1")
        assert ref._actor.get_message_count() == 1
        
        ref.send("Msg2")
        ref.send("Msg3")
        assert ref._actor.get_message_count() == 3
    
    def test_multiple_actors_have_independent_counters(self):
        """Test que cada actor tiene sus propias métricas."""
        ref1 = spawn(SimpleActor, name="actor1")
        ref2 = spawn(SimpleActor, name="actor2")
        
        ref1.send("A")
        ref1.send("B")
        
        ref2.send("X")
        
        assert ref1._actor.get_message_count() == 2
        assert ref2._actor.get_message_count() == 1


class TestActorStateTransitions:
    """Tests de transiciones de estado."""
    
    def test_actor_state_lifecycle(self):
        """Test ciclo de vida completo del estado."""
        actor = SimpleActor()
        
        # 1. Uninitialized
        assert actor.get_state() == ActorState.UNINITIALIZED
        
        # 2. Starting -> Running (por spawn)
        ref = ActorRef("test", actor)
        actor._set_state(ActorState.STARTING)
        actor.pre_start()
        actor._set_state(ActorState.RUNNING)
        
        assert actor.get_state() == ActorState.RUNNING
        
        # 3. Stopping -> Stopped
        actor._set_state(ActorState.STOPPING)
        actor.post_stop()
        actor._set_state(ActorState.STOPPED)
        
        assert actor.get_state() == ActorState.STOPPED
    
    def test_actor_restart_state(self):
        """Test estado de restart."""
        actor = SimpleActor()
        
        actor._set_state(ActorState.RESTARTING)
        assert actor.get_state() == ActorState.RESTARTING


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
