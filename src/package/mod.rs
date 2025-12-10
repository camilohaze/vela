//! Package Manager for Vela
//!
//! This module provides functionality for:
//! - Dependency resolution and conflict management
//! - Package installation and management
//! - Registry communication
//! - Lockfile generation and validation

pub mod resolver;
pub mod registry;
pub mod lockfile;
pub mod manifest;

use crate::common::Error;
use resolver::{DependencyResolver, Resolution};
use manifest::Manifest;

/// Main package manager interface
pub struct PackageManager {
    resolver: DependencyResolver,
    registry_client: registry::Client,
}

impl PackageManager {
    /// Create a new package manager instance
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            resolver: DependencyResolver::new()?,
            registry_client: registry::Client::new()?,
        })
    }

    /// Resolve dependencies for a manifest
    pub fn resolve(&self, manifest: &Manifest) -> Result<Resolution, Error> {
        self.resolver.resolve(manifest)
    }

    /// Install resolved dependencies
    pub fn install(&self, resolution: &Resolution) -> Result<(), Error> {
        // Implementation will use the install logic from TASK-103
        // but with resolved versions instead of individual packages
        Ok(())
    }

    /// Update lockfile with resolved dependencies
    pub fn update_lockfile(&self, resolution: &Resolution) -> Result<(), Error> {
        lockfile::write_lockfile(resolution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_creation() {
        let pm = PackageManager::new();
        assert!(pm.is_ok());
    }
}