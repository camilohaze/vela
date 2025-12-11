//! Example usage of the BDD testing framework

use test_decorators::*;

fn main() {
    println!("BDD Example compiled successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[describe("Calculator Operations")]
    mod calculator_tests {
        use super::*;

        #[describe("Addition")]
        mod addition_tests {
            use super::*;

            #[it("should add two positive numbers")]
            fn test_add_positive_numbers() {
                assert_eq!(2 + 3, 5);
            }

            #[it("should add zero")]
            fn test_add_zero() {
                assert_eq!(5 + 0, 5);
            }
        }

        #[describe("Subtraction")]
        mod subtraction_tests {
            use super::*;

            #[it("should subtract two numbers")]
            fn test_subtract_numbers() {
                assert_eq!(5 - 3, 2);
            }

            #[it("should handle negative results")]
            fn test_negative_result() {
                assert_eq!(3 - 5, -2);
            }
        }

        #[test]
        fn test_multiplication() {
            assert_eq!(3 * 4, 12);
        }

        #[test]
        fn test_division() {
            assert_eq!(8 / 2, 4);
        }
    }

    #[describe("Async Operations")]
    mod async_tests {
        use super::*;

        #[it("should handle async operations")]
        async fn test_async_operation() {
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            assert!(true);
        }
    }

    #[describe("Hooks")]
    mod hooks_tests {
        use super::*;

        static mut SETUP_CALLED: bool = false;
        static mut TEARDOWN_CALLED: bool = false;

        #[before_all]
        fn global_setup() {
            unsafe { SETUP_CALLED = true; }
        }

        #[after_all]
        fn global_teardown() {
            unsafe { TEARDOWN_CALLED = true; }
        }

        #[before_each]
        fn setup() {
            // Setup code here
        }

        #[after_each]
        fn teardown() {
            // Cleanup code here
        }

        #[it("should call hooks")]
        fn test_hooks() {
            unsafe {
                assert!(SETUP_CALLED);
            }
        }
    }

    #[describe("Error Handling")]
    mod error_tests {
        use super::*;

        #[it("should handle panics")]
        #[should_panic]
        fn test_panic() {
            panic!("This should panic");
        }

        #[it("should skip tests")]
        #[ignore]
        fn test_skip() {
            // This test will be skipped
            assert!(false);
        }
    }
}