//! Directory operations API for Vela
//!
//! Provides synchronous directory operations including creating, listing,
//! removing, and path manipulation. Inspired by Rust std::fs and
//! Node.js fs module.

use std::fs;
use std::path::{Path, PathBuf};

/// Directory API for basic directory operations
pub struct Directory;

impl Directory {
    /// Create a directory and all its parent directories if they don't exist
    pub fn create<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::create_dir_all(path)
    }

    /// Create a single directory (fails if parent doesn't exist)
    pub fn create_single<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::create_dir(path)
    }

    /// Remove a directory (must be empty)
    pub fn remove<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::remove_dir(path)
    }

    /// Remove a directory and all its contents recursively
    pub fn remove_all<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::remove_dir_all(path)
    }

    /// List all entries in a directory
    pub fn list<P: AsRef<Path>>(path: P) -> Result<Vec<DirEntry>> {
        let entries = fs::read_dir(path)?;
        let mut result = Vec::new();

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            result.push(DirEntry {
                path: entry.path(),
                file_type: metadata.file_type(),
                metadata,
            });
        }

        Ok(result)
    }

    /// List all entries in a directory as string paths
    pub fn list_paths<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
        let entries = fs::read_dir(path)?;
        let mut result = Vec::new();

        for entry in entries {
            let entry = entry?;
            result.push(entry.path().to_string_lossy().to_string());
        }

        Ok(result)
    }

    /// Check if a directory exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_dir()
    }

    /// Check if path is a directory
    pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_dir()
    }

    /// Get directory metadata
    pub fn metadata<P: AsRef<Path>>(path: P) -> Result<fs::Metadata> {
        fs::metadata(path)
    }

    /// Copy a directory and all its contents recursively
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
        fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
            // Create destination directory
            fs::create_dir_all(dst)?;

            // Copy all entries
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let entry_path = entry.path();
                let dest_path = dst.join(entry.file_name());

                if entry_path.is_dir() {
                    copy_dir_recursive(&entry_path, &dest_path)?;
                } else {
                    fs::copy(&entry_path, &dest_path)?;
                }
            }

            Ok(())
        }

        copy_dir_recursive(from.as_ref(), to.as_ref())
    }

    /// Move/rename a directory
    pub fn move_dir<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
        fs::rename(from, to)
    }
}

/// Path utilities for cross-platform path manipulation
pub struct PathUtil;

impl PathUtil {
    /// Join path segments
    pub fn join<P: AsRef<Path>>(base: P, segments: &[&str]) -> PathBuf {
        let mut path = base.as_ref().to_path_buf();
        for segment in segments {
            path.push(segment);
        }
        path
    }

    /// Get the parent directory of a path
    pub fn parent<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        path.as_ref().parent().map(|p| p.to_path_buf())
    }

    /// Get the file name of a path
    pub fn file_name<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
    }

    /// Get the file stem (name without extension)
    pub fn file_stem<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
    }

    /// Get the file extension
    pub fn extension<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .extension()
            .map(|n| n.to_string_lossy().to_string())
    }

    /// Check if path is absolute
    pub fn is_absolute<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_absolute()
    }

    /// Check if path is relative
    pub fn is_relative<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_relative()
    }

    /// Canonicalize a path (resolve .. and .)
    pub fn canonicalize<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
        fs::canonicalize(path).map_err(Into::into)
    }

    /// Get current working directory
    pub fn current_dir() -> Result<PathBuf> {
        std::env::current_dir().map_err(Into::into)
    }

    /// Set current working directory
    pub fn set_current_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        std::env::set_current_dir(path).map_err(Into::into)
    }
}

/// Directory entry with metadata
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Full path to the entry
    pub path: PathBuf,
    /// File type (file, directory, symlink)
    pub file_type: fs::FileType,
    /// Metadata for the entry
    pub metadata: fs::Metadata,
}

impl DirEntry {
    /// Get the file name of this entry
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    /// Check if this entry is a file
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    /// Check if this entry is a directory
    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    /// Get the size of this entry (0 for directories)
    pub fn size(&self) -> u64 {
        self.metadata.len()
    }
}

/// Type alias for Result
pub type Result<T> = std::result::Result<T, std::io::Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn setup_test_dir(name: &str) -> String {
        let dirname = format!("test_dir_{}", name);
        Directory::create(&dirname).unwrap();
        // Create some test files
        File::write(format!("{}/file1.txt", dirname), "content1").unwrap();
        File::write(format!("{}/file2.txt", dirname), "content2").unwrap();
        Directory::create(format!("{}/subdir", dirname)).unwrap();
        File::write(format!("{}/subdir/file3.txt", dirname), "content3").unwrap();
        dirname
    }

    fn cleanup_test_dir(dirname: &str) {
        let _ = Directory::remove_all(dirname);
    }

    // Helper to use File API in tests
    struct File;
    impl File {
        fn write<P: AsRef<Path>, C: AsRef<str>>(path: P, contents: C) -> Result<()> {
            fs::write(path, contents.as_ref())
        }
    }

    #[test]
    fn test_create_directory() {
        let dirname = "test_create_dir";
        assert!(!Directory::exists(dirname));

        Directory::create(dirname).unwrap();
        assert!(Directory::exists(dirname));

        cleanup_test_dir(dirname);
    }

    #[test]
    fn test_create_nested_directories() {
        let dirname = "test/nested/deep/dir";
        assert!(!Directory::exists(dirname));

        Directory::create(dirname).unwrap();
        assert!(Directory::exists(dirname));

        cleanup_test_dir("test");
    }

    #[test]
    fn test_remove_directory() {
        let dirname = "test_remove_empty";
        Directory::create(&dirname).unwrap();

        assert!(Directory::exists(&dirname));
        Directory::remove(&dirname).unwrap();
        assert!(!Directory::exists(&dirname));
    }

    #[test]
    fn test_remove_all() {
        let dirname = setup_test_dir("remove_all");

        assert!(Directory::exists(&dirname));
        Directory::remove_all(&dirname).unwrap();
        assert!(!Directory::exists(&dirname));
    }

    #[test]
    fn test_list_directory() {
        let dirname = setup_test_dir("list");

        let entries = Directory::list(&dirname).unwrap();
        assert_eq!(entries.len(), 3); // 2 files + 1 subdir

        // Check that we have the expected entries
        let names: Vec<String> = entries.iter().map(|e| e.file_name()).collect();
        assert!(names.contains(&"file1.txt".to_string()));
        assert!(names.contains(&"file2.txt".to_string()));
        assert!(names.contains(&"subdir".to_string()));

        cleanup_test_dir(&dirname);
    }

    #[test]
    fn test_list_paths() {
        let dirname = setup_test_dir("list_paths");

        let paths = Directory::list_paths(&dirname).unwrap();
        assert_eq!(paths.len(), 3);

        // Check that paths contain the directory name
        for path in paths {
            assert!(path.contains(&dirname));
        }

        cleanup_test_dir(&dirname);
    }

    #[test]
    fn test_directory_exists() {
        let dirname = setup_test_dir("exists");

        assert!(Directory::exists(&dirname));
        assert!(!Directory::exists("nonexistent_dir"));

        cleanup_test_dir(&dirname);
    }

    #[test]
    fn test_is_directory() {
        let dirname = setup_test_dir("is_dir");

        assert!(Directory::is_directory(&dirname));
        assert!(!Directory::is_directory("nonexistent"));
        assert!(!Directory::is_directory("Cargo.toml")); // file, not dir

        cleanup_test_dir(&dirname);
    }

    #[test]
    fn test_copy_directory() {
        let source = setup_test_dir("copy_source");
        let dest = "test_copy_dest";

        assert!(!Directory::exists(dest));
        Directory::copy(&source, dest).unwrap();
        assert!(Directory::exists(dest));

        // Check that contents were copied
        let dest_entries = Directory::list(dest).unwrap();
        assert_eq!(dest_entries.len(), 3);

        cleanup_test_dir(&source);
        cleanup_test_dir(dest);
    }

    #[test]
    fn test_move_directory() {
        let source = setup_test_dir("move_source");
        let dest = "test_move_dest";

        assert!(Directory::exists(&source));
        assert!(!Directory::exists(dest));

        Directory::move_dir(&source, dest).unwrap();

        assert!(!Directory::exists(&source));
        assert!(Directory::exists(dest));

        cleanup_test_dir(dest);
    }

    #[test]
    fn test_path_join() {
        let path = PathUtil::join("base", &["dir", "file.txt"]);
        let expected = Path::new("base").join("dir").join("file.txt");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_path_parent() {
        let parent = PathUtil::parent("dir/subdir/file.txt").unwrap();
        assert_eq!(parent.to_string_lossy(), "dir/subdir");
    }

    #[test]
    fn test_path_file_name() {
        let name = PathUtil::file_name("dir/file.txt").unwrap();
        assert_eq!(name, "file.txt");
    }

    #[test]
    fn test_path_file_stem() {
        let stem = PathUtil::file_stem("dir/file.txt").unwrap();
        assert_eq!(stem, "file");
    }

    #[test]
    fn test_path_extension() {
        let ext = PathUtil::extension("dir/file.txt").unwrap();
        assert_eq!(ext, "txt");
    }

    #[test]
    fn test_path_is_absolute() {
        // Test absolute path (use Windows-style absolute path)
        assert!(PathUtil::is_absolute("C:\\absolute\\path"));
        // Test relative path
        assert!(!PathUtil::is_absolute("relative\\path"));
    }

    #[test]
    fn test_current_dir() {
        let current = PathUtil::current_dir().unwrap();
        assert!(current.is_absolute());
        assert!(Directory::exists(&current));
    }
}