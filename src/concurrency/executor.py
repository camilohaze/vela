"""
Thread Pool Executor for Actor System

This module implements a thread pool executor with work stealing and
dynamic sizing for efficient execution of actor message loops.

Jira: VELA-578
Task: TASK-040
Sprint: Sprint 16

Inspiración:
- Java ExecutorService / ForkJoinPool
- Pony work-stealing scheduler
- Tokio runtime (Rust)
"""

from abc import ABC, abstractmethod
from typing import Optional, Callable, List
from enum import Enum
import threading
import queue
import time
from dataclasses import dataclass


class ExecutorState(Enum):
    """Estados del executor."""
    IDLE = "idle"           # No iniciado
    RUNNING = "running"     # Ejecutándose
    SHUTTING_DOWN = "shutting_down"  # Apagándose
    TERMINATED = "terminated"  # Terminado


class Task:
    """
    Tarea ejecutable por el executor.
    
    Wrapper para funciones/callables con metadata.
    """
    
    _next_id = 0
    _id_lock = threading.Lock()
    
    def __init__(self, callable_fn: Callable[[], None], name: Optional[str] = None):
        """
        Crear tarea.
        
        Args:
            callable_fn: Función a ejecutar (sin argumentos)
            name: Nombre descriptivo (opcional)
        """
        with Task._id_lock:
            self.id = Task._next_id
            Task._next_id += 1
        
        self.callable_fn = callable_fn
        self.name = name if name else f"Task-{self.id}"
        self.created_at = time.time()
        self.started_at: Optional[float] = None
        self.completed_at: Optional[float] = None
    
    def execute(self) -> None:
        """Ejecutar la tarea."""
        self.started_at = time.time()
        try:
            self.callable_fn()
        finally:
            self.completed_at = time.time()
    
    def get_wait_time(self) -> float:
        """Tiempo que esperó en queue (segundos)."""
        if self.started_at is None:
            return time.time() - self.created_at
        return self.started_at - self.created_at
    
    def get_execution_time(self) -> Optional[float]:
        """Tiempo de ejecución (segundos), o None si no completó."""
        if self.started_at is None or self.completed_at is None:
            return None
        return self.completed_at - self.started_at


@dataclass
class WorkerStats:
    """Estadísticas de un worker thread."""
    thread_id: int
    tasks_executed: int = 0
    tasks_stolen: int = 0  # Tareas robadas de otros workers
    idle_time: float = 0.0
    active_time: float = 0.0


class WorkStealingQueue:
    """
    Queue con soporte para work stealing.
    
    Permite:
    - Push/pop desde owner (LIFO para mejor cache locality)
    - Steal desde otros workers (FIFO para fairness)
    """
    
    def __init__(self):
        self._deque: List[Task] = []
        self._lock = threading.Lock()
    
    def push(self, task: Task) -> None:
        """Push tarea (por owner)."""
        with self._lock:
            self._deque.append(task)
    
    def pop(self) -> Optional[Task]:
        """Pop tarea (por owner, LIFO)."""
        with self._lock:
            if not self._deque:
                return None
            return self._deque.pop()
    
    def steal(self) -> Optional[Task]:
        """Steal tarea (por otro worker, FIFO)."""
        with self._lock:
            if not self._deque:
                return None
            return self._deque.pop(0)
    
    def size(self) -> int:
        """Tamaño actual del queue."""
        with self._lock:
            return len(self._deque)
    
    def is_empty(self) -> bool:
        """Verificar si está vacío."""
        return self.size() == 0


class WorkerThread:
    """
    Worker thread que ejecuta tareas del pool.
    
    Comportamiento:
    1. Ejecutar tareas de su queue local (LIFO)
    2. Si vacío, intentar robar de otros workers (FIFO)
    3. Si no hay nada, esperar en global queue
    """
    
    def __init__(
        self,
        worker_id: int,
        global_queue: queue.Queue,
        all_workers: List['WorkerThread'],
        max_idle_time: float = 0.1
    ):
        """
        Crear worker thread.
        
        Args:
            worker_id: ID único del worker
            global_queue: Queue global compartido
            all_workers: Lista de todos los workers (para stealing)
            max_idle_time: Tiempo máximo idle antes de sleep
        """
        self.worker_id = worker_id
        self.local_queue = WorkStealingQueue()
        self.global_queue = global_queue
        self.all_workers = all_workers
        self.max_idle_time = max_idle_time
        
        self.stats = WorkerStats(thread_id=worker_id)
        self.running = False
        self.thread: Optional[threading.Thread] = None
    
    def start(self) -> None:
        """Iniciar worker thread."""
        if self.running:
            raise RuntimeError("Worker already running")
        
        self.running = True
        self.thread = threading.Thread(
            target=self._run,
            name=f"Worker-{self.worker_id}",
            daemon=True
        )
        self.thread.start()
    
    def stop(self) -> None:
        """Detener worker thread."""
        self.running = False
        if self.thread and self.thread.is_alive():
            self.thread.join(timeout=1.0)
    
    def submit_local(self, task: Task) -> None:
        """Submit tarea al queue local."""
        self.local_queue.push(task)
    
    def _run(self) -> None:
        """Loop principal del worker."""
        idle_start = None
        
        while self.running:
            task = self._get_next_task()
            
            if task is None:
                # Idle
                if idle_start is None:
                    idle_start = time.time()
                
                # Sleep corto para no consumir CPU
                time.sleep(0.001)
                continue
            
            # Ejecutar tarea
            if idle_start is not None:
                self.stats.idle_time += time.time() - idle_start
                idle_start = None
            
            exec_start = time.time()
            try:
                task.execute()
                self.stats.tasks_executed += 1
            except Exception as e:
                # Log error pero continuar
                print(f"Worker-{self.worker_id}: Error executing {task.name}: {e}")
            finally:
                exec_time = time.time() - exec_start
                self.stats.active_time += exec_time
    
    def _get_next_task(self) -> Optional[Task]:
        """
        Obtener siguiente tarea usando work stealing.
        
        Estrategia:
        1. Pop de local queue (LIFO)
        2. Steal de otros workers (FIFO)
        3. Take de global queue (FIFO)
        """
        # 1. Local queue (LIFO)
        task = self.local_queue.pop()
        if task is not None:
            return task
        
        # 2. Steal de otros workers (FIFO)
        task = self._try_steal()
        if task is not None:
            self.stats.tasks_stolen += 1
            return task
        
        # 3. Global queue
        try:
            task = self.global_queue.get(block=False)
            return task
        except queue.Empty:
            return None
    
    def _try_steal(self) -> Optional[Task]:
        """Intentar robar tarea de otro worker."""
        # Intentar robar de cada worker (excepto self)
        for worker in self.all_workers:
            if worker.worker_id == self.worker_id:
                continue
            
            task = worker.local_queue.steal()
            if task is not None:
                return task
        
        return None


class ThreadPoolExecutor:
    """
    Thread pool executor con work stealing y dynamic sizing.
    
    Características:
    - Work stealing para balance de carga automático
    - Global queue + local queues por worker
    - Dynamic sizing (futuro: ajustar workers según carga)
    - Métricas de performance
    
    Inspirado en:
    - Java ForkJoinPool
    - Pony work-stealing scheduler
    - Tokio runtime
    """
    
    def __init__(
        self,
        min_threads: int = 2,
        max_threads: int = 8,
        queue_size: int = 1000,
        enable_work_stealing: bool = True
    ):
        """
        Crear thread pool executor.
        
        Args:
            min_threads: Mínimo de threads (siempre activos)
            max_threads: Máximo de threads (para dynamic sizing)
            queue_size: Tamaño máximo del global queue
            enable_work_stealing: Habilitar work stealing
        """
        if min_threads < 1:
            raise ValueError("min_threads must be >= 1")
        if max_threads < min_threads:
            raise ValueError("max_threads must be >= min_threads")
        
        self.min_threads = min_threads
        self.max_threads = max_threads
        self.queue_size = queue_size
        self.enable_work_stealing = enable_work_stealing
        
        # Global queue
        self.global_queue: queue.Queue = queue.Queue(maxsize=queue_size)
        
        # Workers
        self.workers: List[WorkerThread] = []
        self._next_worker = 0  # Round-robin para submit
        
        # Estado
        self.state = ExecutorState.IDLE
        self._lock = threading.Lock()
        
        # Métricas
        self.tasks_submitted = 0
        self.tasks_completed = 0
        self.tasks_rejected = 0
    
    def start(self) -> None:
        """Iniciar thread pool."""
        with self._lock:
            if self.state != ExecutorState.IDLE:
                raise RuntimeError("Executor already started")
            
            # Crear y iniciar workers
            for i in range(self.min_threads):
                worker = WorkerThread(
                    worker_id=i,
                    global_queue=self.global_queue,
                    all_workers=self.workers
                )
                self.workers.append(worker)
                worker.start()
            
            self.state = ExecutorState.RUNNING
    
    def shutdown(self, wait: bool = True, timeout: Optional[float] = None) -> None:
        """
        Apagar thread pool.
        
        Args:
            wait: Esperar que terminen tareas en curso
            timeout: Timeout para wait (segundos)
        """
        with self._lock:
            if self.state == ExecutorState.TERMINATED:
                return
            
            self.state = ExecutorState.SHUTTING_DOWN
        
        # Detener workers
        if wait:
            deadline = time.time() + timeout if timeout else None
            
            for worker in self.workers:
                remaining_timeout = None
                if deadline:
                    remaining_timeout = max(0, deadline - time.time())
                
                worker.stop()
        else:
            for worker in self.workers:
                worker.running = False
        
        with self._lock:
            self.state = ExecutorState.TERMINATED
    
    def submit(self, callable_fn: Callable[[], None], name: Optional[str] = None) -> bool:
        """
        Submit tarea al executor.
        
        Args:
            callable_fn: Función a ejecutar
            name: Nombre descriptivo
        
        Returns:
            True si aceptada, False si rechazada
        """
        if self.state != ExecutorState.RUNNING:
            self.tasks_rejected += 1
            return False
        
        task = Task(callable_fn, name)
        
        # Estrategia: intentar submit a worker local, si no al global queue
        if self.enable_work_stealing and self.workers:
            # Round-robin entre workers
            worker = self.workers[self._next_worker % len(self.workers)]
            self._next_worker += 1
            
            worker.submit_local(task)
            self.tasks_submitted += 1
            return True
        else:
            # Global queue
            try:
                self.global_queue.put(task, block=False)
                self.tasks_submitted += 1
                return True
            except queue.Full:
                self.tasks_rejected += 1
                return False
    
    def get_state(self) -> ExecutorState:
        """Obtener estado actual."""
        with self._lock:
            return self.state
    
    def get_active_threads(self) -> int:
        """Obtener cantidad de threads activos."""
        return len(self.workers)
    
    def get_queue_size(self) -> int:
        """Obtener tamaño actual del global queue."""
        return self.global_queue.qsize()
    
    def get_worker_stats(self) -> List[WorkerStats]:
        """Obtener estadísticas de todos los workers."""
        return [worker.stats for worker in self.workers]
    
    def get_metrics(self) -> dict:
        """Obtener métricas del executor."""
        worker_stats = self.get_worker_stats()
        
        total_executed = sum(w.tasks_executed for w in worker_stats)
        total_stolen = sum(w.tasks_stolen for w in worker_stats)
        total_idle = sum(w.idle_time for w in worker_stats)
        total_active = sum(w.active_time for w in worker_stats)
        
        return {
            "state": self.state.value,
            "active_threads": self.get_active_threads(),
            "queue_size": self.get_queue_size(),
            "tasks_submitted": self.tasks_submitted,
            "tasks_completed": total_executed,
            "tasks_rejected": self.tasks_rejected,
            "tasks_stolen": total_stolen,
            "total_idle_time": total_idle,
            "total_active_time": total_active,
            "work_stealing_enabled": self.enable_work_stealing
        }


if __name__ == "__main__":
    """Demo de ThreadPoolExecutor."""
    print("=== Thread Pool Executor Demo ===\n")
    
    # Crear executor
    executor = ThreadPoolExecutor(
        min_threads=4,
        max_threads=8,
        enable_work_stealing=True
    )
    
    # Iniciar
    executor.start()
    print(f"Executor started with {executor.get_active_threads()} threads\n")
    
    # Submit tareas
    print("Submitting tasks...")
    
    def cpu_task(task_id: int):
        """Tarea computacional."""
        total = 0
        for i in range(100000):
            total += i
        print(f"Task {task_id} completed: {total}")
    
    # Submit 20 tareas
    for i in range(20):
        executor.submit(lambda tid=i: cpu_task(tid), name=f"CPUTask-{i}")
    
    # Esperar que completen
    print("Waiting for tasks to complete...\n")
    time.sleep(2.0)
    
    # Métricas
    metrics = executor.get_metrics()
    print("=== Metrics ===")
    print(f"Tasks submitted: {metrics['tasks_submitted']}")
    print(f"Tasks completed: {metrics['tasks_completed']}")
    print(f"Tasks stolen: {metrics['tasks_stolen']}")
    print(f"Work stealing: {metrics['work_stealing_enabled']}")
    print(f"Total active time: {metrics['total_active_time']:.3f}s")
    print(f"Total idle time: {metrics['total_idle_time']:.3f}s")
    
    # Worker stats
    print("\n=== Worker Stats ===")
    for stats in executor.get_worker_stats():
        print(f"Worker-{stats.thread_id}:")
        print(f"  Tasks executed: {stats.tasks_executed}")
        print(f"  Tasks stolen: {stats.tasks_stolen}")
        print(f"  Active time: {stats.active_time:.3f}s")
    
    # Shutdown
    print("\nShutting down...")
    executor.shutdown(wait=True, timeout=5.0)
    print(f"Executor terminated: {executor.get_state().value}")
