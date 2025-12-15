//! iOS Runtime Module
//!
//! This module provides the main iOS runtime integration for Vela,
//! combining rendering, bridging, layout, and event handling.

pub mod bridge;
pub mod events;
pub mod layout;
pub mod renderer;

use std::sync::Arc;

use self::bridge::VelaIOSBridge;
use self::events::VelaEventBridge;
use self::layout::VelaLayoutEngine;
use self::renderer::VelaWidgetRenderer;

/// Main iOS runtime for Vela applications
pub struct VelaIOSRuntime {
    /// Widget renderer
    renderer: VelaWidgetRenderer,
    /// iOS bridge for FFI
    bridge: VelaIOSBridge,
    /// Layout engine
    layout_engine: VelaLayoutEngine,
    /// Event bridge
    event_bridge: VelaEventBridge,
    /// Runtime state
    state_manager: Arc<renderer::VelaStateManager>,
}

impl VelaIOSRuntime {
    /// Create a new iOS runtime
    pub fn new() -> Self {
        let state_manager = Arc::new(renderer::VelaStateManager::new());

        Self {
            renderer: VelaWidgetRenderer::new(state_manager.clone()),
            bridge: VelaIOSBridge::new(),
            layout_engine: VelaLayoutEngine::new(),
            event_bridge: VelaEventBridge::new(),
            state_manager,
        }
    }

    /// Initialize the runtime with iOS view controller
    pub fn initialize(&mut self, view_controller: *mut std::ffi::c_void) {
        self.bridge.initialize(view_controller);
    }

    /// Render a widget tree
    pub fn render_widget_tree(&mut self, root_widget: &crate::ui::Widget) -> Result<(), RuntimeError> {
        // Calculate layout first
        let layout_result = self.layout_engine.calculate_layout(
            &root_widget.id(),
            375.0, // iPhone width
            667.0, // iPhone height
        );

        // Render widget to UIView
        let ui_view = self.renderer.render(root_widget);

        // Send to iOS via bridge
        let widget_json = self.widget_to_json(root_widget)?;
        self.bridge.render_widget(&widget_json)?;

        Ok(())
    }

    /// Handle iOS touch event
    pub fn handle_touch_event(&mut self, touch_event: events::IOSTouchEvent) {
        self.event_bridge.handle_touch_event(touch_event);
        self.event_bridge.process_events();
    }

    /// Handle iOS gesture event
    pub fn handle_gesture_event(&mut self, gesture_event: events::IOSGestureEvent) {
        self.event_bridge.handle_gesture_event(gesture_event);
        self.event_bridge.process_events();
    }

    /// Update widget properties reactively
    pub fn update_widget(&mut self, widget_id: &str, properties: &str) -> Result<(), RuntimeError> {
        self.bridge.update_widget(widget_id, properties)?;
        Ok(())
    }

    /// Register event handler for widget
    pub fn register_event_handler<F>(&mut self, widget_id: &str, handler: F)
    where
        F: Fn(&events::VelaEvent) + Send + Sync + 'static,
    {
        self.event_bridge.register_handler(widget_id, handler);
    }

    /// Add gesture recognizer
    pub fn add_gesture_recognizer(&mut self, recognizer: events::GestureRecognizer) {
        self.event_bridge.add_gesture_recognizer(recognizer);
    }

    /// Get current view hierarchy (for debugging)
    pub fn get_view_hierarchy(&self) -> Result<String, RuntimeError> {
        self.bridge.get_view_hierarchy()
            .map_err(RuntimeError::BridgeError)
    }

    /// Convert widget to JSON representation
    fn widget_to_json(&self, widget: &crate::ui::Widget) -> Result<String, RuntimeError> {
        // Placeholder implementation
        // In real implementation, this would serialize the widget tree to JSON
        Ok(format!(r#"{{"type":"{}","id":"{}"}}"#, widget.widget_type(), widget.id()))
    }
}

/// Runtime error types
#[derive(Debug)]
pub enum RuntimeError {
    BridgeError(bridge::BridgeError),
    LayoutError(layout::LayoutError),
    SerializationError,
}

impl From<bridge::BridgeError> for RuntimeError {
    fn from(error: bridge::BridgeError) -> Self {
        RuntimeError::BridgeError(error)
    }
}

impl From<layout::LayoutError> for RuntimeError {
    fn from(error: layout::LayoutError) -> Self {
        RuntimeError::LayoutError(error)
    }
}

/// iOS runtime configuration
#[derive(Debug, Clone)]
pub struct IOSRuntimeConfig {
    /// Target iOS version
    pub target_version: String,
    /// Enable debug mode
    pub debug_mode: bool,
    /// Viewport size
    pub viewport_width: f32,
    pub viewport_height: f32,
    /// Performance settings
    pub enable_widget_pooling: bool,
    pub enable_layout_caching: bool,
}

impl Default for IOSRuntimeConfig {
    fn default() -> Self {
        Self {
            target_version: "15.0".to_string(),
            debug_mode: false,
            viewport_width: 375.0, // iPhone 12 width
            viewport_height: 667.0, // iPhone 12 height
            enable_widget_pooling: true,
            enable_layout_caching: true,
        }
    }
}

/// Helper for creating iOS runtime with configuration
pub fn create_ios_runtime(config: IOSRuntimeConfig) -> VelaIOSRuntime {
    let mut runtime = VelaIOSRuntime::new();

    // Apply configuration
    if config.debug_mode {
        // Enable debug logging, etc.
    }

    runtime
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = VelaIOSRuntime::new();
        assert!(runtime.get_view_hierarchy().is_err()); // Should fail without initialization
    }

    #[test]
    fn test_config_defaults() {
        let config = IOSRuntimeConfig::default();
        assert_eq!(config.target_version, "15.0");
        assert_eq!(config.viewport_width, 375.0);
        assert_eq!(config.viewport_height, 667.0);
        assert!(config.enable_widget_pooling);
    }
}