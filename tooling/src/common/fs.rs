/*!
File system utilities
*/

use crate::common::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// File system utilities
pub struct FileSystem;

impl FileSystem {
    /// Read file to string
    pub fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }

    /// Write string to file
    pub fn write(path: impl AsRef<Path>, contents: impl AsRef<str>) -> Result<()> {
        Ok(fs::write(path, contents.as_ref())?)
    }

    /// Create directory recursively
    pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
        Ok(fs::create_dir_all(path)?)
    }

    /// Check if path exists
    pub fn exists(path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }

    /// Check if path is file
    pub fn is_file(path: impl AsRef<Path>) -> bool {
        path.as_ref().is_file()
    }

    /// Check if path is directory
    pub fn is_dir(path: impl AsRef<Path>) -> bool {
        path.as_ref().is_dir()
    }

    /// Compute SHA-256 hash of file
    pub fn hash_file(path: impl AsRef<Path>) -> Result<String> {
        let contents = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Find files matching pattern
    pub fn find_files(
        root: impl AsRef<Path>,
        extension: impl AsRef<str>,
    ) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let ext = extension.as_ref();

        for entry in WalkDir::new(root) {
            let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            if entry.file_type().is_file() {
                if let Some(file_ext) = entry.path().extension() {
                    if file_ext == ext {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    /// Get file modification time
    pub fn modified_time(path: impl AsRef<Path>) -> Result<std::time::SystemTime> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.modified()?)
    }

    /// Copy file
    pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
        fs::copy(from, to)?;
        Ok(())
    }

    /// Remove file
    pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
        Ok(fs::remove_file(path)?)
    }

    /// Remove directory recursively
    pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
        Ok(fs::remove_dir_all(path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_write() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");

        FileSystem::write(&file, "Hello, World!").unwrap();
        let contents = FileSystem::read_to_string(&file).unwrap();

        assert_eq!(contents, "Hello, World!");
    }

    #[test]
    fn test_exists() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");

        assert!(!FileSystem::exists(&file));
        FileSystem::write(&file, "test").unwrap();
        assert!(FileSystem::exists(&file));
    }

    #[test]
    fn test_is_file_is_dir() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        let dir = temp.path().join("subdir");

        FileSystem::write(&file, "test").unwrap();
        FileSystem::create_dir_all(&dir).unwrap();

        assert!(FileSystem::is_file(&file));
        assert!(!FileSystem::is_dir(&file));
        assert!(FileSystem::is_dir(&dir));
        assert!(!FileSystem::is_file(&dir));
    }

    #[test]
    fn test_hash_file() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");

        FileSystem::write(&file, "Hello").unwrap();
        let hash1 = FileSystem::hash_file(&file).unwrap();

        FileSystem::write(&file, "World").unwrap();
        let hash2 = FileSystem::hash_file(&file).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_find_files() {
        let temp = TempDir::new().unwrap();

        FileSystem::write(temp.path().join("file1.vela"), "").unwrap();
        FileSystem::write(temp.path().join("file2.vela"), "").unwrap();
        FileSystem::write(temp.path().join("file3.txt"), "").unwrap();

        let vela_files = FileSystem::find_files(temp.path(), "vela").unwrap();
        assert_eq!(vela_files.len(), 2);
    }

    #[test]
    fn test_modified_time() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");

        FileSystem::write(&file, "test").unwrap();
        let time = FileSystem::modified_time(&file);

        assert!(time.is_ok());
    }

    #[test]
    fn test_copy() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("dest.txt");

        FileSystem::write(&src, "content").unwrap();
        FileSystem::copy(&src, &dst).unwrap();

        let contents = FileSystem::read_to_string(&dst).unwrap();
        assert_eq!(contents, "content");
    }

    #[test]
    fn test_remove_file() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");

        FileSystem::write(&file, "test").unwrap();
        assert!(FileSystem::exists(&file));

        FileSystem::remove_file(&file).unwrap();
        assert!(!FileSystem::exists(&file));
    }

    #[test]
    fn test_remove_dir_all() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("subdir");

        FileSystem::create_dir_all(&dir).unwrap();
        FileSystem::write(dir.join("file.txt"), "test").unwrap();

        assert!(FileSystem::exists(&dir));
        FileSystem::remove_dir_all(&dir).unwrap();
        assert!(!FileSystem::exists(&dir));
    }
}
