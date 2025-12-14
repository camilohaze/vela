/*!
# Correctness Tests for Native Backend

Tests to validate that the native backend generates correct executable code.
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
            eprintln!("Warning: LLVM backend not fully available, skipping tests");
            return Err(TestError::LLVMUnavailable);
        }
        Ok(tester)
    }

    #[test]
    fn test_arithmetic_operations() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return, // Skip test
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test basic arithmetic
        let vela_code = r#"
            fn main() -> void {
                let result = 10 + 5;
                print(result);
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Compilation and execution should succeed");

        assert_eq!(result.exit_code, 0);
        assert!(tester.validate_output("15", &result.stdout));
    }

    #[test]
    fn test_complex_arithmetic() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn main() -> void {
                let a = 10;
                let b = 5;
                let c = 3;
                let result = (a + b) * c - 7;
                print(result);
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Complex arithmetic should work");

        assert_eq!(result.exit_code, 0);
        assert!(tester.validate_output("38", &result.stdout));
    }

    #[test]
    fn test_float_operations() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn main() -> void {
                let a = 3.14;
                let b = 2.0;
                let result = a * b + 1.0;
                print(result);
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Float operations should work");

        assert_eq!(result.exit_code, 0);
        // Allow small floating point precision differences
        let output = result.stdout.trim();
        let value: f64 = output.parse().expect("Should parse as float");
        assert!((value - 7.28).abs() < 0.01, "Expected ~7.28, got {}", value);
    }

    #[test]
    fn test_boolean_operations() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn main() -> void {
                let a = true;
                let b = false;
                let result = a && !b;
                if result {
                    print("true");
                } else {
                    print("false");
                }
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Boolean operations should work");

        assert_eq!(result.exit_code, 0);
        assert!(tester.validate_output("true", &result.stdout));
    }

    #[test]
    fn test_if_else_control_flow() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn main() -> void {
                let x = 10;
                if x > 5 {
                    print("greater");
                } else {
                    print("smaller");
                }
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("If-else should work");

        assert_eq!(result.exit_code, 0);
        assert!(tester.validate_output("greater", &result.stdout));
    }

    #[test]
    fn test_function_calls() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn add(a: int, b: int) -> int {
                return a + b;
            }

            fn main() -> void {
                let result = add(5, 3);
                print(result);
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Function calls should work");

        assert_eq!(result.exit_code, 0);
        assert!(tester.validate_output("8", &result.stdout));
    }

    #[test]
    fn test_recursion() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn factorial(n: int) -> int {
                if n <= 1 {
                    return 1;
                }
                return n * factorial(n - 1);
            }

            fn main() -> void {
                let result = factorial(5);
                print(result);
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Recursion should work");

        assert_eq!(result.exit_code, 0);
        assert!(tester.validate_output("120", &result.stdout));
    }

    #[test]
    fn test_string_operations() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn main() -> void {
                let greeting = "Hello";
                let name = "World";
                print(greeting);
                print(" ");
                print(name);
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("String operations should work");

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Hello"));
        assert!(result.stdout.contains("World"));
    }

    #[test]
    fn test_variable_scoping() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn main() -> void {
                let x = 5;
                {
                    let x = 10;
                    print(x);  // Should print 10
                }
                print(x);  // Should print 5
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Variable scoping should work");

        assert_eq!(result.exit_code, 0);
        let output = result.stdout.trim();
        assert!(output.contains("10") && output.contains("5"));
    }

    #[test]
    fn test_comparison_operators() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        let vela_code = r#"
            fn main() -> void {
                let a = 10;
                let b = 5;

                if a > b {
                    print("a_gt_b");
                }
                if a >= b {
                    print("a_ge_b");
                }
                if b < a {
                    print("b_lt_a");
                }
                if b <= a {
                    print("b_le_a");
                }
                if a == a {
                    print("a_eq_a");
                }
                if a != b {
                    print("a_ne_b");
                }
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Comparison operators should work");

        assert_eq!(result.exit_code, 0);
        let output = result.stdout.trim();
        assert!(output.contains("a_gt_b"));
        assert!(output.contains("a_ge_b"));
        assert!(output.contains("b_lt_a"));
        assert!(output.contains("b_le_a"));
        assert!(output.contains("a_eq_a"));
        assert!(output.contains("a_ne_b"));
    }
}