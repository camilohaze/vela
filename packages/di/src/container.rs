//! Dependency Injection Container
//!
//! This module implements the main DI container that manages service
//! registration and resolution.

use crate::{DIError, DIResult, Scope, provider::Provider};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;

/// Service descriptor containing metadata about a registered service
pub struct ServiceDescriptor {
    /// The provider function for creating service instances
    pub provider_fn: Box<dyn Fn(&DIContainer) -> DIResult<Box<dyn Any + Send + Sync>> + Send + Sync>,
    /// The scope of the service
    pub scope: Scope,
    /// Dependencies required by this service
    pub dependencies: Vec<TypeId>,
}

impl ServiceDescriptor {
    /// Create a new service descriptor
    pub fn new<P, T>(provider: P, scope: Scope, dependencies: Vec<TypeId>) -> Self
    where
        P: Provider<T> + 'static,
        T: 'static + Send + Sync,
    {
        let provider_fn = Box::new(move |container: &DIContainer| {
            provider.provide(container).map(|instance| Box::new(instance) as Box<dyn Any + Send + Sync>)
        });

        Self {
            provider_fn,
            scope,
            dependencies,
        }
    }
}

/// The main Dependency Injection container
///
/// This container manages the registration and resolution of services.
/// It supports different scopes (Singleton, Scoped, Transient) and
/// automatic dependency resolution.
pub struct DIContainer {
    /// Registered services
    services: RwLock<HashMap<TypeId, ServiceDescriptor>>,
    /// Singleton instances
    singletons: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    /// Scoped instances (per scope context)
    scoped_instances: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl DIContainer {
    /// Create a new empty DI container
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            singletons: RwLock::new(HashMap::new()),
            scoped_instances: RwLock::new(HashMap::new()),
        }
    }

    /// Register a service with a custom provider
    ///
    /// # Arguments
    /// * `provider` - The provider that creates service instances
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn register_provider<T, P>(&self, provider: P) -> DIResult<()>
    where
        T: 'static + Send + Sync,
        P: Provider<T> + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();

        if services.contains_key(&type_id) {
            return Err(DIError::ServiceAlreadyRegistered {
                service_type: std::any::type_name::<T>().to_string(),
            });
        }

        let scope = provider.scope();
        let descriptor = ServiceDescriptor::new(provider, scope, Vec::new());
        services.insert(type_id, descriptor);

        Ok(())
    }

    /// Register a singleton service
    ///
    /// The service will be created once and shared across all resolutions.
    ///
    /// # Type Parameters
    /// * `T` - The service type to register
    /// * `F` - The factory function type
    ///
    /// # Arguments
    /// * `factory` - Function that creates the service instance
    pub fn register_singleton<T, F>(&self, factory: F) -> DIResult<()>
    where
        T: 'static + Send + Sync + Clone,
        F: Fn() -> DIResult<T> + Send + Sync + 'static,
    {
        use crate::provider::SingletonProvider;
        let provider = SingletonProvider::new(factory);
        self.register_provider::<T, _>(provider)
    }

    /// Register a transient service
    ///
    /// A new instance will be created every time the service is resolved.
    ///
    /// # Type Parameters
    /// * `T` - The service type to register
    /// * `F` - The factory function type
    ///
    /// # Arguments
    /// * `factory` - Function that creates the service instance
    pub fn register_transient<T, F>(&self, factory: F) -> DIResult<()>
    where
        T: 'static + Send + Sync,
        F: Fn() -> DIResult<T> + Send + Sync + 'static,
    {
        use crate::provider::TransientProvider;
        let provider = TransientProvider::new(factory);
        self.register_provider::<T, _>(provider)
    }

    /// Register a service with a factory function
    ///
    /// The factory function has access to the container for resolving dependencies.
    ///
    /// # Type Parameters
    /// * `T` - The service type to register
    /// * `F` - The factory function type
    ///
    /// # Arguments
    /// * `factory` - Function that creates the service instance with container access
    pub fn register_factory<T, F>(&self, factory: F) -> DIResult<()>
    where
        T: 'static + Send + Sync,
        F: Fn(&DIContainer) -> DIResult<T> + Send + Sync + 'static,
    {
        use crate::provider::FactoryProvider;
        let provider = FactoryProvider::new(factory);
        self.register_provider::<T, _>(provider)
    }

    /// Register a pre-created instance as a singleton
    ///
    /// # Arguments
    /// * `instance` - The instance to register
    pub fn register_instance<T>(&self, instance: T) -> DIResult<()>
    where
        T: 'static + Send + Sync + Clone,
    {
        use crate::provider::InstanceProvider;
        let provider = InstanceProvider::new(instance);
        self.register_provider::<T, _>(provider)
    }

    /// Resolve a service instance
    ///
    /// This method resolves a service by finding its provider and creating
    /// an instance according to the provider's strategy.
    ///
    /// # Type Parameters
    /// * `T` - The service type to resolve
    ///
    /// # Returns
    /// The resolved service instance or an error
    pub fn resolve<T>(&self) -> DIResult<T>
    where
        T: 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        let services = self.services.read().unwrap();

        let descriptor = services.get(&type_id).ok_or_else(|| {
            DIError::ServiceNotRegistered {
                service_type: std::any::type_name::<T>().to_string(),
            }
        })?;

        // Call the provider function and downcast the result
        let instance_box = (descriptor.provider_fn)(self)?;
        let instance = instance_box.downcast::<T>().map_err(|_| {
            DIError::ResolutionFailed {
                service_type: std::any::type_name::<T>().to_string(),
                cause: "Provider returned wrong type".to_string(),
            }
        })?;

        Ok(*instance)
    }

    /// Internal method to resolve services (used by providers)
    pub(crate) fn resolve_service<T>(&self) -> DIResult<T>
    where
        T: 'static + Send + Sync,
    {
        self.resolve::<T>()
    }

    /// Check if a service is registered
    ///
    /// # Type Parameters
    /// * `T` - The service type to check
    ///
    /// # Returns
    /// true if the service is registered, false otherwise
    pub fn is_registered<T>(&self) -> bool
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let services = self.services.read().unwrap();
        services.contains_key(&type_id)
    }

    /// Get all registered service types
    ///
    /// This is mainly useful for debugging and introspection.
    ///
    /// # Returns
    /// Vector of type names for all registered services
    pub fn get_registered_services(&self) -> Vec<String> {
        let services = self.services.read().unwrap();
        services
            .keys()
            .map(|type_id| format!("{:?}", type_id))
            .collect()
    }

    /// Clear all singleton instances
    ///
    /// This can be useful for testing or resetting the container state.
    pub fn clear_singletons(&self) {
        let mut singletons = self.singletons.write().unwrap();
        singletons.clear();
    }

    /// Clear all scoped instances
    pub fn clear_scoped(&self) {
        let mut scoped = self.scoped_instances.write().unwrap();
        scoped.clear();
    }
}

impl Default for DIContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestService {
        value: i32,
    }

    #[test]
    fn test_register_and_resolve_singleton() {
        let container = DIContainer::new();

        container
            .register_singleton(|| Ok(TestService { value: 42 }))
            .unwrap();

        let service1 = container.resolve::<TestService>().unwrap();
        let service2 = container.resolve::<TestService>().unwrap();

        assert_eq!(service1.value, 42);
        assert_eq!(service2.value, 42);
        assert_eq!(service1, service2); // Same instance
    }

    #[test]
    fn test_register_and_resolve_transient() {
        let container = DIContainer::new();

        container
            .register_transient(|| Ok(TestService { value: 42 }))
            .unwrap();

        let service1 = container.resolve::<TestService>().unwrap();
        let service2 = container.resolve::<TestService>().unwrap();

        assert_eq!(service1.value, 42);
        assert_eq!(service2.value, 42);
        // Different instances (transient)
    }

    #[test]
    fn test_service_not_registered() {
        let container = DIContainer::new();

        let result = container.resolve::<TestService>();
        assert!(matches!(result, Err(DIError::ServiceNotRegistered { .. })));
    }

    #[test]
    fn test_register_instance() {
        let container = DIContainer::new();
        let instance = TestService { value: 123 };

        container.register_instance(instance).unwrap();

        let resolved = container.resolve::<TestService>().unwrap();
        assert_eq!(resolved.value, 123);
    }

    #[test]
    fn test_is_registered() {
        let container = DIContainer::new();

        assert!(!container.is_registered::<TestService>());

        container
            .register_singleton(|| Ok(TestService { value: 42 }))
            .unwrap();

        assert!(container.is_registered::<TestService>());
    }
}