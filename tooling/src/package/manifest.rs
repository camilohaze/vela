/*!
Vela.toml manifest parser
*/

use crate::common::{Error, FileSystem, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Package manifest (Vela.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, String>,
}

/// Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub edition: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl Manifest {
    /// Load manifest from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = FileSystem::read_to_string(path.as_ref())?;
        toml::from_str(&contents).map_err(|e| Error::TomlParse(e))
    }

    /// Save manifest to file
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        FileSystem::write(path, &contents)
    }

    /// Create a new manifest
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            package: PackageInfo {
                name: name.into(),
                version: version.into(),
                authors: Vec::new(),
                edition: "2025".to_string(),
                license: Some("MIT OR Apache-2.0".to_string()),
                description: None,
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, name: impl Into<String>, version: impl Into<String>) {
        self.dependencies.insert(name.into(), version.into());
    }

    /// Remove a dependency
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some()
    }

    /// Get all dependencies
    pub fn all_dependencies(&self) -> impl Iterator<Item = (&String, &String)> {
        self.dependencies.iter().chain(self.dev_dependencies.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_manifest() {
        let manifest = Manifest::new("my-package", "0.1.0");
        assert_eq!(manifest.package.name, "my-package");
        assert_eq!(manifest.package.version, "0.1.0");
    }

    #[test]
    fn test_add_dependency() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_dependency("http", "^2.0");

        assert_eq!(manifest.dependencies.get("http"), Some(&"^2.0".to_string()));
    }

    #[test]
    fn test_remove_dependency() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_dependency("http", "^2.0");

        assert!(manifest.remove_dependency("http"));
        assert!(!manifest.dependencies.contains_key("http"));
    }

    #[test]
    fn test_to_file_and_from_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("Vela.toml");

        let mut manifest = Manifest::new("test-package", "1.0.0");
        manifest.add_dependency("json", "1.0");

        manifest.to_file(&path).unwrap();
        let loaded = Manifest::from_file(&path).unwrap();

        assert_eq!(loaded.package.name, "test-package");
        assert_eq!(loaded.dependencies.get("json"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_all_dependencies() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_dependency("http", "2.0");
        manifest.dev_dependencies.insert("test-utils".to_string(), "0.1".to_string());

        let all: Vec<_> = manifest.all_dependencies().collect();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_from_file_missing() {
        let result = Manifest::from_file("nonexistent.toml");
        assert!(result.is_err());
    }
}
