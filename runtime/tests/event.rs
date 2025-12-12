//! Tests for the event system

use vela_runtime::{EventBus, EventError, EventResult, EventHandler};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq)]
struct TestEvent {
    id: u32,
    message: String,
}

#[derive(Debug)]
struct TestHandler {
    call_count: Arc<AtomicUsize>,
    should_fail: bool,
}

impl TestHandler {
    fn new(call_count: Arc<AtomicUsize>, should_fail: bool) -> Self {
        Self {
            call_count,
            should_fail,
        }
    }
}

#[async_trait::async_trait]
impl EventHandler<TestEvent> for TestHandler {
    async fn handle(&self, event: &TestEvent) -> EventResult<()> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        if self.should_fail {
            return Err(EventError::HandlerFailed {
                event_type: "TestEvent".to_string(),
                cause: "Test failure".to_string(),
            });
        }

        println!("Handled event: {} - {}", event.id, event.message);
        Ok(())
    }
}

#[tokio::test]
async fn test_publish_without_handlers() {
    let bus = EventBus::new();
    let event = TestEvent {
        id: 1,
        message: "test".to_string(),
    };

    let result = bus.publish(event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_and_publish() {
    let bus = EventBus::new();
    let call_count = Arc::new(AtomicUsize::new(0));

    let handler = TestHandler::new(call_count.clone(), false);
    bus.subscribe(handler).unwrap();

    let event = TestEvent {
        id: 42,
        message: "hello world".to_string(),
    };

    let result = bus.publish(event).await;
    assert!(result.is_ok());
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_multiple_handlers() {
    let bus = EventBus::new();
    let call_count1 = Arc::new(AtomicUsize::new(0));
    let call_count2 = Arc::new(AtomicUsize::new(0));

    let handler1 = TestHandler::new(call_count1.clone(), false);
    let handler2 = TestHandler::new(call_count2.clone(), false);

    bus.subscribe(handler1).unwrap();
    bus.subscribe(handler2).unwrap();

    let event = TestEvent {
        id: 100,
        message: "broadcast".to_string(),
    };

    let result = bus.publish(event).await;
    assert!(result.is_ok());
    assert_eq!(call_count1.load(Ordering::SeqCst), 1);
    assert_eq!(call_count2.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_handler_error() {
    let bus = EventBus::new();
    let call_count = Arc::new(AtomicUsize::new(0));

    let handler = TestHandler::new(call_count.clone(), true);
    bus.subscribe(handler).unwrap();

    let event = TestEvent {
        id: 1,
        message: "fail".to_string(),
    };

    let result = bus.publish(event).await;
    assert!(result.is_err());
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    if let Err(EventError::HandlerFailed { event_type, cause }) = result {
        assert_eq!(event_type, "TestEvent");
        assert_eq!(cause, "Test failure");
    } else {
        panic!("Expected HandlerFailed error");
    }
}

#[tokio::test]
async fn test_different_event_types() {
    let bus = EventBus::new();

    #[derive(Debug, Clone, PartialEq)]
    struct OtherEvent {
        value: i32,
    }

    let call_count1 = Arc::new(AtomicUsize::new(0));
    let call_count2 = Arc::new(AtomicUsize::new(0));

    struct OtherHandler(Arc<AtomicUsize>);
    #[async_trait::async_trait]
    impl EventHandler<OtherEvent> for OtherHandler {
        async fn handle(&self, event: &OtherEvent) -> EventResult<()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    let handler1 = TestHandler::new(call_count1.clone(), false);
    let handler2 = OtherHandler(call_count2.clone());

    bus.subscribe(handler1).unwrap();
    bus.subscribe(handler2).unwrap();

    // Publish TestEvent - only TestHandler should be called
    let test_event = TestEvent {
        id: 1,
        message: "test".to_string(),
    };
    bus.publish(test_event).await.unwrap();
    assert_eq!(call_count1.load(Ordering::SeqCst), 1);
    assert_eq!(call_count2.load(Ordering::SeqCst), 0);

    // Publish OtherEvent - only OtherHandler should be called
    let other_event = OtherEvent { value: 42 };
    bus.publish(other_event).await.unwrap();
    assert_eq!(call_count1.load(Ordering::SeqCst), 1);
    assert_eq!(call_count2.load(Ordering::SeqCst), 1);
}

#[test]
fn test_handler_count() {
    let bus = EventBus::new();

    assert_eq!(bus.handler_count::<TestEvent>(), 0);

    let call_count = Arc::new(AtomicUsize::new(0));
    let handler1 = TestHandler::new(call_count.clone(), false);
    let handler2 = TestHandler::new(call_count, false);

    bus.subscribe(handler1).unwrap();
    assert_eq!(bus.handler_count::<TestEvent>(), 1);

    bus.subscribe(handler2).unwrap();
    assert_eq!(bus.handler_count::<TestEvent>(), 2);
}

#[test]
fn test_shutdown() {
    let mut bus = EventBus::new();
    let call_count = Arc::new(AtomicUsize::new(0));
    let handler = TestHandler::new(call_count, false);

    bus.subscribe(handler).unwrap();
    assert_eq!(bus.handler_count::<TestEvent>(), 1);

    bus.shutdown();
    assert_eq!(bus.handler_count::<TestEvent>(), 0);
    assert!(bus.is_shutdown());
}

#[tokio::test]
async fn test_concurrent_publishing() {
    let bus = Arc::new(EventBus::new());
    let call_count = Arc::new(AtomicUsize::new(0));

    let handler = TestHandler::new(call_count.clone(), false);
    bus.subscribe(handler).unwrap();

    let mut tasks = Vec::new();
    for i in 0..10 {
        let bus_clone = bus.clone();
        let task = tokio::spawn(async move {
            let event = TestEvent {
                id: i,
                message: format!("concurrent {}", i),
            };
            bus_clone.publish(event).await
        });
        tasks.push(task);
    }

    for task in tasks {
        let result = task.await.unwrap();
        assert!(result.is_ok());
    }

    // All events should have been handled
    assert_eq!(call_count.load(Ordering::SeqCst), 10);
}