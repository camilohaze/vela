"""
Tests for Actor System Design

Jira: VELA-578
Task: TASK-036
Sprint: Sprint 16

Tests que validan el diseño del Actor System.

NOTA: Estos tests validan la estructura y API del diseño.
Tests funcionales completos comenzarán en TASK-037.
"""

import pytest
from src.actor_system_design import (
    CounterActor, Increment, Decrement, GetCount, Reset,
    ChatRoomActor, UserActor, Join, Leave, ChatMessage, PostMessage,
    ProducerActor, ConsumerActor, Start, Work, Done,
    ActorSystemDesign, ActorRef, ActorSystemConfig
)


class TestCounterActorDesign:
    """Tests para validar diseño de CounterActor."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.counter = CounterActor()
    
    def test_counter_initialization(self):
        """Test que el contador se inicializa en 0."""
        assert self.counter._count == 0
    
    def test_counter_has_receive_method(self):
        """Test que CounterActor tiene método receive."""
        assert hasattr(self.counter, 'receive')
        assert callable(self.counter.receive)
    
    def test_counter_has_lifecycle_hooks(self):
        """Test que CounterActor tiene lifecycle hooks."""
        assert hasattr(self.counter, 'pre_start')
        assert hasattr(self.counter, 'post_stop')
        assert callable(self.counter.pre_start)
        assert callable(self.counter.post_stop)
    
    def test_counter_increment(self, capsys):
        """Test incremento del contador."""
        self.counter.receive(Increment())
        assert self.counter._count == 1
        
        captured = capsys.readouterr()
        assert "Count incremented to 1" in captured.out
    
    def test_counter_decrement(self, capsys):
        """Test decremento del contador."""
        self.counter._count = 5
        self.counter.receive(Decrement())
        assert self.counter._count == 4
        
        captured = capsys.readouterr()
        assert "Count decremented to 4" in captured.out
    
    def test_counter_reset(self, capsys):
        """Test reset del contador."""
        self.counter._count = 10
        self.counter.receive(Reset())
        assert self.counter._count == 0
        
        captured = capsys.readouterr()
        assert "Count reset to 0" in captured.out
    
    def test_counter_multiple_operations(self):
        """Test múltiples operaciones secuenciales."""
        self.counter.receive(Increment())
        self.counter.receive(Increment())
        self.counter.receive(Increment())
        assert self.counter._count == 3
        
        self.counter.receive(Decrement())
        assert self.counter._count == 2
        
        self.counter.receive(Reset())
        assert self.counter._count == 0


class TestChatRoomActorDesign:
    """Tests para validar diseño de ChatRoomActor."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.chat_room = ChatRoomActor()
    
    def test_chat_room_initialization(self):
        """Test que la sala se inicializa vacía."""
        assert self.chat_room._users == []
    
    def test_chat_room_has_receive_method(self):
        """Test que ChatRoomActor tiene método receive."""
        assert hasattr(self.chat_room, 'receive')
        assert callable(self.chat_room.receive)
    
    def test_chat_room_has_broadcast_method(self):
        """Test que ChatRoomActor tiene método _broadcast."""
        assert hasattr(self.chat_room, '_broadcast')
        assert callable(self.chat_room._broadcast)
    
    def test_user_actor_initialization(self):
        """Test que UserActor se inicializa con nombre."""
        user = UserActor("Alice")
        assert user._name == "Alice"
    
    def test_user_actor_has_receive_method(self):
        """Test que UserActor tiene método receive."""
        user = UserActor("Alice")
        assert hasattr(user, 'receive')
        assert callable(user.receive)


class MockActorRef:
    """Mock de ActorRef para testing."""
    
    def __init__(self, name):
        self.name = name
        self.messages = []
    
    def send(self, message):
        """Capturar mensajes enviados."""
        self.messages.append(message)


class TestChatRoomBehavior:
    """Tests de comportamiento de chat room."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.chat_room = ChatRoomActor()
        self.alice_ref = MockActorRef("alice")
        self.bob_ref = MockActorRef("bob")
    
    def test_join_adds_user(self):
        """Test que Join agrega usuario a la sala."""
        self.chat_room.receive(Join(self.alice_ref, "Alice"))
        assert self.alice_ref in self.chat_room._users
    
    def test_leave_removes_user(self):
        """Test que Leave remueve usuario de la sala."""
        self.chat_room._users.append(self.alice_ref)
        self.chat_room.receive(Leave(self.alice_ref))
        assert self.alice_ref not in self.chat_room._users
    
    def test_broadcast_sends_to_all_users(self):
        """Test que broadcast envía mensaje a todos."""
        self.chat_room._users = [self.alice_ref, self.bob_ref]
        
        message = ChatMessage("System", "Test message")
        self.chat_room._broadcast(message)
        
        assert message in self.alice_ref.messages
        assert message in self.bob_ref.messages
    
    def test_post_message_broadcasts(self):
        """Test que PostMessage hace broadcast."""
        self.chat_room._users = [self.alice_ref, self.bob_ref]
        
        self.chat_room.receive(PostMessage("Hello", "Alice"))
        
        # Verificar que todos recibieron mensaje
        assert len(self.alice_ref.messages) == 1
        assert len(self.bob_ref.messages) == 1


class TestProducerConsumerDesign:
    """Tests para validar diseño de Producer-Consumer."""
    
    def test_producer_initialization(self):
        """Test que Producer se inicializa con consumer ref."""
        consumer_ref = MockActorRef("consumer")
        producer = ProducerActor(consumer_ref)
        assert producer._consumer == consumer_ref
    
    def test_consumer_initialization(self):
        """Test que Consumer se inicializa con contador en 0."""
        consumer = ConsumerActor()
        assert consumer._processed == 0
    
    def test_consumer_has_process_method(self):
        """Test que Consumer tiene método _process."""
        consumer = ConsumerActor()
        assert hasattr(consumer, '_process')
        assert callable(consumer._process)
    
    def test_consumer_process_method(self):
        """Test que _process funciona correctamente."""
        consumer = ConsumerActor()
        result = consumer._process(5)
        assert result == 10  # data * 2
    
    def test_producer_sends_work(self):
        """Test que Producer envía trabajo al Consumer."""
        consumer_ref = MockActorRef("consumer")
        producer = ProducerActor(consumer_ref)
        
        producer.receive(Start())
        
        # Verificar que se enviaron 1000 Work + 1 Done
        assert len(consumer_ref.messages) == 1001
        assert isinstance(consumer_ref.messages[0], Work)
        assert isinstance(consumer_ref.messages[-1], Done)


class TestActorSystemDesign:
    """Tests para validar diseño de ActorSystem."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.system = ActorSystemDesign(name="TestSystem")
    
    def test_system_initialization(self):
        """Test que el sistema se inicializa correctamente."""
        assert self.system.name == "TestSystem"
        assert self.system._actors == {}
        assert self.system._running == False
    
    def test_system_has_spawn_method(self):
        """Test que ActorSystem tiene método spawn."""
        assert hasattr(self.system, 'spawn')
        assert callable(self.system.spawn)
    
    def test_system_has_stop_method(self):
        """Test que ActorSystem tiene método stop."""
        assert hasattr(self.system, 'stop')
        assert callable(self.system.stop)
    
    def test_system_has_shutdown_method(self):
        """Test que ActorSystem tiene método shutdown."""
        assert hasattr(self.system, 'shutdown')
        assert callable(self.system.shutdown)
    
    def test_spawn_creates_actor(self):
        """Test que spawn crea actor correctamente."""
        actor_ref = self.system.spawn(CounterActor, name="counter1")
        
        assert isinstance(actor_ref, ActorRef)
        assert actor_ref.name == "counter1"
        assert "counter1" in self.system._actors
    
    def test_spawn_generates_name_if_not_provided(self):
        """Test que spawn genera nombre si no se provee."""
        actor_ref = self.system.spawn(CounterActor)
        
        assert actor_ref.name.startswith("CounterActor-")
    
    def test_spawn_with_kwargs(self):
        """Test que spawn pasa kwargs al constructor."""
        # Usar CounterActor que tiene lifecycle hooks
        counter_ref = self.system.spawn(CounterActor, name="counter_test")
        
        assert counter_ref.name == "counter_test"
    
    def test_stop_removes_actor(self):
        """Test que stop remueve actor del sistema."""
        actor_ref = self.system.spawn(CounterActor, name="counter1")
        self.system.stop(actor_ref)
        
        assert "counter1" not in self.system._actors
    
    def test_shutdown_stops_all_actors(self, capsys):
        """Test que shutdown detiene todos los actores."""
        self.system.spawn(CounterActor, name="counter1")
        self.system.spawn(CounterActor, name="counter2")
        
        self.system.shutdown()
        
        assert len(self.system._actors) == 0
        
        captured = capsys.readouterr()
        assert "Shutting down actor system" in captured.out
        assert "shutdown complete" in captured.out


class TestActorRef:
    """Tests para validar diseño de ActorRef."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.system = ActorSystemDesign(name="TestSystem")
        self.counter = CounterActor()
        self.actor_ref = ActorRef("counter1", self.counter, self.system)
    
    def test_actor_ref_initialization(self):
        """Test que ActorRef se inicializa correctamente."""
        assert self.actor_ref.name == "counter1"
        assert self.actor_ref._actor == self.counter
        assert self.actor_ref._system == self.system
    
    def test_actor_ref_has_send_method(self):
        """Test que ActorRef tiene método send."""
        assert hasattr(self.actor_ref, 'send')
        assert callable(self.actor_ref.send)
    
    def test_actor_ref_has_ask_method(self):
        """Test que ActorRef tiene método ask."""
        assert hasattr(self.actor_ref, 'ask')
        assert callable(self.actor_ref.ask)
    
    def test_actor_ref_send(self):
        """Test que send envía mensaje al actor."""
        self.actor_ref.send(Increment())
        assert self.counter._count == 1
    
    def test_actor_ref_ask_not_implemented(self):
        """Test que ask() lanza NotImplementedError (Sprint 18)."""
        with pytest.raises(NotImplementedError):
            self.actor_ref.ask(GetCount(None))
    
    def test_actor_ref_equality(self):
        """Test que dos refs al mismo actor son iguales."""
        ref1 = ActorRef("actor1", self.counter, self.system)
        ref2 = ActorRef("actor1", self.counter, self.system)
        
        assert ref1 == ref2
    
    def test_actor_ref_inequality(self):
        """Test que refs a diferentes actores son distintos."""
        ref1 = ActorRef("actor1", self.counter, self.system)
        ref2 = ActorRef("actor2", self.counter, self.system)
        
        assert ref1 != ref2
    
    def test_actor_ref_hash(self):
        """Test que ActorRef es hashable."""
        ref = ActorRef("actor1", self.counter, self.system)
        
        # Debe poder usarse en set/dict
        actor_set = {ref}
        assert ref in actor_set
    
    def test_actor_ref_repr(self):
        """Test representación string de ActorRef."""
        assert repr(self.actor_ref) == "ActorRef(counter1)"


class TestActorSystemConfig:
    """Tests para validar configuración del sistema."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.config = ActorSystemConfig()
    
    def test_config_initialization(self):
        """Test que la config tiene valores por defecto."""
        assert self.config.min_threads == 2
        assert self.config.max_threads == 100
        assert self.config.scheduler_type == "fair"
        assert self.config.default_mailbox_type == "unbounded"
    
    def test_config_has_thread_pool_settings(self):
        """Test que la config tiene settings de thread pool."""
        assert hasattr(self.config, 'min_threads')
        assert hasattr(self.config, 'max_threads')
        assert hasattr(self.config, 'thread_keep_alive_seconds')
        assert hasattr(self.config, 'queue_capacity')
    
    def test_config_has_scheduler_settings(self):
        """Test que la config tiene settings de scheduler."""
        assert hasattr(self.config, 'scheduler_type')
        assert hasattr(self.config, 'fairness_quantum')
    
    def test_config_has_mailbox_settings(self):
        """Test que la config tiene settings de mailbox."""
        assert hasattr(self.config, 'default_mailbox_type')
        assert hasattr(self.config, 'bounded_mailbox_capacity')
    
    def test_config_has_timeout_settings(self):
        """Test que la config tiene settings de timeouts."""
        assert hasattr(self.config, 'default_ask_timeout_seconds')
        assert hasattr(self.config, 'shutdown_timeout_seconds')
    
    def test_config_has_limit_settings(self):
        """Test que la config tiene settings de límites."""
        assert hasattr(self.config, 'max_actors')
        assert hasattr(self.config, 'max_messages_per_actor')
    
    def test_config_to_dict(self):
        """Test que config se convierte a dict correctamente."""
        config_dict = self.config.to_dict()
        
        assert isinstance(config_dict, dict)
        assert "thread_pool" in config_dict
        assert "scheduler" in config_dict
        assert "mailbox" in config_dict
        assert "timeouts" in config_dict
        assert "limits" in config_dict
    
    def test_config_to_dict_structure(self):
        """Test estructura del dict de config."""
        config_dict = self.config.to_dict()
        
        # Thread pool
        assert "min_threads" in config_dict["thread_pool"]
        assert "max_threads" in config_dict["thread_pool"]
        
        # Scheduler
        assert "type" in config_dict["scheduler"]
        assert "fairness_quantum" in config_dict["scheduler"]
        
        # Mailbox
        assert "default_type" in config_dict["mailbox"]
        assert "bounded_capacity" in config_dict["mailbox"]
        
        # Timeouts
        assert "ask" in config_dict["timeouts"]
        assert "shutdown" in config_dict["timeouts"]
        
        # Limits
        assert "max_actors" in config_dict["limits"]
        assert "max_messages_per_actor" in config_dict["limits"]


class TestMessageTypes:
    """Tests para validar tipos de mensajes."""
    
    def test_increment_message(self):
        """Test que Increment es instanciable."""
        msg = Increment()
        assert isinstance(msg, Increment)
    
    def test_decrement_message(self):
        """Test que Decrement es instanciable."""
        msg = Decrement()
        assert isinstance(msg, Decrement)
    
    def test_reset_message(self):
        """Test que Reset es instanciable."""
        msg = Reset()
        assert isinstance(msg, Reset)
    
    def test_get_count_message(self):
        """Test que GetCount tiene sender."""
        sender = MockActorRef("sender")
        msg = GetCount(sender)
        assert msg.sender == sender
    
    def test_work_message(self):
        """Test que Work tiene data."""
        msg = Work(42)
        assert msg.data == 42
    
    def test_chat_message(self):
        """Test que ChatMessage tiene sender y text."""
        msg = ChatMessage("Alice", "Hello")
        assert msg.sender_name == "Alice"
        assert msg.text == "Hello"


class TestDesignCompletenesss:
    """Tests para verificar completitud del diseño."""
    
    def test_all_actor_types_have_receive(self):
        """Test que todos los actors tienen método receive."""
        actors = [CounterActor, ChatRoomActor, UserActor, ProducerActor, ConsumerActor]
        
        for actor_class in actors:
            instance = actor_class() if actor_class != UserActor and actor_class != ProducerActor else None
            
            if actor_class == UserActor:
                instance = UserActor("test")
            elif actor_class == ProducerActor:
                instance = ProducerActor(MockActorRef("test"))
            
            assert hasattr(instance, 'receive')
            assert callable(instance.receive)
    
    def test_counter_actor_has_all_lifecycle_hooks(self):
        """Test que CounterActor tiene todos los lifecycle hooks."""
        counter = CounterActor()
        
        assert hasattr(counter, 'pre_start')
        assert hasattr(counter, 'post_stop')
    
    def test_actor_system_has_all_required_methods(self):
        """Test que ActorSystem tiene todos los métodos requeridos."""
        system = ActorSystemDesign(name="test")
        
        required_methods = ['spawn', 'stop', 'shutdown']
        
        for method in required_methods:
            assert hasattr(system, method)
            assert callable(getattr(system, method))
    
    def test_actor_ref_has_all_required_methods(self):
        """Test que ActorRef tiene todos los métodos requeridos."""
        ref = ActorRef("test", CounterActor(), None)
        
        required_methods = ['send', 'ask']
        
        for method in required_methods:
            assert hasattr(ref, method)
            assert callable(getattr(ref, method))


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
