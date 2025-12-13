/*!
# Widget Testing Framework Tests

Tests for the widget testing framework itself.
These tests verify that the testing framework works correctly.
*/

use vela_testing::*;
use vela_ui::*;

/// Test basic widget mounting
#[tokio::test]
async fn test_widget_mounting() {
    let mut tester = WidgetTester::new();

    // Create a simple text widget
    let text_widget = Text::new("Hello World");

    // Mount the widget
    tester.pump_widget(text_widget).await;

    // Verify the widget was mounted
    let found_widgets = tester.find(find::by_type::<Text>()).await;
    assert_eq!(found_widgets.len(), 1);
}

/// Test text content finding
#[tokio::test]
async fn test_text_finding() {
    let mut tester = WidgetTester::new();

    let text_widget = Text::new("Hello World");
    tester.pump_widget(text_widget).await;

    // Find by text content
    let found_widgets = tester.find(find::by_text("Hello World")).await;
    assert_eq!(found_widgets.len(), 1);

    // Find by partial text
    let partial_widgets = tester.find(find::by_text("Hello")).await;
    assert_eq!(partial_widgets.len(), 1);
}

/// Test text matcher
#[tokio::test]
async fn test_text_matcher() {
    let mut tester = WidgetTester::new();

    let text_widget = Text::new("Hello World");
    tester.pump_widget(text_widget).await;

    // Test successful match
    let result = tester.expect(find::by_type::<Text>().to_have_text("Hello World")).await;
    assert!(result.is_ok());

    // Test failed match
    let result = tester.expect(find::by_type::<Text>().to_have_text("Goodbye World")).await;
    assert!(result.is_err());
}

/// Test button interaction simulation
#[tokio::test]
async fn test_button_interaction() {
    let mut tester = WidgetTester::new();

    // Create a button widget (simplified for testing)
    let button = Text::new("Click Me"); // Using Text as placeholder for Button
    tester.pump_widget(button).await;

    // Find the button
    let buttons = tester.find(find::by_type::<Text>()).await;
    assert_eq!(buttons.len(), 1);

    // Simulate tap (this would normally trigger onPressed)
    let result = tester.tap(find::by_type::<Text>()).await;
    assert!(result.is_ok());
}

/// Test text input simulation
#[tokio::test]
async fn test_text_input() {
    let mut tester = WidgetTester::new();

    // Create a text input widget (simplified)
    let input = Text::new(""); // Placeholder for TextInput
    tester.pump_widget(input).await;

    // Simulate text input
    let result = tester.enter_text(find::by_type::<Text>(), "Hello").await;
    assert!(result.is_ok());
}

/// Test finder combinations
#[tokio::test]
async fn test_finder_combinations() {
    let mut tester = WidgetTester::new();

    // Create a more complex widget tree (simplified)
    let container = Text::new("Container"); // Placeholder
    tester.pump_widget(container).await;

    // Test different finder types
    let by_type = tester.find(find::by_type::<Text>()).await;
    assert!(!by_type.is_empty());

    // Test by_text finder
    let by_text = tester.find(find::by_text("Container")).await;
    assert!(!by_text.is_empty());
}

/// Test interaction sequences
#[tokio::test]
async fn test_interaction_sequence() {
    let mut tester = WidgetTester::new();

    let widget = Text::new("Test");
    tester.pump_widget(widget).await;

    // Create interaction sequence
    let sequence = sequence()
        .tap(find::by_type::<Text>())
        .enter_text(find::by_type::<Text>(), "Updated".to_string());

    // Execute sequence
    let result = sequence.execute(&mut tester.app).await;
    assert!(result.is_ok());
}

/// Test reactive updates
#[tokio::test]
async fn test_reactive_updates() {
    let mut tester = WidgetTester::new();

    let widget = Text::new("Initial");
    tester.pump_widget(widget).await;

    // Simulate some interaction that would trigger reactive update
    tester.tap(find::by_type::<Text>()).await.unwrap();

    // Pump to process reactive updates
    tester.pump().await;

    // Verify the widget tree is still valid
    let widgets = tester.find(find::by_type::<Text>()).await;
    assert!(!widgets.is_empty());
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() {
    let mut tester = WidgetTester::new();

    let widget = Text::new("Test");
    tester.pump_widget(widget).await;

    // Try to find non-existent widget
    let not_found = tester.find(find::by_text("NonExistent")).await;
    assert!(not_found.is_empty());

    // Try to tap non-existent widget
    let result = tester.tap(find::by_text("NonExistent")).await;
    assert!(result.is_err());
}

/// Test visibility matcher
#[tokio::test]
async fn test_visibility_matcher() {
    let mut tester = WidgetTester::new();

    let widget = Text::new("Visible");
    tester.pump_widget(widget).await;

    // Test visibility (assuming widgets are visible by default)
    let result = tester.expect(find::by_type::<Text>().to_be_visible()).await;
    assert!(result.is_ok());
}

/// Performance test for widget finding
#[tokio::test]
async fn test_widget_finding_performance() {
    let mut tester = WidgetTester::new();

    // Create multiple widgets
    let container = Text::new("Container");
    tester.pump_widget(container).await;

    // Measure time for finding operations
    let start = std::time::Instant::now();

    for _ in 0..100 {
        let _widgets = tester.find(find::by_type::<Text>()).await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000); // Should complete within 1 second
}

/// Test concurrent widget testing
#[tokio::test]
async fn test_concurrent_testing() {
    let tester1 = WidgetTester::new();
    let tester2 = WidgetTester::new();

    // Test that multiple testers can work independently
    // This is more of a compilation test than a runtime test
    assert!(tester1.app.widget_tree.is_some());
    assert!(tester2.app.widget_tree.is_some());
}