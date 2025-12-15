//! Vela iOS FFI Implementation
//!
//! This module contains the actual implementations of the FFI functions
//! exposed to Swift/Objective-C for iOS integration.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::mobile::ios::renderer::VelaWidgetRenderer;
use crate::mobile::ios::layout::VelaLayoutEngine;
use crate::mobile::ios::events::VelaEventBridge;
use crate::reactive::state::VelaStateManager;

/// Global registry of active Vela runtimes
static mut RUNTIME_REGISTRY: Option<Mutex<HashMap<u64, Arc<VelaIOSRuntime>>>> = None;

/// Initialize the global runtime registry
fn initialize_registry() {
    unsafe {
        if RUNTIME_REGISTRY.is_none() {
            RUNTIME_REGISTRY = Some(Mutex::new(HashMap::new()));
        }
    }
}

/// Generate unique runtime ID
fn generate_runtime_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// Vela iOS Runtime instance
pub struct VelaIOSRuntime {
    /// Unique runtime identifier
    id: u64,
    /// Widget renderer
    renderer: VelaWidgetRenderer,
    /// Layout engine
    layout_engine: VelaLayoutEngine,
    /// Event bridge
    event_bridge: VelaEventBridge,
    /// State manager
    state_manager: VelaStateManager,
    /// Configuration
    config: IOSRuntimeConfig,
}

impl VelaIOSRuntime {
    /// Create new runtime instance
    fn new(config: IOSRuntimeConfig) -> Self {
        Self {
            id: generate_runtime_id(),
            renderer: VelaWidgetRenderer::new(),
            layout_engine: VelaLayoutEngine::new(),
            event_bridge: VelaEventBridge::new(),
            state_manager: VelaStateManager::new(),
            config,
        }
    }

    /// Get runtime ID
    fn id(&self) -> u64 {
        self.id
    }
}

/// Create Vela runtime instance
#[no_mangle]
pub extern "C" fn vela_ios_create_runtime(config: *const IOSRuntimeConfig) -> *mut VelaIOSRuntime {
    initialize_registry();

    let config = unsafe {
        if config.is_null() {
            IOSRuntimeConfig {
                debug_logging: false,
                max_view_pool_size: 100,
                enable_gestures: true,
            }
        } else {
            *config
        }
    };

    let runtime = Arc::new(VelaIOSRuntime::new(config));

    // Store in global registry
    unsafe {
        if let Some(ref mut registry) = RUNTIME_REGISTRY {
            let mut registry = registry.lock().unwrap();
            let id = runtime.id();
            registry.insert(id, runtime.clone());
        }
    }

    // Return raw pointer (will be managed by Swift ARC)
    Arc::into_raw(runtime) as *mut VelaIOSRuntime
}

/// Destroy Vela runtime instance
#[no_mangle]
pub extern "C" fn vela_ios_destroy_runtime(runtime: *mut VelaIOSRuntime) {
    if runtime.is_null() {
        return;
    }

    // Convert back to Arc and let it drop
    unsafe {
        let _ = Arc::from_raw(runtime);

        // Remove from registry if it exists
        if let Some(ref mut registry) = RUNTIME_REGISTRY {
            if let Ok(mut registry) = registry.lock() {
                // Note: We can't easily get the ID from the raw pointer,
                // so we rely on the Arc drop to clean up naturally
            }
        }
    }
}

/// Render widget and return UIView pointer
#[no_mangle]
pub extern "C" fn vela_ios_render_widget(
    runtime: *mut VelaIOSRuntime,
    widget_json: *const c_char,
    parent_view: *mut c_void
) -> *mut c_void {
    if runtime.is_null() || widget_json.is_null() {
        return std::ptr::null_mut();
    }

    let runtime = unsafe { &*runtime };

    // Convert C string to Rust string
    let widget_json = unsafe {
        match CStr::from_ptr(widget_json).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    // Parse widget JSON and render
    match runtime.renderer.render_widget(widget_json, parent_view) {
        Ok(view_ptr) => view_ptr as *mut c_void,
        Err(_) => std::ptr::null_mut(),
    }
}

/// Update existing widget
#[no_mangle]
pub extern "C" fn vela_ios_update_widget(
    runtime: *mut VelaIOSRuntime,
    widget_id: u64,
    updates_json: *const c_char
) -> i32 {
    if runtime.is_null() || updates_json.is_null() {
        return -1; // Error
    }

    let runtime = unsafe { &*runtime };

    // Convert C string to Rust string
    let updates_json = unsafe {
        match CStr::from_ptr(updates_json).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // Update widget
    match runtime.renderer.update_widget(widget_id, updates_json) {
        Ok(_) => 0, // Success
        Err(_) => -1, // Error
    }
}

/// Destroy widget and free resources
#[no_mangle]
pub extern "C" fn vela_ios_destroy_widget(
    runtime: *mut VelaIOSRuntime,
    widget_id: u64
) -> i32 {
    if runtime.is_null() {
        return -1; // Error
    }

    let runtime = unsafe { &*runtime };

    // Destroy widget
    match runtime.renderer.destroy_widget(widget_id) {
        Ok(_) => 0, // Success
        Err(_) => -1, // Error
    }
}

/// Handle touch event
#[no_mangle]
pub extern "C" fn vela_ios_handle_touch_event(
    runtime: *mut VelaIOSRuntime,
    widget_id: u64,
    event: *const IOSTouchEvent
) -> bool {
    if runtime.is_null() || event.is_null() {
        return false;
    }

    let runtime = unsafe { &*runtime };
    let event = unsafe { &*event };

    // Convert to Vela event and handle
    runtime.event_bridge.handle_touch_event(widget_id, event)
}

/// Handle gesture event
#[no_mangle]
pub extern "C" fn vela_ios_handle_gesture_event(
    runtime: *mut VelaIOSRuntime,
    widget_id: u64,
    event: *const IOSGestureEvent
) -> bool {
    if runtime.is_null() || event.is_null() {
        return false;
    }

    let runtime = unsafe { &*runtime };
    let event = unsafe { &*event };

    // Convert to Vela event and handle
    runtime.event_bridge.handle_gesture_event(widget_id, event)
}

/// Get widget bounds
#[no_mangle]
pub extern "C" fn vela_ios_get_widget_bounds(
    runtime: *mut VelaIOSRuntime,
    widget_id: u64,
    bounds: *mut IOSRect
) -> bool {
    if runtime.is_null() || bounds.is_null() {
        return false;
    }

    let runtime = unsafe { &*runtime };

    // Get widget bounds from layout engine
    match runtime.layout_engine.get_widget_bounds(widget_id) {
        Some(widget_bounds) => {
            unsafe {
                *bounds = IOSRect {
                    x: widget_bounds.x,
                    y: widget_bounds.y,
                    width: widget_bounds.width,
                    height: widget_bounds.height,
                };
            }
            true
        }
        None => false,
    }
}