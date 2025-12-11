//! Implementación del decorador `@guard`
//!
//! El decorador `@guard` marca funciones como guards de autorización
//! que pueden permitir o denegar acceso a rutas/endpoints.

use serde::{Deserialize, Serialize};

/// Tipos de guards disponibles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GuardType {
    /// Guard de autenticación (verifica si usuario está logueado)
    Auth,
    /// Guard de autorización (verifica permisos específicos)
    Role,
    /// Guard custom (lógica personalizada)
    Custom,
}

/// Metadatos del guard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardMetadata {
    pub guard_type: GuardType,
    pub roles: Vec<String>, // Para guards de rol
    pub name: Option<String>,
}

/// Decorador @guard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guard {
    pub metadata: GuardMetadata,
}

impl Guard {
    pub fn new(guard_type: GuardType) -> Self {
        Self {
            metadata: GuardMetadata {
                guard_type,
                roles: Vec::new(),
                name: None,
            },
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.metadata.roles = roles;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.metadata.name = Some(name);
        self
    }

    /// Valida la configuración del guard
    pub fn validate(&self) -> Result<(), String> {
        match self.metadata.guard_type {
            GuardType::Role => {
                if self.metadata.roles.is_empty() {
                    return Err("Role guard must specify at least one role".to_string());
                }
            }
            GuardType::Custom => {
                if self.metadata.name.is_none() {
                    return Err("Custom guard must have a name".to_string());
                }
            }
            _ => {}
        }

        if let Some(name) = &self.metadata.name {
            if name.is_empty() {
                return Err("Guard name cannot be empty".to_string());
            }
        }

        Ok(())
    }

    /// Verifica si el guard permite acceso
    /// (En implementación real, esto consultaría el contexto de seguridad)
    pub fn can_activate(&self, _context: &GuardContext) -> bool {
        // Implementación mock - en producción esto sería más complejo
        match self.metadata.guard_type {
            GuardType::Auth => true, // Simular usuario autenticado
            GuardType::Role => true, // Simular permisos válidos
            GuardType::Custom => true, // Simular guard custom válido
        }
    }
}

/// Contexto de ejecución del guard
#[derive(Debug, Clone)]
pub struct GuardContext {
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub path: String,
    pub method: String,
}

impl Default for GuardContext {
    fn default() -> Self {
        Self {
            user_id: None,
            roles: Vec::new(),
            path: "/".to_string(),
            method: "GET".to_string(),
        }
    }
}

/// Helpers para crear guards fácilmente
pub mod decorators {
    use super::*;

    pub fn guard(guard_type: GuardType) -> Guard {
        Guard::new(guard_type)
    }

    pub fn auth_guard() -> Guard {
        Guard::new(GuardType::Auth)
    }

    pub fn role_guard(roles: Vec<&str>) -> Guard {
        Guard::new(GuardType::Role).with_roles(
            roles.into_iter().map(|s| s.to_string()).collect()
        )
    }

    pub fn custom_guard(name: &str) -> Guard {
        Guard::new(GuardType::Custom).with_name(name.to_string())
    }
}