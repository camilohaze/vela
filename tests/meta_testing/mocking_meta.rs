/*!
# Meta-Tests for Mocking Framework

Tests that validate the mocking framework itself works correctly.
These tests use mocking to test the mocking framework (self-hosting).
*/

use vela_testing::mocking::*;
use std::sync::Arc;

/// Test that mock creation works correctly
#[test]
fn test_mock_creation() {
    // Create a mock using the macro
    mock!(TestService {
        name: String,
        version: u32,
    });

    let mock_service = TestService::new();

    // Verify mock is created
    assert!(mock_service.is_mock());
}

/// Test that method stubbing works
#[test]
fn test_method_stubbing() {
    mock!(CalculatorService {
        base_value: i32,
    });

    let mut mock_calc = CalculatorService::new();

    // Configure stubbing
    mock_calc.when().add(5, 3).returns(8);
    mock_calc.when().multiply(4, 2).returns(8);

    // Test stubbing
    assert_eq!(mock_calc.add(5, 3), 8);
    assert_eq!(mock_calc.multiply(4, 2), 8);

    // Test unstubbed methods return default
    assert_eq!(mock_calc.add(1, 1), 0); // Default for i32
}

/// Test that call verification works
#[test]
fn test_call_verification() {
    mock!(EmailService {
        sent_emails: Vec<String>,
    });

    let mut mock_email = EmailService::new();

    // Call methods
    mock_email.send_email("test@example.com", "Subject", "Body");
    mock_email.send_email("test2@example.com", "Subject2", "Body2");

    // Verify calls
    mock_email.verify_method("send_email").called_times(2);
    mock_email.verify_method("send_email").called_with_args(vec![
        "test@example.com".into(),
        "Subject".into(),
        "Body".into()
    ]);
}

/// Test that argument matching works
#[test]
fn test_argument_matching() {
    mock!(DatabaseService {
        connection_string: String,
    });

    let mut mock_db = DatabaseService::new();

    // Configure with argument matchers
    mock_db.when().query("SELECT * FROM users").returns(vec!["user1", "user2"]);
    mock_db.when().query("SELECT * FROM products").returns(vec!["product1"]);

    // Test matching
    assert_eq!(mock_db.query("SELECT * FROM users"), vec!["user1", "user2"]);
    assert_eq!(mock_db.query("SELECT * FROM products"), vec!["product1"]);

    // Test non-matching
    assert_eq!(mock_db.query("SELECT * FROM orders"), Vec::<String>::new());
}

/// Test sequence verification
#[test]
fn test_sequence_verification() {
    mock!(WorkflowService {
        steps: Vec<String>,
    });

    let mut mock_workflow = WorkflowService::new();

    // Execute steps in sequence
    mock_workflow.step1();
    mock_workflow.step2();
    mock_workflow.step3();

    // Verify sequence
    mock_workflow.verify_method("step1").called_before("step2");
    mock_workflow.verify_method("step2").called_before("step3");
}

/// Test mock reset functionality
#[test]
fn test_mock_reset() {
    mock!(CounterService {
        count: i32,
    });

    let mut mock_counter = CounterService::new();

    // Configure initial behavior
    mock_counter.when().increment().returns(1);
    assert_eq!(mock_counter.increment(), 1);

    // Reset mock
    mock_counter.reset();

    // After reset, should return defaults
    assert_eq!(mock_counter.increment(), 0);

    // Verify no calls recorded after reset
    mock_counter.verify_method("increment").never_called();
}

/// Test mock cloning
#[test]
fn test_mock_cloning() {
    mock!(CloneableService {
        data: String,
    });

    let mut mock1 = CloneableService::new();
    mock1.when().get_data().returns("data1".to_string());

    // Clone the mock
    let mut mock2 = mock1.clone();

    // Both should work independently
    assert_eq!(mock1.get_data(), "data1");
    assert_eq!(mock2.get_data(), "data1");

    // Configure different behavior for clone
    mock2.when().get_data().returns("data2".to_string());
    assert_eq!(mock1.get_data(), "data1");
    assert_eq!(mock2.get_data(), "data2");
}

/// Test error handling in mocks
#[test]
fn test_mock_error_handling() {
    mock!(ErrorService {
        errors: Vec<String>,
    });

    let mut mock_error = ErrorService::new();

    // Configure to throw errors
    mock_error.when().process_data("invalid").throws("Invalid data".to_string());

    // Test error throwing
    let result = mock_error.process_data("invalid");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid data");

    // Test success case
    mock_error.when().process_data("valid").returns("Processed".to_string());
    let result = mock_error.process_data("valid");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Processed");
}

/// Test complex argument matching
#[test]
fn test_complex_argument_matching() {
    mock!(ComplexService {
        complex_data: HashMap<String, Vec<i32>>,
    });

    let mut mock_complex = ComplexService::new();

    // Configure with complex arguments
    let mut expected_map = HashMap::new();
    expected_map.insert("key1".to_string(), vec![1, 2, 3]);

    mock_complex.when().process_map(expected_map.clone()).returns(true);

    // Test with matching arguments
    assert!(mock_complex.process_map(expected_map));

    // Test with non-matching arguments
    let mut different_map = HashMap::new();
    different_map.insert("key2".to_string(), vec![4, 5, 6]);
    assert!(!mock_complex.process_map(different_map));
}

/// Test mock performance under load
#[test]
fn test_mock_performance() {
    mock!(PerformanceService {
        call_count: u64,
    });

    let mut mock_perf = PerformanceService::new();

    // Configure simple return
    mock_perf.when().light_operation().returns(42);

    // Perform many calls
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = mock_perf.light_operation();
    }
    let duration = start.elapsed();

    // Should complete quickly (less than 1 second for 10k calls)
    assert!(duration.as_millis() < 1000);

    // Verify all calls were recorded
    mock_perf.verify_method("light_operation").called_times(10000);
}

// Define mock services for testing
mock!(TestService {
    name: String,
    version: u32,
});

mock!(CalculatorService {
    base_value: i32,
});

mock!(EmailService {
    sent_emails: Vec<String>,
});

mock!(DatabaseService {
    connection_string: String,
});

mock!(WorkflowService {
    steps: Vec<String>,
});

mock!(CounterService {
    count: i32,
});

mock!(CloneableService {
    data: String,
});

mock!(ErrorService {
    errors: Vec<String>,
});

mock!(ComplexService {
    complex_data: HashMap<String, Vec<i32>>,
});

mock!(PerformanceService {
    call_count: u64,
});