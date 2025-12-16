//! JavaScript-WebAssembly Glue Code Generator
//!
//! This module generates JavaScript glue code for seamless interoperation
//! between JavaScript and WebAssembly modules generated from Vela.
//!
//! Jira: VELA-1119
//! Task: TASK-119
//! Date: 2025-12-14

use crate::ir::{IRModule, IRFunction, IRType, IRParam, IRGlobal};

/// JavaScript glue code generator for WASM interop
pub struct JSGlueGenerator {
    module: IRModule,
    class_name: String,
}

impl JSGlueGenerator {
    /// Create a new JS glue generator
    pub fn new(module: IRModule, class_name: String) -> Self {
        Self { module, class_name }
    }

    /// Generate JavaScript glue code
    pub fn generate_glue_code(&self) -> String {
        let mut code = String::new();

        // Class header
        code.push_str(&format!("export class {} {{\n", self.class_name));

        // Constructor
        code.push_str(&self.generate_constructor());

        // Initialization method
        code.push_str(&self.generate_init_method());

        // Function wrappers - all functions are considered public for now
        for function in &self.module.functions {
            code.push_str(&self.generate_function_wrapper(function));
        }

        // Memory access helpers
        code.push_str(&self.generate_memory_helpers());

        // Class footer
        code.push_str("}\n");

        // Export helper functions
        code.push_str(&self.generate_helper_functions());

        code
    }

    /// Generate constructor
    fn generate_constructor(&self) -> String {
        format!("  constructor(wasmModule) {{
    this.wasmModule = wasmModule;
    this.instance = null;
    this.memory = null;
    this.exports = null;
  }}

")
    }

    /// Generate initialization method
    fn generate_init_method(&self) -> String {
        let mut code = String::from("  async init() {
    try {
      const instance = await WebAssembly.instantiate(this.wasmModule);
      this.instance = instance.instance;
      this.exports = this.instance.exports;
      this.memory = this.exports.memory || new WebAssembly.Memory({ initial: 1 });

      // Initialize memory if needed
      if (this.exports.__wasm_init_memory) {
        this.exports.__wasm_init_memory();
      }

      return true;
    } catch (error) {
      console.error('Failed to initialize WASM module:', error);
      throw error;
    }
  }

");

        // Add memory initialization if there are globals
        if !self.module.globals.is_empty() {
            code.push_str("  // Initialize globals
    this.initGlobals();
  }

  initGlobals() {
");
            for global in &self.module.globals {
                let js_type = self.map_type_to_js(&global.ty);
                code.push_str(&format!("    this.{} = {};\n", global.name, self.get_default_value(&global.ty)));
            }
            code.push_str("  }\n\n");
        }

        code
    }

    /// Generate function wrapper
    fn generate_function_wrapper(&self, function: &IRFunction) -> String {
        let mut code = format!("  {}(", function.name);

        // Parameters
        let params: Vec<String> = function.params.iter()
            .enumerate()
            .map(|(i, param)| self.map_type_to_js_param(&param.ty, i))
            .collect();
        code.push_str(&params.join(", "));

        code.push_str(") {\n");

        // Function body
        code.push_str("    try {\n");

        // Call WASM function
        if function.return_type != IRType::Void {
            code.push_str(&format!("      const result = this.exports.{}({});\n",
                function.name,
                (0..function.params.len()).map(|i| format!("arg{}", i)).collect::<Vec<_>>().join(", ")
            ));

            // Convert result back to JS type
            code.push_str(&format!("      return this.{}ToJS(result);\n",
                self.get_type_converter_prefix(&function.return_type)));
        } else {
            code.push_str(&format!("      this.exports.{}({});\n",
                function.name,
                (0..function.params.len()).map(|i| format!("arg{}", i)).collect::<Vec<_>>().join(", ")
            ));
        }

        code.push_str("    } catch (error) {
      console.error(`Error calling ${function.name}:`, error);
      throw error;
    }
  }

");

        code
    }

    /// Generate memory access helpers
    fn generate_memory_helpers(&self) -> String {
        let mut code = String::from("  // Memory access helpers
  readString(ptr, len) {
    const buffer = new Uint8Array(this.memory.buffer, ptr, len);
    return new TextDecoder('utf-8').decode(buffer);
  }

  writeString(str) {
    const encoded = new TextEncoder().encode(str);
    const ptr = this.allocate(encoded.length);
    const buffer = new Uint8Array(this.memory.buffer, ptr, encoded.length);
    buffer.set(encoded);
    return { ptr, len: encoded.length };
  }

  allocate(size) {
    if (this.exports.__wasm_allocate) {
      return this.exports.__wasm_allocate(size);
    }
    // Fallback: assume linear memory
    return 0; // Would need proper allocator
  }

  deallocate(ptr) {
    if (this.exports.__wasm_deallocate) {
      this.exports.__wasm_deallocate(ptr);
    }
  }

  // Type conversion helpers
  i32ToJS(value) { return value; }
  i64ToJS(value) { return Number(value); }
  f32ToJS(value) { return value; }
  f64ToJS(value) { return value; }

  jsToI32(value) { return value; }
  jsToI64(value) { return BigInt(value); }
  jsToF32(value) { return value; }
  jsToF64(value) { return value; }

");

        code
    }

    /// Generate helper functions
    fn generate_helper_functions(&self) -> String {
        let mut code = String::from("// Helper functions for loading and using WASM modules
export async function loadWasmModule(url) {
  try {
    const response = await fetch(url);
    const buffer = await response.arrayBuffer();
    return new Uint8Array(buffer);
  } catch (error) {
    console.error('Failed to load WASM module:', error);
    throw error;
  }
}

export async function createWasmInstance(wasmBytes, className) {
  try {
    const module = await WebAssembly.compile(wasmBytes);
    const instance = new className(module);
    await instance.init();
    return instance;
  } catch (error) {
    console.error('Failed to create WASM instance:', error);
    throw error;
  }
}

// Convenience function for common usage
export async function loadAndInstantiate(wasmUrl, className) {
  const wasmBytes = await loadWasmModule(wasmUrl);
  return await createWasmInstance(wasmBytes, className);
}
");

        code
    }

    /// Map IR type to JavaScript type
    fn map_type_to_js(&self, ir_type: &IRType) -> &'static str {
        match ir_type {
            IRType::Void => "void",
            IRType::Bool => "boolean",
            IRType::Int => "number",
            IRType::Float => "number",
            IRType::String => "string",
            IRType::Array(_) => "array",
            IRType::Object(_) => "object",
        }
    }

    /// Map IR type to JavaScript parameter
    fn map_type_to_js_param(&self, ir_type: &IRType, index: usize) -> String {
        let js_type = self.map_type_to_js(ir_type);
        format!("arg{} /* {} */", index, js_type)
    }

    /// Get default value for type
    fn get_default_value(&self, ir_type: &IRType) -> String {
        match ir_type {
            IRType::Void => "undefined",
            IRType::Bool => "false",
            IRType::Int => "0",
            IRType::Float => "0.0",
            IRType::String => "\"\"",
            IRType::Array(_) => "[]",
            IRType::Object(_) => "{}",
        }.to_string()
    }

    /// Get type converter prefix
    fn get_type_converter_prefix(&self, ir_type: &IRType) -> String {
        match ir_type {
            IRType::Int => "i32", // Assuming i32 for Int
            IRType::Float => "f64", // Assuming f64 for Float
            _ => "",
        }.to_string()
    }
}

/// Generate TypeScript definitions for the glue code
pub struct TypeScriptGenerator {
    module: IRModule,
    class_name: String,
}

impl TypeScriptGenerator {
    pub fn new(module: IRModule, class_name: String) -> Self {
        Self { module, class_name }
    }

    pub fn generate_types(&self) -> String {
        let mut code = format!("// TypeScript definitions for {} WASM bindings
export declare class {} {{
  constructor(wasmModule: WebAssembly.Module);
  init(): Promise<boolean>;

", self.class_name, self.class_name);

        // Function signatures - all functions are considered public
        for function in &self.module.functions {
            code.push_str(&format!("  {}(", function.name));

            let params: Vec<String> = function.params.iter()
                .enumerate()
                .map(|(i, param)| format!("arg{}: {}", i, self.map_type_to_ts(&param.ty)))
                .collect();
            code.push_str(&params.join(", "));

            code.push_str("): ");

            if function.return_type != IRType::Void {
                code.push_str(&self.map_type_to_ts(&function.return_type));
            } else {
                code.push_str("void");
            }

            code.push_str(";\n");
        }

        code.push_str("}\n\n");

        // Helper function types
        code.push_str("export declare function loadWasmModule(url: string): Promise<Uint8Array>;
export declare function createWasmInstance<T>(wasmBytes: Uint8Array, classConstructor: new (module: WebAssembly.Module) => T): Promise<T>;
export declare function loadAndInstantiate<T>(wasmUrl: string, classConstructor: new (module: WebAssembly.Module) => T): Promise<T>;
");

        code
    }

    fn map_type_to_ts(&self, ir_type: &IRType) -> &'static str {
        match ir_type {
            IRType::Void => "void",
            IRType::Bool => "boolean",
            IRType::Int => "number",
            IRType::Float => "number",
            IRType::String => "string",
            IRType::Array(_) => "any[]",
            IRType::Object(_) => "any",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IRModule, IRFunction, IRParam, IRType};

    #[test]
    fn test_js_glue_generation() {
        let function = IRFunction {
            name: "add".to_string(),
            params: vec![
                IRParam { name: "a".to_string(), ty: IRType::Int },
                IRParam { name: "b".to_string(), ty: IRType::Int },
            ],
            return_type: IRType::Int,
            locals: vec![],
            body: vec![],
            position: None,
            is_async: false,
        };

        let module = IRModule {
            name: "test".to_string(),
            functions: vec![function],
            globals: vec![],
            position: None,
        };

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        assert!(glue_code.contains("export class TestModule"));
        assert!(glue_code.contains("add(arg0 /* number */, arg1 /* number */)"));
        assert!(glue_code.contains("this.exports.add(arg0, arg1)"));
        assert!(glue_code.contains("return this.i32ToJS(result)"));
    }

    #[test]
    fn test_typescript_generation() {
        let function = IRFunction {
            name: "multiply".to_string(),
            params: vec![
                IRParam { name: "x".to_string(), ty: IRType::Float },
                IRParam { name: "y".to_string(), ty: IRType::Float },
            ],
            return_type: IRType::Float,
            locals: vec![],
            body: vec![],
            position: None,
            is_async: false,
        };

        let module = IRModule {
            name: "math".to_string(),
            functions: vec![function],
            globals: vec![],
            position: None,
        };

        let generator = TypeScriptGenerator::new(module, "MathModule".to_string());
        let ts_code = generator.generate_types();

        assert!(ts_code.contains("export declare class MathModule"));
        assert!(ts_code.contains("multiply(arg0: number, arg1: number): number"));
    }

    #[test]
    fn test_memory_helpers_included() {
        let module = IRModule {
            name: "test".to_string(),
            functions: vec![],
            globals: vec![],
            position: None,
        };

        let generator = JSGlueGenerator::new(module, "TestModule".to_string());
        let glue_code = generator.generate_glue_code();

        assert!(glue_code.contains("readString"));
        assert!(glue_code.contains("writeString"));
        assert!(glue_code.contains("allocate"));
        assert!(glue_code.contains("deallocate"));
    }
}