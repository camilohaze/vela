//! Provider implementations for dependency injection
//!
//! This module defines the Provider trait and various implementations
//! that control how service instances are created and managed.

use crate::{DIResult, Scope};
use std::any::TypeId;

/// Trait for service providers
///
/// A provider defines how to create instances of a service type.
/// Different implementations provide different creation strategies.
pub trait Provider<T: 'static>: Send + Sync {
    /// Create an instance of the service
    fn provide(&self, container: &crate::container::DIContainer) -> DIResult<T>;

    /// Get the scope of this provider
    fn scope(&self) -> Scope;

    /// Get the service type ID
    fn service_type_id(&self) -> TypeId;
}

/// Singleton provider - creates one shared instance
pub struct SingletonProvider<T, F> {
    factory: F,
    instance: std::sync::RwLock<Option<T>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> SingletonProvider<T, F>
where
    T: 'static + Send + Sync + Clone,
    F: Fn() -> DIResult<T> + Send + Sync,
{
    /// Create a new singleton provider
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            instance: std::sync::RwLock::new(None),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Provider<T> for SingletonProvider<T, F>
where
    T: 'static + Send + Sync + Clone,
    F: Fn() -> DIResult<T> + Send + Sync,
{
    fn provide(&self, _container: &crate::container::DIContainer) -> DIResult<T> {
        {
            let instance = self.instance.read().unwrap();
            if let Some(ref inst) = *instance {
                return Ok(inst.clone());
            }
        }

        {
            let mut instance = self.instance.write().unwrap();
            if let Some(ref inst) = *instance {
                return Ok(inst.clone());
            }

            let new_instance = (self.factory)()?;
            *instance = Some(new_instance.clone());
            Ok(new_instance)
        }
    }

    fn scope(&self) -> Scope {
        Scope::Singleton
    }

    fn service_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// Transient provider - creates new instance every time
pub struct TransientProvider<T, F> {
    factory: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> TransientProvider<T, F>
where
    T: 'static,
    F: Fn() -> DIResult<T> + Send + Sync,
{
    /// Create a new transient provider
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Provider<T> for TransientProvider<T, F>
where
    T: 'static + Send + Sync,
    F: Fn() -> DIResult<T> + Send + Sync,
{
    fn provide(&self, _container: &crate::container::DIContainer) -> DIResult<T> {
        (self.factory)()
    }

    fn scope(&self) -> Scope {
        Scope::Transient
    }

    fn service_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// Factory provider - uses a factory function with container access
pub struct FactoryProvider<T, F> {
    factory: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> FactoryProvider<T, F>
where
    T: 'static,
    F: Fn(&crate::container::DIContainer) -> DIResult<T> + Send + Sync,
{
    /// Create a new factory provider
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Provider<T> for FactoryProvider<T, F>
where
    T: 'static + Send + Sync,
    F: Fn(&crate::container::DIContainer) -> DIResult<T> + Send + Sync,
{
    fn provide(&self, container: &crate::container::DIContainer) -> DIResult<T> {
        (self.factory)(container)
    }

    fn scope(&self) -> Scope {
        Scope::Transient
    }

    fn service_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// Instance provider - provides a pre-created instance
pub struct InstanceProvider<T> {
    instance: T,
}

impl<T> InstanceProvider<T>
where
    T: 'static + Send + Sync + Clone,
{
    /// Create a new instance provider
    pub fn new(instance: T) -> Self {
        Self { instance }
    }
}

impl<T> Provider<T> for InstanceProvider<T>
where
    T: 'static + Send + Sync + Clone,
{
    fn provide(&self, _container: &crate::container::DIContainer) -> DIResult<T> {
        Ok(self.instance.clone())
    }

    fn scope(&self) -> Scope {
        Scope::Singleton
    }

    fn service_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}