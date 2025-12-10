use std::collections::HashMap;

/// Represents a package manifest (like package.json or Cargo.toml)
#[derive(Debug, Clone, Default)]
pub struct Manifest {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Dependencies map: package name -> version constraint string
    pub dependencies: HashMap<String, String>,
}

impl Manifest {
    /// Create a new manifest
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            dependencies: HashMap::new(),
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, name: String, constraint: String) {
        self.dependencies.insert(name, constraint);
    }
}