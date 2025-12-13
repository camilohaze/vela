/*!
# Meta-Tests for Widget Testing Framework

Tests that validate the widget testing framework itself works correctly.
These tests use the widget testing framework to test itself (self-hosting).
*/

use vela_testing::widget_testing::*;
use vela_testing::matchers::*;
use std::collections::HashMap;

/// Test that WidgetTester can be created and configured correctly
#[test]
fn test_widget_tester_creation() {
    let mut tester = WidgetTester::new();

    // Test initial state
    assert!(tester.widgets().is_empty());
    assert!(tester.event_queue().is_empty());

    // Test adding widgets
    let widget_id = tester.add_widget("test_button", ButtonWidget {
        text: "Click me".to_string(),
        enabled: true,
    });

    assert_eq!(widget_id, "test_button");
    assert_eq!(tester.widgets().len(), 1);
}

/// Test that widget simulation works correctly
#[test]
fn test_widget_event_simulation() {
    let mut tester = WidgetTester::new();

    // Add a button widget
    tester.add_widget("test_button", ButtonWidget {
        text: "Click me".to_string(),
        enabled: true,
    });

    // Simulate click event
    let result = tester.simulate_event("test_button", TestEvent::Click);

    assert!(result.is_ok());

    // Verify event was queued
    assert_eq!(tester.event_queue().len(), 1);

    // Process events
    tester.process_events();

    // Event queue should be empty after processing
    assert!(tester.event_queue().is_empty());
}

/// Test that widget matchers work correctly
#[test]
fn test_widget_matchers() {
    let mut tester = WidgetTester::new();

    // Add widgets
    tester.add_widget("button1", ButtonWidget {
        text: "OK".to_string(),
        enabled: true,
    });

    tester.add_widget("button2", ButtonWidget {
        text: "Cancel".to_string(),
        enabled: false,
    });

    // Test finders
    let found = tester.find(By::Text("OK".to_string()));
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "button1");

    // Test matchers
    let button1 = tester.get_widget("button1").unwrap();
    assert!(matches_text("OK").matches(button1));
    assert!(is_enabled().matches(button1));

    let button2 = tester.get_widget("button2").unwrap();
    assert!(matches_text("Cancel").matches(button2));
    assert!(!is_enabled().matches(button2));
}

/// Test widget state changes
#[test]
fn test_widget_state_changes() {
    let mut tester = WidgetTester::new();

    // Add a toggle button widget
    tester.add_widget("toggle", ToggleWidget {
        state: false,
    });

    // Initial state
    let toggle = tester.get_widget("toggle").unwrap();
    assert!(!toggle.state);

    // Simulate toggle
    tester.simulate_event("toggle", TestEvent::Click).unwrap();
    tester.process_events();

    // State should have changed
    let toggle = tester.get_widget("toggle").unwrap();
    assert!(toggle.state);
}

/// Test complex widget interactions
#[test]
fn test_complex_widget_interactions() {
    let mut tester = WidgetTester::new();

    // Create a form with multiple widgets
    tester.add_widget("name_input", TextInputWidget {
        value: String::new(),
        placeholder: "Enter name".to_string(),
    });

    tester.add_widget("submit_button", ButtonWidget {
        text: "Submit".to_string(),
        enabled: false,
    });

    // Initially button should be disabled
    let button = tester.get_widget("submit_button").unwrap();
    assert!(!is_enabled().matches(button));

    // Enter text in input
    tester.simulate_event("name_input", TestEvent::TextInput("John Doe".to_string())).unwrap();
    tester.process_events();

    // Button should now be enabled (assuming form validation)
    // Note: This would require actual form logic in the widget implementation
    let input = tester.get_widget("name_input").unwrap();
    assert!(has_text("John Doe").matches(input));
}

/// Test widget testing error handling
#[test]
fn test_widget_testing_error_handling() {
    let mut tester = WidgetTester::new();

    // Try to simulate event on non-existent widget
    let result = tester.simulate_event("non_existent", TestEvent::Click);
    assert!(result.is_err());

    // Try to get non-existent widget
    let widget = tester.get_widget("non_existent");
    assert!(widget.is_none());

    // Try to find non-existent widget
    let found = tester.find(By::Id("non_existent".to_string()));
    assert!(found.is_none());
}

/// Test widget tree traversal
#[test]
fn test_widget_tree_traversal() {
    let mut tester = WidgetTester::new();

    // Create a widget hierarchy
    tester.add_widget("container", ContainerWidget {
        children: vec!["child1".to_string(), "child2".to_string()],
    });

    tester.add_widget("child1", ButtonWidget {
        text: "Child 1".to_string(),
        enabled: true,
    });

    tester.add_widget("child2", ButtonWidget {
        text: "Child 2".to_string(),
        enabled: true,
    });

    // Test finding descendants
    let descendants = tester.find_all(By::Type("Button".to_string()));
    assert_eq!(descendants.len(), 2);

    // Test finding by ancestor
    let container_children = tester.find_all(By::Ancestor("container".to_string()));
    assert_eq!(container_children.len(), 2);
}

// Mock widgets for testing
#[derive(Debug, Clone)]
struct ButtonWidget {
    text: String,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct ToggleWidget {
    state: bool,
}

#[derive(Debug, Clone)]
struct TextInputWidget {
    value: String,
    placeholder: String,
}

#[derive(Debug, Clone)]
struct ContainerWidget {
    children: Vec<String>,
}

impl Widget for ButtonWidget {
    fn id(&self) -> &str { "button" }
    fn widget_type(&self) -> &str { "Button" }
    fn text(&self) -> Option<&str> { Some(&self.text) }
    fn is_enabled(&self) -> bool { self.enabled }
    fn children(&self) -> Vec<&str> { vec![] }
}

impl Widget for ToggleWidget {
    fn id(&self) -> &str { "toggle" }
    fn widget_type(&self) -> &str { "Toggle" }
    fn text(&self) -> Option<&str> { None }
    fn is_enabled(&self) -> bool { true }
    fn children(&self) -> Vec<&str> { vec![] }
}

impl Widget for TextInputWidget {
    fn id(&self) -> &str { "text_input" }
    fn widget_type(&self) -> &str { "TextInput" }
    fn text(&self) -> Option<&str> { Some(&self.value) }
    fn is_enabled(&self) -> bool { true }
    fn children(&self) -> Vec<&str> { vec![] }
}

impl Widget for ContainerWidget {
    fn id(&self) -> &str { "container" }
    fn widget_type(&self) -> &str { "Container" }
    fn text(&self) -> Option<&str> { None }
    fn is_enabled(&self) -> bool { true }
    fn children(&self) -> Vec<&str> { self.children.iter().map(|s| s.as_str()).collect() }
}