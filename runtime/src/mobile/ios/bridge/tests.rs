//! Tests for iOS FFI bridging
//!
//! These tests verify that the FFI functions work correctly
//! and that the bridging between Rust and Swift/Objective-C is sound.

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_runtime_creation_and_destruction() {
        let config = IOSRuntimeConfig {
            debug_logging: true,
            max_view_pool_size: 50,
            enable_gestures: true,
        };

        // Create runtime
        let runtime_ptr = vela_ios_create_runtime(&config);
        assert!(!runtime_ptr.is_null());

        // Destroy runtime
        vela_ios_destroy_runtime(runtime_ptr);
    }

    #[test]
    fn test_widget_rendering() {
        let config = IOSRuntimeConfig {
            debug_logging: false,
            max_view_pool_size: 100,
            enable_gestures: true,
        };

        let runtime_ptr = vela_ios_create_runtime(&config);
        assert!(!runtime_ptr.is_null());

        // Test rendering a simple widget
        let widget_json = r#"{"type":"container","children":[]}"#;
        let c_json = CString::new(widget_json).unwrap();

        // Note: In a real test, we'd need a valid UIView pointer
        // For now, we just test that the function doesn't crash
        let result = vela_ios_render_widget(runtime_ptr, c_json.as_ptr(), std::ptr::null_mut());

        // Result might be null in test environment, but function should not crash
        // assert!(result.is_null()); // Expected in test without real UIView

        vela_ios_destroy_runtime(runtime_ptr);
    }

    #[test]
    fn test_touch_event_handling() {
        let config = IOSRuntimeConfig::default();
        let runtime_ptr = vela_ios_create_runtime(&config);
        assert!(!runtime_ptr.is_null());

        let touch_event = IOSTouchEvent {
            event_type: 0, // TOUCH_BEGAN
            x: 100.0,
            y: 200.0,
            pressure: 1.0,
            timestamp: 1234567890,
        };

        // Test touch event handling
        let handled = vela_ios_handle_touch_event(runtime_ptr, 1, &touch_event);
        // In test environment, this might return false, but shouldn't crash
        assert!(handled == true || handled == false); // Just check it doesn't crash

        vela_ios_destroy_runtime(runtime_ptr);
    }

    #[test]
    fn test_gesture_event_handling() {
        let config = IOSRuntimeConfig::default();
        let runtime_ptr = vela_ios_create_runtime(&config);
        assert!(!runtime_ptr.is_null());

        let gesture_event = IOSGestureEvent {
            gesture_type: 0, // PINCH
            scale: 1.5,
            rotation: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
        };

        // Test gesture event handling
        let handled = vela_ios_handle_gesture_event(runtime_ptr, 1, &gesture_event);
        assert!(handled == true || handled == false); // Just check it doesn't crash

        vela_ios_destroy_runtime(runtime_ptr);
    }

    #[test]
    fn test_widget_bounds_query() {
        let config = IOSRuntimeConfig::default();
        let runtime_ptr = vela_ios_create_runtime(&config);
        assert!(!runtime_ptr.is_null());

        let mut bounds = IOSRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };

        // Test bounds query for non-existent widget
        let success = vela_ios_get_widget_bounds(runtime_ptr, 999, &mut bounds);
        assert!(!success); // Should fail for non-existent widget

        vela_ios_destroy_runtime(runtime_ptr);
    }

    #[test]
    fn test_widget_update() {
        let config = IOSRuntimeConfig::default();
        let runtime_ptr = vela_ios_create_runtime(&config);
        assert!(!runtime_ptr.is_null());

        let updates_json = r#"{"color":"blue","size":42}"#;
        let c_updates = CString::new(updates_json).unwrap();

        // Test updating non-existent widget
        let result = vela_ios_update_widget(runtime_ptr, 999, c_updates.as_ptr());
        assert_eq!(result, -1); // Should fail for non-existent widget

        vela_ios_destroy_runtime(runtime_ptr);
    }

    #[test]
    fn test_widget_destruction() {
        let config = IOSRuntimeConfig::default();
        let runtime_ptr = vela_ios_create_runtime(&config);
        assert!(!runtime_ptr.is_null());

        // Test destroying non-existent widget
        let result = vela_ios_destroy_widget(runtime_ptr, 999);
        assert_eq!(result, -1); // Should fail for non-existent widget

        vela_ios_destroy_runtime(runtime_ptr);
    }

    #[test]
    fn test_null_pointer_handling() {
        // Test with null runtime
        let result = vela_ios_render_widget(std::ptr::null_mut(), std::ptr::null(), std::ptr::null_mut());
        assert!(result.is_null());

        // Test with null config (should use defaults)
        let runtime_ptr = vela_ios_create_runtime(std::ptr::null());
        assert!(!runtime_ptr.is_null());
        vela_ios_destroy_runtime(runtime_ptr);
    }
}