//! Core Dependency Resolution Algorithm
//!
//! This module contains the main algorithm functions for dependency resolution,
//! including graph building, constraint application, conflict resolution, and optimization.

use crate::common::Error;
use crate::package::manifest::Manifest;
use crate::package::resolver::constraints::VersionConstraint;
use crate::package::resolver::graph::{DependencyGraph, PackageId, PackageInfo};
use crate::package::resolver::solver::SATSolver;
use crate::package::resolver::backtracking::BacktrackingResolver;
use crate::package::resolver::{Resolution, ResolvedPackage, Conflict};
use semver::Version;
use std::collections::HashMap;

/// Build the initial dependency graph from manifest
pub fn build_dependency_graph(manifest: &Manifest) -> Result<DependencyGraph, Error> {
    let mut graph = DependencyGraph::new();

    // Add root dependencies from manifest
    for (name, constraint_str) in &manifest.dependencies {
        let constraint = VersionConstraint::parse(constraint_str)?;
        let package_id = PackageId::new(name.clone());

        // For now, create a basic package info
        // In a real implementation, this would fetch from registry
        let package_info = create_mock_package_info(&package_id, &constraint);

        graph.add_node(package_id.clone(), package_info)?;
        graph.add_root_dependency(package_id);
    }

    // TODO: Recursively add transitive dependencies
    // This would involve fetching package metadata from registry

    Ok(graph)
}

/// Apply version constraints to the dependency graph
pub fn apply_version_constraints(mut graph: DependencyGraph) -> Result<DependencyGraph, Error> {
    // For each package, collect all constraints from its dependents
    for (package_id, node) in &mut graph.nodes {
        let mut all_constraints = Vec::new();

        // Add constraints from dependents
        for dependent in graph.get_dependents(package_id) {
            for (dep_package, constraint) in graph.get_dependencies(&dependent) {
                if dep_package == *package_id {
                    all_constraints.push(constraint);
                }
            }
        }

        // TODO: Merge constraints (find intersection)
        // For now, just add them all
        node.constraints.extend(all_constraints);
    }

    Ok(graph)
}

/// Resolve conflicts using conflict-driven search
pub fn resolve_with_conflict_driven_search(
    graph: DependencyGraph,
    _solver: &SATSolver,
    _backtracker: &BacktrackingResolver,
) -> Result<Resolution, Error> {
    // Create a hybrid resolver for this graph
    let mut hybrid_resolver = crate::package::resolver::backtracking::HybridResolver::new(graph);

    // Try to resolve
    match hybrid_resolver.resolve() {
        Ok(assignments) => {
            // Convert to Resolution format
            let mut packages = HashMap::new();
            let conflicts = Vec::new(); // No conflicts if resolution succeeded

            for (package_id, version) in assignments {
                let resolved_package = ResolvedPackage {
                    name: package_id.name,
                    version,
                    dependencies: HashMap::new(), // TODO: Fill in actual dependencies
                };
                packages.insert(package_id.name, resolved_package);
            }

            Ok(Resolution {
                packages,
                conflicts,
            })
        }
        Err(Error::Unsatisfiable) => {
            // Extract conflicts from the resolver
            let conflicts = hybrid_resolver.backtracker.get_conflict_suggestions()
                .into_iter()
                .map(|suggestion| Conflict {
                    package: "unknown".to_string(), // TODO: Extract from suggestion
                    constraints: Vec::new(), // TODO: Extract actual constraints
                })
                .collect();

            Ok(Resolution {
                packages: HashMap::new(),
                conflicts,
            })
        }
        Err(e) => Err(e),
    }
}

/// Optimize version selection (prefer newer versions, minimize changes, etc.)
pub fn optimize_version_selection(resolution: Resolution) -> Result<Resolution, Error> {
    // For now, just return the resolution as-is
    // Future optimizations could include:
    // - Preferring newer compatible versions
    // - Minimizing the number of version changes
    // - Considering security updates
    // - Balancing download size

    Ok(resolution)
}

/// Create mock package info for testing (replace with real registry calls)
fn create_mock_package_info(package_id: &PackageId, constraint: &VersionConstraint) -> PackageInfo {
    // Generate some mock versions that satisfy the constraint
    let base_versions = vec![
        Version::parse("1.0.0").unwrap(),
        Version::parse("1.1.0").unwrap(),
        Version::parse("2.0.0").unwrap(),
        Version::parse("2.1.0").unwrap(),
    ];

    let satisfying_versions: Vec<Version> = base_versions
        .into_iter()
        .filter(|v| constraint.satisfies(v))
        .collect();

    let latest_version = satisfying_versions.last()
        .cloned()
        .unwrap_or_else(|| Version::parse("1.0.0").unwrap());

    PackageInfo {
        id: package_id.clone(),
        versions: satisfying_versions,
        latest_version,
        dependencies: HashMap::new(), // No mock dependencies for now
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::manifest::Manifest;

    #[test]
    fn test_build_dependency_graph() {
        let mut manifest = Manifest::default();
        manifest.dependencies.insert("lodash".to_string(), "^4.0.0".to_string());

        let result = build_dependency_graph(&manifest);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert!(!graph.root_dependencies.is_empty());
    }

    #[test]
    fn test_apply_constraints() {
        let graph = DependencyGraph::new();
        let result = apply_version_constraints(graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_selection() {
        let resolution = Resolution {
            packages: HashMap::new(),
            conflicts: Vec::new(),
        };

        let result = optimize_version_selection(resolution);
        assert!(result.is_ok());
    }
}