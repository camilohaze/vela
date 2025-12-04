/*!
Common error types for vela-tooling
*/

use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for vela-tooling
#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error
    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("Failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Manifest not found
    #[error("Manifest not found: {path}")]
    #[diagnostic(
        code(vela::manifest::not_found),
        help("Make sure Vela.toml exists in the project root")
    )]
    ManifestNotFound { path: PathBuf },

    /// Invalid manifest
    #[error("Invalid manifest: {message}")]
    #[diagnostic(
        code(vela::manifest::invalid),
        help("Check your Vela.toml syntax")
    )]
    InvalidManifest { message: String },

    /// Dependency resolution error
    #[error("Failed to resolve dependencies: {message}")]
    #[diagnostic(
        code(vela::deps::resolution_failed),
        help("Try running 'vela update' to refresh dependencies")
    )]
    DependencyResolution { message: String },

    /// Package not found in registry
    #[error("Package not found: {name}")]
    #[diagnostic(
        code(vela::registry::not_found),
        help("Check the package name and version")
    )]
    PackageNotFound { name: String },

    /// Build error
    #[error("Build failed: {message}")]
    #[diagnostic(code(vela::build::failed))]
    BuildFailed { message: String },

    /// Cache error
    #[error("Cache error: {message}")]
    CacheError { message: String },

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Version parsing error
    #[error("Invalid version: {0}")]
    VersionParse(#[from] semver::Error),

    /// Project not found
    #[error("Not a Vela project: {path}")]
    #[diagnostic(
        code(vela::project::not_found),
        help("Run 'vela new' to create a new project")
    )]
    ProjectNotFound { path: PathBuf },

    /// Invalid project structure
    #[error("Invalid project structure: {message}")]
    InvalidProject { message: String },
}

impl Error {
    /// Create a ManifestNotFound error
    pub fn manifest_not_found(path: impl Into<PathBuf>) -> Self {
        Self::ManifestNotFound { path: path.into() }
    }

    /// Create an InvalidManifest error
    pub fn invalid_manifest(message: impl Into<String>) -> Self {
        Self::InvalidManifest {
            message: message.into(),
        }
    }

    /// Create a DependencyResolution error
    pub fn dependency_resolution(message: impl Into<String>) -> Self {
        Self::DependencyResolution {
            message: message.into(),
        }
    }

    /// Create a PackageNotFound error
    pub fn package_not_found(name: impl Into<String>) -> Self {
        Self::PackageNotFound { name: name.into() }
    }

    /// Create a BuildFailed error
    pub fn build_failed(message: impl Into<String>) -> Self {
        Self::BuildFailed {
            message: message.into(),
        }
    }

    /// Create a CacheError
    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::CacheError {
            message: message.into(),
        }
    }

    /// Create a ProjectNotFound error
    pub fn project_not_found(path: impl Into<PathBuf>) -> Self {
        Self::ProjectNotFound { path: path.into() }
    }

    /// Create an InvalidProject error
    pub fn invalid_project(message: impl Into<String>) -> Self {
        Self::InvalidProject {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::manifest_not_found("Vela.toml");
        assert!(matches!(err, Error::ManifestNotFound { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = Error::invalid_manifest("Missing package name");
        let msg = err.to_string();
        assert!(msg.contains("Invalid manifest"));
    }

    #[test]
    fn test_dependency_resolution_error() {
        let err = Error::dependency_resolution("Conflicting versions");
        assert!(matches!(err, Error::DependencyResolution { .. }));
    }

    #[test]
    fn test_package_not_found() {
        let err = Error::package_not_found("nonexistent-package");
        let msg = err.to_string();
        assert!(msg.contains("nonexistent-package"));
    }

    #[test]
    fn test_build_failed() {
        let err = Error::build_failed("Compilation error");
        assert!(matches!(err, Error::BuildFailed { .. }));
    }

    #[test]
    fn test_cache_error() {
        let err = Error::cache_error("Cache corruption");
        assert!(matches!(err, Error::CacheError { .. }));
    }

    #[test]
    fn test_project_not_found() {
        let err = Error::project_not_found("/path/to/project");
        assert!(matches!(err, Error::ProjectNotFound { .. }));
    }

    #[test]
    fn test_invalid_project() {
        let err = Error::invalid_project("Missing src directory");
        assert!(matches!(err, Error::InvalidProject { .. }));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        let result = returns_result();
        assert_eq!(result.unwrap(), 42);
    }
}
