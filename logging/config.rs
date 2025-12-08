//! Configuración del Sistema de Logging
//!
//! Define la configuración global del sistema de logging,
//! incluyendo nivel mínimo, transports y opciones de formato.

use crate::{Level, LogTransport};

/// Configuración global del sistema de logging
pub struct LogConfig {
    /// Nivel mínimo de logging
    pub level: Level,
    /// Lista de transports para enviar logs
    pub transports: Vec<Box<dyn LogTransport + Send + Sync>>,
    /// Si usar formato estructurado (JSON)
    pub structured: bool,
    /// Si incluir timestamp en logs
    pub include_timestamp: bool,
    /// Si incluir thread ID
    pub include_thread_id: bool,
    /// Metadata global incluida en todos los logs
    pub global_metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl LogConfig {
    /// Crea configuración por defecto
    pub fn default() -> Self {
        Self {
            level: Level::INFO,
            transports: vec![Box::new(crate::ConsoleTransport::new())],
            structured: false,
            include_timestamp: true,
            include_thread_id: false,
            global_metadata: std::collections::HashMap::new(),
        }
    }

    /// Crea configuración para desarrollo
    pub fn development() -> Self {
        Self {
            level: Level::DEBUG,
            transports: vec![Box::new(crate::ConsoleTransport::colored())],
            structured: false,
            include_timestamp: true,
            include_thread_id: true,
            global_metadata: std::collections::HashMap::new(),
        }
    }

    /// Crea configuración para producción
    pub fn production() -> Self {
        Self {
            level: Level::WARN,
            transports: vec![
                Box::new(crate::ConsoleTransport::new()),
                // Box::new(crate::FileTransport::new("app.log")),
            ],
            structured: true,
            include_timestamp: true,
            include_thread_id: false,
            global_metadata: std::collections::HashMap::new(),
        }
    }

    /// Agrega transport a la configuración
    pub fn with_transport<T: LogTransport + Send + Sync + 'static>(mut self, transport: T) -> Self {
        self.transports.push(Box::new(transport));
        self
    }

    /// Establece nivel mínimo
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Habilita formato estructurado
    pub fn structured(mut self) -> Self {
        self.structured = true;
        self
    }

    /// Agrega metadata global
    pub fn with_global_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.global_metadata.insert(key.into(), value.into());
        self
    }
}

impl Clone for LogConfig {
    fn clone(&self) -> Self {
        Self {
            level: self.level,
            transports: Vec::new(), // No clonamos transports por simplicidad
            structured: self.structured,
            include_timestamp: self.include_timestamp,
            include_thread_id: self.include_thread_id,
            global_metadata: self.global_metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.transports.len(), 1);
        assert!(!config.structured);
        assert!(config.include_timestamp);
        assert!(!config.include_thread_id);
    }

    #[test]
    fn test_log_config_development() {
        let config = LogConfig::development();
        assert_eq!(config.level, Level::DEBUG);
        assert!(config.include_thread_id);
    }

    #[test]
    fn test_log_config_production() {
        let config = LogConfig::production();
        assert_eq!(config.level, Level::WARN);
        assert!(config.structured);
    }

    #[test]
    fn test_log_config_with_transport() {
        let config = LogConfig::default()
            .with_transport(crate::ConsoleTransport::new());

        assert_eq!(config.transports.len(), 2);
    }

    #[test]
    fn test_log_config_with_level() {
        let config = LogConfig::default().with_level(Level::ERROR);
        assert_eq!(config.level, Level::ERROR);
    }

    #[test]
    fn test_log_config_structured() {
        let config = LogConfig::default().structured();
        assert!(config.structured);
    }

    #[test]
    fn test_log_config_global_metadata() {
        let config = LogConfig::default()
            .with_global_metadata("service", "user-service")
            .with_global_metadata("version", "1.0.0");

        assert_eq!(config.global_metadata.get("service").unwrap(), "user-service");
        assert_eq!(config.global_metadata.get("version").unwrap(), "1.0.0");
    }
}