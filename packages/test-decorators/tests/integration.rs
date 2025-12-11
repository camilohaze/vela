//! Integration tests for the test decorators framework

// Note: We can't use #[test] directly in this file because it conflicts with our custom test macro.
// Instead, we'll create a separate test file that uses the standard Rust test framework to verify our macros work.

#[cfg(test)]
mod macro_expansion_tests {
    // These tests verify that our macros expand correctly
    // We can't run them directly because of the macro name conflict

    #[test]
    fn dummy_test() {
        // This is just to ensure the module compiles
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_bdd_example_compiles() {
        // This test verifies that our BDD example compiles correctly
        // The actual test execution is handled by the macro expansion
        let result = std::process::Command::new("cargo")
            .args(&["check", "--example", "bdd_example"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output();

        match result {
            Ok(output) if output.status.success() => {
                // Compilation successful
                assert!(true);
            }
            Ok(output) => {
                // Compilation failed, print error for debugging
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("BDD example compilation failed: {}", stderr);
            }
            Err(e) => {
                panic!("Failed to run cargo check: {}", e);
            }
        }
    }

    #[test]
    fn test_macro_expansion_basic() {
        // Test that our macros can be parsed and expanded
        // This is a basic smoke test for the macro system

        // We can't directly test the macro expansion here due to the proc-macro nature,
        // but we can verify that the crate compiles and the macros are defined
        use std::fs;
        use std::path::Path;

        // Check that the lib.rs file exists and contains our macro definitions
        let lib_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
        assert!(lib_path.exists(), "lib.rs should exist");

        let content = fs::read_to_string(lib_path).expect("Should be able to read lib.rs");
        assert!(content.contains("#[proc_macro_attribute]"), "Should contain proc macro attributes");
        assert!(content.contains("pub fn test("), "Should contain test macro");
        assert!(content.contains("pub fn describe("), "Should contain describe macro");
        assert!(content.contains("pub fn it("), "Should contain it macro");
    }

    #[test]
    fn test_cargo_toml_configuration() {
        // Test that Cargo.toml is properly configured
        use std::fs;
        use std::path::Path;

        let cargo_toml_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        assert!(cargo_toml_path.exists(), "Cargo.toml should exist");

        let content = fs::read_to_string(cargo_toml_path).expect("Should be able to read Cargo.toml");
        assert!(content.contains("proc-macro = true"), "Should be configured as proc-macro");
        assert!(content.contains("syn = { version = \"2.0\""), "Should depend on syn");
        assert!(content.contains("quote = \"1.0\""), "Should depend on quote");
    }
}