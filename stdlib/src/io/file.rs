//! File operations API for Vela
//!
//! Provides synchronous file operations including reading, writing,
//! copying, moving, and deleting files. Inspired by Rust std::fs
//! and Node.js fs module.

use std::fs;
use std::path::Path;

/// File API for basic file operations
pub struct File;

impl File {
    /// Read the entire contents of a file as a string
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
        fs::read_to_string(path)
    }

    /// Read the entire contents of a file as bytes
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
        fs::read(path)
    }

    /// Write a string to a file, creating it if it doesn't exist
    pub fn write<P: AsRef<Path>, C: AsRef<str>>(path: P, contents: C) -> Result<()> {
        fs::write(path, contents.as_ref())
    }

    /// Write bytes to a file, creating it if it doesn't exist
    pub fn write_bytes<P: AsRef<Path>>(path: P, contents: &[u8]) -> Result<()> {
        fs::write(path, contents)
    }

    /// Append a string to a file, creating it if it doesn't exist
    pub fn append<P: AsRef<Path>, C: AsRef<str>>(path: P, contents: C) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        file.write_all(contents.as_ref().as_bytes())?;
        Ok(())
    }

    /// Append bytes to a file, creating it if it doesn't exist
    pub fn append_bytes<P: AsRef<Path>>(path: P, contents: &[u8]) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        file.write_all(contents)?;
        Ok(())
    }

    /// Copy a file from source to destination
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
        fs::copy(from, to)
    }

    /// Move/rename a file from source to destination
    pub fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
        fs::rename(from, to)
    }

    /// Delete a file
    pub fn delete<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::remove_file(path)
    }

    /// Check if a file exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Get file metadata
    pub fn metadata<P: AsRef<Path>>(path: P) -> Result<fs::Metadata> {
        fs::metadata(path)
    }

    /// Get file size in bytes
    pub fn size<P: AsRef<Path>>(path: P) -> Result<u64> {
        Ok(fs::metadata(path)?.len())
    }

    /// Check if path is a file
    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_file()
    }
}

/// Type alias for Result
pub type Result<T> = std::result::Result<T, std::io::Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn setup_test_file(name: &str, content: &str) -> String {
        let filename = format!("test_{}.txt", name);
        File::write(&filename, content).unwrap();
        filename
    }

    fn cleanup_test_file(filename: &str) {
        let _ = File::delete(filename);
    }

    #[test]
    fn test_write_and_read_string() {
        let filename = setup_test_file("write_read", "Hello, Vela!");

        let content = File::read_to_string(&filename).unwrap();
        assert_eq!(content, "Hello, Vela!");

        cleanup_test_file(&filename);
    }

    #[test]
    fn test_write_and_read_bytes() {
        let data = vec![1, 2, 3, 4, 5];
        let filename = "test_bytes.bin";
        File::write_bytes(filename, &data).unwrap();

        let read_data = File::read(filename).unwrap();
        assert_eq!(read_data, data);

        cleanup_test_file(filename);
    }

    #[test]
    fn test_append() {
        let filename = setup_test_file("append", "Hello");

        File::append(&filename, ", World!").unwrap();
        let content = File::read_to_string(&filename).unwrap();
        assert_eq!(content, "Hello, World!");

        cleanup_test_file(&filename);
    }

    #[test]
    fn test_copy() {
        let source = setup_test_file("source", "Content to copy");
        let dest = "test_dest.txt";

        let bytes_copied = File::copy(&source, dest).unwrap();
        // fs::copy returns the number of bytes copied, which should match the file size
        let expected_size = File::size(&source).unwrap();
        assert_eq!(bytes_copied, expected_size);

        let dest_content = File::read_to_string(dest).unwrap();
        assert_eq!(dest_content, "Content to copy");

        cleanup_test_file(&source);
        cleanup_test_file(dest);
    }

    #[test]
    fn test_move_file() {
        let source = setup_test_file("move_source", "Content to move");
        let dest = "test_moved.txt";

        File::move_file(&source, dest).unwrap();

        assert!(!File::exists(&source));
        assert!(File::exists(dest));

        let content = File::read_to_string(dest).unwrap();
        assert_eq!(content, "Content to move");

        cleanup_test_file(dest);
    }

    #[test]
    fn test_delete() {
        let filename = setup_test_file("delete", "To be deleted");

        assert!(File::exists(&filename));
        File::delete(&filename).unwrap();
        assert!(!File::exists(&filename));
    }

    #[test]
    fn test_exists() {
        let filename = setup_test_file("exists", "Test content");

        assert!(File::exists(&filename));
        assert!(!File::exists("nonexistent_file.txt"));

        cleanup_test_file(&filename);
    }

    #[test]
    fn test_metadata_and_size() {
        let content = "This is a test file with known size";
        let filename = setup_test_file("metadata", content);

        let metadata = File::metadata(&filename).unwrap();
        assert!(metadata.is_file());

        let size = File::size(&filename).unwrap();
        assert_eq!(size, content.len() as u64);

        cleanup_test_file(&filename);
    }

    #[test]
    fn test_is_file() {
        let filename = setup_test_file("is_file", "content");

        assert!(File::is_file(&filename));
        assert!(!File::is_file("nonexistent"));
        assert!(!File::is_file(".")); // directory

        cleanup_test_file(&filename);
    }
}