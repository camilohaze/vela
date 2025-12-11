//! Implementación del decorador `@library`
//!
//! El decorador `@library` marca un módulo como biblioteca interna
//! reutilizable dentro del mismo proyecto.

use serde::{Deserialize, Serialize};

/// Metadatos de una biblioteca
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetadata {
    pub name: String,
    pub description: Option<String>,
    pub internal: bool, // Siempre true para bibliotecas
}

/// Decorador @library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub metadata: LibraryMetadata,
}

impl Library {
    pub fn new(metadata: LibraryMetadata) -> Self {
        Self { metadata }
    }

    /// Valida que la biblioteca tenga configuración correcta
    pub fn validate(&self) -> Result<(), String> {
        if self.metadata.name.is_empty() {
            return Err("Library name cannot be empty".to_string());
        }

        if !self.metadata.internal {
            return Err("Libraries must be marked as internal".to_string());
        }

        Ok(())
    }
}

/// Helper para crear bibliotecas fácilmente
pub fn library(name: &str) -> Library {
    Library::new(LibraryMetadata {
        name: name.to_string(),
        description: None,
        internal: true,
    })
}