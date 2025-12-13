/*!
# Matchers

Matchers for asserting widget states and properties in tests.

## Example

```rust,no_run
use vela_testing::matchers::*;

// Text matcher
expect(find.by_key("title")).to_have_text("Hello World");

// Visibility matcher
expect(find.by_key("button")).to_be_visible();

// Style matcher
expect(find.by_key("container")).to_have_style(Style { color: "red" });
```

*/

use crate::widget_testing::TestApp;
use crate::finders::Finder;

/// Trait for matchers that can check widget states
#[async_trait::async_trait]
pub trait Matcher: Send + Sync {
    async fn matches(&self, app: &TestApp) -> Result<(), String>;
}

/// Text content matcher
pub struct TextMatcher {
    pub finder: Box<dyn Finder>,
    pub expected_text: String,
}

impl TextMatcher {
    pub fn new<F: Finder + 'static>(finder: F, text: impl Into<String>) -> Self {
        Self {
            finder: Box::new(finder),
            expected_text: text.into(),
        }
    }
}

#[async_trait::async_trait]
impl Matcher for TextMatcher {
    async fn matches(&self, app: &TestApp) -> Result<(), String> {
        let widgets = app.widgets.read().await;
        let found_widgets = self.finder.find(&*widgets).await?;

        for widget in found_widgets {
            if let Some(text) = widget.get_properties().get("text") {
                if let Some(text_str) = text.as_str() {
                    if text_str != self.expected_text {
                        return Err(format!(
                            "Expected text '{}' but found '{}'",
                            self.expected_text, text_str
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Visibility matcher
pub struct VisibilityMatcher {
    pub finder: Box<dyn Finder>,
    pub expected_visible: bool,
}

impl VisibilityMatcher {
    pub fn visible<F: Finder + 'static>(finder: F) -> Self {
        Self {
            finder: Box::new(finder),
            expected_visible: true,
        }
    }

    pub fn not_visible<F: Finder + 'static>(finder: F) -> Self {
        Self {
            finder: Box::new(finder),
            expected_visible: false,
        }
    }
}

#[async_trait::async_trait]
impl Matcher for VisibilityMatcher {
    async fn matches(&self, app: &TestApp) -> Result<(), String> {
        let widgets = app.widgets.read().await;
        let found_widgets = self.finder.find(&*widgets).await?;

        for widget in found_widgets {
            if widget.is_visible() != self.expected_visible {
                return Err(format!(
                    "Expected widget to be {} but it was {}",
                    if self.expected_visible { "visible" } else { "not visible" },
                    if widget.is_visible() { "visible" } else { "not visible" }
                ));
            }
        }

        Ok(())
    }
}

/// Style matcher
pub struct StyleMatcher {
    pub finder: Box<dyn Finder>,
    pub expected_style: serde_json::Value,
}

impl StyleMatcher {
    pub fn new<F: Finder + 'static>(finder: F, style: serde_json::Value) -> Self {
        Self {
            finder: Box::new(finder),
            expected_style: style,
        }
    }
}

#[async_trait::async_trait]
impl Matcher for StyleMatcher {
    async fn matches(&self, app: &TestApp) -> Result<(), String> {
        let widgets = app.widgets.read().await;
        let found_widgets = self.finder.find(&*widgets).await?;

        for widget in found_widgets {
            if let Some(style) = widget.get_properties().get("style") {
                if style != &self.expected_style {
                    return Err(format!(
                        "Expected style {:?} but found {:?}",
                        self.expected_style, style
                    ));
                }
            }
        }

        Ok(())
    }
}

/// State matcher for reactive state
pub struct StateMatcher {
    pub widget_id: String,
    pub expected_state: serde_json::Value,
}

impl StateMatcher {
    pub fn new(widget_id: &str, state: serde_json::Value) -> Self {
        Self {
            widget_id: widget_id.to_string(),
            expected_state: state,
        }
    }
}

#[async_trait::async_trait]
impl Matcher for StateMatcher {
    async fn matches(&self, app: &TestApp) -> Result<(), String> {
        if let Some(widget) = app.get_widget(&self.widget_id).await {
            if let Some(state) = widget.get_properties().get("state") {
                if state != &self.expected_state {
                    return Err(format!(
                        "Expected state {:?} but found {:?}",
                        self.expected_state, state
                    ));
                }
            }
        } else {
            return Err(format!("Widget '{}' not found", self.widget_id));
        }

        Ok(())
    }
}

/// Convenience functions for creating matchers
pub mod expect {
    use super::*;

    pub fn to_have_text<F: Finder + 'static>(finder: F, text: impl Into<String>) -> TextMatcher {
        TextMatcher::new(finder, text)
    }

    pub fn to_be_visible<F: Finder + 'static>(finder: F) -> VisibilityMatcher {
        VisibilityMatcher::visible(finder)
    }

    pub fn to_be_not_visible<F: Finder + 'static>(finder: F) -> VisibilityMatcher {
        VisibilityMatcher::not_visible(finder)
    }

    pub fn to_have_style<F: Finder + 'static>(finder: F, style: serde_json::Value) -> StyleMatcher {
        StyleMatcher::new(finder, style)
    }

    pub fn to_have_state(widget_id: &str, state: serde_json::Value) -> StateMatcher {
        StateMatcher::new(widget_id, state)
    }
}