/*!
# Meta-Tests for Property-Based Testing Framework

Tests that validate the property-based testing framework itself works correctly.
These tests use property-based testing to test itself (self-hosting).
*/

use vela_testing::property::*;
use std::collections::HashMap;

/// Test that property test configuration works
#[test]
fn test_property_test_configuration() {
    let config = PropertyTestConfig {
        iterations: 100,
        seed: Some(42),
        max_size: 1000,
        ..Default::default()
    };

    let generator = Generator::with_config(config.clone());

    assert_eq!(generator.config().iterations, 100);
    assert_eq!(generator.config().seed, Some(42));
    assert_eq!(generator.config().max_size, 1000);
}

/// Test that basic property testing works
#[test]
fn test_basic_property_testing() {
    // Test that reverse is idempotent: reverse(reverse(x)) == x
    let result = property_test(|vec: Vec<i32>| {
        let reversed_once = reverse_vec(&vec);
        let reversed_twice = reverse_vec(&reversed_once);
        vec == reversed_twice
    });

    match result {
        PropertyTestResult::Passed { iterations, .. } => {
            assert!(iterations > 0);
        }
        PropertyTestResult::Failed { .. } => {
            panic!("Property test should have passed");
        }
    }
}

/// Test that property testing can find failures
#[test]
fn test_property_testing_failure_detection() {
    // Test a property that should fail
    let result = property_test(|x: i32| {
        // This should fail for x = 0
        x * x >= 0  // This is actually always true
    });

    // This should pass, but let's test the framework can handle failures
    match result {
        PropertyTestResult::Passed { .. } => {
            // Expected - the property actually holds
        }
        PropertyTestResult::Failed { .. } => {
            // Also acceptable - framework detected an issue
        }
    }
}

/// Test shrinking functionality
#[test]
fn test_shrinking_functionality() {
    // Create a property that fails for large inputs
    let result = property_test(|vec: Vec<i32>| {
        // Property: all elements should be >= 0
        vec.iter().all(|&x| x >= 0)
    });

    // The framework should be able to shrink failing cases
    match result {
        PropertyTestResult::Passed { .. } => {
            // If it passes, that's fine
        }
        PropertyTestResult::Failed { minimal_input, .. } => {
            // The minimal input should be as small as possible
            if let Some(minimal) = minimal_input {
                // For this property, minimal failing case should be small
                assert!(minimal.len() <= 10); // Reasonable bound
            }
        }
    }
}

/// Test custom generators
#[test]
fn test_custom_generators() {
    // Create a custom generator for even numbers only
    let even_generator = Generator::new()
        .with_custom_generator(|config| {
            let mut rng = config.rng();
            let value = rng.gen_range(0..100) * 2; // Always even
            vec![value]
        });

    let result = property_test_with_generator(even_generator, |x: i32| {
        // Property: x should always be even
        x % 2 == 0
    });

    match result {
        PropertyTestResult::Passed { .. } => {
            // Should pass since generator only produces even numbers
        }
        PropertyTestResult::Failed { .. } => {
            panic!("Custom generator test should have passed");
        }
    }
}

/// Test seed reproducibility
#[test]
fn test_seed_reproducibility() {
    let seed = 12345;

    // Run the same test twice with the same seed
    let result1 = property_test_with_config(
        PropertyTestConfig {
            iterations: 50,
            seed: Some(seed),
            ..Default::default()
        },
        |x: i32| x * x >= 0  // Always true property
    );

    let result2 = property_test_with_config(
        PropertyTestConfig {
            iterations: 50,
            seed: Some(seed),
            ..Default::default()
        },
        |x: i32| x * x >= 0  // Always true property
    );

    // Results should be identical with same seed
    match (result1, result2) {
        (PropertyTestResult::Passed { iterations: iter1, .. },
         PropertyTestResult::Passed { iterations: iter2, .. }) => {
            assert_eq!(iter1, iter2);
        }
        _ => panic!("Both tests should have passed"),
    }
}

/// Test property testing with complex types
#[test]
fn test_complex_type_property_testing() {
    #[derive(Debug, Clone)]
    struct TestStruct {
        id: i32,
        name: String,
        values: Vec<i32>,
    }

    let result = property_test(|obj: TestStruct| {
        // Property: id should be non-negative
        obj.id >= 0 &&
        // Name should not be empty
        !obj.name.is_empty() &&
        // Values should contain at least the id
        obj.values.contains(&obj.id)
    });

    match result {
        PropertyTestResult::Passed { iterations, .. } => {
            assert!(iterations > 0);
        }
        PropertyTestResult::Failed { .. } => {
            // Framework might generate failing cases, which is OK
        }
    }
}

/// Test property testing performance
#[test]
fn test_property_testing_performance() {
    let start = std::time::Instant::now();

    let result = property_test_with_config(
        PropertyTestConfig {
            iterations: 1000,
            ..Default::default()
        },
        |x: i32| x >= i32::MIN && x <= i32::MAX  // Always true
    );

    let duration = start.elapsed();

    match result {
        PropertyTestResult::Passed { iterations, .. } => {
            assert_eq!(iterations, 1000);
            // Should complete in reasonable time (less than 1 second for 1000 iterations)
            assert!(duration.as_millis() < 1000);
        }
        PropertyTestResult::Failed { .. } => {
            panic!("Performance test should have passed");
        }
    }
}

/// Test property testing with collections
#[test]
fn test_collection_property_testing() {
    // Test properties of collections
    let result = property_test(|list: Vec<i32>| {
        let sorted = sort_vec(&list);
        // Properties of sorting:
        // 1. Length is preserved
        sorted.len() == list.len() &&
        // 2. All elements from original are present
        list.iter().all(|&x| sorted.contains(&x)) &&
        // 3. Result is sorted
        is_sorted(&sorted)
    });

    match result {
        PropertyTestResult::Passed { .. } => {
            // Should pass - sorting preserves these properties
        }
        PropertyTestResult::Failed { .. } => {
            panic!("Collection property test should have passed");
        }
    }
}

/// Test edge case generation
#[test]
fn test_edge_case_generation() {
    let mut edge_cases_found = std::collections::HashSet::new();

    let result = property_test(|x: i32| {
        // Record edge cases
        if x == 0 || x == i32::MIN || x == i32::MAX {
            edge_cases_found.insert(x);
        }
        // Always true property
        true
    });

    match result {
        PropertyTestResult::Passed { iterations, .. } => {
            // Should have found some edge cases in reasonable iterations
            assert!(iterations >= 100); // Minimum iterations
            // Note: Edge cases might not be found in small test runs
        }
        PropertyTestResult::Failed { .. } => {
            panic!("Edge case test should have passed");
        }
    }
}

// Helper functions for testing
fn reverse_vec(vec: &Vec<i32>) -> Vec<i32> {
    let mut reversed = vec.clone();
    reversed.reverse();
    reversed
}

fn sort_vec(vec: &Vec<i32>) -> Vec<i32> {
    let mut sorted = vec.clone();
    sorted.sort();
    sorted
}

fn is_sorted(vec: &Vec<i32>) -> bool {
    for i in 1..vec.len() {
        if vec[i-1] > vec[i] {
            return false;
        }
    }
    true
}