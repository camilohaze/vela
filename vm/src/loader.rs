/*!
Bytecode Loader for Vela VM

This module implements loading of .velac bytecode files from the filesystem
and integration with the module system.

Implementation of: VELA-588 - TASK-080
Story: Module Loader
Date: 2025-12-09

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

        // Deserialize the bytecode using bincode
        bincode::deserialize(&bytes).map_err(|e| Error::ImportError {
            module: path.to_string_lossy().to_string(),
            message: format!("Failed to deserialize bytecode: {}", e),
        })
    }

    /// Extract exported symbols from bytecode
    fn extract_exports(&self, bytecode: &Bytecode) -> Result<HashMap<String, usize>, Error> {
        let mut exports = HashMap::new();

        // First, try to get exports from metadata
        if let Some(export_data) = bytecode.metadata.get("exports") {
            // Try to deserialize as HashMap<String, usize>
            match bincode::deserialize::<HashMap<String, usize>>(export_data) {
                Ok(metadata_exports) => {
                    exports.extend(metadata_exports);
                }
                Err(_) => {
                    // If deserialization fails, try to parse as a simple list of strings
                    // This is a fallback for simpler export formats
                    if let Ok(export_names) = bincode::deserialize::<Vec<String>>(export_data) {
                        for (index, name) in export_names.iter().enumerate() {
                            exports.insert(name.clone(), index);
                        }
                    }
                }
            }
        }

        // If no exports found in metadata, try to extract from the main code object
        // The main module code object is typically the first one
        if exports.is_empty() {
            if let Some(main_code) = bytecode.code_objects.first() {
                // Look for exported names in the names table
                // This is a heuristic - in a real implementation, the compiler would
                // mark which names are exported
                for (index, name_idx) in main_code.names.iter().enumerate() {
                    if let Some(name) = bytecode.strings.get(*name_idx as usize) {
                        // For now, export all global names (this is a simplification)
                        // In a real implementation, only explicitly exported names would be included
                        exports.insert(name.clone(), index);
                    }
                }
            }
        }

        Ok(exports)
    }

    /// Clear the module cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Check if a module is loaded
    pub fn is_module_loaded(&self, name: &str) -> bool {
        self.cache.contains_key(name)
    }

    /// Get loaded module by name
    pub fn get_loaded_module(&self, name: &str) -> Option<&LoadedModule> {
        self.cache.get(name)
    }

    /// Get all loaded modules
    pub fn get_loaded_modules(&self) -> Vec<&LoadedModule> {
        self.cache.values().collect()
    }

    /// Validate bytecode integrity
    pub fn validate_bytecode(&self, bytecode: &Bytecode) -> Result<(), Error> {
        // Check magic number
        if bytecode.magic != Bytecode::MAGIC {
            return Err(Error::ImportError {
                module: "unknown".to_string(),
                message: format!("Invalid magic number: {:08x}", bytecode.magic),
            });
        }

        // Check version compatibility (for now, accept any 0.x.x version)
        if bytecode.version.0 != 0 {
            return Err(Error::ImportError {
                module: "unknown".to_string(),
                message: format!("Unsupported version: {}.{}.{}", bytecode.version.0, bytecode.version.1, bytecode.version.2),
            });
        }

        // Basic structure validation
        if bytecode.code_objects.is_empty() {
            return Err(Error::ImportError {
                module: "unknown".to_string(),
                message: "Bytecode contains no code objects".to_string(),
            });
        }

        Ok(())
    }

    /// Save bytecode to file (utility for testing/compilation)
    pub fn save_bytecode(&self, bytecode: &Bytecode, path: &Path) -> Result<(), Error> {
        let data = bincode::serialize(bytecode).map_err(|e| Error::ImportError {
            module: path.to_string_lossy().to_string(),
            message: format!("Failed to serialize bytecode: {}", e),
        })?;

        fs::write(path, data).map_err(|e| Error::ImportError {
            module: path.to_string_lossy().to_string(),
            message: format!("Failed to write bytecode file: {}", e),
        })?;

        Ok(())
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
    use crate::{CodeObject, Constant};

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

    #[test]
    fn test_bytecode_validation() {
        let loader = BytecodeLoader::new();

        // Valid bytecode
        let mut valid_bytecode = Bytecode::new();
        valid_bytecode.code_objects.push(CodeObject::new(0, 0));
        assert!(loader.validate_bytecode(&valid_bytecode).is_ok());

        // Invalid magic number
        let mut invalid_magic = valid_bytecode.clone();
        invalid_magic.magic = 0x12345678;
        assert!(loader.validate_bytecode(&invalid_magic).is_err());

        // Unsupported version
        let mut invalid_version = valid_bytecode.clone();
        invalid_version.version = (1, 0, 0);
        assert!(loader.validate_bytecode(&invalid_version).is_err());

        // No code objects
        let mut no_code = valid_bytecode.clone();
        no_code.code_objects.clear();
        assert!(loader.validate_bytecode(&no_code).is_err());
    }

    #[test]
    fn test_save_and_load_bytecode() {
        let loader = BytecodeLoader::new();

        // Create test bytecode
        let mut bytecode = Bytecode::new();
        bytecode.strings.push("test_module".to_string());
        bytecode.strings.push("test.vela".to_string());

        let mut code_obj = CodeObject::new(0, 1);
        code_obj.arg_count = 1;
        code_obj.local_count = 2;
        code_obj.stack_size = 10;
        code_obj.bytecode = vec![0x00, 0x01, 0x00]; // LoadConst(1)
        code_obj.constants.push(Constant::Int(42));
        code_obj.names.push(0); // "test_var"

        bytecode.code_objects.push(code_obj);

        // Add some exports to metadata
        let exports = HashMap::from([
            ("public_function".to_string(), 0),
            ("public_var".to_string(), 1),
        ]);
        let export_data = bincode::serialize(&exports).unwrap();
        bytecode.metadata.insert("exports".to_string(), export_data);

        // Save to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        loader.save_bytecode(&bytecode, &temp_path).unwrap();

        // Load back
        let loaded_bytecode = loader.load_bytecode_file(&temp_path).unwrap();

        // Verify contents
        assert_eq!(loaded_bytecode.magic, Bytecode::MAGIC);
        assert_eq!(loaded_bytecode.version, (0, 1, 0));
        assert_eq!(loaded_bytecode.strings.len(), 2);
        assert_eq!(loaded_bytecode.strings[0], "test_module");
        assert_eq!(loaded_bytecode.strings[1], "test.vela");
        assert_eq!(loaded_bytecode.code_objects.len(), 1);

        let loaded_code = &loaded_bytecode.code_objects[0];
        assert_eq!(loaded_code.arg_count, 1);
        assert_eq!(loaded_code.local_count, 2);
        assert_eq!(loaded_code.stack_size, 10);
        assert_eq!(loaded_code.bytecode, vec![0x00, 0x01, 0x00]);
        assert_eq!(loaded_code.constants, vec![Constant::Int(42)]);
        assert_eq!(loaded_code.names, vec![0]);
    }

    #[test]
    fn test_extract_exports_from_metadata() {
        let loader = BytecodeLoader::new();

        // Create bytecode with exports in metadata
        let mut bytecode = Bytecode::new();
        let exports = HashMap::from([
            ("function_a".to_string(), 0),
            ("variable_b".to_string(), 1),
            ("class_c".to_string(), 2),
        ]);
        let export_data = bincode::serialize(&exports).unwrap();
        bytecode.metadata.insert("exports".to_string(), export_data);

        let extracted = loader.extract_exports(&bytecode).unwrap();
        assert_eq!(extracted, exports);
    }

    #[test]
    fn test_extract_exports_fallback() {
        let loader = BytecodeLoader::new();

        // Create bytecode without exports metadata but with names
        let mut bytecode = Bytecode::new();
        bytecode.strings.push("var1".to_string());
        bytecode.strings.push("var2".to_string());
        bytecode.strings.push("func1".to_string());

        let mut code_obj = CodeObject::new(0, 0);
        code_obj.names = vec![0, 1, 2]; // indices into string table
        bytecode.code_objects.push(code_obj);

        let extracted = loader.extract_exports(&bytecode).unwrap();

        // Should extract all names from the main code object
        assert_eq!(extracted.get("var1"), Some(&0));
        assert_eq!(extracted.get("var2"), Some(&1));
        assert_eq!(extracted.get("func1"), Some(&2));
    }

    #[test]
    fn test_module_loading_integration() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut loader = BytecodeLoader::with_project_root(temp_dir.path().to_path_buf());

        // Create a test bytecode file
        let mut bytecode = Bytecode::new();
        bytecode.strings.push("test_module".to_string());
        bytecode.strings.push("test.vela".to_string());

        let code_obj = CodeObject::new(0, 1);
        bytecode.code_objects.push(code_obj);

        // Add exports
        let exports = HashMap::from([("test_export".to_string(), 0)]);
        let export_data = bincode::serialize(&exports).unwrap();
        bytecode.metadata.insert("exports".to_string(), export_data);

        // Save the bytecode
        let module_path = temp_dir.path().join("test.velac");
        loader.save_bytecode(&bytecode, &module_path).unwrap();

        // Try to load the module
        let loaded = loader.load_module("test").unwrap();

        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.path, module_path);
        assert_eq!(loaded.exports.get("test_export"), Some(&0));

        // Check cache
        assert!(loader.is_module_loaded("test"));
        assert_eq!(loader.get_loaded_modules().len(), 1);
    }

    #[test]
    fn test_cache_operations() {
        let mut loader = BytecodeLoader::new();

        // Initially empty
        assert!(!loader.is_module_loaded("test"));
        assert_eq!(loader.get_loaded_modules().len(), 0);
        assert!(loader.get_loaded_module("test").is_none());

        // Create a dummy loaded module for testing
        let dummy_module = LoadedModule {
            name: "test".to_string(),
            path: PathBuf::from("dummy.velac"),
            bytecode: Bytecode::new(),
            exports: HashMap::from([("dummy".to_string(), 0)]),
        };

        // Manually insert into cache (normally done by load_module)
        loader.cache.insert("test".to_string(), dummy_module);

        // Now should be available
        assert!(loader.is_module_loaded("test"));
        assert_eq!(loader.get_loaded_modules().len(), 1);
        assert!(loader.get_loaded_module("test").is_some());

        // Clear cache
        loader.clear_cache();
        assert!(!loader.is_module_loaded("test"));
        assert_eq!(loader.get_loaded_modules().len(), 0);
    }

    #[test]
    fn test_corrupted_bytecode_file() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write valid magic but then corrupted data
        temp_file.write_all(&Bytecode::MAGIC.to_le_bytes()).unwrap();
        temp_file.write_all(b"corrupted data that won't deserialize").unwrap();

        let temp_path = temp_file.path().to_path_buf();
        let loader = BytecodeLoader::new();

        let result = loader.load_bytecode_file(&temp_path);
        assert!(result.is_err());

        // Error should mention deserialization failure
        match result.err().unwrap() {
            Error::ImportError { message, .. } => {
                assert!(message.contains("deserialize"));
            }
            _ => panic!("Expected ImportError"),
        }
    }

    #[test]
    fn test_empty_bytecode_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        let loader = BytecodeLoader::new();
        let result = loader.load_bytecode_file(&temp_path);

        assert!(result.is_err());
        match result.err().unwrap() {
            Error::ImportError { message, .. } => {
                assert!(message.contains("too small"));
            }
            _ => panic!("Expected ImportError"),
        }
    }
}