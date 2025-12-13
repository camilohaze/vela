/*!
# Widget Testing Framework

Framework for testing UI widgets with support for:
- User interaction simulation
- State verification
- Reactive updates testing
- Component lifecycle testing

This module provides the core testing infrastructure that can be extended
by UI frameworks to provide widget-specific testing capabilities.
*/

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

/// Abstract widget trait for testing
#[async_trait::async_trait]
pub trait TestableWidget: Send + Sync {
    fn get_id(&self) -> String;
    fn get_type(&self) -> String;
    async fn get_properties(&self) -> HashMap<String, serde_json::Value>;
    async fn get_children(&self) -> Vec<Box<dyn TestableWidget>>;
    fn is_visible(&self) -> bool;
    async fn is_enabled(&self) -> bool;
    async fn is_focused(&self) -> bool;
    async fn get_bounds(&self) -> Option<crate::snapshot::WidgetBounds>;
    async fn clone_box(&self) -> Box<dyn TestableWidget>;
}

/// Test application that manages widget tree and interactions
#[derive(Clone)]
pub struct TestApp {
    pub widgets: Arc<RwLock<HashMap<String, Box<dyn TestableWidget>>>>,
    event_handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn Fn(&TestEvent) + Send + Sync>>>>>,
    event_log: Arc<RwLock<Vec<(String, TestEvent)>>>,
}

impl TestApp {
    pub fn new() -> Self {
        Self {
            widgets: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            event_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Mount a widget in the test app
    pub async fn mount(&mut self, widget: Box<dyn TestableWidget>) -> Result<(), String> {
        let id = widget.get_id();
        let mut widgets = self.widgets.write().await;
        widgets.insert(id, widget);
        Ok(())
    }

    /// Get a widget by ID
    pub async fn get_widget(&self, id: &str) -> Option<Box<dyn TestableWidget>> {
        let widgets = self.widgets.read().await;
        if let Some(w) = widgets.get(id) {
            Some(w.clone_box().await)
        } else {
            None
        }
    }

    /// Simulate an event on a widget
    pub async fn simulate_event(&mut self, widget_id: &str, event: TestEvent) -> Result<(), String> {
        // Log the event
        {
            let mut event_log = self.event_log.write().await;
            event_log.push((widget_id.to_string(), event.clone()));
        }

        // Execute handlers
        {
            let event_handlers = self.event_handlers.read().await;
            if let Some(handlers) = event_handlers.get(widget_id) {
                for handler in handlers {
                    handler(&event);
                }
            }
        }

        // Simulate state changes
        self.simulate_state_change(widget_id, &event).await
    }

    /// Register an event handler for a widget
    pub async fn register_event_handler<F>(&mut self, widget_id: &str, handler: F)
    where
        F: Fn(&TestEvent) + Send + Sync + 'static,
    {
        let mut event_handlers = self.event_handlers.write().await;
        event_handlers
            .entry(widget_id.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    /// Get event log
    pub async fn get_event_log(&self) -> Vec<(String, TestEvent)> {
        let event_log = self.event_log.read().await;
        event_log.clone()
    }

    /// Clear event log
    pub async fn clear_event_log(&mut self) {
        let mut event_log = self.event_log.write().await;
        event_log.clear();
    }

    /// Simulate state changes based on events
    async fn simulate_state_change(&mut self, widget_id: &str, event: &TestEvent) -> Result<(), String> {
        let mut widgets = self.widgets.write().await;
        if let Some(widget) = widgets.get_mut(widget_id) {
            // This would be implemented by concrete widget types
            // For now, just return success
            Ok(())
        } else {
            Err(format!("Widget '{}' not found", widget_id))
        }
    }
}

/// Widget tester for performing test operations
pub struct WidgetTester {
    app: Arc<RwLock<TestApp>>,
}

impl WidgetTester {
    pub fn new(app: &TestApp) -> Self {
        Self {
            app: Arc::new(RwLock::new(app.clone())),
        }
    }

    /// Find widgets using a finder
    pub async fn find(&self, finder: Box<dyn crate::finders::Finder>) -> Result<Vec<Box<dyn TestableWidget>>, String> {
        let app = self.app.read().await;
        let widgets = app.widgets.read().await;
        finder.find(&*widgets).await
    }

    /// Perform an interaction
    pub async fn perform(&self, interaction: Box<dyn crate::interactions::Interaction>) -> Result<(), String> {
        let mut app = self.app.write().await;
        interaction.perform(&mut *app).await
    }

    /// Expect a matcher to pass
    pub async fn expect(&self, matcher: Box<dyn crate::matchers::Matcher>) -> Result<(), String> {
        matcher.matches(&*self.app.read().await).await
    }
}

/// Test events that can be simulated
#[derive(Debug, Clone, PartialEq)]
pub enum TestEvent {
    Click,
    DoubleClick,
    Hover,
    Unhover,
    Focus,
    Blur,
    KeyPress(String),
    Input(String),
    Scroll(i32, i32),
    Drag(i32, i32),
    Custom(String, serde_json::Value),
}

/// Test utilities and helpers
pub mod test_utils {
    use super::*;

    /// Create a test widget with common setup
    pub fn create_test_widget() -> Box<dyn TestableWidget> {
        // Implementation for creating test widgets
        Box::new(MockWidget::new("test-widget", "TestWidget"))
    }

    /// Setup test environment
    pub async fn setup_test_env() -> TestApp {
        let mut app = TestApp::new();
        // Additional test setup
        app
    }

    /// Cleanup test environment
    pub async fn cleanup_test_env(_app: TestApp) {
        // Cleanup logic
    }
}

/// Mock widget for testing
pub struct MockWidget {
    id: String,
    widget_type: String,
    properties: HashMap<String, serde_json::Value>,
    children: Vec<Box<dyn TestableWidget>>,
    visible: bool,
    enabled: bool,
    focused: bool,
    bounds: Option<crate::snapshot::WidgetBounds>,
}

impl MockWidget {
    pub fn new(id: &str, widget_type: &str) -> Self {
        Self {
            id: id.to_string(),
            widget_type: widget_type.to_string(),
            properties: HashMap::new(),
            children: Vec::new(),
            visible: true,
            enabled: true,
            focused: false,
            bounds: None,
        }
    }

    pub fn with_property(mut self, key: &str, value: serde_json::Value) -> Self {
        self.properties.insert(key.to_string(), value);
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn TestableWidget>>) -> Self {
        self.children = children;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_bounds(mut self, bounds: crate::snapshot::WidgetBounds) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn with_widget_type(mut self, widget_type: &str) -> Self {
        self.widget_type = widget_type.to_string();
        self
    }
}

#[async_trait::async_trait]
impl TestableWidget for MockWidget {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_type(&self) -> String {
        self.widget_type.clone()
    }

    async fn get_properties(&self) -> HashMap<String, serde_json::Value> {
        self.properties.clone()
    }

    async fn get_children(&self) -> Vec<Box<dyn TestableWidget>> {
        let mut children = Vec::new();
        for child in &self.children {
            children.push(child.clone_box().await);
        }
        children
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    async fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn is_focused(&self) -> bool {
        self.focused
    }

    async fn get_bounds(&self) -> Option<crate::snapshot::WidgetBounds> {
        self.bounds.clone()
    }

    async fn clone_box(&self) -> Box<dyn TestableWidget> {
        Box::new(MockWidget {
            id: self.id.clone(),
            widget_type: self.widget_type.clone(),
            properties: self.properties.clone(),
            children: {
                let mut cloned_children = Vec::new();
                for child in &self.children {
                    cloned_children.push(child.clone_box().await);
                }
                cloned_children
            },
            visible: self.visible,
            enabled: self.enabled,
            focused: self.focused,
            bounds: self.bounds.clone(),
        })
    }
}