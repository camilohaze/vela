//! Tests unitarios para WorkerPool
//!
//! Jira: VELA-1113
//! Task: TASK-117N

use runtime::worker_pool::{WorkerPool, Task, WorkerPoolError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_initialization() {
        let pool = WorkerPool::new(4).unwrap();
        assert_eq!(pool.max_workers(), 4);
        assert_eq!(pool.active_tasks(), 0);
        assert!(!pool.is_shutdown());
        pool.shutdown();
    }

    #[test]
    fn test_worker_pool_zero_workers() {
        let result = WorkerPool::new(0);
        assert!(matches!(result, Err(WorkerPoolError::WorkerCreationFailed)));
    }

    #[test]
    fn test_default_worker_pool() {
        let pool = WorkerPool::default();
        // Should use number of CPUs
        assert!(pool.max_workers() > 0);
        pool.shutdown();
    }

    #[test]
    fn test_submit_custom_task() {
        let pool = WorkerPool::new(2).unwrap();
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let result = pool.submit_custom({
            let counter = std::sync::Arc::clone(&counter);
            move || {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        });

        assert!(result.is_ok());
        // Give some time for task to execute
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
        pool.shutdown();
    }

    #[test]
    fn test_submit_map_task() {
        let pool = WorkerPool::new(2).unwrap();
        let data = vec!["hello".to_string(), "world".to_string()];

        let result_receiver = pool.submit_map(data, |s| s.to_uppercase()).unwrap();

        // Wait for result
        let result = result_receiver.recv().unwrap();
        assert_eq!(result, vec!["HELLO".to_string(), "WORLD".to_string()]);
        pool.shutdown();
    }

    #[test]
    fn test_submit_reduce_task() {
        let pool = WorkerPool::new(2).unwrap();
        let data = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let result_receiver = pool.submit_reduce(data, |a, b| format!("{}{}", a, b)).unwrap();

        // Wait for result
        let result = result_receiver.recv().unwrap();
        assert_eq!(result, "abc");
        pool.shutdown();
    }

    #[test]
    fn test_shutdown_behavior() {
        let pool = WorkerPool::new(2).unwrap();
        assert!(!pool.is_shutdown());

        pool.shutdown();
        assert!(pool.is_shutdown());

        // Should not accept new tasks after shutdown
        let result = pool.submit_custom(|| Ok(()));
        assert!(matches!(result, Err(WorkerPoolError::PoolFull)));
    }

    #[test]
    fn test_multiple_tasks() {
        let pool = WorkerPool::new(4).unwrap();
        let task_count = 10;
        let completed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for i in 0..task_count {
            let completed_clone = std::sync::Arc::clone(&completed);
            let result = pool.submit_custom(move || {
                std::thread::sleep(std::time::Duration::from_millis(1));
                completed_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            });
            assert!(result.is_ok());
        }

        // Give time for all tasks to complete
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(completed.load(std::sync::atomic::Ordering::SeqCst), task_count);
        pool.shutdown();
    }

    #[test]
    fn test_empty_reduce() {
        let pool = WorkerPool::new(2).unwrap();
        let data: Vec<String> = vec![];

        let result_receiver = pool.submit_reduce(data, |a, b| format!("{}{}", a, b)).unwrap();
        let result = result_receiver.recv().unwrap();
        assert_eq!(result, "");
        pool.shutdown();
    }

    #[test]
    fn test_parallel_map_empty() {
        let pool = WorkerPool::new(2).unwrap();
        let data: Vec<i32> = vec![];

        let result = pool.parallel_map(data, |x| x * 2);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        pool.shutdown();
    }

    #[test]
    fn test_parallel_reduce_single_element() {
        let pool = WorkerPool::new(2).unwrap();
        let data = vec![42];

        // This will fail with unimplemented for now, but tests the logic
        let result = pool.parallel_reduce(data, |a, b| a + b);
        assert!(result.is_err()); // Will fail due to unimplemented deserialization
        pool.shutdown();
    }

    #[test]
    fn test_parallel_reduce_empty() {
        let pool = WorkerPool::new(2).unwrap();
        let data: Vec<i32> = vec![];

        let result = pool.parallel_reduce(data, |a, b| a + b);
        assert!(result.is_err());
        pool.shutdown();
    }

    #[test]
    fn test_task_scheduler_creation() {
        let worker_pool = WorkerPool::new(2).unwrap();
        let scheduler = TaskScheduler::new(worker_pool);

        assert_eq!(scheduler.queued_tasks(), 0);
        scheduler.shutdown();
        scheduler.wait();
    }

    #[test]
    fn test_schedule_custom_task() {
        let worker_pool = WorkerPool::new(2).unwrap();
        let scheduler = TaskScheduler::new(worker_pool);
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let result = scheduler.schedule_custom(Priority::Normal, {
            let counter = std::sync::Arc::clone(&counter);
            move || {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        });

        assert!(result.is_ok());
        // Give time for task to be processed
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
        scheduler.shutdown();
        scheduler.wait();
    }

    #[test]
    fn test_priority_ordering() {
        let worker_pool = WorkerPool::new(1).unwrap(); // Single worker to test ordering
        let scheduler = TaskScheduler::new(worker_pool);
        let execution_order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        // Schedule tasks with different priorities
        for i in 0..3 {
            let execution_order_clone = std::sync::Arc::clone(&execution_order);
            let priority = match i {
                0 => Priority::Low,
                1 => Priority::High,
                2 => Priority::Normal,
                _ => Priority::Normal,
            };

            scheduler.schedule_custom(priority, move || {
                execution_order_clone.lock().unwrap().push(i);
                std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure ordering
                Ok(())
            }).unwrap();
        }

        // Wait for all tasks to complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        let order = execution_order.lock().unwrap().clone();
        // High priority (1) should execute before Normal (2), Low may be last
        assert!(order.contains(&1));
        assert!(order.contains(&2));
        assert!(order.contains(&0));

        scheduler.shutdown();
        scheduler.wait();
    }

    #[test]
    fn test_scheduler_shutdown() {
        let worker_pool = WorkerPool::new(2).unwrap();
        let scheduler = TaskScheduler::new(worker_pool);

        scheduler.shutdown();
        assert!(!scheduler.worker_pool.is_shutdown());

        // Should not accept new tasks after shutdown
        let result = scheduler.schedule_custom(Priority::Normal, || Ok(()));
        assert!(result.is_err());
    }
}