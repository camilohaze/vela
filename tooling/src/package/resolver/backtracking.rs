//! Backtracking Algorithm for Dependency Resolution
//!
//! This module implements a backtracking algorithm as an alternative or complement
//! to the SAT solver. It's particularly useful for smaller dependency graphs or
//! when the SAT solver encounters performance issues.

use crate::common::Error;
use crate::package::resolver::constraints::VersionConstraint;
use crate::package::resolver::graph::{DependencyGraph, PackageId};
use semver::Version;
use std::collections::{HashMap, HashSet, VecDeque};

/// State of the backtracking search
#[derive(Debug, Clone)]
pub struct BacktrackState {
    pub assignments: HashMap<PackageId, Version>,
    pub constraint_violations: Vec<ConstraintViolation>,
    pub depth: usize,
}

/// A constraint violation found during resolution
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub package: PackageId,
    pub required_constraint: VersionConstraint,
    pub conflicting_version: Version,
}

/// Backtracking resolver for dependency resolution
#[derive(Debug)]
pub struct BacktrackingResolver {
    pub graph: DependencyGraph,
    pub max_depth: usize,
    pub conflict_history: Vec<ConstraintViolation>,
}

impl BacktrackingResolver {
    pub fn new(graph: DependencyGraph) -> Self {
        Self {
            graph,
            max_depth: 100, // Reasonable default
            conflict_history: Vec::new(),
        }
    }

    /// Resolve dependencies using backtracking algorithm
    pub fn resolve(&mut self) -> Result<HashMap<PackageId, Version>, Error> {
        let mut state = BacktrackState {
            assignments: HashMap::new(),
            constraint_violations: Vec::new(),
            depth: 0,
        };

        // Start with root dependencies
        let root_packages: Vec<PackageId> = self.graph.root_dependencies.clone();

        match self.backtrack(state, root_packages) {
            Some(final_state) => {
                // Validate final solution
                self.validate_solution(&final_state.assignments)?;
                Ok(final_state.assignments)
            }
            None => Err(Error::Unsatisfiable),
        }
    }

    /// Recursive backtracking function
    fn backtrack(
        &mut self,
        mut state: BacktrackState,
        remaining_packages: Vec<PackageId>,
    ) -> Option<BacktrackState> {
        // Check depth limit
        if state.depth >= self.max_depth {
            return None;
        }

        // If no more packages to process, we have a solution
        if remaining_packages.is_empty() {
            return Some(state);
        }

        // Get next package to process
        let current_package = remaining_packages[0].clone();
        let remaining = remaining_packages[1..].to_vec();

        // Get possible versions for this package
        let node = match self.graph.nodes.get(&current_package) {
            Some(node) => node,
            None => return None,
        };

        // Try each version in order (preferring higher versions first)
        let mut versions = node.package.versions.clone();
        versions.sort_by(|a, b| b.cmp(a)); // Higher versions first

        for version in versions {
            // Check if this assignment violates constraints
            if let Some(violation) = self.check_constraint_violation(&current_package, &version, &state.assignments) {
                state.constraint_violations.push(violation);
                continue; // Try next version
            }

            // Make assignment
            state.assignments.insert(current_package.clone(), version.clone());
            state.depth += 1;

            // Get new dependencies introduced by this version
            let new_deps = self.get_new_dependencies(&current_package, &version);

            // Recurse with remaining packages plus new dependencies
            let mut next_remaining = remaining.clone();
            for (dep_package, _) in &new_deps {
                if !state.assignments.contains_key(dep_package) {
                    next_remaining.push(dep_package.clone());
                }
            }

            // Try to resolve remaining dependencies
            if let Some(solution) = self.backtrack(state.clone(), next_remaining) {
                return Some(solution);
            }

            // Backtrack: remove assignment
            state.assignments.remove(&current_package);
            state.depth -= 1;
        }

        // No solution found for this branch
        None
    }

    /// Check if assigning a version to a package violates any constraints
    fn check_constraint_violation(
        &self,
        package: &PackageId,
        version: &Version,
        assignments: &HashMap<PackageId, Version>,
    ) -> Option<ConstraintViolation> {
        // Check constraints on this package
        if let Some(node) = self.graph.nodes.get(package) {
            for constraint in &node.constraints {
                if !constraint.satisfies(version) {
                    return Some(ConstraintViolation {
                        package: package.clone(),
                        required_constraint: constraint.clone(),
                        conflicting_version: version.clone(),
                    });
                }
            }
        }

        // Check if this assignment conflicts with existing assignments
        // (same package different version - shouldn't happen in our setup)
        if let Some(existing_version) = assignments.get(package) {
            if existing_version != version {
                return Some(ConstraintViolation {
                    package: package.clone(),
                    required_constraint: VersionConstraint::Exact(existing_version.clone()),
                    conflicting_version: version.clone(),
                });
            }
        }

        // Check constraints from dependent packages
        for dependent in self.graph.get_dependents(package) {
            if let Some(assigned_version) = assignments.get(&dependent) {
                // Find the constraint that dependent puts on this package
                if let Some((_, constraint)) = self.graph.get_dependencies(&dependent)
                    .iter()
                    .find(|(dep, _)| dep == package) {
                    if !constraint.satisfies(version) {
                        return Some(ConstraintViolation {
                            package: dependent.clone(),
                            required_constraint: constraint.clone(),
                            conflicting_version: version.clone(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Get dependencies introduced by a specific package version
    fn get_new_dependencies(&self, package: &PackageId, version: &Version) -> Vec<(PackageId, VersionConstraint)> {
        if let Some(node) = self.graph.nodes.get(package) {
            // In a real implementation, we'd have version-specific dependencies
            // For now, use the package's general dependencies
            node.package.dependencies.iter()
                .map(|(name, constraint)| {
                    (PackageId::new(name.clone()), constraint.clone())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Validate that the solution satisfies all constraints
    fn validate_solution(&self, assignments: &HashMap<PackageId, Version>) -> Result<(), Error> {
        for (package, version) in assignments {
            if let Some(violation) = self.check_constraint_violation(package, version, assignments) {
                return Err(Error::ConstraintViolation {
                    package: package.name.clone(),
                    version: violation.conflicting_version.to_string(),
                    constraint: violation.required_constraint.to_string(),
                });
            }
        }

        // Check that all dependencies are satisfied
        for (package, version) in assignments {
            let deps = self.get_new_dependencies(package, version);
            for (dep_package, constraint) in deps {
                if let Some(dep_version) = assignments.get(&dep_package) {
                    if !constraint.satisfies(dep_version) {
                        return Err(Error::UnsatisfiableDependency {
                            dependent: package.name.clone(),
                            dependency: dep_package.name.clone(),
                            required: constraint.to_string(),
                            available: dep_version.to_string(),
                        });
                    }
                } else {
                    return Err(Error::MissingDependency {
                        dependent: package.name.clone(),
                        dependency: dep_package.name.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get conflict resolution suggestions
    pub fn get_conflict_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Analyze constraint violations to suggest fixes
        for violation in &self.conflict_history {
            suggestions.push(format!(
                "Package '{}' version {} violates constraint {} from dependent package '{}'",
                violation.package.name,
                violation.conflicting_version,
                violation.required_constraint,
                // We don't have the dependent info in ConstraintViolation
                "unknown"
            ));
        }

        if suggestions.is_empty() {
            suggestions.push("No conflicts detected in resolution history".to_string());
        }

        suggestions
    }
}

/// Hybrid resolver that combines SAT solving with backtracking
pub struct HybridResolver {
    pub sat_solver: super::solver::SATSolver,
    pub backtracker: BacktrackingResolver,
}

impl HybridResolver {
    pub fn new(graph: DependencyGraph) -> Self {
        Self {
            sat_solver: super::solver::SATSolver::new(),
            backtracker: BacktrackingResolver::new(graph.clone()),
        }
    }

    /// Try SAT solver first, fall back to backtracking if it fails
    pub fn resolve(&mut self) -> Result<HashMap<PackageId, Version>, Error> {
        // Try SAT solver first
        match self.sat_solver.solve() {
            Ok(solution) => Ok(solution),
            Err(_) => {
                // Fall back to backtracking
                self.backtracker.resolve()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn test_backtracking_resolver_creation() {
        let graph = DependencyGraph::new();
        let resolver = BacktrackingResolver::new(graph);
        assert_eq!(resolver.max_depth, 100);
        assert!(resolver.conflict_history.is_empty());
    }

    #[test]
    fn test_constraint_violation_detection() {
        let mut graph = DependencyGraph::new();

        let package_id = PackageId::new("test-package".to_string());
        let package_info = crate::package::resolver::graph::PackageInfo {
            id: package_id.clone(),
            versions: vec![Version::parse("1.0.0").unwrap(), Version::parse("2.0.0").unwrap()],
            latest_version: Version::parse("2.0.0").unwrap(),
            dependencies: HashMap::new(),
        };

        graph.add_node(package_id.clone(), package_info).unwrap();

        let mut resolver = BacktrackingResolver::new(graph);
        let assignments = HashMap::new();

        // Test with version that satisfies constraints (no violation)
        let constraint = VersionConstraint::parse("^1.0.0").unwrap();
        let mut node = resolver.graph.nodes.get_mut(&package_id).unwrap();
        node.constraints.push(constraint);

        let violation = resolver.check_constraint_violation(&package_id, &Version::parse("1.0.0").unwrap(), &assignments);
        assert!(violation.is_none());

        // Test with version that violates constraints
        let violation = resolver.check_constraint_violation(&package_id, &Version::parse("3.0.0").unwrap(), &assignments);
        assert!(violation.is_some());
    }

    #[test]
    fn test_simple_resolution() {
        let mut graph = DependencyGraph::new();

        // Create a simple package with no dependencies
        let package_id = PackageId::new("simple-package".to_string());
        let package_info = crate::package::resolver::graph::PackageInfo {
            id: package_id.clone(),
            versions: vec![Version::parse("1.0.0").unwrap()],
            latest_version: Version::parse("1.0.0").unwrap(),
            dependencies: HashMap::new(),
        };

        graph.add_node(package_id.clone(), package_info).unwrap();
        graph.add_root_dependency(package_id.clone());

        let mut resolver = BacktrackingResolver::new(graph);
        let result = resolver.resolve();

        assert!(result.is_ok());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 1);
        assert!(assignments.contains_key(&package_id));
    }
}