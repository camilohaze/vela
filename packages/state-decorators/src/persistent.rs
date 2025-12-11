//! # @persistent Decorator
//!
//! Decorador para persistencia automática de estado.
//!
//! ## Uso
//!
//! ```vela
//! store UserPreferences {
//!   @persistent(localStorage, key="user_prefs")
//!   state theme: String = "light"
//!
//!   @persistent(sessionStorage, key="temp_data", ttl=3600)
//!   state tempData: String = ""
//!
//!   @persistent(indexedDB, key="large_data", compress=true)
//!   state largeData: LargeObject
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Tipos de almacenamiento soportados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    /// LocalStorage del navegador
    LocalStorage,
    /// SessionStorage del navegador
    SessionStorage,
    /// IndexedDB para datos grandes
    IndexedDB,
    /// Almacenamiento en archivo (desktop)
    FileSystem,
    /// Base de datos SQLite
    SQLite,
}

/// Decorador @persistent para persistencia automática
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persistent {
    /// Tipo de almacenamiento
    pub storage_type: StorageType,
    /// Clave para almacenar el valor
    pub key: String,
    /// Time-to-live en segundos (opcional)
    pub ttl: Option<u64>,
    /// Si debe comprimir los datos
    pub compress: bool,
    /// Serializador personalizado (opcional)
    pub serializer: Option<String>,
    /// Si debe migrar datos antiguos
    pub migrate: bool,
}

impl Persistent {
    /// Crea un nuevo decorador @persistent
    pub fn new(storage_type: StorageType, key: String) -> Self {
        Self {
            storage_type,
            key,
            ttl: None,
            compress: false,
            serializer: None,
            migrate: false,
        }
    }

    /// Establece TTL para expiración automática
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl = Some(ttl_seconds);
        self
    }

    /// Habilita compresión de datos
    pub fn with_compression(mut self) -> Self {
        self.compress = true;
        self
    }

    /// Establece serializador personalizado
    pub fn with_serializer(mut self, serializer: String) -> Self {
        self.serializer = Some(serializer);
        self
    }

    /// Habilita migración de datos antiguos
    pub fn with_migration(mut self) -> Self {
        self.migrate = true;
        self
    }

    /// Valida que el decorador esté correctamente configurado
    pub fn validate(&self) -> Result<(), String> {
        if self.key.is_empty() {
            return Err("Storage key cannot be empty".to_string());
        }

        // Validar caracteres permitidos en la key
        if !self.key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Storage key can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }

        // Validar TTL razonable
        if let Some(ttl) = self.ttl {
            if ttl == 0 {
                return Err("TTL must be greater than 0".to_string());
            }
            if ttl > 365 * 24 * 60 * 60 { // 1 año máximo
                return Err("TTL cannot exceed 1 year".to_string());
            }
        }

        Ok(())
    }

    /// Genera código de persistencia para la propiedad
    pub fn generate_persistence_code(&self, property_name: &str, property_type: &str) -> String {
        let mut code = String::new();

        code.push_str(&format!("  // Persistence setup for {}\n", property_name));

        // Generar código de carga inicial
        code.push_str(&format!(
            "  private fn load_{}() -> {} {{\n",
            property_name, property_type
        ));
        code.push_str(&format!("    // Load from {} with key '{}'\n", self.storage_type_str(), self.key));

        if self.compress {
            code.push_str("    // Decompress data if needed\n");
        }

        if let Some(ttl) = self.ttl {
            code.push_str(&format!("    // Check TTL ({} seconds)\n", ttl));
        }

        code.push_str("    // Return loaded value or default\n");
        code.push_str("  }\n\n");

        // Generar código de guardado
        code.push_str(&format!(
            "  private fn save_{}() -> void {{\n",
            property_name
        ));
        code.push_str(&format!("    // Save to {} with key '{}'\n", self.storage_type_str(), self.key));

        if self.compress {
            code.push_str("    // Compress data before saving\n");
        }

        if let Some(serializer) = &self.serializer {
            code.push_str(&format!("    // Use custom serializer: {}\n", serializer));
        }

        code.push_str("  }\n\n");

        // Generar effect para auto-guardado
        code.push_str(&format!(
            "  effect {{\n    // Auto-save {} when it changes\n    this.save_{}()\n  }}\n",
            property_name, property_name
        ));

        code
    }

    /// Retorna string descriptivo del tipo de almacenamiento
    fn storage_type_str(&self) -> &str {
        match self.storage_type {
            StorageType::LocalStorage => "LocalStorage",
            StorageType::SessionStorage => "SessionStorage",
            StorageType::IndexedDB => "IndexedDB",
            StorageType::FileSystem => "FileSystem",
            StorageType::SQLite => "SQLite",
        }
    }

    /// Verifica si el almacenamiento está disponible en el entorno actual
    pub fn is_storage_available(&self) -> bool {
        match self.storage_type {
            StorageType::LocalStorage | StorageType::SessionStorage => {
                // Verificar si estamos en navegador
                true // Asumir disponible por ahora
            }
            StorageType::IndexedDB => {
                // Verificar soporte de IndexedDB
                true // Asumir disponible por ahora
            }
            StorageType::FileSystem => {
                // Verificar permisos de filesystem
                true // Asumir disponible por ahora
            }
            StorageType::SQLite => {
                // Verificar si SQLite está disponible
                true // Asumir disponible por ahora
            }
        }
    }
}

/// Función helper para crear decoradores @persistent con LocalStorage
pub fn persistent_local(key: &str) -> Persistent {
    Persistent::new(StorageType::LocalStorage, key.to_string())
}

/// Función helper para crear decoradores @persistent con SessionStorage
pub fn persistent_session(key: &str) -> Persistent {
    Persistent::new(StorageType::SessionStorage, key.to_string())
}

/// Función helper para crear decoradores @persistent con IndexedDB
pub fn persistent_indexeddb(key: &str) -> Persistent {
    Persistent::new(StorageType::IndexedDB, key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistent_creation() {
        let persistent = Persistent::new(StorageType::LocalStorage, "user_prefs".to_string());
        assert_eq!(persistent.key, "user_prefs");
        assert!(!persistent.compress);
        assert!(persistent.ttl.is_none());
    }

    #[test]
    fn test_persistent_with_ttl() {
        let persistent = Persistent::new(StorageType::SessionStorage, "temp_data".to_string())
            .with_ttl(3600);
        assert_eq!(persistent.ttl, Some(3600));
    }

    #[test]
    fn test_persistent_validation() {
        let valid_persistent = Persistent::new(StorageType::LocalStorage, "user_prefs".to_string());
        assert!(valid_persistent.validate().is_ok());

        let invalid_key = Persistent::new(StorageType::LocalStorage, "".to_string());
        assert!(invalid_key.validate().is_err());

        let invalid_chars = Persistent::new(StorageType::LocalStorage, "user prefs".to_string());
        assert!(invalid_chars.validate().is_err());

        let invalid_ttl = Persistent::new(StorageType::LocalStorage, "test".to_string())
            .with_ttl(0);
        assert!(invalid_ttl.validate().is_err());
    }

    #[test]
    fn test_persistence_code_generation() {
        let persistent = Persistent::new(StorageType::LocalStorage, "theme".to_string());
        let code = persistent.generate_persistence_code("theme", "String");

        assert!(code.contains("load_theme"));
        assert!(code.contains("save_theme"));
        assert!(code.contains("effect"));
        assert!(code.contains("LocalStorage"));
    }

    #[test]
    fn test_helper_functions() {
        let local = persistent_local("prefs");
        assert!(matches!(local.storage_type, StorageType::LocalStorage));

        let session = persistent_session("temp");
        assert!(matches!(session.storage_type, StorageType::SessionStorage));

        let indexeddb = persistent_indexeddb("large");
        assert!(matches!(indexeddb.storage_type, StorageType::IndexedDB));
    }
}