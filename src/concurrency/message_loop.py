"""
Message Processing Loop for Actor System

This module implements the message processing loop that connects
Actor instances with their Mailboxes, enabling sequential message
processing with error handling and lifecycle management.

Jira: VELA-578
Task: TASK-039
Sprint: Sprint 16
"""

from abc import ABC, abstractmethod
from typing import Any, Optional, Callable
from enum import Enum
import threading
import time
from src.concurrency.actor import Actor, ActorState
from src.concurrency.mailbox import Mailbox, UnboundedMailbox


class MessageLoopState(Enum):
    """Estados posibles del message loop."""
    IDLE = "idle"           # Loop inactivo (no iniciado)
    RUNNING = "running"     # Loop ejecutándose
    PAUSED = "paused"       # Loop pausado temporalmente
    STOPPING = "stopping"   # Loop deteniéndose
    STOPPED = "stopped"     # Loop detenido


class MessageProcessor(ABC):
    """
    Abstract base class para procesadores de mensajes.
    
    Define la interfaz que deben implementar los procesadores
    que manejan mensajes desde el mailbox.
    """
    
    @abstractmethod
    def process_message(self, message: Any) -> None:
        """
        Procesar un mensaje.
        
        Args:
            message: El mensaje a procesar
        """
        pass
    
    @abstractmethod
    def handle_error(self, error: Exception, message: Any) -> None:
        """
        Manejar un error durante el procesamiento.
        
        Args:
            error: La excepción que ocurrió
            message: El mensaje que causó el error
        """
        pass


class MessageLoop:
    """
    Loop de procesamiento de mensajes para un actor.
    
    Responsabilidades:
    - Extraer mensajes del mailbox
    - Procesarlos secuencialmente (uno a la vez)
    - Manejar errores durante procesamiento
    - Controlar ciclo de vida del loop (start, stop, pause)
    - Recolectar métricas de procesamiento
    
    Inspiración:
    - Erlang: Proceso con mailbox + receive loop
    - Akka: ActorCell con message dispatcher
    - Pony: Actor runtime con behavior execution
    """
    
    def __init__(
        self,
        mailbox: Mailbox,
        processor: MessageProcessor,
        max_throughput: Optional[int] = None,
        idle_sleep_ms: int = 1
    ):
        """
        Inicializar message loop.
        
        Args:
            mailbox: Mailbox del cual leer mensajes
            processor: Procesador que maneja los mensajes
            max_throughput: Máximo de mensajes por ciclo (None = sin límite)
            idle_sleep_ms: Milisegundos a dormir cuando mailbox vacío
        """
        self._mailbox = mailbox
        self._processor = processor
        self._max_throughput = max_throughput
        self._idle_sleep_ms = idle_sleep_ms
        
        # Estado del loop
        self._state: MessageLoopState = MessageLoopState.IDLE
        self._thread: Optional[threading.Thread] = None
        self._lock = threading.Lock()
        
        # Métricas
        self._messages_processed = 0
        self._errors_count = 0
        self._cycles_count = 0
        self._total_processing_time = 0.0
    
    def start(self) -> None:
        """
        Iniciar el message loop en un thread separado.
        
        Raises:
            RuntimeError: Si el loop ya está corriendo
        """
        with self._lock:
            if self._state == MessageLoopState.RUNNING:
                raise RuntimeError("MessageLoop already running")
            
            if self._state == MessageLoopState.PAUSED:
                raise RuntimeError("MessageLoop is paused, use resume() instead")
            
            self._state = MessageLoopState.RUNNING
            self._thread = threading.Thread(target=self._run_loop, daemon=True)
            self._thread.start()
    
    def stop(self, timeout: Optional[float] = None) -> None:
        """
        Detener el message loop.
        
        Args:
            timeout: Segundos máximos a esperar que termine (None = sin límite)
        
        Raises:
            RuntimeError: Si el loop no está corriendo
        """
        with self._lock:
            if self._state not in (MessageLoopState.RUNNING, MessageLoopState.PAUSED):
                raise RuntimeError("MessageLoop not running")
            
            self._state = MessageLoopState.STOPPING
        
        # Esperar que termine el thread
        if self._thread and self._thread.is_alive():
            self._thread.join(timeout=timeout)
        
        with self._lock:
            self._state = MessageLoopState.STOPPED
    
    def pause(self) -> None:
        """
        Pausar el message loop temporalmente.
        
        Raises:
            RuntimeError: Si el loop no está corriendo
        """
        with self._lock:
            if self._state != MessageLoopState.RUNNING:
                raise RuntimeError("MessageLoop not running")
            
            self._state = MessageLoopState.PAUSED
    
    def resume(self) -> None:
        """
        Resumir el message loop pausado.
        
        Raises:
            RuntimeError: Si el loop no está pausado
        """
        with self._lock:
            if self._state != MessageLoopState.PAUSED:
                raise RuntimeError("MessageLoop not paused")
            
            self._state = MessageLoopState.RUNNING
    
    def _run_loop(self) -> None:
        """
        Loop principal de procesamiento (ejecutado en thread separado).
        """
        while True:
            with self._lock:
                current_state = self._state
            
            # Verificar si debemos detener
            if current_state == MessageLoopState.STOPPING:
                break
            
            # Verificar si estamos pausados
            if current_state == MessageLoopState.PAUSED:
                time.sleep(self._idle_sleep_ms / 1000.0)
                continue
            
            # Procesar un ciclo de mensajes
            self._process_cycle()
    
    def _process_cycle(self) -> None:
        """
        Procesar un ciclo de mensajes del mailbox.
        """
        self._cycles_count += 1
        messages_in_cycle = 0
        
        # Procesar hasta max_throughput mensajes (o todos si None)
        while True:
            # Verificar límite de throughput
            if self._max_throughput and messages_in_cycle >= self._max_throughput:
                break
            
            # Intentar extraer mensaje
            message = self._mailbox.dequeue()
            
            if message is None:
                # Mailbox vacío, dormir un poco
                time.sleep(self._idle_sleep_ms / 1000.0)
                break
            
            # Procesar mensaje
            start_time = time.time()
            try:
                self._processor.process_message(message)
                self._messages_processed += 1
            except Exception as e:
                self._errors_count += 1
                try:
                    self._processor.handle_error(e, message)
                except Exception as handler_error:
                    # Error handler falló, solo contar
                    self._errors_count += 1
            finally:
                processing_time = time.time() - start_time
                self._total_processing_time += processing_time
            
            messages_in_cycle += 1
    
    def get_state(self) -> MessageLoopState:
        """Obtener estado actual del loop."""
        with self._lock:
            return self._state
    
    def is_running(self) -> bool:
        """Verificar si el loop está corriendo."""
        return self.get_state() == MessageLoopState.RUNNING
    
    def get_messages_processed(self) -> int:
        """Obtener cantidad de mensajes procesados exitosamente."""
        return self._messages_processed
    
    def get_errors_count(self) -> int:
        """Obtener cantidad de errores durante procesamiento."""
        return self._errors_count
    
    def get_cycles_count(self) -> int:
        """Obtener cantidad de ciclos de procesamiento ejecutados."""
        return self._cycles_count
    
    def get_average_processing_time(self) -> float:
        """
        Obtener tiempo promedio de procesamiento por mensaje (en segundos).
        
        Returns:
            Tiempo promedio en segundos, o 0.0 si no hay mensajes procesados
        """
        if self._messages_processed == 0:
            return 0.0
        return self._total_processing_time / self._messages_processed


class ActorMessageProcessor(MessageProcessor):
    """
    MessageProcessor que delega procesamiento a un Actor.
    
    Integra Actor + MessageLoop conectando el loop con el método receive()
    del actor.
    """
    
    def __init__(self, actor: Actor):
        """
        Inicializar processor con un actor.
        
        Args:
            actor: Actor que procesará los mensajes
        """
        self._actor = actor
    
    def process_message(self, message: Any) -> None:
        """
        Procesar mensaje delegando al actor.
        
        Args:
            message: Mensaje a procesar
        """
        # Verificar que actor esté en estado válido
        if self._actor._state != ActorState.RUNNING:
            raise RuntimeError(f"Actor not running: {self._actor._state}")
        
        # Delegar al método receive() del actor
        self._actor.receive(message)
        
        # Incrementar contador de mensajes del actor
        self._actor._message_count += 1
    
    def handle_error(self, error: Exception, message: Any) -> None:
        """
        Manejar error delegando al actor.
        
        Args:
            error: Excepción que ocurrió
            message: Mensaje que causó el error
        """
        # Incrementar contador de errores del actor
        self._actor._error_count += 1
        
        # Por ahora, solo re-lanzar (en TASK-041 agregaremos supervisión)
        raise error


class ActorWithMessageLoop(Actor):
    """
    Actor example con MessageLoop integrado.
    
    Demuestra cómo usar MessageLoop con un Actor para procesar
    mensajes de forma asíncrona desde un mailbox.
    """
    
    def __init__(
        self,
        name: str,
        mailbox: Optional[Mailbox] = None,
        max_throughput: Optional[int] = None
    ):
        """
        Inicializar actor con message loop.
        
        Args:
            name: Nombre del actor
            mailbox: Mailbox a usar (default: UnboundedMailbox)
            max_throughput: Máximo de mensajes por ciclo
        """
        self.name = name
        self._state = ActorState.UNINITIALIZED
        self._message_count = 0
        self._error_count = 0
        
        # Mailbox
        self._mailbox = mailbox if mailbox else UnboundedMailbox()
        
        # Message loop
        processor = ActorMessageProcessor(self)
        self._message_loop = MessageLoop(
            mailbox=self._mailbox,
            processor=processor,
            max_throughput=max_throughput
        )
        
        # Almacenar mensajes procesados (para testing)
        self._processed_messages = []
    
    def receive(self, message: Any) -> None:
        """
        Manejar mensaje recibido.
        
        Args:
            message: Mensaje a procesar
        """
        # Almacenar para testing
        self._processed_messages.append(message)
        
        # Log simple
        print(f"[{self.name}] Received: {message}")
    
    def start(self) -> None:
        """Iniciar actor (lifecycle hook + message loop)."""
        # Lifecycle hook
        self.pre_start()
        
        # Cambiar estado
        self._state = ActorState.STARTING
        self._state = ActorState.RUNNING
        
        # Iniciar message loop
        self._message_loop.start()
    
    def stop(self, timeout: Optional[float] = None) -> None:
        """Detener actor (message loop + lifecycle hook)."""
        # Cambiar estado
        self._state = ActorState.STOPPING
        
        # Detener message loop
        self._message_loop.stop(timeout=timeout)
        
        # Lifecycle hook
        self.post_stop()
        
        # Estado final
        self._state = ActorState.STOPPED
    
    def send(self, message: Any) -> bool:
        """
        Enviar mensaje al actor (enqueue en mailbox).
        
        Args:
            message: Mensaje a enviar
        
        Returns:
            True si el mensaje fue aceptado, False si rechazado
        """
        return self._mailbox.enqueue(message)
    
    def get_processed_messages(self) -> list:
        """Obtener lista de mensajes procesados (para testing)."""
        return self._processed_messages.copy()
    
    def get_message_loop_metrics(self) -> dict:
        """Obtener métricas del message loop."""
        return {
            "state": self._message_loop.get_state().value,
            "messages_processed": self._message_loop.get_messages_processed(),
            "errors_count": self._message_loop.get_errors_count(),
            "cycles_count": self._message_loop.get_cycles_count(),
            "avg_processing_time": self._message_loop.get_average_processing_time()
        }


# Example: Counter Actor con MessageLoop
class CounterActorWithLoop(ActorWithMessageLoop):
    """Counter actor con message loop para testing."""
    
    def __init__(self, name: str = "counter"):
        super().__init__(name)
        self.count = 0
    
    def receive(self, message: Any) -> None:
        """Handle counter messages."""
        # Almacenar mensaje
        self._processed_messages.append(message)
        
        if message == "increment":
            self.count += 1
        elif message == "decrement":
            self.count -= 1
        elif message == "reset":
            self.count = 0
        elif isinstance(message, dict) and message.get("type") == "add":
            self.count += message.get("value", 0)


if __name__ == "__main__":
    """Demo de MessageLoop con Actor."""
    print("=== Message Loop Demo ===\n")
    
    # Crear actor con message loop
    actor = CounterActorWithLoop(name="MyCounter")
    
    # Iniciar actor (esto inicia el message loop)
    actor.start()
    print(f"Actor started: {actor.name}")
    print(f"State: {actor._state.value}\n")
    
    # Enviar mensajes
    print("Sending messages...")
    actor.send("increment")
    actor.send("increment")
    actor.send("increment")
    actor.send({"type": "add", "value": 5})
    actor.send("decrement")
    
    # Esperar que se procesen
    time.sleep(0.1)
    
    # Ver resultados
    print(f"\nCounter value: {actor.count}")
    print(f"Messages processed: {actor.get_message_loop_metrics()['messages_processed']}")
    print(f"Processed messages: {actor.get_processed_messages()}")
    
    # Detener actor
    print("\nStopping actor...")
    actor.stop(timeout=1.0)
    print(f"Actor stopped. Final state: {actor._state.value}")
    
    # Métricas finales
    metrics = actor.get_message_loop_metrics()
    print(f"\n=== Metrics ===")
    print(f"Messages processed: {metrics['messages_processed']}")
    print(f"Errors: {metrics['errors_count']}")
    print(f"Cycles: {metrics['cycles_count']}")
    print(f"Avg processing time: {metrics['avg_processing_time']:.6f}s")
