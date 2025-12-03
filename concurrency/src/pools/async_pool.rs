//! Async task pool using Tokio runtime.
//!
//! A pool for IO-bound async tasks using Tokio's work-stealing scheduler.

use std::future::Future;
use std::sync::Arc;
use parking_lot::Mutex;
use thiserror::Error;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

/// Errors that can occur when using an async pool.
#[derive(Debug, Error, Clone)]
pub enum AsyncPoolError {
    /// The pool is shutting down and cannot accept new tasks.
    #[error("async pool is shutting down")]
    ShuttingDown,

    /// Failed to create the runtime.
    #[error("failed to create runtime: {0}")]
    CreationFailed(String),

    /// A task was cancelled.
    #[error("task was cancelled")]
    TaskCancelled,

    /// A task panicked during execution.
    #[error("task panicked: {0}")]
    TaskPanicked(String),
}

/// Configuration for an async pool.
#[derive(Debug, Clone)]
pub struct AsyncPoolConfig {
    /// Number of worker threads. Defaults to number of CPU cores.
    pub worker_threads: Option<usize>,

    /// Maximum number of blocking threads.
    pub max_blocking_threads: Option<usize>,

    /// Name prefix for worker threads.
    pub thread_name_prefix: String,

    /// Stack size for worker threads in bytes.
    pub thread_stack_size: Option<usize>,

    /// Enable IO driver.
    pub enable_io: bool,

    /// Enable time driver.
    pub enable_time: bool,
}

impl Default for AsyncPoolConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // Use CPU count
            max_blocking_threads: Some(512),
            thread_name_prefix: "vela-async-worker".to_string(),
            thread_stack_size: None,
            enable_io: true,
            enable_time: true,
        }
    }
}

/// An async task pool using Tokio.
///
/// Provides a multi-threaded async runtime for IO-bound tasks.
///
/// # Examples
///
/// ```rust
/// use vela_concurrency::pools::AsyncPool;
///
/// let pool = AsyncPool::new().unwrap();
///
/// // Spawn an async task
/// let handle = pool.spawn(async {
///     println!("Running async task");
///     42
/// }).unwrap();
///
/// // Wait for the task
/// let result = pool.block_on(async { handle.await.unwrap() });
/// assert_eq!(result, 42);
/// ```
pub struct AsyncPool {
    runtime: Arc<Runtime>,
    shutdown: Arc<Mutex<bool>>,
}

impl AsyncPool {
    /// Create a new async pool with default configuration.
    pub fn new() -> Result<Self, AsyncPoolError> {
        Self::with_config(AsyncPoolConfig::default())
    }

    /// Create a new async pool with custom configuration.
    pub fn with_config(config: AsyncPoolConfig) -> Result<Self, AsyncPoolError> {
        let mut builder = Builder::new_multi_thread();

        if let Some(worker_threads) = config.worker_threads {
            builder.worker_threads(worker_threads);
        }

        if let Some(max_blocking_threads) = config.max_blocking_threads {
            builder.max_blocking_threads(max_blocking_threads);
        }

        builder.thread_name(config.thread_name_prefix);

        if let Some(stack_size) = config.thread_stack_size {
            builder.thread_stack_size(stack_size);
        }

        if config.enable_io {
            builder.enable_io();
        }

        if config.enable_time {
            builder.enable_time();
        }

        let runtime = builder
            .build()
            .map_err(|e| AsyncPoolError::CreationFailed(e.to_string()))?;

        Ok(Self {
            runtime: Arc::new(runtime),
            shutdown: Arc::new(Mutex::new(false)),
        })
    }

    /// Spawn an async task on the pool.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to execute
    ///
    /// # Returns
    ///
    /// A join handle for the task.
    ///
    /// # Errors
    ///
    /// Returns `AsyncPoolError::ShuttingDown` if the pool is shutting down.
    pub fn spawn<F>(&self, future: F) -> Result<JoinHandle<F::Output>, AsyncPoolError>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        if *self.shutdown.lock() {
            return Err(AsyncPoolError::ShuttingDown);
        }

        Ok(self.runtime.spawn(future))
    }

    /// Spawn a blocking task on the pool.
    ///
    /// Uses Tokio's blocking thread pool for CPU-intensive work.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to execute
    ///
    /// # Returns
    ///
    /// A join handle for the task.
    pub fn spawn_blocking<F, R>(&self, f: F) -> Result<JoinHandle<R>, AsyncPoolError>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        if *self.shutdown.lock() {
            return Err(AsyncPoolError::ShuttingDown);
        }

        Ok(self.runtime.spawn_blocking(f))
    }

    /// Block on a future until completion.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to block on
    ///
    /// # Returns
    ///
    /// The result of the future.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        self.runtime.block_on(future)
    }

    /// Spawn multiple futures in parallel.
    ///
    /// # Arguments
    ///
    /// * `futures` - Vector of futures to execute in parallel
    ///
    /// # Returns
    ///
    /// Vector of join handles.
    pub fn spawn_many<F>(
        &self,
        futures: Vec<F>,
    ) -> Result<Vec<JoinHandle<F::Output>>, AsyncPoolError>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        if *self.shutdown.lock() {
            return Err(AsyncPoolError::ShuttingDown);
        }

        Ok(futures
            .into_iter()
            .map(|fut| self.runtime.spawn(fut))
            .collect())
    }

    /// Shut down the pool (prevents new tasks from being submitted).
    pub fn shutdown(&self) {
        *self.shutdown.lock() = true;
    }

    /// Check if the pool is shutting down.
    pub fn is_shutting_down(&self) -> bool {
        *self.shutdown.lock()
    }

    /// Get a handle to the underlying Tokio runtime.
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }
}

impl Clone for AsyncPool {
    fn clone(&self) -> Self {
        Self {
            runtime: Arc::clone(&self.runtime),
            shutdown: Arc::clone(&self.shutdown),
        }
    }
}

impl Default for AsyncPool {
    fn default() -> Self {
        Self::new().expect("Failed to create default async pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::time::sleep;

    #[test]
    fn test_async_pool_new() {
        let pool = AsyncPool::new();
        assert!(pool.is_ok());
    }

    #[test]
    fn test_async_pool_spawn() {
        let pool = AsyncPool::new().unwrap();

        let handle = pool
            .spawn(async {
                sleep(Duration::from_millis(10)).await;
                42
            })
            .unwrap();

        let result = pool.block_on(async { handle.await.unwrap() });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_async_pool_spawn_blocking() {
        let pool = AsyncPool::new().unwrap();

        let handle = pool
            .spawn_blocking(|| {
                std::thread::sleep(Duration::from_millis(10));
                42
            })
            .unwrap();

        let result = pool.block_on(async { handle.await.unwrap() });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_async_pool_block_on() {
        let pool = AsyncPool::new().unwrap();

        let result = pool.block_on(async {
            sleep(Duration::from_millis(10)).await;
            42
        });

        assert_eq!(result, 42);
    }

    #[test]
    fn test_async_pool_spawn_many() {
        let pool = AsyncPool::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));

        let futures: Vec<_> = (0..10)
            .map(|_| {
                let counter = Arc::clone(&counter);
                async move {
                    sleep(Duration::from_millis(10)).await;
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            })
            .collect();

        let handles = pool.spawn_many(futures).unwrap();

        pool.block_on(async {
            for handle in handles {
                handle.await.unwrap();
            }
        });

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_async_pool_shutdown() {
        let pool = AsyncPool::new().unwrap();
        assert!(!pool.is_shutting_down());

        pool.shutdown();
        assert!(pool.is_shutting_down());

        let result = pool.spawn(async { 42 });
        assert!(matches!(result, Err(AsyncPoolError::ShuttingDown)));
    }

    #[test]
    fn test_async_pool_custom_config() {
        let config = AsyncPoolConfig {
            worker_threads: Some(4),
            max_blocking_threads: Some(256),
            thread_name_prefix: "test-async".to_string(),
            thread_stack_size: Some(2 * 1024 * 1024), // 2MB
            enable_io: true,
            enable_time: true,
        };

        let pool = AsyncPool::with_config(config);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_async_pool_clone() {
        let pool = AsyncPool::new().unwrap();
        let pool_clone = pool.clone();

        let handle = pool_clone.spawn(async { 42 }).unwrap();

        let result = pool.block_on(async { handle.await.unwrap() });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_async_pool_concurrent_tasks() {
        let pool = AsyncPool::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..100)
            .map(|_| {
                let counter = Arc::clone(&counter);
                pool.spawn(async move {
                    sleep(Duration::from_millis(1)).await;
                    counter.fetch_add(1, Ordering::SeqCst);
                })
                .unwrap()
            })
            .collect();

        pool.block_on(async {
            for handle in handles {
                handle.await.unwrap();
            }
        });

        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }
}
