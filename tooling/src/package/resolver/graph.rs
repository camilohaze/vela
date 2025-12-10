//! Dependency Graph Structures
//!
//! This module provides data structures for representing and manipulating
//! dependency graphs during resolution.

use crate::common::Error;
use crate::package::resolver::constraints::VersionConstraint;
use std::collections::{HashMap, HashSet};

/// Unique identifier for a package
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: String,
    pub source: PackageSource,
}

/// Source of a package (registry, git, local, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageSource {
    Registry,
    Git(String), // URL
    Local(String), // Path
}

impl PackageId {
    pub fn new(name: String) -> Self {
        Self {
            name,
            source: PackageSource::Registry,
        }
    }

    pub fn from_registry(name: String) -> Self {
        Self {
            name,
            source: PackageSource::Registry,
        }
    }

    pub fn from_git(name: String, url: String) -> Self {
        Self {
            name,
            source: PackageSource::Git(url),
        }
    }

    pub fn from_local(name: String, path: String) -> Self {
        Self {
            name,
            source: PackageSource::Local(path),
        }
    }
}

/// Information about a package
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub id: PackageId,
    pub versions: Vec<semver::Version>,
    pub latest_version: semver::Version,
    pub dependencies: HashMap<String, VersionConstraint>,
}

/// A dependency graph node
#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub package: PackageInfo,
    pub selected_version: Option<semver::Version>,
    pub constraints: Vec<VersionConstraint>,
}

/// The complete dependency graph
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: HashMap<PackageId, DependencyNode>,
    pub edges: HashMap<PackageId, Vec<(PackageId, VersionConstraint)>>,
    pub root_dependencies: Vec<PackageId>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            root_dependencies: Vec::new(),
        }
    }

    /// Add a package node to the graph
    pub fn add_node(&mut self, package_id: PackageId, package_info: PackageInfo) -> Result<(), Error> {
        if self.nodes.contains_key(&package_id) {
            return Err(Error::DuplicatePackage { name: package_id.name });
        }

        let node = DependencyNode {
            package: package_info,
            selected_version: None,
            constraints: Vec::new(),
        };

        self.nodes.insert(package_id, node);
        Ok(())
    }

    /// Add a dependency edge between packages
    pub fn add_edge(&mut self, from: PackageId, to: PackageId, constraint: VersionConstraint) -> Result<(), Error> {
        if !self.nodes.contains_key(&from) {
            return Err(Error::PackageNotFound { name: from.name });
        }

        if !self.nodes.contains_key(&to) {
            return Err(Error::PackageNotFound { name: to.name });
        }

        self.edges.entry(from).or_insert(Vec::new()).push((to, constraint));
        Ok(())
    }

    /// Add a root dependency (directly specified in manifest)
    pub fn add_root_dependency(&mut self, package_id: PackageId) {
        if !self.root_dependencies.contains(&package_id) {
            self.root_dependencies.push(package_id);
        }
    }

    /// Get all packages that depend on a given package
    pub fn get_dependents(&self, package_id: &PackageId) -> Vec<PackageId> {
        self.edges
            .iter()
            .filter_map(|(from, deps)| {
                if deps.iter().any(|(to, _)| to == package_id) {
                    Some(from.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all dependencies of a package
    pub fn get_dependencies(&self, package_id: &PackageId) -> Vec<(PackageId, VersionConstraint)> {
        self.edges.get(package_id).cloned().unwrap_or_default()
    }

    /// Check if the graph has cycles
    pub fn has_cycles(&self) -> bool {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for node in self.nodes.keys() {
            if self.has_cycles_recursive(node, &mut visited, &mut recursion_stack) {
                return true;
            }
        }

        false
    }

    fn has_cycles_recursive(
        &self,
        node: &PackageId,
        visited: &mut HashSet<PackageId>,
        recursion_stack: &mut HashSet<PackageId>,
    ) -> bool {
        if recursion_stack.contains(node) {
            return true;
        }

        if visited.contains(node) {
            return false;
        }

        visited.insert(node.clone());
        recursion_stack.insert(node.clone());

        for (dep, _) in self.get_dependencies(node) {
            if self.has_cycles_recursive(&dep, visited, recursion_stack) {
                return true;
            }
        }

        recursion_stack.remove(node);
        false
    }

    /// Get topological order of packages (for installation)
    pub fn topological_sort(&self) -> Result<Vec<PackageId>, Error> {
        if self.has_cycles() {
            return Err(Error::CircularDependency);
        }

        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for node in &self.root_dependencies {
            self.topological_sort_recursive(node, &mut visited, &mut temp_visited, &mut result);
        }

        // Add any remaining nodes not reachable from root
        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.topological_sort_recursive(node, &mut visited, &mut temp_visited, &mut result);
            }
        }

        // No need to reverse - result already has dependencies first
        Ok(result)
    }

    fn topological_sort_recursive(
        &self,
        node: &PackageId,
        visited: &mut HashSet<PackageId>,
        temp_visited: &mut HashSet<PackageId>,
        result: &mut Vec<PackageId>,
    ) {
        if visited.contains(node) {
            return;
        }

        temp_visited.insert(node.clone());

        for (dep, _) in self.get_dependencies(node) {
            if temp_visited.contains(&dep) {
                // Cycle detected (shouldn't happen as we check beforehand)
                return;
            }
            self.topological_sort_recursive(&dep, visited, temp_visited, result);
        }

        temp_visited.remove(node);
        visited.insert(node.clone());
        result.push(node.clone());
    }

    /// Validate that all constraints are satisfied
    pub fn validate_constraints(&self) -> Result<(), Error> {
        for (package_id, node) in &self.nodes {
            if let Some(selected_version) = &node.selected_version {
                for constraint in &node.constraints {
                    if !constraint.satisfies(selected_version) {
                        return Err(Error::ConstraintViolation {
                            package: package_id.name.clone(),
                            version: selected_version.to_string(),
                            constraint: constraint.to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn test_graph_creation() {
        let graph = DependencyGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut graph = DependencyGraph::new();
        let package_id = PackageId::new("test-package".to_string());
        let package_info = PackageInfo {
            id: package_id.clone(),
            versions: vec![Version::parse("1.0.0").unwrap()],
            latest_version: Version::parse("1.0.0").unwrap(),
            dependencies: HashMap::new(),
        };

        assert!(graph.add_node(package_id.clone(), package_info).is_ok());
        assert!(graph.nodes.contains_key(&package_id));
    }

    #[test]
    fn test_add_duplicate_node() {
        let mut graph = DependencyGraph::new();
        let package_id = PackageId::new("test-package".to_string());
        let package_info = PackageInfo {
            id: package_id.clone(),
            versions: vec![Version::parse("1.0.0").unwrap()],
            latest_version: Version::parse("1.0.0").unwrap(),
            dependencies: HashMap::new(),
        };

        graph.add_node(package_id.clone(), package_info.clone()).unwrap();
        assert!(graph.add_node(package_id, package_info).is_err());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();

        // Create packages: A depends on B, B depends on C
        let a_id = PackageId::new("package-a".to_string());
        let b_id = PackageId::new("package-b".to_string());
        let c_id = PackageId::new("package-c".to_string());

        let a_info = PackageInfo {
            id: a_id.clone(),
            versions: vec![Version::parse("1.0.0").unwrap()],
            latest_version: Version::parse("1.0.0").unwrap(),
            dependencies: HashMap::new(),
        };

        let b_info = PackageInfo {
            id: b_id.clone(),
            versions: vec![Version::parse("1.0.0").unwrap()],
            latest_version: Version::parse("1.0.0").unwrap(),
            dependencies: HashMap::new(),
        };

        let c_info = PackageInfo {
            id: c_id.clone(),
            versions: vec![Version::parse("1.0.0").unwrap()],
            latest_version: Version::parse("1.0.0").unwrap(),
            dependencies: HashMap::new(),
        };

        graph.add_node(a_id.clone(), a_info).unwrap();
        graph.add_node(b_id.clone(), b_info).unwrap();
        graph.add_node(c_id.clone(), c_info).unwrap();

        let constraint = VersionConstraint::parse("^1.0.0").unwrap();
        graph.add_edge(a_id.clone(), b_id.clone(), constraint.clone()).unwrap();
        graph.add_edge(b_id.clone(), c_id.clone(), constraint).unwrap();

        graph.add_root_dependency(a_id.clone());

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
        // Dependencies should come first: C, then B, then A
        assert_eq!(sorted[0], c_id);
        assert_eq!(sorted[1], b_id);
        assert_eq!(sorted[2], a_id);
    }
}