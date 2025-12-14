//! Tests unitarios para WorkerPool
//!
//! Jira: VELA-1113
//! Task: TASK-117M

use runtime::worker_pool::{WorkerPool, Task};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_initialization() {
        let pool = WorkerPool::new(4);
        assert_eq!(pool.max_workers, 4);
        assert_eq!(pool.active_tasks(), 0);
        pool.shutdown();
    }

    #[test]
    fn test_default_worker_pool() {
        let pool = WorkerPool::default();
        // Should use number of CPUs
        assert!(pool.max_workers > 0);
        pool.shutdown();
    }

    #[test]
    fn test_submit_custom_task() {
        let pool = WorkerPool::new(2);
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let task = Task::Custom {
            function: {
                let counter = std::sync::Arc::clone(&counter);
                Box::new(move || {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                })
            },
        };

        assert!(pool.submit(task).is_ok());
        // Give some time for task to execute
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
        pool.shutdown();
    }

    #[test]
    fn test_map_task() {
        let pool = WorkerPool::new(2);
        let data = vec!["hello".to_string(), "world".to_string()];

        let task = Task::Map {
            data,
            mapper: Box::new(|s| s.to_uppercase()),
        };

        assert!(pool.submit(task).is_ok());
        pool.shutdown();
    }

    #[test]
    fn test_reduce_task() {
        let pool = WorkerPool::new(2);
        let data = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let task = Task::Reduce {
            data,
            reducer: Box::new(|a, b| format!("{}{}", a, b)),
        };

        assert!(pool.submit(task).is_ok());
        pool.shutdown();
    }

    #[test]
    fn test_multiple_tasks() {
        let pool = WorkerPool::new(4);

        for i in 0..10 {
            let task = Task::Custom {
                function: Box::new(move || {
                    println!("Task {} executed", i);
                    Ok(())
                }),
            };
            assert!(pool.submit(task).is_ok());
        }

        pool.shutdown();
    }
}