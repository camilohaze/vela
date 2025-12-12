"""
Config Management System para Vela

Implementación de: VELA-609
Historia: VELA-609
Fecha: 2025-12-11

Descripción:
Sistema de configuración type-safe con múltiples fuentes y validación.
Esta implementación resuelve la necesidad de config management en microservicios.
"""

use std::collections::HashMap;
use std::fs;
use std::env;
use serde::{Deserialize, Serialize};

/// Fuentes de configuración en orden de prioridad (menor a mayor)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigSource {
    File(String),      // Archivo JSON/YAML
    Environment,       // Variables de entorno
    Consul(String),    // Consul key-value store
    Vault(String),     // HashiCorp Vault
}

/// Configuración cargada con metadata
#[derive(Debug, Clone)]
pub struct ConfigValue {
    pub value: String,
    pub source: ConfigSource,
    pub profile: Option<String>,
}

/// Loader principal de configuración
pub struct ConfigLoader {
    sources: Vec<ConfigSource>,
    profile: Option<String>,
    cache: HashMap<String, ConfigValue>,
}

impl ConfigLoader {
    /// Crear nuevo ConfigLoader con fuentes por defecto
    pub fn new() -> Self {
        Self {
            sources: vec![
                ConfigSource::File("config.json".to_string()),
                ConfigSource::Environment,
            ],
            profile: None,
            cache: HashMap::new(),
        }
    }

    /// Agregar fuente de configuración
    pub fn add_source(mut self, source: ConfigSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Establecer perfil (dev, staging, prod)
    pub fn with_profile(mut self, profile: String) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Cargar configuración desde todas las fuentes
    pub fn load(&mut self) -> Result<(), ConfigError> {
        self.cache.clear();

        for source in &self.sources {
            match self.load_from_source(source) {
                Ok(values) => {
                    // Valores de mayor prioridad sobrescriben
                    for (key, value) in values {
                        self.cache.insert(key, ConfigValue {
                            value,
                            source: source.clone(),
                            profile: self.profile.clone(),
                        });
                    }
                }
                Err(e) => {
                    // Log error pero continua con otras fuentes
                    eprintln!("Error loading from {:?}: {}", source, e);
                }
            }
        }

        Ok(())
    }

    /// Obtener valor de configuración
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.cache.get(key)
    }

    /// Obtener valor como String
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).map(|v| v.value.clone())
    }

    /// Obtener valor como i64
    pub fn get_int(&self, key: &str) -> Option<Result<i64, ConfigError>> {
        self.get_string(key).map(|s| s.parse().map_err(ConfigError::ParseInt))
    }

    /// Obtener valor como bool
    pub fn get_bool(&self, key: &str) -> Option<Result<bool, ConfigError>> {
        self.get_string(key).map(|s| s.parse().map_err(ConfigError::ParseBool))
    }

    /// Cargar desde una fuente específica
    fn load_from_source(&self, source: &ConfigSource) -> Result<HashMap<String, String>, ConfigError> {
        match source {
            ConfigSource::File(path) => self.load_from_file(path),
            ConfigSource::Environment => self.load_from_env(),
            ConfigSource::Consul(_) => Err(ConfigError::NotImplemented("Consul support")),
            ConfigSource::Vault(_) => Err(ConfigError::NotImplemented("Vault support")),
        }
    }

    /// Cargar desde archivo JSON
    fn load_from_file(&self, path: &str) -> Result<HashMap<String, String>, ConfigError> {
        let content = fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        fn flatten_json(prefix: String, value: &serde_json::Value, result: &mut HashMap<String, String>) {
            match value {
                serde_json::Value::Object(obj) => {
                    for (k, v) in obj {
                        let new_prefix = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                        flatten_json(new_prefix, v, result);
                    }
                }
                serde_json::Value::Array(arr) => {
                    for (i, v) in arr.iter().enumerate() {
                        let new_prefix = format!("{}[{}]", prefix, i);
                        flatten_json(new_prefix, v, result);
                    }
                }
                _ => {
                    result.insert(prefix, value.to_string().trim_matches('"').to_string());
                }
            }
        }

        let mut result = HashMap::new();
        flatten_json(String::new(), &json, &mut result);
        Ok(result)
    }

    /// Cargar desde variables de entorno
    fn load_from_env(&self) -> Result<HashMap<String, String>, ConfigError> {
        let mut result = HashMap::new();

        for (key, value) in env::vars() {
            if key.starts_with("VELA_") || key.starts_with("APP_") {
                result.insert(key.to_lowercase(), value);
            }
        }

        Ok(result)
    }
}

/// Errores de configuración
#[derive(Debug, Clone)]
pub enum ConfigError {
    Io(String),
    Json(String),
    ParseInt(String),
    ParseBool(String),
    NotImplemented(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Io(msg) => write!(f, "IO Error: {}", msg),
            ConfigError::Json(msg) => write!(f, "JSON Error: {}", msg),
            ConfigError::ParseInt(msg) => write!(f, "Parse Int Error: {}", msg),
            ConfigError::ParseBool(msg) => write!(f, "Parse Bool Error: {}", msg),
            ConfigError::NotImplemented(feature) => write!(f, "Not Implemented: {}", feature),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::Json(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_creation() {
        let loader = ConfigLoader::new();
        assert_eq!(loader.sources.len(), 2);
        assert!(loader.profile.is_none());
    }

    #[test]
    fn test_add_source() {
        let loader = ConfigLoader::new()
            .add_source(ConfigSource::Consul("localhost:8500".to_string()));

        assert_eq!(loader.sources.len(), 3);
    }

    #[test]
    fn test_with_profile() {
        let loader = ConfigLoader::new()
            .with_profile("dev".to_string());

        assert_eq!(loader.profile, Some("dev".to_string()));
    }

    #[test]
    fn test_load_from_env() {
        // Set test env var
        env::set_var("VELA_TEST_KEY", "test_value");

        let loader = ConfigLoader::new();
        let result = loader.load_from_env().unwrap();

        assert_eq!(result.get("vela_test_key"), Some(&"test_value".to_string()));

        // Clean up
        env::remove_var("VELA_TEST_KEY");
    }

    #[test]
    fn test_get_methods() {
        let mut loader = ConfigLoader::new();
        loader.cache.insert("test_key".to_string(), ConfigValue {
            value: "42".to_string(),
            source: ConfigSource::Environment,
            profile: None,
        });

        assert_eq!(loader.get_string("test_key"), Some("42".to_string()));
        assert_eq!(loader.get_int("test_key").unwrap().unwrap(), 42);
    }
}