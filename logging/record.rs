//! Estructura de Registro de Log
//!
//! Define la estructura de datos para un registro de log,
//! incluyendo timestamp, nivel, mensaje y metadata.

use crate::Level;
use std::collections::HashMap;

/// Registro de log estructurado
#[derive(Debug, Clone, serde::Serialize)]
pub struct LogRecord {
    /// Timestamp del log
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Nivel del log
    pub level: Level,
    /// Mensaje del log
    pub message: String,
    /// Nombre del logger
    pub logger_name: String,
    /// ID del thread (opcional)
    pub thread_id: Option<String>,
    /// ID de correlación para tracing distribuido
    pub correlation_id: Option<String>,
    /// Metadata adicional
    pub metadata: HashMap<String, serde_json::Value>,
    /// Archivo fuente (opcional)
    pub file: Option<String>,
    /// Línea fuente (opcional)
    pub line: Option<u32>,
    /// Función fuente (opcional)
    pub function: Option<String>,
}

impl LogRecord {
    /// Crea un nuevo registro de log
    pub fn new(
        level: Level,
        message: impl Into<String>,
        logger_name: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level,
            message: message.into(),
            logger_name: logger_name.into(),
            thread_id: None,
            correlation_id: None,
            metadata: HashMap::new(),
            file: None,
            line: None,
            function: None,
        }
    }

    /// Agrega thread ID
    pub fn with_thread_id(mut self, thread_id: impl Into<String>) -> Self {
        self.thread_id = Some(thread_id.into());
        self
    }

    /// Agrega ID de correlación
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Agrega metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Agrega información de ubicación
    pub fn with_location(mut self, file: impl Into<String>, line: u32, function: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self.function = Some(function.into());
        self
    }

    /// Fusiona metadata global
    pub fn merge_global_metadata(mut self, global_metadata: &HashMap<String, serde_json::Value>) -> Self {
        for (key, value) in global_metadata {
            if !self.metadata.contains_key(key) {
                self.metadata.insert(key.clone(), value.clone());
            }
        }
        self
    }

    /// Convierte a JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convierte a JSON pretty-printed
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Formatea como string legible
    pub fn format(&self, include_timestamp: bool, include_thread: bool) -> String {
        let mut parts = Vec::new();

        if include_timestamp {
            parts.push(format!("[{}]", self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC")));
        }

        parts.push(format!("[{}]", self.level.as_str().to_uppercase()));

        if include_thread {
            if let Some(thread_id) = &self.thread_id {
                parts.push(format!("[{}]", thread_id));
            }
        }

        parts.push(format!("[{}]", self.logger_name));

        if let Some(file) = &self.file {
            if let Some(line) = self.line {
                parts.push(format!("{}:{}", file, line));
            }
        }

        parts.push(self.message.clone());

        if !self.metadata.is_empty() {
            let metadata_str = self.metadata.iter()
                .map(|(k, v)| {
                    match v {
                        serde_json::Value::String(s) => format!("{}={}", k, s),
                        serde_json::Value::Number(n) => format!("{}={}", k, n),
                        serde_json::Value::Bool(b) => format!("{}={}", k, b),
                        _ => format!("{}={}", k, v), // Para arrays y objetos usa el formato JSON
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("{{{}}}", metadata_str));
        }

        parts.join(" ")
    }
}

impl Default for LogRecord {
    fn default() -> Self {
        Self::new(Level::INFO, "", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_record_creation() {
        let record = LogRecord::new(Level::INFO, "Test message", "test_logger");

        assert_eq!(record.level, Level::INFO);
        assert_eq!(record.message, "Test message");
        assert_eq!(record.logger_name, "test_logger");
        assert!(record.thread_id.is_none());
        assert!(record.correlation_id.is_none());
        assert!(record.metadata.is_empty());
    }

    #[test]
    fn test_log_record_with_metadata() {
        let record = LogRecord::new(Level::DEBUG, "Debug message", "test")
            .with_metadata("user_id", 123)
            .with_metadata("action", "login");

        assert_eq!(record.metadata.len(), 2);
        assert_eq!(record.metadata.get("user_id").unwrap(), 123);
        assert_eq!(record.metadata.get("action").unwrap(), "login");
    }

    #[test]
    fn test_log_record_with_location() {
        let record = LogRecord::new(Level::ERROR, "Error occurred", "error_logger")
            .with_location("main.rs", 42, "main");

        assert_eq!(record.file.as_ref().unwrap(), "main.rs");
        assert_eq!(record.line.unwrap(), 42);
        assert_eq!(record.function.as_ref().unwrap(), "main");
    }

    #[test]
    fn test_log_record_merge_global_metadata() {
        let mut global_metadata = HashMap::new();
        global_metadata.insert("service".to_string(), serde_json::json!("user-service"));
        global_metadata.insert("version".to_string(), serde_json::json!("1.0.0"));

        let record = LogRecord::new(Level::INFO, "Test", "test")
            .with_metadata("request_id", "abc-123")
            .merge_global_metadata(&global_metadata);

        assert_eq!(record.metadata.len(), 3);
        assert_eq!(record.metadata.get("service").unwrap(), "user-service");
        assert_eq!(record.metadata.get("version").unwrap(), "1.0.0");
        assert_eq!(record.metadata.get("request_id").unwrap(), "abc-123");
    }

    #[test]
    fn test_log_record_format() {
        let record = LogRecord::new(Level::WARN, "Warning message", "test_logger")
            .with_thread_id("thread-1")
            .with_metadata("key", "value");

        let formatted = record.format(true, true);
        assert!(formatted.contains("[WARN]"));
        assert!(formatted.contains("[thread-1]"));
        assert!(formatted.contains("[test_logger]"));
        assert!(formatted.contains("Warning message"));
        assert!(formatted.contains("{key=value}"));
    }

    #[test]
    fn test_log_record_to_json() {
        let record = LogRecord::new(Level::INFO, "JSON test", "json_logger");

        let json = record.to_json().unwrap();
        assert!(json.contains("\"level\":\"INFO\""));
        assert!(json.contains("\"message\":\"JSON test\""));
        assert!(json.contains("\"logger_name\":\"json_logger\""));
    }
}