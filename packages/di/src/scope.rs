//! Scope management for dependency injection
//!
//! This module defines the different scopes available for service lifetimes
//! and provides utilities for managing scoped instances.

use std::fmt;

/// Defines the lifetime scope of a service instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Single instance shared across the entire application
    Singleton,

    /// One instance per scope (e.g., per web request)
    Scoped,

    /// New instance created every time the service is requested
    Transient,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Singleton => write!(f, "Singleton"),
            Scope::Scoped => write!(f, "Scoped"),
            Scope::Transient => write!(f, "Transient"),
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Scope::Transient
    }
}

/// Scope context for managing scoped instances
///
/// This is used internally by the DI container to track instances
/// that belong to a specific scope (like a web request).
#[derive(Debug)]
pub struct ScopeContext {
    instances: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl ScopeContext {
    /// Create a new empty scope context
    pub fn new() -> Self {
        Self {
            instances: std::collections::HashMap::new(),
        }
    }

    /// Get or create a scoped instance
    pub fn get_or_create<T, F>(&mut self, factory: F) -> &T
    where
        T: 'static + Send + Sync,
        F: FnOnce() -> T,
    {
        let type_id = std::any::TypeId::of::<T>();

        self.instances
            .entry(type_id)
            .or_insert_with(|| Box::new(factory()))
            .downcast_ref::<T>()
            .expect("Type mismatch in scoped instance")
    }

    /// Clear all instances in this scope
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// Check if a type is already instantiated in this scope
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = std::any::TypeId::of::<T>();
        self.instances.contains_key(&type_id)
    }
}

impl Default for ScopeContext {
    fn default() -> Self {
        Self::new()
    }
}