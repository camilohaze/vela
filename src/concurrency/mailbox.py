"""
Mailbox System Implementation

Jira: VELA-578
Task: TASK-038
Sprint: Sprint 16
Fecha: 2025-12-02

Implementación de sistema de mailboxes con:
- UnboundedMailbox (sin límite)
- BoundedMailbox (con límite, backpressure)
- PriorityMailbox (ordenado por prioridad)
- FIFO ordering (mismo sender)
"""

from abc import ABC, abstractmethod
from typing import Any, Optional, Callable, List
from collections import deque
from queue import Queue, PriorityQueue
from threading import Lock
import heapq


class Mailbox(ABC):
    """
    Clase base abstracta para mailboxes.
    
    Responsabilidades:
    - Almacenar mensajes en cola
    - Ordenamiento (FIFO, priority)
    - Backpressure (bounded mailboxes)
    """
    
    def __init__(self):
        """Inicializar mailbox."""
        self._lock = Lock()
        self._message_count = 0
    
    @abstractmethod
    def enqueue(self, message: Any) -> bool:
        """
        Agregar mensaje a la cola.
        
        Args:
            message: Mensaje a encolar
        
        Returns:
            bool: True si se encoló, False si se rechazó
        
        Nota:
        - BoundedMailbox puede rechazar si está lleno
        - UnboundedMailbox siempre retorna True
        """
        pass
    
    @abstractmethod
    def dequeue(self) -> Optional[Any]:
        """
        Extraer mensaje de la cola (no-blocking).
        
        Returns:
            Optional[Any]: Mensaje si hay, None si vacío
        """
        pass
    
    @abstractmethod
    def is_empty(self) -> bool:
        """
        Check si el mailbox está vacío.
        
        Returns:
            bool: True si no hay mensajes
        """
        pass
    
    @abstractmethod
    def size(self) -> int:
        """
        Obtener cantidad de mensajes en cola.
        
        Returns:
            int: Número de mensajes
        """
        pass
    
    def get_message_count(self) -> int:
        """
        Obtener total de mensajes procesados (lifetime).
        
        Returns:
            int: Total de mensajes encolados
        """
        return self._message_count


class UnboundedMailbox(Mailbox):
    """
    Mailbox sin límite de capacidad.
    
    Características:
    - FIFO ordering
    - No rechaza mensajes
    - Riesgo: OutOfMemory si producer >> consumer
    
    Uso:
        mailbox = UnboundedMailbox()
        mailbox.enqueue("Message1")
        msg = mailbox.dequeue()
    """
    
    def __init__(self):
        """Inicializar unbounded mailbox."""
        super().__init__()
        self._queue: deque = deque()
    
    def enqueue(self, message: Any) -> bool:
        """
        Agregar mensaje (siempre exitoso).
        
        Args:
            message: Mensaje a encolar
        
        Returns:
            bool: Siempre True
        """
        with self._lock:
            self._queue.append(message)
            self._message_count += 1
        return True
    
    def dequeue(self) -> Optional[Any]:
        """
        Extraer mensaje (FIFO).
        
        Returns:
            Optional[Any]: Mensaje si hay, None si vacío
        """
        with self._lock:
            if len(self._queue) == 0:
                return None
            return self._queue.popleft()
    
    def is_empty(self) -> bool:
        """
        Check si está vacío.
        
        Returns:
            bool: True si no hay mensajes
        """
        with self._lock:
            return len(self._queue) == 0
    
    def size(self) -> int:
        """
        Cantidad de mensajes en cola.
        
        Returns:
            int: Número de mensajes
        """
        with self._lock:
            return len(self._queue)


class BoundedMailbox(Mailbox):
    """
    Mailbox con límite de capacidad (backpressure).
    
    Características:
    - FIFO ordering
    - Rechaza mensajes cuando lleno
    - Backpressure automático
    
    Uso:
        mailbox = BoundedMailbox(capacity=1000)
        success = mailbox.enqueue("Message")
        if not success:
            print("Mailbox full!")
    """
    
    def __init__(self, capacity: int = 1000):
        """
        Inicializar bounded mailbox.
        
        Args:
            capacity: Capacidad máxima (default: 1000)
        """
        super().__init__()
        if capacity <= 0:
            raise ValueError(f"Capacity must be positive, got {capacity}")
        
        self._capacity = capacity
        self._queue: deque = deque(maxlen=capacity)
        self._rejected_count = 0
    
    def enqueue(self, message: Any) -> bool:
        """
        Agregar mensaje (puede fallar si lleno).
        
        Args:
            message: Mensaje a encolar
        
        Returns:
            bool: True si se encoló, False si rechazado
        """
        with self._lock:
            if len(self._queue) >= self._capacity:
                self._rejected_count += 1
                return False
            
            self._queue.append(message)
            self._message_count += 1
            return True
    
    def dequeue(self) -> Optional[Any]:
        """
        Extraer mensaje (FIFO).
        
        Returns:
            Optional[Any]: Mensaje si hay, None si vacío
        """
        with self._lock:
            if len(self._queue) == 0:
                return None
            return self._queue.popleft()
    
    def is_empty(self) -> bool:
        """
        Check si está vacío.
        
        Returns:
            bool: True si no hay mensajes
        """
        with self._lock:
            return len(self._queue) == 0
    
    def size(self) -> int:
        """
        Cantidad de mensajes en cola.
        
        Returns:
            int: Número de mensajes
        """
        with self._lock:
            return len(self._queue)
    
    def is_full(self) -> bool:
        """
        Check si el mailbox está lleno.
        
        Returns:
            bool: True si no acepta más mensajes
        """
        with self._lock:
            return len(self._queue) >= self._capacity
    
    def get_capacity(self) -> int:
        """
        Obtener capacidad máxima.
        
        Returns:
            int: Capacidad configurada
        """
        return self._capacity
    
    def get_rejected_count(self) -> int:
        """
        Obtener cantidad de mensajes rechazados.
        
        Returns:
            int: Número de rechazos
        """
        return self._rejected_count


class PriorityMailbox(Mailbox):
    """
    Mailbox ordenado por prioridad.
    
    Características:
    - Priority ordering (menor número = mayor prioridad)
    - System messages > High > Normal > Low
    - FIFO dentro de la misma prioridad
    
    Uso:
        mailbox = PriorityMailbox(
            priority_fn = lambda msg: 0 if msg.type == "System" else 10
        )
        mailbox.enqueue("NormalMessage")
        mailbox.enqueue("SystemMessage")
        # SystemMessage se procesa primero
    """
    
    def __init__(self, priority_fn: Optional[Callable[[Any], int]] = None):
        """
        Inicializar priority mailbox.
        
        Args:
            priority_fn: Función que asigna prioridad (default: prioridad 10)
        """
        super().__init__()
        self._priority_fn = priority_fn or (lambda msg: 10)
        self._heap: List = []
        self._counter = 0  # Para mantener FIFO dentro de misma prioridad
    
    def enqueue(self, message: Any) -> bool:
        """
        Agregar mensaje con prioridad.
        
        Args:
            message: Mensaje a encolar
        
        Returns:
            bool: Siempre True
        """
        with self._lock:
            priority = self._priority_fn(message)
            # Heap: (priority, counter, message)
            # Counter garantiza FIFO dentro de misma prioridad
            heapq.heappush(self._heap, (priority, self._counter, message))
            self._counter += 1
            self._message_count += 1
        return True
    
    def dequeue(self) -> Optional[Any]:
        """
        Extraer mensaje de mayor prioridad.
        
        Returns:
            Optional[Any]: Mensaje si hay, None si vacío
        """
        with self._lock:
            if len(self._heap) == 0:
                return None
            
            priority, counter, message = heapq.heappop(self._heap)
            return message
    
    def is_empty(self) -> bool:
        """
        Check si está vacío.
        
        Returns:
            bool: True si no hay mensajes
        """
        with self._lock:
            return len(self._heap) == 0
    
    def size(self) -> int:
        """
        Cantidad de mensajes en cola.
        
        Returns:
            int: Número de mensajes
        """
        with self._lock:
            return len(self._heap)


# ============================================================================
# MAILBOX FACTORY
# ============================================================================

class MailboxType:
    """Tipos de mailbox disponibles."""
    UNBOUNDED = "unbounded"
    BOUNDED = "bounded"
    PRIORITY = "priority"


def create_mailbox(
    mailbox_type: str = MailboxType.UNBOUNDED,
    **kwargs
) -> Mailbox:
    """
    Factory para crear mailboxes.
    
    Args:
        mailbox_type: Tipo de mailbox (unbounded, bounded, priority)
        **kwargs: Argumentos específicos del tipo
    
    Returns:
        Mailbox: Instancia del mailbox
    
    Examples:
        # Unbounded
        mailbox = create_mailbox("unbounded")
        
        # Bounded
        mailbox = create_mailbox("bounded", capacity=500)
        
        # Priority
        mailbox = create_mailbox(
            "priority",
            priority_fn=lambda msg: 0 if msg == "System" else 10
        )
    """
    if mailbox_type == MailboxType.UNBOUNDED:
        return UnboundedMailbox()
    
    elif mailbox_type == MailboxType.BOUNDED:
        capacity = kwargs.get("capacity", 1000)
        return BoundedMailbox(capacity=capacity)
    
    elif mailbox_type == MailboxType.PRIORITY:
        priority_fn = kwargs.get("priority_fn")
        return PriorityMailbox(priority_fn=priority_fn)
    
    else:
        raise ValueError(f"Unknown mailbox type: {mailbox_type}")


# ============================================================================
# INTEGRATION WITH ACTOR
# ============================================================================

class ActorWithMailbox:
    """
    Example: Actor integrado con Mailbox.
    
    Demuestra cómo se usará en TASK-039 (Message Loop).
    """
    
    def __init__(self, mailbox: Mailbox):
        """
        Crear actor con mailbox.
        
        Args:
            mailbox: Mailbox para mensajes
        """
        self._mailbox = mailbox
        self._running = True
    
    def send(self, message: Any) -> bool:
        """
        Enviar mensaje al mailbox.
        
        Args:
            message: Mensaje a enviar
        
        Returns:
            bool: True si se encoló exitosamente
        """
        return self._mailbox.enqueue(message)
    
    def receive(self, message: Any) -> None:
        """
        Procesar mensaje (implementar en subclase).
        
        Args:
            message: Mensaje a procesar
        """
        print(f"Received: {message}")
    
    def process_next_message(self) -> bool:
        """
        Procesar un mensaje del mailbox.
        
        Returns:
            bool: True si procesó un mensaje, False si vacío
        """
        message = self._mailbox.dequeue()
        if message is None:
            return False
        
        self.receive(message)
        return True


if __name__ == "__main__":
    print("=" * 80)
    print("Mailbox System Implementation - Examples")
    print("=" * 80)
    
    # Example 1: UnboundedMailbox
    print("\n1. UnboundedMailbox:")
    unbounded = UnboundedMailbox()
    
    unbounded.enqueue("Message 1")
    unbounded.enqueue("Message 2")
    unbounded.enqueue("Message 3")
    
    print(f"Size: {unbounded.size()}")
    print(f"Dequeue: {unbounded.dequeue()}")
    print(f"Dequeue: {unbounded.dequeue()}")
    print(f"Size: {unbounded.size()}")
    
    # Example 2: BoundedMailbox
    print("\n2. BoundedMailbox:")
    bounded = BoundedMailbox(capacity=2)
    
    print(f"Enqueue 1: {bounded.enqueue('A')}")  # True
    print(f"Enqueue 2: {bounded.enqueue('B')}")  # True
    print(f"Enqueue 3: {bounded.enqueue('C')}")  # False (full)
    
    print(f"Size: {bounded.size()}")
    print(f"Is full: {bounded.is_full()}")
    print(f"Rejected: {bounded.get_rejected_count()}")
    
    # Example 3: PriorityMailbox
    print("\n3. PriorityMailbox:")
    priority = PriorityMailbox(
        priority_fn=lambda msg: 0 if msg.startswith("URGENT") else 10
    )
    
    priority.enqueue("Normal message")
    priority.enqueue("URGENT: Critical!")
    priority.enqueue("Another normal")
    
    print(f"Size: {priority.size()}")
    print(f"Dequeue: {priority.dequeue()}")  # URGENT primero
    print(f"Dequeue: {priority.dequeue()}")  # Normal
    print(f"Dequeue: {priority.dequeue()}")  # Normal
    
    print("\n" + "=" * 80)
    print("Tests in tests/unit/concurrency/test_mailbox.py")
    print("=" * 80)
