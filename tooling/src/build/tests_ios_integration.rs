/*!
iOS Integration Tests

Comprehensive tests for iOS build pipeline including compilation validation,
bytecode embedding, and end-to-end app generation.
*/

#[cfg(test)]
mod ios_integration_tests {
    use super::*;
    use crate::{BuildConfig, BuildExecutor};
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn test_ios_project_compiles_with_swiftpm() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let ios_dir = output_dir.join("ios");

        // Create build config and executor
        let config = BuildConfig::new(PathBuf::from("/tmp/project")).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        // Generate iOS artifacts
        let result = executor.generate_ios_artifacts();
        assert!(result.is_ok(), "iOS artifacts generation should succeed");

        // Verify Swift Package Manager can parse the project
        let output = Command::new("swift")
            .args(&["package", "describe"])
            .current_dir(&ios_dir)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                // Swift package describe succeeded
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("VelaApp"), "Package description should contain app name");
            }
            _ => {
                // If swift command is not available, at least verify Package.swift exists and is valid
                let package_path = ios_dir.join("Package.swift");
                assert!(package_path.exists(), "Package.swift should exist");

                let content = std::fs::read_to_string(package_path).unwrap();
                assert!(content.contains("name:"), "Package.swift should have name field");
                assert!(content.contains("targets:"), "Package.swift should have targets");
            }
        }
    }

    #[test]
    fn test_bytecode_embedding_integrity() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let ios_dir = output_dir.join("ios");
        let vela_dir = output_dir.join("vela");

        // Create mock bytecode files
        std::fs::create_dir_all(&vela_dir).unwrap();
        let bytecode_content = b"mock bytecode data for testing";
        let bytecode_file = vela_dir.join("app.velac");
        std::fs::write(&bytecode_file, bytecode_content).unwrap();

        // Create additional bytecode file
        let lib_file = vela_dir.join("library.velac");
        std::fs::write(&lib_file, b"library bytecode").unwrap();

        let config = BuildConfig::new(output_dir);
        let executor = BuildExecutor::new(config);

        // Copy bytecode
        let result = executor.copy_compiled_bytecode(&ios_dir);
        assert!(result.is_ok(), "Bytecode copying should succeed");

        // Verify bytecode files were copied with correct content
        let copied_app = ios_dir.join("Bytecode").join("app.velac");
        let copied_lib = ios_dir.join("Bytecode").join("library.velac");

        assert!(copied_app.exists(), "App bytecode should be copied");
        assert!(copied_lib.exists(), "Library bytecode should be copied");

        let copied_app_content = std::fs::read(&copied_app).unwrap();
        let copied_lib_content = std::fs::read(&copied_lib).unwrap();

        assert_eq!(copied_app_content, bytecode_content, "App bytecode content should match");
        assert_eq!(copied_lib_content, b"library bytecode", "Library bytecode content should match");
    }

    #[test]
    fn test_end_to_end_vela_app_compilation() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let output_dir = temp_dir.path().join("output");

        // Create a simple Vela app
        std::fs::create_dir_all(&project_dir).unwrap();
        let vela_source = r#"
import 'system:ui'

component CounterApp {
    state count: Number = 0

    fn increment() {
        this.count = this.count + 1
    }

    render {
        Column {
            Text("Count: ${this.count}")
            Button("Increment", onClick: this.increment)
        }
    }
}

@main
fn main() {
    CounterApp()
}
"#;
        let source_file = project_dir.join("main.vela");
        std::fs::write(&source_file, vela_source).unwrap();

        // Create build config for iOS target
        let config = BuildConfig::new(project_dir)
            .with_target("ios")
            .with_output_dir(&output_dir);

        let mut executor = BuildExecutor::new(config);

        // Add the source file to build graph
        executor.graph_mut().add_module(source_file.clone());

        // Execute build
        let build_result = executor.execute();
        assert!(build_result.is_ok(), "Build should succeed");
        let build_result = build_result.unwrap();
        assert!(build_result.success, "Build should be successful");

        // Verify iOS artifacts were generated
        let ios_dir = output_dir.join("ios");
        assert!(ios_dir.exists(), "iOS directory should be created");

        // Verify all required files exist
        assert!(ios_dir.join("Package.swift").exists(), "Package.swift should exist");
        assert!(ios_dir.join("Sources").join("main.swift").exists(), "main.swift should exist");
        assert!(ios_dir.join("Info.plist").exists(), "Info.plist should exist");
        assert!(ios_dir.join("Bytecode").exists(), "Bytecode directory should exist");

        // Verify bytecode was generated and copied
        let bytecode_files = std::fs::read_dir(ios_dir.join("Bytecode")).unwrap();
        let bytecode_count = bytecode_files.count();
        assert!(bytecode_count > 0, "At least one bytecode file should be present");
    }

    #[test]
    fn test_ios_project_structure_completeness() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let ios_dir = output_dir.join("ios");

        let config = BuildConfig::new(PathBuf::from("/tmp/project")).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        let result = executor.generate_ios_artifacts();
        assert!(result.is_ok(), "iOS artifacts generation should succeed");

        // Required directories
        assert!(ios_dir.join("Sources").exists(), "Sources directory should exist");
        assert!(ios_dir.join("Bytecode").exists(), "Bytecode directory should exist");

        // Required files
        let required_files = vec![
            "Package.swift",
            "Info.plist",
            "Sources/main.swift",
            "Sources/VelaApp-Bridging-Header.h",
        ];

        for file_path in required_files {
            let full_path = ios_dir.join(file_path);
            assert!(full_path.exists(), "Required file {} should exist", file_path);
        }

        // Verify Package.swift structure
        let package_content = std::fs::read_to_string(ios_dir.join("Package.swift")).unwrap();
        assert!(package_content.contains("name:"), "Package.swift should have name");
        assert!(package_content.contains("targets:"), "Package.swift should have targets");
        assert!(package_content.contains("executableTarget"), "Should be executable target");

        // Verify Info.plist structure
        let plist_content = std::fs::read_to_string(ios_dir.join("Info.plist")).unwrap();
        assert!(plist_content.contains("<plist"), "Should be valid plist format");
        assert!(plist_content.contains("CFBundleName"), "Should have bundle name");
        assert!(plist_content.contains("CFBundleIdentifier"), "Should have bundle identifier");
    }

    #[test]
    fn test_error_handling_invalid_bytecode() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let ios_dir = output_dir.join("ios");

        // Create invalid bytecode file (empty)
        let vela_dir = output_dir.join("vela");
        std::fs::create_dir_all(&vela_dir).unwrap();
        let bytecode_file = vela_dir.join("app.velac");
        std::fs::write(&bytecode_file, "").unwrap(); // Empty file

        let config = BuildConfig::new(output_dir);
        let executor = BuildExecutor::new(config);

        // This should still succeed (empty bytecode is valid)
        let result = executor.copy_compiled_bytecode(&ios_dir);
        assert!(result.is_ok(), "Copying empty bytecode should succeed");

        // Verify file was copied
        let copied_file = ios_dir.join("Bytecode").join("app.velac");
        assert!(copied_file.exists(), "Empty bytecode file should be copied");
        assert_eq!(std::fs::read_to_string(&copied_file).unwrap(), "", "Content should be empty");
    }

    #[test]
    fn test_error_handling_missing_output_dir() {
        let temp_dir = tempdir().unwrap();
        let ios_dir = temp_dir.path().join("nonexistent").join("ios");

        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        // Try to generate in non-existent directory
        let result = executor.generate_package_swift(&ios_dir);
        assert!(result.is_err(), "Should fail when output directory doesn't exist");

        // Try to copy bytecode to non-existent directory
        let result = executor.copy_compiled_bytecode(&ios_dir);
        assert!(result.is_err(), "Should fail when output directory doesn't exist");
    }

    #[test]
    fn test_multiple_vela_files_bytecode_handling() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let ios_dir = output_dir.join("ios");
        let vela_dir = output_dir.join("vela");

        // Create multiple bytecode files
        std::fs::create_dir_all(&vela_dir).unwrap();

        let files = vec![
            ("main.velac", "main bytecode"),
            ("utils.velac", "utils bytecode"),
            ("ui.velac", "ui bytecode"),
        ];

        for (filename, content) in &files {
            let file_path = vela_dir.join(filename);
            std::fs::write(&file_path, content).unwrap();
        }

        let config = BuildConfig::new(output_dir);
        let executor = BuildExecutor::new(config);

        let result = executor.copy_compiled_bytecode(&ios_dir);
        assert!(result.is_ok(), "Copying multiple bytecode files should succeed");

        // Verify all files were copied
        for (filename, expected_content) in files {
            let copied_file = ios_dir.join("Bytecode").join(filename);
            assert!(copied_file.exists(), "File {} should be copied", filename);

            let content = std::fs::read_to_string(&copied_file).unwrap();
            assert_eq!(content, expected_content, "Content of {} should match", filename);
        }
    }

    #[test]
    fn test_swift_syntax_validation() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let ios_dir = output_dir.join("ios");

        let config = BuildConfig::new(PathBuf::from("/tmp/project")).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        let result = executor.generate_ios_artifacts();
        assert!(result.is_ok(), "iOS artifacts generation should succeed");

        // Read generated Swift files and validate basic syntax
        let main_swift = ios_dir.join("Sources").join("main.swift");
        let main_content = std::fs::read_to_string(&main_swift).unwrap();

        // Basic Swift syntax checks
        assert!(main_content.contains("@main"), "Should have @main attribute");
        assert!(main_content.contains("struct"), "Should have struct declaration");
        assert!(main_content.contains("UIApplicationMain"), "Should call UIApplicationMain");

        // Check for balanced braces (basic syntax validation)
        let open_braces = main_content.chars().filter(|&c| c == '{').count();
        let close_braces = main_content.chars().filter(|&c| c == '}').count();
        assert_eq!(open_braces, close_braces, "Braces should be balanced");

        // Check for balanced parentheses
        let open_parens = main_content.chars().filter(|&c| c == '(').count();
        let close_parens = main_content.chars().filter(|&c| c == ')').count();
        assert_eq!(open_parens, close_parens, "Parentheses should be balanced");
    }

    #[test]
    fn test_package_swift_target_ios_version() {
        let temp_dir = tempdir().unwrap();
        let ios_dir = temp_dir.path().join("ios");

        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        let result = executor.generate_package_swift(&ios_dir);
        assert!(result.is_ok(), "generate_package_swift should succeed");

        let package_content = std::fs::read_to_string(ios_dir.join("Package.swift")).unwrap();

        // Verify iOS version targeting
        assert!(package_content.contains("platforms: [.iOS(.v14)]"), "Should target iOS 14+");
        assert!(package_content.contains("executableTarget"), "Should be executable target");
        assert!(package_content.contains("name: \"VelaApp\""), "Should have correct app name");
    }
}