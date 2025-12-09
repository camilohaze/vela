//! # Vela UI Framework
//!
//! Declarative UI framework with Virtual DOM and reactive widgets.
//!
//! This crate provides the core UI framework for Vela, featuring:
//! - Declarative widget composition
//! - Virtual DOM with efficient diffing
//! - Reactive integration with signals
//! - Lifecycle hooks
//! - Component-based architecture

pub mod widget;
pub mod vdom;
pub mod diff;
pub mod patch;
pub mod context;
pub mod lifecycle;
pub mod key;
pub mod layout;
pub mod input_widgets;

pub use widget::{Widget, StatelessWidget, StatefulWidget, Container, Row, Column, Stack, PositionedChild, Button, ButtonVariant, TextField, Checkbox};
pub use layout::{
    BoxConstraints, Size, Offset, EdgeInsets, Alignment,
    MainAxisAlignment, CrossAxisAlignment, MainAxisSize,
    Position, StackFit
};
pub use vdom::{VDomNode, VDomTree};
pub use diff::{diff_trees, Patch};
pub use patch::{apply_patches};
pub use context::BuildContext;
pub use key::Key;

/// Initialize the UI framework
pub fn init() {
    // Initialize web-sys bindings for WASM
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).unwrap();
    }
}

/// Run the UI framework with a root widget
pub async fn run<W: Widget + 'static>(root_widget: W) -> Result<(), Box<dyn std::error::Error>> {
    init();

    let mut vdom_tree = VDomTree::new(root_widget);
    let mut renderer = Renderer::new();

    // Initial render
    renderer.render(&vdom_tree)?;

    // Main render loop - in real implementation this would be event-driven
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // ~60 FPS

        // Check for reactive updates
        if vdom_tree.needs_update() {
            let new_tree = vdom_tree.rebuild()?;
            let patches = diff_trees(&vdom_tree, &new_tree);
            renderer.apply_patches(patches)?;
            vdom_tree = new_tree;
        }
    }
}

/// Internal renderer for DOM manipulation
struct Renderer {
    // In WASM this would hold web-sys references
}

impl Renderer {
    fn new() -> Self {
        Self {}
    }

    fn render(&mut self, _tree: &VDomTree) -> Result<(), Box<dyn std::error::Error>> {
        // Initial DOM rendering logic
        Ok(())
    }

    fn apply_patches(&mut self, _patches: Vec<Patch>) -> Result<(), Box<dyn std::error::Error>> {
        // Apply patches to real DOM
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ui_framework_init() {
        init();
        // Basic initialization test
    }
}