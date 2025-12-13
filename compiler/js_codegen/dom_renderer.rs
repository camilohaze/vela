//! Vela DOM Renderer for JavaScript
//!
//! This file contains the DOM renderer that converts Vela widgets/VNodes
//! into real DOM elements for browser rendering.

/// Vela DOM Renderer JavaScript Code
pub const VELA_DOM_RENDERER_JS: &str = r#"
// Vela DOM Renderer for JavaScript
// Generated automatically - do not modify

const vela = vela || {};

// ===== DOM RENDERER =====

/**
 * DOM Renderer for Vela widgets
 * Converts VNodes/widgets to real DOM elements
 */
vela.DOMRenderer = class {
  constructor() {
    this._mountedElements = new WeakMap();
    this._eventListeners = new WeakMap();
    this._componentInstances = new WeakMap();
  }

  /**
   * Render a Vela widget/VNode to DOM element
   * @param {Object} vnode - Vela VNode or widget
   * @param {HTMLElement} container - Container element (optional)
   * @returns {HTMLElement|Text|Comment|DocumentFragment}
   */
  render(vnode, container = null) {
    if (!vnode) return null;

    // Handle different VNode types
    switch (vnode.type) {
      case 'element':
        return this._renderElement(vnode);
      case 'text':
        return this._renderText(vnode);
      case 'fragment':
        return this._renderFragment(vnode);
      case 'comment':
        return this._renderComment(vnode);
      case 'component':
        return this._renderComponent(vnode);
      default:
        throw new Error(`Unknown VNode type: ${vnode.type}`);
    }
  }

  /**
   * Mount rendered element to DOM
   * @param {HTMLElement} element - Rendered element
   * @param {HTMLElement} container - DOM container
   */
  mount(element, container) {
    if (!element || !container) return;

    container.appendChild(element);
    this._mountedElements.set(element, container);

    // Trigger mount lifecycle for components
    this._triggerLifecycle(element, 'mount');
  }

  /**
   * Update a mounted element with new VNode
   * @param {HTMLElement} element - Current DOM element
   * @param {Object} newVNode - New VNode
   */
  update(element, newVNode) {
    if (!element || !newVNode) return;

    const oldVNode = this._getVNodeFromElement(element);
    if (!oldVNode) return;

    // Diff and patch
    const patches = this._diff(oldVNode, newVNode);
    this._applyPatches(element, patches);

    // Update stored VNode
    this._setVNodeOnElement(element, newVNode);

    // Trigger update lifecycle
    this._triggerLifecycle(element, 'update');
  }

  /**
   * Unmount element from DOM
   * @param {HTMLElement} element - Element to unmount
   */
  unmount(element) {
    if (!element) return;

    const container = this._mountedElements.get(element);
    if (container && container.contains(element)) {
      container.removeChild(element);
    }

    // Cleanup event listeners
    this._cleanupEventListeners(element);

    // Trigger destroy lifecycle
    this._triggerLifecycle(element, 'destroy');

    // Cleanup references
    this._mountedElements.delete(element);
    this._componentInstances.delete(element);
  }

  /**
   * Render HTML element
   * @private
   */
  _renderElement(vnode) {
    const { tag, props = {}, children = [] } = vnode;

    // Create element
    let element;
    if (tag === 'svg' || this._isSVGElement(tag)) {
      element = document.createElementNS('http://www.w3.org/2000/svg', tag);
    } else {
      element = document.createElement(tag);
    }

    // Store VNode reference
    this._setVNodeOnElement(element, vnode);

    // Apply props
    this._applyProps(element, props);

    // Render and append children
    children.forEach(child => {
      const childElement = this.render(child);
      if (childElement) {
        element.appendChild(childElement);
      }
    });

    return element;
  }

  /**
   * Render text node
   * @private
   */
  _renderText(vnode) {
    const textNode = document.createTextNode(vnode.text || '');
    this._setVNodeOnElement(textNode, vnode);
    return textNode;
  }

  /**
   * Render fragment
   * @private
   */
  _renderFragment(vnode) {
    const fragment = document.createDocumentFragment();

    (vnode.children || []).forEach(child => {
      const childElement = this.render(child);
      if (childElement) {
        fragment.appendChild(childElement);
      }
    });

    this._setVNodeOnElement(fragment, vnode);
    return fragment;
  }

  /**
   * Render comment node
   * @private
   */
  _renderComment(vnode) {
    const comment = document.createComment(vnode.text || '');
    this._setVNodeOnElement(comment, vnode);
    return comment;
  }

  /**
   * Render Vela component
   * @private
   */
  _renderComponent(vnode) {
    const { component, props = {}, children = [] } = vnode;

    // Create component instance
    const instance = new component.constructor(props);

    // Store instance reference
    const element = this._renderElement(instance.render());
    this._componentInstances.set(element, instance);

    return element;
  }

  /**
   * Apply props to DOM element
   * @private
   */
  _applyProps(element, props) {
    Object.keys(props).forEach(key => {
      const value = props[key];

      if (key === 'key' || key === 'ref') {
        // Special Vela props, skip
        return;
      }

      if (key === 'className') {
        // className -> class
        element.setAttribute('class', value);
      } else if (key === 'htmlFor') {
        // htmlFor -> for
        element.setAttribute('for', value);
      } else if (key.startsWith('on') && typeof value === 'function') {
        // Event handler
        this._addEventListener(element, key.slice(2).toLowerCase(), value);
      } else if (key === 'style' && typeof value === 'object') {
        // Style object
        this._applyStyles(element, value);
      } else if (key === 'dangerouslySetInnerHTML') {
        // Raw HTML
        element.innerHTML = value.__html || '';
      } else if (typeof value === 'boolean') {
        // Boolean attributes
        if (value) {
          element.setAttribute(key, '');
        } else {
          element.removeAttribute(key);
        }
      } else {
        // Regular attributes
        element.setAttribute(key, String(value));
      }
    });
  }

  /**
   * Apply styles object to element
   * @private
   */
  _applyStyles(element, styles) {
    Object.keys(styles).forEach(prop => {
      const value = styles[prop];
      const cssProp = prop.replace(/([A-Z])/g, '-$1').toLowerCase();
      element.style.setProperty(cssProp, value);
    });
  }

  /**
   * Add event listener to element
   * @private
   */
  _addEventListener(element, eventType, handler) {
    const listeners = this._eventListeners.get(element) || [];
    const eventListener = (e) => handler(e);

    element.addEventListener(eventType, eventListener);
    listeners.push({ eventType, handler: eventListener });
    this._eventListeners.set(element, listeners);
  }

  /**
   * Cleanup event listeners for element
   * @private
   */
  _cleanupEventListeners(element) {
    const listeners = this._eventListeners.get(element);
    if (listeners) {
      listeners.forEach(({ eventType, handler }) => {
        element.removeEventListener(eventType, handler);
      });
      this._eventListeners.delete(element);
    }
  }

  /**
   * Check if element is SVG
   * @private
   */
  _isSVGElement(tag) {
    const svgElements = new Set([
      'svg', 'circle', 'rect', 'line', 'path', 'text', 'g', 'defs',
      'linearGradient', 'radialGradient', 'stop', 'polygon', 'polyline'
    ]);
    return svgElements.has(tag);
  }

  /**
   * Store VNode reference on DOM element
   * @private
   */
  _setVNodeOnElement(element, vnode) {
    element._velaVNode = vnode;
  }

  /**
   * Get VNode from DOM element
   * @private
   */
  _getVNodeFromElement(element) {
    return element._velaVNode;
  }

  /**
   * Trigger lifecycle method on component
   * @private
   */
  _triggerLifecycle(element, method) {
    const instance = this._componentInstances.get(element);
    if (instance && typeof instance[method] === 'function') {
      instance[method]();
    }
  }

  /**
   * Simple diff algorithm for updates
   * @private
   */
  _diff(oldVNode, newVNode) {
    // Basic diff implementation
    // In a full implementation, this would be much more sophisticated
    const patches = [];

    if (!oldVNode || !newVNode) return patches;

    if (oldVNode.type !== newVNode.type) {
      patches.push({ type: 'REPLACE', oldVNode, newVNode });
      return patches;
    }

    // Check props changes
    if (this._propsChanged(oldVNode.props, newVNode.props)) {
      patches.push({ type: 'UPDATE_PROPS', element: null, props: newVNode.props });
    }

    // Check children changes
    if (oldVNode.children && newVNode.children) {
      const childrenPatches = this._diffChildren(oldVNode.children, newVNode.children);
      patches.push(...childrenPatches);
    }

    return patches;
  }

  /**
   * Check if props changed
   * @private
   */
  _propsChanged(oldProps = {}, newProps = {}) {
    const allKeys = new Set([...Object.keys(oldProps), ...Object.keys(newProps)]);

    for (const key of allKeys) {
      if (oldProps[key] !== newProps[key]) {
        return true;
      }
    }

    return false;
  }

  /**
   * Diff children
   * @private
   */
  _diffChildren(oldChildren = [], newChildren = []) {
    const patches = [];

    const maxLen = Math.max(oldChildren.length, newChildren.length);

    for (let i = 0; i < maxLen; i++) {
      const oldChild = oldChildren[i];
      const newChild = newChildren[i];

      if (!oldChild && newChild) {
        patches.push({ type: 'INSERT', index: i, vnode: newChild });
      } else if (oldChild && !newChild) {
        patches.push({ type: 'REMOVE', index: i });
      } else if (oldChild && newChild) {
        const childPatches = this._diff(oldChild, newChild);
        if (childPatches.length > 0) {
          patches.push({ type: 'UPDATE', index: i, patches: childPatches });
        }
      }
    }

    return patches;
  }

  /**
   * Apply patches to DOM
   * @private
   */
  _applyPatches(element, patches) {
    patches.forEach(patch => {
      switch (patch.type) {
        case 'REPLACE':
          const newElement = this.render(patch.newVNode);
          element.parentNode.replaceChild(newElement, element);
          break;
        case 'UPDATE_PROPS':
          this._applyProps(element, patch.props);
          break;
        case 'INSERT':
          const childElement = this.render(patch.vnode);
          const referenceNode = element.children[patch.index];
          element.insertBefore(childElement, referenceNode);
          break;
        case 'REMOVE':
          element.removeChild(element.children[patch.index]);
          break;
        case 'UPDATE':
          const child = element.children[patch.index];
          this._applyPatches(child, patch.patches);
          break;
      }
    });
  }
};

// ===== VELA WIDGETS TO VNODE CONVERTERS =====

/**
 * Convert Vela widget to VNode
 */
vela.widgetToVNode = function(widget) {
  if (!widget) return null;

  // Handle different widget types
  if (widget.type === 'container') {
    return {
      type: 'element',
      tag: 'div',
      props: {
        className: widget.props?.className || '',
        style: widget.props?.style || {}
      },
      children: (widget.children || []).map(vela.widgetToVNode)
    };
  }

  if (widget.type === 'text') {
    return {
      type: 'text',
      text: widget.props?.text || ''
    };
  }

  if (widget.type === 'button') {
    return {
      type: 'element',
      tag: 'button',
      props: {
        className: widget.props?.className || '',
        onClick: widget.props?.onPressed,
        disabled: widget.props?.disabled || false
      },
      children: [{
        type: 'text',
        text: widget.props?.text || ''
      }]
    };
  }

  // Add more widget converters as needed

  return null;
};

// ===== REACTIVITY INTEGRATION =====

/**
 * Reactive renderer that auto-updates on signal changes
 */
vela.ReactiveRenderer = class extends vela.DOMRenderer {
  constructor() {
    super();
    this._reactiveElements = new WeakMap();
  }

  /**
   * Render reactive widget that auto-updates
   */
  renderReactive(widget, container) {
    const element = this.render(widget, container);

    // Create effect to watch for changes
    const effect = vela.effect(() => {
      const newVNode = vela.widgetToVNode(widget);
      this.update(element, newVNode);
    });

    this._reactiveElements.set(element, effect);

    return element;
  }

  /**
   * Unmount reactive element
   */
  unmount(element) {
    // Cleanup reactive effect
    const effect = this._reactiveElements.get(element);
    if (effect && typeof effect.destroy === 'function') {
      effect.destroy();
    }

    super.unmount(element);
  }
};

// Export for global use
if (typeof module !== 'undefined' && module.exports) {
  module.exports = vela;
} else if (typeof window !== 'undefined') {
  window.vela = vela;
}
"#;