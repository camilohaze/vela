//! Tests unitarios para FFI bridge
//!
//! Estos tests validan la funcionalidad básica del puente FFI:
//! - Conversión de tipos individuales
//! - Validación de traits FFIType/FFIArgs
//! - Manejo básico de errores
//! - Creación y configuración del bridge

use vela_runtime::ffi::*;
use std::ffi::CString;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_bool_conversion() {
        // Test conversión bool -> C
        let c_val = true.to_c_value();
        assert!(!c_val.is_null());

        // Test conversión C -> bool
        let back = bool::from_c_value(c_val);
        assert_eq!(back, true);

        // Test false
        let c_false = false.to_c_value();
        let back_false = bool::from_c_value(c_false);
        assert_eq!(back_false, false);
    }

    #[test]
    fn test_i32_conversion() {
        let val = 42i32;

        // Test conversión i32 -> C
        let c_val = val.to_c_value();
        assert!(!c_val.is_null());

        // Test conversión C -> i32
        let back = i32::from_c_value(c_val);
        assert_eq!(back, val);

        // Test valores negativos
        let neg_val = -123i32;
        let c_neg = neg_val.to_c_value();
        let back_neg = i32::from_c_value(c_neg);
        assert_eq!(back_neg, neg_val);
    }

    #[test]
    fn test_i64_conversion() {
        let val = 9223372036854775807i64; // i64::MAX

        let c_val = val.to_c_value();
        let back = i64::from_c_value(c_val);
        assert_eq!(back, val);

        let neg_val = -9223372036854775808i64; // i64::MIN
        let c_neg = neg_val.to_c_value();
        let back_neg = i64::from_c_value(c_neg);
        assert_eq!(back_neg, neg_val);
    }

    #[test]
    fn test_u32_conversion() {
        let val = 4294967295u32; // u32::MAX

        let c_val = val.to_c_value();
        let back = u32::from_c_value(c_val);
        assert_eq!(back, val);
    }

    #[test]
    fn test_u64_conversion() {
        let val = 18446744073709551615u64; // u64::MAX

        let c_val = val.to_c_value();
        let back = u64::from_c_value(c_val);
        assert_eq!(back, val);
    }

    #[test]
    fn test_f32_conversion() {
        let val = 3.14159f32;

        let c_val = val.to_c_value();
        let back = f32::from_c_value(c_val);
        assert!((back - val).abs() < f32::EPSILON);
    }

    #[test]
    fn test_f64_conversion() {
        let val = 3.141592653589793f64;

        let c_val = val.to_c_value();
        let back = f64::from_c_value(c_val);
        assert!((back - val).abs() < f64::EPSILON);
    }

    #[test]
    fn test_string_conversion() {
        let val = "Hello FFI World!".to_string();

        // Test conversión String -> C
        let c_val = val.to_c_value();
        assert!(!c_val.is_null());

        // Test conversión C -> String
        let back = String::from_c_value(c_val);
        assert_eq!(back, val);

        // Test string vacío
        let empty = String::new();
        let c_empty = empty.to_c_value();
        let back_empty = String::from_c_value(c_empty);
        assert_eq!(back_empty, empty);
    }

    #[test]
    fn test_ffi_args_single_values() {
        // Test tupla de un elemento
        let args = (42i32,);
        let c_args = args.to_c_args();
        let types = args.ffi_types();

        assert_eq!(c_args.len(), 1);
        assert_eq!(types.len(), 1);

        // Verificar que el valor se convirtió correctamente
        let back = i32::from_c_value(c_args[0]);
        assert_eq!(back, 42i32);
    }

    #[test]
    fn test_ffi_args_multiple_values() {
        // Test tupla de múltiples elementos
        let args = (1i32, 2.5f64, true);
        let c_args = args.to_c_args();
        let types = args.ffi_types();

        assert_eq!(c_args.len(), 3);
        assert_eq!(types.len(), 3);

        // Verificar conversiones
        let back_i32 = i32::from_c_value(c_args[0]);
        let back_f64 = f64::from_c_value(c_args[1]);
        let back_bool = bool::from_c_value(c_args[2]);

        assert_eq!(back_i32, 1i32);
        assert!((back_f64 - 2.5f64).abs() < f64::EPSILON);
        assert_eq!(back_bool, true);
    }

    #[test]
    fn test_ffi_args_max_supported() {
        // Test límite máximo de argumentos (5)
        let args = (1i32, 2i32, 3i32, 4i32, 5i32);
        let c_args = args.to_c_args();
        let types = args.ffi_types();

        assert_eq!(c_args.len(), 5);
        assert_eq!(types.len(), 5);

        for i in 0..5 {
            let back = i32::from_c_value(c_args[i]);
            assert_eq!(back, (i + 1) as i32);
        }
    }

    #[test]
    fn test_ffi_bridge_creation() {
        let bridge = FFIBridge::new();
        // Bridge debería crearse sin errores
        assert!(true); // Si llega aquí, la creación fue exitosa
    }

    #[test]
    fn test_ffi_error_types() {
        // Test que los tipos de error existen y se pueden crear
        let lib_error = FFIError::LibraryLoadError("test".to_string());
        let symbol_error = FFIError::SymbolNotFound("test_func".to_string());
        let call_error = FFIError::CallError("call failed".to_string());
        let type_error = FFIError::TypeConversionError("bad type".to_string());
        let mem_error = FFIError::MemoryError("memory error".to_string());

        // Verificar que implementan Display
        assert!(!lib_error.to_string().is_empty());
        assert!(!symbol_error.to_string().is_empty());
        assert!(!call_error.to_string().is_empty());
        assert!(!type_error.to_string().is_empty());
        assert!(!mem_error.to_string().is_empty());
    }

    #[test]
    fn test_ffi_library_creation() {
        // FFILibrary no se puede crear con new(), se carga desde un path
        // Este test verifica que el método load() existe y maneja errores correctamente
        let result = FFILibrary::load("nonexistent_library.so");
        assert!(result.is_err()); // Debería fallar con librería inexistente
        // Verificar que es un error de carga de librería
        match result {
            Err(FFIError::LibraryLoadError(_)) => {} // OK
            _ => panic!("Expected LibraryLoadError"),
        }
    }

    #[test]
    fn test_ffi_type_descriptors() {
        // Test que los descriptores de tipo libffi se crean correctamente
        let i32_type = i32::ffi_type();
        let f64_type = f64::ffi_type();
        let bool_type = bool::ffi_type();

        // Los descriptores no deberían ser null
        assert!(!i32_type.is_null());
        assert!(!f64_type.is_null());
        assert!(!bool_type.is_null());
    }

    #[test]
    fn test_string_edge_cases() {
        // Test strings con caracteres especiales
        let special = "Hello\n\tWorld\r\n!".to_string();
        let c_val = special.to_c_value();
        let back = String::from_c_value(c_val);
        assert_eq!(back, special);

        // Test string muy largo
        let long_string = "A".repeat(10000);
        let c_long = long_string.to_c_value();
        let back_long = String::from_c_value(c_long);
        assert_eq!(back_long, long_string);
    }

    #[test]
    fn test_numeric_edge_cases() {
        // Test valores límite
        let max_i32 = i32::MAX;
        let c_max = max_i32.to_c_value();
        let back_max = i32::from_c_value(c_max);
        assert_eq!(back_max, max_i32);

        let min_i32 = i32::MIN;
        let c_min = min_i32.to_c_value();
        let back_min = i32::from_c_value(c_min);
        assert_eq!(back_min, min_i32);

        // Test cero
        let zero = 0i32;
        let c_zero = zero.to_c_value();
        let back_zero = i32::from_c_value(c_zero);
        assert_eq!(back_zero, zero);
    }
}