/*!
Error types for the i18n system.
*/

use thiserror::Error;

/// Result type alias for i18n operations
pub type Result<T> = std::result::Result<T, I18nError>;

/// Errors that can occur in the i18n system
#[derive(Error, Debug)]
pub enum I18nError {
    #[error("Translation key not found: {key}")]
    TranslationNotFound { key: String },

    #[error("Locale not supported: {locale}")]
    UnsupportedLocale { locale: String },

    #[error("Failed to load translation file: {path}")]
    FileLoadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse translation file: {path}")]
    ParseError {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Invalid interpolation syntax: {syntax}")]
    InterpolationError { syntax: String },

    #[error("Pluralization rule not found for locale: {locale}")]
    PluralizationError { locale: String },

    #[error("Formatting error: {message}")]
    FormattingError { message: String },

    #[error("Hot reload error: {message}")]
    HotReloadError { message: String },

    #[error("IO error: {message}")]
    IoError {
        message: String,
        #[source]
        source: std::io::Error,
    },
}

impl I18nError {
    /// Create a new translation not found error
    pub fn translation_not_found<S: Into<String>>(key: S) -> Self {
        Self::TranslationNotFound { key: key.into() }
    }

    /// Create a new unsupported locale error
    pub fn unsupported_locale<S: Into<String>>(locale: S) -> Self {
        Self::UnsupportedLocale { locale: locale.into() }
    }

    /// Create a new file load error
    pub fn file_load_error<P: AsRef<std::path::Path>>(path: P, source: std::io::Error) -> Self {
        Self::FileLoadError {
            path: path.as_ref().to_string_lossy().to_string(),
            source,
        }
    }

    /// Create a new parse error
    pub fn parse_error<P: AsRef<std::path::Path>>(path: P, source: serde_json::Error) -> Self {
        Self::ParseError {
            path: path.as_ref().to_string_lossy().to_string(),
            source,
        }
    }

    /// Create a new interpolation error
    pub fn interpolation_error<S: Into<String>>(syntax: S) -> Self {
        Self::InterpolationError { syntax: syntax.into() }
    }

    /// Create a new pluralization error
    pub fn pluralization_error<S: Into<String>>(locale: S) -> Self {
        Self::PluralizationError { locale: locale.into() }
    }

    /// Create a new formatting error
    pub fn formatting_error<S: Into<String>>(message: S) -> Self {
        Self::FormattingError { message: message.into() }
    }

    /// Create a new hot reload error
    pub fn hot_reload_error<S: Into<String>>(message: S) -> Self {
        Self::HotReloadError { message: message.into() }
    }

    /// Create a new IO error
    pub fn io_error<S: Into<String>>(message: S, source: std::io::Error) -> Self {
        Self::IoError { message: message.into(), source }
    }
}