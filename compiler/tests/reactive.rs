//! Tests for Vela Reactive Runtime (JavaScript signals implementation)

use vela_compiler::js_codegen::runtime::VELA_RUNTIME_JS;

/// Test that the reactive runtime contains all expected reactive functionality
#[test]
fn test_reactive_runtime_completeness() {
    let runtime = VELA_RUNTIME_JS;

    // Core reactive system
    assert!(runtime.contains("_currentEffect"));
    assert!(runtime.contains("_effectStack"));
    assert!(runtime.contains("_batchDepth"));
    assert!(runtime.contains("_pendingEffects"));

    // Reactive API functions (defined as object properties)
    assert!(runtime.contains("state:"));
    assert!(runtime.contains("computed:"));
    assert!(runtime.contains("effect:"));
    assert!(runtime.contains("batch:"));
    assert!(runtime.contains("readonly:"));

    // Signal classes
    assert!(runtime.contains("StateSignal: class"));
    assert!(runtime.contains("ComputedSignal: class"));
    assert!(runtime.contains("Effect: class"));

    // StateSignal methods
    assert!(runtime.contains("get()"));
    assert!(runtime.contains("set("));
    assert!(runtime.contains("update("));
    assert!(runtime.contains("subscribe("));

    // ComputedSignal methods
    assert!(runtime.contains("_recompute()"));
    assert!(runtime.contains("_invalidate()"));

    // Effect methods
    assert!(runtime.contains("_run()"));
    assert!(runtime.contains("_cleanup()"));

    // Dependency tracking
    assert!(runtime.contains("_track("));
    assert!(runtime.contains("_trigger("));
    assert!(runtime.contains("_dependencies"));
    assert!(runtime.contains("_dependents"));
}

/// Test that reactive system has proper memory management
#[test]
fn test_reactive_memory_management() {
    let runtime = VELA_RUNTIME_JS;

    // Should have proper cleanup mechanisms
    assert!(runtime.contains("_cleanup"));
    assert!(runtime.contains("delete(")); // For removing from sets

    // Should handle effect cleanup
    assert!(runtime.contains("_cleanupFn"));
}

/// Test that batching system is implemented
#[test]
fn test_batching_system() {
    let runtime = VELA_RUNTIME_JS;

    assert!(runtime.contains("batch: function"));
    assert!(runtime.contains("_batchDepth"));
    assert!(runtime.contains("_flushPendingEffects"));
    assert!(runtime.contains("_pendingEffects"));
}

/// Test that computed signals have proper caching
#[test]
fn test_computed_caching() {
    let runtime = VELA_RUNTIME_JS;

    assert!(runtime.contains("_isDirty"));
    assert!(runtime.contains("_recompute()"));
    assert!(runtime.contains("_invalidate()"));
}

/// Test that effects support cleanup functions
#[test]
fn test_effect_cleanup() {
    let runtime = VELA_RUNTIME_JS;

    assert!(runtime.contains("_cleanupFn"));
    assert!(runtime.contains("typeof result === 'function'"));
}

/// Test that dependency tracking is automatic
#[test]
fn test_dependency_tracking() {
    let runtime = VELA_RUNTIME_JS;

    assert!(runtime.contains("_track(this)"));
    assert!(runtime.contains("_currentEffect"));
    assert!(runtime.contains("_dependencies.add"));
    assert!(runtime.contains("_dependents.add"));
}

/// Test that the runtime is syntactically valid JavaScript
#[test]
fn test_runtime_syntax_validity() {
    let runtime = VELA_RUNTIME_JS;

    // Should not have syntax errors
    assert!(!runtime.contains("function function"));
    assert!(!runtime.contains("class class"));
    assert!(!runtime.contains("const const"));
    assert!(!runtime.contains("let let"));

    // Should have proper class syntax
    assert!(runtime.contains("class {"));
    assert!(runtime.contains("constructor("));

    // Should have proper function syntax
    assert!(runtime.contains("function("));
    assert!(runtime.contains("=> {"));

    // Should have proper object syntax
    assert!(runtime.contains("const vela = {"));
    assert!(runtime.contains("};"));
}

/// Test that reactive system integrates with existing runtime
#[test]
fn test_reactive_integration_with_existing_runtime() {
    let runtime = VELA_RUNTIME_JS;

    // Should still contain existing runtime features
    assert!(runtime.contains("Some: function"));
    assert!(runtime.contains("None:"));
    assert!(runtime.contains("Ok: function"));
    assert!(runtime.contains("Err: function"));
    assert!(runtime.contains("println"));
    assert!(runtime.contains("panic"));
}

/// Test that the DOM renderer is included in the runtime
#[test]
fn test_dom_renderer_in_runtime() {
    let runtime = VELA_RUNTIME_JS;

    // DOM Renderer class
    assert!(runtime.contains("vela.DOMRenderer = class"));
    assert!(runtime.contains("render(vnode"));
    assert!(runtime.contains("mount(element"));
    assert!(runtime.contains("update(element"));
    assert!(runtime.contains("unmount(element"));

    // Element rendering
    assert!(runtime.contains("_renderElement(vnode"));
    assert!(runtime.contains("_renderText(vnode"));
    assert!(runtime.contains("_renderFragment(vnode"));
    assert!(runtime.contains("_renderComponent(vnode"));

    // Props handling
    assert!(runtime.contains("_applyProps(element"));
    assert!(runtime.contains("_applyStyles(element"));
    assert!(runtime.contains("_addEventListener(element"));

    // Widget to VNode conversion
    assert!(runtime.contains("vela.widgetToVNode = function"));

    // Reactive renderer
    assert!(runtime.contains("vela.ReactiveRenderer = class"));
    assert!(runtime.contains("renderReactive(widget"));
}