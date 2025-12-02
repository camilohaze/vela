"""
Actor System Architecture - Design Example

Jira: VELA-578
Task: TASK-036
Sprint: Sprint 16
Fecha: 2025-12-02

Este archivo contiene ejemplos de código que demuestran el diseño
del Actor System documentado en ADR-009.

NOTA: Este es código de diseño (no ejecutable aún).
La implementación real comenzará en TASK-037.
"""

# ============================================================================
# EJEMPLO 1: Counter Actor (Simple)
# ============================================================================

"""
Ejemplo básico de un actor con estado privado y message handlers.

Características:
- Estado privado (count)
- Message handlers con pattern matching
- Lifecycle hooks
"""

from typing import Union


# Message types (union type)
class Increment:
    pass


class Decrement:
    pass


class GetCount:
    def __init__(self, sender):
        self.sender = sender


class Reset:
    pass


Message = Union[Increment, Decrement, GetCount, Reset]


class CounterActor:
    """
    Actor simple que mantiene un contador.
    
    Estado privado:
        count: Number - Contador interno
    
    Mensajes aceptados:
        - Increment: Incrementa el contador
        - Decrement: Decrementa el contador
        - GetCount: Retorna el valor actual
        - Reset: Resetea a 0
    """
    
    def __init__(self):
        self._count = 0  # Estado privado
    
    def pre_start(self):
        """Lifecycle hook: Llamado antes de procesar mensajes."""
        print(f"CounterActor starting with count={self._count}")
    
    def post_stop(self):
        """Lifecycle hook: Llamado al detener el actor."""
        print(f"CounterActor stopped with final count={self._count}")
    
    def receive(self, message: Message):
        """
        Message handler principal.
        
        Pattern matching para determinar acción según mensaje.
        """
        if isinstance(message, Increment):
            self._count += 1
            print(f"Count incremented to {self._count}")
        
        elif isinstance(message, Decrement):
            self._count -= 1
            print(f"Count decremented to {self._count}")
        
        elif isinstance(message, GetCount):
            # Send response to sender
            message.sender.send(self._count)
        
        elif isinstance(message, Reset):
            self._count = 0
            print("Count reset to 0")


# ============================================================================
# EJEMPLO 2: Chat Room (Multiple Actors)
# ============================================================================

"""
Sistema de chat con múltiples actores comunicándose.

Características:
- Múltiples actores interactuando
- Broadcast de mensajes
- Gestión de estado (lista de usuarios)
"""


class Join:
    def __init__(self, user_ref, name):
        self.user_ref = user_ref
        self.name = name


class Leave:
    def __init__(self, user_ref):
        self.user_ref = user_ref


class ChatMessage:
    def __init__(self, sender_name, text):
        self.sender_name = sender_name
        self.text = text


class PostMessage:
    def __init__(self, text, sender_name):
        self.text = text
        self.sender_name = sender_name


class ChatRoomActor:
    """
    Chat room que gestiona usuarios y broadcast de mensajes.
    
    Estado privado:
        users: List[ActorRef] - Lista de usuarios conectados
    """
    
    def __init__(self):
        self._users = []  # Lista de ActorRefs
    
    def receive(self, message):
        if isinstance(message, Join):
            # Agregar usuario
            self._users.append(message.user_ref)
            print(f"{message.name} joined the chat")
            
            # Broadcast a todos
            self._broadcast(ChatMessage("System", f"{message.name} joined"))
        
        elif isinstance(message, Leave):
            # Remover usuario
            if message.user_ref in self._users:
                self._users.remove(message.user_ref)
                self._broadcast(ChatMessage("System", f"User left"))
        
        elif isinstance(message, PostMessage):
            # Broadcast mensaje a todos los usuarios
            self._broadcast(
                ChatMessage(message.sender_name, message.text)
            )
    
    def _broadcast(self, message):
        """Enviar mensaje a todos los usuarios."""
        for user in self._users:
            user.send(message)


class UserActor:
    """
    Usuario de chat que recibe mensajes.
    
    Estado privado:
        name: String - Nombre del usuario
    """
    
    def __init__(self, name: str):
        self._name = name
    
    def receive(self, message):
        if isinstance(message, ChatMessage):
            print(f"[{self._name}] {message.sender_name}: {message.text}")


# ============================================================================
# EJEMPLO 3: Producer-Consumer Pipeline
# ============================================================================

"""
Pipeline de procesamiento con actores.

Características:
- Producer genera trabajo
- Consumer procesa trabajo
- Comunicación asíncrona
- Señal de completitud
"""


class Start:
    pass


class Work:
    def __init__(self, data):
        self.data = data


class Done:
    pass


class ProducerActor:
    """
    Productor que genera trabajo y lo envía al consumer.
    
    Estado privado:
        consumer: ActorRef - Referencia al consumer
    """
    
    def __init__(self, consumer_ref):
        self._consumer = consumer_ref
    
    def receive(self, message):
        if isinstance(message, Start):
            print("Producer starting...")
            
            # Generar 1000 trabajos
            for i in range(1, 1001):
                self._consumer.send(Work(i))
            
            # Señal de completitud
            self._consumer.send(Done())
            print("Producer finished sending work")


class ConsumerActor:
    """
    Consumer que procesa trabajo.
    
    Estado privado:
        processed: Number - Contador de items procesados
    """
    
    def __init__(self):
        self._processed = 0
    
    def receive(self, message):
        if isinstance(message, Work):
            # Procesar trabajo (simulado)
            result = self._process(message.data)
            self._processed += 1
            
            if self._processed % 100 == 0:
                print(f"Processed {self._processed} items...")
        
        elif isinstance(message, Done):
            print(f"Consumer finished. Total processed: {self._processed}")
    
    def _process(self, data):
        """Simulación de procesamiento."""
        return data * 2


# ============================================================================
# EJEMPLO 4: Actor System API (Diseño)
# ============================================================================

"""
API propuesta para el Actor System.

Esto muestra cómo se usaría el sistema una vez implementado.
"""


class ActorSystemDesign:
    """
    Diseño del API del Actor System.
    
    Responsabilidades:
    - Crear y gestionar actores
    - Configurar thread pool y scheduler
    - Shutdown graceful del sistema
    """
    
    def __init__(self, name: str, config: dict = None):
        """
        Inicializar actor system.
        
        Args:
            name: Nombre del sistema
            config: Configuración (thread pool, scheduler, etc.)
        """
        self.name = name
        self.config = config or {}
        self._actors = {}  # actor_name -> ActorRef
        self._running = False
    
    def spawn(self, actor_class, name: str = None, **kwargs):
        """
        Crear instancia de actor.
        
        Args:
            actor_class: Clase del actor
            name: Nombre opcional del actor
            **kwargs: Argumentos para constructor del actor
        
        Returns:
            ActorRef: Referencia al actor
        """
        # Generar nombre si no se provee
        if name is None:
            name = f"{actor_class.__name__}-{len(self._actors)}"
        
        # Crear instancia del actor
        actor_instance = actor_class(**kwargs)
        
        # Crear ActorRef (proxy)
        actor_ref = ActorRef(name, actor_instance, self)
        
        # Registrar actor
        self._actors[name] = actor_ref
        
        # Llamar lifecycle hook
        actor_instance.pre_start()
        
        return actor_ref
    
    def stop(self, actor_ref):
        """
        Detener un actor.
        
        Args:
            actor_ref: Referencia al actor
        """
        # Llamar lifecycle hook
        actor_ref._actor.post_stop()
        
        # Remover del sistema
        del self._actors[actor_ref.name]
    
    def shutdown(self, timeout_seconds: int = 10):
        """
        Shutdown graceful del sistema.
        
        Args:
            timeout_seconds: Tiempo máximo de espera
        """
        print(f"Shutting down actor system '{self.name}'...")
        
        # Detener todos los actores
        for actor_ref in list(self._actors.values()):
            self.stop(actor_ref)
        
        self._running = False
        print("Actor system shutdown complete")


class ActorRef:
    """
    Referencia a un actor (proxy).
    
    Características:
    - Location transparency (local y remoto igual)
    - Type-safe (tipado con actor type)
    - Serializable (puede pasar por red)
    """
    
    def __init__(self, name: str, actor, system):
        self.name = name
        self._actor = actor
        self._system = system
    
    def send(self, message):
        """
        Enviar mensaje al actor (asíncrono).
        
        Args:
            message: Mensaje a enviar
        """
        # Agregar mensaje al mailbox del actor
        # (esto será implementado en TASK-038)
        self._actor.receive(message)
    
    def ask(self, message, timeout_seconds: int = 5):
        """
        Request-response pattern (síncrono con timeout).
        
        Args:
            message: Mensaje a enviar
            timeout_seconds: Timeout para respuesta
        
        Returns:
            Respuesta del actor
        """
        # Implementación futura (requiere Futures - Sprint 18)
        raise NotImplementedError("ask() requires Futures (Sprint 18)")
    
    def __eq__(self, other):
        """Dos refs al mismo actor son iguales."""
        return isinstance(other, ActorRef) and self.name == other.name
    
    def __hash__(self):
        return hash(self.name)
    
    def __repr__(self):
        return f"ActorRef({self.name})"


# ============================================================================
# EJEMPLO DE USO COMPLETO
# ============================================================================

def example_usage():
    """
    Ejemplo completo de uso del Actor System.
    
    Demuestra:
    - Creación de sistema
    - Spawning de actores
    - Envío de mensajes
    - Shutdown graceful
    """
    
    # 1. Crear actor system
    system = ActorSystemDesign(name="MyAppSystem")
    
    # 2. Spawn actors
    counter = system.spawn(CounterActor, name="counter1")
    
    # 3. Send messages
    counter.send(Increment())
    counter.send(Increment())
    counter.send(Increment())
    # Output: Count incremented to 1, 2, 3
    
    # 4. Chat example
    chat_room = system.spawn(ChatRoomActor, name="room1")
    alice = system.spawn(UserActor, name="alice", name_arg="Alice")
    bob = system.spawn(UserActor, name="bob", name_arg="Bob")
    
    chat_room.send(Join(alice, "Alice"))
    chat_room.send(Join(bob, "Bob"))
    chat_room.send(PostMessage("Hello everyone!", "Alice"))
    
    # 5. Pipeline example
    consumer = system.spawn(ConsumerActor, name="consumer1")
    producer = system.spawn(ProducerActor, name="producer1", 
                           consumer_ref=consumer)
    
    producer.send(Start())
    
    # 6. Shutdown system
    system.shutdown()


# ============================================================================
# CONFIGURACIÓN DEL SISTEMA
# ============================================================================

class ActorSystemConfig:
    """
    Configuración del Actor System.
    
    Permite customizar:
    - Thread pool (min/max threads)
    - Scheduler (fair, priority, work-conserving)
    - Default mailbox (bounded, unbounded, priority)
    - Timeouts y límites
    """
    
    def __init__(self):
        # Thread Pool config
        self.min_threads = 2
        self.max_threads = 100
        self.thread_keep_alive_seconds = 60
        self.queue_capacity = 1000
        
        # Scheduler config
        self.scheduler_type = "fair"  # fair | priority | work-conserving
        self.fairness_quantum = 10  # Mensajes por quantum
        
        # Mailbox config
        self.default_mailbox_type = "unbounded"  # unbounded | bounded | priority
        self.bounded_mailbox_capacity = 1000
        
        # Timeouts
        self.default_ask_timeout_seconds = 5
        self.shutdown_timeout_seconds = 10
        
        # Limits
        self.max_actors = 10000
        self.max_messages_per_actor = 100000
    
    def to_dict(self):
        """Convertir a diccionario para serialización."""
        return {
            "thread_pool": {
                "min_threads": self.min_threads,
                "max_threads": self.max_threads,
                "keep_alive": self.thread_keep_alive_seconds,
                "queue_capacity": self.queue_capacity,
            },
            "scheduler": {
                "type": self.scheduler_type,
                "fairness_quantum": self.fairness_quantum,
            },
            "mailbox": {
                "default_type": self.default_mailbox_type,
                "bounded_capacity": self.bounded_mailbox_capacity,
            },
            "timeouts": {
                "ask": self.default_ask_timeout_seconds,
                "shutdown": self.shutdown_timeout_seconds,
            },
            "limits": {
                "max_actors": self.max_actors,
                "max_messages_per_actor": self.max_messages_per_actor,
            }
        }


if __name__ == "__main__":
    print("=" * 80)
    print("Actor System Architecture - Design Examples")
    print("=" * 80)
    print("\nNOTA: Estos son ejemplos de diseño (no ejecutables aún).")
    print("La implementación comenzará en TASK-037.\n")
    
    # Mostrar configuración por defecto
    config = ActorSystemConfig()
    print("Configuración por defecto:")
    print(config.to_dict())
    
    print("\n" + "=" * 80)
    print("Ejemplo de uso (conceptual):")
    print("=" * 80)
    
    # Mostrar ejemplo de código
    print("""
    # 1. Crear sistema
    system = ActorSystem(name="MySystem")
    
    # 2. Spawn actors
    counter = system.spawn(Counter)
    
    # 3. Send messages
    counter.send(Increment)
    
    # 4. Shutdown
    system.shutdown()
    """)
