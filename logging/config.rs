//! Configuración del Sistema de Logging
//!
//! Define la configuración global del sistema de logging,
//! incluyendo nivel mínimo, transports y opciones de formato.

use crate::{Level, LogTransport};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Función de filtro para logs basado en metadata
pub type LogFilter = Box<dyn Fn(&crate::LogRecord) -> bool + Send + Sync>;

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
    /// Filtros adicionales para logs
    pub filters: Vec<LogFilter>,
    /// Tasa de sampling (0.0 = ninguno, 1.0 = todos)
    pub sampling_rate: f64,
    /// Rate limiting: máximo de logs por segundo (-1 = sin límite)
    pub rate_limit_per_second: i64,
    /// Estado interno para rate limiting (compartido entre instancias)
    rate_limit_state: Arc<RateLimitState>,
}

/// Estado interno para rate limiting
#[derive(Debug)]
struct RateLimitState {
    last_reset: std::sync::Mutex<Instant>,
    counter: AtomicI64,
}

impl RateLimitState {
    fn new() -> Self {
        Self {
            last_reset: std::sync::Mutex::new(Instant::now()),
            counter: AtomicI64::new(0),
        }
    }
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
            filters: Vec::new(),
            sampling_rate: 1.0, // Loggear todos los logs por defecto
            rate_limit_per_second: -1, // Sin límite por defecto
            rate_limit_state: Arc::new(RateLimitState::new()),
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
            filters: Vec::new(),
            sampling_rate: 1.0,
            rate_limit_per_second: -1,
            rate_limit_state: Arc::new(RateLimitState::new()),
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
            filters: Vec::new(),
            sampling_rate: 0.1, // Solo 10% de logs en producción
            rate_limit_per_second: 100, // Máximo 100 logs por segundo
            rate_limit_state: Arc::new(RateLimitState::new()),
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

    /// Agrega filtro personalizado
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&crate::LogRecord) -> bool + Send + Sync + 'static,
    {
        self.filters.push(Box::new(filter));
        self
    }

    /// Establece tasa de sampling (0.0 = ninguno, 1.0 = todos)
    pub fn with_sampling_rate(mut self, rate: f64) -> Self {
        self.sampling_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Establece rate limiting (logs por segundo, -1 = sin límite)
    pub fn with_rate_limit(mut self, logs_per_second: i64) -> Self {
        self.rate_limit_per_second = logs_per_second;
        self
    }

    /// Agrega filtro para excluir logs con metadata específica
    pub fn exclude_by_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let key = key.into();
        let value = value.into();
        self.filters.push(Box::new(move |record: &crate::LogRecord| {
            !record.metadata.get(&key).map_or(false, |v| v == &value)
        }));
        self
    }

    /// Agrega filtro para incluir solo logs con metadata específica
    pub fn include_only_by_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let key = key.into();
        let value = value.into();
        self.filters.push(Box::new(move |record: &crate::LogRecord| {
            record.metadata.get(&key).map_or(false, |v| v == &value)
        }));
        self
    }

    /// Verifica si un log record debe ser procesado según filtros y sampling
    pub fn should_log(&self, record: &crate::LogRecord) -> bool {
        // 1. Verificar nivel mínimo
        if record.level < self.level {
            return false;
        }

        // 2. Aplicar filtros personalizados
        for filter in &self.filters {
            if !filter(record) {
                return false;
            }
        }

        // 3. Aplicar sampling
        if self.sampling_rate < 1.0 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            record.timestamp.hash(&mut hasher);
            record.message.hash(&mut hasher);
            let hash = hasher.finish();

            // Usar los bits bajos del hash para determinar si loggear
            let threshold = (self.sampling_rate * u64::MAX as f64) as u64;
            if hash > threshold {
                return false;
            }
        }

        // 4. Aplicar rate limiting
        if self.rate_limit_per_second > 0 {
            let now = Instant::now();
            let mut last_reset = self.rate_limit_state.last_reset.lock().unwrap();

            // Reset counter si ha pasado más de 1 segundo
            if now.duration_since(*last_reset) >= Duration::from_secs(1) {
                self.rate_limit_state.counter.store(0, Ordering::SeqCst);
                *last_reset = now;
            }

            // Verificar límite
            let current_count = self.rate_limit_state.counter.fetch_add(1, Ordering::SeqCst);
            if current_count >= self.rate_limit_per_second {
                return false;
            }
        }

        true
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
            filters: Vec::new(), // No clonamos closures
            sampling_rate: self.sampling_rate,
            rate_limit_per_second: self.rate_limit_per_second,
            rate_limit_state: Arc::clone(&self.rate_limit_state), // Compartir estado
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

    #[test]
    fn test_log_config_with_sampling_rate() {
        let config = LogConfig::default().with_sampling_rate(0.5);
        assert_eq!(config.sampling_rate, 0.5);

        // Test clamping
        let config_low = LogConfig::default().with_sampling_rate(-0.1);
        assert_eq!(config_low.sampling_rate, 0.0);

        let config_high = LogConfig::default().with_sampling_rate(1.5);
        assert_eq!(config_high.sampling_rate, 1.0);
    }

    #[test]
    fn test_log_config_with_rate_limit() {
        let config = LogConfig::default().with_rate_limit(50);
        assert_eq!(config.rate_limit_per_second, 50);
    }

    #[test]
    fn test_log_config_exclude_by_metadata() {
        let config = LogConfig::default()
            .exclude_by_metadata("component", "test");

        let record = crate::LogRecord::new(Level::INFO, "test message", "logger")
            .with_metadata("component", "test");

        assert!(!config.should_log(&record));
    }

    #[test]
    fn test_log_config_include_only_by_metadata() {
        let config = LogConfig::default()
            .include_only_by_metadata("service", "api");

        let record_matching = crate::LogRecord::new(Level::INFO, "test message", "logger")
            .with_metadata("service", "api");

        let record_not_matching = crate::LogRecord::new(Level::INFO, "test message", "logger")
            .with_metadata("service", "web");

        assert!(config.should_log(&record_matching));
        assert!(!config.should_log(&record_not_matching));
    }

    #[test]
    fn test_log_config_should_log_with_filters() {
        let config = LogConfig::default()
            .with_level(Level::INFO)
            .exclude_by_metadata("level", "debug");

        // Should log: INFO level, no excluded metadata
        let record_info = crate::LogRecord::new(Level::INFO, "info message", "logger");
        assert!(config.should_log(&record_info));

        // Should not log: DEBUG level
        let record_debug = crate::LogRecord::new(Level::DEBUG, "debug message", "logger");
        assert!(!config.should_log(&record_debug));

        // Should not log: excluded metadata
        let record_excluded = crate::LogRecord::new(Level::INFO, "info message", "logger")
            .with_metadata("level", "debug");
        assert!(!config.should_log(&record_excluded));
    }
}