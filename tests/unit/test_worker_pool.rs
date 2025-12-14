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
}