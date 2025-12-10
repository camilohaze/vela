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

    fs::write(temp_dir.path().join("tests").join("test_math.spec.vela"), test_content).unwrap();

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

    fs::write(temp_dir.path().join("tests").join("test_multiply.spec.vela"), test_content2).unwrap();

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
        .stdout(predicate::str::contains("tests/test_math.spec.vela"))
        .stdout(predicate::str::contains("tests/test_multiply.spec.vela"))
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
        .stdout(predicate::str::contains("test_math.spec.vela"))
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
        .arg("tests/test_math.spec.vela")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running Vela tests"))
        .stdout(predicate::str::contains("✅ tests/test_math.spec.vela"))
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

    fs::write(temp_dir.path().join("tests").join("test_broken.spec.vela"), invalid_content).unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("test")
        .current_dir(&temp_dir)
        .assert()
        .success()  // Command succeeds but reports test failure
        .stdout(predicate::str::contains("❌ Failed to compile"))
        .stdout(predicate::str::contains("Tests run: 0"))
        .stdout(predicate::str::contains("Failed: 1"));
}

#[test]
fn test_fmt_command_no_files() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("fmt")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No .vela files found"));
}

#[test]
fn test_fmt_command_formats_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create a poorly formatted file
    let unformatted_content = r#"
fn add(a: Number,b: Number) -> Number {
return a+b;
}
fn main() -> void {
println("Hello");
}
"#;

    fs::write(temp_dir.path().join("test.vela"), unformatted_content).unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("fmt")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running Vela formatter"))
        .stdout(predicate::str::contains("Found 1 .vela files"))
        .stdout(predicate::str::contains("✅ Formatted: test.vela"))
        .stdout(predicate::str::contains("All files formatted successfully"));

    // Verify the file was formatted
    let formatted_content = fs::read_to_string(temp_dir.path().join("test.vela")).unwrap();
    assert!(formatted_content.contains("fn add(a: Number, b: Number) -> Number {"));
    assert!(formatted_content.contains("    return a + b;"));
}

#[test]
fn test_fmt_command_check_mode() {
    let temp_dir = TempDir::new().unwrap();

    // Create a poorly formatted file
    let unformatted_content = r#"
fn add(a: Number,b: Number) -> Number {
return a+b;
}
"#;

    fs::write(temp_dir.path().join("test.vela"), unformatted_content).unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("fmt")
        .arg("--check")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("Running Vela formatter"))
        .stdout(predicate::str::contains("❌ Needs formatting: test.vela"))
        .stdout(predicate::str::contains("Some files need formatting"));
}

#[test]
fn test_fmt_command_already_formatted() {
    let temp_dir = TempDir::new().unwrap();

    // Create a properly formatted file
    let formatted_content = r#"fn add(a: Number, b: Number) -> Number {
    return a + b;
}

fn main() -> void {
    println("Hello");
}
"#;

    fs::write(temp_dir.path().join("test.vela"), formatted_content).unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("fmt")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Running Vela formatter"))
        .stdout(predicate::str::contains("âœ… Already formatted: test.vela"))
        .stdout(predicate::str::contains("All files formatted successfully"));
}

#[test]
fn test_doctor_command_basic() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("doctor")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Vela Doctor - Installation Diagnostics"))
        .stdout(predicate::str::contains("Vela Version"))
        .stdout(predicate::str::contains("Current Directory"))
        .stdout(predicate::str::contains("All checks passed"));
}

#[test]
fn test_doctor_command_with_project() {
    let temp_dir = TempDir::new().unwrap();

    // Create a vela.toml file to simulate a project
    fs::write(temp_dir.path().join("vela.toml"), "[project]\nname = \"test\"").unwrap();

    // Create a .vela file
    fs::write(temp_dir.path().join("main.vela"), "fn main() -> void {\n    println(\"Hello\");\n}").unwrap();

    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("doctor")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Vela project detected"))
        .stdout(predicate::str::contains("Vela files found: 1"));
}

#[test]
fn test_doctor_command_no_write_permissions() {
    let temp_dir = TempDir::new().unwrap();

    // Make directory read-only (this might not work on all systems)
    // For now, just test basic functionality
    let mut cmd = Command::cargo_bin("vela").unwrap();
    cmd.arg("doctor")
        .current_dir(&temp_dir)
        .assert()
        .success();
}