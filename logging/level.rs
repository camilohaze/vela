//! Niveles de Logging
//!
//! Define los niveles estándar de logging con prioridades

use serde::{Deserialize, Serialize};
/// para filtrado y configuración.

use std::fmt;

/// Nivel de logging con prioridad numérica
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    /// Información de debugging detallada (prioridad más baja)
    DEBUG = 10,
    /// Información general de ejecución
    INFO = 20,
    /// Advertencias que no detienen la ejecución
    WARN = 30,
    /// Errores que afectan funcionalidad
    ERROR = 40,
    /// Errores críticos que pueden terminar la aplicación
    FATAL = 50,
}

impl Level {
    /// Convierte string a Level
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Some(Level::DEBUG),
            "INFO" => Some(Level::INFO),
            "WARN" | "WARNING" => Some(Level::WARN),
            "ERROR" => Some(Level::ERROR),
            "FATAL" => Some(Level::FATAL),
            _ => None,
        }
    }

    /// Retorna nombre del nivel en mayúsculas
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::DEBUG => "DEBUG",
            Level::INFO => "INFO",
            Level::WARN => "WARN",
            Level::ERROR => "ERROR",
            Level::FATAL => "FATAL",
        }
    }

    /// Verifica si este nivel debe loggearse dado un nivel mínimo
    pub fn should_log(&self, min_level: Level) -> bool {
        *self >= min_level
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_from_str() {
        assert_eq!(Level::from_str("DEBUG"), Some(Level::DEBUG));
        assert_eq!(Level::from_str("info"), Some(Level::INFO));
        assert_eq!(Level::from_str("WARN"), Some(Level::WARN));
        assert_eq!(Level::from_str("warning"), Some(Level::WARN));
        assert_eq!(Level::from_str("ERROR"), Some(Level::ERROR));
        assert_eq!(Level::from_str("FATAL"), Some(Level::FATAL));
        assert_eq!(Level::from_str("invalid"), None);
    }

    #[test]
    fn test_level_as_str() {
        assert_eq!(Level::DEBUG.as_str(), "DEBUG");
        assert_eq!(Level::INFO.as_str(), "INFO");
        assert_eq!(Level::WARN.as_str(), "WARN");
        assert_eq!(Level::ERROR.as_str(), "ERROR");
        assert_eq!(Level::FATAL.as_str(), "FATAL");
    }

    #[test]
    fn test_level_ordering() {
        assert!(Level::DEBUG < Level::INFO);
        assert!(Level::INFO < Level::WARN);
        assert!(Level::WARN < Level::ERROR);
        assert!(Level::ERROR < Level::FATAL);
    }

    #[test]
    fn test_level_should_log() {
        assert!(Level::DEBUG.should_log(Level::DEBUG));
        assert!(Level::INFO.should_log(Level::DEBUG));
        assert!(Level::ERROR.should_log(Level::WARN));
        assert!(!Level::DEBUG.should_log(Level::INFO));
        assert!(!Level::WARN.should_log(Level::ERROR));
    }
}