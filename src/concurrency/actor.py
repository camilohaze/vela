"""
Actor Instance Implementation

Jira: VELA-578
Task: TASK-037
Sprint: Sprint 16
Fecha: 2025-12-02

Implementación de Actor instances con:
- Estado privado encapsulado
- Message handlers con receive()
- Lifecycle hooks (pre_start, post_stop, pre_restart, post_restart)
- ActorRef para location transparency
"""

from abc import ABC, abstractmethod
from typing import Any, Optional, Dict, Callable
from enum import Enum


class ActorState(Enum):
    """
    Estados del ciclo de vida de un actor.
    
    Valores:
        UNINITIALIZED: Actor creado pero no iniciado
        STARTING: En proceso de inicialización
        RUNNING: Activo y procesando mensajes
        STOPPING: En proceso de detención
        STOPPED: Detenido completamente
        RESTARTING: En proceso de reinicio
    """
    UNINITIALIZED = "uninitialized"
    STARTING = "starting"
    RUNNING = "running"
    STOPPING = "stopping"
    STOPPED = "stopped"
    RESTARTING = "restarting"


class Actor(ABC):
    """
    Clase base abstracta para todos los actores.
    
    Características:
    - Estado privado (no accesible desde fuera)
    - Message handler único (receive)
    - Lifecycle hooks
    - Single-threaded processing (garantizado por mailbox)
    
    Uso:
        class MyActor(Actor):
            def __init__(self):
                super().__init__()
                self._state_var = 0
            
            def receive(self, message):
                if isinstance(message, MyMessage):
                    self._state_var += 1
    """
    
    def __init__(self):
        """
        Inicializar actor base.
        
        Nota: Subclases deben llamar super().__init__()
        """
        self._actor_state = ActorState.UNINITIALIZED
        self._actor_ref: Optional['ActorRef'] = None
        self._message_count = 0
        self._error_count = 0
    
    # ========================================================================
    # LIFECYCLE HOOKS
    # ========================================================================
    
    def pre_start(self) -> None:
        """
        Lifecycle hook: Llamado antes de procesar mensajes.
        
        Útil para:
        - Inicializar recursos (DB connections, file handles)
        - Subscribir a eventos
        - Configurar estado inicial
        
        Override en subclase si necesitas inicialización custom.
        """
        pass
    
    def post_stop(self) -> None:
        """
        Lifecycle hook: Llamado al detener el actor.
        
        Útil para:
        - Cerrar recursos (DB connections, file handles)
        - Unsubscribe de eventos
        - Cleanup de estado
        
        Override en subclase si necesitas cleanup custom.
        """
        pass
    
    def pre_restart(self, error: Exception) -> None:
        """
        Lifecycle hook: Llamado antes de reiniciar por error.
        
        Args:
            error: Excepción que causó el restart
        
        Útil para:
        - Log del error
        - Cleanup parcial
        - Notificar supervisores
        
        Override en subclase si necesitas restart custom.
        """
        pass
    
    def post_restart(self, error: Exception) -> None:
        """
        Lifecycle hook: Llamado después de reiniciar.
        
        Args:
            error: Excepción que causó el restart
        
        Útil para:
        - Reinicializar estado
        - Reconectar recursos
        
        Override en subclase si necesitas post-restart custom.
        """
        pass
    
    # ========================================================================
    # MESSAGE HANDLING (ABSTRACT - OBLIGATORIO IMPLEMENTAR)
    # ========================================================================
    
    @abstractmethod
    def receive(self, message: Any) -> None:
        """
        Message handler principal.
        
        DEBE ser implementado por todas las subclases.
        
        Args:
            message: Mensaje a procesar
        
        Ejemplo:
            def receive(self, message):
                if isinstance(message, Increment):
                    self._count += 1
                elif isinstance(message, Decrement):
                    self._count -= 1
        
        Nota:
        - Este método es llamado secuencialmente (un mensaje a la vez)
        - NO usar blocking operations aquí
        - Si lanza excepción, el supervisor decide qué hacer
        """
        pass
    
    # ========================================================================
    # ACTOR REFERENCE
    # ========================================================================
    
    def self(self) -> 'ActorRef':
        """
        Obtener referencia al propio actor.
        
        Returns:
            ActorRef: Referencia a este actor
        
        Útil para:
        - Pasarse a sí mismo como sender en mensajes
        - Auto-enviarse mensajes (recursión)
        
        Example:
            other_actor.send(GetValue(sender=self.self()))
        """
        if self._actor_ref is None:
            raise RuntimeError("Actor not initialized (no ActorRef)")
        return self._actor_ref
    
    def _set_actor_ref(self, ref: 'ActorRef') -> None:
        """
        Set actor reference (internal use only).
        
        Args:
            ref: ActorRef para este actor
        """
        self._actor_ref = ref
    
    # ========================================================================
    # STATE MANAGEMENT
    # ========================================================================
    
    def get_state(self) -> ActorState:
        """
        Obtener estado actual del actor.
        
        Returns:
            ActorState: Estado actual
        """
        return self._actor_state
    
    def _set_state(self, state: ActorState) -> None:
        """
        Set actor state (internal use only).
        
        Args:
            state: Nuevo estado
        """
        self._actor_state = state
    
    # ========================================================================
    # METRICS
    # ========================================================================
    
    def get_message_count(self) -> int:
        """
        Obtener número de mensajes procesados.
        
        Returns:
            int: Cantidad de mensajes procesados
        """
        return self._message_count
    
    def get_error_count(self) -> int:
        """
        Obtener número de errores encontrados.
        
        Returns:
            int: Cantidad de errores
        """
        return self._error_count
    
    def _increment_message_count(self) -> None:
        """Incrementar contador de mensajes (internal use only)."""
        self._message_count += 1
    
    def _increment_error_count(self) -> None:
        """Incrementar contador de errores (internal use only)."""
        self._error_count += 1


class ActorRef:
    """
    Referencia a un actor (proxy location-transparent).
    
    Características:
    - Location transparency (local y remoto igual)
    - Type-safe (puede tiparse: ActorRef[CounterActor])
    - Serializable (puede pasar por red - futuro)
    - Inmutable (no puede cambiar el actor al que apunta)
    
    Uso:
        ref = spawn(MyActor)
        ref.send(MyMessage())
    """
    
    def __init__(self, name: str, actor: Actor):
        """
        Crear ActorRef.
        
        Args:
            name: Nombre único del actor
            actor: Instancia del actor
        
        Nota: Normalmente no se crea directamente, usar spawn()
        """
        self._name = name
        self._actor = actor
        self._stopped = False
        
        # Set reference en el actor
        actor._set_actor_ref(self)
    
    @property
    def name(self) -> str:
        """
        Nombre del actor.
        
        Returns:
            str: Nombre único
        """
        return self._name
    
    @property
    def path(self) -> str:
        """
        Path del actor (location-transparent).
        
        Returns:
            str: Path único del actor
        
        Example:
            "akka://MySystem/user/counter1"
        
        Nota: Futuro - soportará actores remotos
        """
        return f"local://{self._name}"
    
    def send(self, message: Any) -> None:
        """
        Enviar mensaje al actor (asíncrono, fire-and-forget).
        
        Args:
            message: Mensaje a enviar
        
        Características:
        - Non-blocking (retorna inmediatamente)
        - No garantiza orden entre diferentes senders
        - Garantiza orden FIFO del mismo sender
        
        Example:
            counter.send(Increment())
        """
        if self._stopped:
            raise RuntimeError(f"Actor {self._name} is stopped")
        
        # Temporalmente llamar receive directamente
        # En TASK-038 esto irá al mailbox
        self._actor.receive(message)
        self._actor._increment_message_count()
    
    def tell(self, message: Any) -> None:
        """
        Alias de send() (Akka-style).
        
        Args:
            message: Mensaje a enviar
        """
        self.send(message)
    
    def stop(self) -> None:
        """
        Detener el actor.
        
        Características:
        - Llama post_stop() hook
        - Procesa mensajes pendientes antes de detener
        - Marca como stopped (rechaza nuevos mensajes)
        """
        if self._stopped:
            return
        
        self._actor._set_state(ActorState.STOPPING)
        self._actor.post_stop()
        self._actor._set_state(ActorState.STOPPED)
        self._stopped = True
    
    def is_stopped(self) -> bool:
        """
        Check si el actor está detenido.
        
        Returns:
            bool: True si está stopped
        """
        return self._stopped
    
    def __eq__(self, other) -> bool:
        """
        Igualdad de ActorRef (por nombre).
        
        Args:
            other: Otro ActorRef
        
        Returns:
            bool: True si apuntan al mismo actor
        """
        return isinstance(other, ActorRef) and self._name == other._name
    
    def __hash__(self) -> int:
        """
        Hash de ActorRef (por nombre).
        
        Returns:
            int: Hash del nombre
        """
        return hash(self._name)
    
    def __repr__(self) -> str:
        """
        Representación string.
        
        Returns:
            str: Representación legible
        """
        status = "stopped" if self._stopped else "running"
        return f"ActorRef({self._name}, {status})"


# ============================================================================
# SPAWN FUNCTION (Actor Creation)
# ============================================================================

_actor_counter = 0


def spawn(actor_class: type, name: Optional[str] = None, **kwargs) -> ActorRef:
    """
    Crear instancia de un actor.
    
    Args:
        actor_class: Clase del actor a crear
        name: Nombre opcional del actor (auto-generado si None)
        **kwargs: Argumentos para el constructor del actor
    
    Returns:
        ActorRef: Referencia al actor creado
    
    Example:
        counter = spawn(CounterActor, name="counter1")
        user = spawn(UserActor, name="alice", user_name="Alice")
    
    Nota:
    - Llama pre_start() lifecycle hook
    - Actor comienza en estado RUNNING
    """
    global _actor_counter
    
    # Generar nombre si no se provee
    if name is None:
        _actor_counter += 1
        name = f"{actor_class.__name__}-{_actor_counter}"
    
    # Crear instancia del actor
    actor_instance = actor_class(**kwargs)
    
    # Validar que es subclase de Actor
    if not isinstance(actor_instance, Actor):
        raise TypeError(f"{actor_class.__name__} must inherit from Actor")
    
    # Crear ActorRef
    actor_ref = ActorRef(name, actor_instance)
    
    # Lifecycle: pre_start
    actor_instance._set_state(ActorState.STARTING)
    actor_instance.pre_start()
    actor_instance._set_state(ActorState.RUNNING)
    
    return actor_ref


# ============================================================================
# EXAMPLE ACTORS
# ============================================================================

class CounterActor(Actor):
    """
    Actor de ejemplo: contador simple.
    
    Mensajes soportados:
        - Increment: Incrementa contador
        - Decrement: Decrementa contador
        - GetCount: Retorna valor actual
        - Reset: Resetea a 0
    """
    
    def __init__(self):
        super().__init__()
        self._count = 0
    
    def pre_start(self):
        print(f"CounterActor starting with count={self._count}")
    
    def post_stop(self):
        print(f"CounterActor stopped with final count={self._count}")
    
    def receive(self, message):
        if message == "Increment":
            self._count += 1
        elif message == "Decrement":
            self._count -= 1
        elif message == "Reset":
            self._count = 0
        elif isinstance(message, tuple) and message[0] == "GetCount":
            sender = message[1]
            sender.send(("CountResult", self._count))
    
    def get_count(self) -> int:
        """Get current count (for testing only)."""
        return self._count


if __name__ == "__main__":
    print("=" * 80)
    print("Actor Instance Implementation - Examples")
    print("=" * 80)
    
    # Example 1: Simple counter
    print("\n1. Counter Actor:")
    counter = spawn(CounterActor, name="counter1")
    
    counter.send("Increment")
    counter.send("Increment")
    counter.send("Increment")
    print(f"Count: {counter._actor._count}")  # Should be 3
    
    counter.send("Decrement")
    print(f"Count: {counter._actor._count}")  # Should be 2
    
    counter.send("Reset")
    print(f"Count: {counter._actor._count}")  # Should be 0
    
    # Stop actor
    counter.stop()
    print(f"Actor stopped: {counter.is_stopped()}")
    
    print("\n" + "=" * 80)
    print("Tests in tests/unit/concurrency/test_actor.py")
    print("=" * 80)
