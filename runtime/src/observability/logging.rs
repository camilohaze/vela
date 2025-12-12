use std::io::Write;
// Structured logging for Vela services
//
// This module provides structured logging with automatic trace context
// injection and multiple output sinks.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    TRACE = 0,
    DEBUG = 1,
    INFO = 2,
    WARN = 3,
    ERROR = 4,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::TRACE => write!(f, "TRACE"),
            Level::DEBUG => write!(f, "DEBUG"),
            Level::INFO => write!(f, "INFO"),
            Level::WARN => write!(f, "WARN"),
            Level::ERROR => write!(f, "ERROR"),
        }
    }
}

/// Structured log record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    /// Timestamp in RFC3339 format
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: Level,
    /// Logger name
    pub logger: String,
    /// Log message
    pub message: String,
    /// Structured fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Trace context (if available)
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    /// Error information
    pub error: Option<String>,
    /// Source location
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl LogRecord {
    /// Create a new log record
    pub fn new(level: Level, logger: &str, message: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            logger: logger.to_string(),
            message: message.to_string(),
            fields: HashMap::new(),
            trace_id: None,
            span_id: None,
            error: None,
            file: None,
            line: None,
        }
    }

    /// Add a structured field
    pub fn with_field(mut self, key: &str, value: serde_json::Value) -> Self {
        self.fields.insert(key.to_string(), value);
        self
    }

    /// Add multiple fields
    pub fn with_fields(mut self, fields: HashMap<String, serde_json::Value>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Set trace context
    pub fn with_trace_context(mut self, trace_id: Option<String>, span_id: Option<String>) -> Self {
        self.trace_id = trace_id;
        self.span_id = span_id;
        self
    }

    /// Set error information
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }

    /// Set source location
    pub fn with_location(mut self, file: &str, line: u32) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self
    }
}

/// Trait for log sinks
#[async_trait::async_trait]
pub trait LogSink: Send + Sync {
    /// Write a log record to the sink
    async fn log(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error>>;

    /// Flush any buffered records
    async fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Close the sink
    async fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Console sink for logging to stdout/stderr
pub struct ConsoleSink {
    use_colors: bool,
}

impl ConsoleSink {
    /// Create a new console sink
    pub fn new() -> Self {
        Self { use_colors: true }
    }

    /// Create a console sink without colors
    pub fn without_colors() -> Self {
        Self { use_colors: false }
    }

    fn color_code(&self, level: Level) -> &'static str {
        if !self.use_colors {
            return "";
        }

        match level {
            Level::TRACE => "\x1b[37m", // White
            Level::DEBUG => "\x1b[36m", // Cyan
            Level::INFO => "\x1b[32m",  // Green
            Level::WARN => "\x1b[33m",  // Yellow
            Level::ERROR => "\x1b[31m", // Red
        }
    }

    fn reset_code(&self) -> &'static str {
        if self.use_colors { "\x1b[0m" } else { "" }
    }
}

#[async_trait::async_trait]
impl LogSink for ConsoleSink {
    async fn log(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error>> {
        let color = self.color_code(record.level);
        let reset = self.reset_code();

        use std::io::Write;
        let mut stderr = std::io::stderr();
        let mut stdout = std::io::stdout();
        let output: &mut dyn Write = if record.level >= Level::WARN {
            &mut stderr
        } else {
            &mut stdout
        };

        let json = serde_json::to_string(record)?;
        writeln!(output, "{}{}{}", color, json, reset)?;

        Ok(())
    }
}

/// File sink for logging to files
pub struct FileSink {
    file: Arc<tokio::sync::Mutex<std::fs::File>>,
}

impl FileSink {
    /// Create a new file sink
    pub async fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            file: Arc::new(tokio::sync::Mutex::new(file)),
        })
    }
}

#[async_trait::async_trait]
impl LogSink for FileSink {
    async fn log(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(record)?;
        let mut file = self.file.lock().await;
        use std::io::Write;
        writeln!(file, "{}", json)?;
        file.flush()?;
        Ok(())
    }

    async fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = self.file.lock().await;
        file.flush()?;
        Ok(())
    }
}

/// In-memory sink for testing
#[derive(Debug, Default)]
pub struct MemorySink {
    records: Arc<tokio::sync::Mutex<Vec<LogRecord>>>,
}

impl MemorySink {
    /// Create a new memory sink
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all logged records
    pub async fn records(&self) -> Vec<LogRecord> {
        self.records.lock().await.clone()
    }

    /// Clear all records
    pub async fn clear(&self) {
        self.records.lock().await.clear();
    }

    /// Get record count
    pub async fn len(&self) -> usize {
        self.records.lock().await.len()
    }
}

#[async_trait::async_trait]
impl LogSink for MemorySink {
    async fn log(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error>> {
        self.records.lock().await.push(record.clone());
        Ok(())
    }
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Logger name
    pub name: String,
    /// Minimum log level
    pub level: Level,
    /// Whether to include location info
    pub include_location: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            name: "vela".to_string(),
            level: Level::INFO,
            include_location: false,
        }
    }
}

/// Logger instance
#[derive(Clone)]
pub struct Logger {
    config: LoggerConfig,
    sinks: Arc<Vec<Box<dyn LogSink>>>,
}

impl Logger {
    /// Create a new logger
    pub fn new(config: LoggerConfig, sinks: Vec<Box<dyn LogSink>>) -> Self {
        Self {
            config,
            sinks: Arc::new(sinks),
        }
    }

    /// Create a logger with a single sink
    pub fn with_sink<S: LogSink + 'static>(config: LoggerConfig, sink: S) -> Self {
        Self::new(config, vec![Box::new(sink)])
    }

    /// Check if a level is enabled
    pub fn is_enabled(&self, level: Level) -> bool {
        level >= self.config.level
    }

    /// Log a record
    pub async fn log(&self, mut record: LogRecord) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_enabled(record.level) {
            return Ok(());
        }

        // Add location info if enabled
        if self.config.include_location {
            // In a real implementation, this would use backtrace or similar
            record = record.with_location("unknown", 0);
        }

        // Send to all sinks
        for sink in self.sinks.iter() {
            sink.log(&record).await?;
        }

        Ok(())
    }

    /// Log at TRACE level
    pub async fn trace(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let record = LogRecord::new(Level::TRACE, &self.config.name, message);
        self.log(record).await
    }

    /// Log at DEBUG level
    pub async fn debug(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let record = LogRecord::new(Level::DEBUG, &self.config.name, message);
        self.log(record).await
    }

    /// Log at INFO level
    pub async fn info(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let record = LogRecord::new(Level::INFO, &self.config.name, message);
        self.log(record).await
    }

    /// Log at WARN level
    pub async fn warn(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let record = LogRecord::new(Level::WARN, &self.config.name, message);
        self.log(record).await
    }

    /// Log at ERROR level
    pub async fn error(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let record = LogRecord::new(Level::ERROR, &self.config.name, message);
        self.log(record).await
    }

    /// Log with structured fields
    pub async fn log_with_fields(
        &self,
        level: Level,
        message: &str,
        fields: HashMap<String, serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let record = LogRecord::new(level, &self.config.name, message).with_fields(fields);
        self.log(record).await
    }

    /// Flush all sinks
    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        for sink in self.sinks.iter() {
            sink.flush().await?;
        }
        Ok(())
    }

    /// Close all sinks
    pub async fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        for sink in self.sinks.iter() {
            sink.close().await?;
        }
        Ok(())
    }
}

/// Global logging registry
pub struct LoggingRegistry {
    logger: Arc<tokio::sync::RwLock<Option<Logger>>>,
}

impl LoggingRegistry {
    /// Create a new logging registry
    pub fn new() -> Self {
        Self {
            logger: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Initialize the global logger
    pub async fn init(&self, config: LoggerConfig, sinks: Vec<Box<dyn LogSink>>) -> Result<(), Box<dyn std::error::Error>> {
        let logger = Logger::new(config, sinks);
        *self.logger.write().await = Some(logger);
        Ok(())
    }

    /// Get the global logger
    pub async fn get_logger(&self) -> Option<Logger> {
        self.logger.read().await.clone()
    }
}

impl Default for LoggingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global static registry instance
static LOGGING_REGISTRY: once_cell::sync::Lazy<LoggingRegistry> = once_cell::sync::Lazy::new(|| {
    LoggingRegistry::new()
});

/// Get the global logging registry
pub fn global_logging() -> &'static LoggingRegistry {
    &LOGGING_REGISTRY
}

/// Initialize global logging
pub async fn init_logging(config: LoggerConfig, sinks: Vec<Box<dyn LogSink>>) -> Result<(), Box<dyn std::error::Error>> {
    global_logging().init(config, sinks).await
}

/// Get the global logger
pub async fn get_logger() -> Option<Logger> {
    global_logging().get_logger().await
}

/// Convenience function to log at INFO level
pub async fn info(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(logger) = get_logger().await {
        logger.info(message).await
    } else {
        // Fallback to println if no logger is configured
        println!("[INFO] {}", message);
        Ok(())
    }
}

/// Convenience function to log at ERROR level
pub async fn error(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(logger) = get_logger().await {
        logger.error(message).await
    } else {
        // Fallback to eprintln if no logger is configured
        eprintln!("[ERROR] {}", message);
        Ok(())
    }
}