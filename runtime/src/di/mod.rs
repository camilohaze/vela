//! Dependency Injection System
//!
//! This crate provides a comprehensive dependency injection system for Rust
//! applications. It supports multiple scopes (Singleton, Scoped, Transient),
//! automatic dependency resolution, and thread-safe operations.
//!
//! # Example
//!
//! ```rust
//! use vela_runtime::di::*;
//!
//! // Create a container
//! let container = DIContainer::new();
//!
//! // Register services
//! container.register_singleton(|| Ok(DatabaseConnection::new())).unwrap();
//! container.register_transient(|| Ok(UserService::new())).unwrap();
//!
//! // Resolve services
//! let user_service: UserService = container.resolve().unwrap();
//! ```

pub mod container;
pub mod error;
pub mod provider;
pub mod resolver;
pub mod scope;

// Re-export main types
pub use container::DIContainer;
pub use error::{DIError, DIResult};
pub use provider::Provider;
pub use resolver::{DependencyResolver, Injectable, AutoResolvable};
pub use scope::{Scope, ScopeContext};