//! Implementación del decorador `@controller`
//!
//! El decorador `@controller` marca una clase como controlador REST
//! con una ruta base para todos sus endpoints.

use serde::{Deserialize, Serialize};

/// Metadatos del controlador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerMetadata {
    pub base_path: String,
    pub description: Option<String>,
}

/// Decorador @controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Controller {
    pub metadata: ControllerMetadata,
}

impl Controller {
    pub fn new(base_path: String) -> Self {
        Self {
            metadata: ControllerMetadata {
                base_path,
                description: None,
            },
        }
    }

    /// Valida la configuración del controlador
    pub fn validate(&self) -> Result<(), String> {
        if self.metadata.base_path.is_empty() {
            return Err("Controller base path cannot be empty".to_string());
        }

        if !self.metadata.base_path.starts_with('/') {
            return Err("Controller base path must start with '/'".to_string());
        }

        Ok(())
    }

    /// Obtiene la ruta base del controlador
    pub fn base_path(&self) -> &str {
        &self.metadata.base_path
    }
}

/// Helper para crear controladores fácilmente
pub fn controller(base_path: &str) -> Controller {
    Controller::new(base_path.to_string())
}