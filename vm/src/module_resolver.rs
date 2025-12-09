/*!
Module Resolution System for Vela VM

This module implements resolution of module imports with prefixes like:
- 'module:name' -> resolves to project modules
- 'library:name' -> resolves to internal libraries
- 'package:name' -> resolves to external packages
- 'system:name' -> resolves to built-in system modules
- 'extension:name' -> resolves to language extensions
- 'assets:name' -> resolves to asset files

Implementation of: VELA-079 - TASK-079
Story: Module Resolution
Date: 2025-12-09

Description:
Converts module import names with prefixes into actual file system paths
and integrates with the bytecode loader for module loading.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::Error;

/// Module resolver for converting prefixed module names to file paths
pub struct ModuleResolver {
    /// Base project directory
    project_root: PathBuf,
    /// Custom search paths for different module types
    search_paths: HashMap<String, Vec<PathBuf>>,
    /// Cache of resolved module paths
    path_cache: HashMap<String, PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleType {
    /// Project modules (module:*)
    Project,
    /// Internal libraries (library:*)
    Library,
    /// External packages (package:*)
    Package,
    /// Built-in system modules (system:*)
    System,
    /// Language extensions (extension:*)
    Extension,
    /// Asset files (assets:*)
    Asset,
    /// Direct file path (no prefix)
    Direct,
}

impl ModuleType {
    /// Convert a prefix string to ModuleType
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "module" => Some(ModuleType::Project),
            "library" => Some(ModuleType::Library),
            "package" => Some(ModuleType::Package),
            "system" => Some(ModuleType::System),
            "extension" => Some(ModuleType::Extension),
            "assets" => Some(ModuleType::Asset),
            _ => None,
        }
    }
}

impl ModuleResolver {
    /// Create a new module resolver with default configuration
    pub fn new(project_root: PathBuf) -> Self {
        let mut search_paths = HashMap::new();

        // Default search paths for each module type
        search_paths.insert("module".to_string(), vec![
            project_root.join("src"),
            project_root.join("modules"),
        ]);

        search_paths.insert("library".to_string(), vec![
            project_root.join("lib"),
            project_root.join("libraries"),
        ]);

        search_paths.insert("package".to_string(), vec![
            project_root.join("packages"),
            project_root.join("node_modules"),
            project_root.join("vendor"),
        ]);

        search_paths.insert("system".to_string(), vec![
            project_root.join("runtime"),
            project_root.join("stdlib"),
        ]);

        search_paths.insert("extension".to_string(), vec![
            project_root.join("extensions"),
            project_root.join("packages"),
        ]);

        search_paths.insert("assets".to_string(), vec![
            project_root.join("assets"),
            project_root.join("public"),
            project_root.join("static"),
        ]);

        Self {
            project_root,
            search_paths,
            path_cache: HashMap::new(),
        }
    }

    /// Create resolver with custom search paths
    pub fn with_custom_paths(project_root: PathBuf, custom_paths: HashMap<String, Vec<PathBuf>>) -> Self {
        let mut resolver = Self::new(project_root);
        for (prefix, paths) in custom_paths {
            resolver.search_paths.insert(prefix, paths);
        }
        resolver
    }

    /// Resolve a module name to a file path
    pub fn resolve_module(&mut self, module_name: &str) -> Result<PathBuf, Error> {
        // Check cache first
        if let Some(cached_path) = self.path_cache.get(module_name) {
            return Ok(cached_path.clone());
        }

        // Parse module name
        let (module_type, module_path) = self.parse_module_name(module_name)?;

        // Resolve based on type
        let resolved_path = match module_type {
            ModuleType::Project => self.resolve_project_module(&module_path),
            ModuleType::Library => self.resolve_library_module(&module_path),
            ModuleType::Package => self.resolve_package_module(&module_path),
            ModuleType::System => self.resolve_system_module(&module_path),
            ModuleType::Extension => self.resolve_extension_module(&module_path),
            ModuleType::Asset => self.resolve_asset(&module_path),
            ModuleType::Direct => self.resolve_direct_path(&module_path),
        }?;

        // Cache the result
        self.path_cache.insert(module_name.to_string(), resolved_path.clone());

        Ok(resolved_path)
    }

    /// Parse module name into type and path components
    fn parse_module_name(&self, module_name: &str) -> Result<(ModuleType, String), Error> {
        if let Some(colon_pos) = module_name.find(':') {
            let prefix = &module_name[..colon_pos];
            let path = &module_name[colon_pos + 1..];

            let module_type = match prefix {
                "module" => ModuleType::Project,
                "library" => ModuleType::Library,
                "package" => ModuleType::Package,
                "system" => ModuleType::System,
                "extension" => ModuleType::Extension,
                "assets" => ModuleType::Asset,
                _ => return Err(Error::ImportError {
                    module: module_name.to_string(),
                    message: format!("Unknown module prefix: {}", prefix),
                }),
            };

            Ok((module_type, path.to_string()))
        } else {
            // No prefix - treat as direct path or relative import
            Ok((ModuleType::Direct, module_name.to_string()))
        }
    }

    /// Resolve project module (module:*)
    fn resolve_project_module(&self, module_path: &str) -> Result<PathBuf, Error> {
        self.resolve_in_paths(module_path, "module")
    }

    /// Resolve library module (library:*)
    fn resolve_library_module(&self, module_path: &str) -> Result<PathBuf, Error> {
        self.resolve_in_paths(module_path, "library")
    }

    /// Resolve package module (package:*)
    fn resolve_package_module(&self, module_path: &str) -> Result<PathBuf, Error> {
        self.resolve_in_paths(module_path, "package")
    }

    /// Resolve system module (system:*)
    fn resolve_system_module(&self, module_path: &str) -> Result<PathBuf, Error> {
        // System modules are built-in, so we look for .velac files in system paths
        self.resolve_in_paths(module_path, "system")
    }

    /// Resolve extension module (extension:*)
    fn resolve_extension_module(&self, module_path: &str) -> Result<PathBuf, Error> {
        self.resolve_in_paths(module_path, "extension")
    }

    /// Resolve asset (assets:*)
    fn resolve_asset(&self, module_path: &str) -> Result<PathBuf, Error> {
        // Assets don't have .velac extension, return direct path
        if let Some(paths) = self.search_paths.get("assets") {
            for base_path in paths {
                let candidate = base_path.join(module_path);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }

        Err(Error::ImportError {
            module: format!("assets:{}", module_path),
            message: format!("Asset not found: {}", module_path),
        })
    }

    /// Resolve direct path (no prefix)
    fn resolve_direct_path(&self, module_path: &str) -> Result<PathBuf, Error> {
        // Try relative to current directory first
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let candidate = current_dir.join(module_path);

        if candidate.exists() {
            return Ok(candidate);
        }

        // Try as .velac file
        let candidate_velac = current_dir.join(format!("{}.velac", module_path));
        if candidate_velac.exists() {
            return Ok(candidate_velac);
        }

        Err(Error::ImportError {
            module: module_path.to_string(),
            message: format!("Module not found: {}", module_path),
        })
    }

    /// Helper to resolve module in search paths
    fn resolve_in_paths(&self, module_path: &str, prefix: &str) -> Result<PathBuf, Error> {
        if let Some(paths) = self.search_paths.get(prefix) {
            // Try different file extensions and naming conventions
            let candidates = vec![
                format!("{}.velac", module_path),           // Compiled bytecode
                format!("{}/mod.velac", module_path),      // Module directory with mod.velac
                format!("{}/index.velac", module_path),    // Directory with index.velac
                format!("{}.rs", module_path),             // Source file (for development)
                module_path.to_string(),                   // Direct path
            ];

            for base_path in paths {
                for candidate_name in &candidates {
                    let candidate = base_path.join(candidate_name);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }

        Err(Error::ImportError {
            module: format!("{}:{}", prefix, module_path),
            message: format!("Module not found in {} paths: {}", prefix, module_path),
        })
    }

    /// Check if a module can be resolved
    pub fn can_resolve(&mut self, module_name: &str) -> bool {
        self.resolve_module(module_name).is_ok()
    }

    /// Clear the path cache
    pub fn clear_cache(&mut self) {
        self.path_cache.clear();
    }

    /// Add a custom search path for a prefix
    pub fn add_search_path(&mut self, prefix: &str, path: PathBuf) {
        self.search_paths.entry(prefix.to_string())
            .or_insert_with(Vec::new)
            .push(path);
    }

    /// Get all cached resolved paths
    pub fn get_cached_paths(&self) -> &HashMap<String, PathBuf> {
        &self.path_cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_module_name_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        // Test prefixed modules
        assert_eq!(resolver.parse_module_name("module:auth").unwrap(),
                  (ModuleType::Project, "auth".to_string()));
        assert_eq!(resolver.parse_module_name("library:utils").unwrap(),
                  (ModuleType::Library, "utils".to_string()));
        assert_eq!(resolver.parse_module_name("package:lodash").unwrap(),
                  (ModuleType::Package, "lodash".to_string()));
        assert_eq!(resolver.parse_module_name("system:ui").unwrap(),
                  (ModuleType::System, "ui".to_string()));

        // Test direct path
        assert_eq!(resolver.parse_module_name("mymodule").unwrap(),
                  (ModuleType::Direct, "mymodule".to_string()));
    }

    #[test]
    fn test_unknown_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        let result = resolver.parse_module_name("unknown:module");
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::ImportError { message, .. } => {
                assert!(message.contains("Unknown module prefix"));
            }
            _ => panic!("Expected ImportError"),
        }
    }

    #[test]
    fn test_resolver_creation() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        // Check that default search paths are set
        assert!(resolver.search_paths.contains_key("module"));
        assert!(resolver.search_paths.contains_key("library"));
        assert!(resolver.search_paths.contains_key("package"));
        assert!(resolver.search_paths.contains_key("system"));
    }

    #[test]
    fn test_cache_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        // Create a test file
        let test_file = temp_dir.path().join("test.velac");
        std::fs::write(&test_file, "test").unwrap();

        // First resolution should work
        let result1 = resolver.resolve_module("test");
        assert!(result1.is_ok());

        // Check cache
        assert!(resolver.path_cache.contains_key("test"));
        assert_eq!(resolver.path_cache.get("test").unwrap(), &test_file);

        // Second resolution should use cache
        let result2 = resolver.resolve_module("test");
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_resolve_prefixed_modules() {
        let temp_dir = TempDir::new().unwrap();

        // Create module directories
        std::fs::create_dir_all(temp_dir.path().join("modules")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("lib")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("packages")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("extensions")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("assets")).unwrap();

        // Create test files
        std::fs::write(temp_dir.path().join("modules").join("auth.velac"), b"auth bytecode").unwrap();
        std::fs::write(temp_dir.path().join("lib").join("utils.velac"), b"utils bytecode").unwrap();
        std::fs::write(temp_dir.path().join("packages").join("http.velac"), b"http bytecode").unwrap();
        std::fs::write(temp_dir.path().join("extensions").join("charts.velac"), b"charts bytecode").unwrap();
        std::fs::write(temp_dir.path().join("assets").join("logo.png"), b"fake png data").unwrap();

        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        // Test module: prefix
        let result = resolver.resolve_module("module:auth");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("modules").join("auth.velac"));

        // Test library: prefix
        let result = resolver.resolve_module("library:utils");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("lib").join("utils.velac"));

        // Test package: prefix
        let result = resolver.resolve_module("package:http");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("packages").join("http.velac"));

        // Test extension: prefix
        let result = resolver.resolve_module("extension:charts");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("extensions").join("charts.velac"));
    }

    #[test]
    fn test_resolve_asset_files() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir_all(temp_dir.path().join("assets")).unwrap();
        std::fs::write(temp_dir.path().join("assets").join("logo.png"), b"fake png data").unwrap();

        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        // Test assets: prefix (should return direct path without .velac extension)
        let result = resolver.resolve_module("assets:logo.png");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("assets").join("logo.png"));
    }

    #[test]
    fn test_add_custom_search_path() {
        let temp_dir = TempDir::new().unwrap();
        let mut resolver = ModuleResolver::new(temp_dir.path().to_path_buf());

        // Add custom path for module prefix
        let custom_path = temp_dir.path().join("custom_modules");
        std::fs::create_dir_all(&custom_path).unwrap();
        std::fs::write(custom_path.join("custom.velac"), b"custom bytecode").unwrap();

        resolver.add_search_path("module", custom_path.clone());

        let result = resolver.resolve_module("module:custom");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), custom_path.join("custom.velac"));
    }

    #[test]
    fn test_module_type_enum() {
        assert_eq!(ModuleType::from_prefix("module"), Some(ModuleType::Project));
        assert_eq!(ModuleType::from_prefix("library"), Some(ModuleType::Library));
        assert_eq!(ModuleType::from_prefix("package"), Some(ModuleType::Package));
        assert_eq!(ModuleType::from_prefix("system"), Some(ModuleType::System));
        assert_eq!(ModuleType::from_prefix("extension"), Some(ModuleType::Extension));
        assert_eq!(ModuleType::from_prefix("assets"), Some(ModuleType::Asset));
        assert_eq!(ModuleType::from_prefix("invalid"), None);
    }
}