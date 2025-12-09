/*!
Build cache for incremental compilation
*/

use crate::common::{FileSystem, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache entry for a compiled module
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// SHA-256 hash of source file
    pub source_hash: String,
    /// Timestamp of last compilation
    pub timestamp: SystemTime,
    /// Dependencies of this module
    pub dependencies: Vec<PathBuf>,
}

/// Build cache manager
#[derive(Debug, Clone)]
pub struct BuildCache {
    entries: HashMap<PathBuf, CacheEntry>,
}

impl BuildCache {
    /// Create a new build cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Check if cache entry is valid
    pub fn is_valid(&self, path: &Path) -> Result<bool> {
        if let Some(entry) = self.entries.get(path) {
            // Check if source file still exists
            if !FileSystem::exists(path) {
                return Ok(false);
            }

            // Compute current hash
            let current_hash = FileSystem::hash_file(path)?;

            // Check if hash matches
            if current_hash != entry.source_hash {
                return Ok(false);
            }

            // Check dependencies
            for dep in &entry.dependencies {
                if !FileSystem::exists(dep) {
                    return Ok(false);
                }

                let dep_time = FileSystem::modified_time(dep)?;
                if dep_time > entry.timestamp {
                    return Ok(false);
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Store entry in cache
    pub fn store(&mut self, path: PathBuf, dependencies: Vec<PathBuf>) -> Result<()> {
        let source_hash = FileSystem::hash_file(&path)?;
        let timestamp = SystemTime::now();

        self.entries.insert(
            path,
            CacheEntry {
                source_hash,
                timestamp,
                dependencies,
            },
        );

        Ok(())
    }

    /// Get cache entry
    pub fn get(&self, path: &Path) -> Option<&CacheEntry> {
        self.entries.get(path)
    }

    /// Invalidate cache entry
    pub fn invalidate(&mut self, path: &Path) {
        self.entries.remove(path);
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get number of cached entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for BuildCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_cache() {
        let cache = BuildCache::new();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_store_entry() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.vela");
        FileSystem::write(&file, "test content").unwrap();

        let mut cache = BuildCache::new();
        cache.store(file.clone(), vec![]).unwrap();

        assert_eq!(cache.len(), 1);
        assert!(cache.get(&file).is_some());
    }

    #[test]
    fn test_is_valid_fresh() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.vela");
        FileSystem::write(&file, "test content").unwrap();

        let mut cache = BuildCache::new();
        cache.store(file.clone(), vec![]).unwrap();

        assert!(cache.is_valid(&file).unwrap());
    }

    #[test]
    fn test_is_valid_modified() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.vela");
        FileSystem::write(&file, "original").unwrap();

        let mut cache = BuildCache::new();
        cache.store(file.clone(), vec![]).unwrap();

        // Modify file
        std::thread::sleep(std::time::Duration::from_millis(10));
        FileSystem::write(&file, "modified").unwrap();

        assert!(!cache.is_valid(&file).unwrap());
    }

    #[test]
    fn test_is_valid_missing_file() {
        let cache = BuildCache::new();
        let nonexistent = PathBuf::from("nonexistent.vela");

        assert!(!cache.is_valid(&nonexistent).unwrap());
    }

    #[test]
    fn test_invalidate() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.vela");
        FileSystem::write(&file, "test").unwrap();

        let mut cache = BuildCache::new();
        cache.store(file.clone(), vec![]).unwrap();
        assert_eq!(cache.len(), 1);

        cache.invalidate(&file);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_clear() {
        let temp = TempDir::new().unwrap();
        let file1 = temp.path().join("test1.vela");
        let file2 = temp.path().join("test2.vela");
        FileSystem::write(&file1, "test").unwrap();
        FileSystem::write(&file2, "test").unwrap();

        let mut cache = BuildCache::new();
        cache.store(file1, vec![]).unwrap();
        cache.store(file2, vec![]).unwrap();
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_dependencies_invalidation() {
        let temp = TempDir::new().unwrap();
        let main = temp.path().join("main.vela");
        let dep = temp.path().join("dep.vela");
        FileSystem::write(&main, "import dep").unwrap();
        FileSystem::write(&dep, "lib").unwrap();

        let mut cache = BuildCache::new();
        cache.store(main.clone(), vec![dep.clone()]).unwrap();

        // Modify dependency
        std::thread::sleep(std::time::Duration::from_millis(10));
        FileSystem::write(&dep, "lib modified").unwrap();

        // Cache should be invalid due to dependency change
        assert!(!cache.is_valid(&main).unwrap());
    }
}
