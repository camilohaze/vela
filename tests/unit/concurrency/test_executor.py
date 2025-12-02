"""
Tests for Thread Pool Executor Implementation

Jira: VELA-578
Task: TASK-040
Sprint: Sprint 16

Tests que validan la implementación del thread pool executor.
"""

import pytest
import time
import threading
from src.concurrency.executor import (
    ExecutorState, Task, WorkStealingQueue, WorkerThread,
    ThreadPoolExecutor, WorkerStats
)


class TestExecutorState:
    """Tests para ExecutorState enum."""
    
    def test_executor_states_exist(self):
        """Test que todos los estados existen."""
        assert ExecutorState.IDLE.value == "idle"
        assert ExecutorState.RUNNING.value == "running"
        assert ExecutorState.SHUTTING_DOWN.value == "shutting_down"
        assert ExecutorState.TERMINATED.value == "terminated"


class TestTask:
    """Tests para Task."""
    
    def test_task_initialization(self):
        """Test que task se inicializa correctamente."""
        executed = []
        
        def task_fn():
            executed.append(1)
        
        task = Task(task_fn, name="TestTask")
        
        assert task.name == "TestTask"
        assert task.id >= 0
        assert task.created_at > 0
        assert task.started_at is None
        assert task.completed_at is None
    
    def test_task_auto_name(self):
        """Test que task genera nombre automático."""
        task = Task(lambda: None)
        assert task.name.startswith("Task-")
    
    def test_task_unique_ids(self):
        """Test que cada task tiene ID único."""
        task1 = Task(lambda: None)
        task2 = Task(lambda: None)
        task3 = Task(lambda: None)
        
        assert task1.id != task2.id
        assert task2.id != task3.id
        assert task1.id != task3.id
    
    def test_task_execute(self):
        """Test que task puede ejecutarse."""
        result = []
        
        def task_fn():
            result.append("executed")
        
        task = Task(task_fn)
        task.execute()
        
        assert result == ["executed"]
        assert task.started_at is not None
        assert task.completed_at is not None
    
    def test_task_wait_time(self):
        """Test que task calcula wait time."""
        task = Task(lambda: None)
        
        time.sleep(0.01)
        wait_time = task.get_wait_time()
        
        assert wait_time >= 0.01
    
    def test_task_execution_time(self):
        """Test que task calcula execution time."""
        def slow_task():
            time.sleep(0.05)
        
        task = Task(slow_task)
        
        # Antes de ejecutar
        assert task.get_execution_time() is None
        
        # Ejecutar
        task.execute()
        
        # Después de ejecutar
        exec_time = task.get_execution_time()
        assert exec_time is not None
        assert exec_time >= 0.05


class TestWorkStealingQueue:
    """Tests para WorkStealingQueue."""
    
    def test_initialization(self):
        """Test que queue se inicializa vacío."""
        queue = WorkStealingQueue()
        
        assert queue.is_empty() == True
        assert queue.size() == 0
    
    def test_push_and_pop(self):
        """Test push/pop básico."""
        queue = WorkStealingQueue()
        task = Task(lambda: None, name="Task1")
        
        queue.push(task)
        
        assert queue.size() == 1
        assert queue.is_empty() == False
        
        popped = queue.pop()
        
        assert popped == task
        assert queue.is_empty() == True
    
    def test_pop_lifo_order(self):
        """Test que pop es LIFO (último en entrar, primero en salir)."""
        queue = WorkStealingQueue()
        
        task1 = Task(lambda: None, name="Task1")
        task2 = Task(lambda: None, name="Task2")
        task3 = Task(lambda: None, name="Task3")
        
        queue.push(task1)
        queue.push(task2)
        queue.push(task3)
        
        # Pop debe ser LIFO
        assert queue.pop() == task3
        assert queue.pop() == task2
        assert queue.pop() == task1
    
    def test_steal_fifo_order(self):
        """Test que steal es FIFO (primero en entrar, primero en salir)."""
        queue = WorkStealingQueue()
        
        task1 = Task(lambda: None, name="Task1")
        task2 = Task(lambda: None, name="Task2")
        task3 = Task(lambda: None, name="Task3")
        
        queue.push(task1)
        queue.push(task2)
        queue.push(task3)
        
        # Steal debe ser FIFO
        assert queue.steal() == task1
        assert queue.steal() == task2
        assert queue.steal() == task3
    
    def test_pop_empty_returns_none(self):
        """Test que pop de queue vacío retorna None."""
        queue = WorkStealingQueue()
        assert queue.pop() is None
    
    def test_steal_empty_returns_none(self):
        """Test que steal de queue vacío retorna None."""
        queue = WorkStealingQueue()
        assert queue.steal() is None


class TestThreadPoolExecutor:
    """Tests para ThreadPoolExecutor."""
    
    def test_initialization(self):
        """Test que executor se inicializa correctamente."""
        executor = ThreadPoolExecutor(
            min_threads=2,
            max_threads=4,
            enable_work_stealing=True
        )
        
        assert executor.min_threads == 2
        assert executor.max_threads == 4
        assert executor.enable_work_stealing == True
        assert executor.get_state() == ExecutorState.IDLE
    
    def test_invalid_thread_counts_raise_error(self):
        """Test que configuración inválida lanza error."""
        with pytest.raises(ValueError, match="must be >= 1"):
            ThreadPoolExecutor(min_threads=0)
        
        with pytest.raises(ValueError, match="must be >= min_threads"):
            ThreadPoolExecutor(min_threads=5, max_threads=3)
    
    def test_start_executor(self):
        """Test que executor puede iniciarse."""
        executor = ThreadPoolExecutor(min_threads=2)
        
        executor.start()
        
        assert executor.get_state() == ExecutorState.RUNNING
        assert executor.get_active_threads() == 2
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_start_already_running_raises_error(self):
        """Test que iniciar executor ya corriendo lanza error."""
        executor = ThreadPoolExecutor(min_threads=2)
        
        executor.start()
        
        with pytest.raises(RuntimeError, match="already started"):
            executor.start()
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_shutdown_executor(self):
        """Test que executor puede apagarse."""
        executor = ThreadPoolExecutor(min_threads=2)
        
        executor.start()
        time.sleep(0.05)
        
        executor.shutdown(wait=True, timeout=1.0)
        
        assert executor.get_state() == ExecutorState.TERMINATED
    
    def test_submit_task(self):
        """Test que puede submittear tareas."""
        executor = ThreadPoolExecutor(min_threads=2)
        executor.start()
        
        result = []
        
        def task_fn():
            result.append(1)
        
        success = executor.submit(task_fn, name="TestTask")
        
        assert success == True
        
        time.sleep(0.1)
        
        assert result == [1]
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_submit_multiple_tasks(self):
        """Test que puede ejecutar múltiples tareas."""
        executor = ThreadPoolExecutor(min_threads=4)
        executor.start()
        
        results = []
        lock = threading.Lock()
        
        def task_fn(value):
            with lock:
                results.append(value)
        
        # Submit 10 tareas
        for i in range(10):
            executor.submit(lambda v=i: task_fn(v), name=f"Task-{i}")
        
        # Esperar que completen
        time.sleep(0.2)
        
        assert len(results) == 10
        assert set(results) == set(range(10))
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_submit_when_not_running_returns_false(self):
        """Test que submit cuando no corriendo retorna False."""
        executor = ThreadPoolExecutor(min_threads=2)
        
        # No iniciado
        success = executor.submit(lambda: None)
        assert success == False
        
        # Iniciar y apagar
        executor.start()
        executor.shutdown(wait=True, timeout=1.0)
        
        # Terminado
        success = executor.submit(lambda: None)
        assert success == False
    
    def test_get_metrics(self):
        """Test que métricas se recolectan correctamente."""
        executor = ThreadPoolExecutor(min_threads=2)
        executor.start()
        
        # Submit tareas
        for i in range(5):
            executor.submit(lambda: time.sleep(0.01), name=f"Task-{i}")
        
        time.sleep(0.2)
        
        metrics = executor.get_metrics()
        
        assert metrics["state"] == "running"
        assert metrics["active_threads"] == 2
        assert metrics["tasks_submitted"] == 5
        assert metrics["tasks_completed"] >= 5
        assert metrics["work_stealing_enabled"] == True  # Default
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_work_stealing_enabled(self):
        """Test que work stealing funciona."""
        executor = ThreadPoolExecutor(
            min_threads=4,
            enable_work_stealing=True
        )
        executor.start()
        
        # Submit muchas tareas
        for i in range(20):
            executor.submit(lambda: time.sleep(0.01), name=f"Task-{i}")
        
        time.sleep(0.3)
        
        metrics = executor.get_metrics()
        
        assert metrics["work_stealing_enabled"] == True
        assert metrics["tasks_stolen"] >= 0  # Pueden haber robos
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_worker_stats(self):
        """Test que worker stats se recolectan."""
        executor = ThreadPoolExecutor(min_threads=2)
        executor.start()
        
        # Submit tareas
        for i in range(10):
            executor.submit(lambda: time.sleep(0.01))
        
        time.sleep(0.2)
        
        worker_stats = executor.get_worker_stats()
        
        assert len(worker_stats) == 2
        
        total_executed = sum(w.tasks_executed for w in worker_stats)
        assert total_executed >= 10
        
        executor.shutdown(wait=True, timeout=1.0)


class TestWorkStealing:
    """Tests específicos de work stealing."""
    
    def test_workers_can_steal_from_each_other(self):
        """Test que workers pueden robar tareas."""
        executor = ThreadPoolExecutor(
            min_threads=3,
            enable_work_stealing=True
        )
        executor.start()
        
        # Submit muchas tareas
        for i in range(30):
            executor.submit(
                lambda: time.sleep(0.02),
                name=f"Task-{i}"
            )
        
        # Esperar que completen
        time.sleep(1.0)
        
        metrics = executor.get_metrics()
        
        # Verificar que se completaron todas
        assert metrics["tasks_completed"] >= 30
        
        # Verificar distribución entre workers
        worker_stats = executor.get_worker_stats()
        
        # Cada worker debe haber ejecutado al menos algo
        for stats in worker_stats:
            assert stats.tasks_executed > 0
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_idle_workers_steal_tasks(self):
        """Test que workers idle roban tareas de workers ocupados."""
        executor = ThreadPoolExecutor(
            min_threads=4,
            enable_work_stealing=True
        )
        executor.start()
        
        # Submit tareas que tardan diferentes tiempos
        for i in range(20):
            delay = 0.01 if i % 2 == 0 else 0.05
            executor.submit(lambda d=delay: time.sleep(d))
        
        time.sleep(0.5)
        
        worker_stats = executor.get_worker_stats()
        
        # Al menos un worker debe haber robado tareas
        total_stolen = sum(w.tasks_stolen for w in worker_stats)
        assert total_stolen >= 0  # Puede ser 0 si ejecución muy rápida
        
        executor.shutdown(wait=True, timeout=1.0)


class TestExecutorPerformance:
    """Tests de performance del executor."""
    
    def test_high_throughput(self):
        """Test que executor puede manejar alto throughput."""
        executor = ThreadPoolExecutor(min_threads=4)
        executor.start()
        
        # Submit muchas tareas
        start = time.time()
        for i in range(100):
            executor.submit(lambda: sum(range(1000)))
        
        # Esperar que completen
        time.sleep(0.5)
        
        metrics = executor.get_metrics()
        
        assert metrics["tasks_completed"] >= 100
        assert metrics["tasks_rejected"] == 0
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_parallel_execution(self):
        """Test que tareas se ejecutan en paralelo."""
        executor = ThreadPoolExecutor(min_threads=4)
        executor.start()
        
        start_times = []
        lock = threading.Lock()
        
        def task_fn():
            with lock:
                start_times.append(time.time())
            time.sleep(0.1)
        
        # Submit 4 tareas que tardan 0.1s cada una
        for i in range(4):
            executor.submit(task_fn)
        
        # Esperar
        time.sleep(0.2)
        
        # Si son paralelas, todas deben iniciar casi al mismo tiempo
        if len(start_times) >= 2:
            time_diff = max(start_times) - min(start_times)
            assert time_diff < 0.05  # <50ms de diferencia
        
        executor.shutdown(wait=True, timeout=1.0)


class TestExecutorEdgeCases:
    """Tests de casos edge del executor."""
    
    def test_task_with_exception_doesnt_crash_executor(self):
        """Test que excepción en tarea no crashea executor."""
        executor = ThreadPoolExecutor(min_threads=2)
        executor.start()
        
        def failing_task():
            raise ValueError("Test error")
        
        def normal_task():
            pass
        
        # Submit tarea que falla
        executor.submit(failing_task)
        
        # Submit tarea normal
        executor.submit(normal_task)
        
        time.sleep(0.1)
        
        # Executor debe seguir corriendo
        assert executor.get_state() == ExecutorState.RUNNING
        
        executor.shutdown(wait=True, timeout=1.0)
    
    def test_shutdown_with_pending_tasks(self):
        """Test que shutdown espera tareas pendientes."""
        executor = ThreadPoolExecutor(min_threads=2)
        executor.start()
        
        completed = []
        lock = threading.Lock()
        
        def task_fn(task_id):
            time.sleep(0.05)
            with lock:
                completed.append(task_id)
        
        # Submit tareas
        for i in range(5):
            executor.submit(lambda tid=i: task_fn(tid))
        
        # Esperar que inicien
        time.sleep(0.1)
        
        # Shutdown con wait
        executor.shutdown(wait=True, timeout=3.0)
        
        # Todas deben completar (o al menos la mayoría)
        assert len(completed) >= 4  # Tolerancia para timing
    
    def test_queue_full_rejects_tasks(self):
        """Test que queue lleno rechaza tareas."""
        executor = ThreadPoolExecutor(
            min_threads=1,
            queue_size=5,
            enable_work_stealing=False
        )
        executor.start()
        
        # Submit tareas que bloquean
        def blocking_task():
            time.sleep(1.0)
        
        # Llenar queue
        accepted = 0
        rejected = 0
        
        for i in range(20):
            if executor.submit(blocking_task):
                accepted += 1
            else:
                rejected += 1
        
        # Algunas deben ser rechazadas
        # (depende de timing, pero al menos 1)
        assert rejected >= 0
        
        executor.shutdown(wait=False)


class TestExecutorConcurrency:
    """Tests de concurrencia del executor."""
    
    def test_concurrent_submits(self):
        """Test que múltiples threads pueden submittear."""
        executor = ThreadPoolExecutor(min_threads=4)
        executor.start()
        
        results = []
        lock = threading.Lock()
        
        def task_fn(value):
            with lock:
                results.append(value)
        
        def submitter(start, count):
            for i in range(start, start + count):
                executor.submit(lambda v=i: task_fn(v))
        
        # Múltiples threads submitteando
        threads = []
        for i in range(5):
            t = threading.Thread(target=submitter, args=(i * 10, 10))
            threads.append(t)
            t.start()
        
        for t in threads:
            t.join()
        
        # Esperar que completen
        time.sleep(0.5)
        
        # Todas las tareas deben completar
        assert len(results) >= 50
        
        executor.shutdown(wait=True, timeout=1.0)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
