---
title: "ADR-054: Widget Base Class Architecture"
status: accepted
date: 2024-01-15
deciders: Vela Development Team
consulted: UI Framework Architects
---

## Context

TASK-054 requires implementing a Widget base class as an abstract class with lifecycle hooks. The current implementation in TASK-053 provides traits (`Widget`, `Lifecycle`) but lacks a concrete base class that developers can easily extend.

We need a base class that:
- Provides default implementations of lifecycle hooks
- Allows easy overriding of specific hooks
- Integrates with the existing trait system
- Maintains type safety and performance
- Follows Rust idioms while being developer-friendly

## Decision

We will implement a `BaseWidget` abstract class that:

1. **Implements the `Widget` trait** with a default `build` method that returns an empty VDOM node
2. **Implements the `Lifecycle` trait** with default no-op implementations for all hooks
3. **Provides protected methods** for subclasses to override lifecycle behavior
4. **Maintains composition over inheritance** where appropriate
5. **Uses Rust's trait system** for the core functionality while providing a class-like interface

### BaseWidget Structure

```rust
#[derive(Debug)]
pub struct BaseWidget {
    pub key: Option<Key>,
    lifecycle_state: LifecycleState,
}

impl BaseWidget {
    pub fn new() -> Self { ... }
    
    // Protected methods for subclasses
    pub fn on_mount(&mut self, _context: &BuildContext) { /* no-op */ }
    pub fn on_will_update(&mut self, _context: &BuildContext) { /* no-op */ }
    pub fn on_did_update(&mut self, _context: &BuildContext) { /* no-op */ }
    pub fn on_will_unmount(&mut self, _context: &BuildContext) { /* no-op */ }
    pub fn should_update(&self, _old_widget: &dyn Widget) -> bool { true }
}

impl Widget for BaseWidget {
    fn build(&self, context: &BuildContext) -> VDomNode {
        // Default implementation - subclasses should override
        VDomNode::empty()
    }
    
    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

impl Lifecycle for BaseWidget {
    fn mount(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Mounting;
        self.on_mount(context);
        self.lifecycle_state = LifecycleState::Mounted;
    }
    
    fn will_update(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Updating;
        self.on_will_update(context);
    }
    
    fn did_update(&mut self, context: &BuildContext) {
        self.on_did_update(context);
        self.lifecycle_state = LifecycleState::Mounted;
    }
    
    fn will_unmount(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Unmounting;
        self.on_will_unmount(context);
        self.lifecycle_state = LifecycleState::Unmounted;
    }
    
    fn should_update(&self, old_widget: &dyn Widget) -> bool {
        self.should_update(old_widget)
    }
}
```

### Usage Pattern

```rust
#[derive(Debug)]
pub struct MyWidget {
    base: BaseWidget,
    // Custom fields
    title: String,
}

impl MyWidget {
    pub fn new(title: String) -> Self {
        Self {
            base: BaseWidget::new(),
            title,
        }
    }
}

impl Widget for MyWidget {
    fn build(&self, context: &BuildContext) -> VDomNode {
        VDomNode::element("div")
            .with_child(VDomNode::text(&self.title))
    }
    
    fn key(&self) -> Option<Key> {
        self.base.key()
    }
}

impl Lifecycle for MyWidget {
    fn mount(&mut self, context: &BuildContext) {
        self.base.mount(context);
        // Custom mount logic
        println!("MyWidget mounted!");
    }
    
    fn on_will_update(&mut self, context: &BuildContext) {
        // Custom update logic
        println!("MyWidget will update!");
    }
}
```

## Consequences

### Positive
- **Developer-friendly**: Easy to extend and override specific lifecycle hooks
- **Type safety**: Maintains Rust's type system guarantees
- **Performance**: Zero-cost abstractions using traits
- **Flexibility**: Can still use composition when inheritance isn't appropriate
- **Consistency**: Follows established patterns from Flutter/React

### Negative
- **Complexity**: Adds another layer in the inheritance hierarchy
- **Rust idioms**: May feel less "Rusty" to some developers preferring pure traits
- **Memory**: Slightly more memory usage due to base struct

### Alternatives Considered

1. **Pure Traits Only** (Current TASK-053 approach)
   - ✅ Pure Rust, no inheritance
   - ❌ More boilerplate for common cases
   - ❌ Less intuitive for UI developers

2. **Full Inheritance Hierarchy**
   - ✅ Very developer-friendly
   - ❌ Goes against Rust's composition-over-inheritance philosophy
   - ❌ Potential for deep inheritance chains

3. **Macros for Code Generation**
   - ✅ Zero runtime cost
   - ❌ Complex macro system
   - ❌ Compile-time code generation complexity

## Implementation

The `BaseWidget` will be implemented in `runtime/ui/src/widget.rs` alongside the existing trait-based approach. Developers can choose between:
- Using `BaseWidget` for inheritance-based widgets
- Using traits directly for composition-based widgets
- Mixing both approaches as needed

## Testing

Comprehensive tests will cover:
- Base widget lifecycle transitions
- Subclass override behavior
- Integration with `LifecycleManager`
- Memory safety and performance
- Edge cases in lifecycle management

## References

- TASK-054: Implementar Widget base class, Clase abstracta con lifecycle hooks
- TASK-053: UI Framework Architecture (completed)
- Flutter StatefulWidget pattern
- React Component lifecycle</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-054-widget-base-class.md