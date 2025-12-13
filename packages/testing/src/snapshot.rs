/*!
# Snapshot Testing

Framework for snapshot testing of widget trees and UI states.

## Example

```rust,no_run
use vela_testing::snapshot::*;

// Create snapshot
let snapshot = Snapshot::capture(&app).await;
snapshot.save("button_snapshot.json").await;

// Verify against snapshot
assert_snapshot_matches!("button_snapshot.json", &app).await;
```

*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Snapshot of widget tree state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub widgets: HashMap<String, WidgetSnapshot>,
    pub timestamp: u64,
}

/// Snapshot of individual widget state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSnapshot {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub children: Vec<String>,
    pub visible: bool,
    pub enabled: bool,
    pub focused: bool,
}

impl Snapshot {
    /// Capture snapshot from test app
    pub async fn capture(app: &crate::widget_testing::TestApp) -> Self {
        let widgets = app.widgets.read().await;
        let mut widget_snapshots = HashMap::new();

        for (id, widget) in widgets.iter() {
            widget_snapshots.insert(id.clone(), WidgetSnapshot {
                id: id.clone(),
                properties: widget.get_properties(),
                children: widget.get_children(),
                visible: widget.is_visible(),
                enabled: widget.is_enabled(),
                focused: widget.is_focused(),
            });
        }

        Self {
            widgets: widget_snapshots,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Save snapshot to file
    pub async fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Load snapshot from file
    pub async fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = tokio::fs::read_to_string(path).await?;
        let snapshot = serde_json::from_str(&json)?;
        Ok(snapshot)
    }

    /// Compare with another snapshot
    pub fn matches(&self, other: &Snapshot) -> bool {
        // Simple comparison - in real implementation would be more sophisticated
        self.widgets.len() == other.widgets.len() &&
        self.widgets.keys().all(|k| other.widgets.contains_key(k))
    }
}

/// Macro for snapshot testing
#[macro_export]
macro_rules! assert_snapshot_matches {
    ($path:expr, $app:expr) => {
        let current = Snapshot::capture($app).await;
        let expected = Snapshot::load($path).await
            .unwrap_or_else(|_| panic!("Snapshot file '{}' not found", $path));

        if !current.matches(&expected) {
            panic!("Snapshot mismatch for '{}'", $path);
        }
    };
}