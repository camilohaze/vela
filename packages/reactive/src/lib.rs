//! # Vela Reactive System
//!
//! Reactive primitives for building reactive applications in Vela.
//!
//! This crate provides the core reactive system including:
//! - `Signal<T>`: Mutable reactive state
//! - `Computed<T>`: Derived reactive values
//! - `Effect`: Side effects that run when dependencies change
//! - `ReactiveGraph`: Dependency tracking and propagation
//! - `Watch`, `Batch`, `Scheduler`: Advanced reactive utilities

pub mod signal;
pub mod computed;
pub mod effect;
pub mod graph;
pub mod watch;
pub mod batch;
pub mod scheduler;
pub mod optimization;

pub use signal::{Signal, signal};
pub use computed::Computed;
pub use effect::Effect;
pub use watch::{Watch, watch, watch_with_options};
pub use batch::{Batch, batch, init_global_batch, global_batch};
pub use graph::{ReactiveGraph, ReactiveNode, NodeType, NodeState};
pub use scheduler::{ReactiveScheduler, SchedulerPriority};
pub use optimization::{SignalGraphAnalyzer, OptimizationStats, MemoizedSignal, LazySignal};