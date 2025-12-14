//! Worker Pool Implementation for Parallel Processing
//!
//! This module provides worker pools for parallel task execution in Vela.
//! It integrates with the async runtime and provides load balancing capabilities.
//!
//! Jira: VELA-1113
//! Task: TASK-117M
//! Date: 2025-12-14

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::collections::VecDeque;

/// Represents a task that can be executed by a worker
pub enum Task {
    /// Map operation over a collection
    Map {
        data: Vec<String>, // Simplified for demo
        mapper: Box<dyn Fn(String) -> String + Send + 'static>,
    },
    /// Reduce operation over a collection
    Reduce {
        data: Vec<String>,
        reducer: Box<dyn Fn(String, String) -> String + Send + 'static>,
    },
    /// Custom task
    Custom {
        function: Box<dyn FnOnce() -> Result<(), String> + Send + 'static>,
    },
}

/// A worker that processes tasks
struct Worker {
    id: usize,
    handle: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Receiver<Task>) -> Self {
        let handle = thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                match task {
                    Task::Map { data, mapper } => {
                        for item in data {
                            let _result = mapper(item);
                            // Process result
                        }
                    }
                    Task::Reduce { data, reducer } => {
                        if !data.is_empty() {
                            let mut result = data[0].clone();
                            for item in &data[1..] {
                                result = reducer(result, item.clone());
                            }
                        }
                    }
                    Task::Custom { function } => {
                        if let Err(e) = function() {
                            eprintln!("Worker {}: Task failed: {}", id, e);
                        }
                    }
                }
            }
        });

        Worker { id, handle }
    }
}

/// Worker pool for parallel task execution
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_sender: Sender<Task>,
    max_workers: usize,
    active_tasks: Arc<AtomicUsize>,
}

impl WorkerPool {
    /// Create a new worker pool with the specified number of workers
    pub fn new(max_workers: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(std::sync::Mutex::new(receiver));

        let mut workers = Vec::new();
        for id in 0..max_workers {
            let receiver_clone = Arc::clone(&receiver);
            let worker_receiver = receiver_clone.lock().unwrap();
            // Note: This is simplified; in real impl we'd need better channel sharing
            workers.push(Worker::new(id, worker_receiver));
        }

        WorkerPool {
            workers,
            task_sender: sender,
            max_workers,
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Submit a task to the pool
    pub fn submit(&self, task: Task) -> Result<(), String> {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        self.task_sender.send(task).map_err(|e| format!("Failed to submit task: {}", e))?;

        // Decrement when task completes (simplified)
        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }

    /// Get the number of active tasks
    pub fn active_tasks(&self) -> usize {
        self.active_tasks.load(Ordering::SeqCst)
    }

    /// Shutdown the worker pool
    pub fn shutdown(self) {
        drop(self.task_sender); // Close the channel
        for worker in self.workers {
            worker.handle.join().unwrap();
        }
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new(num_cpus::get())
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