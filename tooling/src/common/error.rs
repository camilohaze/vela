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

    /// Unsatisfiable dependency constraints
    #[error("Unsatisfiable dependency constraints")]
    #[diagnostic(
        code(vela::deps::unsatisfiable),
        help("Check for conflicting version requirements")
    )]
    Unsatisfiable,

    /// Invalid version constraint
    #[error("Invalid version constraint: {constraint}")]
    #[diagnostic(
        code(vela::deps::invalid_constraint),
        help("Check version constraint syntax (e.g., ^1.0.0, >=2.0.0)")
    )]
    InvalidVersionConstraint { constraint: String },

    /// Duplicate package in dependency graph
    #[error("Duplicate package: {name}")]
    #[diagnostic(
        code(vela::deps::duplicate_package),
        help("Remove duplicate package declarations")
    )]
    DuplicatePackage { name: String },

    /// Circular dependency detected
    #[error("Circular dependency detected")]
    #[diagnostic(
        code(vela::deps::circular_dependency),
        help("Break the circular dependency by refactoring")
    )]
    CircularDependency,

    /// Version constraint violation
    #[error("Version constraint violation: {package}@{version} violates {constraint}")]
    #[diagnostic(
        code(vela::deps::constraint_violation),
        help("Update version constraints to be compatible")
    )]
    ConstraintViolation {
        package: String,
        version: String,
        constraint: String,
    },

    /// Unsatisfiable constraint in solver
    #[error("Unsatisfiable constraint: {package}@{constraint}")]
    #[diagnostic(
        code(vela::solver::unsatisfiable_constraint),
        help("Review constraint logic")
    )]
    UnsatisfiableConstraint {
        package: String,
        constraint: String,
    },

    /// Solver error
    #[error("Solver error: {message}")]
    #[diagnostic(code(vela::solver::error))]
    SolverError { message: String },

    /// Unsatisfiable dependency
    #[error("Unsatisfiable dependency: {dependent} requires {dependency}@{required} but {available} is available")]
    #[diagnostic(
        code(vela::deps::unsatisfiable_dependency),
        help("Check if the required version exists")
    )]
    UnsatisfiableDependency {
        dependent: String,
        dependency: String,
        required: String,
        available: String,
    },

    /// Missing dependency
    #[error("Missing dependency: {dependent} requires {dependency}")]
    #[diagnostic(
        code(vela::deps::missing),
        help("Add the missing dependency to your manifest")
    )]
    MissingDependency {
        dependent: String,
        dependency: String,
    },
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

    /// Create an Unsatisfiable error
    pub fn unsatisfiable() -> Self {
        Self::Unsatisfiable
    }

    /// Create an InvalidVersionConstraint error
    pub fn invalid_version_constraint(constraint: impl Into<String>) -> Self {
        Self::InvalidVersionConstraint {
            constraint: constraint.into(),
        }
    }

    /// Create a DuplicatePackage error
    pub fn duplicate_package(name: impl Into<String>) -> Self {
        Self::DuplicatePackage { name: name.into() }
    }

    /// Create a CircularDependency error
    pub fn circular_dependency() -> Self {
        Self::CircularDependency
    }

    /// Create a ConstraintViolation error
    pub fn constraint_violation(
        package: impl Into<String>,
        version: impl Into<String>,
        constraint: impl Into<String>,
    ) -> Self {
        Self::ConstraintViolation {
            package: package.into(),
            version: version.into(),
            constraint: constraint.into(),
        }
    }

    /// Create an UnsatisfiableConstraint error
    pub fn unsatisfiable_constraint(
        package: impl Into<String>,
        constraint: impl Into<String>,
    ) -> Self {
        Self::UnsatisfiableConstraint {
            package: package.into(),
            constraint: constraint.into(),
        }
    }

    /// Create a SolverError
    pub fn solver_error(message: impl Into<String>) -> Self {
        Self::SolverError {
            message: message.into(),
        }
    }

    /// Create an UnsatisfiableDependency error
    pub fn unsatisfiable_dependency(
        dependent: impl Into<String>,
        dependency: impl Into<String>,
        required: impl Into<String>,
        available: impl Into<String>,
    ) -> Self {
        Self::UnsatisfiableDependency {
            dependent: dependent.into(),
            dependency: dependency.into(),
            required: required.into(),
            available: available.into(),
        }
    }

    /// Create a MissingDependency error
    pub fn missing_dependency(
        dependent: impl Into<String>,
        dependency: impl Into<String>,
    ) -> Self {
        Self::MissingDependency {
            dependent: dependent.into(),
            dependency: dependency.into(),
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
