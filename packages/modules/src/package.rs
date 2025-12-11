//! Implementación del decorador `@package`
//!
//! El decorador `@package` marca un módulo como paquete publicable
//! que puede ser distribuido y consumido por otros proyectos.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadatos de un paquete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub dependencies: HashMap<String, String>,
}

/// Decorador @package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub metadata: PackageMetadata,
}

impl Package {
    pub fn new(metadata: PackageMetadata) -> Self {
        Self { metadata }
    }

    /// Valida que el paquete tenga todos los campos requeridos
    pub fn validate(&self) -> Result<(), String> {
        if self.metadata.name.is_empty() {
            return Err("Package name cannot be empty".to_string());
        }

        if self.metadata.version.is_empty() {
            return Err("Package version cannot be empty".to_string());
        }

        Ok(())
    }
}

/// Helper para crear paquetes fácilmente
pub fn package(name: &str, version: &str) -> Package {
    Package::new(PackageMetadata {
        name: name.to_string(),
        version: version.to_string(),
        description: None,
        authors: Vec::new(),
        license: None,
        repository: None,
        dependencies: HashMap::new(),
    })
}