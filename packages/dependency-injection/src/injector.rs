//! Implementación del decorador `@inject`
//!
//! El decorador `@inject` marca parámetros y propiedades
//! para inyección automática de dependencias.

use serde::{Deserialize, Serialize};

/// Metadatos de inyección
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectMetadata {
    pub service_type: String,
    pub optional: bool,
}

/// Decorador @inject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inject {
    pub metadata: InjectMetadata,
}

impl Inject {
    pub fn new(service_type: String) -> Self {
        Self {
            metadata: InjectMetadata {
                service_type,
                optional: false,
            },
        }
    }

    pub fn optional(service_type: String) -> Self {
        Self {
            metadata: InjectMetadata {
                service_type,
                optional: true,
            },
        }
    }
}

/// Helper para crear inyecciones fácilmente
pub mod decorators {
    use super::*;

    pub fn inject(service_type: &str) -> Inject {
        Inject::new(service_type.to_string())
    }

    pub fn inject_optional(service_type: &str) -> Inject {
        Inject::optional(service_type.to_string())
    }
}

/// Función helper para inyección en tiempo de ejecución
/// (equivalente a inject() en otros frameworks)
pub fn inject<T: 'static + Default>() -> T {
    // En una implementación real, esto consultaría el contenedor DI global
    // Por ahora, retornamos un valor por defecto
    T::default()
}