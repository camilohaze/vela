//! Sistema de Logging Estructurado para Vela
//!
//! Este módulo implementa el sistema de logging definido en ADR-113L,
//! proporcionando logging estructurado con metadata, múltiples transports
//! y configuración granular.

pub mod logger;
pub mod transport;
pub mod level;
pub mod config;
pub mod record;

pub use logger::Logger;
pub use transport::{LogTransport, ConsoleTransport, FileTransport};
pub use level::Level;
pub use config::LogConfig;
pub use record::LogRecord;