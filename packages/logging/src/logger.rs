//! Logger Principal
//!
//! Implementa la clase Logger<T> genérica que es el núcleo
//! del sistema de logging estructurado.

use crate::{Level, LogRecord, LogConfig};
use std::collections::HashMap;
use std::sync::Arc;

/// Logger genérico con tipo de contexto T
pub struct Logger<T> {
    /// Nombre del logger
    name: String,
    /// Configuración del logger
    config: Arc<LogConfig>,
    /// Contexto específico del logger
    context: T,
    /// Metadata específica del logger
    metadata: HashMap<String, serde_json::Value>,
}

impl<T> Logger<T> {
    /// Crea un nuevo logger
    pub fn new(name: impl Into<String>, config: Arc<LogConfig>, context: T) -> Self {
        Self {
            name: name.into(),
            config,
            context,
            metadata: HashMap::new(),
        }
    }

    /// Crea logger con metadata inicial
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Obtiene el nombre del logger
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Obtiene el contexto
    pub fn context(&self) -> &T {
        &self.context
    }

    /// Obtiene la configuración
    pub fn config(&self) -> &LogConfig {
        &self.config
    }

    /// Crea un registro de log base
    fn create_record(&self, level: Level, message: impl Into<String>) -> LogRecord {
        let mut record = LogRecord::new(level, message, &self.name)
            .merge_global_metadata(&self.config.global_metadata);

        // Agregar metadata específica del logger
        for (key, value) in &self.metadata {
            record.metadata.insert(key.clone(), value.clone());
        }

        // Agregar información de thread si está habilitado
        if self.config.include_thread_id {
            let thread_id = format!("{:?}", std::thread::current().id());
            record = record.with_thread_id(thread_id);
        }

        record
    }

    /// Escribe un registro de log a través de todos los transports
    async fn write_record(&self, record: LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Aplicar filtros avanzados, sampling y rate limiting
        if !self.config.should_log(&record) {
            return Ok(());
        }

        // Enviar a todos los transports
        for transport in &self.config.transports {
            if transport.should_write(record.level) {
                transport.write(&record).await?;
            }
        }

        Ok(())
    }

    /// Log a nivel DEBUG
    pub async fn debug(&self, message: impl Into<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = self.create_record(Level::DEBUG, message);
        self.write_record(record).await
    }

    /// Log a nivel INFO
    pub async fn info(&self, message: impl Into<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = self.create_record(Level::INFO, message);
        self.write_record(record).await
    }

    /// Log a nivel WARN
    pub async fn warn(&self, message: impl Into<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = self.create_record(Level::WARN, message);
        self.write_record(record).await
    }

    /// Log a nivel ERROR
    pub async fn error(&self, message: impl Into<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = self.create_record(Level::ERROR, message);
        self.write_record(record).await
    }

    /// Log a nivel FATAL
    pub async fn fatal(&self, message: impl Into<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = self.create_record(Level::FATAL, message);
        self.write_record(record).await
    }

    /// Log con metadata adicional (un solo uso)
    pub async fn log_with_metadata(
        &self,
        metadata: HashMap<String, serde_json::Value>,
        message: impl Into<String>,
        level: Level,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut record = self.create_record(level, message);

        // Agregar metadata adicional
        for (key, value) in metadata {
            record.metadata.insert(key, value);
        }

        self.write_record(record).await
    }

    /// Log con ID de correlación
    pub async fn with_correlation_id(
        &self,
        correlation_id: impl Into<String>,
        message: impl Into<String>,
        level: Level,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = self.create_record(level, message)
            .with_correlation_id(correlation_id);
        self.write_record(record).await
    }

    /// Log con ubicación de código (útil para debugging)
    pub async fn with_location(
        &self,
        message: impl Into<String>,
        level: Level,
        file: impl Into<String>,
        line: u32,
        function: impl Into<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = self.create_record(level, message)
            .with_location(file, line, function);
        self.write_record(record).await
    }
}

impl<T: Clone> Clone for Logger<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            config: self.config.clone(),
            context: self.context.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// Logger builder para configuración fluida
pub struct LoggerBuilder<T> {
    name: String,
    config: Arc<LogConfig>,
    context: T,
    metadata: HashMap<String, serde_json::Value>,
}

impl<T> LoggerBuilder<T> {
    /// Crea un nuevo builder
    pub fn new(name: impl Into<String>, config: Arc<LogConfig>, context: T) -> Self {
        Self {
            name: name.into(),
            config,
            context,
            metadata: HashMap::new(),
        }
    }

    /// Agrega metadata
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Construye el logger
    pub fn build(self) -> Logger<T> {
        let mut logger = Logger::new(self.name, self.config, self.context);
        logger.metadata = self.metadata;
        logger
    }
}

/// Logger global para casos simples
pub type SimpleLogger = Logger<()>;

impl SimpleLogger {
    /// Crea logger simple con configuración por defecto
    pub fn simple(name: impl Into<String>) -> Self {
        let config = Arc::new(LogConfig::default());
        Logger::new(name, config, ())
    }

    /// Crea logger simple con configuración personalizada
    pub fn with_config(name: impl Into<String>, config: Arc<LogConfig>) -> Self {
        Logger::new(name, config, ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_logger_creation() {
        let config = Arc::new(LogConfig::default());
        let logger: Logger<()> = Logger::new("test", config, ());

        assert_eq!(logger.name(), "test");
    }

    #[tokio::test]
    async fn test_logger_debug() {
        let config = Arc::new(LogConfig::development());
        let logger: Logger<()> = Logger::new("test", config, ());

        let result = logger.debug("Debug message").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logger_info() {
        let config = Arc::new(LogConfig::default());
        let logger: Logger<()> = Logger::new("test", config, ());

        let result = logger.info("Info message").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logger_with_metadata() {
        let config = Arc::new(LogConfig::default());
        let logger: Logger<()> = Logger::new("test", config, ())
            .with_metadata("service", "test-service");

        assert_eq!(logger.metadata.get("service").unwrap(), "test-service");
    }

    #[tokio::test]
    async fn test_logger_log_with_context() {
        let config = Arc::new(LogConfig::default());
        let logger: Logger<()> = Logger::new("test", config, ());

        let mut context_metadata = HashMap::new();
        context_metadata.insert("request_id".to_string(), serde_json::json!("abc-123"));

        let result = logger.log_with_metadata(context_metadata, "Message with context", Level::INFO).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logger_builder() {
        let config = Arc::new(LogConfig::default());
        let logger: Logger<()> = LoggerBuilder::new("test", config, ())
            .add_metadata("version", "1.0.0")
            .build();

        assert_eq!(logger.name(), "test");
        assert_eq!(logger.metadata.get("version").unwrap(), "1.0.0");
    }

    #[tokio::test]
    async fn test_simple_logger() {
        let logger = SimpleLogger::simple("simple_test");

        let result = logger.info("Simple log message").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_level_filtering() {
        let config = Arc::new(LogConfig::production()); // WARN level mínimo
        let logger: Logger<()> = Logger::new("test", config, ());

        // DEBUG debería ser filtrado
        let debug_result = logger.debug("Debug should be filtered").await;
        assert!(debug_result.is_ok()); // No error, solo filtrado

        // WARN debería pasar
        let warn_result = logger.warn("Warn should pass").await;
        assert!(warn_result.is_ok());
    }
}