/*!
# User Interaction Simulation

Simulate user interactions for widget testing.

## Example

```rust,no_run
use vela_testing::interactions::*;

// Simulate tap
await tester.tap(find.by_key("button"));

// Simulate text input
await tester.enter_text(find.by_key("input"), "Hello World");
```

*/

use crate::widget_testing::{TestApp, TestEvent};
use crate::finders::*;

/// Trait for user interactions
#[async_trait::async_trait]
pub trait Interaction: Send + Sync {
    async fn perform(&self, app: &mut TestApp) -> Result<(), String>;
}

/// Tap interaction
pub struct TapInteraction {
    finder: Box<dyn Finder>,
}

impl TapInteraction {
    pub fn new(finder: Box<dyn Finder>) -> Self {
        Self { finder }
    }
}

#[async_trait::async_trait]
impl Interaction for TapInteraction {
    async fn perform(&self, app: &mut TestApp) -> Result<(), String> {
        let widget_ids = {
            let widgets = app.widgets.read().await;
            let found_widgets = self.finder.find(&*widgets).await?;
            if found_widgets.is_empty() {
                return Err("No widgets found for tap interaction".to_string());
            }
            found_widgets.iter().map(|w| w.get_id()).collect::<Vec<_>>()
        };

        for widget_id in widget_ids {
            app.simulate_event(&widget_id, TestEvent::Click).await?;
        }

        Ok(())
    }
}

/// Text input interaction
pub struct TextInputInteraction {
    finder: Box<dyn Finder>,
    text: String,
}

impl TextInputInteraction {
    pub fn new(finder: Box<dyn Finder>, text: &str) -> Self {
        Self {
            finder,
            text: text.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Interaction for TextInputInteraction {
    async fn perform(&self, app: &mut TestApp) -> Result<(), String> {
        let widget_ids = {
            let widgets = app.widgets.read().await;
            let found_widgets = self.finder.find(&*widgets).await?;
            if found_widgets.is_empty() {
                return Err("No widgets found for text input interaction".to_string());
            }
            found_widgets.iter().map(|w| w.get_id()).collect::<Vec<_>>()
        };

        for widget_id in widget_ids {
            app.simulate_event(&widget_id, TestEvent::Input(self.text.clone())).await?;
        }

        Ok(())
    }
}

/// Focus interaction
pub struct FocusInteraction {
    finder: Box<dyn Finder>,
}

impl FocusInteraction {
    pub fn new(finder: Box<dyn Finder>) -> Self {
        Self { finder }
    }
}

#[async_trait::async_trait]
impl Interaction for FocusInteraction {
    async fn perform(&self, app: &mut TestApp) -> Result<(), String> {
        let widget_ids = {
            let widgets = app.widgets.read().await;
            let found_widgets = self.finder.find(&*widgets).await?;
            if found_widgets.is_empty() {
                return Err("No widgets found for focus interaction".to_string());
            }
            found_widgets.iter().map(|w| w.get_id()).collect::<Vec<_>>()
        };

        for widget_id in widget_ids {
            app.simulate_event(&widget_id, TestEvent::Focus).await?;
        }

        Ok(())
    }
}

/// Convenience functions for common interactions
pub mod convenience {
    use super::*;

    /// Tap a widget
    pub async fn tap(app: &mut TestApp, finder: impl Finder) -> Result<(), String> {
        let widget_ids = {
            let widgets = app.widgets.read().await;
            let found_widgets = finder.find(&*widgets).await?;
            if found_widgets.is_empty() {
                return Err("No widgets found for tap".to_string());
            }
            found_widgets.iter().map(|w| w.get_id()).collect::<Vec<_>>()
        };

        for widget_id in widget_ids {
            app.simulate_event(&widget_id, TestEvent::Click).await?;
        }

        Ok(())
    }

    /// Enter text into a widget
    pub async fn enter_text(app: &mut TestApp, finder: impl Finder, text: &str) -> Result<(), String> {
        let widget_ids = {
            let widgets = app.widgets.read().await;
            let found_widgets = finder.find(&*widgets).await?;
            if found_widgets.is_empty() {
                return Err("No widgets found for text input".to_string());
            }
            found_widgets.iter().map(|w| w.get_id()).collect::<Vec<_>>()
        };

        for widget_id in widget_ids {
            app.simulate_event(&widget_id, TestEvent::Input(text.to_string())).await?;
        }

        Ok(())
    }

    /// Focus a widget
    pub async fn focus(app: &mut TestApp, finder: impl Finder) -> Result<(), String> {
        let widget_ids = {
            let widgets = app.widgets.read().await;
            let found_widgets = finder.find(&*widgets).await?;
            if found_widgets.is_empty() {
                return Err("No widgets found for focus".to_string());
            }
            found_widgets.iter().map(|w| w.get_id()).collect::<Vec<_>>()
        };

        for widget_id in widget_ids {
            app.simulate_event(&widget_id, TestEvent::Focus).await?;
        }

        Ok(())
    }

    /// Hover over a widget
    pub async fn hover(app: &mut TestApp, finder: impl Finder) -> Result<(), String> {
        let widget_ids = {
            let widgets = app.widgets.read().await;
            let found_widgets = finder.find(&*widgets).await?;
            if found_widgets.is_empty() {
                return Err("No widgets found for hover".to_string());
            }
            found_widgets.iter().map(|w| w.get_id()).collect::<Vec<_>>()
        };

        for widget_id in widget_ids {
            app.simulate_event(&widget_id, TestEvent::Hover).await?;
        }

        Ok(())
    }
}