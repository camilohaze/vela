/*!
# Property-Based Testing Tests

Comprehensive test suite for the property-based testing framework.
Tests cover data generation, shrinking, property testing, and edge cases.
*/

use super::property::*;
use serde_json::Value;

#[cfg(test)]
mod property_tests {
    use super::*;

    // Test data generation for various types

    #[test]
    fn test_arbitrary_i32_generation() {
        for _ in 0..100 {
            let value: i32 = i32::arbitrary();
            assert!(value >= -10000 && value <= 10000);
        }
    }

    #[test]
    fn test_arbitrary_u32_generation() {
        for _ in 0..100 {
            let value: u32 = u32::arbitrary();
            assert!(value <= 100000);
        }
    }

    #[test]
    fn test_arbitrary_bool_generation() {
        let mut true_count = 0;
        let mut false_count = 0;

        for _ in 0..1000 {
            let value: bool = bool::arbitrary();
            if value {
                true_count += 1;
            } else {
                false_count += 1;
            }
        }

        // Should be roughly balanced
        assert!(true_count > 300 && false_count > 300);
    }

    #[test]
    fn test_arbitrary_string_generation() {
        for _ in 0..100 {
            let value: String = String::arbitrary();
            assert!(value.len() <= 20);
            // All characters should be printable ASCII
            assert!(value.chars().all(|c| c.is_ascii() && (c.is_alphanumeric() || c.is_ascii_punctuation() || c == ' ')));
        }
    }

    #[test]
    fn test_arbitrary_vec_generation() {
        for _ in 0..50 {
            let value: Vec<i32> = Vec::arbitrary();
            assert!(value.len() <= 10);
            for &item in &value {
                assert!(item >= -10000 && item <= 10000);
            }
        }
    }

    #[test]
    fn test_arbitrary_option_generation() {
        let mut some_count = 0;
        let mut none_count = 0;

        for _ in 0..1000 {
            let value: Option<i32> = Option::arbitrary();
            match value {
                Some(_) => some_count += 1,
                None => none_count += 1,
            }
        }

        // Should be roughly balanced
        assert!(some_count > 300 && none_count > 300);
    }

    #[test]
    fn test_arbitrary_tuple_generation() {
        for _ in 0..50 {
            let value: (i32, String) = <(i32, String)>::arbitrary();
            assert!(value.0 >= -10000 && value.0 <= 10000);
            assert!(value.1.len() <= 20);
        }
    }

    // Test shrinking functionality

    #[test]
    fn test_i32_shrinking() {
        let value: i32 = 1000;
        let shrunk = value.shrink();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0));
        assert!(shrunk.contains(&500));

        // All shrunk values should be smaller in absolute value
        for s in &shrunk {
            assert!(s.abs() < value.abs());
        }
    }

    #[test]
    fn test_negative_i32_shrinking() {
        let value: i32 = -500;
        let shrunk = value.shrink();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0));
        assert!(shrunk.contains(&250)); // -(-500)/2 = 250

        for s in &shrunk {
            assert!(s.abs() < value.abs());
        }
    }

    #[test]
    fn test_zero_i32_no_shrinking() {
        let value = 0;
        let shrunk = value.shrink();
        assert!(shrunk.is_empty());
    }

    #[test]
    fn test_string_shrinking() {
        let value = "hello world".to_string();
        let shrunk = value.shrink();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&String::new()));
        assert!(shrunk.contains(&"ello world".to_string()));
        assert!(shrunk.contains(&"hello worl".to_string()));

        // All shrunk strings should be shorter
        for s in &shrunk {
            assert!(s.len() < value.len());
        }
    }

    #[test]
    fn test_empty_string_no_shrinking() {
        let value = String::new();
        let shrunk = value.shrink();
        assert!(shrunk.is_empty());
    }

    #[test]
    fn test_vec_shrinking() {
        let value = vec![1, 2, 3, 4, 5];
        let shrunk = value.shrink();

        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&Vec::<i32>::new()));
        assert!(shrunk.contains(&vec![2, 3, 4, 5]));
        assert!(shrunk.contains(&vec![1, 2, 3, 4]));

        for s in &shrunk {
            assert!(s.len() < value.len());
        }
    }

    #[test]
    fn test_empty_vec_no_shrinking() {
        let value: Vec<i32> = Vec::new();
        let shrunk = value.shrink();
        assert!(shrunk.is_empty());
    }

    #[test]
    fn test_bool_shrinking() {
        let true_value = true;
        let shrunk_true = true_value.shrink();
        assert_eq!(shrunk_true, vec![false]);

        let false_value = false;
        let shrunk_false = false_value.shrink();
        assert!(shrunk_false.is_empty());
    }

    #[test]
    fn test_option_shrinking() {
        let some_value = Some(42);
        let shrunk_some = some_value.shrink();
        assert!(!shrunk_some.is_empty());
        assert!(shrunk_some.contains(&None));

        let none_value: Option<i32> = None;
        let shrunk_none = none_value.shrink();
        assert!(shrunk_none.is_empty());
    }

    // Test property testing functionality

    #[test]
    fn test_property_test_passing() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 50,
                ..Default::default()
            },
            |x: i32| x * 2 == x + x, // Always true
        );

        match result {
            PropertyTestResult::Passed { iterations, .. } => {
                assert_eq!(iterations, 50);
            }
            PropertyTestResult::Failed { .. } => panic!("Property should pass"),
        }
    }

    #[test]
    fn test_property_test_failing() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 100,
                ..Default::default()
            },
            |x: i32| x < 5000, // Will eventually fail for large positive numbers
        );

        match result {
            PropertyTestResult::Passed { .. } => panic!("Property should fail"),
            PropertyTestResult::Failed { iterations, failing_case, .. } => {
                assert!(iterations <= 100);
                // Should be able to parse the failing case
                if let Value::Number(n) = failing_case {
                    assert!(n.as_i64().unwrap() >= 5000);
                } else {
                    panic!("Expected number in failing case");
                }
            }
        }
    }

    #[test]
    fn test_property_test2_passing() {
        let result = property_test2_with_config(
            PropertyTestConfig {
                iterations: 50,
                ..Default::default()
            },
            |a: i32, b: i32| a + b == b + a, // Commutative property
        );

        match result {
            PropertyTestResult::Passed { iterations, .. } => {
                assert_eq!(iterations, 50);
            }
            PropertyTestResult::Failed { .. } => panic!("Property should pass"),
        }
    }

    #[test]
    fn test_property_test2_failing() {
        let result = property_test2_with_config(
            PropertyTestConfig {
                iterations: 100,
                ..Default::default()
            },
            |a: i32, b: i32| a + b < 10000, // Will fail for large numbers
        );

        match result {
            PropertyTestResult::Passed { .. } => panic!("Property should fail"),
            PropertyTestResult::Failed { iterations, failing_case, .. } => {
                assert!(iterations <= 100);
                // Should be able to parse the failing case as an array
                if let Value::Array(arr) = failing_case {
                    assert_eq!(arr.len(), 2);
                } else {
                    panic!("Expected array in failing case");
                }
            }
        }
    }

    #[test]
    fn test_property_test_with_shrinking() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 10,
                shrink: true,
                ..Default::default()
            },
            |x: i32| x < 50, // Will fail for x >= 50, should shrink to smaller values
        );

        match result {
            PropertyTestResult::Passed { .. } => panic!("Property should fail"),
            PropertyTestResult::Failed { shrunk_case, .. } => {
                if let Some(Value::Number(n)) = shrunk_case {
                    let shrunk_val = n.as_i64().unwrap() as i32;
                    // The shrunk case should be the smallest failing value, which is 50
                    assert_eq!(shrunk_val, 50);
                } else {
                    panic!("Expected shrunk case to be a number");
                }
            }
        }
    }

    #[test]
    fn test_property_test_no_shrinking() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 10,
                shrink: false,
                ..Default::default()
            },
            |x: i32| x < 100,
        );

        match result {
            PropertyTestResult::Passed { .. } => panic!("Property should fail"),
            PropertyTestResult::Failed { shrunk_case, .. } => {
                assert!(shrunk_case.is_none());
            }
        }
    }

    // Test configuration

    #[test]
    fn test_config_defaults() {
        let config = PropertyTestConfig::default();
        assert_eq!(config.iterations, 100);
        assert!(config.seed.is_none());
        assert_eq!(config.max_size, 100);
        assert!(config.shrink);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_config_custom() {
        let config = PropertyTestConfig {
            iterations: 50,
            seed: Some(12345),
            max_size: 50,
            shrink: false,
            timeout_secs: 60,
        };

        assert_eq!(config.iterations, 50);
        assert_eq!(config.seed, Some(12345));
        assert_eq!(config.max_size, 50);
        assert!(!config.shrink);
        assert_eq!(config.timeout_secs, 60);
    }

    // Test generator

    #[test]
    fn test_generator_creation() {
        let generator = Generator::new();
        // Should not panic
    }

    #[test]
    fn test_generator_with_config() {
        let config = PropertyTestConfig {
            iterations: 50,
            seed: Some(42),
            ..Default::default()
        };
        let generator = Generator::with_config(config);
        // Should not panic
    }

    #[test]
    fn test_generator_generate() {
        let mut generator = Generator::new();

        let i32_val: i32 = generator.generate();
        assert!(i32_val >= -10000 && i32_val <= 10000);

        let string_val: String = generator.generate();
        assert!(string_val.len() <= 20);

        let vec_val: Vec<i32> = generator.generate();
        assert!(vec_val.len() <= 10);
    }

    #[test]
    fn test_generator_generate_vec() {
        let mut generator = Generator::new();

        let vec = generator.generate_vec::<i32>(5);
        assert_eq!(vec.len(), 5);

        let empty_vec = generator.generate_vec::<i32>(0);
        assert_eq!(empty_vec.len(), 0);

        // Should respect max_size
        let mut config = PropertyTestConfig::default();
        config.max_size = 3;
        let mut generator = Generator::with_config(config);
        let vec = generator.generate_vec::<i32>(10);
        assert_eq!(vec.len(), 3); // Limited by max_size
    }

    // Test edge cases

    #[test]
    fn test_property_test_zero_iterations() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 0,
                ..Default::default()
            },
            |_: i32| false, // Would fail, but no iterations
        );

        match result {
            PropertyTestResult::Passed { iterations, .. } => {
                assert_eq!(iterations, 0);
            }
            _ => panic!("Should pass with 0 iterations"),
        }
    }

    #[test]
    fn test_property_test_single_iteration() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 1,
                ..Default::default()
            },
            |x: i32| x >= -10000, // Should pass
        );

        match result {
            PropertyTestResult::Passed { iterations, .. } => {
                assert_eq!(iterations, 1);
            }
            _ => panic!("Should pass"),
        }
    }

    // Test complex properties

    #[test]
    fn test_reverse_reverse_property() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 100,
                ..Default::default()
            },
            |vec: Vec<i32>| {
                let reversed = reverse_vec(&vec);
                let double_reversed = reverse_vec(&reversed);
                vec == double_reversed
            },
        );

        match result {
            PropertyTestResult::Passed { .. } => {}
            _ => panic!("Reverse reverse property should hold"),
        }
    }

    #[test]
    fn test_sort_stability_property() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 50,
                ..Default::default()
            },
            |mut vec: Vec<i32>| {
                let original_len = vec.len();
                vec.sort();
                vec.len() == original_len
            },
        );

        match result {
            PropertyTestResult::Passed { .. } => {}
            _ => panic!("Sort should preserve length"),
        }
    }

    // Helper functions for tests

    fn reverse_vec(vec: &[i32]) -> Vec<i32> {
        let mut reversed = vec.to_vec();
        reversed.reverse();
        reversed
    }
}