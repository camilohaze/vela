//! Transportes de Log
//!
//! Define la interfaz para transportes de log y implementaciones
//! comunes como consola, archivo, HTTP y syslog.

use crate::{LogRecord, Level};
use async_trait::async_trait;
use std::io::Write;

/// Interfaz para transportes de log
#[async_trait]
pub trait LogTransport: Send + Sync {
    /// Escribe un registro de log
    async fn write(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Filtra si debe escribir este nivel
    fn should_write(&self, _level: Level) -> bool {
        true // Por defecto escribe todos los niveles
    }

    /// Nombre del transport
    fn name(&self) -> &str;
}

/// Transport para consola
pub struct ConsoleTransport {
    colored: bool,
}

impl ConsoleTransport {
    /// Crea transport sin colores
    pub fn new() -> Self {
        Self { colored: false }
    }

    /// Crea transport con colores
    pub fn colored() -> Self {
        Self { colored: true }
    }

    fn format_colored(&self, record: &LogRecord) -> String {
        if !self.colored {
            return record.format(true, false);
        }

        let color_code = match record.level {
            Level::DEBUG => "\x1b[36m", // Cyan
            Level::INFO => "\x1b[32m",  // Green
            Level::WARN => "\x1b[33m",  // Yellow
            Level::ERROR => "\x1b[31m", // Red
            Level::FATAL => "\x1b[35m", // Magenta
        };

        let reset = "\x1b[0m";
        let base = record.format(true, false);

        format!("{}{}{}", color_code, base, reset)
    }
}

#[async_trait]
impl LogTransport for ConsoleTransport {
    async fn write(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let output = self.format_colored(record);
        let mut writer: Box<dyn Write> = match record.level {
            Level::ERROR | Level::FATAL => Box::new(std::io::stderr()),
            _ => Box::new(std::io::stdout()),
        };

        writeln!(writer, "{}", output)?;
        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

/// Transport para archivo
pub struct FileTransport {
    path: std::path::PathBuf,
    file: std::sync::Mutex<Option<std::fs::File>>,
}

impl FileTransport {
    /// Crea transport para archivo
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
            file: std::sync::Mutex::new(None),
        }
    }

    fn ensure_file(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut file_guard = self.file.lock().unwrap();
        if file_guard.is_none() {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
            *file_guard = Some(file);
        }
        Ok(())
    }
}

#[async_trait]
impl LogTransport for FileTransport {
    async fn write(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.ensure_file()?;
        let mut file_guard = self.file.lock().unwrap();
        if let Some(file) = file_guard.as_mut() {
            let output = record.format(true, false);
            writeln!(file, "{}", output)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "file"
    }
}

impl Drop for FileTransport {
    fn drop(&mut self) {
        // El archivo se cierra automáticamente cuando se libera el Mutex
    }
}

/// Transport para HTTP (envía logs a endpoint remoto)
pub struct HttpTransport {
    client: reqwest::Client,
    url: String,
    headers: reqwest::header::HeaderMap,
}

impl HttpTransport {
    /// Crea transport HTTP
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.into(),
            headers: reqwest::header::HeaderMap::new(),
        }
    }

    /// Agrega header personalizado
    pub fn with_header(mut self, key: impl Into<reqwest::header::HeaderName>, value: impl Into<reqwest::header::HeaderValue>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Agrega header de autorización
    pub fn with_auth_bearer(mut self, token: impl Into<String>) -> Self {
        let value = format!("Bearer {}", token.into());
        self.headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&value).unwrap(),
        );
        self
    }
}

#[async_trait]
impl LogTransport for HttpTransport {
    async fn write(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = record.to_json()?;
        let mut request = self.client
            .post(&self.url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(json);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(format!("HTTP request failed with status: {}", response.status()).into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "http"
    }
}

/// Transport para syslog (Unix systems)
#[cfg(unix)]
pub struct SyslogTransport {
    facility: syslog::Facility,
    sender: std::sync::Mutex<Option<syslog::Logger>>,
}

#[cfg(unix)]
impl SyslogTransport {
    /// Crea transport syslog
    pub fn new(facility: syslog::Facility) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            facility,
            sender: std::sync::Mutex::new(None),
        })
    }

    fn ensure_logger(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut logger_guard = self.sender.lock().unwrap();
        if logger_guard.is_none() {
            let logger = syslog::Logger::new(
                self.facility,
                &[syslog::LogOption::LOG_PID],
                syslog::LogTarget::System,
            )?;
            *logger_guard = Some(logger);
        }
        Ok(())
    }
}

#[cfg(unix)]
#[async_trait]
impl LogTransport for SyslogTransport {
    async fn write(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.ensure_logger()?;
        let mut logger_guard = self.sender.lock().unwrap();
        if let Some(logger) = logger_guard.as_mut() {
            let priority = match record.level {
                Level::DEBUG => syslog::LogLevel::Debug,
                Level::INFO => syslog::LogLevel::Info,
                Level::WARN => syslog::LogLevel::Warning,
                Level::ERROR => syslog::LogLevel::Error,
                Level::FATAL => syslog::LogLevel::Critical,
            };

            let message = record.format(false, false);
            logger.log(priority, &message)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "syslog"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[tokio::test]
    async fn test_console_transport() {
        let transport = ConsoleTransport::new();
        let record = LogRecord::new(Level::INFO, "Test message", "test");

        // Solo verifica que no paniquea
        let result = transport.write(&record).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_transport() {
        let temp_file = std::env::temp_dir().join("test_log.txt");
        let transport = FileTransport::new(&temp_file);
        let record = LogRecord::new(Level::INFO, "File test", "test");

        let result = transport.write(&record).await;
        assert!(result.is_ok());

        // Verifica que el archivo existe y contiene el mensaje
        let mut content = String::new();
        std::fs::File::open(&temp_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert!(content.contains("File test"));

        // Limpia
        std::fs::remove_file(temp_file).unwrap();
    }

    #[tokio::test]
    async fn test_http_transport_mock() {
        // Nota: Este test requiere un servidor mock
        // En producción, usar un mock server o integration test
        let transport = HttpTransport::new("http://localhost:8080/logs");
        assert_eq!(transport.url, "http://localhost:8080/logs");
    }

    #[test]
    fn test_log_record_format() {
        let record = LogRecord::new(Level::WARN, "Warning message", "test_logger")
            .with_metadata("key", "value");

        let formatted = record.format(true, false);
        assert!(formatted.contains("[WARN]"));
        assert!(formatted.contains("[test_logger]"));
        assert!(formatted.contains("Warning message"));
        assert!(formatted.contains("{key=value}"));
    }
}