//! iOS Bridge Layer
//!
//! This module implements the FFI bridge between Vela (Rust) and iOS (Swift/Objective-C),
//! enabling communication between the Vela runtime and native iOS components.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

pub mod ffi;
#[cfg(test)]
mod tests;

/// iOS bridge for widget rendering
pub struct VelaIOSBridge {
    /// Pointer to iOS UIViewController
    view_controller: *mut c_void,
    /// State synchronization manager
    state_manager: VelaStateManager,
}

impl VelaIOSBridge {
    /// Create a new iOS bridge
    pub fn new() -> Self {
        Self {
            view_controller: std::ptr::null_mut(),
            state_manager: VelaStateManager::new(),
        }
    }

    /// Initialize the bridge with iOS view controller
    pub fn initialize(&mut self, view_controller: *mut c_void) {
        self.view_controller = view_controller;
        // Initialize iOS-specific setup
        unsafe {
            vela_ios_initialize(view_controller);
        }
    }

    /// Render a widget tree to iOS views
    pub fn render_widget(&self, widget_json: &str) -> Result<(), BridgeError> {
        let c_json = CString::new(widget_json)
            .map_err(|_| BridgeError::InvalidString)?;

        unsafe {
            let result = vela_ios_render_widget(self.view_controller, c_json.as_ptr());
            if result == 0 {
                Ok(())
            } else {
                Err(BridgeError::RenderFailed)
            }
        }
    }

    /// Update widget properties reactively
    pub fn update_widget(&self, widget_id: &str, properties: &str) -> Result<(), BridgeError> {
        let c_id = CString::new(widget_id)
            .map_err(|_| BridgeError::InvalidString)?;
        let c_props = CString::new(properties)
            .map_err(|_| BridgeError::InvalidString)?;

        unsafe {
            let result = vela_ios_update_widget(self.view_controller, c_id.as_ptr(), c_props.as_ptr());
            if result == 0 {
                Ok(())
            } else {
                Err(BridgeError::UpdateFailed)
            }
        }
    }

    /// Handle iOS events and gestures
    pub fn handle_event(&self, event_json: &str) -> Result<(), BridgeError> {
        let c_event = CString::new(event_json)
            .map_err(|_| BridgeError::InvalidString)?;

        unsafe {
            let result = vela_ios_handle_event(self.view_controller, c_event.as_ptr());
            if result == 0 {
                Ok(())
            } else {
                Err(BridgeError::EventFailed)
            }
        }
    }

    /// Get current iOS view hierarchy as JSON
    pub fn get_view_hierarchy(&self) -> Result<String, BridgeError> {
        unsafe {
            let c_str = vela_ios_get_view_hierarchy(self.view_controller);
            if c_str.is_null() {
                return Err(BridgeError::HierarchyFailed);
            }

            let rust_str = CStr::from_ptr(c_str)
                .to_str()
                .map_err(|_| BridgeError::InvalidString)?
                .to_string();

            // Free the C string
            vela_ios_free_string(c_str);

            Ok(rust_str)
        }
    }
}

impl Drop for VelaIOSBridge {
    fn drop(&mut self) {
        if !self.view_controller.is_null() {
            unsafe {
                vela_ios_cleanup(self.view_controller);
            }
        }
    }
}

/// State manager for iOS bridge
pub struct VelaStateManager {
    /// Pending state updates
    pending_updates: Vec<StateUpdate>,
}

impl VelaStateManager {
    pub fn new() -> Self {
        Self {
            pending_updates: Vec::new(),
        }
    }

    /// Queue a state update
    pub fn queue_update(&mut self, update: StateUpdate) {
        self.pending_updates.push(update);
    }

    /// Flush pending updates to iOS
    pub fn flush_updates(&mut self, bridge: &VelaIOSBridge) {
        for update in &self.pending_updates {
            let _ = bridge.update_widget(&update.widget_id, &update.properties);
        }
        self.pending_updates.clear();
    }
}

/// State update structure
pub struct StateUpdate {
    pub widget_id: String,
    pub properties: String,
}

/// Bridge error types
#[derive(Debug, Clone)]
pub enum BridgeError {
    InvalidString,
    RenderFailed,
    UpdateFailed,
    EventFailed,
    HierarchyFailed,
}

/// Opaque pointer to Vela iOS runtime
pub type VelaIOSRuntime = c_void;

/// iOS runtime configuration
#[repr(C)]
pub struct IOSRuntimeConfig {
    /// Enable debug logging
    pub debug_logging: bool,
    /// Maximum UIView pool size
    pub max_view_pool_size: u32,
    /// Enable gesture recognition
    pub enable_gestures: bool,
}

/// iOS touch event data
#[repr(C)]
pub struct IOSTouchEvent {
    /// Event type (0=touch_began, 1=touch_moved, 2=touch_ended)
    pub event_type: u32,
    /// Touch X coordinate
    pub x: f32,
    /// Touch Y coordinate
    pub y: f32,
    /// Touch pressure (0.0-1.0)
    pub pressure: f32,
    /// Timestamp
    pub timestamp: u64,
}

/// iOS gesture event data
#[repr(C)]
pub struct IOSGestureEvent {
    /// Gesture type (0=pinch, 1=rotate, 2=pan, 3=long_press)
    pub gesture_type: u32,
    /// Gesture scale (for pinch)
    pub scale: f32,
    /// Gesture rotation in radians (for rotate)
    pub rotation: f32,
    /// Gesture velocity X (for pan)
    pub velocity_x: f32,
    /// Gesture velocity Y (for pan)
    pub velocity_y: f32,
}

/// iOS rectangle structure
#[repr(C)]
pub struct IOSRect {
    /// Origin X
    pub x: f32,
    /// Origin Y
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

// FFI declarations for iOS functions
extern "C" {
    /// Initialize iOS bridge
    fn vela_ios_initialize(view_controller: *mut c_void) -> i32;

    /// Render widget to iOS view
    fn vela_ios_render_widget(view_controller: *mut c_void, widget_json: *const c_char) -> i32;

    /// Update widget properties
    fn vela_ios_update_widget(
        view_controller: *mut c_void,
        widget_id: *const c_char,
        properties: *const c_char
    ) -> i32;

    /// Handle iOS events
    fn vela_ios_handle_event(view_controller: *mut c_void, event_json: *const c_char) -> i32;

    /// Get view hierarchy as JSON
    fn vela_ios_get_view_hierarchy(view_controller: *mut c_void) -> *const c_char;

    /// Free C string allocated by iOS
    fn vela_ios_free_string(c_str: *const c_char);

    /// Cleanup iOS bridge
    fn vela_ios_cleanup(view_controller: *mut c_void);

    /// Create Vela runtime instance (returns opaque pointer)
    fn vela_ios_create_runtime(config: *const IOSRuntimeConfig) -> *mut VelaIOSRuntime;

    /// Destroy Vela runtime instance
    fn vela_ios_destroy_runtime(runtime: *mut VelaIOSRuntime);

    /// Render widget and return UIView pointer
    fn vela_ios_render_widget(
        runtime: *mut VelaIOSRuntime,
        widget_json: *const c_char,
        parent_view: *mut c_void
    ) -> *mut c_void;

    /// Update existing widget
    fn vela_ios_update_widget(
        runtime: *mut VelaIOSRuntime,
        widget_id: u64,
        updates_json: *const c_char
    ) -> i32;

    /// Destroy widget and free resources
    fn vela_ios_destroy_widget(
        runtime: *mut VelaIOSRuntime,
        widget_id: u64
    ) -> i32;

    /// Handle touch event
    fn vela_ios_handle_touch_event(
        runtime: *mut VelaIOSRuntime,
        widget_id: u64,
        event: *const IOSTouchEvent
    ) -> bool;

    /// Handle gesture event
    fn vela_ios_handle_gesture_event(
        runtime: *mut VelaIOSRuntime,
        widget_id: u64,
        event: *const IOSGestureEvent
    ) -> bool;

    /// Get widget bounds
    fn vela_ios_get_widget_bounds(
        runtime: *mut VelaIOSRuntime,
        widget_id: u64,
        bounds: *mut IOSRect
    ) -> bool;
}

/// iOS event types
#[derive(Debug, Clone)]
pub enum IOSEvent {
    TouchBegan { x: f32, y: f32 },
    TouchMoved { x: f32, y: f32 },
    TouchEnded { x: f32, y: f32 },
    GesturePinch { scale: f32 },
    GestureRotate { rotation: f32 },
    ButtonPressed { button_id: String },
}

/// Convert iOS event to JSON for Vela
impl IOSEvent {
    pub fn to_json(&self) -> String {
        match self {
            IOSEvent::TouchBegan { x, y } => {
                format!(r#"{{"type":"touch_began","x":{},"y":{}}}"#, x, y)
            }
            IOSEvent::TouchMoved { x, y } => {
                format!(r#"{{"type":"touch_moved","x":{},"y":{}}}"#, x, y)
            }
            IOSEvent::TouchEnded { x, y } => {
                format!(r#"{{"type":"touch_ended","x":{},"y":{}}}"#, x, y)
            }
            IOSEvent::GesturePinch { scale } => {
                format!(r#"{{"type":"gesture_pinch","scale":{}}}"#, scale)
            }
            IOSEvent::GestureRotate { rotation } => {
                format!(r#"{{"type":"gesture_rotate","rotation":{}}}"#, rotation)
            }
            IOSEvent::ButtonPressed { button_id } => {
                format!(r#"{{"type":"button_pressed","button_id":"{}"}}"#, button_id)
            }
        }
    }
}