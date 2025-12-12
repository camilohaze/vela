//! Config Management System para Vela
//!
//! Implementación de: VELA-609
//! Historia: VELA-609
//! Fecha: 2025-12-11
//!
//! Descripción:
//! Sistema de configuración type-safe con múltiples fuentes y validación.
//! Esta implementación resuelve la necesidad de config management en microservicios.

use std::collections::HashMap;
use std::fs;
use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::broadcast;

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

/// Trait para validadores de configuración
pub trait ConfigValidator: Send + Sync {
    fn validate(&self, key: &str, value: &str) -> Result<(), ValidationError>;
    fn clone_box(&self) -> Box<dyn ConfigValidator + Send + Sync>;
}

impl Clone for Box<dyn ConfigValidator + Send + Sync> {
    fn clone(&self) -> Box<dyn ConfigValidator + Send + Sync> {
        self.clone_box()
    }
}

/// Validador requerido
#[derive(Clone)]
pub struct RequiredValidator;

impl ConfigValidator for RequiredValidator {
    fn validate(&self, key: &str, value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::RequiredField(key.to_string()));
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ConfigValidator + Send + Sync> {
        Box::new(self.clone())
    }
}

/// Validador de rango numérico
#[derive(Clone)]
pub struct RangeValidator {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl ConfigValidator for RangeValidator {
    fn validate(&self, key: &str, value: &str) -> Result<(), ValidationError> {
        let num: i64 = value.parse().map_err(|_| ValidationError::InvalidType(key.to_string(), "number".to_string()))?;

        if let Some(min) = self.min {
            if num < min {
                return Err(ValidationError::OutOfRange(key.to_string(), num, min, self.max));
            }
        }

        if let Some(max) = self.max {
            if num > max {
                return Err(ValidationError::OutOfRange(key.to_string(), num, self.min.unwrap_or(i64::MIN), Some(max)));
            }
        }

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ConfigValidator + Send + Sync> {
        Box::new(self.clone())
    }
}

/// Validador de email básico
#[derive(Clone)]
pub struct EmailValidator;

impl ConfigValidator for EmailValidator {
    fn validate(&self, key: &str, value: &str) -> Result<(), ValidationError> {
        if !value.contains('@') || !value.contains('.') {
            return Err(ValidationError::InvalidFormat(key.to_string(), "email".to_string()));
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ConfigValidator + Send + Sync> {
        Box::new(self.clone())
    }
}

/// Errores de validación
#[derive(Debug, Clone)]
pub enum ValidationError {
    RequiredField(String),
    InvalidType(String, String),
    OutOfRange(String, i64, i64, Option<i64>),
    InvalidFormat(String, String),
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidationError::RequiredField(key) => write!(f, "Required field '{}' is missing or empty", key),
            ValidationError::InvalidType(key, expected) => write!(f, "Field '{}' must be of type {}", key, expected),
            ValidationError::OutOfRange(key, value, min, max) => {
                if let Some(max) = max {
                    write!(f, "Field '{}' value {} is out of range [{}, {}]", key, value, min, max)
                } else {
                    write!(f, "Field '{}' value {} is below minimum {}", key, value, min)
                }
            }
            ValidationError::InvalidFormat(key, format) => write!(f, "Field '{}' has invalid {} format", key, format),
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

/// Loader principal de configuración con validación y hot reload
#[derive(Clone)]
pub struct ConfigLoader {
    pub sources: Vec<ConfigSource>,
    pub profile: Option<String>,
    pub cache: HashMap<String, ConfigValue>,
    pub validators: HashMap<String, Box<dyn ConfigValidator + Send + Sync>>,
    pub hot_reload_enabled: bool,
    pub reload_tx: broadcast::Sender<()>,
}


impl ConfigLoader {
    /// Crear nuevo ConfigLoader con fuentes por defecto
    pub fn new() -> Self {
        let (reload_tx, _) = broadcast::channel(10);

        Self {
            sources: vec![
                ConfigSource::File("config.json".to_string()),
                ConfigSource::Environment,
            ],
            profile: None,
            cache: HashMap::new(),
            validators: HashMap::new(),
            hot_reload_enabled: false,
            reload_tx,
        }
    }

    /// Agregar fuente de configuración
    pub fn add_source(mut self, source: ConfigSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Limpiar todas las fuentes de configuración
    pub fn clear_sources(mut self) -> Self {
        self.sources.clear();
        self
    }

    /// Establecer perfil (dev, staging, prod)
    pub fn with_profile(mut self, profile: String) -> Self {
        self.profile = Some(profile.clone());

        // Agregar archivos específicos del perfil después del archivo base
        // para que el perfil tenga mayor prioridad que la base
        self.sources.insert(1, ConfigSource::File(format!("config-{}.json", profile)));

        self
    }

    /// Agregar validador para una clave
    pub fn add_validator<V: ConfigValidator + Send + Sync + 'static>(
        mut self,
        key: String,
        validator: V,
    ) -> Self {
        self.validators.insert(key, Box::new(validator));
        self
    }

    /// Habilitar hot reload
    pub fn enable_hot_reload(mut self) -> Self {
        self.hot_reload_enabled = true;
        self
    }

    /// Obtener canal para notificaciones de reload
    pub fn reload_channel(&self) -> broadcast::Receiver<()> {
        self.reload_tx.subscribe()
    }

    /// Recargar configuración desde todas las fuentes
    pub fn load(&mut self) -> Result<(), ConfigError> {
        // Crear backup de la configuración actual para recuperación en caso de error
        let backup_cache = self.cache.clone();
        
        // Limpiar configuración actual
        self.cache.clear();
        
        // Intentar recargar desde todas las fuentes en orden de prioridad
        let result = self.load_from_sources();
        
        match result {
            Ok(_) => {
                // Notificar cambios
                let _ = self.reload_tx.send(());
                Ok(())
            }
            Err(e) => {
                // Restaurar configuración anterior en caso de error
                self.cache = backup_cache;
                Err(e)
            }
        }
    }

    /// Cargar desde todas las fuentes (extraído para mejor manejo de errores)
    fn load_from_sources(&mut self) -> Result<(), ConfigError> {
        let sources = self.sources.clone();
        for source in sources {
            match source {
                ConfigSource::File(path) => {
                    self.load_from_file(&path)?;
                }
                ConfigSource::Environment => {
                    let env_vars = self.load_from_env()?;
                    for (key, value) in env_vars {
                        self.cache.insert(key, ConfigValue {
                            value,
                            source: source.clone(),
                            profile: self.profile.clone(),
                        });
                    }
                }
                ConfigSource::Consul(_) => {
                    // TODO: Implementar carga desde Consul
                }
                ConfigSource::Vault(_) => {
                    // TODO: Implementar carga desde Vault
                }
            }
        }
        
        // Validar configuración
        self.validate_config()?;
        
        Ok(())
    }

    /// Cargar desde variables de entorno
    pub fn load_from_env(&self) -> Result<HashMap<String, String>, ConfigError> {
        let mut result = HashMap::new();
        
        for (key, value) in env::vars() {
            // Convertir a lowercase y reemplazar _ por .
            if key.starts_with("VELA_") {
                // Strip VELA_ prefix, then convert
                let converted = key[5..].to_lowercase().replace("_", ".");
                result.insert(converted, value);
            } else {
                let converted = key.to_lowercase().replace("_", ".");
                result.insert(converted, value);
            }
        }
        
        Ok(result)
    }

    /// Cargar desde archivo JSON
    pub fn load_from_file(&mut self, path: &str) -> Result<(), ConfigError> {
        let content = fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        
        self.load_from_json_value(json, path);
        Ok(())
    }

    /// Cargar desde valor JSON
    fn load_from_json_value(&mut self, value: serde_json::Value, source_path: &str) {
        self.load_from_json_recursive(value, "".to_string(), source_path);
    }

    /// Cargar recursivamente desde JSON
    fn load_from_json_recursive(&mut self, value: serde_json::Value, prefix: String, source_path: &str) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    let full_key = if prefix.is_empty() {
                        key
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    self.load_from_json_recursive(val, full_key, source_path);
                }
            }
            serde_json::Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let full_key = format!("{}[{}]", prefix, index);
                    self.load_from_json_recursive(val.clone(), full_key, source_path);
                }
            }
            _ => {
                // Para valores primitivos, almacenar en cache
                let config_value = ConfigValue {
                    value: value.to_string().trim_matches('"').to_string(),
                    source: ConfigSource::File(source_path.to_string()),
                    profile: self.profile.clone(),
                };
                self.cache.insert(prefix, config_value);
            }
        }
    }

    /// Validar configuración
    fn validate_config(&self) -> Result<(), ConfigError> {
        for (key, validator) in &self.validators {
            if let Some(config_value) = self.cache.get(key) {
                validator.validate(key, &config_value.value)?;
            } else {
                // If no value is set for a required field, check if it's required
                // For RequiredValidator, we need to check if the field is missing
                // But since RequiredValidator.validate() checks if value is empty,
                // and we don't have a value, we should fail
                if let Some(required_validator) = self.validators.get(key) {
                    // For RequiredValidator, we need to check if the field is missing
                    // But since RequiredValidator.validate() checks if value is empty,
                    // and we don't have a value, we should fail
                    required_validator.validate(key, "")?;
                }
            }
        }
        Ok(())
    }

    /// Verificar si una clave es requerida
    fn is_required(&self, key: &str) -> bool {
        // Por defecto, ninguna clave es requerida
        false
    }

    /// Obtener un valor como String
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.cache.get(key).map(|config_value| config_value.value.clone())
    }

    /// Obtener un valor como i32
    pub fn get_int(&self, key: &str) -> Option<Result<i32, std::num::ParseIntError>> {
        self.cache.get(key).map(|config_value| config_value.value.parse::<i32>())
    }

    /// Obtener un valor como bool
    pub fn get_bool(&self, key: &str) -> Option<Result<bool, std::str::ParseBoolError>> {
        self.cache.get(key).map(|config_value| config_value.value.parse::<bool>())
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
    Validation(ValidationError),
    Watcher(String),
    Notify(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Io(msg) => write!(f, "IO Error: {}", msg),
            ConfigError::Json(msg) => write!(f, "JSON Error: {}", msg),
            ConfigError::ParseInt(msg) => write!(f, "Parse Int Error: {}", msg),
            ConfigError::ParseBool(msg) => write!(f, "Parse Bool Error: {}", msg),
            ConfigError::NotImplemented(feature) => write!(f, "Not Implemented: {}", feature),
            ConfigError::Validation(err) => write!(f, "Validation Error: {}", err),
            ConfigError::Watcher(msg) => write!(f, "Watcher Error: {}", msg),
            ConfigError::Notify(msg) => write!(f, "Notify Error: {}", msg),
        }
    }
}

impl From<ValidationError> for ConfigError {
    fn from(err: ValidationError) -> Self {
        ConfigError::Validation(err)
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

        assert_eq!(result.get("test.key"), Some(&"test_value".to_string()));

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

impl From<notify::Error> for ConfigError {
    fn from(err: notify::Error) -> Self {
        ConfigError::Notify(err.to_string())
    }
}