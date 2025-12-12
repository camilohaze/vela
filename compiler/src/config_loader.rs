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
pub trait ConfigValidator {
    fn validate(&self, key: &str, value: &str) -> Result<(), ValidationError>;
}

/// Validador requerido
pub struct RequiredValidator;

impl ConfigValidator for RequiredValidator {
    fn validate(&self, key: &str, value: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            return Err(ValidationError::RequiredField(key.to_string()));
        }
        Ok(())
    }
}

/// Validador de rango numérico
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
                return Err(ValidationError::OutOfRange(key.to_string(), num, self.min.unwrap_or(i64::MIN), max));
            }
        }

        Ok(())
    }
}

/// Validador de email básico
pub struct EmailValidator;

impl ConfigValidator for EmailValidator {
    fn validate(&self, key: &str, value: &str) -> Result<(), ValidationError> {
        if !value.contains('@') || !value.contains('.') {
            return Err(ValidationError::InvalidFormat(key.to_string(), "email".to_string()));
        }
        Ok(())
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
pub struct ConfigLoader {
    sources: Vec<ConfigSource>,
    profile: Option<String>,
    cache: HashMap<String, ConfigValue>,
    validators: HashMap<String, Box<dyn ConfigValidator + Send + Sync>>,
    hot_reload_enabled: bool,
    reload_tx: broadcast::Sender<()>,
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

    /// Establecer perfil (dev, staging, prod)
    pub fn with_profile(mut self, profile: String) -> Self {
        self.profile = Some(profile.clone());

        // Agregar archivos específicos del perfil
        self.sources.insert(0, ConfigSource::File(format!("config-{}.json", profile)));

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

    /// Cargar configuración desde todas las fuentes con validación
    pub fn load(&mut self) -> Result<(), ConfigError> {
        self.cache.clear();

        for source in &self.sources {
            match self.load_from_source(source) {
                Ok(values) => {
                    // Valores de mayor prioridad sobrescriben
                    for (key, value) in values {
                        self.cache.insert(key.clone(), ConfigValue {
                            value: value.clone(),
                            source: source.clone(),
                            profile: self.profile.clone(),
                        });

                        // Validar el valor
                        if let Some(validator) = self.validators.get(&key) {
                            validator.validate(&key, &value)
                                .map_err(|e| ConfigError::Validation(e))?;
                        }
                    }
                }
                Err(e) => {
                    // Log error pero continua con otras fuentes
                    eprintln!("Error loading from {:?}: {}", source, e);
                }
            }
        }

        // Iniciar hot reload si está habilitado
        if self.hot_reload_enabled {
            self.start_hot_reload()?;
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
            ConfigSource::Consul(endpoint) => self.load_from_consul(endpoint),
            ConfigSource::Vault(endpoint) => self.load_from_vault(endpoint),
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

    /// Cargar desde Consul (implementación básica)
    fn load_from_consul(&self, endpoint: &str) -> Result<HashMap<String, String>, ConfigError> {
        // TODO: Implementar cliente HTTP real para Consul
        // Por ahora, simular carga desde un archivo local que representaría Consul
        let consul_file = format!("consul-{}.json", endpoint.replace(":", "-"));
        if Path::new(&consul_file).exists() {
            self.load_from_file(&consul_file)
        } else {
            // Simular algunos valores por defecto de Consul
            let mut result = HashMap::new();
            result.insert("consul.service_name".to_string(), "vela-app".to_string());
            result.insert("consul.health_check".to_string(), "true".to_string());
            Ok(result)
        }
    }

    /// Cargar desde Vault (implementación básica)
    fn load_from_vault(&self, endpoint: &str) -> Result<HashMap<String, String>, ConfigError> {
        // TODO: Implementar cliente HTTP real para Vault
        // Por ahora, simular carga desde un archivo local que representaría Vault
        let vault_file = format!("vault-{}.json", endpoint.replace(":", "-"));
        if Path::new(&vault_file).exists() {
            self.load_from_file(&vault_file)
        } else {
            // Simular algunos secrets por defecto de Vault
            let mut result = HashMap::new();
            result.insert("vault.database_password".to_string(), "secret123".to_string());
            result.insert("vault.api_key".to_string(), "vault-key-123".to_string());
            Ok(result)
        }
    }

    /// Iniciar hot reload con file watchers
    fn start_hot_reload(&self) -> Result<(), ConfigError> {
        let tx = self.reload_tx.clone();
        let watched_files: Vec<String> = self.sources.iter()
            .filter_map(|source| match source {
                ConfigSource::File(path) => Some(path.clone()),
                _ => None,
            })
            .collect();

        if watched_files.is_empty() {
            return Ok(());
        }

        // Crear watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        // Notificar cambio de configuración
                        let _ = tx.send(());
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        })?;

        // Watch archivos
        for file in watched_files {
            if Path::new(&file).exists() {
                watcher.watch(Path::new(&file), RecursiveMode::NonRecursive)?;
            }
        }

        // Mantener watcher vivo (en un escenario real, esto iría en un task separado)
        // Por ahora, solo lo creamos

        Ok(())
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