"""
Tests for Mailbox System Implementation

Jira: VELA-578
Task: TASK-038
Sprint: Sprint 16

Tests que validan la implementación de mailboxes.
"""

import pytest
import threading
import time
from src.concurrency.mailbox import (
    Mailbox, UnboundedMailbox, BoundedMailbox, PriorityMailbox,
    MailboxType, create_mailbox, ActorWithMailbox
)


class TestMailboxInterface:
    """Tests para interfaz base Mailbox."""
    
    def test_mailbox_is_abstract(self):
        """Test que Mailbox no puede instanciarse directamente."""
        with pytest.raises(TypeError):
            Mailbox()
    
    def test_mailbox_has_required_methods(self):
        """Test que Mailbox define métodos abstractos."""
        required_methods = ['enqueue', 'dequeue', 'is_empty', 'size']
        
        for method in required_methods:
            assert hasattr(Mailbox, method)


class TestUnboundedMailbox:
    """Tests para UnboundedMailbox."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.mailbox = UnboundedMailbox()
    
    def test_initialization(self):
        """Test que mailbox se inicializa vacío."""
        assert self.mailbox.is_empty() == True
        assert self.mailbox.size() == 0
        assert self.mailbox.get_message_count() == 0
    
    def test_enqueue_message(self):
        """Test que enqueue agrega mensaje."""
        result = self.mailbox.enqueue("Message1")
        
        assert result == True
        assert self.mailbox.is_empty() == False
        assert self.mailbox.size() == 1
    
    def test_dequeue_message(self):
        """Test que dequeue extrae mensaje."""
        self.mailbox.enqueue("Message1")
        
        message = self.mailbox.dequeue()
        
        assert message == "Message1"
        assert self.mailbox.is_empty() == True
    
    def test_fifo_ordering(self):
        """Test que mantiene orden FIFO."""
        self.mailbox.enqueue("First")
        self.mailbox.enqueue("Second")
        self.mailbox.enqueue("Third")
        
        assert self.mailbox.dequeue() == "First"
        assert self.mailbox.dequeue() == "Second"
        assert self.mailbox.dequeue() == "Third"
    
    def test_dequeue_empty_returns_none(self):
        """Test que dequeue de mailbox vacío retorna None."""
        message = self.mailbox.dequeue()
        assert message is None
    
    def test_multiple_enqueue_dequeue(self):
        """Test múltiples operaciones enqueue/dequeue."""
        self.mailbox.enqueue("A")
        self.mailbox.enqueue("B")
        
        assert self.mailbox.dequeue() == "A"
        
        self.mailbox.enqueue("C")
        
        assert self.mailbox.dequeue() == "B"
        assert self.mailbox.dequeue() == "C"
        assert self.mailbox.is_empty() == True
    
    def test_message_count_increases(self):
        """Test que contador de mensajes aumenta."""
        assert self.mailbox.get_message_count() == 0
        
        self.mailbox.enqueue("Msg1")
        assert self.mailbox.get_message_count() == 1
        
        self.mailbox.enqueue("Msg2")
        assert self.mailbox.get_message_count() == 2
        
        # Dequeue no afecta message_count (es lifetime)
        self.mailbox.dequeue()
        assert self.mailbox.get_message_count() == 2
    
    def test_no_limit(self):
        """Test que no tiene límite de capacidad."""
        # Agregar muchos mensajes
        for i in range(10000):
            result = self.mailbox.enqueue(f"Message-{i}")
            assert result == True
        
        assert self.mailbox.size() == 10000


class TestBoundedMailbox:
    """Tests para BoundedMailbox."""
    
    def test_initialization_with_capacity(self):
        """Test que se inicializa con capacidad."""
        mailbox = BoundedMailbox(capacity=100)
        
        assert mailbox.is_empty() == True
        assert mailbox.size() == 0
        assert mailbox.get_capacity() == 100
    
    def test_initialization_default_capacity(self):
        """Test capacidad por defecto."""
        mailbox = BoundedMailbox()
        assert mailbox.get_capacity() == 1000
    
    def test_initialization_invalid_capacity_raises_error(self):
        """Test que capacidad inválida lanza error."""
        with pytest.raises(ValueError, match="must be positive"):
            BoundedMailbox(capacity=0)
        
        with pytest.raises(ValueError):
            BoundedMailbox(capacity=-10)
    
    def test_enqueue_within_capacity(self):
        """Test enqueue dentro de capacidad."""
        mailbox = BoundedMailbox(capacity=3)
        
        assert mailbox.enqueue("A") == True
        assert mailbox.enqueue("B") == True
        assert mailbox.enqueue("C") == True
        
        assert mailbox.size() == 3
    
    def test_enqueue_exceeds_capacity(self):
        """Test enqueue cuando lleno rechaza mensaje."""
        mailbox = BoundedMailbox(capacity=2)
        
        assert mailbox.enqueue("A") == True
        assert mailbox.enqueue("B") == True
        assert mailbox.enqueue("C") == False  # Rechazado
        
        assert mailbox.size() == 2
    
    def test_is_full(self):
        """Test que is_full detecta mailbox lleno."""
        mailbox = BoundedMailbox(capacity=2)
        
        assert mailbox.is_full() == False
        
        mailbox.enqueue("A")
        assert mailbox.is_full() == False
        
        mailbox.enqueue("B")
        assert mailbox.is_full() == True
    
    def test_rejected_count(self):
        """Test que cuenta mensajes rechazados."""
        mailbox = BoundedMailbox(capacity=2)
        
        assert mailbox.get_rejected_count() == 0
        
        mailbox.enqueue("A")
        mailbox.enqueue("B")
        mailbox.enqueue("C")  # Rechazado
        mailbox.enqueue("D")  # Rechazado
        
        assert mailbox.get_rejected_count() == 2
    
    def test_dequeue_frees_space(self):
        """Test que dequeue libera espacio."""
        mailbox = BoundedMailbox(capacity=2)
        
        mailbox.enqueue("A")
        mailbox.enqueue("B")
        assert mailbox.is_full() == True
        
        mailbox.dequeue()
        assert mailbox.is_full() == False
        
        # Ahora puede encolar nuevo mensaje
        assert mailbox.enqueue("C") == True
    
    def test_fifo_ordering(self):
        """Test que mantiene orden FIFO."""
        mailbox = BoundedMailbox(capacity=10)
        
        mailbox.enqueue("First")
        mailbox.enqueue("Second")
        mailbox.enqueue("Third")
        
        assert mailbox.dequeue() == "First"
        assert mailbox.dequeue() == "Second"
        assert mailbox.dequeue() == "Third"


class TestPriorityMailbox:
    """Tests para PriorityMailbox."""
    
    def test_initialization_default_priority(self):
        """Test inicialización sin función de prioridad."""
        mailbox = PriorityMailbox()
        
        assert mailbox.is_empty() == True
        assert mailbox.size() == 0
    
    def test_initialization_with_priority_function(self):
        """Test inicialización con función de prioridad."""
        priority_fn = lambda msg: 0 if msg == "URGENT" else 10
        mailbox = PriorityMailbox(priority_fn=priority_fn)
        
        assert mailbox.is_empty() == True
    
    def test_dequeue_by_priority(self):
        """Test que dequeue retorna mensaje de mayor prioridad."""
        # Prioridad: menor número = mayor prioridad
        priority_fn = lambda msg: 0 if msg.startswith("HIGH") else 10
        mailbox = PriorityMailbox(priority_fn=priority_fn)
        
        mailbox.enqueue("Normal message")
        mailbox.enqueue("HIGH: Important!")
        mailbox.enqueue("Another normal")
        
        # HIGH debe salir primero
        assert mailbox.dequeue() == "HIGH: Important!"
        assert mailbox.dequeue() == "Normal message"
        assert mailbox.dequeue() == "Another normal"
    
    def test_fifo_within_same_priority(self):
        """Test que mantiene FIFO dentro de misma prioridad."""
        priority_fn = lambda msg: 10  # Todos misma prioridad
        mailbox = PriorityMailbox(priority_fn=priority_fn)
        
        mailbox.enqueue("First")
        mailbox.enqueue("Second")
        mailbox.enqueue("Third")
        
        # FIFO porque misma prioridad
        assert mailbox.dequeue() == "First"
        assert mailbox.dequeue() == "Second"
        assert mailbox.dequeue() == "Third"
    
    def test_multiple_priority_levels(self):
        """Test con múltiples niveles de prioridad."""
        priority_fn = lambda msg: {
            "CRITICAL": 0,
            "HIGH": 5,
            "NORMAL": 10,
            "LOW": 15
        }.get(msg.split(":")[0], 10)
        
        mailbox = PriorityMailbox(priority_fn=priority_fn)
        
        mailbox.enqueue("NORMAL: Message 1")
        mailbox.enqueue("CRITICAL: Emergency!")
        mailbox.enqueue("LOW: Can wait")
        mailbox.enqueue("HIGH: Important")
        mailbox.enqueue("NORMAL: Message 2")
        
        # Orden esperado: CRITICAL, HIGH, NORMAL (FIFO), LOW
        assert mailbox.dequeue().startswith("CRITICAL")
        assert mailbox.dequeue().startswith("HIGH")
        assert mailbox.dequeue() == "NORMAL: Message 1"
        assert mailbox.dequeue() == "NORMAL: Message 2"
        assert mailbox.dequeue().startswith("LOW")
    
    def test_size_and_empty(self):
        """Test que size e is_empty funcionan correctamente."""
        mailbox = PriorityMailbox()
        
        assert mailbox.size() == 0
        assert mailbox.is_empty() == True
        
        mailbox.enqueue("Message")
        assert mailbox.size() == 1
        assert mailbox.is_empty() == False
        
        mailbox.dequeue()
        assert mailbox.size() == 0
        assert mailbox.is_empty() == True


class TestMailboxFactory:
    """Tests para create_mailbox factory."""
    
    def test_create_unbounded_mailbox(self):
        """Test crear UnboundedMailbox."""
        mailbox = create_mailbox(MailboxType.UNBOUNDED)
        
        assert isinstance(mailbox, UnboundedMailbox)
    
    def test_create_bounded_mailbox(self):
        """Test crear BoundedMailbox."""
        mailbox = create_mailbox(MailboxType.BOUNDED, capacity=500)
        
        assert isinstance(mailbox, BoundedMailbox)
        assert mailbox.get_capacity() == 500
    
    def test_create_bounded_mailbox_default_capacity(self):
        """Test crear BoundedMailbox con capacidad default."""
        mailbox = create_mailbox(MailboxType.BOUNDED)
        
        assert isinstance(mailbox, BoundedMailbox)
        assert mailbox.get_capacity() == 1000
    
    def test_create_priority_mailbox(self):
        """Test crear PriorityMailbox."""
        priority_fn = lambda msg: 0
        mailbox = create_mailbox(
            MailboxType.PRIORITY,
            priority_fn=priority_fn
        )
        
        assert isinstance(mailbox, PriorityMailbox)
    
    def test_create_invalid_type_raises_error(self):
        """Test que tipo inválido lanza error."""
        with pytest.raises(ValueError, match="Unknown mailbox type"):
            create_mailbox("invalid-type")


class TestThreadSafety:
    """Tests de thread-safety de mailboxes."""
    
    def test_unbounded_mailbox_thread_safe(self):
        """Test que UnboundedMailbox es thread-safe."""
        mailbox = UnboundedMailbox()
        messages = []
        
        def producer(start, count):
            for i in range(start, start + count):
                mailbox.enqueue(f"Message-{i}")
        
        def consumer(result_list):
            for _ in range(100):
                msg = mailbox.dequeue()
                if msg is not None:
                    result_list.append(msg)
                time.sleep(0.0001)
        
        # Múltiples producers
        threads = []
        threads.append(threading.Thread(target=producer, args=(0, 50)))
        threads.append(threading.Thread(target=producer, args=(50, 50)))
        
        # Múltiples consumers
        threads.append(threading.Thread(target=consumer, args=(messages,)))
        threads.append(threading.Thread(target=consumer, args=(messages,)))
        
        for t in threads:
            t.start()
        
        for t in threads:
            t.join()
        
        # Verificar que no se perdieron mensajes
        assert len(messages) == 100
    
    def test_bounded_mailbox_thread_safe(self):
        """Test que BoundedMailbox es thread-safe."""
        mailbox = BoundedMailbox(capacity=50)
        
        def producer(count):
            for i in range(count):
                mailbox.enqueue(f"Message-{i}")
        
        threads = []
        threads.append(threading.Thread(target=producer, args=(30,)))
        threads.append(threading.Thread(target=producer, args=(30,)))
        
        for t in threads:
            t.start()
        
        for t in threads:
            t.join()
        
        # No debe exceder capacidad
        assert mailbox.size() <= 50
        assert mailbox.get_rejected_count() >= 10


class TestActorWithMailbox:
    """Tests de integración Actor + Mailbox."""
    
    def test_actor_with_unbounded_mailbox(self):
        """Test actor con UnboundedMailbox."""
        mailbox = UnboundedMailbox()
        actor = ActorWithMailbox(mailbox)
        
        assert actor.send("Message1") == True
        assert actor.send("Message2") == True
        
        assert actor._mailbox.size() == 2
    
    def test_actor_with_bounded_mailbox(self):
        """Test actor con BoundedMailbox."""
        mailbox = BoundedMailbox(capacity=2)
        actor = ActorWithMailbox(mailbox)
        
        assert actor.send("A") == True
        assert actor.send("B") == True
        assert actor.send("C") == False  # Rechazado
    
    def test_actor_process_next_message(self):
        """Test que actor procesa mensaje del mailbox."""
        mailbox = UnboundedMailbox()
        actor = ActorWithMailbox(mailbox)
        
        actor.send("Test message")
        
        # Procesar mensaje
        result = actor.process_next_message()
        
        assert result == True
        assert mailbox.is_empty() == True
    
    def test_actor_process_empty_mailbox(self):
        """Test que actor no procesa si mailbox vacío."""
        mailbox = UnboundedMailbox()
        actor = ActorWithMailbox(mailbox)
        
        result = actor.process_next_message()
        
        assert result == False


class TestMailboxPerformance:
    """Tests de performance de mailboxes."""
    
    def test_unbounded_mailbox_large_volume(self):
        """Test UnboundedMailbox con gran volumen."""
        mailbox = UnboundedMailbox()
        
        # Encolar 10,000 mensajes
        start = time.time()
        for i in range(10000):
            mailbox.enqueue(f"Message-{i}")
        enqueue_time = time.time() - start
        
        assert mailbox.size() == 10000
        assert enqueue_time < 1.0  # Debe ser rápido
        
        # Dequeue todos
        start = time.time()
        count = 0
        while not mailbox.is_empty():
            mailbox.dequeue()
            count += 1
        dequeue_time = time.time() - start
        
        assert count == 10000
        assert dequeue_time < 1.0
    
    def test_priority_mailbox_performance(self):
        """Test PriorityMailbox performance."""
        priority_fn = lambda msg: int(msg.split("-")[1]) % 10
        mailbox = PriorityMailbox(priority_fn=priority_fn)
        
        # Encolar 1,000 mensajes con prioridades
        start = time.time()
        for i in range(1000):
            mailbox.enqueue(f"Message-{i}")
        enqueue_time = time.time() - start
        
        assert mailbox.size() == 1000
        assert enqueue_time < 0.5
        
        # Dequeue todos (ordenados por prioridad)
        start = time.time()
        count = 0
        while not mailbox.is_empty():
            mailbox.dequeue()
            count += 1
        dequeue_time = time.time() - start
        
        assert count == 1000
        assert dequeue_time < 0.5


class TestMailboxEdgeCases:
    """Tests de casos edge de mailboxes."""
    
    def test_enqueue_none_message(self):
        """Test que puede encolar None como mensaje."""
        mailbox = UnboundedMailbox()
        
        result = mailbox.enqueue(None)
        assert result == True
        
        message = mailbox.dequeue()
        assert message is None
    
    def test_enqueue_complex_objects(self):
        """Test que puede encolar objetos complejos."""
        mailbox = UnboundedMailbox()
        
        obj = {"type": "complex", "data": [1, 2, 3], "nested": {"key": "value"}}
        mailbox.enqueue(obj)
        
        retrieved = mailbox.dequeue()
        assert retrieved == obj
    
    def test_multiple_dequeue_empty(self):
        """Test múltiples dequeue en mailbox vacío."""
        mailbox = UnboundedMailbox()
        
        assert mailbox.dequeue() is None
        assert mailbox.dequeue() is None
        assert mailbox.dequeue() is None


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
