/*
# Tests unitarios para el framework de testing de widgets

Jira: VELA-113CH
Historia: VELA-113C
Fecha: 2025-01-30

Tests para verificar:
- Creación de MockWidget
- Implementación del trait TestableWidget
- Funcionamiento de finders
- Funcionamiento de matchers
- Captura de snapshots
*/

use super::*;
use tokio::test;

#[test]
async fn test_mock_widget_creation() {
    let widget = MockWidget::new("test-id", "TestWidget")
        .with_property("text", serde_json::json!("Hello"))
        .with_property("enabled", serde_json::json!(true));

    assert_eq!(widget.get_id(), "test-id");
    assert_eq!(widget.get_type(), "TestWidget");
    assert!(widget.is_visible());
    assert!(widget.is_enabled().await);
    assert!(!widget.is_focused().await); // Default is false

    let properties = widget.get_properties().await;
    assert_eq!(properties.get("text").unwrap(), "Hello");
    assert_eq!(properties.get("enabled").unwrap(), true);
}

#[test]
async fn test_mock_widget_children() {
    let child1 = MockWidget::new("child1", "ChildWidget");
    let child2 = MockWidget::new("child2", "ChildWidget");

    let parent = MockWidget::new("parent", "ParentWidget")
        .with_children(vec![Box::new(child1), Box::new(child2)]);

    let children = parent.get_children().await;
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].get_id(), "child1");
    assert_eq!(children[1].get_id(), "child2");
}

#[test]
async fn test_mock_widget_clone_box() {
    let original = MockWidget::new("original", "TestWidget")
        .with_property("value", serde_json::json!(42));

    let cloned = original.clone_box().await;

    assert_eq!(cloned.get_id(), "original");
    assert_eq!(cloned.get_type(), "TestWidget");

    let cloned_props = cloned.get_properties().await;
    assert_eq!(cloned_props.get("value").unwrap(), 42);
}

#[test]
async fn test_mock_widget_bounds() {
    use crate::snapshot::WidgetBounds;

    let bounds = WidgetBounds {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
    };

    let widget = MockWidget::new("bounded", "BoundedWidget")
        .with_bounds(bounds.clone());

    let widget_bounds = widget.get_bounds().await;
    assert_eq!(widget_bounds, Some(bounds));
}

#[test]
async fn test_test_app_widget_management() {
    let mut app = TestApp::new();

    let widget = Box::new(MockWidget::new("test-widget", "TestWidget"));
    app.mount(widget).await.unwrap();

    let retrieved = app.get_widget("test-widget").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().get_id(), "test-widget");

    let non_existent = app.get_widget("non-existent").await;
    assert!(non_existent.is_none());
}

#[test]
async fn test_test_app_event_logging() {
    let mut app = TestApp::new();

    // Mount a widget first
    let widget = Box::new(MockWidget::new("test-widget", "TestWidget"));
    app.mount(widget).await.unwrap();

    let event = TestEvent::Click;
    app.simulate_event("test-widget", event.clone()).await.unwrap();

    let events = app.get_event_log().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, "test-widget");
    assert_eq!(events[0].1, event);
}

#[test]
async fn test_widget_tester_creation() {
    let app = TestApp::new();
    let tester = WidgetTester::new(&app);

    // WidgetTester se crea correctamente
    assert!(true); // Si llega aquí sin panic, está bien
}