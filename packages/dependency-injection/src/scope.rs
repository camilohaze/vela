//! Scopes para Dependency Injection
//!
//! Define los diferentes scopes de vida útil para los servicios
//! inyectables en el contenedor DI.

use serde::{Deserialize, Serialize};

/// Scopes disponibles para servicios inyectables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Scope {
    /// Una sola instancia compartida en toda la aplicación
    Singleton,
    /// Una instancia por scope (ej: por request HTTP)
    Scoped,
    /// Nueva instancia cada vez que se solicita
    Transient,
}

impl Default for Scope {
    fn default() -> Self {
        Scope::Singleton
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Singleton => write!(f, "Singleton"),
            Scope::Scoped => write!(f, "Scoped"),
            Scope::Transient => write!(f, "Transient"),
        }
    }
}