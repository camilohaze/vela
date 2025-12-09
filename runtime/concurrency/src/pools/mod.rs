//! Worker pool implementations.
//!
//! This module provides different types of worker pools for executing tasks:
//!
//! - [`ThreadPool`]: CPU-bound tasks using Rayon
//! - [`AsyncPool`]: IO-bound async tasks using Tokio
//!
//! # Examples
//!
//! ## Thread Pool (CPU-bound)
//!
//! ```rust
//! use vela_concurrency::pools::ThreadPool;
//!
//! let pool = ThreadPool::new().unwrap();
//!
//! pool.execute(|| {
//!     // CPU-intensive work
//!     let sum: u64 = (0..1_000_000).sum();
//!     println!("Sum: {}", sum);
//! }).unwrap();
//!
//! pool.join();
//! ```
//!
//! ## Async Pool (IO-bound)
//!
//! ```rust
//! use vela_concurrency::pools::AsyncPool;
//!
//! let pool = AsyncPool::new().unwrap();
//!
//! let handle = pool.spawn(async {
//!     // IO-bound work
//!     tokio::time::sleep(std::time::Duration::from_millis(100)).await;
//!     42
//! }).unwrap();
//!
//! let result = pool.block_on(async { handle.await.unwrap() });
//! assert_eq!(result, 42);
//! ```

mod thread_pool;
mod async_pool;

pub use thread_pool::{ThreadPool, ThreadPoolConfig, ThreadPoolError};
pub use async_pool::{AsyncPool, AsyncPoolConfig, AsyncPoolError};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_thread_pool_and_async_pool_together() {
        let thread_pool = ThreadPool::new().unwrap();
        let async_pool = AsyncPool::new().unwrap();

        let counter = Arc::new(AtomicUsize::new(0));

        // CPU-bound task
        let counter_clone = Arc::clone(&counter);
        thread_pool
            .execute(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();

        // IO-bound task
        let counter_clone = Arc::clone(&counter);
        let handle = async_pool
            .spawn(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();

        thread_pool.join();
        async_pool.block_on(async { handle.await.unwrap() });

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_pools_can_be_cloned() {
        let thread_pool = ThreadPool::new().unwrap();
        let async_pool = AsyncPool::new().unwrap();

        let thread_pool_clone = thread_pool.clone();
        let async_pool_clone = async_pool.clone();

        let counter = Arc::new(AtomicUsize::new(0));

        // Use clones
        let counter_clone = Arc::clone(&counter);
        thread_pool_clone
            .execute(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();

        let counter_clone = Arc::clone(&counter);
        let handle = async_pool_clone
            .spawn(async move {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();

        thread_pool.join();
        async_pool.block_on(async { handle.await.unwrap() });

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
