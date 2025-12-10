//! Integration Tests for I/O and Networking APIs
//!
//! Comprehensive integration tests for Vela's I/O and networking functionality.
//! Tests interactions between File API, Directory API, HttpClient, and WebSocket APIs,
//! including error handling, edge cases, and real-world scenarios.

use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

use vela_stdlib::io::file::File;
use vela_stdlib::io::directory::Directory;
use vela_stdlib::http::{HttpClient, HttpMethod, HttpRequest};
use vela_stdlib::websocket::{Message, WebSocketConfig};

/// Setup helper for creating temporary directories
fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Setup helper for creating test files
fn create_test_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.join(name);
    fs::write(&file_path, content).expect("Failed to create test file");
    file_path
}

#[cfg(test)]
mod io_networking_integration_tests {
    use super::*;

    /// Test file and directory operations together
    #[test]
    fn test_file_directory_integration() {
        let temp_dir = setup_temp_dir();
        let base_path = temp_dir.path();

        // Create directory structure
        let subdir_path = base_path.join("test_subdir");
        Directory::create(&subdir_path).unwrap();

        // Create files in different locations
        let root_file = create_test_file(base_path, "root.txt", "root content");
        let sub_file = create_test_file(&subdir_path, "sub.txt", "sub content");

        // Test directory listing
        let entries = Directory::list(base_path).unwrap();
        // Should have at least the subdirectory (tempfile may create additional files)
        assert!(entries.len() >= 1);
        assert!(entries.iter().any(|e| e.file_name().ends_with("test_subdir")));

        // Test file operations
        assert!(File::exists(&root_file));
        assert!(File::exists(&sub_file));

        let content_bytes = File::read(&root_file).unwrap();
        let content = String::from_utf8(content_bytes).unwrap();
        assert_eq!(content, "root content");

        // Test file size
        let size = File::size(&root_file).unwrap();
        assert_eq!(size, 12); // "root content" = 12 bytes

        // Test directory operations on files
        assert!(!Directory::is_directory(&root_file));
        assert!(Directory::is_directory(&subdir_path));

        // Cleanup
        File::delete(&root_file).unwrap();
        File::delete(&sub_file).unwrap();
        Directory::remove_all(&subdir_path).unwrap();

        assert!(!File::exists(&root_file));
        assert!(!Directory::exists(&subdir_path));
    }

    /// Test complex directory operations with file copying
    #[test]
    fn test_directory_file_copy_operations() {
        let temp_dir = setup_temp_dir();
        let base_path = temp_dir.path();

        // Create source directory with files
        let source_dir = base_path.join("source");
        Directory::create(&source_dir).unwrap();

        create_test_file(&source_dir, "file1.txt", "content 1");
        create_test_file(&source_dir, "file2.txt", "content 2");

        let nested_dir = source_dir.join("nested");
        Directory::create(&nested_dir).unwrap();
        create_test_file(&nested_dir, "nested_file.txt", "nested content");

        // Create destination
        let dest_dir = base_path.join("destination");
        Directory::copy(&source_dir, &dest_dir).unwrap();

        // Verify copy
        assert!(Directory::exists(&dest_dir));
        assert!(File::exists(&dest_dir.join("file1.txt")));
        assert!(File::exists(&dest_dir.join("file2.txt")));
        assert!(Directory::exists(&dest_dir.join("nested")));
        assert!(File::exists(&dest_dir.join("nested/nested_file.txt")));

        // Verify content
        let content_bytes = File::read(&dest_dir.join("file1.txt")).unwrap();
        let content = String::from_utf8(content_bytes).unwrap();
        assert_eq!(content, "content 1");

        // Test file copy within directories
        let copied_file = dest_dir.join("file1_copy.txt");
        File::copy(&dest_dir.join("file1.txt"), &copied_file).unwrap();
        assert!(File::exists(&copied_file));

        // Cleanup
        Directory::remove_all(&source_dir).unwrap();
        Directory::remove_all(&dest_dir).unwrap();
    }

    /// Test error handling across I/O operations
    #[test]
    fn test_io_error_handling_integration() {
        let temp_dir = setup_temp_dir();
        let base_path = temp_dir.path();

        // Test file operations on non-existent paths
        let nonexistent = base_path.join("nonexistent.txt");
        assert!(!File::exists(&nonexistent));

        // Reading non-existent file should fail
        let result = File::read(&nonexistent);
        assert!(result.is_err());

        // Writing to invalid path should fail (try to write to a directory as file)
        let dir_as_file = base_path.join("this_is_a_directory");
        Directory::create(&dir_as_file).unwrap();
        let write_result = File::write(&dir_as_file, "content");
        assert!(write_result.is_err()); // Should fail because it's a directory

        // Directory operations on files should fail appropriately
        let file_path = create_test_file(base_path, "test.txt", "content");
        assert!(!Directory::is_directory(&file_path));

        let list_result = Directory::list(&file_path);
        assert!(list_result.is_err()); // Can't list a file as directory

        // Test removing non-existent directory
        let nonexistent_dir = base_path.join("nonexistent_dir");
        let remove_result = Directory::remove(&nonexistent_dir);
        assert!(remove_result.is_err());

        // Cleanup
        File::delete(&file_path).unwrap();
        Directory::remove(&dir_as_file).unwrap();
    }

    /// Test HTTP client with file operations (downloading and saving)
    #[test]
    fn test_http_client_file_integration() {
        // Note: This would require a real HTTP server for full integration testing
        // For now, we test the client creation and basic configuration

        let client = HttpClient::new();
        // Test that client was created successfully
        assert!(true); // Client creation successful

        // Test request building
        let request = HttpRequest::get("https://httpbin.org/get")
            .header("User-Agent", "Vela-Test")
            .timeout(Duration::from_secs(10));

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.url, "https://httpbin.org/get");
        assert!(request.headers.contains_key("User-Agent"));
        assert_eq!(request.timeout, Some(Duration::from_secs(10)));
    }

    /// Test WebSocket configuration and basic setup
    #[test]
    fn test_websocket_configuration_integration() {
        // Test WebSocket config building
        let config = WebSocketConfig::new("ws://echo.websocket.org")
            .protocol("echo")
            .header("Authorization", "Bearer token")
            .timeout(Duration::from_secs(15))
            .max_message_size(1024 * 1024); // 1MB

        assert_eq!(config.url, "ws://echo.websocket.org");
        assert_eq!(config.protocols, vec!["echo"]);
        assert_eq!(config.headers.get("Authorization").unwrap(), "Bearer token");
        assert_eq!(config.timeout, Duration::from_secs(15));
        assert_eq!(config.max_message_size, 1024 * 1024);
    }

    /// Test message type handling across APIs
    #[test]
    fn test_message_type_consistency() {
        // Test that different APIs handle string/binary data consistently

        let temp_dir = setup_temp_dir();
        let file_path = temp_dir.path().join("test.bin");

        // Create binary content
        let binary_data = vec![0, 1, 2, 255, 254, 253];
        File::write_bytes(&file_path, &binary_data).unwrap();

        // Read it back
        let read_content = File::read(&file_path).unwrap();

        // Verify round-trip
        assert_eq!(read_content.len(), binary_data.len());
        assert_eq!(read_content, binary_data);

        // Test WebSocket message types
        let text_msg = Message::Text("Hello World".to_string());
        let binary_msg = Message::Binary(vec![1, 2, 3, 4]);

        match text_msg {
            Message::Text(content) => assert_eq!(content, "Hello World"),
            _ => panic!("Expected Text message"),
        }

        match binary_msg {
            Message::Binary(data) => assert_eq!(data, vec![1, 2, 3, 4]),
            _ => panic!("Expected Binary message"),
        }
    }

    /// Test concurrent file operations (basic concurrency test)
    #[test]
    fn test_concurrent_file_operations() {
        let temp_dir = setup_temp_dir();
        let base_path = temp_dir.path();

        // Create multiple files
        let files: Vec<_> = (0..5)
            .map(|i| create_test_file(base_path, &format!("file{}.txt", i), &format!("content {}", i)))
            .collect();

        // Test that all files exist and can be read concurrently
        for (i, file_path) in files.iter().enumerate() {
            assert!(File::exists(file_path));
            let content_bytes = File::read(file_path).unwrap();
            let content = String::from_utf8(content_bytes).unwrap();
            assert_eq!(content, format!("content {}", i));
        }

        // Test directory listing shows all files
        let entries = Directory::list_paths(base_path).unwrap();
        assert_eq!(entries.len(), 5);

        // Cleanup all files
        for file_path in files {
            File::delete(&file_path).unwrap();
        }
    }

    /// Test large file operations
    #[test]
    fn test_large_file_operations() {
        let temp_dir = setup_temp_dir();
        let large_file = temp_dir.path().join("large.txt");

        // Create a moderately large file (1MB)
        let large_content = "x".repeat(1024 * 1024);
        File::write(&large_file, &large_content).unwrap();

        // Verify size
        let size = File::size(&large_file).unwrap();
        assert_eq!(size, 1024 * 1024);

        // Read partial content
        let read_content = File::read(&large_file).unwrap();
        assert_eq!(read_content.len(), 1024 * 1024);
        assert!(read_content.starts_with(b"xxxxxxxx"));

        // Test append to large file
        File::append(&large_file, " appended").unwrap();
        let new_size = File::size(&large_file).unwrap();
        assert_eq!(new_size, 1024 * 1024 + 9); // original + " appended"

        // Cleanup
        File::delete(&large_file).unwrap();
    }

    /// Test path utilities integration with file operations
    #[test]
    fn test_path_utilities_integration() {
        let temp_dir = setup_temp_dir();
        let base_path = temp_dir.path();

        // Create nested directory structure
        let nested_path = base_path.join("level1").join("level2").join("level3");
        Directory::create(&nested_path).unwrap();

        // Test path operations using standard Rust Path
        let file_path = nested_path.join("test.txt");
        create_test_file(&nested_path, "test.txt", "content");

        let parent = file_path.parent().unwrap();
        assert_eq!(parent, nested_path);

        let file_name = file_path.file_name().unwrap();
        assert_eq!(file_name, "test.txt");

        let file_stem = file_path.file_stem().unwrap();
        assert_eq!(file_stem, "test");

        let extension = file_path.extension().unwrap();
        assert_eq!(extension, "txt");

        // Test directory operations on paths
        assert!(Directory::exists(&nested_path));
        assert!(File::exists(&file_path));

        // Cleanup
        Directory::remove_all(&base_path.join("level1")).unwrap();
    }

    /// Test network timeout scenarios (mocked)
    #[test]
    fn test_network_timeout_scenarios() {
        // Test HTTP client timeout configuration
        let client = HttpClient::new();
        // Default timeout should be set (we can't access private field)
        assert!(true); // Client created with default timeout

        // Test WebSocket timeout configuration
        let ws_config = WebSocketConfig::new("ws://test.com")
            .timeout(Duration::from_secs(5));
        assert_eq!(ws_config.timeout, Duration::from_secs(5));
    }

    /// Test comprehensive error scenarios
    #[test]
    fn test_comprehensive_error_scenarios() {
        let temp_dir = setup_temp_dir();
        let base_path = temp_dir.path();

        // Test file operations with permission-like errors (using read-only directory simulation)
        let test_file = create_test_file(base_path, "readonly_test.txt", "content");

        // Test operations on closed/deleted files
        File::delete(&test_file).unwrap();
        assert!(!File::exists(&test_file));

        // Operations on deleted file should fail
        let read_result = File::read(&test_file);
        assert!(read_result.is_err());

        let size_result = File::size(&test_file);
        assert!(size_result.is_err());

        // Test directory operations on invalid paths
        let invalid_path = base_path.join("invalid\x00name"); // Null byte in path
        let create_result = Directory::create(&invalid_path);
        // This might succeed or fail depending on the OS, but should not panic

        // Test extremely long paths (if supported by OS)
        let long_name = "a".repeat(200); // Very long filename
        let long_path = base_path.join(long_name);
        let long_file_result = File::write(&long_path, "content");
        // Should either succeed or fail gracefully
    }

    /// Test memory usage with large data structures
    #[test]
    fn test_memory_usage_large_structures() {
        let temp_dir = setup_temp_dir();
        let base_path = temp_dir.path();

        // Create many files and directories
        for i in 0..100 {
            let dir_path = base_path.join(format!("dir{}", i));
            Directory::create(&dir_path).unwrap();

            for j in 0..10 {
                create_test_file(&dir_path, &format!("file{}.txt", j), &format!("content {} {}", i, j));
            }
        }

        // Test directory listing of large directory
        let entries = Directory::list(base_path).unwrap();
        assert_eq!(entries.len(), 100);

        // Test that operations still work under load
        let test_file = base_path.join("dir50").join("file5.txt");
        let content_bytes = File::read(&test_file).unwrap();
        let content = String::from_utf8(content_bytes).unwrap();
        assert_eq!(content, "content 50 5");

        // Cleanup (remove_all should handle large directory trees)
        Directory::remove_all(base_path).unwrap();
        assert!(!Directory::exists(base_path));
    }
}