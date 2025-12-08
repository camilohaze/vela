/*!
# Vela Package Manager

Package management functionality for Vela projects.
Handles dependency resolution, installation, and manifest parsing.
*/

pub mod manifest;
pub mod resolver;
pub mod registry;

// Re-export main types for convenience
pub use manifest::{VelaManifest, ManifestBuilder, VersionRange, BuildTarget, Platform, OptimizationLevel};
pub use resolver::{DependencyResolver, ResolvedDependency, DependencySource, DependencyConflict};
pub use registry::{RegistryClient, RegistryConfig};