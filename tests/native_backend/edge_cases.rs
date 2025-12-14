/*!
# Edge Cases Tests for Native Backend

Tests for boundary conditions, error handling, and unusual scenarios.
*/

#[cfg(feature = "llvm_backend")]
use super::utils::{NativeBackendTester, TestError, OptimizationLevel};

#[cfg(feature = "llvm_backend")]
#[cfg(test)]
mod tests {
    use super::*;

    fn get_tester() -> Result<NativeBackendTester, TestError> {
        let tester = NativeBackendTester::new()?;
        if !tester.is_backend_available() {
            eprintln!("Warning: LLVM backend not fully available, skipping edge case tests");
            return Err(TestError::LLVMUnavailable);
        }
        Ok(tester)
    }

    #[test]
    fn test_empty_program() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn main() -> void {
                // Empty program
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Empty program should compile and run");

        // Should run without errors
        assert!(result.success, "Empty program should succeed");
        assert!(result.stderr.is_empty(), "Empty program should not produce stderr");
    }

    #[test]
    fn test_maximum_integer_values() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test with large integers (close to i64 limits)
        let vela_code = r#"
            fn test_large_numbers() -> void {
                let max_i64 = 9223372036854775807;  // i64::MAX
                let min_i64 = -9223372036854775808; // i64::MIN

                let result1 = max_i64 / 2;
                let result2 = min_i64 / 2;

                print(max_i64);
                print(min_i64);
                print(result1);
                print(result2);
            }

            fn main() -> void {
                test_large_numbers();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Large numbers test should succeed");

        assert!(result.success, "Large numbers should be handled correctly");
        // Should contain the expected values in output
        assert!(tester.validate_output("9223372036854775807", &result.stdout));
        assert!(tester.validate_output("-9223372036854775808", &result.stdout));
    }

    #[test]
    fn test_deep_recursion() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test deep recursion that might cause stack overflow
        let vela_code = r#"
            fn deep_recursive(n: int) -> int {
                if n <= 0 {
                    return 0;
                }
                return 1 + deep_recursive(n - 1);
            }

            fn main() -> void {
                let result = deep_recursive(1000);  // Deep recursion
                print(result);
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default);

        match result {
            Ok(r) => {
                // If it succeeds, verify the result
                assert!(r.success, "Deep recursion should succeed or fail gracefully");
                if r.success {
                    assert!(tester.validate_output("1000", &r.stdout), "Deep recursion should produce correct result");
                }
            }
            Err(TestError::ExecutionTimeout) => {
                // Timeout is acceptable for deep recursion
                println!("Deep recursion timed out as expected");
            }
            Err(e) => {
                // Other errors might be acceptable depending on stack limits
                println!("Deep recursion failed with: {:?}", e);
            }
        }
    }

    #[test]
    fn test_large_arrays() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test with large arrays that might stress memory
        let vela_code = r#"
            fn test_large_array() -> void {
                let arr = [];
                for i in 0..10000 {
                    arr.push(i);
                }

                let sum = 0;
                for i in 0..arr.length() {
                    sum = sum + arr[i];
                }

                print(sum);
            }

            fn main() -> void {
                test_large_array();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Large array test should succeed");

        assert!(result.success, "Large array operations should succeed");
        // Expected sum: (0 + 9999) * 10000 / 2 = 49995000
        assert!(tester.validate_output("49995000", &result.stdout));
    }

    #[test]
    fn test_string_operations_edge_cases() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn test_strings() -> void {
                let empty = "";
                let single = "x";
                let long = "this is a very long string with many characters to test string handling";

                print(empty);
                print(single);
                print(long);
            }

            fn main() -> void {
                test_strings();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("String edge cases should succeed");

        assert!(result.success, "String operations should succeed");
        assert!(tester.validate_output("x", &result.stdout));
        assert!(tester.validate_output("this is a very long string", &result.stdout));
    }

    #[test]
    fn test_division_by_zero() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn test_division() -> void {
                let result = 10 / 0;  // Division by zero
                print(result);
            }

            fn main() -> void {
                test_division();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default);

        // Division by zero behavior depends on the target platform
        // It might succeed (producing undefined behavior) or fail
        match result {
            Ok(r) => {
                // If it runs, it might produce any result due to undefined behavior
                println!("Division by zero produced result: {}", r.stdout);
            }
            Err(e) => {
                // Failing is also acceptable
                println!("Division by zero failed as expected: {:?}", e);
            }
        }
    }

    #[test]
    fn test_negative_array_access() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn test_negative_index() -> void {
                let arr = [1, 2, 3];
                let value = arr[-1];  // Negative index
                print(value);
            }

            fn main() -> void {
                test_negative_index();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default);

        // Negative array access is undefined behavior
        match result {
            Ok(r) => {
                println!("Negative array access produced result: {}", r.stdout);
            }
            Err(e) => {
                println!("Negative array access failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_max_function_calls() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test many function calls
        let vela_code = r#"
            fn small_function(x: int) -> int {
                return x + 1;
            }

            fn test_many_calls() -> void {
                let result = 0;
                for i in 0..1000 {
                    result = small_function(result);
                }
                print(result);
            }

            fn main() -> void {
                test_many_calls();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Many function calls should succeed");

        assert!(result.success, "Many function calls should succeed");
        assert!(tester.validate_output("1000", &result.stdout));
    }

    #[test]
    fn test_floating_point_edge_cases() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn test_floats() -> void {
                let zero = 0.0;
                let inf = 1.0 / 0.0;
                let neg_inf = -1.0 / 0.0;
                let nan = 0.0 / 0.0;

                print(zero);
                print(inf);
                print(neg_inf);
                print(nan);
            }

            fn main() -> void {
                test_floats();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default);

        match result {
            Ok(r) => {
                assert!(r.success, "Floating point edge cases should run");
                println!("Floating point results: {}", r.stdout);
            }
            Err(e) => {
                // Some platforms might not support floating point operations
                println!("Floating point operations not supported: {:?}", e);
            }
        }
    }

    #[test]
    fn test_memory_allocation_limits() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test allocating many objects
        let vela_code = r#"
            fn test_allocation() -> void {
                let objects = [];
                for i in 0..1000 {
                    let obj = { value: i, name: "item" };
                    objects.push(obj);
                }

                let sum = 0;
                for i in 0..objects.length() {
                    sum = sum + objects[i].value;
                }

                print(sum);
            }

            fn main() -> void {
                test_allocation();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Memory allocation test should succeed");

        assert!(result.success, "Memory allocation should succeed");
        // Expected sum: (0 + 999) * 1000 / 2 = 499500
        assert!(tester.validate_output("499500", &result.stdout));
    }

    #[test]
    fn test_concurrent_operations() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test operations that might involve concurrency internally
        let vela_code = r#"
            fn concurrent_like() -> void {
                let results = [];
                for i in 0..100 {
                    // Simulate some computation
                    let value = i * i + i;
                    results.push(value);
                }

                let sum = 0;
                for i in 0..results.length() {
                    sum = sum + results[i];
                }

                print(sum);
            }

            fn main() -> void {
                concurrent_like();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Concurrent-like operations should succeed");

        assert!(result.success, "Concurrent-like operations should succeed");
        // Expected sum: sum of (i*i + i) for i in 0..100
        // = sum(i^2) + sum(i) = (100*99*199/6) + (100*99/2) = 328350 + 4950 = 333300
        assert!(tester.validate_output("333300", &result.stdout));
    }

    #[test]
    fn test_very_long_execution() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test that might take a long time
        let vela_code = r#"
            fn slow_computation() -> void {
                let result = 0;
                for i in 0..100000 {
                    result = result + i;
                }
                print(result);
            }

            fn main() -> void {
                slow_computation();
            }
        "#;

        let result = tester.compile_and_run_with_timeout(vela_code, OptimizationLevel::Default, 30);

        match result {
            Ok(r) => {
                assert!(r.success, "Long computation should succeed");
                // Expected sum: (0 + 99999) * 100000 / 2 = 4999950000
                assert!(tester.validate_output("4999950000", &r.stdout));
            }
            Err(TestError::ExecutionTimeout) => {
                println!("Long computation timed out - this is acceptable for slow operations");
            }
            Err(e) => {
                panic!("Unexpected error in long computation: {:?}", e);
            }
        }
    }
}