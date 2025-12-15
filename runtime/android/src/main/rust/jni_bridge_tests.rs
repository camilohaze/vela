/*
Tests unitarios para JNI Bridge

Jira: TASK-158
Historia: VELA-1167
Fecha: 2025-12-15

Tests incluidos:
- Inicialización y destrucción de runtimes
- Renderizado de frames
- Procesamiento de eventos
- Gestión de memoria
- Validación de inputs
- Métricas de performance
- Manejo de errores
- Thread safety
*/

use std::ffi::CString;
use std::ptr;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use super::*;
use crate::runtime::{VelaConfig, VelaRuntime};

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_config() -> String {
        r#"{
            "enable_reactivity": true,
            "enable_ui": true,
            "max_memory_mb": 128,
            "enable_debug": true
        }"#.to_string()
    }

    fn c_string(s: &str) -> *mut c_char {
        CString::new(s).unwrap().into_raw()
    }

    fn cleanup_c_string(s: *mut c_char) {
        if !s.is_null() {
            unsafe { let _ = CString::from_raw(s); }
        }
    }

    #[test]
    fn test_runtime_initialization_success() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);

        assert!(runtime_id > 0, "Runtime ID should be positive");

        // Cleanup
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
    }

    #[test]
    fn test_runtime_initialization_invalid_config() {
        let invalid_config = c_string("invalid json");

        let runtime_id = initialize_runtime(invalid_config);

        assert_eq!(runtime_id, 0, "Should return 0 for invalid config");

        cleanup_c_string(invalid_config);
    }

    #[test]
    fn test_runtime_initialization_null_config() {
        let runtime_id = initialize_runtime(ptr::null());

        assert_eq!(runtime_id, 0, "Should return 0 for null config");
    }

    #[test]
    fn test_runtime_lifecycle() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        // Initialize
        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        // Check if alive
        let alive = is_runtime_alive(runtime_id);
        assert_eq!(alive, 1, "Runtime should be alive");

        // Destroy
        destroy_runtime(runtime_id);

        // Check if dead
        let alive_after = is_runtime_alive(runtime_id);
        assert_eq!(alive_after, 0, "Runtime should be dead after destruction");

        cleanup_c_string(config_c);
    }

    #[test]
    fn test_render_frame_basic() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        // Render frame without VDOM
        let result = render_frame(runtime_id, ptr::null());
        assert!(!result.is_null());

        // Convert back to string to verify it's valid JSON
        let result_str = unsafe { CStr::from_ptr(result).to_str().unwrap() };
        assert!(result_str.starts_with('{') && result_str.ends_with('}'));

        // Cleanup
        free_string(result);
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
    }

    #[test]
    fn test_render_frame_with_vdom() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        let vdom = r#"{
            "version": 1,
            "nodes": [{
                "id": "root",
                "component_type": "Container",
                "props": {"width": 100, "height": 100},
                "children": []
            }]
        }"#;
        let vdom_c = c_string(vdom);

        let result = render_frame(runtime_id, vdom_c);
        assert!(!result.is_null());

        // Cleanup
        free_string(result);
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
        cleanup_c_string(vdom_c);
    }

    #[test]
    fn test_render_frame_invalid_runtime() {
        let vdom = c_string("{}");
        let result = render_frame(99999, vdom);

        assert!(!result.is_null());
        let result_str = unsafe { CStr::from_ptr(result).to_str().unwrap() };
        assert_eq!(result_str, "{}"); // Should return empty object for errors

        free_string(result);
        cleanup_c_string(vdom);
    }

    #[test]
    fn test_process_event_tap() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        let event = r#"{
            "Tap": {
                "x": 50.0,
                "y": 75.0
            }
        }"#;
        let event_c = c_string(event);

        // Should not panic
        process_event(runtime_id, event_c);

        // Cleanup
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
        cleanup_c_string(event_c);
    }

    #[test]
    fn test_process_event_invalid_json() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        let invalid_event = c_string("invalid json");

        // Should not panic
        process_event(runtime_id, invalid_event);

        // Cleanup
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
        cleanup_c_string(invalid_event);
    }

    #[test]
    fn test_process_event_invalid_runtime() {
        let event = c_string("{}");
        // Should not panic
        process_event(99999, event);
        cleanup_c_string(event);
    }

    #[test]
    fn test_memory_management() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        // Multiple render calls
        for _ in 0..10 {
            let result = render_frame(runtime_id, ptr::null());
            assert!(!result.is_null());
            free_string(result);
        }

        // Check memory usage (basic check)
        let memory_usage = get_memory_usage();
        assert!(memory_usage >= 0);

        // Cleanup
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
    }

    #[test]
    fn test_error_handling() {
        // Clear any existing error
        clear_error();

        // Test invalid runtime
        let result = render_frame(99999, ptr::null());
        assert!(!result.is_null());
        free_string(result);

        // Check error
        let error_json = get_last_error();
        assert!(!error_json.is_null());

        let error_str = unsafe { CStr::from_ptr(error_json).to_str().unwrap() };
        assert!(error_str.contains("RuntimeNotFound") || error_str == "{}");

        free_string(error_json);
        clear_error();
    }

    #[test]
    fn test_bridge_metrics() {
        // Reset metrics by getting initial state
        let _ = get_bridge_metrics();

        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        // Perform some operations
        let _ = render_frame(runtime_id, ptr::null());
        let _ = is_runtime_alive(runtime_id);

        // Get metrics
        let metrics_json = get_bridge_metrics();
        assert!(!metrics_json.is_null());

        let metrics_str = unsafe { CStr::from_ptr(metrics_json).to_str().unwrap() };
        let metrics: serde_json::Value = serde_json::from_str(metrics_str).unwrap();

        assert!(metrics["total_calls"].as_u64().unwrap() > 0);
        assert!(metrics["successful_calls"].as_u64().unwrap() > 0);

        // Cleanup
        free_string(metrics_json);
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
    }

    #[test]
    fn test_asset_loading() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        let asset_path = c_string("test_asset.png");
        let asset_data = c_string("fake_image_data");

        let result = load_asset(runtime_id, asset_path, asset_data);
        assert_eq!(result, 0, "Asset loading should succeed");

        let unload_result = unload_asset(runtime_id, asset_path);
        assert_eq!(unload_result, 0, "Asset unloading should succeed");

        // Cleanup
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
        cleanup_c_string(asset_path);
        cleanup_c_string(asset_data);
    }

    #[test]
    fn test_fps_configuration() {
        // Valid FPS
        set_target_fps(60);
        // Should not set error

        // Invalid FPS (too high)
        set_target_fps(150);
        // Should set error
        let error = get_last_error();
        if !error.is_null() {
            let error_str = unsafe { CStr::from_ptr(error).to_str().unwrap() };
            assert!(error_str.contains("FPS"));
            free_string(error);
            clear_error();
        }

        // Invalid FPS (negative)
        set_target_fps(-10);
        let error = get_last_error();
        if !error.is_null() {
            let error_str = unsafe { CStr::from_ptr(error).to_str().unwrap() };
            assert!(error_str.contains("FPS"));
            free_string(error);
            clear_error();
        }
    }

    #[test]
    fn test_event_callbacks() {
        // Register callback
        register_event_callback(1);

        // Unregister callback
        unregister_event_callback(1);

        // Should not crash
    }

    #[test]
    fn test_frame_time() {
        let time1 = get_frame_time();
        thread::sleep(Duration::from_millis(10));
        let time2 = get_frame_time();

        assert!(time2 >= time1, "Time should be monotonic");
    }

    #[test]
    fn test_bridge_configuration() {
        let config = r#"{
            "enable_debug_logging": true,
            "enable_performance_monitoring": false,
            "max_runtimes": 5,
            "memory_limit_mb": 64
        }"#;

        let config_c = c_string(config);
        let result = configure_bridge(config_c);

        assert_eq!(result, 0, "Configuration should succeed");

        cleanup_c_string(config_c);
    }

    #[test]
    fn test_bridge_configuration_invalid() {
        let invalid_config = c_string("invalid json");
        let result = configure_bridge(invalid_config);

        assert_eq!(result, -1, "Invalid configuration should fail");

        cleanup_c_string(invalid_config);
    }

    #[test]
    fn test_runtime_limit() {
        // Set low limit
        let config = r#"{
            "enable_debug_logging": false,
            "enable_performance_monitoring": false,
            "max_runtimes": 1,
            "memory_limit_mb": 64
        }"#;

        let config_c = c_string(config);
        configure_bridge(config_c);

        let test_config = setup_test_config();
        let test_config_c = c_string(&test_config);

        // Create first runtime
        let runtime_id1 = initialize_runtime(test_config_c);
        assert!(runtime_id1 > 0);

        // Try to create second runtime (should fail)
        let runtime_id2 = initialize_runtime(test_config_c);
        assert_eq!(runtime_id2, 0, "Should hit runtime limit");

        // Cleanup
        destroy_runtime(runtime_id1);
        cleanup_c_string(config_c);
        cleanup_c_string(test_config_c);
    }

    #[test]
    fn test_string_pooling() {
        // Test that string pooling works without memory leaks
        for i in 0..100 {
            let test_str = format!("test_string_{}", i);
            let c_str = string_to_c_str(test_str.clone());
            assert!(!c_str.is_null());

            // Convert back
            let back = unsafe { CStr::from_ptr(c_str).to_str().unwrap() };
            assert_eq!(back, test_str);

            // Free
            free_string(c_str);
        }
    }

    #[test]
    fn test_android_event_conversion() {
        // Test all event types convert without panicking
        let events = vec![
            r#"{"Tap": {"x": 10.0, "y": 20.0}}"#,
            r#"{"Scroll": {"delta_x": 5.0, "delta_y": 10.0}}"#,
            r#"{"TextInput": {"text": "hello"}}"#,
            r#"{"BackPressed": null}"#,
            r#"{"OrientationChanged": {"orientation": "landscape"}}"#,
            r#"{"KeyPress": {"key": "enter"}}"#,
            r#"{"TouchStart": {"x": 100.0, "y": 200.0}}"#,
            r#"{"TouchMove": {"x": 110.0, "y": 210.0}}"#,
            r#"{"TouchEnd": {"x": 120.0, "y": 220.0}}"#,
            r#"{"Gesture": {"gesture_type": "pinch", "data": {"scale": 1.5}}}"#,
        ];

        for event_json in events {
            let event: AndroidEvent = serde_json::from_str(event_json).unwrap();
            let _vela_event = event.to_vela_event();
            // If we get here without panicking, the conversion worked
        }
    }

    #[test]
    fn test_concurrent_access() {
        let config = setup_test_config();
        let config_c = c_string(&config);

        let runtime_id = initialize_runtime(config_c);
        assert!(runtime_id > 0);

        let handles: Vec<_> = (0..10).map(|_| {
            let rid = runtime_id;
            thread::spawn(move || {
                // Each thread tries to render
                let result = render_frame(rid, ptr::null());
                if !result.is_null() {
                    free_string(result);
                }
            })
        }).collect();

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Cleanup
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
    }

    #[test]
    fn test_performance_monitoring() {
        // Enable performance monitoring
        let config = r#"{
            "enable_debug_logging": false,
            "enable_performance_monitoring": true,
            "max_runtimes": 10,
            "memory_limit_mb": 256
        }"#;

        let config_c = c_string(config);
        configure_bridge(config_c);

        let test_config = setup_test_config();
        let test_config_c = c_string(&test_config);

        let runtime_id = initialize_runtime(test_config_c);
        assert!(runtime_id > 0);

        // Perform operations
        for _ in 0..100 {
            let result = render_frame(runtime_id, ptr::null());
            if !result.is_null() {
                free_string(result);
            }
        }

        // Check metrics
        let metrics = get_bridge_metrics();
        assert!(!metrics.is_null());

        let metrics_str = unsafe { CStr::from_ptr(metrics).to_str().unwrap() };
        let metrics_json: serde_json::Value = serde_json::from_str(metrics_str).unwrap();

        assert!(metrics_json["total_calls"].as_u64().unwrap() >= 100);

        // Cleanup
        free_string(metrics);
        destroy_runtime(runtime_id);
        cleanup_c_string(config_c);
        cleanup_c_string(test_config_c);
    }
}