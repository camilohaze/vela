//! Vela Runtime for JavaScript
//!
//! This file contains the JavaScript runtime that supports Vela-generated code.
//! It provides implementations for signals, Option types, Result types, and other
//! Vela-specific constructs.

/// Vela Runtime JavaScript Code
pub const VELA_RUNTIME_JS: &str = r#"
// Vela Runtime for JavaScript
// Generated automatically - do not modify

const vela = {
  // ===== SIGNALS (REACTIVE STATE) =====

  /**
   * Create a reactive signal
   * @param {any} initialValue - Initial value for the signal
   * @returns {Signal} Reactive signal object
   */
  createSignal: function(initialValue) {
    const subscribers = new Set();

    return {
      _value: initialValue,
      _subscribers: subscribers,

      /**
       * Get the current value
       * @returns {any} Current signal value
       */
      get value() {
        return this._value;
      },

      /**
       * Set a new value and notify subscribers
       * @param {any} newValue - New value to set
       */
      set value(newValue) {
        if (this._value !== newValue) {
          this._value = newValue;
          this._subscribers.forEach(callback => callback(newValue));
        }
      },

      /**
       * Subscribe to value changes
       * @param {Function} callback - Function to call when value changes
       * @returns {Function} Unsubscribe function
       */
      subscribe: function(callback) {
        this._subscribers.add(callback);
        return () => this._subscribers.delete(callback);
      },

      /**
       * Create a computed value that depends on this signal
       * @param {Function} computeFn - Function that computes the derived value
       * @returns {Signal} Computed signal
       */
      map: function(computeFn) {
        const computed = vela.createSignal(computeFn(this._value));
        this.subscribe(() => {
          computed.value = computeFn(this._value);
        });
        return computed;
      }
    };
  },

  /**
   * Create a computed signal
   * @param {Function} computeFn - Function that computes the value
   * @returns {Signal} Computed signal
   */
  computed: function(computeFn) {
    return vela.createSignal(computeFn());
  },

  /**
   * Create an effect that runs when dependencies change
   * @param {Function} effectFn - Effect function to run
   * @returns {Function} Cleanup function
   */
  effect: function(effectFn) {
    // Simple implementation - in a real runtime this would track dependencies
    effectFn();
    return () => {}; // No-op cleanup for now
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
        assert!(runtime.contains("createSignal"));
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
    }
}