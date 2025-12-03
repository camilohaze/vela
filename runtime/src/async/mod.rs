//! Async runtime implementation using Tokio

use std::future::Future;
use std::pin::Pin;
use tokio::runtime::{Builder, Runtime as TokioRuntime};
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::{timeout, Duration};
use futures::future::select_all;

/// Result type for async operations
pub type AsyncResult<T> = Result<T, AsyncError>;

/// Errors that can occur in async operations
#[derive(Debug, thiserror::Error)]
pub enum AsyncError {
    #[error("Task panicked: {0}")]
    TaskPanic(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Runtime not initialized")]
    RuntimeNotInitialized,

    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

/// Async executor using Tokio runtime
pub struct AsyncExecutor {
    runtime: TokioRuntime,
}

impl AsyncExecutor {
    /// Create a new async executor with default configuration
    pub fn new() -> AsyncResult<Self> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| AsyncError::TaskPanic(e.to_string()))?;

        Ok(Self { runtime })
    }

    /// Create executor with custom worker threads
    pub fn with_workers(workers: usize) -> AsyncResult<Self> {
        let runtime = Builder::new_multi_thread()
            .worker_threads(workers)
            .enable_all()
            .build()
            .map_err(|e| AsyncError::TaskPanic(e.to_string()))?;

        Ok(Self { runtime })
    }

    /// Spawn a task and return a handle
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// Spawn a blocking task
    pub fn spawn_blocking<F, R>(&self, f: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.runtime.spawn_blocking(f)
    }

    /// Run a future to completion
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}

/// Future abstraction
pub struct VelaFuture<T> {
    inner: Pin<Box<dyn Future<Output = T> + Send>>,
}

impl<T> VelaFuture<T> {
    /// Create a new future
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self {
            inner: Box::pin(future),
        }
    }

    /// Await the future (blocking)
    pub fn await_blocking(self, executor: &AsyncExecutor) -> AsyncResult<T> {
        Ok(executor.block_on(async { self.inner.await }))
    }
}

/// Promise abstraction
pub struct Promise<T> {
    rx: tokio::sync::oneshot::Receiver<AsyncResult<T>>,
}

impl<T> Promise<T> {
    /// Create a new promise
    pub fn new() -> (PromiseSender<T>, Self) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        (
            PromiseSender { tx },
            Self { rx },
        )
    }

    /// Wait for the promise to resolve
    pub async fn await_promise(self) -> AsyncResult<T> {
        match self.rx.await {
            Ok(result) => result,
            Err(_) => Err(AsyncError::TaskPanic("Promise sender dropped".to_string())),
        }
    }
}

/// Sender part of a promise
pub struct PromiseSender<T> {
    tx: tokio::sync::oneshot::Sender<AsyncResult<T>>,
}

impl<T> PromiseSender<T> {
    /// Resolve the promise with a value
    pub fn resolve(self, value: T) {
        let _ = self.tx.send(Ok(value));
    }

    /// Reject the promise with an error
    pub fn reject(self, error: AsyncError) {
        let _ = self.tx.send(Err(error));
    }
}

/// Task abstraction
pub struct Task<T> {
    handle: JoinHandle<T>,
}

impl<T> Task<T> {
    /// Create a new task
    pub fn spawn<F>(executor: &AsyncExecutor, future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Self {
            handle: executor.spawn(future),
        }
    }

    /// Wait for task completion
    pub async fn await_task(self) -> AsyncResult<T> {
        self.handle.await.map_err(AsyncError::JoinError)
    }

    /// Cancel the task
    pub fn cancel(self) {
        self.handle.abort();
    }
}

/// Utility functions for async operations
pub mod utils {
    use super::*;

    /// Execute with timeout
    pub async fn with_timeout<T, F>(
        future: F,
        duration: Duration,
    ) -> AsyncResult<T>
    where
        F: Future<Output = T>,
    {
        timeout(duration, future)
            .await
            .map_err(|_| AsyncError::Timeout)
    }

    /// Race multiple futures, return first to complete
    pub async fn select_first<T>(
        futures: Vec<Pin<Box<dyn Future<Output = T> + Send>>>,
    ) -> AsyncResult<T> {
        if futures.is_empty() {
            return Err(AsyncError::TaskPanic("No futures provided".to_string()));
        }

        let (result, _, _) = select_all(futures).await;
        Ok(result)
    }

    /// Execute futures concurrently and collect results
    pub async fn join_all<T>(
        futures: Vec<Pin<Box<dyn Future<Output = AsyncResult<T>> + Send>>>,
    ) -> Vec<AsyncResult<T>>
    where
        T: Send + 'static,
    {
        let mut results = Vec::with_capacity(futures.len());
        let mut join_set = JoinSet::new();

        for future in futures {
            join_set.spawn(future);
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(output) => results.push(output),
                Err(e) => results.push(Err(AsyncError::JoinError(e))),
            }
        }

        results
    }
}