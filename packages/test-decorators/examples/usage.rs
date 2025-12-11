//! Example usage of test decorators in Vela
//!
//! This file demonstrates how to use the test decorators implemented
//! in the test-decorators crate.

use test_decorators::*;

// Example of @test decorator with various configurations
#[test(name = "custom test name", timeout = 1000)]
fn test_with_custom_config() {
    assert_eq!(2 + 2, 4);
}

#[test(ignore = true)]
fn test_ignored() {
    // This test will be skipped
    assert!(false);
}

#[test(should_panic = "expected panic")]
fn test_should_panic() {
    panic!("expected panic");
}

// Example of BDD-style tests with @describe and @it
#[describe("Calculator operations")]
mod calculator_tests {
    #[it("should add two numbers correctly")]
    fn test_addition() {
        assert_eq!(2 + 3, 5);
    }

    #[it("should multiply two numbers correctly", timeout = 500)]
    fn test_multiplication() {
        assert_eq!(4 * 5, 20);
    }

    #[it("should handle division by zero", skip = true)]
    fn test_division_by_zero() {
        // This test is skipped
    }
}

// Alternative syntax using @context (alias for @describe)
#[context("String utilities")]
mod string_tests {
    #[it("should trim whitespace")]
    fn test_trim() {
        let s = "  hello  ".trim();
        assert_eq!(s, "hello");
    }

    #[it("should convert to uppercase")]
    fn test_uppercase() {
        let s = "hello".to_uppercase();
        assert_eq!(s, "HELLO");
    }
}

// Example of lifecycle hooks
#[before_each(timeout = 100)]
fn setup_each_test() {
    // This runs before each test
    println!("Setting up test...");
}

#[after_each]
fn cleanup_each_test() {
    // This runs after each test
    println!("Cleaning up test...");
}

#[before_all(timeout = 500)]
fn global_setup() {
    // This runs once before all tests in the module
    println!("Global setup...");
}

#[after_all]
fn global_cleanup() {
    // This runs once after all tests in the module
    println!("Global cleanup...");
}

// Regular test without decorators (for comparison)
fn regular_function() {
    // This is not a test
}

#[test]
fn integration_test() {
    // This is a test that can use the setup/cleanup functions
    assert!(true);
}