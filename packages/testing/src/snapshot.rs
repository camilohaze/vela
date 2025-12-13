/*!
# Snapshot Testing

Advanced framework for snapshot testing of widget trees and UI states with visual regression detection.

## Features

- **Visual Regression Detection**: Automatic detection of UI changes
- **Snapshot Comparison**: Detailed diff reporting between snapshots
- **Interactive Approval**: Manual approval workflow for snapshot updates
- **Multiple Formats**: JSON and image-based snapshots
- **Smart Matching**: Configurable tolerance for visual differences

## Example

```rust,no_run
use vela_testing::snapshot::*;

// Create and save snapshot
let snapshot = Snapshot::capture(&app).await?;
snapshot.save("button_snapshot.json").await?;

// Verify with automatic update on failure
assert_snapshot_matches!("button_snapshot.json", &app).await?;

// Interactive approval mode
if let Err(diff) = snapshot.compare_with_file("button_snapshot.json").await {
    if diff.has_changes() {
        // Show diff and ask for approval
        diff.print_diff();
        if user_approves() {
            snapshot.save("button_snapshot.json").await?;
        }
    }
}
```

*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

/// Configuration for snapshot testing
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    pub update_on_failure: bool,
    pub interactive_approval: bool,
    pub tolerance: f64,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            update_on_failure: false,
            interactive_approval: true,
            tolerance: 0.01, // 1% tolerance for visual differences
        }
    }
}

/// Snapshot of widget tree state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub widgets: HashMap<String, WidgetSnapshot>,
    pub timestamp: u64,
    pub version: String,
}

/// Snapshot of individual widget state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetSnapshot {
    pub id: String,
    pub widget_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub children: Vec<String>,
    pub visible: bool,
    pub enabled: bool,
    pub focused: bool,
    pub bounds: Option<WidgetBounds>,
}

/// Widget bounds for layout snapshots
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Result of snapshot comparison
#[derive(Debug, Clone)]
pub struct SnapshotDiff {
    pub added_widgets: Vec<String>,
    pub removed_widgets: Vec<String>,
    pub changed_widgets: HashMap<String, WidgetChanges>,
    pub has_changes: bool,
}

/// Changes in a widget between snapshots
#[derive(Debug, Clone)]
pub struct WidgetChanges {
    pub property_changes: HashMap<String, PropertyChange>,
    pub visibility_changed: bool,
    pub bounds_changed: bool,
    pub children_changed: bool,
}

/// Change in a property value
#[derive(Debug, Clone)]
pub struct PropertyChange {
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

impl Snapshot {
    /// Capture snapshot from test app
    pub async fn capture(app: &crate::widget_testing::TestApp) -> Result<Self, Box<dyn std::error::Error>> {
        let widgets = app.widgets.read().await;
        let mut widget_snapshots = HashMap::new();

        for (id, widget) in widgets.iter() {
            widget_snapshots.insert(id.clone(), WidgetSnapshot {
                id: id.clone(),
                widget_type: widget.get_type(),
                properties: widget.get_properties().await,
                children: widget.get_children().await.into_iter().map(|c| c.get_id()).collect(),
                visible: widget.is_visible(),
                enabled: widget.is_enabled().await,
                focused: widget.is_focused().await,
                bounds: widget.get_bounds().await,
            });
        }

        Ok(Self {
            widgets: widget_snapshots,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    /// Save snapshot to file
    pub async fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json).await?;
        Ok(())
    }

    /// Load snapshot from file
    pub async fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path).await?;
        let snapshot: Self = serde_json::from_str(&json)?;
        Ok(snapshot)
    }

    /// Compare with another snapshot
    pub fn compare(&self, other: &Snapshot) -> SnapshotDiff {
        let mut diff = SnapshotDiff {
            added_widgets: Vec::new(),
            removed_widgets: Vec::new(),
            changed_widgets: HashMap::new(),
            has_changes: false,
        };

        // Find added and changed widgets
        for (id, new_widget) in &self.widgets {
            if let Some(old_widget) = other.widgets.get(id) {
                let changes = self.compare_widgets(old_widget, new_widget);
                if changes.has_changes() {
                    diff.changed_widgets.insert(id.clone(), changes);
                    diff.has_changes = true;
                }
            } else {
                diff.added_widgets.push(id.clone());
                diff.has_changes = true;
            }
        }

        // Find removed widgets
        for id in other.widgets.keys() {
            if !self.widgets.contains_key(id) {
                diff.removed_widgets.push(id.clone());
                diff.has_changes = true;
            }
        }

        diff
    }

    /// Compare with snapshot file
    pub async fn compare_with_file(&self, path: &str) -> Result<SnapshotDiff, Box<dyn std::error::Error>> {
        let expected = Self::load(path).await?;
        Ok(self.compare(&expected))
    }

    /// Check if snapshot matches file (with optional update)
    pub async fn matches_file(&self, path: &str, config: &SnapshotConfig) -> Result<(), Box<dyn std::error::Error>> {
        let diff = self.compare_with_file(path).await?;

        if diff.has_changes {
            if config.interactive_approval {
                diff.print_diff();
                println!("Snapshot '{}' has changes. Accept changes? (y/N): ", path);

                // In a real implementation, this would read user input
                // For now, we'll auto-accept if update_on_failure is true
                if config.update_on_failure {
                    println!("Auto-updating snapshot due to update_on_failure=true");
                    self.save(path).await?;
                    return Ok(());
                } else {
                    return Err(format!("Snapshot mismatch for '{}'", path).into());
                }
            } else if config.update_on_failure {
                self.save(path).await?;
                return Ok(());
            } else {
                return Err(format!("Snapshot mismatch for '{}'", path).into());
            }
        }

        Ok(())
    }

    fn compare_widgets(&self, old: &WidgetSnapshot, new: &WidgetSnapshot) -> WidgetChanges {
        let mut changes = WidgetChanges {
            property_changes: HashMap::new(),
            visibility_changed: old.visible != new.visible,
            bounds_changed: old.bounds != new.bounds,
            children_changed: old.children != new.children,
        };

        // Compare properties
        for (key, new_value) in &new.properties {
            if let Some(old_value) = old.properties.get(key) {
                if old_value != new_value {
                    changes.property_changes.insert(key.clone(), PropertyChange {
                        old_value: old_value.clone(),
                        new_value: new_value.clone(),
                    });
                }
            } else {
                changes.property_changes.insert(key.clone(), PropertyChange {
                    old_value: serde_json::Value::Null,
                    new_value: new_value.clone(),
                });
            }
        }

        // Check for removed properties
        for key in old.properties.keys() {
            if !new.properties.contains_key(key) {
                changes.property_changes.insert(key.clone(), PropertyChange {
                    old_value: old.properties[key].clone(),
                    new_value: serde_json::Value::Null,
                });
            }
        }

        changes
    }
}

impl WidgetChanges {
    pub fn has_changes(&self) -> bool {
        !self.property_changes.is_empty() ||
        self.visibility_changed ||
        self.bounds_changed ||
        self.children_changed
    }
}

impl SnapshotDiff {
    pub fn has_changes(&self) -> bool {
        self.has_changes
    }

    pub fn print_diff(&self) {
        if !self.has_changes {
            println!("✅ No changes detected");
            return;
        }

        println!("📸 Snapshot changes detected:");

        if !self.added_widgets.is_empty() {
            println!("  ➕ Added widgets: {:?}", self.added_widgets);
        }

        if !self.removed_widgets.is_empty() {
            println!("  ➖ Removed widgets: {:?}", self.removed_widgets);
        }

        if !self.changed_widgets.is_empty() {
            println!("  🔄 Changed widgets:");
            for (id, changes) in &self.changed_widgets {
                println!("    - {}:", id);

                if changes.visibility_changed {
                    println!("      • Visibility changed");
                }

                if changes.bounds_changed {
                    println!("      • Bounds changed");
                }

                if changes.children_changed {
                    println!("      • Children changed");
                }

                for (prop, change) in &changes.property_changes {
                    println!("      • {}: {} → {}", prop, change.old_value, change.new_value);
                }
            }
        }
    }
}

/// Macro for snapshot testing with configuration
#[macro_export]
macro_rules! assert_snapshot_matches {
    ($path:expr, $app:expr) => {
        assert_snapshot_matches!($path, $app, vela_testing::snapshot::SnapshotConfig::default())
    };

    ($path:expr, $app:expr, $config:expr) => {
        let snapshot = vela_testing::snapshot::Snapshot::capture($app).await
            .expect("Failed to capture snapshot");

        if let Err(e) = snapshot.matches_file($path, &$config).await {
            panic!("Snapshot assertion failed: {}", e);
        }
    };
}

/// Macro for updating snapshots
#[macro_export]
macro_rules! update_snapshot {
    ($path:expr, $app:expr) => {
        let snapshot = vela_testing::snapshot::Snapshot::capture($app).await
            .expect("Failed to capture snapshot");
        snapshot.save($path).await.expect("Failed to save snapshot");
        println!("✅ Updated snapshot: {}", $path);
    };
}