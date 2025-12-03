//! Dependency resolution engine
//!
//! This module provides the logic for automatically resolving dependencies
//! by analyzing constructor parameters and recursively resolving required services.

use super::{DIError, DIResult, DIContainer};
use std::any::TypeId;

/// Engine for resolving dependencies automatically
pub struct DependencyResolver {
    resolution_stack: Vec<TypeId>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            resolution_stack: Vec::new(),
        }
    }

    /// Resolve a service with automatic dependency injection
    ///
    /// This method attempts to create an instance of T by automatically
    /// resolving its dependencies. It uses constructor injection based
    /// on the service's registered dependencies.
    pub fn resolve<T>(&mut self, container: &DIContainer) -> DIResult<T>
    where
        T: 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();

        // Check for circular dependencies
        if self.resolution_stack.contains(&type_id) {
            return Err(DIError::CircularDependency {
                service_chain: self.get_service_chain_names(),
            });
        }

        // Add to resolution stack
        self.resolution_stack.push(type_id);

        // Resolve the service
        let result = self.resolve_service::<T>(container);

        // Remove from resolution stack
        self.resolution_stack.pop();

        result
    }

    /// Internal method to resolve a service
    fn resolve_service<T>(&self, container: &DIContainer) -> DIResult<T>
    where
        T: 'static + Send + Sync,
    {
        // Try to get the service from the container
        container.resolve_service::<T>()
    }

    /// Get the current resolution stack as service names for error reporting
    fn get_service_chain_names(&self) -> Vec<String> {
        // This is a simplified implementation
        // In a real implementation, you'd maintain a mapping of TypeId to names
        self.resolution_stack
            .iter()
            .map(|type_id| format!("{:?}", type_id))
            .collect()
    }

    /// Check if we're currently resolving a service
    pub fn is_resolving(&self, type_id: TypeId) -> bool {
        self.resolution_stack.contains(&type_id)
    }

    /// Get the current resolution depth
    pub fn resolution_depth(&self) -> usize {
        self.resolution_stack.len()
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be automatically resolved
///
/// This trait is implemented for types that support constructor injection.
/// It's used by the dependency resolver to create instances automatically.
pub trait Injectable: 'static + Send + Sync {
    /// Create a new instance with automatic dependency injection
    fn inject(container: &DIContainer) -> DIResult<Self>
    where
        Self: Sized;
}

/// Macro to implement Injectable for types with constructor injection
///
/// This macro generates the necessary code to resolve dependencies
/// from the DI container based on constructor parameters.
///
/// # Example
/// ```rust,ignore
/// struct UserService {
///     repository: UserRepository,
///     cache: Cache,
/// }
///
/// impl_injectable!(UserService, repository: UserRepository, cache: Cache);
/// ```
#[macro_export]
macro_rules! impl_injectable {
    ($struct_name:ident, $($field:ident: $field_type:ty),* $(,)?) => {
        impl $crate::di::resolver::Injectable for $struct_name {
            fn inject(container: &DIContainer) -> DIResult<Self> {
                Ok(Self {
                    $(
                        $field: container.resolve()?,
                    )*
                })
            }
        }
    };
}

/// Helper trait for automatic dependency resolution
///
/// Types that implement this trait can be resolved automatically
/// by the DI container without explicit registration.
pub trait AutoResolvable: Injectable {}

/// Blanket implementation for types that are Injectable
impl<T> AutoResolvable for T where T: Injectable {}