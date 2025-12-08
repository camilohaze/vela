/*!
# Tests for vela test command

Tests for the CLI test command functionality.
*/

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use assert_cmd::Command;
use predicates::prelude::*;

/// Test helper to create a temporary project structure
fn create_test_project() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create tests directory
    fs::create_dir(temp_dir.path().join("tests")).unwrap();

    // Create a simple test file
    let test_content = r#"
// Simple test file
fn add(a: Number, b: Number) -> Number {
    return a + b;
}

fn main() -> void {
    let result = add(2, 3);
    print("Result: ${result}");
}
"#;

    fs::write(temp_dir.path().join("tests").join("test_math.vela"), test_content).unwrap();

    // Create another test file
    let test_content2 = r#"
// Another test file
fn multiply(x: Number, y: Number) -> Number {
    return x * y;
}

fn main() -> void {
    let result = multiply(4, 5);
    print("Result: ${result}");
}
"#;

    fs::write(temp_dir.path().join("tests").join("test_multiply.vela"), test_content2).unwrap();

    temp_dir
}

#[test]
fn test_test_command_no_tests_directory() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No tests/ directory found"));
}

#[test]
fn test_test_command_empty_tests_directory() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join("tests")).unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No test files found"));
}

#[test]
fn test_test_command_runs_tests() {
    let temp_dir = create_test_project();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running Vela tests"))
        .stdout(predicate::str::contains("Found 2 test files"))
        .stdout(predicate::str::contains("✅"))
        .stdout(predicate::str::contains("All tests passed"));
}

#[test]
fn test_test_command_verbose_output() {
    let temp_dir = create_test_project();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .arg("--verbose")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running Vela tests"))
        .stdout(predicate::str::contains("Found 2 test files"))
        .stdout(predicate::str::contains("tests/test_math.vela"))
        .stdout(predicate::str::contains("tests/test_multiply.vela"))
        .stdout(predicate::str::contains("Running tests in"))
        .stdout(predicate::str::contains("✅"))
        .stdout(predicate::str::contains("All tests passed"));
}

#[test]
fn test_test_command_with_filter() {
    let temp_dir = create_test_project();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .arg("--filter")
        .arg("math")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running Vela tests"))
        .stdout(predicate::str::contains("Found 2 test files"))
        .stdout(predicate::str::contains("test_math.vela"))
        .stdout(predicate::str::contains("✅"))
        .stdout(predicate::str::contains("Tests run: 1"));
}

#[test]
fn test_test_command_with_time() {
    let temp_dir = create_test_project();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .arg("--time")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running Vela tests"))
        .stdout(predicate::str::contains("Total time:"))
        .stdout(predicate::str::contains("Average time:"))
        .stdout(predicate::str::contains("All tests passed"));
}

#[test]
fn test_test_command_specific_files() {
    let temp_dir = create_test_project();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .arg("tests/test_math.vela")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running Vela tests"))
        .stdout(predicate::str::contains("✅ tests/test_math.vela"))
        .stdout(predicate::str::contains("Tests run: 1"));
}

#[test]
fn test_test_command_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join("tests")).unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .arg("tests/nonexistent.vela")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to compile"));
}

#[test]
fn test_test_command_invalid_extension() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join("tests")).unwrap();

    // Create a file with wrong extension
    fs::write(temp_dir.path().join("tests").join("test.txt"), "not vela code").unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No test files found"));
}

#[test]
fn test_test_command_compilation_error() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join("tests")).unwrap();

    // Create a file with syntax error
    let invalid_content = r#"
// Invalid syntax
fn broken() -> Number {
    return "string";  // Type mismatch
}
"#;

    fs::write(temp_dir.path().join("tests").join("test_broken.vela"), invalid_content).unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .current_dir(&temp_dir)
        .assert()
        .success()  // Command succeeds but reports test failure
        .stdout(predicate::str::contains("❌ Failed to compile"))
        .stdout(predicate::str::contains("Tests run: 0"))
        .stdout(predicate::str::contains("Failed: 1"));
}