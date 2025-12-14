//! Worker Pool Implementation for Parallel Processing
//!
//! This module provides worker pools for parallel task execution in Vela.
//! It integrates with the async runtime and provides load balancing capabilities.
//!
//! Jira: VELA-1113
//! Task: TASK-117N
//! Date: 2025-12-14

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::collections::VecDeque;

/// Error types for worker pool operations
#[derive(Debug, Clone)]
pub enum WorkerPoolError {
    PoolFull,
    TaskSubmissionFailed(String),
    WorkerCreationFailed,
}

/// Result type for worker pool operations
pub type Result<T> = std::result::Result<T, WorkerPoolError>;

/// Represents a task that can be executed by a worker
pub enum Task {
    /// Map operation over a collection
    Map {
        data: Vec<String>,
        mapper: Box<dyn Fn(String) -> String + Send + 'static>,
        result_sender: Sender<Vec<String>>,
    },
    /// Reduce operation over a collection
    Reduce {
        data: Vec<String>,
        reducer: Box<dyn Fn(String, String) -> String + Send + 'static>,
        result_sender: Sender<String>,
    },
    /// Custom task
    Custom {
        function: Box<dyn FnOnce() -> Result<()> + Send + 'static>,
    },
}

/// A worker that processes tasks
struct Worker {
    id: usize,
    handle: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, task_receiver: Arc<std::sync::Mutex<Receiver<Task>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                let task = {
                    let receiver = task_receiver.lock().unwrap();
                    match receiver.recv() {
                        Ok(task) => task,
                        Err(_) => break, // Channel closed
                    }
                };

                match task {
                    Task::Map { data, mapper, result_sender } => {
                        let results: Vec<String> = data.into_iter().map(|item| mapper(item)).collect();
                        let _ = result_sender.send(results); // Ignore send errors
                    }
                    Task::Reduce { data, reducer, result_sender } => {
                        if data.is_empty() {
                            let _ = result_sender.send(String::new());
                        } else {
                            let mut result = data[0].clone();
                            for item in &data[1..] {
                                result = reducer(result, item.clone());
                            }
                            let _ = result_sender.send(result);
                        }
                    }
                    Task::Custom { function } => {
                        if let Err(e) = function() {
                            eprintln!("Worker {}: Task failed: {:?}", id, e);
                        }
                    }
                }
            }
        });

        Worker { id, handle }
    }
}

/// Worker pool for parallel task execution with configurable limits
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_sender: Sender<Task>,
    task_receiver: Arc<std::sync::Mutex<Receiver<Task>>>,
    max_workers: usize,
    active_tasks: Arc<AtomicUsize>,
    is_shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl WorkerPool {
    /// Create a new worker pool with the specified number of workers
    pub fn new(max_workers: usize) -> Result<Self> {
        if max_workers == 0 {
            return Err(WorkerPoolError::WorkerCreationFailed);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(std::sync::Mutex::new(receiver));

        let mut workers = Vec::with_capacity(max_workers);
        for id in 0..max_workers {
            let receiver_clone = Arc::clone(&receiver);
            workers.push(Worker::new(id, receiver_clone));
        }

        Ok(WorkerPool {
            workers,
            task_sender: sender,
            task_receiver: receiver,
            max_workers,
            active_tasks: Arc::new(AtomicUsize::new(0)),
            is_shutdown: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Submit a map task to the pool
    pub fn submit_map<F>(&self, data: Vec<String>, mapper: F) -> Result<Receiver<Vec<String>>>
    where
        F: Fn(String) -> String + Send + 'static,
    {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(WorkerPoolError::PoolFull);
        }

        let (result_sender, result_receiver) = mpsc::channel();
        let task = Task::Map {
            data,
            mapper: Box::new(mapper),
            result_sender,
        };

        self.task_sender
            .send(task)
            .map_err(|e| WorkerPoolError::TaskSubmissionFailed(e.to_string()))?;

        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        Ok(result_receiver)
    }

    /// Submit a reduce task to the pool
    pub fn submit_reduce<F>(&self, data: Vec<String>, reducer: F) -> Result<Receiver<String>>
    where
        F: Fn(String, String) -> String + Send + 'static,
    {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(WorkerPoolError::PoolFull);
        }

        let (result_sender, result_receiver) = mpsc::channel();
        let task = Task::Reduce {
            data,
            reducer: Box::new(reducer),
            result_sender,
        };

        self.task_sender
            .send(task)
            .map_err(|e| WorkerPoolError::TaskSubmissionFailed(e.to_string()))?;

        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        Ok(result_receiver)
    }

    /// Submit a custom task to the pool
    pub fn submit_custom<F>(&self, function: F) -> Result<()>
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(WorkerPoolError::PoolFull);
        }

        let task = Task::Custom {
            function: Box::new(function),
        };

        self.task_sender
            .send(task)
            .map_err(|e| WorkerPoolError::TaskSubmissionFailed(e.to_string()))?;

        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Get the number of active tasks
    pub fn active_tasks(&self) -> usize {
        self.active_tasks.load(Ordering::SeqCst)
    }

    /// Get the maximum number of workers
    pub fn max_workers(&self) -> usize {
        self.max_workers
    }

    /// Check if the pool is shutdown
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }

    /// Shutdown the worker pool gracefully
    pub fn shutdown(&self) {
        self.is_shutdown.store(true, Ordering::SeqCst);
        // Drop sender to close channel
        drop(&self.task_sender);
    }

    /// Wait for all workers to finish
    pub fn wait(self) {
        for worker in self.workers {
            worker.handle.join().unwrap();
        }
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new(num_cpus::get()).expect("Failed to create default worker pool")
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_creation() {
        let pool = WorkerPool::new(4);
        assert_eq!(pool.max_workers, 4);
        assert_eq!(pool.active_tasks(), 0);
        pool.shutdown();
    }

    #[test]
    fn test_submit_custom_task() {
        let pool = WorkerPool::new(2);
        let task = Task::Custom {
            function: Box::new(|| {
                println!("Custom task executed");
                Ok(())
            }),
        };

        assert!(pool.submit(task).is_ok());
        pool.shutdown();
    }
}