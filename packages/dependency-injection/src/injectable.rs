//! Implementación del decorador `@injectable`
//!
//! El decorador `@injectable` marca una clase como disponible
//! para inyección de dependencias en el contenedor DI.

use serde::{Deserialize, Serialize};
use crate::scope::Scope;

/// Metadatos de inyección
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectableMetadata {
    pub scope: Scope,
}

/// Decorador @injectable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Injectable {
    pub metadata: InjectableMetadata,
}

impl Injectable {
    pub fn new(scope: Scope) -> Self {
        Self {
            metadata: InjectableMetadata { scope },
        }
    }

    /// Crea un injectable con scope Singleton
    pub fn singleton() -> Self {
        Self::new(Scope::Singleton)
    }

    /// Crea un injectable con scope Scoped
    pub fn scoped() -> Self {
        Self::new(Scope::Scoped)
    }

    /// Crea un injectable con scope Transient
    pub fn transient() -> Self {
        Self::new(Scope::Transient)
    }
}

/// Helper para crear injectables fácilmente
pub mod decorators {
    use super::*;

    pub fn injectable(scope: Scope) -> Injectable {
        Injectable::new(scope)
    }

    pub fn singleton() -> Injectable {
        Injectable::singleton()
    }

    pub fn scoped() -> Injectable {
        Injectable::scoped()
    }

    pub fn transient() -> Injectable {
        Injectable::transient()
    }
}