"""
Tests para Executor y Task

Tests de:
- Task lifecycle (PENDING → RUNNING → COMPLETED)
- TaskHandle API
- Executor spawn y polling
- Executor run_until_complete
- Executor run con step
- block_on helper
- Runtime singleton
- Task cancellation
- Error handling
- Concurrencia

Jira: VELA-580
Historia: TASK-048
"""

import pytest
import time
from src.runtime.async_runtime import (
    Future, Poll, Waker,
    Task, TaskHandle, TaskId, TaskState,
    Executor, Runtime, block_on,
    Promise
)


class TestTask:
    """Tests de Task lifecycle"""
    
    def test_create_task(self):
        """Test crear task"""
        future = Future.ready(42)
        task = Task(future)
        
        assert task.state == TaskState.PENDING
        assert not task.is_completed()
        assert not task.is_failed()
        assert not task.is_cancelled()
        assert task.is_pending()
    
    def test_poll_ready_task(self):
        """Test poll task que ya está ready"""
        future = Future.ready(42)
        task = Task(future)
        
        waker = Waker.noop()
        poll = task.poll(waker)
        
        assert poll.is_ready()
        assert poll.unwrap() == 42
        assert task.is_completed()
        assert task.result() == 42
    
    def test_poll_pending_task(self):
        """Test poll task pendiente"""
        future = Future.pending()
        task = Task(future)
        
        waker = Waker.noop()
        poll = task.poll(waker)
        
        assert poll.is_pending()
        assert task.is_pending()
    
    def test_task_result_before_completion(self):
        """Test obtener result antes de completar"""
        future = Future.pending()
        task = Task(future)
        
        with pytest.raises(RuntimeError, match="not completed"):
            task.result()
    
    def test_task_error(self):
        """Test task que falla"""
        # Crear ErrorFuture que lanza excepción
        class FailingFuture(Future[int]):
            def poll(self, waker):
                raise ValueError("test error")
        
        future = FailingFuture()
        task = Task(future)
        waker = Waker.noop()
        
        with pytest.raises(ValueError):
            task.poll(waker)
        
        assert task.is_failed()
        assert isinstance(task.error(), ValueError)
    
    def test_task_cancel(self):
        """Test cancelar task"""
        future = Future.pending()
        task = Task(future)
        
        # Cancelar
        cancelled = task.cancel()
        assert cancelled
        assert task.is_cancelled()
        
        # No se puede cancelar dos veces
        cancelled = task.cancel()
        assert cancelled  # Sigue cancelado
    
    def test_cannot_poll_cancelled_task(self):
        """Test no se puede poll task cancelado"""
        future = Future.pending()
        task = Task(future)
        
        task.cancel()
        
        waker = Waker.noop()
        with pytest.raises(RuntimeError, match="cancelled"):
            task.poll(waker)
    
    def test_cannot_cancel_completed_task(self):
        """Test no se puede cancelar task completado"""
        future = Future.ready(42)
        task = Task(future)
        
        waker = Waker.noop()
        task.poll(waker)
        
        assert task.is_completed()
        
        # Intentar cancelar
        cancelled = task.cancel()
        assert not cancelled  # No cancelado (ya completó)
    
    def test_task_id_unique(self):
        """Test TaskId es único"""
        task1 = Task(Future.ready(1))
        task2 = Task(Future.ready(2))
        
        assert task1.task_id != task2.task_id
        assert hash(task1.task_id) != hash(task2.task_id)


class TestTaskHandle:
    """Tests de TaskHandle"""
    
    def test_task_handle_creation(self):
        """Test crear TaskHandle"""
        future = Future.ready(42)
        task = Task(future)
        handle = TaskHandle(task)
        
        assert handle.task_id() == task.task_id
        assert not handle.is_completed()
    
    def test_task_handle_result(self):
        """Test obtener resultado via handle"""
        future = Future.ready(42)
        task = Task(future)
        handle = TaskHandle(task)
        
        # Poll para completar
        waker = Waker.noop()
        task.poll(waker)
        
        assert handle.is_completed()
        assert handle.result() == 42
    
    def test_task_handle_cancel(self):
        """Test cancelar via handle"""
        future = Future.pending()
        task = Task(future)
        handle = TaskHandle(task)
        
        cancelled = handle.cancel()
        assert cancelled
        assert handle.is_cancelled()


class TestExecutor:
    """Tests de Executor"""
    
    def test_create_executor(self):
        """Test crear executor"""
        executor = Executor()
        
        assert executor.active_tasks() == 0
        assert executor.ready_tasks() == 0
        assert executor.waiting_tasks() == 0
    
    def test_spawn_task(self):
        """Test spawn task"""
        executor = Executor()
        future = Future.ready(42)
        
        handle = executor.spawn(future)
        
        assert not handle.is_completed()
        assert executor.ready_tasks() == 1
    
    def test_executor_step_ready(self):
        """Test step con task ready"""
        executor = Executor()
        future = Future.ready(42)
        handle = executor.spawn(future)
        
        # Step procesa task
        processed = executor.step()
        
        assert processed
        assert handle.is_completed()
        assert handle.result() == 42
        assert executor.active_tasks() == 0
    
    def test_executor_step_pending(self):
        """Test step con task pending"""
        executor = Executor()
        future = Future.pending()
        handle = executor.spawn(future)
        
        # Step procesa pero queda pending
        processed = executor.step()
        
        assert processed
        assert not handle.is_completed()
        assert executor.waiting_tasks() == 1
    
    def test_executor_run_until_complete(self):
        """Test run_until_complete"""
        executor = Executor()
        future = Future.ready(42)
        
        result = executor.run_until_complete(future)
        
        assert result == 42
    
    def test_executor_run_with_promise(self):
        """Test executor con promise que se resuelve"""
        executor = Executor()
        promise = Promise[int]()
        future = promise.future()
        
        # Spawn future
        handle = executor.spawn(future)
        
        # Step (queda pending)
        executor.step()
        assert not handle.is_completed()
        assert executor.waiting_tasks() == 1
        
        # Resolver promise (despierta waker)
        promise.resolve(42)
        
        # Step (ahora completa)
        executor.step()
        assert handle.is_completed()
        assert handle.result() == 42
    
    def test_executor_multiple_tasks(self):
        """Test executor con múltiples tasks"""
        executor = Executor()
        
        # Spawn 3 tasks
        h1 = executor.spawn(Future.ready(1))
        h2 = executor.spawn(Future.ready(2))
        h3 = executor.spawn(Future.ready(3))
        
        assert executor.ready_tasks() == 3
        
        # Procesar todos
        executor.step()
        executor.step()
        executor.step()
        
        assert h1.result() == 1
        assert h2.result() == 2
        assert h3.result() == 3
        assert executor.active_tasks() == 0
    
    def test_executor_run_with_max_iterations(self):
        """Test run con max_iterations"""
        executor = Executor()
        
        # Spawn tasks
        executor.spawn(Future.ready(1))
        executor.spawn(Future.ready(2))
        
        # Run solo 2 iteraciones
        executor.run(max_iterations=2)
        
        assert executor.active_tasks() == 0
    
    def test_executor_stop(self):
        """Test stop executor"""
        executor = Executor()
        
        # Spawn task
        executor.spawn(Future.ready(42))
        
        # Stop
        executor.stop()
        
        assert not executor._running


class TestBlockOn:
    """Tests de block_on helper"""
    
    def test_block_on_ready(self):
        """Test block_on con future ready"""
        future = Future.ready(42)
        result = block_on(future)
        
        assert result == 42
    
    def test_block_on_promise(self):
        """Test block_on con promise"""
        promise = Promise[int]()
        future = promise.future()
        
        # Resolver inmediatamente
        promise.resolve(100)
        
        result = block_on(future)
        assert result == 100
    
    def test_block_on_map(self):
        """Test block_on con future map"""
        future = Future.ready(10).map(lambda x: x * 2)
        result = block_on(future)
        
        assert result == 20


class TestRuntime:
    """Tests de Runtime singleton"""
    
    def test_runtime_singleton(self):
        """Test Runtime es singleton"""
        rt1 = Runtime.get()
        rt2 = Runtime.get()
        
        assert rt1 is rt2
    
    def test_runtime_spawn(self):
        """Test spawn via Runtime"""
        runtime = Runtime.get()
        future = Future.ready(42)
        
        handle = runtime.spawn(future)
        
        # Procesar
        runtime.executor.step()
        
        assert handle.is_completed()
        assert handle.result() == 42
    
    def test_runtime_block_on(self):
        """Test block_on via Runtime"""
        runtime = Runtime.get()
        future = Future.ready(42)
        
        result = runtime.block_on(future)
        
        assert result == 42


class TestIntegration:
    """Tests de integración"""
    
    def test_chained_futures(self):
        """Test executor con futures encadenados"""
        executor = Executor()
        
        future = (Future.ready(5)
            .map(lambda x: x * 2)      # 10
            .map(lambda x: x + 5)      # 15
            .map(lambda x: x / 3))     # 5.0
        
        result = executor.run_until_complete(future)
        
        assert result == 5.0
    
    def test_future_all_with_executor(self):
        """Test Future.all con executor"""
        executor = Executor()
        
        futures = [
            Future.ready(1),
            Future.ready(2),
            Future.ready(3)
        ]
        
        all_future = Future.all(futures)
        result = executor.run_until_complete(all_future)
        
        assert result == [1, 2, 3]
    
    def test_future_race_with_executor(self):
        """Test Future.race con executor"""
        executor = Executor()
        
        futures = [
            Future.ready(10),
            Future.pending(),
            Future.pending()
        ]
        
        race_future = Future.race(futures)
        result = executor.run_until_complete(race_future)
        
        assert result == 10
    
    def test_promise_resolution_flow(self):
        """Test flujo completo con promise resolution"""
        executor = Executor()
        promise = Promise[str]()
        future = promise.future()
        
        # Spawn
        handle = executor.spawn(future)
        
        # Step (pending)
        executor.step()
        assert not handle.is_completed()
        
        # Resolver
        promise.resolve("success")
        
        # Step (wake up y completa)
        executor.step()
        assert handle.is_completed()
        assert handle.result() == "success"
    
    def test_error_propagation(self):
        """Test propagación de errores"""
        executor = Executor()
        
        # Crear future que falla
        class FailingFuture(Future[int]):
            def poll(self, waker):
                raise ValueError("test error")
        
        future = FailingFuture()
        
        with pytest.raises(ValueError):
            executor.run_until_complete(future)


class TestEdgeCases:
    """Tests de casos borde"""
    
    def test_empty_executor_step(self):
        """Test step en executor vacío"""
        executor = Executor()
        
        processed = executor.step()
        
        assert not processed
    
    def test_task_double_poll(self):
        """Test poll task dos veces"""
        future = Future.ready(42)
        task = Task(future)
        waker = Waker.noop()
        
        # Primera poll
        poll1 = task.poll(waker)
        assert poll1.unwrap() == 42
        
        # Segunda poll (retorna mismo resultado)
        poll2 = task.poll(waker)
        assert poll2.unwrap() == 42
    
    def test_cancel_then_get_result(self):
        """Test obtener result después de cancelar"""
        future = Future.pending()
        task = Task(future)
        
        task.cancel()
        
        with pytest.raises(RuntimeError, match="cancelled"):
            task.result()


# Helper para Result (OK/Err)
class Ok:
    def __init__(self, value):
        self.value = value
    
    def is_ok(self):
        return True
    
    def unwrap(self):
        return self.value


class Err:
    def __init__(self, error):
        self.error = error
    
    def is_ok(self):
        return False
    
    def unwrap(self):
        raise self.error


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
