/*!
# Performance Tests for Native Backend

Benchmarks to validate that optimization levels improve performance as expected.
*/

#[cfg(feature = "llvm_backend")]
use super::utils::{NativeBackendTester, TestError, OptimizationLevel, BenchmarkResult};

#[cfg(feature = "llvm_backend")]
#[cfg(test)]
mod tests {
    use super::*;

    fn get_tester() -> Result<NativeBackendTester, TestError> {
        let tester = NativeBackendTester::new()?;
        if !tester.is_backend_available() {
            eprintln!("Warning: LLVM backend not fully available, skipping performance tests");
            return Err(TestError::LLVMUnavailable);
        }
        Ok(tester)
    }

    #[test]
    fn test_optimization_speedup_basic() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Simple compute-bound code
        let vela_code = r#"
            fn compute(n: int) -> int {
                let result = 0;
                for i in 0..n {
                    result = result + i * i;
                }
                return result;
            }

            fn main() -> void {
                let result = compute(1000);
                print(result);
            }
        "#;

        let benchmark = tester.benchmark_code(vela_code, 3)
            .expect("Benchmark should succeed");

        // Basic checks: optimized versions should be faster than O0
        assert!(benchmark.o1_time <= benchmark.o0_time,
               "O1 should be faster than O0: {:?} vs {:?}", benchmark.o1_time, benchmark.o0_time);
        assert!(benchmark.o2_time <= benchmark.o1_time,
               "O2 should be faster than O1: {:?} vs {:?}", benchmark.o2_time, benchmark.o1_time);

        // Speedup should be reasonable (at least 1.1x for basic optimizations)
        assert!(benchmark.speedup_o1 >= 1.0, "O1 speedup should be >= 1.0, got {}", benchmark.speedup_o1);
        assert!(benchmark.speedup_o2 >= benchmark.speedup_o1,
               "O2 speedup should be >= O1 speedup: {} vs {}", benchmark.speedup_o2, benchmark.speedup_o1);
    }

    #[test]
    fn test_fibonacci_performance() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Fibonacci is good for testing optimization effectiveness
        let vela_code = r#"
            fn fibonacci(n: int) -> int {
                if n <= 1 {
                    return n;
                }
                return fibonacci(n - 1) + fibonacci(n - 2);
            }

            fn main() -> void {
                let result = fibonacci(30);  // Large enough to see optimization effects
                print(result);
            }
        "#;

        let benchmark = tester.benchmark_code(vela_code, 3)
            .expect("Fibonacci benchmark should succeed");

        // Verify correctness first
        let correctness_test = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Correctness test should succeed");
        assert!(tester.validate_output("832040", &correctness_test.stdout));

        // Performance checks
        assert!(benchmark.speedup_o2 > 1.5,
               "O2 should provide significant speedup for recursive code: {}", benchmark.speedup_o2);
        assert!(benchmark.speedup_o3 >= benchmark.speedup_o2,
               "O3 should be at least as fast as O2: {} vs {}", benchmark.speedup_o3, benchmark.speedup_o2);
    }

    #[test]
    fn test_memory_operations_performance() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test memory-intensive operations
        let vela_code = r#"
            fn memory_intensive(n: int) -> int {
                let arr = [];
                for i in 0..n {
                    arr.push(i);
                }

                let sum = 0;
                for i in 0..arr.length() {
                    sum = sum + arr[i];
                }

                return sum;
            }

            fn main() -> void {
                let result = memory_intensive(1000);
                print(result);
            }
        "#;

        let benchmark = tester.benchmark_code(vela_code, 3)
            .expect("Memory benchmark should succeed");

        // Memory operations should benefit from optimizations
        assert!(benchmark.speedup_o1 >= 1.0, "Should see some benefit from basic optimizations");
        assert!(benchmark.speedup_o2 >= benchmark.speedup_o1,
               "O2 should improve on O1 for memory operations");
    }

    #[test]
    fn test_compilation_time_vs_optimization() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Complex code that benefits from optimization
        let vela_code = r#"
            fn complex_computation(n: int) -> int {
                let result = 0;
                for i in 0..n {
                    for j in 0..n {
                        if i * j % 2 == 0 {
                            result = result + i + j;
                        }
                    }
                }
                return result;
            }

            fn main() -> void {
                let result = complex_computation(50);
                print(result);
            }
        "#;

        let benchmark = tester.benchmark_code(vela_code, 3)
            .expect("Complex computation benchmark should succeed");

        // Higher optimization levels should provide better performance
        // but we expect diminishing returns
        assert!(benchmark.speedup_o3 >= benchmark.speedup_o2,
               "O3 should be at least as good as O2");

        // O3 should provide significant speedup for complex code
        assert!(benchmark.speedup_o3 > 2.0,
               "O3 should provide >2x speedup for complex nested loops: {}", benchmark.speedup_o3);
    }

    #[test]
    fn test_optimization_consistency() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test that optimization results are consistent across runs
        let vela_code = r#"
            fn consistent_compute() -> int {
                let result = 0;
                for i in 0..100 {
                    result = result + i;
                }
                return result;
            }

            fn main() -> void {
                let result = consistent_compute();
                print(result);
            }
        "#;

        // Run benchmark multiple times to check consistency
        let benchmark1 = tester.benchmark_code(vela_code, 3).expect("First benchmark should succeed");
        let benchmark2 = tester.benchmark_code(vela_code, 3).expect("Second benchmark should succeed");

        // Results should be reasonably consistent (within 10% for O3)
        let time_diff = (benchmark1.o3_time.as_millis() as f64 - benchmark2.o3_time.as_millis() as f64).abs();
        let avg_time = (benchmark1.o3_time.as_millis() + benchmark2.o3_time.as_millis()) as f64 / 2.0;
        let variation = time_diff / avg_time;

        assert!(variation < 0.1, "Benchmark results should be consistent: variation {} > 10%", variation);
    }

    #[test]
    fn test_different_code_sizes() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test small code
        let small_code = r#"
            fn main() -> void {
                let result = 42;
                print(result);
            }
        "#;

        let small_benchmark = tester.benchmark_code(small_code, 2)
            .expect("Small code benchmark should succeed");

        // Test larger code
        let large_code = r#"
            fn compute_large(n: int) -> int {
                let result = 0;
                for i in 0..n {
                    for j in 0..10 {
                        result = result + i * j;
                    }
                }
                return result;
            }

            fn main() -> void {
                let result = compute_large(100);
                print(result);
            }
        "#;

        let large_benchmark = tester.benchmark_code(large_code, 2)
            .expect("Large code benchmark should succeed");

        // Larger code should benefit more from optimizations
        assert!(large_benchmark.speedup_o2 >= small_benchmark.speedup_o2,
               "Larger code should benefit more from optimizations: {} vs {}",
               large_benchmark.speedup_o2, small_benchmark.speedup_o2);
    }

    #[test]
    fn test_optimization_levels_ordering() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn ordered_test(n: int) -> int {
                let result = 0;
                for i in 0..n {
                    if i % 2 == 0 {
                        result = result + i;
                    } else {
                        result = result - i;
                    }
                }
                return result;
            }

            fn main() -> void {
                let result = ordered_test(1000);
                print(result);
            }
        "#;

        let benchmark = tester.benchmark_code(vela_code, 3)
            .expect("Ordering test should succeed");

        // Verify that optimization levels are ordered correctly
        // O0 should be slowest, O3 should be fastest
        assert!(benchmark.o0_time >= benchmark.o1_time,
               "O0 should be slower than O1: {:?} vs {:?}", benchmark.o0_time, benchmark.o1_time);
        assert!(benchmark.o1_time >= benchmark.o2_time,
               "O1 should be slower than O2: {:?} vs {:?}", benchmark.o1_time, benchmark.o2_time);
        assert!(benchmark.o2_time >= benchmark.o3_time,
               "O2 should be slower than O3: {:?} vs {:?}", benchmark.o2_time, benchmark.o3_time);
    }
}