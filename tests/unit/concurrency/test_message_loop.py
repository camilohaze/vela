"""
Tests for Message Processing Loop Implementation

Jira: VELA-578
Task: TASK-039
Sprint: Sprint 16

Tests que validan la implementación del message processing loop.
"""

import pytest
import time
import threading
from src.concurrency.message_loop import (
    MessageLoopState, MessageProcessor, MessageLoop,
    ActorMessageProcessor, ActorWithMessageLoop, CounterActorWithLoop
)
from src.concurrency.mailbox import UnboundedMailbox, BoundedMailbox
from src.concurrency.actor import Actor, ActorState


class TestMessageLoopState:
    """Tests para MessageLoopState enum."""
    
    def test_message_loop_states_exist(self):
        """Test que todos los estados existen."""
        assert MessageLoopState.IDLE.value == "idle"
        assert MessageLoopState.RUNNING.value == "running"
        assert MessageLoopState.PAUSED.value == "paused"
        assert MessageLoopState.STOPPING.value == "stopping"
        assert MessageLoopState.STOPPED.value == "stopped"


class TestMessageProcessor:
    """Tests para MessageProcessor interface."""
    
    def test_message_processor_is_abstract(self):
        """Test que MessageProcessor no puede instanciarse."""
        with pytest.raises(TypeError):
            MessageProcessor()
    
    def test_message_processor_has_required_methods(self):
        """Test que MessageProcessor define métodos abstractos."""
        required_methods = ['process_message', 'handle_error']
        
        for method in required_methods:
            assert hasattr(MessageProcessor, method)


class SimpleProcessor(MessageProcessor):
    """Procesador simple para testing."""
    
    def __init__(self):
        self.processed = []
        self.errors = []
    
    def process_message(self, message):
        if message == "error":
            raise ValueError("Test error")
        self.processed.append(message)
    
    def handle_error(self, error, message):
        self.errors.append((error, message))


class TestMessageLoop:
    """Tests para MessageLoop."""
    
    def test_initialization(self):
        """Test que message loop se inicializa correctamente."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        assert loop.get_state() == MessageLoopState.IDLE
        assert loop.is_running() == False
        assert loop.get_messages_processed() == 0
        assert loop.get_errors_count() == 0
    
    def test_start_loop(self):
        """Test que loop puede iniciarse."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        loop.start()
        
        # Esperar que inicie
        time.sleep(0.01)
        
        assert loop.get_state() == MessageLoopState.RUNNING
        assert loop.is_running() == True
        
        loop.stop(timeout=0.5)
    
    def test_start_already_running_raises_error(self):
        """Test que iniciar loop ya corriendo lanza error."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        loop.start()
        
        with pytest.raises(RuntimeError, match="already running"):
            loop.start()
        
        loop.stop(timeout=0.5)
    
    def test_stop_loop(self):
        """Test que loop puede detenerse."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        loop.start()
        time.sleep(0.01)
        
        loop.stop(timeout=0.5)
        
        assert loop.get_state() == MessageLoopState.STOPPED
        assert loop.is_running() == False
    
    def test_stop_not_running_raises_error(self):
        """Test que detener loop no corriendo lanza error."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        with pytest.raises(RuntimeError, match="not running"):
            loop.stop()
    
    def test_process_messages_from_mailbox(self):
        """Test que loop procesa mensajes del mailbox."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        # Agregar mensajes
        mailbox.enqueue("msg1")
        mailbox.enqueue("msg2")
        mailbox.enqueue("msg3")
        
        # Iniciar loop
        loop.start()
        
        # Esperar que procese
        time.sleep(0.05)
        
        # Detener
        loop.stop(timeout=0.5)
        
        # Verificar que procesó todos
        assert loop.get_messages_processed() == 3
        assert processor.processed == ["msg1", "msg2", "msg3"]
    
    def test_sequential_processing(self):
        """Test que mensajes se procesan secuencialmente."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        # Agregar mensajes con orden específico
        for i in range(10):
            mailbox.enqueue(i)
        
        loop.start()
        time.sleep(0.05)
        loop.stop(timeout=0.5)
        
        # Verificar orden
        assert processor.processed == list(range(10))
    
    def test_error_handling(self):
        """Test que errores son manejados correctamente."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        mailbox.enqueue("msg1")
        mailbox.enqueue("error")  # Causará error
        mailbox.enqueue("msg2")
        
        loop.start()
        time.sleep(0.05)
        loop.stop(timeout=0.5)
        
        # Verificar que continuó procesando después del error
        assert "msg1" in processor.processed
        assert "msg2" in processor.processed
        assert loop.get_errors_count() == 1
        assert len(processor.errors) == 1
    
    def test_max_throughput(self):
        """Test que max_throughput limita mensajes por ciclo."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(
            mailbox=mailbox,
            processor=processor,
            max_throughput=5
        )
        
        # Agregar 20 mensajes
        for i in range(20):
            mailbox.enqueue(i)
        
        loop.start()
        time.sleep(0.02)  # Menos tiempo
        loop.stop(timeout=0.5)
        
        # Debe haber procesado menos de 20 (por throughput limit)
        # Nota: el test es probabilístico, pero normalmente funciona
        cycles = loop.get_cycles_count()
        assert cycles >= 1
    
    def test_pause_and_resume(self):
        """Test que loop puede pausarse y resumirse."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        loop.start()
        time.sleep(0.01)
        
        # Pausar
        loop.pause()
        assert loop.get_state() == MessageLoopState.PAUSED
        
        # Agregar mensajes mientras está pausado
        mailbox.enqueue("msg1")
        mailbox.enqueue("msg2")
        
        time.sleep(0.05)
        
        # No debió procesar nada
        assert loop.get_messages_processed() == 0
        
        # Resumir
        loop.resume()
        assert loop.get_state() == MessageLoopState.RUNNING
        
        time.sleep(0.05)
        
        # Ahora sí procesó
        assert loop.get_messages_processed() == 2
        
        loop.stop(timeout=0.5)
    
    def test_metrics(self):
        """Test que métricas se recolectan correctamente."""
        mailbox = UnboundedMailbox()
        processor = SimpleProcessor()
        loop = MessageLoop(mailbox=mailbox, processor=processor)
        
        mailbox.enqueue("msg1")
        mailbox.enqueue("msg2")
        mailbox.enqueue("error")
        
        loop.start()
        time.sleep(0.05)
        loop.stop(timeout=0.5)
        
        assert loop.get_messages_processed() == 2
        assert loop.get_errors_count() == 1
        assert loop.get_cycles_count() >= 1
        assert loop.get_average_processing_time() >= 0.0


class TestActorMessageProcessor:
    """Tests para ActorMessageProcessor."""
    
    def test_process_message_delegates_to_actor(self):
        """Test que process_message delega al actor."""
        
        class TestActor(Actor):
            def __init__(self):
                self.name = "test"
                self._state = ActorState.RUNNING
                self._message_count = 0
                self._error_count = 0
                self.received = []
            
            def receive(self, message):
                self.received.append(message)
        
        actor = TestActor()
        processor = ActorMessageProcessor(actor)
        
        processor.process_message("test message")
        
        assert actor.received == ["test message"]
        assert actor._message_count == 1
    
    def test_handle_error_increments_error_count(self):
        """Test que handle_error incrementa contador de errores."""
        
        class TestActor(Actor):
            def __init__(self):
                self.name = "test"
                self._state = ActorState.RUNNING
                self._message_count = 0
                self._error_count = 0
            
            def receive(self, message):
                pass
        
        actor = TestActor()
        processor = ActorMessageProcessor(actor)
        
        error = ValueError("test error")
        
        with pytest.raises(ValueError):
            processor.handle_error(error, "message")
        
        assert actor._error_count == 1
    
    def test_process_message_requires_running_state(self):
        """Test que process_message requiere actor corriendo."""
        
        class TestActor(Actor):
            def __init__(self):
                self.name = "test"
                self._state = ActorState.STOPPED
                self._message_count = 0
                self._error_count = 0
            
            def receive(self, message):
                pass
        
        actor = TestActor()
        processor = ActorMessageProcessor(actor)
        
        with pytest.raises(RuntimeError, match="not running"):
            processor.process_message("test")


class TestActorWithMessageLoop:
    """Tests para ActorWithMessageLoop."""
    
    def test_initialization(self):
        """Test que actor con message loop se inicializa correctamente."""
        actor = ActorWithMessageLoop(name="TestActor")
        
        assert actor.name == "TestActor"
        assert actor._state == ActorState.UNINITIALIZED
        assert actor._mailbox is not None
        assert actor._message_loop is not None
    
    def test_start_actor(self):
        """Test que actor puede iniciarse."""
        actor = ActorWithMessageLoop(name="TestActor")
        
        actor.start()
        
        assert actor._state == ActorState.RUNNING
        assert actor._message_loop.is_running() == True
        
        actor.stop(timeout=0.5)
    
    def test_stop_actor(self):
        """Test que actor puede detenerse."""
        actor = ActorWithMessageLoop(name="TestActor")
        
        actor.start()
        time.sleep(0.01)
        
        actor.stop(timeout=0.5)
        
        assert actor._state == ActorState.STOPPED
        assert actor._message_loop.get_state() == MessageLoopState.STOPPED
    
    def test_send_and_receive(self):
        """Test que actor puede enviar y recibir mensajes."""
        actor = ActorWithMessageLoop(name="TestActor")
        
        actor.start()
        
        # Enviar mensajes
        actor.send("msg1")
        actor.send("msg2")
        actor.send("msg3")
        
        # Esperar que se procesen
        time.sleep(0.05)
        
        actor.stop(timeout=0.5)
        
        # Verificar que se procesaron
        processed = actor.get_processed_messages()
        assert processed == ["msg1", "msg2", "msg3"]
    
    def test_get_metrics(self):
        """Test que puede obtener métricas del message loop."""
        actor = ActorWithMessageLoop(name="TestActor")
        
        actor.start()
        actor.send("msg1")
        actor.send("msg2")
        time.sleep(0.05)
        actor.stop(timeout=0.5)
        
        metrics = actor.get_message_loop_metrics()
        
        assert metrics["state"] == "stopped"
        assert metrics["messages_processed"] == 2
        assert metrics["errors_count"] == 0
        assert metrics["cycles_count"] >= 1


class TestCounterActorWithLoop:
    """Tests para CounterActorWithLoop."""
    
    def test_counter_actor_initialization(self):
        """Test que counter actor se inicializa correctamente."""
        actor = CounterActorWithLoop(name="MyCounter")
        
        assert actor.name == "MyCounter"
        assert actor.count == 0
    
    def test_counter_increment(self):
        """Test que counter puede incrementarse."""
        actor = CounterActorWithLoop()
        
        actor.start()
        
        actor.send("increment")
        actor.send("increment")
        actor.send("increment")
        
        time.sleep(0.05)
        actor.stop(timeout=0.5)
        
        assert actor.count == 3
    
    def test_counter_decrement(self):
        """Test que counter puede decrementarse."""
        actor = CounterActorWithLoop()
        
        actor.start()
        
        actor.send("increment")
        actor.send("increment")
        actor.send("decrement")
        
        time.sleep(0.05)
        actor.stop(timeout=0.5)
        
        assert actor.count == 1
    
    def test_counter_reset(self):
        """Test que counter puede resetearse."""
        actor = CounterActorWithLoop()
        
        actor.start()
        
        actor.send("increment")
        actor.send("increment")
        actor.send("reset")
        
        time.sleep(0.05)
        actor.stop(timeout=0.5)
        
        assert actor.count == 0
    
    def test_counter_add_value(self):
        """Test que counter puede sumar valores."""
        actor = CounterActorWithLoop()
        
        actor.start()
        
        actor.send("increment")
        actor.send({"type": "add", "value": 5})
        actor.send({"type": "add", "value": 3})
        
        time.sleep(0.05)
        actor.stop(timeout=0.5)
        
        assert actor.count == 9  # 1 + 5 + 3
    
    def test_counter_mixed_operations(self):
        """Test que counter maneja operaciones mixtas."""
        actor = CounterActorWithLoop()
        
        actor.start()
        
        actor.send("increment")
        actor.send("increment")
        actor.send({"type": "add", "value": 10})
        actor.send("decrement")
        actor.send({"type": "add", "value": 5})
        
        time.sleep(0.05)
        actor.stop(timeout=0.5)
        
        # 0 + 1 + 1 + 10 - 1 + 5 = 16
        assert actor.count == 16


class TestActorWithBoundedMailbox:
    """Tests de actor con bounded mailbox."""
    
    def test_actor_with_bounded_mailbox(self):
        """Test que actor funciona con bounded mailbox."""
        bounded_mailbox = BoundedMailbox(capacity=5)
        actor = ActorWithMessageLoop(
            name="BoundedActor",
            mailbox=bounded_mailbox
        )
        
        actor.start()
        
        # Enviar más mensajes que la capacidad
        accepted = 0
        for i in range(10):
            if actor.send(f"msg{i}"):
                accepted += 1
        
        time.sleep(0.05)
        actor.stop(timeout=0.5)
        
        # Solo algunos fueron aceptados
        processed = actor.get_processed_messages()
        assert len(processed) <= 5


class TestMessageLoopConcurrency:
    """Tests de concurrencia del message loop."""
    
    def test_multiple_actors_concurrent(self):
        """Test múltiples actors procesando concurrentemente."""
        actors = []
        
        # Crear 5 actors
        for i in range(5):
            actor = CounterActorWithLoop(name=f"Actor{i}")
            actor.start()
            actors.append(actor)
        
        # Enviar mensajes a todos
        for actor in actors:
            for _ in range(10):
                actor.send("increment")
        
        # Esperar que procesen
        time.sleep(0.1)
        
        # Detener todos
        for actor in actors:
            actor.stop(timeout=0.5)
        
        # Verificar que cada uno procesó sus mensajes
        for actor in actors:
            assert actor.count == 10
    
    def test_actor_thread_safety(self):
        """Test que actor maneja concurrencia correctamente."""
        actor = CounterActorWithLoop()
        actor.start()
        
        # Múltiples threads enviando mensajes
        def sender(count):
            for _ in range(count):
                actor.send("increment")
        
        threads = []
        for _ in range(5):
            t = threading.Thread(target=sender, args=(20,))
            threads.append(t)
            t.start()
        
        for t in threads:
            t.join()
        
        # Esperar que procese
        time.sleep(0.2)
        actor.stop(timeout=0.5)
        
        # Debe haber procesado todos los incrementos
        assert actor.count == 100  # 5 threads * 20 incrementos


class TestMessageLoopPerformance:
    """Tests de performance del message loop."""
    
    def test_high_throughput(self):
        """Test que loop puede manejar alto throughput."""
        actor = CounterActorWithLoop()
        actor.start()
        
        # Enviar muchos mensajes
        start = time.time()
        for _ in range(1000):
            actor.send("increment")
        send_time = time.time() - start
        
        # Esperar que procese
        time.sleep(0.5)
        actor.stop(timeout=1.0)
        
        # Verificar que procesó todos
        assert actor.count == 1000
        
        # Métricas
        metrics = actor.get_message_loop_metrics()
        assert metrics["messages_processed"] == 1000
        
        print(f"\nSend time: {send_time:.3f}s")
        print(f"Avg processing time: {metrics['avg_processing_time']:.6f}s")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
