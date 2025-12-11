//! Implementación del decorador `@middleware`
//!
//! El decorador `@middleware` marca funciones como middleware HTTP
//! que pueden interceptar y modificar requests/responses.

use serde::{Deserialize, Serialize};

/// Tipos de middleware disponibles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MiddlewareType {
    /// Middleware global (se ejecuta en todas las rutas)
    Global,
    /// Middleware de ruta (se ejecuta solo en rutas específicas)
    Route,
    /// Middleware de error (maneja errores HTTP)
    Error,
}

/// Metadatos del middleware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareMetadata {
    pub middleware_type: MiddlewareType,
    pub priority: i32, // Orden de ejecución (menor = primero)
    pub name: Option<String>,
}

/// Decorador @middleware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Middleware {
    pub metadata: MiddlewareMetadata,
}

impl Middleware {
    pub fn new(middleware_type: MiddlewareType) -> Self {
        Self {
            metadata: MiddlewareMetadata {
                middleware_type,
                priority: 0,
                name: None,
            },
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.metadata.priority = priority;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.metadata.name = Some(name);
        self
    }

    /// Valida la configuración del middleware
    pub fn validate(&self) -> Result<(), String> {
        if let Some(name) = &self.metadata.name {
            if name.is_empty() {
                return Err("Middleware name cannot be empty".to_string());
            }
        }

        Ok(())
    }
}

/// Helpers para crear middleware fácilmente
pub mod decorators {
    use super::*;

    pub fn middleware(middleware_type: MiddlewareType) -> Middleware {
        Middleware::new(middleware_type)
    }

    pub fn global_middleware() -> Middleware {
        Middleware::new(MiddlewareType::Global)
    }

    pub fn route_middleware() -> Middleware {
        Middleware::new(MiddlewareType::Route)
    }

    pub fn error_middleware() -> Middleware {
        Middleware::new(MiddlewareType::Error)
    }
}