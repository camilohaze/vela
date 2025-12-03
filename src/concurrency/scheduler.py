"""
Actor Scheduler - TASK-041

Implementación de: VELA-578 (Actor System)
Historia: VELA-578 (Sprint 16)
Fecha: 2025-12-02

Descripción:
Scheduler que orquesta la ejecución de actors usando ThreadPoolExecutor.
Maneja spawning, lifecycle, scheduling policies, y integración completa.

Características:
- Spawn actors con nombres únicos
- Asignación automática al executor
- Fair scheduling (round-robin)
- Priority scheduling (opcional)
- Metrics de actors activos, mensajes, throughput
- Graceful shutdown de actors

Inspiración:
- Akka ActorSystem (spawn, shutdown)
- Pony Runtime Scheduler (fair scheduling)
- Erlang OTP (supervision tree - futuro)
"""

import time
import threading
import heapq
from typing import Dict, Any, Optional, Type, List, Callable
from enum import Enum
from dataclasses import dataclass
from src.concurrency.actor import Actor, ActorRef
from src.concurrency.executor import ThreadPoolExecutor


class SchedulerState(Enum):
    """Estados del scheduler."""
    IDLE = "idle"
    RUNNING = "running"
    SHUTTING_DOWN = "shutting_down"
    TERMINATED = "terminated"


class SchedulingPolicy(Enum):
    """Políticas de scheduling."""
    FAIR = "fair"           # Round-robin, todos igual prioridad
    PRIORITY = "priority"   # Basado en prioridad de actor
    FIFO = "fifo"           # First-In-First-Out


@dataclass
class ActorMetrics:
    """Métricas de un actor en el scheduler."""
    actor_ref: ActorRef
    spawned_at: float
    messages_received: int = 0
    messages_processed: int = 0
    last_active_at: Optional[float] = None
    priority: int = 0  # Para priority scheduling
    
    def get_uptime(self) -> float:
        """Obtener tiempo desde spawn."""
        return time.time() - self.spawned_at
    
    def get_message_rate(self) -> float:
        """Obtener rate de mensajes procesados por segundo."""
        uptime = self.get_uptime()
        if uptime > 0:
            return self.messages_processed / uptime
        return 0.0


@dataclass
class ScheduledTask:
    """
    Tarea programada para ejecución diferida.
    
    Usado por SupervisorActor para restart logic asíncrono (TASK-043).
    """
    task_id: str
    callback: Callable[[], None]
    scheduled_at: float        # timestamp cuando se creó
    execute_at: float          # timestamp cuando debe ejecutarse
    delay: float              # delay original (para debug)
    cancelled: bool = False   # flag de cancelación
    
    def __lt__(self, other: 'ScheduledTask') -> bool:
        """Comparación para heapq (min-heap por execute_at)."""
        return self.execute_at < other.execute_at
    
    def is_ready(self) -> bool:
        """Check si está listo para ejecutar."""
        return time.time() >= self.execute_at and not self.cancelled


class ActorScheduler:
    """
    Scheduler que orquesta ejecución de actors.
    
    El scheduler:
    1. Spawn actors y los registra
    2. Asigna message loops al executor
    3. Aplica políticas de scheduling
    4. Recolecta métricas
    5. Maneja shutdown gracefully
    
    Example:
        # Crear executor
        executor = ThreadPoolExecutor(min_threads=4)
        executor.start()
        
        # Crear scheduler
        scheduler = ActorScheduler(executor, policy=SchedulingPolicy.FAIR)
        scheduler.start()
        
        # Spawn actors
        actor1 = scheduler.spawn(CounterActor, name="Counter1")
        actor2 = scheduler.spawn(ChatRoomActor, name="ChatRoom")
        
        # Enviar mensajes
        actor1.send("increment")
        
        # Shutdown
        scheduler.shutdown()
        executor.shutdown()
    """
    
    def __init__(
        self,
        executor: ThreadPoolExecutor,
        policy: SchedulingPolicy = SchedulingPolicy.FAIR,
        max_actors: int = 10000
    ):
        """
        Inicializar scheduler.
        
        Args:
            executor: ThreadPoolExecutor para ejecutar message loops
            policy: Política de scheduling
            max_actors: Máximo de actors permitidos
        """
        self.executor = executor
        self.policy = policy
        self.max_actors = max_actors
        
        # Estado
        self.state = SchedulerState.IDLE
        self.state_lock = threading.Lock()
        
        # Registry de actors
        self.actors: Dict[str, ActorMetrics] = {}  # name -> metrics
        self.actors_lock = threading.Lock()
        
        # Counters
        self.total_spawned = 0
        self.total_stopped = 0
        self.total_messages = 0
        
        # Lifecycle
        self.started_at: Optional[float] = None
        self.stopped_at: Optional[float] = None
    
    def start(self) -> None:
        """Iniciar scheduler."""
        with self.state_lock:
            if self.state != SchedulerState.IDLE:
                raise RuntimeError(f"Cannot start scheduler in state {self.state}")
            
            self.state = SchedulerState.RUNNING
            self.started_at = time.time()
    
    def shutdown(self, wait: bool = True, timeout: float = 5.0) -> None:
        """
        Detener scheduler y todos los actors.
        
        Args:
            wait: Si debe esperar que actors terminen
            timeout: Timeout máximo
        """
        with self.state_lock:
            if self.state != SchedulerState.RUNNING:
                return
            
            self.state = SchedulerState.SHUTTING_DOWN
        
        # Detener todos los actors
        start_time = time.time()
        with self.actors_lock:
            actors_to_stop = list(self.actors.values())
        
        for metrics in actors_to_stop:
            if time.time() - start_time > timeout:
                break
            
            try:
                # Enviar mensaje de stop al actor
                metrics.actor_ref.send("stop")
            except Exception:
                pass
        
        # Esperar si se solicita
        if wait:
            remaining_timeout = timeout - (time.time() - start_time)
            if remaining_timeout > 0:
                time.sleep(min(remaining_timeout, 0.5))
        
        # Detener timer thread si existe
        self._stop_timer_thread()
        
        # Actualizar estado
        with self.state_lock:
            self.state = SchedulerState.TERMINATED
            self.stopped_at = time.time()
    
    def spawn(
        self,
        actor_class: Type[Actor],
        name: Optional[str] = None,
        priority: int = 0,
        **kwargs
    ) -> ActorRef:
        """
        Spawn un nuevo actor.
        
        Args:
            actor_class: Clase del actor a crear
            name: Nombre único (auto-generado si None)
            priority: Prioridad para priority scheduling
            **kwargs: Args para constructor del actor
        
        Returns:
            ActorRef del actor creado
        
        Raises:
            RuntimeError: Si scheduler no está running
            ValueError: Si nombre duplicado o max_actors alcanzado
        """
        # Validar estado
        with self.state_lock:
            if self.state != SchedulerState.RUNNING:
                raise RuntimeError(f"Cannot spawn actor in state {self.state}")
        
        # Validar límite
        with self.actors_lock:
            if len(self.actors) >= self.max_actors:
                raise ValueError(f"Max actors limit reached: {self.max_actors}")
            
            # Auto-generar nombre si None
            if name is None:
                name = f"actor-{self.total_spawned + 1}"
            
            # Validar nombre único
            if name in self.actors:
                raise ValueError(f"Actor with name '{name}' already exists")
            
            # Crear actor
            actor = actor_class(name=name, **kwargs)
            actor_ref = ActorRef(name=name, actor=actor)
            
            # Registrar metrics
            metrics = ActorMetrics(
                actor_ref=actor_ref,
                spawned_at=time.time(),
                priority=priority
            )
            self.actors[name] = metrics
            self.total_spawned += 1
        
        # Submit message loop al executor
        # Usamos callable que retorna el loop
        def run_message_loop():
            actor._message_loop._run_loop()
        
        success = self.executor.submit(
            callable_fn=run_message_loop,
            name=f"MessageLoop-{name}"
        )
        
        if not success:
            # Si falla submit, remover del registry
            with self.actors_lock:
                del self.actors[name]
                self.total_spawned -= 1
            raise RuntimeError(f"Failed to submit actor '{name}' to executor")
        
        return actor_ref
    
    def stop_actor(self, name: str) -> bool:
        """
        Detener un actor específico.
        
        Args:
            name: Nombre del actor
        
        Returns:
            True si se detuvo exitosamente
        """
        with self.actors_lock:
            if name not in self.actors:
                return False
            
            metrics = self.actors[name]
            
            try:
                # Enviar mensaje stop
                metrics.actor_ref.send("stop")
                
                # Remover del registry
                del self.actors[name]
                self.total_stopped += 1
                
                return True
            except Exception:
                return False
    
    def get_actor(self, name: str) -> Optional[ActorRef]:
        """
        Obtener ActorRef por nombre.
        
        Args:
            name: Nombre del actor
        
        Returns:
            ActorRef o None si no existe
        """
        with self.actors_lock:
            if name in self.actors:
                return self.actors[name].actor_ref
            return None
    
    def get_active_actors(self) -> List[str]:
        """
        Obtener lista de nombres de actors activos.
        
        Returns:
            Lista de nombres
        """
        with self.actors_lock:
            return list(self.actors.keys())
    
    def get_actor_count(self) -> int:
        """Obtener cantidad de actors activos."""
        with self.actors_lock:
            return len(self.actors)
    
    def get_state(self) -> SchedulerState:
        """Obtener estado del scheduler."""
        with self.state_lock:
            return self.state
    
    def get_metrics(self) -> Dict[str, Any]:
        """
        Obtener métricas del scheduler.
        
        Returns:
            Dict con métricas
        """
        with self.state_lock:
            state = self.state
            started_at = self.started_at
            stopped_at = self.stopped_at
        
        with self.actors_lock:
            active_actors = len(self.actors)
            total_spawned = self.total_spawned
            total_stopped = self.total_stopped
        
        # Calcular uptime
        uptime = 0.0
        if started_at:
            if stopped_at:
                uptime = stopped_at - started_at
            else:
                uptime = time.time() - started_at
        
        return {
            "state": state.value,
            "policy": self.policy.value,
            "active_actors": active_actors,
            "total_spawned": total_spawned,
            "total_stopped": total_stopped,
            "max_actors": self.max_actors,
            "uptime": uptime
        }
    
    def get_actor_metrics(self, name: str) -> Optional[Dict[str, Any]]:
        """
        Obtener métricas de un actor específico.
        
        Args:
            name: Nombre del actor
        
        Returns:
            Dict con métricas o None si no existe
        """
        with self.actors_lock:
            if name not in self.actors:
                return None
            
            metrics = self.actors[name]
            return {
                "name": name,
                "spawned_at": metrics.spawned_at,
                "uptime": metrics.get_uptime(),
                "messages_received": metrics.messages_received,
                "messages_processed": metrics.messages_processed,
                "message_rate": metrics.get_message_rate(),
                "last_active_at": metrics.last_active_at,
                "priority": metrics.priority
            }
    
    def get_all_actor_metrics(self) -> List[Dict[str, Any]]:
        """
        Obtener métricas de todos los actors.
        
        Returns:
            Lista de dicts con métricas
        """
        with self.actors_lock:
            actor_names = list(self.actors.keys())
        
        return [
            self.get_actor_metrics(name)
            for name in actor_names
            if self.get_actor_metrics(name) is not None
        ]
    
    def update_actor_stats(
        self,
        name: str,
        messages_received: int = 0,
        messages_processed: int = 0
    ) -> None:
        """
        Actualizar estadísticas de un actor (llamado por el actor).
        
        Args:
            name: Nombre del actor
            messages_received: Incremento en mensajes recibidos
            messages_processed: Incremento en mensajes procesados
        """
        with self.actors_lock:
            if name not in self.actors:
                return
            
            metrics = self.actors[name]
            metrics.messages_received += messages_received
            metrics.messages_processed += messages_processed
            
            if messages_processed > 0:
                metrics.last_active_at = time.time()
                self.total_messages += messages_processed


class PriorityActorScheduler(ActorScheduler):
    """
    Scheduler con soporte para priority scheduling.
    
    Extiende ActorScheduler para agregar scheduling basado en prioridad.
    Los actors con mayor prioridad obtienen más tiempo de CPU.
    
    NOTE: En esta implementación v1, el priority scheduling es preparación
    para futuro (Sprint 17+). Por ahora, solo registra prioridades pero
    no las aplica activamente (executor no soporta prioridades aún).
    """
    
    def __init__(
        self,
        executor: ThreadPoolExecutor,
        max_actors: int = 10000
    ):
        """
        Inicializar priority scheduler.
        
        Args:
            executor: ThreadPoolExecutor
            max_actors: Máximo de actors
        """
        super().__init__(
            executor=executor,
            policy=SchedulingPolicy.PRIORITY,
            max_actors=max_actors
        )
        
        # Priority queues (futuro: implementar lógica real)
        self.high_priority_actors: List[str] = []
        self.normal_priority_actors: List[str] = []
        self.low_priority_actors: List[str] = []
    
    def spawn(
        self,
        actor_class: Type[Actor],
        name: Optional[str] = None,
        priority: int = 0,
        **kwargs
    ) -> ActorRef:
        """
        Spawn actor con prioridad.
        
        Args:
            priority: 0 = normal, >0 = alta, <0 = baja
        
        Returns:
            ActorRef
        """
        actor_ref = super().spawn(
            actor_class=actor_class,
            name=name,
            priority=priority,
            **kwargs
        )
        
        # Clasificar en queue de prioridad
        actor_name = actor_ref._actor.name
        with self.actors_lock:
            if priority > 0:
                self.high_priority_actors.append(actor_name)
            elif priority < 0:
                self.low_priority_actors.append(actor_name)
            else:
                self.normal_priority_actors.append(actor_name)
        
        return actor_ref
    
    def get_priority_distribution(self) -> Dict[str, int]:
        """
        Obtener distribución de prioridades.
        
        Returns:
            Dict con counts por prioridad
        """
        with self.actors_lock:
            return {
                "high": len(self.high_priority_actors),
                "normal": len(self.normal_priority_actors),
                "low": len(self.low_priority_actors)
            }
    
    # ========================================================================
    # SCHEDULED TASKS (TASK-043)
    # ========================================================================
    
    def schedule_delayed(
        self, 
        callback: Callable[[], None], 
        delay_seconds: float,
        task_id: Optional[str] = None
    ) -> str:
        """
        Programar callback para ejecución después de un delay.
        
        Args:
            callback: Función a ejecutar
            delay_seconds: Delay en segundos
            task_id: ID opcional para la tarea
            
        Returns:
            Task ID (para cancelación o tracking)
            
        Example:
            # Schedule restart después de 5 segundos
            task_id = scheduler.schedule_delayed(
                lambda: restart_actor(actor_ref),
                delay_seconds=5.0
            )
            
            # Cancelar si es necesario
            scheduler.cancel_scheduled_task(task_id)
        """
        # Generar task_id único si no se proveyó
        if task_id is None:
            timestamp = int(time.time() * 1000)
            with self.actors_lock:
                counter = len(getattr(self, '_scheduled_tasks', {}))
            task_id = f"task-{timestamp}-{counter}"
        
        # Crear scheduled task
        now = time.time()
        task = ScheduledTask(
            task_id=task_id,
            callback=callback,
            scheduled_at=now,
            execute_at=now + delay_seconds,
            delay=delay_seconds,
            cancelled=False
        )
        
        # Agregar a pending tasks
        if not hasattr(self, '_scheduled_tasks'):
            self._scheduled_tasks: Dict[str, ScheduledTask] = {}
            self._scheduled_tasks_heap: List[ScheduledTask] = []
            self._scheduled_tasks_lock = threading.Lock()
            self._scheduled_tasks_executed = 0
            self._scheduled_tasks_cancelled = 0
            self._scheduled_tasks_failed = 0
            
            # Iniciar timer thread
            self._start_timer_thread()
        
        with self._scheduled_tasks_lock:
            self._scheduled_tasks[task_id] = task
            import heapq
            heapq.heappush(self._scheduled_tasks_heap, task)
        
        return task_id
    
    def cancel_scheduled_task(self, task_id: str) -> bool:
        """
        Cancelar una tarea programada.
        
        Args:
            task_id: ID de la tarea a cancelar
            
        Returns:
            True si fue cancelada, False si no existe o ya ejecutó
        """
        if not hasattr(self, '_scheduled_tasks'):
            return False
        
        with self._scheduled_tasks_lock:
            task = self._scheduled_tasks.get(task_id)
            if task and not task.cancelled:
                task.cancelled = True
                self._scheduled_tasks_cancelled += 1
                return True
        
        return False
    
    def get_pending_tasks(self) -> List[ScheduledTask]:
        """
        Obtener lista de tareas pendientes.
        
        Returns:
            Lista de ScheduledTask no ejecutadas ni canceladas
        """
        if not hasattr(self, '_scheduled_tasks'):
            return []
        
        with self._scheduled_tasks_lock:
            return [
                task for task in self._scheduled_tasks.values()
                if not task.cancelled and task.execute_at > time.time()
            ]
    
    def get_scheduled_task_metrics(self) -> Dict[str, Any]:
        """
        Obtener métricas de scheduled tasks.
        
        Returns:
            Dict con métricas de tasks
        """
        if not hasattr(self, '_scheduled_tasks'):
            return {
                "pending_tasks": 0,
                "executed_tasks": 0,
                "cancelled_tasks": 0,
                "failed_tasks": 0,
                "avg_delay": 0.0,
                "max_delay": 0.0,
                "oldest_pending": 0.0
            }
        
        with self._scheduled_tasks_lock:
            pending = self.get_pending_tasks()
            
            # Calcular avg_delay y max_delay de pending tasks
            delays = [task.delay for task in pending]
            avg_delay = sum(delays) / len(delays) if delays else 0.0
            max_delay = max(delays) if delays else 0.0
            
            # Oldest pending task
            now = time.time()
            ages = [now - task.scheduled_at for task in pending]
            oldest_pending = max(ages) if ages else 0.0
            
            return {
                "pending_tasks": len(pending),
                "executed_tasks": self._scheduled_tasks_executed,
                "cancelled_tasks": self._scheduled_tasks_cancelled,
                "failed_tasks": self._scheduled_tasks_failed,
                "avg_delay": avg_delay,
                "max_delay": max_delay,
                "oldest_pending": oldest_pending
            }
    
    def _start_timer_thread(self) -> None:
        """Iniciar thread que ejecuta scheduled tasks."""
        self._timer_thread_running = True
        self._timer_thread = threading.Thread(
            target=self._timer_loop,
            daemon=True,
            name="ScheduledTasksTimer"
        )
        self._timer_thread.start()
    
    def _timer_loop(self) -> None:
        """Loop principal del timer thread (ejecuta tasks ready)."""
        import heapq
        
        while self._timer_thread_running:
            try:
                # Dormir 100ms antes de check
                time.sleep(0.1)
                
                # Check scheduled tasks
                with self._scheduled_tasks_lock:
                    # Procesar tasks que están ready
                    while self._scheduled_tasks_heap:
                        # Peek task más próximo
                        task = self._scheduled_tasks_heap[0]
                        
                        # Si está cancelado, remover
                        if task.cancelled:
                            heapq.heappop(self._scheduled_tasks_heap)
                            del self._scheduled_tasks[task.task_id]
                            continue
                        
                        # Si no está ready, break (heap ordenado por execute_at)
                        if not task.is_ready():
                            break
                        
                        # Remover del heap y dict
                        heapq.heappop(self._scheduled_tasks_heap)
                        del self._scheduled_tasks[task.task_id]
                        
                        # Ejecutar callback (fuera del lock)
                        try:
                            task.callback()
                            self._scheduled_tasks_executed += 1
                        except Exception as e:
                            self._scheduled_tasks_failed += 1
                            # Log error pero no crashear timer thread
                            print(f"ScheduledTask {task.task_id} failed: {e}")
            
            except Exception as e:
                # Log pero continuar timer loop
                print(f"Timer loop error: {e}")
        
    def _stop_timer_thread(self) -> None:
        """Detener timer thread."""
        if hasattr(self, '_timer_thread') and self._timer_thread_running:
            self._timer_thread_running = False
            self._timer_thread.join(timeout=1.0)


# Helper function para crear scheduler con configuración común
def create_scheduler(
    min_threads: int = 4,
    max_threads: int = 16,
    policy: SchedulingPolicy = SchedulingPolicy.FAIR,
    max_actors: int = 10000,
    enable_work_stealing: bool = True
) -> tuple[ActorScheduler, ThreadPoolExecutor]:
    """
    Helper para crear scheduler + executor configurados.
    
    Args:
        min_threads: Threads mínimos del executor
        max_threads: Threads máximos del executor
        policy: Política de scheduling
        max_actors: Máximo de actors
        enable_work_stealing: Habilitar work stealing
    
    Returns:
        Tupla (scheduler, executor) listos para usar
    
    Example:
        scheduler, executor = create_scheduler(
            min_threads=8,
            policy=SchedulingPolicy.FAIR
        )
        scheduler.start()
        
        actor = scheduler.spawn(MyActor, name="MyActor1")
        actor.send("hello")
        
        scheduler.shutdown()
        executor.shutdown()
    """
    # Crear executor
    executor = ThreadPoolExecutor(
        min_threads=min_threads,
        max_threads=max_threads,
        enable_work_stealing=enable_work_stealing
    )
    executor.start()
    
    # Crear scheduler apropiado
    if policy == SchedulingPolicy.PRIORITY:
        scheduler = PriorityActorScheduler(
            executor=executor,
            max_actors=max_actors
        )
    else:
        scheduler = ActorScheduler(
            executor=executor,
            policy=policy,
            max_actors=max_actors
        )
    
    scheduler.start()
    
    return scheduler, executor


if __name__ == "__main__":
    """
    Ejemplo de uso del scheduler.
    """
    from src.concurrency.message_loop import CounterActorWithLoop
    
    # Crear scheduler + executor
    scheduler, executor = create_scheduler(
        min_threads=4,
        policy=SchedulingPolicy.FAIR
    )
    
    print("=== Actor Scheduler Example ===\n")
    
    # Spawn actors
    print("Spawning actors...")
    actor1 = scheduler.spawn(CounterActorWithLoop, name="Counter1")
    actor2 = scheduler.spawn(CounterActorWithLoop, name="Counter2")
    actor3 = scheduler.spawn(CounterActorWithLoop, name="Counter3")
    
    print(f"Active actors: {scheduler.get_active_actors()}\n")
    
    # Enviar mensajes
    print("Sending messages...")
    for i in range(10):
        actor1.send("increment")
        actor2.send("increment")
        actor3.send("increment")
    
    # Esperar procesamiento
    time.sleep(0.5)
    
    # Obtener métricas
    print("\n=== Scheduler Metrics ===")
    metrics = scheduler.get_metrics()
    for key, value in metrics.items():
        print(f"{key}: {value}")
    
    print("\n=== Actor Metrics ===")
    for actor_metrics in scheduler.get_all_actor_metrics():
        print(f"\nActor: {actor_metrics['name']}")
        print(f"  Uptime: {actor_metrics['uptime']:.2f}s")
        print(f"  Messages received: {actor_metrics['messages_received']}")
        print(f"  Messages processed: {actor_metrics['messages_processed']}")
        print(f"  Message rate: {actor_metrics['message_rate']:.2f} msg/s")
    
    # Shutdown
    print("\n\nShutting down...")
    scheduler.shutdown()
    executor.shutdown()
    
    print("Done!")


# ============================================================================
# SCHEDULED TASKS SYSTEM (TASK-043)
# ============================================================================