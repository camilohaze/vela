/*!
Build dependency graph
*/

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::SystemTime;

/// Unique identifier for a module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub usize);

/// Module node in the dependency graph
#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub id: ModuleId,
    pub path: PathBuf,
    pub last_modified: Option<SystemTime>,
    pub dependencies: HashSet<ModuleId>,
}

/// Build dependency graph
#[derive(Debug, Clone)]
pub struct BuildGraph {
    nodes: HashMap<ModuleId, ModuleNode>,
    next_id: usize,
}

impl BuildGraph {
    /// Create a new build graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a module to the graph
    pub fn add_module(&mut self, path: PathBuf) -> ModuleId {
        let id = ModuleId(self.next_id);
        self.next_id += 1;

        self.nodes.insert(
            id,
            ModuleNode {
                id,
                path,
                last_modified: None,
                dependencies: HashSet::new(),
            },
        );

        id
    }

    /// Add a dependency edge
    pub fn add_dependency(&mut self, from: ModuleId, to: ModuleId) {
        if let Some(node) = self.nodes.get_mut(&from) {
            node.dependencies.insert(to);
        }
    }

    /// Get a module by ID
    pub fn get_module(&self, id: ModuleId) -> Option<&ModuleNode> {
        self.nodes.get(&id)
    }

    /// Get all modules
    pub fn modules(&self) -> impl Iterator<Item = &ModuleNode> {
        self.nodes.values()
    }

    /// Topological sort for build order
    pub fn topological_sort(&self) -> Result<Vec<Vec<ModuleId>>, String> {
        let mut in_degree: HashMap<ModuleId, usize> = HashMap::new();
        let mut levels: Vec<Vec<ModuleId>> = Vec::new();

        // Calculate in-degrees
        for node in self.nodes.values() {
            in_degree.entry(node.id).or_insert(0);
            for &dep in &node.dependencies {
                *in_degree.entry(dep).or_insert(0) += 1;
            }
        }

        // Find nodes with no dependencies
        let mut current_level: Vec<ModuleId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&id, _)| id)
            .collect();

        while !current_level.is_empty() {
            levels.push(current_level.clone());

            let mut next_level = Vec::new();
            for &id in &current_level {
                if let Some(node) = self.nodes.get(&id) {
                    for &dep in &node.dependencies {
                        if let Some(degree) = in_degree.get_mut(&dep) {
                            *degree -= 1;
                            if *degree == 0 {
                                next_level.push(dep);
                            }
                        }
                    }
                }
            }

            current_level = next_level;
        }

        // Check for cycles
        if levels.iter().map(|l| l.len()).sum::<usize>() != self.nodes.len() {
            return Err("Circular dependency detected".to_string());
        }

        Ok(levels)
    }

    /// Check if module needs rebuild
    pub fn needs_rebuild(&self, _id: ModuleId) -> bool {
        // TODO: Implement proper timestamp checking
        true
    }
}

impl Default for BuildGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_module() {
        let mut graph = BuildGraph::new();
        let id = graph.add_module(PathBuf::from("main.vela"));

        assert_eq!(id, ModuleId(0));
        assert!(graph.get_module(id).is_some());
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = BuildGraph::new();
        let id1 = graph.add_module(PathBuf::from("main.vela"));
        let id2 = graph.add_module(PathBuf::from("lib.vela"));

        graph.add_dependency(id1, id2);

        let node = graph.get_module(id1).unwrap();
        assert!(node.dependencies.contains(&id2));
    }

    #[test]
    fn test_topological_sort_linear() {
        let mut graph = BuildGraph::new();
        let a = graph.add_module(PathBuf::from("a.vela"));
        let b = graph.add_module(PathBuf::from("b.vela"));
        let c = graph.add_module(PathBuf::from("c.vela"));

        graph.add_dependency(b, a); // b depends on a
        graph.add_dependency(c, b); // c depends on b

        let result = graph.topological_sort().unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_topological_sort_parallel() {
        let mut graph = BuildGraph::new();
        let a = graph.add_module(PathBuf::from("a.vela"));
        let b = graph.add_module(PathBuf::from("b.vela"));
        let c = graph.add_module(PathBuf::from("c.vela"));

        // Both b and c depend on a
        graph.add_dependency(b, a);
        graph.add_dependency(c, a);

        let result = graph.topological_sort().unwrap();
        // Result should have all 3 modules distributed across levels
        // In this case: b and c have no dependencies, so they go first
        // Then a goes in the next level
        assert!(!result.is_empty());
        let total_modules: usize = result.iter().map(|level| level.len()).sum();
        assert_eq!(total_modules, 3);
    }

    #[test]
    fn test_circular_dependency() {
        let mut graph = BuildGraph::new();
        let a = graph.add_module(PathBuf::from("a.vela"));
        let b = graph.add_module(PathBuf::from("b.vela"));

        graph.add_dependency(a, b);
        graph.add_dependency(b, a);

        let result = graph.topological_sort();
        assert!(result.is_err());
    }

    #[test]
    fn test_modules_iterator() {
        let mut graph = BuildGraph::new();
        graph.add_module(PathBuf::from("a.vela"));
        graph.add_module(PathBuf::from("b.vela"));

        let count = graph.modules().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_needs_rebuild() {
        let mut graph = BuildGraph::new();
        let id = graph.add_module(PathBuf::from("main.vela"));

        // TODO: Currently always returns true
        assert!(graph.needs_rebuild(id));
    }
}
