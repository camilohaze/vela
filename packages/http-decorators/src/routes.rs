//! Implementación de decoradores de rutas HTTP
//!
//! Decoradores para definir endpoints HTTP:
//! `@get`, `@post`, `@put`, `@patch`, `@delete`

use serde::{Deserialize, Serialize};

/// Métodos HTTP soportados
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::DELETE => write!(f, "DELETE"),
        }
    }
}

/// Metadatos de una ruta HTTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMetadata {
    pub method: HttpMethod,
    pub path: String,
    pub description: Option<String>,
}

/// Decorador base para rutas HTTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub metadata: RouteMetadata,
}

impl Route {
    pub fn new(method: HttpMethod, path: String) -> Self {
        Self {
            metadata: RouteMetadata {
                method,
                path,
                description: None,
            },
        }
    }

    /// Valida la configuración de la ruta
    pub fn validate(&self) -> Result<(), String> {
        if self.metadata.path.is_empty() {
            return Err("Route path cannot be empty".to_string());
        }

        // Validar parámetros de ruta
        if self.metadata.path.contains(':') {
            // Contar parámetros nombrados
            let param_count = self.metadata.path.matches(':').count();
            if param_count == 0 {
                return Err("Route path contains ':' but no parameter name".to_string());
            }
        }

        Ok(())
    }

    /// Obtiene el método HTTP
    pub fn method(&self) -> &HttpMethod {
        &self.metadata.method
    }

    /// Obtiene la ruta relativa
    pub fn path(&self) -> &str {
        &self.metadata.path
    }
}

/// Decoradores específicos para cada método HTTP
pub mod decorators {
    use super::*;

    pub fn get(path: &str) -> Route {
        Route::new(HttpMethod::GET, path.to_string())
    }

    pub fn post(path: &str) -> Route {
        Route::new(HttpMethod::POST, path.to_string())
    }

    pub fn put(path: &str) -> Route {
        Route::new(HttpMethod::PUT, path.to_string())
    }

    pub fn patch(path: &str) -> Route {
        Route::new(HttpMethod::PATCH, path.to_string())
    }

    pub fn delete(path: &str) -> Route {
        Route::new(HttpMethod::DELETE, path.to_string())
    }
}