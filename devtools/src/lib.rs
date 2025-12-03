/*!
# Vela DevTools

Development tools for Vela including UI Inspector, Signal Graph visualizer,
performance profiler, and debugging utilities.
*/

pub mod inspector;
pub mod profiler;
pub mod debugger;
pub mod signal_graph;

/// Re-export main tools
pub use inspector::UIInspector;
pub use profiler::Profiler;
pub use debugger::Debugger;

/// Initialize DevTools
pub fn init() {
    println!("Vela DevTools initialized");
    // TODO: Initialize all dev tools
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_devtools_init() {
        init(); // Should not panic
    }
}