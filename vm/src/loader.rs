/*!
Bytecode Loader for Vela VM

This module implements loading of .velac bytecode files from the filesystem
and integration with the module system.

Implementation of: VELA-588 - TASK-081
Story: Module Loader
Date: 2025-12-03

Description:
Loads compiled Vela bytecode (.velac) files and makes them available
to the VM for execution through the module system.
*/

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bytecode::Bytecode;
use crate::error::Error;
use crate::module_resolver::ModuleResolver;

/// Bytecode loader for loading .velac files
pub struct BytecodeLoader {
    /// Module resolver for converting module names to paths
    resolver: ModuleResolver,
    /// Cache of loaded modules
    cache: HashMap<String, LoadedModule>,
}

#[derive(Debug, Clone)]
pub struct LoadedModule {
    /// Module name
    pub name: String,
    /// File path where loaded from
    pub path: PathBuf,
    /// Parsed bytecode
    pub bytecode: Bytecode,
    /// Exported symbols (name -> value index in constants)
    pub exports: HashMap<String, usize>,
}

impl BytecodeLoader {
    /// Create a new bytecode loader with default search paths
    pub fn new() -> Self {
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            resolver: ModuleResolver::new(project_root),
            cache: HashMap::new(),
        }
    }

    /// Create loader with custom project root
    pub fn with_project_root(project_root: PathBuf) -> Self {
        Self {
            resolver: ModuleResolver::new(project_root),
            cache: HashMap::new(),
        }
    }

    /// Create loader with custom resolver
    pub fn with_resolver(resolver: ModuleResolver) -> Self {
        Self {
            resolver,
            cache: HashMap::new(),
        }
    }

    /// Load a module by name
    pub fn load_module(&mut self, name: &str) -> Result<&LoadedModule, Error> {
        // Check cache first (without borrowing)
        if self.cache.contains_key(name) {
            return Ok(self.cache.get(name).unwrap());
        }

        // Resolve module path using the resolver
        let module_path = self.resolver.resolve_module(name)?;

        // Load and parse bytecode
        let bytecode = self.load_bytecode_file(&module_path)?;

        // Extract exports
        let exports = self.extract_exports(&bytecode)?;

        // Create loaded module
        let module = LoadedModule {
            name: name.to_string(),
            path: module_path,
            bytecode,
            exports,
        };

        // Cache it
        self.cache.insert(name.to_string(), module);

        Ok(self.cache.get(name).unwrap())
    }

    /// Check if a module is loaded
    pub fn is_module_loaded(&self, name: &str) -> bool {
        self.cache.contains_key(name)
    }

    /// Get a loaded module
    pub fn get_loaded_module(&self, name: &str) -> Option<&LoadedModule> {
        self.cache.get(name)
    }

    /// Get all loaded modules
    pub fn get_loaded_modules(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }

    /// Find module file in search paths
    fn find_module_file(&self, name: &str) -> Result<PathBuf, Error> {
        // This method is now deprecated - use resolver instead
        // Keeping for backward compatibility during transition
        let file_name = format!("{}.velac", name);

        // Try current directory first
        let candidate = PathBuf::from(".").join(&file_name);
        if candidate.exists() {
            return Ok(candidate);
        }

        // Try modules directory
        let candidate = PathBuf::from("modules").join(&file_name);
        if candidate.exists() {
            return Ok(candidate);
        }

        // Try lib directory
        let candidate = PathBuf::from("lib").join(&file_name);
        if candidate.exists() {
            return Ok(candidate);
        }

        Err(Error::ImportError {
            module: name.to_string(),
            message: format!("Module file {}.velac not found in search paths", name),
        })
    }

    /// Load bytecode from file
    fn load_bytecode_file(&self, path: &Path) -> Result<Bytecode, Error> {
        let bytes = fs::read(path).map_err(|e| Error::ImportError {
            module: path.to_string_lossy().to_string(),
            message: format!("Failed to read bytecode file: {}", e),
        })?;

        // Basic validation
        if bytes.len() < 4 {
            return Err(Error::ImportError {
                module: path.to_string_lossy().to_string(),
                message: "Bytecode file too small".to_string(),
            });
        }

        // Check magic number
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != Bytecode::MAGIC {
            return Err(Error::ImportError {
                module: path.to_string_lossy().to_string(),
                message: format!("Invalid magic number: {:08x}, expected {:08x}", magic, Bytecode::MAGIC),
            });
        }

        // For now, return a placeholder - full deserialization would require
        // implementing a proper binary format parser
        // TODO: Implement full bytecode deserialization
        Err(Error::ImportError {
            module: path.to_string_lossy().to_string(),
            message: "Bytecode deserialization not yet implemented".to_string(),
        })
    }

    /// Extract exported symbols from bytecode
    fn extract_exports(&self, bytecode: &Bytecode) -> Result<HashMap<String, usize>, Error> {
        // TODO: Extract exports from bytecode metadata or special code objects
        // For now, return empty exports
        Ok(HashMap::new())
    }

    /// Clear the module cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Add a search path for a specific module prefix
    pub fn add_search_path(&mut self, prefix: &str, path: PathBuf) {
        self.resolver.add_search_path(prefix, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_loader_creation() {
        let loader = BytecodeLoader::new();
        assert!(!loader.is_module_loaded("test"));
        assert_eq!(loader.get_loaded_modules().len(), 0);
    }

    #[test]
    fn test_loader_with_custom_root() {
        let custom_root = PathBuf::from("/tmp");
        let loader = BytecodeLoader::with_project_root(custom_root.clone());
        // Test that resolver was initialized with custom root
        // (We can't directly test internal state, but we can test behavior)
        assert!(!loader.is_module_loaded("test"));
    }

    #[test]
    fn test_module_not_found() {
        let mut loader = BytecodeLoader::new();
        let result = loader.load_module("nonexistent");
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::ImportError { module, .. } => assert_eq!(module, "nonexistent"),
            _ => panic!("Expected ImportError"),
        }
    }

    #[test]
    fn test_invalid_magic_number() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"INVALID").unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let loader = BytecodeLoader::new();
        let result = loader.load_bytecode_file(&temp_path);
        assert!(result.is_err());
    }
}