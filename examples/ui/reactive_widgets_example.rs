//! Reactive Widgets Example
//!
//! Demonstrates reactive widgets with automatic dependency tracking
//! and selective rebuilds based on signal changes.
//!
//! Implementation of: US-13 - TASK-058
//! Story: Reactive System Integration
//! Date: 2025-12-09

#[cfg(feature = "reactive")]
use vela_ui::{
    ReactiveWidget, ReactiveBuildContext,
    WidgetId, ReactiveWidgetTree, Text, Column, Row
};
#[cfg(feature = "reactive")]
use vela_reactive::{Signal, Computed};

#[cfg(feature = "reactive")]
/// Counter widget that reacts to signal changes
#[derive(Debug, Clone)]
struct ReactiveCounter {
    count_signal: Signal<i32>,
    widget_id: WidgetId,
}

#[cfg(feature = "reactive")]
impl ReactiveCounter {
    fn new(initial_count: i32) -> Self {
        Self {
            count_signal: Signal::new(initial_count),
            widget_id: WidgetId::new(),
        }
    }

    fn increment(&self) {
        self.count_signal.update(|count| *count + 1);
    }

    fn decrement(&self) {
        self.count_signal.update(|count| *count - 1);
    }
}

#[cfg(feature = "reactive")]
impl ReactiveWidget for ReactiveCounter {
    fn reactive_build(&self, context: &mut ReactiveBuildContext) -> vela_ui::VDomNode {
        // Set current widget ID
        let previous_id = context.current_widget_id.clone();
        context.current_widget_id = self.widget_id.clone();

        // Read signal with automatic dependency tracking
        let count = context.signal(&self.count_signal);

        // Create computed value for display text
        let display_text = format!("Count: {}", count);

        // Build UI
        let ui = Column::new(vec![
            Text::new(&display_text).into(),
            Row::new(vec![
                Button::new("Increment", || {
                    // In real implementation, this would trigger the increment
                    println!("Increment clicked");
                }).into(),
                Button::new("Decrement", || {
                    // In real implementation, this would trigger the decrement
                    println!("Decrement clicked");
                }).into(),
            ]).into(),
        ]);

        // Build the widget
        let node = ui.build(&context.base_context);

        // Restore previous widget ID
        context.current_widget_id = previous_id;

        node
    }

    fn widget_id(&self) -> WidgetId {
        self.widget_id.clone()
    }
}

#[cfg(feature = "reactive")]
/// Reactive text widget that displays computed values
#[derive(Debug, Clone)]
struct ReactiveText {
    text_signal: Signal<String>,
    widget_id: WidgetId,
}

#[cfg(feature = "reactive")]
impl ReactiveText {
    fn new(initial_text: String) -> Self {
        Self {
            text_signal: Signal::new(initial_text),
            widget_id: WidgetId::new(),
        }
    }

    fn update_text(&self, new_text: String) {
        self.text_signal.set(new_text);
    }
}

#[cfg(feature = "reactive")]
impl ReactiveWidget for ReactiveText {
    fn reactive_build(&self, context: &mut ReactiveBuildContext) -> vela_ui::VDomNode {
        let previous_id = context.current_widget_id.clone();
        context.current_widget_id = self.widget_id.clone();

        // Read signal with dependency tracking
        let text = context.signal(&self.text_signal);

        let node = Text::new(&text).build(&context.base_context);

        context.current_widget_id = previous_id;
        node
    }

    fn widget_id(&self) -> WidgetId {
        self.widget_id.clone()
    }
}

#[cfg(feature = "reactive")]
/// Example demonstrating reactive widget tree
pub fn reactive_widgets_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Reactive Widgets Example");
    println!("============================");

    // Create reactive counter
    let counter = ReactiveCounter::new(0);
    println!("Created reactive counter with initial value: 0");

    // Create reactive widget tree
    let mut widget_tree = ReactiveWidgetTree::new(counter.clone());
    println!("Created reactive widget tree");

    // Simulate reactive updates
    println!("\n📊 Simulating Reactive Updates:");
    println!("-------------------------------");

    // Initial state
    println!("Initial count: {}", counter.count_signal.get());

    // Increment
    counter.increment();
    println!("After increment: {}", counter.count_signal.get());

    // Trigger rebuild
    widget_tree.rebuild()?;
    println!("Widget tree rebuilt after increment");

    // Decrement
    counter.decrement();
    counter.decrement();
    println!("After two decrements: {}", counter.count_signal.get());

    // Trigger another rebuild
    widget_tree.rebuild()?;
    println!("Widget tree rebuilt after decrements");

    // Create reactive text example
    println!("\n📝 Reactive Text Example:");
    println!("------------------------");

    let reactive_text = ReactiveText::new("Hello, Vela!".to_string());
    println!("Created reactive text: '{}'", reactive_text.text_signal.get());

    reactive_text.update_text("Hello, Reactive World!".to_string());
    println!("Updated text to: '{}'", reactive_text.text_signal.get());

    // Create computed example
    println!("\n🧮 Computed Values Example:");
    println!("--------------------------");

    let base_count = Signal::new(5);
    let doubled = Computed::new(move || base_count.get() * 2);
    let description = Computed::new(move || {
        format!("Base: {}, Doubled: {}", base_count.get(), doubled.get())
    });

    println!("Computed: {}", description.get());

    base_count.set(10);
    println!("After updating base to 10: {}", description.get());

    base_count.set(25);
    println!("After updating base to 25: {}", description.get());

    println!("\n✅ Reactive widgets example completed successfully!");
    println!("Features demonstrated:");
    println!("  • Automatic dependency tracking");
    println!("  • Reactive widget rebuilding");
    println!("  • Signal-based state management");
    println!("  • Computed values");
    println!("  • Selective invalidation");

    Ok(())
}

#[cfg(not(feature = "reactive"))]
pub fn reactive_widgets_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  Reactive widgets example requires the 'reactive' feature to be enabled");
    println!("   Run with: cargo run --example reactive_widgets --features reactive");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "reactive")]
    #[test]
    fn test_reactive_counter_widget() {
        let counter = ReactiveCounter::new(5);
        assert_eq!(counter.count_signal.get(), 5);

        counter.increment();
        assert_eq!(counter.count_signal.get(), 6);

        counter.decrement();
        assert_eq!(counter.count_signal.get(), 5);
    }

    #[cfg(feature = "reactive")]
    #[test]
    fn test_reactive_text_widget() {
        let text_widget = ReactiveText::new("Initial".to_string());
        assert_eq!(text_widget.text_signal.get(), "Initial");

        text_widget.update_text("Updated".to_string());
        assert_eq!(text_widget.text_signal.get(), "Updated");
    }

    #[cfg(feature = "reactive")]
    #[test]
    fn test_reactive_widget_tree() {
        let counter = ReactiveCounter::new(0);
        let mut tree = ReactiveWidgetTree::new(counter.clone());

        // Initial build should work
        assert!(tree.rebuild().is_ok());

        // Update signal and rebuild
        counter.increment();
        assert!(tree.rebuild().is_ok());
    }

    #[cfg(feature = "reactive")]
    #[test]
    fn test_computed_values_integration() {
        let count = Signal::new(3);
        let doubled = Computed::new(move || count.get() * 2);

        assert_eq!(doubled.get(), 6);

        count.set(7);
        assert_eq!(doubled.get(), 14);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    reactive_widgets_example()
}