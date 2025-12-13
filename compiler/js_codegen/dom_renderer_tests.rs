//! Tests for DOM Renderer functionality

use super::runtime::VELA_RUNTIME_JS;

/// Test that DOM renderer classes are present
#[test]
fn test_dom_renderer_classes() {
    let runtime = VELA_RUNTIME_JS;

    // DOMRenderer class
    assert!(runtime.contains("vela.DOMRenderer = class"));
    assert!(runtime.contains("constructor()"));
    assert!(runtime.contains("this._mountedElements"));
    assert!(runtime.contains("this._eventListeners"));
    assert!(runtime.contains("this._componentInstances"));
}

/// Test DOM renderer methods
#[test]
fn test_dom_renderer_methods() {
    let runtime = VELA_RUNTIME_JS;

    // Core rendering methods
    assert!(runtime.contains("render(vnode"));
    assert!(runtime.contains("mount(element"));
    assert!(runtime.contains("update(element"));
    assert!(runtime.contains("unmount(element"));

    // Private rendering methods
    assert!(runtime.contains("_renderElement(vnode"));
    assert!(runtime.contains("_renderText(vnode"));
    assert!(runtime.contains("_renderFragment(vnode"));
    assert!(runtime.contains("_renderComment(vnode"));
    assert!(runtime.contains("_renderComponent(vnode"));
}

/// Test props handling
#[test]
fn test_props_handling() {
    let runtime = VELA_RUNTIME_JS;

    // Props application
    assert!(runtime.contains("_applyProps(element"));
    assert!(runtime.contains("_applyStyles(element"));
    assert!(runtime.contains("_addEventListener(element"));
    assert!(runtime.contains("_cleanupEventListeners(element"));

    // Special prop handling
    assert!(runtime.contains("className"));
    assert!(runtime.contains("htmlFor"));
    assert!(runtime.contains("onClick"));
    assert!(runtime.contains("dangerouslySetInnerHTML"));
}

/// Test widget to VNode conversion
#[test]
fn test_widget_to_vnode_conversion() {
    let runtime = VELA_RUNTIME_JS;

    // Widget converters
    assert!(runtime.contains("vela.widgetToVNode = function"));
    assert!(runtime.contains("widget.type === 'container'"));
    assert!(runtime.contains("widget.type === 'text'"));
    assert!(runtime.contains("widget.type === 'button'"));

    // VNode structure
    assert!(runtime.contains("type: 'element'"));
    assert!(runtime.contains("type: 'text'"));
    assert!(runtime.contains("tag: 'div'"));
    assert!(runtime.contains("tag: 'button'"));
}

/// Test reactive renderer
#[test]
fn test_reactive_renderer() {
    let runtime = VELA_RUNTIME_JS;

    // ReactiveRenderer class
    assert!(runtime.contains("vela.ReactiveRenderer = class"));
    assert!(runtime.contains("extends vela.DOMRenderer"));
    assert!(runtime.contains("renderReactive(widget"));
    assert!(runtime.contains("this._reactiveElements"));

    // Reactivity integration
    assert!(runtime.contains("vela.effect(() => {"));
    assert!(runtime.contains("vela.widgetToVNode(widget)"));
}

/// Test diffing and patching
#[test]
fn test_diffing_and_patching() {
    let runtime = VELA_RUNTIME_JS;

    // Diff algorithm
    assert!(runtime.contains("_diff(oldVNode"));
    assert!(runtime.contains("_propsChanged(oldProps"));
    assert!(runtime.contains("_diffChildren(oldChildren"));

    // Patch types
    assert!(runtime.contains("REPLACE"));
    assert!(runtime.contains("UPDATE_PROPS"));
    assert!(runtime.contains("INSERT"));
    assert!(runtime.contains("REMOVE"));
    assert!(runtime.contains("UPDATE"));

    // Patch application
    assert!(runtime.contains("_applyPatches(element"));
}

/// Test SVG support
#[test]
fn test_svg_support() {
    let runtime = VELA_RUNTIME_JS;

    // SVG detection
    assert!(runtime.contains("_isSVGElement(tag"));
    assert!(runtime.contains("createElementNS"));
    assert!(runtime.contains("'http://www.w3.org/2000/svg'"));

    // SVG elements
    assert!(runtime.contains("'svg', 'circle', 'rect'"));
}

/// Test lifecycle management
#[test]
fn test_lifecycle_management() {
    let runtime = VELA_RUNTIME_JS;

    // Lifecycle triggers
    assert!(runtime.contains("_triggerLifecycle(element"));
    assert!(runtime.contains("'mount'"));
    assert!(runtime.contains("'update'"));
    assert!(runtime.contains("'destroy'"));

    // Component instances
    assert!(runtime.contains("this._componentInstances"));
    assert!(runtime.contains("instance.render()"));
}

/// Test memory management
#[test]
fn test_memory_management() {
    let runtime = VELA_RUNTIME_JS;

    // WeakMap usage
    assert!(runtime.contains("new WeakMap()"));
    assert!(runtime.contains("this._mountedElements"));
    assert!(runtime.contains("this._eventListeners"));

    // Cleanup methods
    assert!(runtime.contains("_cleanupEventListeners(element"));
    assert!(runtime.contains("_mountedElements.delete(element"));
    assert!(runtime.contains("_componentInstances.delete(element"));
}

/// Test VNode storage
#[test]
fn test_vnode_storage() {
    let runtime = VELA_RUNTIME_JS;

    // VNode storage/retrieval
    assert!(runtime.contains("_setVNodeOnElement(element"));
    assert!(runtime.contains("_getVNodeFromElement(element"));
    assert!(runtime.contains("element._velaVNode"));
}