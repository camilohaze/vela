/*!
iOS Build Pipeline Integration Tests

End-to-end tests for the complete iOS build pipeline from Vela source
to compiled iOS application.
*/

#[cfg(test)]
mod ios_build_pipeline_tests {
    use vela_tooling::build::{BuildConfig, BuildExecutor};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_full_build_pipeline_ios() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let output_dir = temp_dir.path().join("output");

        // Create project structure
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create Vela source files
        let main_vela = r#"
import 'system:ui'

component HelloApp {
    render {
        Text("Hello from Vela on iOS!")
    }
}

@main
fn main() {
    HelloApp()
}
"#;

        let utils_vela = r#"
// Utility functions for iOS app
fn format_greeting(name: String) -> String {
    "Hello, ${name}!"
}

export { format_greeting }
"#;

        std::fs::write(project_dir.join("main.vela"), main_vela).unwrap();
        std::fs::write(project_dir.join("utils.vela"), utils_vela).unwrap();

        // Create build config for iOS
        let config = BuildConfig::new(project_dir.clone())
            .with_target("ios")
            .with_output_dir(&output_dir);

        let mut executor = BuildExecutor::new(config);

        // Add source files to build graph
        executor.graph_mut().add_module(project_dir.join("main.vela"));
        executor.graph_mut().add_module(project_dir.join("utils.vela"));

        // Execute full build pipeline
        let result = executor.execute();
        assert!(result.is_ok(), "Full build pipeline should succeed");
        let result = result.unwrap();
        assert!(result.success, "Build should be successful");

        // Verify iOS artifacts were generated
        let ios_dir = output_dir.join("ios");
        assert!(ios_dir.exists(), "iOS output directory should exist");

        // Verify complete project structure
        verify_ios_project_structure(&ios_dir);

        // Verify bytecode files
        let bytecode_dir = ios_dir.join("Bytecode");
        assert!(bytecode_dir.exists(), "Bytecode directory should exist");

        let bytecode_files: Vec<_> = std::fs::read_dir(&bytecode_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();

        assert!(!bytecode_files.is_empty(), "Should have bytecode files");

        // Verify each bytecode file is readable
        for bytecode_file in bytecode_files {
            let content = std::fs::read(&bytecode_file).unwrap();
            assert!(!content.is_empty(), "Bytecode file should not be empty: {:?}", bytecode_file);
        }
    }

    #[test]
    fn test_multiple_vela_files_ios_generation() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let output_dir = temp_dir.path().join("output");

        std::fs::create_dir_all(&project_dir).unwrap();

        // Create multiple Vela files with dependencies
        let files = vec![
            ("main.vela", r#"
import 'system:ui'
import './components.vela'
import './services.vela'

@main
component MainApp {
    render {
        AppContainer()
    }
}
"#),
            ("components.vela", r#"
component AppContainer {
    render {
        Column {
            Header()
            Content()
            Footer()
        }
    }
}

component Header {
    render {
        Text("Vela iOS App")
    }
}

component Content {
    render {
        Text("Main content area")
    }
}

component Footer {
    render {
        Text("Footer content")
    }
}
"#),
            ("services.vela", r#"
// Service functions
fn get_app_title() -> String {
    "Vela Multi-File App"
}

fn get_version() -> String {
    "1.0.0"
}

export { get_app_title, get_version }
"#),
        ];

        // Write all files
        for (filename, content) in files {
            std::fs::write(project_dir.join(filename), content).unwrap();
        }

        // Build for iOS
        let config = BuildConfig::new(project_dir.clone())
            .with_target("ios")
            .with_output_dir(&output_dir);

        let mut executor = BuildExecutor::new(config);

        // Add all source files
        for (filename, _) in &files {
            executor.graph_mut().add_module(project_dir.join(filename));
        }

        let result = executor.execute();
        assert!(result.is_ok(), "Multi-file build should succeed");
        let result = result.unwrap();
        assert!(result.success, "Build should be successful");

        // Verify iOS output
        let ios_dir = output_dir.join("ios");
        verify_ios_project_structure(&ios_dir);

        // Should have bytecode for all source files
        let bytecode_dir = ios_dir.join("Bytecode");
        let bytecode_files: Vec<_> = std::fs::read_dir(&bytecode_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();

        // Should have .velac files for each .vela file
        for (filename, _) in &files {
            let bytecode_name = filename.replace(".vela", ".velac");
            assert!(bytecode_files.contains(&bytecode_name),
                "Should have bytecode for {}", filename);
        }
    }

    #[test]
    fn test_ios_build_with_dependencies() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let output_dir = temp_dir.path().join("output");

        std::fs::create_dir_all(&project_dir).unwrap();

        // Create interdependent modules
        let data_vela = r#"
struct User {
    id: Number
    name: String
    email: String
}

fn create_user(id: Number, name: String, email: String) -> User {
    User { id, name, email }
}

export { User, create_user }
"#;

        let ui_vela = r#"
import './data.vela'
import 'system:ui'

component UserCard(user: User) {
    render {
        Column {
            Text("ID: ${user.id}")
            Text("Name: ${user.name}")
            Text("Email: ${user.email}")
        }
    }
}

export { UserCard }
"#;

        let main_vela = r#"
import './ui.vela'
import './data.vela'

@main
component UserApp {
    state users: List<User> = [
        create_user(1, "Alice", "alice@example.com"),
        create_user(2, "Bob", "bob@example.com")
    ]

    render {
        Column {
            Text("User Management App")
            for user in this.users {
                UserCard(user)
            }
        }
    }
}
"#;

        std::fs::write(project_dir.join("data.vela"), data_vela).unwrap();
        std::fs::write(project_dir.join("ui.vela"), ui_vela).unwrap();
        std::fs::write(project_dir.join("main.vela"), main_vela).unwrap();

        // Build with dependencies
        let config = BuildConfig::new(project_dir.clone())
            .with_target("ios")
            .with_output_dir(&output_dir);

        let mut executor = BuildExecutor::new(config);

        // Add modules with dependencies
        let data_id = executor.graph_mut().add_module(project_dir.join("data.vela"));
        let ui_id = executor.graph_mut().add_module(project_dir.join("ui.vela"));
        let main_id = executor.graph_mut().add_module(project_dir.join("main.vela"));

        // Set up dependencies
        executor.graph_mut().add_dependency(ui_id, data_id);
        executor.graph_mut().add_dependency(main_id, ui_id);
        executor.graph_mut().add_dependency(main_id, data_id);

        let result = executor.execute();
        assert!(result.is_ok(), "Build with dependencies should succeed");
        let result = result.unwrap();
        assert!(result.success, "Build should be successful");

        // Verify all bytecode files were generated
        let ios_dir = output_dir.join("ios");
        let bytecode_dir = ios_dir.join("Bytecode");

        let expected_files = vec!["data.velac", "ui.velac", "main.velac"];
        for filename in expected_files {
            assert!(bytecode_dir.join(filename).exists(),
                "Bytecode file {} should exist", filename);
        }
    }

    #[test]
    fn test_ios_build_error_handling() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let output_dir = temp_dir.path().join("output");

        std::fs::create_dir_all(&project_dir).unwrap();

        // Create Vela file with syntax error
        let invalid_vela = r#"
import 'system:ui'

component InvalidApp {
    // Missing closing brace and invalid syntax
    render {
        Text("Hello"
        // Missing closing brace for component
"#;

        std::fs::write(project_dir.join("invalid.vela"), invalid_vela).unwrap();

        let config = BuildConfig::new(project_dir.clone())
            .with_target("ios")
            .with_output_dir(&output_dir);

        let mut executor = BuildExecutor::new(config);
        executor.graph_mut().add_module(project_dir.join("invalid.vela"));

        // Build should fail due to syntax error
        let result = executor.execute();
        assert!(result.is_err(), "Build with syntax error should fail");

        // iOS directory should not be created if build fails
        let ios_dir = output_dir.join("ios");
        assert!(!ios_dir.exists(), "iOS directory should not be created when build fails");
    }

    #[test]
    fn test_ios_build_empty_project() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let output_dir = temp_dir.path().join("output");

        std::fs::create_dir_all(&project_dir).unwrap();

        let config = BuildConfig::new(project_dir)
            .with_target("ios")
            .with_output_dir(&output_dir);

        let executor = BuildExecutor::new(config);

        // Build empty project
        let result = executor.execute();
        assert!(result.is_ok(), "Empty project build should succeed");
        let result = result.unwrap();
        assert!(result.success, "Build should be successful");
        assert_eq!(result.modules_compiled, 0, "No modules should be compiled");

        // iOS directory should still be created (for template)
        let ios_dir = output_dir.join("ios");
        assert!(ios_dir.exists(), "iOS directory should be created even for empty project");

        // But bytecode directory should be empty or not exist
        let bytecode_dir = ios_dir.join("Bytecode");
        if bytecode_dir.exists() {
            let entries: Vec<_> = std::fs::read_dir(&bytecode_dir).unwrap().collect();
            assert!(entries.is_empty(), "Bytecode directory should be empty for empty project");
        }
    }

    #[test]
    fn test_ios_build_incremental_compilation() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let output_dir = temp_dir.path().join("output");

        std::fs::create_dir_all(&project_dir).unwrap();

        // Create initial Vela file
        let initial_code = r#"
import 'system:ui'

component TestApp {
    render {
        Text("Initial version")
    }
}

@main
fn main() {
    TestApp()
}
"#;

        std::fs::write(project_dir.join("app.vela"), initial_code).unwrap();

        let config = BuildConfig::new(project_dir.clone())
            .with_target("ios")
            .with_output_dir(&output_dir);

        let mut executor = BuildExecutor::new(config.clone());
        executor.graph_mut().add_module(project_dir.join("app.vela"));

        // First build
        let result1 = executor.execute().unwrap();
        assert!(result1.success);

        // Modify the file
        let updated_code = r#"
import 'system:ui'

component TestApp {
    render {
        Text("Updated version")
    }
}

@main
fn main() {
    TestApp()
}
"#;

        std::fs::write(project_dir.join("app.vela"), updated_code).unwrap();

        // Second build (should detect changes)
        let mut executor2 = BuildExecutor::new(config);
        executor2.graph_mut().add_module(project_dir.join("app.vela"));

        let result2 = executor2.execute().unwrap();
        assert!(result2.success);

        // Verify iOS artifacts exist and are up to date
        let ios_dir = output_dir.join("ios");
        assert!(ios_dir.exists());

        let bytecode_file = ios_dir.join("Bytecode").join("app.velac");
        assert!(bytecode_file.exists(), "Updated bytecode should exist");
    }

    // Helper function to verify iOS project structure
    fn verify_ios_project_structure(ios_dir: &PathBuf) {
        assert!(ios_dir.exists(), "iOS directory should exist");

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
            assert!(ios_dir.join(file_path).exists(),
                "Required file {} should exist", file_path);
        }
    }
}