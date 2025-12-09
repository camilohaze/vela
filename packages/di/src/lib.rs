//! # Vela DI
//!
//! Sistema de Dependency Injection para aplicaciones Rust.
//!
//! Este crate proporciona un sistema completo de inyección de dependencias
//! con soporte para múltiples scopes (Singleton, Scoped, Transient),
//! resolución automática de dependencias y operaciones thread-safe.
//!
//! # Example
//!
//! ```rust
//! use di::*;
//!
//! // Create a container
//! let container = DIContainer::new();
//!
//! // Register services
//! container.register_singleton(|| Ok(String::from("database_url"))).unwrap();
//! container.register_transient(|| Ok(42i32)).unwrap();
//!
//! // Resolve services
//! let db_url: String = container.resolve().unwrap();
//! let number: i32 = container.resolve().unwrap();
//!
//! assert_eq!(db_url, "database_url");
//! assert_eq!(number, 42);
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