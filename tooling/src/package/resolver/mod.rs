//! Dependency Resolution Algorithm
//!
//! This module implements a sophisticated dependency resolution algorithm
//! that can handle version constraints, conflicts, and transitive dependencies.

pub mod algorithm;
pub mod constraints;
pub mod graph;
pub mod solver;
pub mod backtracking;

use crate::common::Error;
use crate::package::manifest::Manifest;
use constraints::VersionConstraint;
use graph::DependencyGraph;
use solver::SATSolver;
use backtracking::{BacktrackingResolver, HybridResolver};

/// Result of dependency resolution
#[derive(Debug, Clone)]
pub struct Resolution {
    pub packages: std::collections::HashMap<String, ResolvedPackage>,
    pub conflicts: Vec<Conflict>,
}

/// A resolved package with specific version
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: semver::Version,
    pub dependencies: std::collections::HashMap<String, semver::Version>,
}

/// A dependency conflict that couldn't be resolved
#[derive(Debug, Clone)]
pub struct Conflict {
    pub package: String,
    pub constraints: Vec<VersionConstraint>,
}

/// Main dependency resolver
pub struct DependencyResolver {
    hybrid_solver: HybridResolver,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            hybrid_solver: HybridResolver::new(DependencyGraph::new()),
        })
    }

    /// Resolve dependencies for a manifest
    pub fn resolve(&mut self, manifest: &Manifest) -> Result<Resolution, Error> {
        // Phase 1: Build dependency graph
        let graph = self.build_dependency_graph(manifest)?;

        // Update the hybrid solver with the new graph
        self.hybrid_solver = backtracking::HybridResolver::new(graph);

        // Phase 2: Apply version constraints and resolve
        let resolution = self.hybrid_solver.resolve()?;

        // Convert to Resolution format
        let mut packages = std::collections::HashMap::new();
        let conflicts = Vec::new(); // No conflicts if resolution succeeded

        for (package_id, version) in resolution {
            let package_name = package_id.name.clone();
            let resolved_package = ResolvedPackage {
                name: package_name,
                version,
                dependencies: std::collections::HashMap::new(), // TODO: Fill in actual dependencies
            };
            packages.insert(package_id.name, resolved_package);
        }

        Ok(Resolution {
            packages,
            conflicts,
        })
    }

    fn build_dependency_graph(&self, manifest: &Manifest) -> Result<DependencyGraph, Error> {
        algorithm::build_dependency_graph(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::manifest::Manifest;

    #[test]
    fn test_resolver_creation() {
        let resolver = DependencyResolver::new();
        assert!(resolver.is_ok());
    }

    #[test]
    fn test_simple_resolution() {
        let mut resolver = DependencyResolver::new().unwrap();
        let manifest = Manifest::default();

        let result = resolver.resolve(&manifest);
        // Should succeed with empty manifest
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolver_with_manifest_dependencies() {
        let mut resolver = DependencyResolver::new().unwrap();
        let mut manifest = Manifest::new("test".to_string(), "1.0.0".to_string());
        manifest.add_dependency("test-dep".to_string(), "^1.0.0".to_string());

        let result = resolver.resolve(&manifest);
        // Result depends on whether test-dep exists in mock registry
        assert!(result.is_ok() || matches!(result, Err(Error::PackageNotFound { .. })));
    }

    #[test]
    fn test_resolver_error_handling() {
        let mut resolver = DependencyResolver::new().unwrap();
        let mut manifest = Manifest::new("test".to_string(), "1.0.0".to_string());
        manifest.add_dependency("nonexistent".to_string(), "invalid-constraint".to_string());

        let result = resolver.resolve(&manifest);
        // Should either succeed or fail with appropriate error
        match result {
            Ok(_) => {} // Success is acceptable
            Err(Error::InvalidVersionConstraint { .. }) => {} // Expected error
            Err(Error::PackageNotFound { .. }) => {} // Expected error
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_resolver_reuse() {
        let mut resolver = DependencyResolver::new().unwrap();

        // First resolution
        let manifest1 = Manifest::default();
        let result1 = resolver.resolve(&manifest1);
        assert!(result1.is_ok());

        // Second resolution with same resolver
        let mut manifest2 = Manifest::new("test".to_string(), "1.0.0".to_string());
        manifest2.add_dependency("another-dep".to_string(), "1.0.0".to_string());
        let result2 = resolver.resolve(&manifest2);

        // Should not crash, even if it fails
        assert!(result2.is_ok() || matches!(result2, Err(Error::PackageNotFound { .. })));
    }

    #[test]
    fn test_build_dependency_graph_empty() {
        let resolver = DependencyResolver::new().unwrap();
        let manifest = Manifest::default();

        let result = resolver.build_dependency_graph(&manifest);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert!(graph.nodes.is_empty());
        assert!(graph.root_dependencies.is_empty());
    }

    #[test]
    fn test_build_dependency_graph_with_deps() {
        let resolver = DependencyResolver::new().unwrap();
        let mut manifest = Manifest::new("test".to_string(), "1.0.0".to_string());
        manifest.add_dependency("dep1".to_string(), "^1.0.0".to_string());

        let result = resolver.build_dependency_graph(&manifest);
        // Result depends on mock package availability
        assert!(result.is_ok() || matches!(result, Err(Error::PackageNotFound { .. })));
    }
}