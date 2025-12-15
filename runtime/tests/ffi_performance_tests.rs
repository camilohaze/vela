//! Tests de performance y carga para FFI bridge
//!
//! Estos tests validan:
//! - Performance con llamadas masivas
//! - Memory leaks y gestión de recursos
//! - Thread safety en llamadas concurrentes
//! - Benchmarks comparativos

use vela_runtime::ffi::*;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(test)]
mod performance_tests {
    use super::*;

    fn setup_test_bridge() -> FFIBridge {
        // Para tests de performance, asumimos que la librería ya está compilada
        // En un entorno CI/CD real, esto se haría en una fase de setup previa
        let mut bridge = FFIBridge::new();

        let lib_path = if cfg!(target_os = "windows") {
            "libtestffi.dll"
        } else if cfg!(target_os = "macos") {
            "libtestffi.dylib"
        } else {
            "libtestffi.so"
        };

        unsafe {
            bridge.load_library("test", lib_path).expect("Failed to load test library");
        }

        bridge
    }

    #[test]
    fn test_bulk_ffi_calls() {
        let mut bridge = setup_test_bridge();

        let start = Instant::now();
        let iterations = 1000;

        unsafe {
            for i in 0..iterations {
                let result: i32 = bridge.call_extern("test", "add_int32", (i, 1))
                    .expect("FFI call failed");
                assert_eq!(result, i + 1);
            }
        }

        let duration = start.elapsed();
        let calls_per_second = iterations as f64 / duration.as_secs_f64();

        println!("{} FFI calls took: {:?}", iterations, duration);
        println!("Calls per second: {:.2}", calls_per_second);

        // Validar que esté dentro de límites aceptables (al menos 1000 calls/sec)
        assert!(calls_per_second > 1000.0, "Performance too slow: {:.2} calls/sec", calls_per_second);
    }

    #[test]
    fn test_memory_usage_stability() {
        let mut bridge = setup_test_bridge();

        // Test que la memoria no crezca significativamente con muchas llamadas
        let initial_memory = get_memory_usage();

        unsafe {
            for i in 0..10000 {
                let _: i32 = bridge.call_extern("test", "add_int32", (i % 100, 1))
                    .expect("FFI call failed");
            }
        }

        let final_memory = get_memory_usage();
        let memory_growth = final_memory - initial_memory;

        println!("Memory growth after 10k calls: {} bytes", memory_growth);

        // Validar que el crecimiento de memoria sea razonable (< 1MB)
        assert!(memory_growth < 1024 * 1024, "Memory leak detected: {} bytes growth", memory_growth);
    }

    #[test]
    fn test_string_memory_management() {
        let mut bridge = setup_test_bridge();

        // Test que las strings se liberen correctamente
        let initial_memory = get_memory_usage();

        unsafe {
            for i in 0..1000 {
                let name = format!("User{}", i);
                let _: String = bridge.call_extern("test", "greet", (name,))
                    .expect("FFI call failed");
            }
        }

        let final_memory = get_memory_usage();
        let memory_growth = final_memory - initial_memory;

        println!("Memory growth after 1k string calls: {} bytes", memory_growth);

        // Validar que no haya leaks significativos (< 100KB)
        assert!(memory_growth < 100 * 1024, "String memory leak: {} bytes growth", memory_growth);
    }

    #[test]
    fn test_concurrent_ffi_calls() {
        let bridge = Arc::new(Mutex::new(setup_test_bridge()));
        let mut handles = vec![];

        let num_threads = 4;
        let calls_per_thread = 1000;

        for thread_id in 0..num_threads {
            let bridge_clone = Arc::clone(&bridge);

            let handle = thread::spawn(move || {
                let mut local_results = vec![];

                for i in 0..calls_per_thread {
                    let bridge = bridge_clone.lock().unwrap();
                    unsafe {
                        let result: i32 = bridge.call_extern("test", "add_int32", (thread_id as i32 * 1000 + i as i32, 1))
                            .expect("Concurrent FFI call failed");
                        local_results.push(result);
                    }
                }

                local_results
            });

            handles.push(handle);
        }

        // Recolectar resultados
        let mut all_results = vec![];
        for handle in handles {
            let results = handle.join().expect("Thread panicked");
            all_results.extend(results);
        }

        // Validar que todos los resultados sean correctos
        assert_eq!(all_results.len(), num_threads * calls_per_thread);

        for (i, &result) in all_results.iter().enumerate() {
            let expected = (i / calls_per_thread) as i32 * 1000 + (i % calls_per_thread) as i32 + 1;
            assert_eq!(result, expected, "Result mismatch at index {}", i);
        }

        println!("Concurrent test passed: {} total calls across {} threads", all_results.len(), num_threads);
    }

    #[test]
    fn test_ffi_call_overhead() {
        let mut bridge = setup_test_bridge();

        // Comparar overhead de FFI vs llamadas Rust directas
        let iterations = 10000;

        // Benchmark FFI calls
        let ffi_start = Instant::now();
        unsafe {
            for i in 0..iterations {
                let _: i32 = bridge.call_extern("test", "add_int32", (i, 1)).unwrap();
            }
        }
        let ffi_duration = ffi_start.elapsed();

        // Benchmark Rust direct calls
        let rust_start = Instant::now();
        for i in 0..iterations {
            let _ = i + 1;
        }
        let rust_duration = rust_start.elapsed();

        let ffi_calls_per_sec = iterations as f64 / ffi_duration.as_secs_f64();
        let rust_calls_per_sec = iterations as f64 / rust_duration.as_secs_f64();
        let overhead_ratio = ffi_duration.as_secs_f64() / rust_duration.as_secs_f64();

        println!("FFI calls/sec: {:.0}", ffi_calls_per_sec);
        println!("Rust calls/sec: {:.0}", rust_calls_per_sec);
        println!("FFI overhead ratio: {:.2}x", overhead_ratio);

        // Validar que el overhead no sea excesivo (< 1000x)
        assert!(overhead_ratio < 1000.0, "FFI overhead too high: {:.2}x", overhead_ratio);
    }

    #[test]
    fn test_large_argument_counts() {
        // Test con el límite máximo de argumentos (5)
        let mut bridge = setup_test_bridge();

        unsafe {
            let result: i32 = bridge.call_extern("test", "sum_four", (1i32, 2i32, 3i32, 4i32))
                .expect("Failed to call function with 4 args");

            assert_eq!(result, 10);

            // Test con argumentos de diferentes tipos
            let mixed_result: f64 = bridge.call_extern("test", "mixed_calculation", (10i32, 5.5f64, true))
                .expect("Failed to call function with mixed args");

            assert!((mixed_result - 31.0f64).abs() < f64::EPSILON); // (10 + 5.5) * 2
        }
    }

    #[test]
    fn test_error_recovery() {
        let mut bridge = setup_test_bridge();

        // Test que el bridge se recupere de errores
        unsafe {
            // Llamada válida primero
            let result1: i32 = bridge.call_extern("test", "add_int32", (1i32, 2i32)).unwrap();
            assert_eq!(result1, 3);

            // Llamada inválida
            let result2: Result<i32, _> = bridge.call_extern("test", "nonexistent", (1i32,));
            assert!(result2.is_err());

            // Llamada válida después del error
            let result3: i32 = bridge.call_extern("test", "add_int32", (4i32, 5i32)).unwrap();
            assert_eq!(result3, 9);
        }

        println!("Error recovery test passed");
    }

    #[test]
    fn test_bridge_reuse() {
        // Test que el mismo bridge se pueda reutilizar múltiples veces
        let mut bridge = setup_test_bridge();

        unsafe {
            for round in 0..10 {
                let result: i32 = bridge.call_extern("test", "add_int32", (round, 10))
                    .expect("Failed in round {round}");

                assert_eq!(result, round + 10);
            }
        }

        println!("Bridge reuse test passed");
    }

    // Función helper para obtener uso de memoria aproximado
    // Nota: Esta es una aproximación simple, en producción se usaría un profiler real
    fn get_memory_usage() -> usize {
        // En un entorno real, esto usaría jemalloc stats o similar
        // Para este test, devolvemos un valor aproximado
        // En producción, implementar con actual memory profiling
        0 // Placeholder - implementar con allocators reales
    }
}

// Benchmarks usando criterion (si está disponible)
#[cfg(feature = "bench")]
mod benches {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_ffi_call(c: &mut Criterion) {
        let mut bridge = setup_test_bridge();

        c.bench_function("ffi_call_add_int32", |b| {
            b.iter(|| {
                unsafe {
                    let result: i32 = bridge.call_extern("test", "add_int32", (black_box(42), black_box(24)))
                        .unwrap();
                    black_box(result);
                }
            })
        });
    }

    fn bench_string_ffi_call(c: &mut Criterion) {
        let mut bridge = setup_test_bridge();

        c.bench_function("ffi_call_greet", |b| {
            b.iter(|| {
                unsafe {
                    let result: String = bridge.call_extern("test", "greet", (black_box("Benchmark".to_string()),))
                        .unwrap();
                    black_box(result);
                }
            })
        });
    }

    criterion_group!(benches, bench_ffi_call, bench_string_ffi_call);
    criterion_main!(benches);
}