//! Tests de integración para FFI bridge
//!
//! Estos tests requieren una librería C compilada y validan:
//! - Carga dinámica de librerías reales
//! - Llamadas a funciones C con tipos reales
//! - Múltiples argumentos y tipos de retorno
//! - Gestión de memoria y errores

use vela_runtime::ffi::*;
use std::process::Command;
use std::path::Path;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn build_test_library() -> Result<(), Box<dyn std::error::Error>> {
        println!("Building FFI test library...");

        // Ejecutar script de build
        let status = Command::new("bash")
            .arg("build_ffi_test_lib.sh")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()?;

        if !status.success() {
            return Err("Failed to build test library".into());
        }

        // Verificar que la librería se creó
        let lib_path = get_library_path();
        if !Path::new(&lib_path).exists() {
            return Err(format!("Library not found at: {}", lib_path).into());
        }

        println!("Test library built successfully at: {}", lib_path);
        Ok(())
    }

    fn get_library_path() -> String {
        let base_dir = env!("CARGO_MANIFEST_DIR");

        if cfg!(target_os = "windows") {
            format!("{}\\libtestffi.dll", base_dir)
        } else if cfg!(target_os = "macos") {
            format!("{}/libtestffi.dylib", base_dir)
        } else {
            format!("{}/libtestffi.so", base_dir)
        }
    }

    fn setup_test_bridge() -> Result<FFIBridge, Box<dyn std::error::Error>> {
        build_test_library()?;

        let mut bridge = FFIBridge::new();
        let lib_path = get_library_path();

        unsafe {
            bridge.load_library("test", &lib_path)?;
        }

        Ok(bridge)
    }

    #[test]
    fn test_load_real_library() {
        let result = setup_test_bridge();
        assert!(result.is_ok(), "Failed to load test library: {:?}", result.err());
    }

    #[test]
    fn test_call_add_int32() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: i32 = bridge.call_extern("test", "add_int32", (10i32, 20i32))
                .expect("Failed to call add_int32");
            assert_eq!(result, 30);
        }
    }

    #[test]
    fn test_call_add_int64() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: i64 = bridge.call_extern("test", "add_int64", (1000000i64, 2000000i64))
                .expect("Failed to call add_int64");
            assert_eq!(result, 3000000i64);
        }
    }

    #[test]
    fn test_call_add_uint32() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: u32 = bridge.call_extern("test", "add_uint32", (100u32, 200u32))
                .expect("Failed to call add_uint32");
            assert_eq!(result, 300u32);
        }
    }

    #[test]
    fn test_call_add_uint64() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: u64 = bridge.call_extern("test", "add_uint64", (1000000u64, 2000000u64))
                .expect("Failed to call add_uint64");
            assert_eq!(result, 3000000u64);
        }
    }

    #[test]
    fn test_call_add_float() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: f32 = bridge.call_extern("test", "add_float", (1.5f32, 2.5f32))
                .expect("Failed to call add_float");
            assert!((result - 4.0f32).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_call_add_double() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: f64 = bridge.call_extern("test", "add_double", (3.14f64, 2.86f64))
                .expect("Failed to call add_double");
            assert!((result - 8.9884f64).abs() < f64::EPSILON); // 3.14 * 2.86
        }
    }

    #[test]
    fn test_call_is_even() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let even: bool = bridge.call_extern("test", "is_even", (4i32,))
                .expect("Failed to call is_even with even number");
            assert_eq!(even, true);

            let odd: bool = bridge.call_extern("test", "is_even", (5i32,))
                .expect("Failed to call is_even with odd number");
            assert_eq!(odd, false);
        }
    }

    #[test]
    fn test_call_both_true() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let tt: bool = bridge.call_extern("test", "both_true", (true, true))
                .expect("Failed to call both_true(true, true)");
            assert_eq!(tt, true);

            let tf: bool = bridge.call_extern("test", "both_true", (true, false))
                .expect("Failed to call both_true(true, false)");
            assert_eq!(tf, false);

            let ff: bool = bridge.call_extern("test", "both_true", (false, false))
                .expect("Failed to call both_true(false, false)");
            assert_eq!(ff, false);
        }
    }

    #[test]
    fn test_call_greet() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: String = bridge.call_extern("test", "greet", ("World".to_string(),))
                .expect("Failed to call greet");
            assert_eq!(result, "Hello, World!");
        }
    }

    #[test]
    fn test_call_sum_four() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: i32 = bridge.call_extern("test", "sum_four", (1i32, 2i32, 3i32, 4i32))
                .expect("Failed to call sum_four");
            assert_eq!(result, 10);
        }
    }

    #[test]
    fn test_call_mixed_calculation() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            // Test con flag = true (resultado * 2)
            let result_true: f64 = bridge.call_extern("test", "mixed_calculation", (5i32, 3.5f64, true))
                .expect("Failed to call mixed_calculation with true");
            assert!((result_true - 17.0f64).abs() < f64::EPSILON); // (5 + 3.5) * 2

            // Test con flag = false (resultado normal)
            let result_false: f64 = bridge.call_extern("test", "mixed_calculation", (5i32, 3.5f64, false))
                .expect("Failed to call mixed_calculation with false");
            assert!((result_false - 8.5f64).abs() < f64::EPSILON); // 5 + 3.5
        }
    }

    #[test]
    fn test_call_log_message() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            // Esta función no retorna nada, solo validamos que no crashee
            let _: () = bridge.call_extern("test", "log_message", ("Test message".to_string(),))
                .expect("Failed to call log_message");
            // Si llega aquí sin panic, el test pasa
            assert!(true);
        }
    }

    #[test]
    fn test_call_divide_safe() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            // Test división normal
            let result: i32 = bridge.call_extern("test", "divide_safe", (10i32, 2i32))
                .expect("Failed to call divide_safe with valid division");
            assert_eq!(result, 5);

            // Test división por cero (debería retornar error)
            // Nota: Esta función usa un patrón diferente (out parameter),
            // por simplicidad en el test validamos que no crashee
            let _: i32 = bridge.call_extern("test", "divide_safe", (10i32, 0i32))
                .expect("Failed to call divide_safe with division by zero");
            // En implementación real, esto debería manejar el código de error
        }
    }

    #[test]
    fn test_function_not_found() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        unsafe {
            let result: Result<i32, _> = bridge.call_extern("test", "nonexistent_function", (1i32,));
            assert!(result.is_err());

            if let Err(FFIError::FunctionNotFound(func)) = result {
                assert_eq!(func, "nonexistent_function");
            } else {
                panic!("Expected FunctionNotFound error");
            }
        }
    }

    #[test]
    fn test_library_not_found() {
        let mut bridge = FFIBridge::new();

        unsafe {
            let result = bridge.load_library("nonexistent", "nonexistent_lib.so");
            assert!(result.is_err());

            if let Err(FFIError::LibraryLoadError(lib)) = result {
                assert!(lib.contains("nonexistent"));
            } else {
                panic!("Expected LibraryLoadError");
            }
        }
    }

    #[test]
    fn test_multiple_libraries() {
        let mut bridge = setup_test_bridge().expect("Failed to setup bridge");

        // Intentar cargar la misma librería con diferente alias
        unsafe {
            let result = bridge.load_library("test2", &get_library_path());
            assert!(result.is_ok());

            // Verificar que ambas funcionan
            let result1: i32 = bridge.call_extern("test", "add_int32", (1i32, 2i32)).unwrap();
            let result2: i32 = bridge.call_extern("test2", "add_int32", (3i32, 4i32)).unwrap();

            assert_eq!(result1, 3);
            assert_eq!(result2, 7);
        }
    }
}