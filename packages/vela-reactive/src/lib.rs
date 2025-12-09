//! # Vela Reactive System
//!
//! Reactive primitives for Vela's UI framework.
//!
//! This crate provides the foundational reactive system including:
//! - Signals for reactive state management
//! - Computed values that automatically update
//! - Effects for side effects
//! - Reactive collections and primitives

/// A reactive signal that holds a value and notifies subscribers of changes
pub struct Signal<T> {
    value: T,
}

impl<T> Signal<T> {
    /// Create a new signal with an initial value
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// Get the current value of the signal
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Set a new value for the signal
    pub fn set(&mut self, value: T) {
        self.value = value;
        // TODO: Notify subscribers
    }
}

/// A computed value that automatically updates when its dependencies change
pub struct Computed<T> {
    value: T,
}

impl<T> Computed<T> {
    /// Create a new computed value
    pub fn new<F>(compute_fn: F) -> Self
    where
        F: Fn() -> T,
    {
        Self {
            value: compute_fn(),
        }
    }

    /// Get the current computed value
    pub fn get(&self) -> &T {
        &self.value
    }
}

/// An effect that runs side effects when reactive dependencies change
pub struct Effect {
    // TODO: Implement effect system
}

impl Effect {
    /// Create a new effect
    pub fn new<F>(_effect_fn: F) -> Self
    where
        F: Fn() + 'static,
    {
        Self {}
    }
}

/// Initialize the reactive system
pub fn init() {
    // Initialize reactive runtime
}