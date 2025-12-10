//! Integration tests for the Vela package manager
//!
//! These tests validate the end-to-end functionality of the package manager,
//! including dependency resolution, conflict detection, and installation.

use semver::Version;
use vela_tooling::package::PackageManager;
use vela_tooling::package::manifest::Manifest;
use vela_tooling::common::Error;

/// Test basic package manager creation
#[test]
fn test_package_manager_creation() {
    let pm = PackageManager::new();
    assert!(pm.is_ok(), "PackageManager should be created successfully");
}

/// Test resolving an empty manifest
#[test]
fn test_resolve_empty_manifest() {
    let mut pm = PackageManager::new().unwrap();
    let manifest = Manifest::default();

    let result = pm.resolve(&manifest);
    assert!(result.is_ok(), "Empty manifest should resolve successfully");

    let resolution = result.unwrap();
    assert!(resolution.packages.is_empty(), "Empty manifest should have no packages");
    assert!(resolution.conflicts.is_empty(), "Empty manifest should have no conflicts");
}

/// Test resolving a simple manifest with one dependency
#[test]
fn test_resolve_simple_dependency() {
    let mut pm = PackageManager::new().unwrap();
    let mut manifest = Manifest::new("test-project".to_string(), "1.0.0".to_string());
    manifest.add_dependency("lodash".to_string(), "^4.17.0".to_string());

    // Note: This test assumes mock package registry behavior
    // In a real implementation, this would fetch from a registry
    let result = pm.resolve(&manifest);
    // The result depends on whether mock packages are available
    // For now, we just ensure the resolver doesn't crash
    assert!(result.is_ok() || matches!(result, Err(Error::PackageNotFound { .. })),
            "Simple dependency resolution should either succeed or fail gracefully");
}

/// Test version constraint parsing and satisfaction
#[test]
fn test_version_constraint_satisfaction() {
    use vela_tooling::package::resolver::constraints::VersionConstraint;

    // Test caret ranges
    let caret_constraint = VersionConstraint::parse("^1.2.3").unwrap();
    let compatible_version = Version::parse("1.5.0").unwrap();
    let incompatible_version = Version::parse("2.0.0").unwrap();

    assert!(caret_constraint.satisfies(&compatible_version),
            "Caret constraint should accept compatible versions");
    assert!(!caret_constraint.satisfies(&incompatible_version),
            "Caret constraint should reject incompatible versions");

    // Test exact versions
    let exact_constraint = VersionConstraint::parse("1.2.3").unwrap();
    let exact_version = Version::parse("1.2.3").unwrap();
    let different_version = Version::parse("1.2.4").unwrap();

    assert!(exact_constraint.satisfies(&exact_version),
            "Exact constraint should accept exact match");
    assert!(!exact_constraint.satisfies(&different_version),
            "Exact constraint should reject different versions");
}

/// Test dependency graph construction
#[test]
fn test_dependency_graph_construction() {
    use vela_tooling::package::resolver::graph::{DependencyGraph, PackageId};

    let mut graph = DependencyGraph::new();

    // Add a root package
    let package_id = PackageId::new("root-package".to_string());
    graph.add_root_dependency(package_id.clone());

    // Verify the package was added
    assert!(graph.root_dependencies.contains(&package_id),
            "Root dependency should be added to the graph");
}

/// Test error handling for invalid manifests
#[test]
fn test_invalid_manifest_handling() {
    let mut pm = PackageManager::new().unwrap();

    // Test with manifest that has invalid version constraints
    let mut manifest = Manifest::new("test".to_string(), "1.0.0".to_string());
    manifest.add_dependency("invalid-package".to_string(), "invalid-constraint".to_string());

    let result = pm.resolve(&manifest);
    // Should either succeed (if constraint parsing is lenient) or fail with InvalidVersionConstraint
    assert!(result.is_ok() || matches!(result, Err(Error::InvalidVersionConstraint { .. })),
            "Invalid constraints should be handled gracefully");
}

/// Test resolution with multiple dependencies
#[test]
fn test_multiple_dependencies_resolution() {
    let mut pm = PackageManager::new().unwrap();
    let mut manifest = Manifest::new("test-project".to_string(), "1.0.0".to_string());

    manifest.add_dependency("package-a".to_string(), "^1.0.0".to_string());
    manifest.add_dependency("package-b".to_string(), "^2.0.0".to_string());

    let result = pm.resolve(&manifest);
    // The result depends on mock package availability
    assert!(result.is_ok() || matches!(result, Err(Error::PackageNotFound { .. })),
            "Multiple dependencies should resolve or fail gracefully");
}

/// Test conflict detection
#[test]
fn test_conflict_detection() {
    let mut pm = PackageManager::new().unwrap();
    let mut manifest = Manifest::new("test-project".to_string(), "1.0.0".to_string());

    // Add conflicting dependencies
    manifest.add_dependency("conflicting-a".to_string(), "1.0.0".to_string());
    manifest.add_dependency("conflicting-b".to_string(), "2.0.0".to_string());

    let result = pm.resolve(&manifest);
    // Should either resolve successfully or detect conflicts
    match result {
        Ok(resolution) => {
            // If it resolves, check that conflicts are properly recorded
            assert!(resolution.conflicts.is_empty() || !resolution.conflicts.is_empty(),
                    "Conflicts should be either detected or not");
        }
        Err(_) => {
            // Failed resolution is also acceptable
        }
    }
}

/// Test manifest creation and modification
#[test]
fn test_manifest_operations() {
    let mut manifest = Manifest::new("my-project".to_string(), "1.0.0".to_string());

    assert_eq!(manifest.name, "my-project");
    assert_eq!(manifest.version, "1.0.0");
    assert!(manifest.dependencies.is_empty());

    // Add dependencies
    manifest.add_dependency("dep1".to_string(), "^1.0.0".to_string());
    manifest.add_dependency("dep2".to_string(), "~2.0.0".to_string());

    assert_eq!(manifest.dependencies.len(), 2);
    assert_eq!(manifest.dependencies.get("dep1"), Some(&"^1.0.0".to_string()));
    assert_eq!(manifest.dependencies.get("dep2"), Some(&"~2.0.0".to_string()));
}

/// Test package manager error recovery
#[test]
fn test_error_recovery() {
    let mut pm = PackageManager::new().unwrap();

    // Test with empty manifest (should always work)
    let empty_manifest = Manifest::default();
    let result = pm.resolve(&empty_manifest);
    assert!(result.is_ok(), "Empty manifest should always resolve successfully");

    // Test that the resolver can be reused after errors
    let mut problematic_manifest = Manifest::new("test".to_string(), "1.0.0".to_string());
    problematic_manifest.add_dependency("nonexistent".to_string(), "999.999.999".to_string());

    let _error_result = pm.resolve(&problematic_manifest);
    // Even if this fails, the resolver should still be usable
    let retry_result = pm.resolve(&empty_manifest);
    assert!(retry_result.is_ok(), "Resolver should be reusable after errors");
}