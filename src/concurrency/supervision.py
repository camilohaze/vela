"""
Supervision System for Actor Model

Implementación de: VELA-579 (TASK-042)
Historia: Sprint 17 - Supervision Hierarchy
Fecha: 2025-12-02

Este módulo implementa el sistema de supervisión para actors,
inspirado en Erlang/OTP y Akka. Provee tolerancia a fallos mediante
estrategias de reinicio automático y escalación de errores.

Características:
- 3 estrategias de supervisión (OneForOne, OneForAll, RestForOne)
- Políticas de reinicio configurables con backoff
- Escalación jerárquica de errores
- Directivas de supervisión (Resume, Restart, Stop, Escalate)

Referencias:
- ADR-010: Supervision Hierarchy
- Erlang/OTP Supervisor: http://erlang.org/doc/design_principles/sup_princ.html
- Akka Supervision: https://doc.akka.io/docs/akka/current/typed/fault-tolerance.html
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional, Callable, Any
import time
import logging
import threading
from datetime import datetime, timedelta

# Imports del proyecto
from src.concurrency.actor import Actor, ActorRef, ActorState


# ============================================================================
# ENUMS Y TIPOS
# ============================================================================

class SupervisorDirective(Enum):
    """
    Decisiones que puede tomar un supervisor ante el fallo de un child.
    
    Inspirado en Akka Supervisor Directives.
    """
    
    RESUME = "resume"      # Ignorar error y continuar procesamiento
    RESTART = "restart"    # Reiniciar el actor (comportamiento por defecto)
    STOP = "stop"          # Detener el actor permanentemente
    ESCALATE = "escalate"  # Escalar error al supervisor padre


class BackoffStrategy(Enum):
    """
    Estrategias de backoff para espaciar reintentos.
    """
    
    CONSTANT = "constant"        # Delay fijo
    LINEAR = "linear"            # Incremento lineal
    EXPONENTIAL = "exponential"  # Incremento exponencial


@dataclass
class RestartStats:
    """
    Estadísticas de reinicio para un actor child.
    
    Usado para rastrear historial de fallos y aplicar restart policy.
    """
    
    actor_ref: ActorRef
    failure_count: int = 0
    last_failure_time: Optional[float] = None
    restart_times: List[float] = field(default_factory=list)
    total_restarts: int = 0
    
    def record_failure(self) -> None:
        """Registra un nuevo fallo."""
        self.failure_count += 1
        self.last_failure_time = time.time()
    
    def record_restart(self) -> None:
        """Registra un reinicio exitoso."""
        self.restart_times.append(time.time())
        self.total_restarts += 1
    
    def reset_failure_count(self) -> None:
        """Resetea el contador de fallos (después de ventana de tiempo)."""
        self.failure_count = 0
    
    def get_failures_in_window(self, window_seconds: float) -> int:
        """
        Cuenta fallos dentro de una ventana de tiempo.
        
        Args:
            window_seconds: Tamaño de la ventana en segundos
            
        Returns:
            Número de fallos en la ventana
        """
        if not self.restart_times:
            return 0
        
        cutoff_time = time.time() - window_seconds
        recent_restarts = [t for t in self.restart_times if t > cutoff_time]
        return len(recent_restarts)


@dataclass
class RestartPolicy:
    """
    Política de reinicio de actors.
    
    Define cuándo y cómo reiniciar actors fallidos.
    
    Attributes:
        max_retries: Máximo número de reinicios permitidos
        within_time_window: Ventana de tiempo para contar reinicios (segundos)
        backoff_strategy: Estrategia para espaciar reintentos
        initial_delay: Delay inicial antes del primer reinicio (segundos)
        max_delay: Delay máximo entre reintentos (segundos)
    """
    
    max_retries: int = 3
    within_time_window: float = 60.0
    backoff_strategy: BackoffStrategy = BackoffStrategy.EXPONENTIAL
    initial_delay: float = 1.0
    max_delay: float = 30.0
    
    def calculate_delay(self, retry_count: int) -> float:
        """
        Calcula el delay antes del próximo reinicio.
        
        Args:
            retry_count: Número de reintentos realizados
            
        Returns:
            Delay en segundos
        """
        if self.backoff_strategy == BackoffStrategy.CONSTANT:
            return min(self.initial_delay, self.max_delay)
        
        elif self.backoff_strategy == BackoffStrategy.LINEAR:
            delay = self.initial_delay * (retry_count + 1)
            return min(delay, self.max_delay)
        
        elif self.backoff_strategy == BackoffStrategy.EXPONENTIAL:
            delay = self.initial_delay * (2 ** retry_count)
            return min(delay, self.max_delay)
        
        else:
            return self.initial_delay
    
    def should_restart(self, stats: RestartStats) -> bool:
        """
        Determina si se debe reiniciar un actor basado en su historial.
        
        Args:
            stats: Estadísticas de reinicio del actor
            
        Returns:
            True si se debe reiniciar, False si se excedió el límite
        """
        failures_in_window = stats.get_failures_in_window(self.within_time_window)
        return failures_in_window < self.max_retries


# ============================================================================
# SUPERVISION STRATEGY (INTERFACE)
# ============================================================================

class SupervisionStrategy(ABC):
    """
    Estrategia de supervisión (interface).
    
    Define cómo un supervisor maneja fallos de sus children.
    """
    
    def __init__(self, restart_policy: Optional[RestartPolicy] = None):
        """
        Inicializa la estrategia.
        
        Args:
            restart_policy: Política de reinicio (None = default)
        """
        self.restart_policy = restart_policy or RestartPolicy()
        self.logger = logging.getLogger(self.__class__.__name__)
    
    @abstractmethod
    def handle_failure(
        self,
        supervisor: 'SupervisorActor',
        failed_child: ActorRef,
        error: Exception
    ) -> None:
        """
        Maneja el fallo de un child actor.
        
        Args:
            supervisor: Supervisor que maneja el fallo
            failed_child: Referencia al child que falló
            error: Excepción que causó el fallo
        """
        pass
    
    def decide_directive(
        self,
        supervisor: 'SupervisorActor',
        failed_child: ActorRef,
        error: Exception
    ) -> SupervisorDirective:
        """
        Decide qué directiva aplicar ante un fallo.
        
        Puede ser sobrescrito para custom logic.
        
        Args:
            supervisor: Supervisor tomando la decisión
            failed_child: Child que falló
            error: Error ocurrido
            
        Returns:
            Directiva a aplicar
        """
        # Default: siempre reiniciar si no se excedió max_retries
        stats = supervisor.get_restart_stats(failed_child)
        
        if self.restart_policy.should_restart(stats):
            return SupervisorDirective.RESTART
        else:
            # Excedió límite → escalar
            self.logger.warning(
                f"Child {failed_child.name} exceeded restart limit. Escalating."
            )
            return SupervisorDirective.ESCALATE


# ============================================================================
# ONE-FOR-ONE STRATEGY
# ============================================================================

class OneForOneStrategy(SupervisionStrategy):
    """
    OneForOne Supervision Strategy.
    
    Cuando un child falla, solo ese child se reinicia.
    Los otros children no se ven afectados.
    
    Uso recomendado:
        - Children son independientes entre sí
        - El fallo de uno no afecta a los demás
        - Ejemplo: Workers independientes en un pool
    """
    
    def handle_failure(
        self,
        supervisor: 'SupervisorActor',
        failed_child: ActorRef,
        error: Exception
    ) -> None:
        """
        Maneja fallo reiniciando solo el child que falló.
        
        Args:
            supervisor: Supervisor que maneja el fallo
            failed_child: Child que falló
            error: Error que causó el fallo
        """
        self.logger.info(
            f"OneForOne: Child {failed_child.name} failed with {error}. "
            f"Restarting only this child."
        )
        
        # Obtener directiva
        directive = self.decide_directive(supervisor, failed_child, error)
        
        # Aplicar directiva
        if directive == SupervisorDirective.RESTART:
            supervisor.restart_child(failed_child)
        
        elif directive == SupervisorDirective.STOP:
            supervisor.stop_child(failed_child)
        
        elif directive == SupervisorDirective.ESCALATE:
            supervisor.escalate_failure(failed_child, error)
        
        elif directive == SupervisorDirective.RESUME:
            # No hacer nada, continuar
            self.logger.info(f"Resuming child {failed_child.name}")


# ============================================================================
# ONE-FOR-ALL STRATEGY
# ============================================================================

class OneForAllStrategy(SupervisionStrategy):
    """
    OneForAll Supervision Strategy.
    
    Cuando un child falla, TODOS los children se reinician.
    
    Uso recomendado:
        - Children comparten estado
        - El fallo de uno invalida el estado de todos
        - Ejemplo: Cluster de réplicas con estado compartido
    """
    
    def handle_failure(
        self,
        supervisor: 'SupervisorActor',
        failed_child: ActorRef,
        error: Exception
    ) -> None:
        """
        Maneja fallo reiniciando TODOS los children.
        
        Args:
            supervisor: Supervisor que maneja el fallo
            failed_child: Child que falló (trigger del restart)
            error: Error que causó el fallo
        """
        self.logger.info(
            f"OneForAll: Child {failed_child.name} failed with {error}. "
            f"Restarting ALL children."
        )
        
        # Obtener directiva
        directive = self.decide_directive(supervisor, failed_child, error)
        
        # Aplicar directiva
        if directive == SupervisorDirective.RESTART:
            # Reiniciar TODOS los children
            for child_ref in supervisor.get_all_children():
                supervisor.restart_child(child_ref)
        
        elif directive == SupervisorDirective.STOP:
            # Detener TODOS los children
            for child_ref in supervisor.get_all_children():
                supervisor.stop_child(child_ref)
        
        elif directive == SupervisorDirective.ESCALATE:
            supervisor.escalate_failure(failed_child, error)
        
        elif directive == SupervisorDirective.RESUME:
            # No hacer nada
            self.logger.info("Resuming all children")


# ============================================================================
# REST-FOR-ONE STRATEGY
# ============================================================================

class RestForOneStrategy(SupervisionStrategy):
    """
    RestForOne Supervision Strategy.
    
    Cuando un child falla:
    1. Se reinicia el child que falló
    2. Se reinician todos los children que fueron creados DESPUÉS de él
    
    Asume que los children tienen dependencia temporal (orden de spawn).
    
    Uso recomendado:
        - Children forman un pipeline
        - Cada child depende de los anteriores
        - Ejemplo: Producer → Processor → Consumer (si Processor falla, Consumer debe reiniciarse)
    """
    
    def handle_failure(
        self,
        supervisor: 'SupervisorActor',
        failed_child: ActorRef,
        error: Exception
    ) -> None:
        """
        Maneja fallo reiniciando el child y los posteriores.
        
        Args:
            supervisor: Supervisor que maneja el fallo
            failed_child: Child que falló
            error: Error que causó el fallo
        """
        self.logger.info(
            f"RestForOne: Child {failed_child.name} failed with {error}. "
            f"Restarting failed child and all subsequent children."
        )
        
        # Obtener directiva
        directive = self.decide_directive(supervisor, failed_child, error)
        
        # Aplicar directiva
        if directive == SupervisorDirective.RESTART:
            # Reiniciar el fallido y los posteriores
            children_to_restart = supervisor.get_children_after(failed_child)
            
            # Incluir el child que falló
            children_to_restart.insert(0, failed_child)
            
            for child_ref in children_to_restart:
                supervisor.restart_child(child_ref)
        
        elif directive == SupervisorDirective.STOP:
            # Detener el fallido y los posteriores
            children_to_stop = supervisor.get_children_after(failed_child)
            children_to_stop.insert(0, failed_child)
            
            for child_ref in children_to_stop:
                supervisor.stop_child(child_ref)
        
        elif directive == SupervisorDirective.ESCALATE:
            supervisor.escalate_failure(failed_child, error)
        
        elif directive == SupervisorDirective.RESUME:
            self.logger.info(f"Resuming child {failed_child.name} and subsequent")


# ============================================================================
# SUPERVISOR ACTOR (BASE CLASS)
# ============================================================================

class SupervisorActor(Actor):
    """
    Actor que supervisa otros actors (children).
    
    Proporciona tolerancia a fallos mediante:
    - Monitoreo de children
    - Reinicio automático según estrategia
    - Escalación de errores no recuperables
    
    Attributes:
        strategy: Estrategia de supervisión (OneForOne, OneForAll, RestForOne)
        children: Dict de children supervisados {name: ActorRef}
        restart_stats: Estadísticas de reinicio por child
        parent_supervisor: Supervisor padre (para escalación)
    """
    
    def __init__(
        self,
        strategy: Optional[SupervisionStrategy] = None,
        parent_supervisor: Optional['SupervisorActor'] = None
    ):
        """
        Inicializa el supervisor.
        
        Args:
            strategy: Estrategia de supervisión (default: OneForOne)
            parent_supervisor: Supervisor padre para escalación
        """
        super().__init__()
        
        self.strategy = strategy or OneForOneStrategy()
        self.parent_supervisor = parent_supervisor
        
        # Estado de supervisión
        self.children: Dict[str, ActorRef] = {}
        self.children_order: List[str] = []  # Para RestForOne
        self.restart_stats: Dict[str, RestartStats] = {}
        
        # Pending restarts (threading.Timer instances - TASK-043)
        self._pending_restarts: Dict[str, threading.Timer] = {}  # child_name -> Timer
        
        # Logger (sin nombre hasta que se asigne ref)
        self.logger = logging.getLogger(f"SupervisorActor")
    
    def cancel_pending_restarts(self, child_ref: ActorRef) -> int:
        """
        Cancela restart pendiente de un child (TASK-043).
        
        Útil cuando:
        - Se detiene un child antes de que se reinicie
        - Se quiere cancelar restart manual
        
        Args:
            child_ref: Child cuyo restart se quiere cancelar
            
        Returns:
            Número de restarts cancelados (0 o 1)
        """
        child_name = self._get_child_name(child_ref)
        
        if not child_name or child_name not in self._pending_restarts:
            return 0
        
        # Cancelar threading.Timer
        timer = self._pending_restarts[child_name]
        timer.cancel()
        del self._pending_restarts[child_name]
        self.logger.info(f"Cancelled pending restart for child '{child_name}'")
        return 1
    
    # ------------------------------------------------------------------------
    # CHILD MANAGEMENT
    # ------------------------------------------------------------------------
    
    def spawn_child(
        self,
        actor_class: type,
        name: str,
        *args,
        **kwargs
    ) -> ActorRef:
        """
        Crea un child actor supervisado.
        
        Args:
            actor_class: Clase del actor a crear
            name: Nombre único del child
            *args: Argumentos posicionales para el constructor
            **kwargs: Argumentos nombrados para el constructor
            
        Returns:
            Referencia al child creado
            
        Raises:
            ValueError: Si ya existe un child con ese nombre
        """
        if name in self.children:
            raise ValueError(f"Child with name '{name}' already exists")
        
        # Crear child actor
        child_actor = actor_class(*args, **kwargs)
        
        # Crear ActorRef para el child (TASK-044: con supervisor)
        child_ref = ActorRef(name=name, actor=child_actor, supervisor=self)
        child_actor.state = ActorState.RUNNING
        
        # Registrar como child
        self.children[name] = child_ref
        self.children_order.append(name)
        
        # Inicializar estadísticas
        self.restart_stats[name] = RestartStats(actor_ref=child_ref)
        
        self.logger.info(f"Spawned child '{name}' of type {actor_class.__name__}")
        
        return child_ref
    
    def stop_child(self, child_ref: ActorRef) -> None:
        """
        Detiene un child actor.
        
        TASK-043: Cancela restart pendiente si existe.
        
        Args:
            child_ref: Referencia al child a detener
        """
        child_name = self._get_child_name(child_ref)
        
        if child_name:
            # TASK-043: Cancelar restart pendiente antes de detener
            self.cancel_pending_restarts(child_ref)
            
            # Detener el actor
            # (En implementación real, enviaría mensaje PoisonPill al actor)
            child_ref.actor.state = ActorState.STOPPED
            
            # Remover de tracking
            del self.children[child_name]
            self.children_order.remove(child_name)
            del self.restart_stats[child_name]
            
            self.logger.info(f"Stopped child '{child_name}'")
    
    def get_all_children(self) -> List[ActorRef]:
        """
        Obtiene lista de todos los children.
        
        Returns:
            Lista de referencias a children
        """
        return list(self.children.values())
    
    def get_children_after(self, failed_child: ActorRef) -> List[ActorRef]:
        """
        Obtiene children que fueron creados después del child especificado.
        
        Usado por RestForOne strategy.
        
        Args:
            failed_child: Child de referencia
            
        Returns:
            Lista de children posteriores (en orden de creación)
        """
        child_name = self._get_child_name(failed_child)
        
        if not child_name:
            return []
        
        # Encontrar índice del child
        try:
            child_index = self.children_order.index(child_name)
        except ValueError:
            return []
        
        # Retornar children posteriores
        subsequent_names = self.children_order[child_index + 1:]
        return [self.children[name] for name in subsequent_names]
    
    def get_restart_stats(self, child_ref: ActorRef) -> Optional[RestartStats]:
        """
        Obtiene estadísticas de reinicio de un child.
        
        Args:
            child_ref: Child del cual obtener estadísticas
            
        Returns:
            RestartStats o None si no existe
        """
        child_name = self._get_child_name(child_ref)
        return self.restart_stats.get(child_name) if child_name else None
    
    def _get_child_name(self, child_ref: ActorRef) -> Optional[str]:
        """
        Obtiene el nombre de un child por su referencia.
        
        Args:
            child_ref: Referencia del child
            
        Returns:
            Nombre del child o None
        """
        for name, ref in self.children.items():
            if ref == child_ref:
                return name
        return None
    
    # ------------------------------------------------------------------------
    # FAILURE HANDLING
    # ------------------------------------------------------------------------
    
    def handle_child_failure(self, child_ref: ActorRef, error: Exception) -> None:
        """
        Maneja el fallo de un child actor.
        
        Delega a la estrategia de supervisión configurada.
        
        Args:
            child_ref: Child que falló
            error: Excepción que causó el fallo
        """
        child_name = self._get_child_name(child_ref)
        
        if not child_name:
            self.logger.warning(f"Received failure for unknown child: {child_ref.name}")
            return
        
        self.logger.error(
            f"Child '{child_name}' failed with error: {error.__class__.__name__}: {error}"
        )
        
        # Registrar fallo en estadísticas
        stats = self.restart_stats[child_name]
        stats.record_failure()
        
        # Delegar a strategy
        self.strategy.handle_failure(self, child_ref, error)
    
    def restart_child(self, child_ref: ActorRef) -> None:
        """
        Reinicia un child actor.
        
        TASK-043: Restart asíncrono con threading.Timer
        - No bloquea el supervisor
        - Usa backoff strategy
        - Permite cancelar restarts pendientes
        
        Args:
            child_ref: Child a reiniciar
        """
        child_name = self._get_child_name(child_ref)
        
        if not child_name:
            self.logger.warning(f"Cannot restart unknown child: {child_ref.name}")
            return
        
        stats = self.restart_stats[child_name]
        
        # Calcular delay con backoff
        delay = self.strategy.restart_policy.calculate_delay(stats.failure_count)
        
        self.logger.info(
            f"Restarting child '{child_name}' after {delay:.2f}s delay "
            f"(attempt {stats.failure_count}/{self.strategy.restart_policy.max_retries})"
        )
        
        # Función de restart
        def do_restart():
            """Ejecuta el restart real."""
            # Hooks de reinicio
            child_ref.actor.pre_restart(error=None)  # Cleanup
            child_ref.actor.state = ActorState.RUNNING
            child_ref.actor.post_restart(error=None)  # Re-init
            
            # Registrar reinicio exitoso
            stats.record_restart()
            
            # Remover de pending restarts
            if child_name in self._pending_restarts:
                del self._pending_restarts[child_name]
            
            self.logger.info(f"Child '{child_name}' restarted successfully")
        
        # TASK-043: Restart asíncrono con threading.Timer (no bloquea)
        # Cancelar restart pendiente si existe
        if child_name in self._pending_restarts:
            old_timer = self._pending_restarts[child_name]
            old_timer.cancel()
        
        # Schedule restart asíncrono
        timer = threading.Timer(delay, do_restart)
        timer.daemon = True  # Daemon thread para que no bloquee shutdown
        timer.start()
        self._pending_restarts[child_name] = timer
    
    def escalate_failure(self, child_ref: ActorRef, error: Exception) -> None:
        """
        Escala un fallo al supervisor padre.
        
        Args:
            child_ref: Child cuyo fallo se escala
            error: Error a escalar
        """
        child_name = self._get_child_name(child_ref)
        
        self.logger.warning(
            f"Escalating failure of child '{child_name}' to parent supervisor"
        )
        
        if self.parent_supervisor:
            # Escalar al padre
            self.parent_supervisor.handle_child_failure(self.ref, error)
        else:
            # No hay padre → log y detener
            self.logger.error(
                f"No parent supervisor to escalate to. Stopping child '{child_name}'."
            )
            self.stop_child(child_ref)
    
    # ------------------------------------------------------------------------
    # ACTOR MESSAGE HANDLING
    # ------------------------------------------------------------------------
    
    def receive(self, message: Any) -> None:
        """
        Message handler del supervisor.
        
        Por defecto, los supervisors no procesan mensajes de negocio.
        Subclases pueden sobrescribir para custom logic.
        
        Args:
            message: Mensaje recibido
        """
        self.logger.debug(f"Supervisor received message: {message}")
        # Default: no action
    
    # ------------------------------------------------------------------------
    # ACTOR LIFECYCLE HOOKS (OVERRIDE)
    # ------------------------------------------------------------------------
    
    def pre_restart(self, error: Optional[Exception] = None) -> None:
        """
        Hook antes de reiniciar el supervisor.
        
        Detiene todos los children antes de reiniciar.
        """
        self.logger.info("Pre-restart: stopping all children")
        
        for child_ref in list(self.children.values()):
            self.stop_child(child_ref)
        
        super().pre_restart(error)
    
    def post_restart(self, error: Optional[Exception] = None) -> None:
        """
        Hook después de reiniciar el supervisor.
        
        Subclases deben sobrescribir para re-crear children.
        """
        self.logger.info("Post-restart: supervisor restarted")
        super().post_restart(error)


# ============================================================================
# EXPORTS
# ============================================================================

__all__ = [
    # Enums
    'SupervisorDirective',
    'BackoffStrategy',
    
    # Data classes
    'RestartStats',
    'RestartPolicy',
    
    # Strategies
    'SupervisionStrategy',
    'OneForOneStrategy',
    'OneForAllStrategy',
    'RestForOneStrategy',
    
    # Supervisor
    'SupervisorActor',
]
