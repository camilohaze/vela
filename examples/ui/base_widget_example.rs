//! Example usage of BaseWidget

use vela_ui::widget::{BaseWidget, Widget};
use vela_ui::vdom::VDomNode;
use vela_ui::context::BuildContext;
use vela_ui::lifecycle::{Lifecycle, LifecycleState};
use vela_ui::key::Key;

/// Example widget using BaseWidget inheritance
#[derive(Debug)]
pub struct CounterWidget {
    base: BaseWidget,
    count: i32,
    title: String,
}

impl CounterWidget {
    /// Create a new counter widget
    pub fn new(title: String) -> Self {
        Self {
            base: BaseWidget::new(),
            count: 0,
            title,
        }
    }

    /// Create with key
    pub fn with_key(title: String, key: Key) -> Self {
        Self {
            base: BaseWidget::with_key(key),
            count: 0,
            title,
        }
    }

    /// Increment counter
    pub fn increment(&mut self) {
        self.count += 1;
    }

    /// Get current count
    pub fn count(&self) -> i32 {
        self.count
    }
}

impl Widget for CounterWidget {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        VDomNode::element("div")
            .with_attribute("class", "counter-widget")
            .with_child(
                VDomNode::element("h2")
                    .with_child(VDomNode::text(&self.title))
            )
            .with_child(
                VDomNode::element("p")
                    .with_child(VDomNode::text(&format!("Count: {}", self.count)))
            )
            .with_child(
                VDomNode::element("button")
                    .with_attribute("type", "button")
                    .with_child(VDomNode::text("Increment"))
            )
    }

    fn key(&self) -> Option<Key> {
        self.base.key()
    }
}

impl Lifecycle for CounterWidget {
    fn on_mount(&mut self, _context: &BuildContext) {
        println!("CounterWidget '{}' mounted with initial count: {}", self.title, self.count);
    }

    fn on_will_update(&mut self, _context: &BuildContext) {
        println!("CounterWidget '{}' will update (current count: {})", self.title, self.count);
    }

    fn on_did_update(&mut self, _context: &BuildContext) {
        println!("CounterWidget '{}' updated (new count: {})", self.title, self.count);
    }

    fn on_will_unmount(&mut self, _context: &BuildContext) {
        println!("CounterWidget '{}' will unmount (final count: {})", self.title, self.count);
    }

    fn should_update(&self, old_widget: &dyn Widget) -> bool {
        // Only update if count changed
        if let Some(old_counter) = old_widget.as_any().downcast_ref::<CounterWidget>() {
            old_counter.count != self.count
        } else {
            true // Different widget type, update
        }
    }
}

impl AsRef<dyn Widget> for CounterWidget {
    fn as_ref(&self) -> &(dyn Widget + 'static) {
        self
    }
}

impl AsMut<dyn Widget> for CounterWidget {
    fn as_mut(&mut self) -> &mut (dyn Widget + 'static) {
        self
    }
}

impl std::any::Any for CounterWidget {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<CounterWidget>()
    }
}

/// Example widget demonstrating composition over inheritance
#[derive(Debug)]
pub struct DashboardWidget {
    base: BaseWidget,
    counters: Vec<CounterWidget>,
}

impl DashboardWidget {
    pub fn new() -> Self {
        Self {
            base: BaseWidget::new(),
            counters: vec![
                CounterWidget::with_key("Users".to_string(), Key::String("users-counter".to_string())),
                CounterWidget::with_key("Orders".to_string(), Key::String("orders-counter".to_string())),
                CounterWidget::with_key("Revenue".to_string(), Key::String("revenue-counter".to_string())),
            ],
        }
    }

    pub fn increment_counter(&mut self, index: usize) {
        if let Some(counter) = self.counters.get_mut(index) {
            counter.increment();
        }
    }
}

impl Widget for DashboardWidget {
    fn build(&self, context: &BuildContext) -> VDomNode {
        let mut container = VDomNode::element("div")
            .with_attribute("class", "dashboard");

        // Add title
        container.children.push(
            VDomNode::element("h1")
                .with_child(VDomNode::text("Dashboard"))
        );

        // Add counters
        for counter in &self.counters {
            container.children.push(counter.build(context));
        }

        container
    }

    fn key(&self) -> Option<Key> {
        self.base.key()
    }
}

impl Lifecycle for DashboardWidget {
    fn on_mount(&mut self, context: &BuildContext) {
        println!("DashboardWidget mounted");
        // Mount all child widgets
        for counter in &mut self.counters {
            counter.base.mount(context);
        }
    }

    fn on_will_update(&mut self, context: &BuildContext) {
        println!("DashboardWidget will update");
    }

    fn on_did_update(&mut self, context: &BuildContext) {
        println!("DashboardWidget updated");
    }

    fn on_will_unmount(&mut self, context: &BuildContext) {
        println!("DashboardWidget will unmount");
        // Unmount all child widgets
        for counter in &mut self.counters {
            counter.base.will_unmount(context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_widget_creation() {
        let counter = CounterWidget::new("Test Counter".to_string());
        assert_eq!(counter.count(), 0);
        assert_eq!(counter.base.lifecycle_state(), LifecycleState::Unmounted);
    }

    #[test]
    fn test_counter_widget_with_key() {
        let key = Key::String("my-counter".to_string());
        let counter = CounterWidget::with_key("Test Counter".to_string(), key.clone());
        assert_eq!(counter.key(), Some(key));
    }

    #[test]
    fn test_counter_widget_increment() {
        let mut counter = CounterWidget::new("Test Counter".to_string());
        assert_eq!(counter.count(), 0);

        counter.increment();
        assert_eq!(counter.count(), 1);

        counter.increment();
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_counter_widget_build() {
        let counter = CounterWidget::new("My Counter".to_string());
        let context = BuildContext::new();
        let node = counter.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert!(node.attributes.contains_key("class"));
        assert_eq!(node.children.len(), 3); // h2, p, button
    }

    #[test]
    fn test_counter_widget_lifecycle() {
        let mut counter = CounterWidget::new("Test Counter".to_string());
        let context = BuildContext::new();

        // Test mount
        counter.base.mount(&context);
        assert_eq!(counter.base.lifecycle_state(), LifecycleState::Mounted);

        // Test update cycle
        counter.base.will_update(&context);
        assert_eq!(counter.base.lifecycle_state(), LifecycleState::Updating);

        counter.base.did_update(&context);
        assert_eq!(counter.base.lifecycle_state(), LifecycleState::Mounted);

        // Test unmount
        counter.base.will_unmount(&context);
        assert_eq!(counter.base.lifecycle_state(), LifecycleState::Unmounting);
    }

    #[test]
    fn test_counter_widget_should_update() {
        let mut counter1 = CounterWidget::new("Counter".to_string());
        let counter2 = CounterWidget::new("Counter".to_string());

        // Same count, shouldn't update
        counter1.count = 5;
        counter2.count = 5;
        assert!(!counter1.should_update(&counter2));

        // Different count, should update
        counter2.count = 10;
        assert!(counter1.should_update(&counter2));
    }

    #[test]
    fn test_dashboard_widget() {
        let dashboard = DashboardWidget::new();
        assert_eq!(dashboard.counters.len(), 3);

        let context = BuildContext::new();
        let node = dashboard.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.children.len(), 4); // h1 + 3 counters
    }

    #[test]
    fn test_dashboard_widget_increment() {
        let mut dashboard = DashboardWidget::new();

        // Initial state
        assert_eq!(dashboard.counters[0].count(), 0);

        // Increment first counter
        dashboard.increment_counter(0);
        assert_eq!(dashboard.counters[0].count(), 1);
        assert_eq!(dashboard.counters[1].count(), 0); // Others unchanged
    }
}</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\examples\ui\base_widget_example.rs