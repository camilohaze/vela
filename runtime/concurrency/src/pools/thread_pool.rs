//! Thread-based worker pool using Rayon.
//!
//! A thread pool for CPU-bound tasks using work-stealing scheduling.

use std::sync::Arc;
use rayon::{ThreadPool as RayonPool, ThreadPoolBuilder};
use parking_lot::Mutex;
use thiserror::Error;

/// Errors that can occur when using a thread pool.
#[derive(Debug, Error, Clone)]
pub enum ThreadPoolError {
    /// The pool is shutting down and cannot accept new tasks.
    #[error("thread pool is shutting down")]
    ShuttingDown,

    /// Failed to create the thread pool.
    #[error("failed to create thread pool: {0}")]
    CreationFailed(String),

    /// A task panicked during execution.
    #[error("task panicked: {0}")]
    TaskPanicked(String),
}

/// Configuration for a thread pool.
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// Number of worker threads. Defaults to number of CPU cores.
    pub num_threads: Option<usize>,

    /// Name prefix for worker threads.
    pub thread_name_prefix: String,

    /// Stack size for worker threads in bytes.
    pub stack_size: Option<usize>,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            num_threads: None, // Use CPU count
            thread_name_prefix: "vela-worker".to_string(),
            stack_size: None, // Use Rayon default
        }
    }
}

/// A thread pool for CPU-bound tasks.
///
/// Uses Rayon's work-stealing scheduler for efficient parallelism.
///
/// # Examples
///
/// ```rust
/// use vela_concurrency::pools::ThreadPool;
///
/// let pool = ThreadPool::new().unwrap();
///
/// // Execute a task
/// pool.execute(|| {
///     println!("Running in thread pool");
/// }).unwrap();
///
/// // Wait for all tasks to complete
/// pool.join();
/// ```
pub struct ThreadPool {
    pool: Arc<RayonPool>,
    shutdown: Arc<Mutex<bool>>,
}

impl ThreadPool {
    /// Create a new thread pool with default configuration.
    pub fn new() -> Result<Self, ThreadPoolError> {
        Self::with_config(ThreadPoolConfig::default())
    }

    /// Create a new thread pool with custom configuration.
    pub fn with_config(config: ThreadPoolConfig) -> Result<Self, ThreadPoolError> {
        let mut builder = ThreadPoolBuilder::new();

        if let Some(num_threads) = config.num_threads {
            builder = builder.num_threads(num_threads);
        }

        builder = builder.thread_name(move |i| format!("{}-{}", config.thread_name_prefix, i));

        if let Some(stack_size) = config.stack_size {
            builder = builder.stack_size(stack_size);
        }

        let pool = builder
            .build()
            .map_err(|e| ThreadPoolError::CreationFailed(e.to_string()))?;

        Ok(Self {
            pool: Arc::new(pool),
            shutdown: Arc::new(Mutex::new(false)),
        })
    }

    /// Execute a closure on the thread pool.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to execute
    ///
    /// # Errors
    ///
    /// Returns `ThreadPoolError::ShuttingDown` if the pool is shutting down.
    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        if *self.shutdown.lock() {
            return Err(ThreadPoolError::ShuttingDown);
        }

        self.pool.spawn(f);
        Ok(())
    }

    /// Execute a closure and return a result via channel.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to execute
    ///
    /// # Returns
    ///
    /// A receiver for the result.
    pub fn execute_with_result<F, T>(&self, f: F) -> Result<std::sync::mpsc::Receiver<T>, ThreadPoolError>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        if *self.shutdown.lock() {
            return Err(ThreadPoolError::ShuttingDown);
        }

        let (tx, rx) = std::sync::mpsc::channel();

        self.pool.spawn(move || {
            let result = f();
            let _ = tx.send(result); // Ignore send error (receiver might be dropped)
        });

        Ok(rx)
    }

    /// Execute multiple closures in parallel.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Vector of closures to execute in parallel
    ///
    /// # Errors
    ///
    /// Returns `ThreadPoolError::ShuttingDown` if the pool is shutting down.
    pub fn execute_parallel<F>(&self, tasks: Vec<F>) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        if *self.shutdown.lock() {
            return Err(ThreadPoolError::ShuttingDown);
        }

        self.pool.scope(|s| {
            for task in tasks {
                s.spawn(|_| task());
            }
        });

        Ok(())
    }

    /// Wait for all currently executing tasks to complete.
    pub fn join(&self) {
        // Rayon doesn't have explicit join, but we can use broadcast
        self.pool.broadcast(|_| {});
    }

    /// Shut down the pool (prevents new tasks from being submitted).
    pub fn shutdown(&self) {
        *self.shutdown.lock() = true;
    }

    /// Check if the pool is shutting down.
    pub fn is_shutting_down(&self) -> bool {
        *self.shutdown.lock()
    }

    /// Get the number of threads in the pool.
    pub fn num_threads(&self) -> usize {
        self.pool.current_num_threads()
    }
}

impl Clone for ThreadPool {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            shutdown: Arc::clone(&self.shutdown),
        }
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new().expect("Failed to create default thread pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn test_thread_pool_new() {
        let pool = ThreadPool::new();
        assert!(pool.is_ok());
    }

    #[test]
    fn test_thread_pool_execute() {
        let pool = ThreadPool::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        pool.execute(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();

        pool.join();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_thread_pool_execute_with_result() {
        let pool = ThreadPool::new().unwrap();

        let rx = pool
            .execute_with_result(|| {
                std::thread::sleep(Duration::from_millis(10));
                42
            })
            .unwrap();

        let result = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_thread_pool_execute_parallel() {
        let pool = ThreadPool::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));

        let tasks: Vec<_> = (0..10)
            .map(|_| {
                let counter = Arc::clone(&counter);
                move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            })
            .collect();

        pool.execute_parallel(tasks).unwrap();
        pool.join();

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_thread_pool_shutdown() {
        let pool = ThreadPool::new().unwrap();
        assert!(!pool.is_shutting_down());

        pool.shutdown();
        assert!(pool.is_shutting_down());

        let result = pool.execute(|| {});
        assert!(matches!(result, Err(ThreadPoolError::ShuttingDown)));
    }

    #[test]
    fn test_thread_pool_custom_config() {
        let config = ThreadPoolConfig {
            num_threads: Some(4),
            thread_name_prefix: "test-worker".to_string(),
            stack_size: Some(2 * 1024 * 1024), // 2MB
        };

        let pool = ThreadPool::with_config(config).unwrap();
        assert_eq!(pool.num_threads(), 4);
    }

    #[test]
    fn test_thread_pool_clone() {
        let pool = ThreadPool::new().unwrap();
        let pool_clone = pool.clone();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        pool_clone
            .execute(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();

        pool.join();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
