/*!
# Meta-Tests for Snapshot Testing Framework

Tests that validate the snapshot testing framework itself works correctly.
These tests use snapshot testing to test itself (self-hosting).
*/

use vela_testing::snapshot::*;
use std::collections::HashMap;
use serde_json::json;

/// Test that snapshot creation works
#[test]
fn test_snapshot_creation() {
    // Create test data
    let test_data = create_test_widget_tree();

    // Create snapshot
    let snapshot = Snapshot::capture(&test_data).unwrap();

    // Verify snapshot was created
    assert!(!snapshot.data().is_empty());
    assert!(snapshot.timestamp() > 0);
}

/// Test that snapshot saving and loading works
#[test]
fn test_snapshot_save_load() {
    let test_data = create_test_widget_tree();
    let snapshot = Snapshot::capture(&test_data).unwrap();

    // Save snapshot
    let file_path = "test_snapshot.json";
    snapshot.save(file_path).unwrap();

    // Load snapshot
    let loaded = Snapshot::load(file_path).unwrap();

    // Verify they match
    assert_eq!(snapshot.data(), loaded.data());

    // Cleanup
    std::fs::remove_file(file_path).unwrap();
}

/// Test that snapshot comparison works
#[test]
fn test_snapshot_comparison() {
    let test_data1 = create_test_widget_tree();
    let test_data2 = create_modified_widget_tree();

    let snapshot1 = Snapshot::capture(&test_data1).unwrap();
    let snapshot2 = Snapshot::capture(&test_data2).unwrap();

    // Compare snapshots
    let diff = snapshot1.compare(&snapshot2).unwrap();

    // Should detect differences
    assert!(diff.has_changes());

    // Should have specific changes
    assert!(!diff.added().is_empty() || !diff.removed().is_empty() || !diff.modified().is_empty());
}

/// Test snapshot approval workflow
#[test]
fn test_snapshot_approval_workflow() {
    let test_data = create_test_widget_tree();
    let snapshot = Snapshot::capture(&test_data).unwrap();

    let file_path = "approval_test.json";

    // First save
    snapshot.save(file_path).unwrap();

    // Modify data
    let modified_data = create_modified_widget_tree();
    let new_snapshot = Snapshot::capture(&modified_data).unwrap();

    // Compare with existing
    let diff = new_snapshot.compare_with_file(file_path).unwrap();

    // Should detect changes
    assert!(diff.has_changes());

    // Simulate approval
    new_snapshot.save(file_path).unwrap();

    // Now comparison should show no changes
    let diff_after_approval = new_snapshot.compare_with_file(file_path).unwrap();
    assert!(!diff_after_approval.has_changes());

    // Cleanup
    std::fs::remove_file(file_path).unwrap();
}

/// Test snapshot configuration
#[test]
fn test_snapshot_configuration() {
    let config = SnapshotConfig {
        update_on_failure: true,
        interactive_approval: false,
        tolerance: 0.01,
        format: SnapshotFormat::JSON,
    };

    let snapshot = Snapshot::with_config(config.clone());

    assert_eq!(snapshot.config().update_on_failure, true);
    assert_eq!(snapshot.config().interactive_approval, false);
    assert_eq!(snapshot.config().tolerance, 0.01);
    assert_eq!(snapshot.config().format, SnapshotFormat::JSON);
}

/// Test different snapshot formats
#[test]
fn test_snapshot_formats() {
    let test_data = create_test_widget_tree();

    // Test JSON format
    let json_snapshot = Snapshot::capture(&test_data).unwrap();
    assert!(json_snapshot.data().starts_with("{"));

    // Test pretty JSON
    let pretty_config = SnapshotConfig {
        format: SnapshotFormat::PrettyJSON,
        ..Default::default()
    };
    let pretty_snapshot = Snapshot::with_config(pretty_config);
    pretty_snapshot.capture(&test_data).unwrap();
    // Pretty JSON should have newlines
    assert!(pretty_snapshot.data().contains("\n"));
}

/// Test snapshot diff reporting
#[test]
fn test_snapshot_diff_reporting() {
    let original = json!({
        "type": "Container",
        "children": [
            {"type": "Button", "text": "OK"},
            {"type": "Button", "text": "Cancel"}
        ]
    });

    let modified = json!({
        "type": "Container",
        "children": [
            {"type": "Button", "text": "OK"},
            {"type": "Button", "text": "Close"}
        ]
    });

    let snapshot1 = Snapshot::from_data(original);
    let snapshot2 = Snapshot::from_data(modified);

    let diff = snapshot1.compare(&snapshot2).unwrap();

    // Should detect the text change
    assert!(diff.has_changes());

    // Check diff output
    let diff_string = diff.to_string();
    assert!(diff_string.contains("Cancel"));
    assert!(diff_string.contains("Close"));
}

/// Test snapshot metadata
#[test]
fn test_snapshot_metadata() {
    let test_data = create_test_widget_tree();
    let mut snapshot = Snapshot::capture(&test_data).unwrap();

    // Add metadata
    snapshot.add_metadata("test_run", "meta_test_001");
    snapshot.add_metadata("environment", "test");

    // Verify metadata
    assert_eq!(snapshot.get_metadata("test_run"), Some(&"meta_test_001".to_string()));
    assert_eq!(snapshot.get_metadata("environment"), Some(&"test".to_string()));
    assert_eq!(snapshot.get_metadata("nonexistent"), None);
}

/// Test snapshot validation
#[test]
fn test_snapshot_validation() {
    let test_data = create_test_widget_tree();
    let snapshot = Snapshot::capture(&test_data).unwrap();

    // Valid snapshot should pass validation
    assert!(snapshot.validate().is_ok());

    // Test with invalid data
    let invalid_snapshot = Snapshot::from_data(json!("invalid"));
    assert!(invalid_snapshot.validate().is_err());
}

/// Test snapshot filtering
#[test]
fn test_snapshot_filtering() {
    let complex_data = create_complex_widget_tree();
    let snapshot = Snapshot::capture(&complex_data).unwrap();

    // Filter to only buttons
    let filtered = snapshot.filter(|value| {
        value.get("type").and_then(|t| t.as_str()) == Some("Button")
    });

    // Should only contain buttons
    let data: serde_json::Value = serde_json::from_str(filtered.data()).unwrap();
    if let serde_json::Value::Array(children) = &data["children"] {
        for child in children {
            assert_eq!(child["type"], "Button");
        }
    }
}

/// Test snapshot merging
#[test]
fn test_snapshot_merging() {
    let data1 = json!({"buttons": [{"text": "OK"}]});
    let data2 = json!({"inputs": [{"placeholder": "Name"}]});

    let snapshot1 = Snapshot::from_data(data1);
    let snapshot2 = Snapshot::from_data(data2);

    let merged = snapshot1.merge(&snapshot2);

    let merged_data: serde_json::Value = serde_json::from_str(merged.data()).unwrap();

    // Should contain both buttons and inputs
    assert!(merged_data.get("buttons").is_some());
    assert!(merged_data.get("inputs").is_some());
}

/// Test snapshot performance
#[test]
fn test_snapshot_performance() {
    let large_data = create_large_widget_tree();

    let start = std::time::Instant::now();
    let snapshot = Snapshot::capture(&large_data).unwrap();
    let capture_time = start.elapsed();

    // Should capture quickly
    assert!(capture_time.as_millis() < 100);

    let start = std::time::Instant::now();
    let _diff = snapshot.compare(&snapshot).unwrap();
    let compare_time = start.elapsed();

    // Self-comparison should be very fast
    assert!(compare_time.as_millis() < 50);
}

/// Test snapshot error handling
#[test]
fn test_snapshot_error_handling() {
    // Test loading non-existent file
    let result = Snapshot::load("non_existent_file.json");
    assert!(result.is_err());

    // Test saving to invalid path
    let snapshot = Snapshot::from_data(json!({"test": "data"}));
    let result = snapshot.save("/invalid/path/file.json");
    assert!(result.is_err());
}

// Helper functions for creating test data

fn create_test_widget_tree() -> serde_json::Value {
    json!({
        "type": "Container",
        "children": [
            {
                "type": "Button",
                "text": "OK",
                "enabled": true
            },
            {
                "type": "TextInput",
                "placeholder": "Enter text",
                "value": ""
            }
        ]
    })
}

fn create_modified_widget_tree() -> serde_json::Value {
    json!({
        "type": "Container",
        "children": [
            {
                "type": "Button",
                "text": "OK",
                "enabled": true
            },
            {
                "type": "TextInput",
                "placeholder": "Enter your name",
                "value": ""
            }
        ]
    })
}

fn create_complex_widget_tree() -> serde_json::Value {
    json!({
        "type": "Container",
        "children": [
            {
                "type": "Button",
                "text": "Submit",
                "enabled": true
            },
            {
                "type": "Container",
                "children": [
                    {
                        "type": "TextInput",
                        "placeholder": "Name",
                        "value": ""
                    },
                    {
                        "type": "Button",
                        "text": "Cancel",
                        "enabled": false
                    }
                ]
            }
        ]
    })
}

fn create_large_widget_tree() -> serde_json::Value {
    let mut children = Vec::new();
    for i in 0..100 {
        children.push(json!({
            "type": "Button",
            "text": format!("Button {}", i),
            "enabled": i % 2 == 0
        }));
    }

    json!({
        "type": "Container",
        "children": children
    })
}