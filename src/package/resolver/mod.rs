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
            let resolved_package = ResolvedPackage {
                name: package_id.name,
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
}