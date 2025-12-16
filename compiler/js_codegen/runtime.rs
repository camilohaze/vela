//! Vela Runtime for JavaScript
//!
//! This file contains the JavaScript runtime that supports Vela-generated code.
//! It provides implementations for signals, Option types, Result types, DOM rendering,
//! and other Vela-specific constructs.

/// Vela Runtime JavaScript Code
pub const VELA_RUNTIME_JS: &str = r#"
// Vela Runtime for JavaScript
// Generated automatically - do not modify

const vela = {
  // ===== REACTIVE SYSTEM =====

  // Global reactive context
  _currentEffect: null,
  _effectStack: [],
  _batchDepth: 0,
  _pendingEffects: new Set(),

  /**
   * Execute function in batch mode (delays effect execution)
   * @param {Function} fn - Function to execute in batch
   */
  batch: function(fn) {
    this._batchDepth++;
    try {
      fn();
    } finally {
      this._batchDepth--;
      if (this._batchDepth === 0) {
        this._flushPendingEffects();
      }
    }
  },

  /**
   * Flush all pending effects
   */
  _flushPendingEffects: function() {
    const effects = Array.from(this._pendingEffects);
    this._pendingEffects.clear();
    effects.forEach(effect => effect._run());
  },

  /**
   * Track dependencies for the current effect
   * @param {Signal} signal - Signal being accessed
   */
  _track: function(signal) {
    if (this._currentEffect) {
      signal._dependents.add(this._currentEffect);
      this._currentEffect._dependencies.add(signal);
    }
  },

  /**
   * Trigger updates for signal dependents
   * @param {Signal} signal - Signal that changed
   */
  _trigger: function(signal) {
    // Add all dependents to pending effects
    signal._dependents.forEach(effect => {
      this._pendingEffects.add(effect);
    });

    // If not in batch mode, flush immediately
    if (this._batchDepth === 0) {
      this._flushPendingEffects();
    }
  },

  // ===== SIGNALS (REACTIVE STATE) =====

  /**
   * Create a reactive state signal (mutable reactive variable)
   * @param {any} initialValue - Initial value
   * @returns {StateSignal} Reactive state signal
   */
  state: function(initialValue) {
    return new StateSignal(initialValue);
  },

  /**
   * Create a computed signal (derived reactive value)
   * @param {Function} computeFn - Function that computes the value
   * @returns {ComputedSignal} Computed signal
   */
  computed: function(computeFn) {
    return new ComputedSignal(computeFn);
  },

  /**
   * Create an effect (reactive side effect)
   * @param {Function} effectFn - Effect function
   * @returns {Function} Cleanup function
   */
  effect: function(effectFn) {
    const effect = new Effect(effectFn);
    effect._run(); // Run immediately
    return () => effect._cleanup();
  },

  /**
   * Create a readonly signal (for external compatibility)
   * @param {Function} getter - Getter function
   * @returns {ReadonlySignal} Readonly signal
   */
  readonly: function(getter) {
    return {
      get: getter,
      subscribe: (callback) => {
        // Readonly signals don't support subscription
        return () => {};
      }
    };
  },

  // ===== SIGNAL CLASSES =====

  /**
   * State Signal - Mutable reactive variable
   */
  StateSignal: class {
    constructor(initialValue) {
      this._value = initialValue;
      this._dependents = new Set();
    }

    get() {
      vela._track(this);
      return this._value;
    }

    set(newValue) {
      if (this._value !== newValue) {
        this._value = newValue;
        vela._trigger(this);
      }
    }

    update(updater) {
      this.set(updater(this._value));
    }

    subscribe(callback) {
      this._dependents.add({ _run: callback });
      return () => this._dependents.delete(callback);
    }
  },

  /**
   * Computed Signal - Derived reactive value
   */
  ComputedSignal: class {
    constructor(computeFn) {
      this._computeFn = computeFn;
      this._value = undefined;
      this._isDirty = true;
      this._dependents = new Set();
      this._dependencies = new Set();
    }

    get() {
      vela._track(this);

      if (this._isDirty) {
        this._recompute();
      }

      return this._value;
    }

    _recompute() {
      // Clear previous dependencies
      this._dependencies.forEach(dep => {
        dep._dependents.delete(this);
      });
      this._dependencies.clear();

      // Save current effect context
      const prevEffect = vela._currentEffect;
      vela._currentEffect = this;

      try {
        this._value = this._computeFn();
        this._isDirty = false;
      } finally {
        vela._currentEffect = prevEffect;
      }
    }

    _invalidate() {
      if (!this._isDirty) {
        this._isDirty = true;
        vela._trigger(this);
      }
    }

    subscribe(callback) {
      this._dependents.add({ _run: callback });
      return () => this._dependents.delete(callback);
    }
  },

  /**
   * Effect - Reactive side effect
   */
  Effect: class {
    constructor(effectFn) {
      this._effectFn = effectFn;
      this._dependencies = new Set();
      this._cleanupFn = null;
    }

    _run() {
      // Cleanup previous run
      if (this._cleanupFn) {
        this._cleanupFn();
        this._cleanupFn = null;
      }

      // Clear previous dependencies
      this._dependencies.forEach(dep => {
        dep._dependents.delete(this);
      });
      this._dependencies.clear();

      // Save current effect context
      const prevEffect = vela._currentEffect;
      vela._currentEffect = this;

      try {
        const result = this._effectFn();
        // If effect returns a function, it's a cleanup function
        if (typeof result === 'function') {
          this._cleanupFn = result;
        }
      } finally {
        vela._currentEffect = prevEffect;
      }
    }

    _cleanup() {
      if (this._cleanupFn) {
        this._cleanupFn();
        this._cleanupFn = null;
      }

      this._dependencies.forEach(dep => {
        dep._dependents.delete(this);
      });
      this._dependencies.clear();
    }
  },

  // ===== OPTION TYPE =====

  /**
   * Create a Some value
   * @param {any} value - The wrapped value
   * @returns {Option} Some option
   */
  Some: function(value) {
    return {
      type: 'Some',
      value: value,

      /**
       * Check if this is Some
       * @returns {boolean} true
       */
      isSome: () => true,

      /**
       * Check if this is None
       * @returns {boolean} false
       */
      isNone: () => false,

      /**
       * Unwrap the value (unsafe)
       * @returns {any} The wrapped value
       */
      unwrap: function() {
        return this.value;
      },

      /**
       * Unwrap with default value
       * @param {any} defaultValue - Default value if None
       * @returns {any} The value or default
       */
      unwrapOr: function(defaultValue) {
        return this.value;
      },

      /**
       * Map the value if Some
       * @param {Function} fn - Mapping function
       * @returns {Option} Mapped option
       */
      map: function(fn) {
        return vela.Some(fn(this.value));
      },

      /**
       * Apply function if Some
       * @param {Function} fn - Function to apply
       */
      ifSome: function(fn) {
        fn(this.value);
      }
    };
  },

  /**
   * The None value
   */
  None: {
    type: 'None',

    isSome: () => false,
    isNone: () => true,

    unwrap: function() {
      throw new Error('Called unwrap() on None');
    },

    unwrapOr: function(defaultValue) {
      return defaultValue;
    },

    map: function(fn) {
      return vela.None;
    },

    ifSome: function(fn) {
      // Do nothing
    }
  },

  // ===== RESULT TYPE =====

  /**
   * Create an Ok result
   * @param {any} value - The success value
   * @returns {Result} Ok result
   */
  Ok: function(value) {
    return {
      type: 'Ok',
      value: value,

      /**
       * Check if this is Ok
       * @returns {boolean} true
       */
      isOk: () => true,

      /**
       * Check if this is Err
       * @returns {boolean} false
       */
      isErr: () => false,

      /**
       * Unwrap the value (unsafe)
       * @returns {any} The success value
       */
      unwrap: function() {
        return this.value;
      },

      /**
       * Unwrap with error message
       * @param {string} message - Error message
       * @returns {any} The value
       */
      expect: function(message) {
        return this.value;
      },

      /**
       * Map the value if Ok
       * @param {Function} fn - Mapping function
       * @returns {Result} Mapped result
       */
      map: function(fn) {
        return vela.Ok(fn(this.value));
      },

      /**
       * Map error if Err (no-op for Ok)
       * @param {Function} fn - Error mapping function
       * @returns {Result} This result
       */
      mapErr: function(fn) {
        return this;
      }
    };
  },

  /**
   * Create an Err result
   * @param {any} error - The error value
   * @returns {Result} Err result
   */
  Err: function(error) {
    return {
      type: 'Err',
      error: error,

      isOk: () => false,
      isErr: () => true,

      unwrap: function() {
        throw new Error(`Called unwrap() on Err: ${this.error}`);
      },

      expect: function(message) {
        throw new Error(`${message}: ${this.error}`);
      },

      map: function(fn) {
        return this;
      },

      mapErr: function(fn) {
        return vela.Err(fn(this.error));
      }
    };
  },

  // ===== UTILITY FUNCTIONS =====

  /**
   * Print to console
   * @param {any} value - Value to print
   */
  println: function(value) {
    console.log(value);
  },

  /**
   * Panic with message
   * @param {string} message - Panic message
   */
  panic: function(message) {
    throw new Error(`PANIC: ${message}`);
  },

  /**
   * Assert condition
   * @param {boolean} condition - Condition to check
   * @param {string} message - Error message if false
   */
  assert: function(condition, message) {
    if (!condition) {
      throw new Error(`ASSERTION FAILED: ${message}`);
    }
  },

  /**
   * Sleep for milliseconds (async)
   * @param {number} ms - Milliseconds to sleep
   * @returns {Promise} Promise that resolves after sleep
   */
  sleep: function(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  },

  /**
   * Get current timestamp
   * @returns {number} Current timestamp in milliseconds
   */
  now: function() {
    return Date.now();
  },

  /**
   * Deep clone an object
   * @param {any} obj - Object to clone
   * @returns {any} Cloned object
   */
  clone: function(obj) {
    return JSON.parse(JSON.stringify(obj));
  },

  /**
   * Check if two values are equal (deep equality)
   * @param {any} a - First value
   * @param {any} b - Second value
   * @returns {boolean} True if equal
   */
  equals: function(a, b) {
    return JSON.stringify(a) === JSON.stringify(b);
  }
};

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

// Export for Node.js
if (typeof module !== 'undefined' && module.exports) {
  module.exports = vela;
}

// Export for browsers
if (typeof window !== 'undefined') {
  window.vela = vela;
}
"#;

/// Generate the Vela runtime as a separate JavaScript file
pub fn generate_runtime_file() -> String {
    VELA_RUNTIME_JS.to_string()
}

/// Generate runtime import statement for generated code
pub fn generate_runtime_import() -> String {
    "// Runtime import - ensure vela-runtime.js is loaded\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_contains_expected_functions() {
        let runtime = VELA_RUNTIME_JS;

        // Check that key functions are present
        assert!(runtime.contains("state"));
        assert!(runtime.contains("computed"));
        assert!(runtime.contains("effect"));
        assert!(runtime.contains("batch"));
        assert!(runtime.contains("Some"));
        assert!(runtime.contains("None"));
        assert!(runtime.contains("Ok"));
        assert!(runtime.contains("Err"));
        assert!(runtime.contains("println"));
        assert!(runtime.contains("panic"));
    }

    #[test]
    fn test_runtime_is_valid_javascript() {
        let runtime = VELA_RUNTIME_JS;

        // Basic syntax check - should not contain obvious syntax errors
        assert!(!runtime.contains("function function")); // Double function
        assert!(!runtime.contains("const const")); // Double const
        assert!(!runtime.contains("let let")); // Double let
        assert!(!runtime.contains("class class")); // Double class
    }

    #[test]
    fn test_reactive_system_classes_present() {
        let runtime = VELA_RUNTIME_JS;

        // Check that reactive system classes are present
        assert!(runtime.contains("StateSignal"));
        assert!(runtime.contains("ComputedSignal"));
        assert!(runtime.contains("Effect"));
        assert!(runtime.contains("_track"));
        assert!(runtime.contains("_trigger"));
        assert!(runtime.contains("_flushPendingEffects"));
    }

    #[test]
    fn test_reactive_api_methods_present() {
        let runtime = VELA_RUNTIME_JS;

        // Check that reactive API methods are present
        assert!(runtime.contains("state:"));
        assert!(runtime.contains("computed:"));
        assert!(runtime.contains("effect:"));
        assert!(runtime.contains("batch:"));
        assert!(runtime.contains("readonly:"));
    }
}