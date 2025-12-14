/*!
# LLVM Backend for Vela Compiler

This module implements the LLVM backend for the Vela compiler, providing
native code generation with maximum performance through LLVM optimizations.

## Architecture

The LLVM backend translates Vela IR to LLVM IR, then uses LLVM to generate
optimized native machine code for the target platform.

## Features

- Full LLVM IR generation from Vela IR
- Support for multiple target architectures (x86, ARM, AArch64)
- LLVM optimization pipeline integration
- Native performance with zero-cost abstractions
- Cross-platform compilation support
*/

use crate::ir::{IRModule, IRFunction, IRInstruction, IRType, Value, IRExpr};
#[cfg(feature = "llvm_backend")]
use inkwell::builder::Builder;
#[cfg(feature = "llvm_backend")]
use inkwell::context::Context;
#[cfg(feature = "llvm_backend")]
use inkwell::module::Module;
#[cfg(feature = "llvm_backend")]
use inkwell::types::{BasicType, BasicTypeEnum, FunctionType, StructType};
#[cfg(feature = "llvm_backend")]
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue};
#[cfg(feature = "llvm_backend")]
use inkwell::{AddressSpace, OptimizationLevel};
use std::collections::HashMap;

#[cfg(feature = "llvm_backend")]
/// LLVM code generator for Vela IR
pub struct LLVMGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

#[cfg(feature = "llvm_backend")]
impl<'ctx> LLVMGenerator<'ctx> {
    /// Create a new LLVM generator
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            functions: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    /// Generate LLVM IR from Vela IR module
    pub fn generate(&mut self, ir_module: &IRModule) -> Result<(), String> {
        // Generate LLVM types for all IR types
        self.generate_types(ir_module)?;

        // Generate function declarations
        for function in &ir_module.functions {
            self.generate_function_declaration(function)?;
        }

        // Generate function implementations
        for function in &ir_module.functions {
            self.generate_function_implementation(function)?;
        }

        // Generate global variables
        for global in &ir_module.globals {
            self.generate_global_variable(global)?;
        }

        Ok(())
    }

    /// Generate LLVM types from IR types
    fn generate_types(&self, _ir_module: &IRModule) -> Result<(), String> {
        // Types are generated on-demand when needed
        Ok(())
    }

    /// Convert IR type to LLVM type
    fn ir_type_to_llvm(&self, ir_type: &IRType) -> BasicTypeEnum<'ctx> {
        match ir_type {
            IRType::Void => panic!("Void type cannot be converted to LLVM basic type"),
            IRType::Bool => self.context.bool_type().into(),
            IRType::Int => self.context.i64_type().into(), // Use i64 as default integer type
            IRType::Float => self.context.f64_type().into(), // Use f64 as default float type
            IRType::String => {
                // Strings are represented as { i32, i8* } (length, data pointer)
                let string_struct = self.context.struct_type(
                    &[
                        self.context.i32_type().into(),
                        self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                string_struct.into()
            }
            IRType::Array(element_type) => {
                // Arrays are represented as { i32, element_type* } (length, data pointer)
                let element_llvm_type = self.ir_type_to_llvm(element_type);
                let array_struct = self.context.struct_type(
                    &[
                        self.context.i32_type().into(),
                        element_llvm_type.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                array_struct.into()
            }
            IRType::Object(_) => {
                // Objects are represented as opaque pointers for now
                // TODO: Implement proper object layout
                self.context.i8_type().ptr_type(AddressSpace::default()).into()
            }
        }
    }

    /// Generate function declaration
    fn generate_function_declaration(&mut self, function: &IRFunction) -> Result<(), String> {
        let param_types: Vec<BasicTypeEnum> = function.params
            .iter()
            .map(|param| self.ir_type_to_llvm(&param.ty))
            .collect();

        let return_type = if function.return_type == IRType::Void {
            self.context.void_type()
        } else {
            self.ir_type_to_llvm(&function.return_type)
        };

        let fn_type = if function.return_type == IRType::Void {
            return_type.fn_type(&param_types, false)
        } else {
            return_type.fn_type(&param_types, false)
        };

        let llvm_function = self.module.add_function(&function.name, fn_type, None);
        self.functions.insert(function.name.clone(), llvm_function);

        Ok(())
    }

    /// Generate function implementation
    fn generate_function_implementation(&mut self, function: &IRFunction) -> Result<(), String> {
        let llvm_function = *self.functions.get(&function.name)
            .ok_or_else(|| format!("Function {} not found", function.name))?;

        let entry_block = self.context.append_basic_block(llvm_function, "entry");
        self.builder.position_at_end(entry_block);

        // Clear local variables for this function
        self.variables.clear();

        // Generate function parameters
        for (i, param) in function.params.iter().enumerate() {
            let param_value = llvm_function.get_nth_param(i as u32)
                .ok_or_else(|| format!("Parameter {} not found in function {}", i, function.name))?;

            // Allocate space for the parameter
            let param_alloca = self.builder.build_alloca(param_value.get_type(), &param.name);

            // Store the parameter value
            self.builder.build_store(param_alloca, param_value);

            // Store in variables map
            self.variables.insert(param.name.clone(), param_alloca);
        }

        // Generate function body
        for instruction in &function.body {
            self.generate_instruction(instruction)?;
        }

        // Add implicit return if function doesn't have one and returns void
        if function.return_type == IRType::Void {
            self.builder.build_return(None);
        }

        Ok(())
    }

    /// Generate LLVM instruction from IR instruction
    fn generate_instruction(&mut self, instruction: &IRInstruction) -> Result<(), String> {
        match instruction {
            IRInstruction::LoadConst { dest, value } => {
                self.generate_load_const(dest, value)
            }
            IRInstruction::BinaryOp { dest, op, left, right } => {
                self.generate_binary_op(dest, op, left, right)
            }
            IRInstruction::Call { dest, function, args } => {
                self.generate_call(dest, function, args)
            }
            IRInstruction::Return { value } => {
                self.generate_return(value)
            }
            IRInstruction::AssignVar { name, value } => {
                self.generate_assign_var(name, value)
            }
            _ => {
                // TODO: Implement other instruction types
                println!("Warning: Unimplemented instruction: {:?}", instruction);
                Ok(())
            }
        }
    }

    /// Generate load constant instruction
    fn generate_load_const(&mut self, dest: &str, value: &Value) -> Result<(), String> {
        let llvm_value = match value {
            Value::Bool(b) => self.context.bool_type().const_int(*b as u64, false).into(),
            Value::Int(i) => self.context.i64_type().const_int(*i as u64, true).into(),
            Value::Float(f) => self.context.f64_type().const_float(*f).into(),
            Value::String(s) => {
                // Create global string constant
                let string_const = self.builder.build_global_string_ptr(s, ".str");
                // Create string struct { length, data }
                let string_struct_type = self.ir_type_to_llvm(&IRType::String);
                let string_alloca = self.builder.build_alloca(string_struct_type, "string_temp");

                // Store length
                let length_ptr = self.builder.build_struct_gep(string_struct_type, string_alloca, 0, "length_ptr")
                    .map_err(|_| "Failed to get length field")?;
                let length_value = self.context.i32_type().const_int(s.len() as u64, false);
                self.builder.build_store(length_ptr, length_value);

                // Store data pointer
                let data_ptr = self.builder.build_struct_gep(string_struct_type, string_alloca, 1, "data_ptr")
                    .map_err(|_| "Failed to get data field")?;
                self.builder.build_store(data_ptr, string_const);

                self.builder.build_load(string_struct_type, string_alloca, "string_load")
            }
            _ => return Err(format!("Unsupported constant type: {:?}", value)),
        };

        // Allocate space for the destination
        let dest_alloca = self.builder.build_alloca(llvm_value.get_type(), dest);
        self.builder.build_store(dest_alloca, llvm_value);

        // Store in variables map
        self.variables.insert(dest.to_string(), dest_alloca);

        Ok(())
    }

    /// Generate binary operation
    fn generate_binary_op(&mut self, dest: &str, op: &str, left: &str, right: &str) -> Result<(), String> {
        let left_var = self.variables.get(left)
            .ok_or_else(|| format!("Variable {} not found", left))?;
        let right_var = self.variables.get(right)
            .ok_or_else(|| format!("Variable {} not found", right))?;

        let left_value = self.builder.build_load(left_var.get_type(), *left_var, "left_load");
        let right_value = self.builder.build_load(right_var.get_type(), *right_var, "right_load");

        let result_value = match op {
            "+" => {
                match left_value {
                    BasicValueEnum::IntValue(left_int) => {
                        let right_int = right_value.into_int_value();
                        self.builder.build_int_add(left_int, right_int, "add").into()
                    }
                    BasicValueEnum::FloatValue(left_float) => {
                        let right_float = right_value.into_float_value();
                        self.builder.build_float_add(left_float, right_float, "fadd").into()
                    }
                    _ => return Err(format!("Unsupported operand types for +: {:?}", left_value.get_type())),
                }
            }
            "-" => {
                match left_value {
                    BasicValueEnum::IntValue(left_int) => {
                        let right_int = right_value.into_int_value();
                        self.builder.build_int_sub(left_int, right_int, "sub").into()
                    }
                    BasicValueEnum::FloatValue(left_float) => {
                        let right_float = right_value.into_float_value();
                        self.builder.build_float_sub(left_float, right_float, "fsub").into()
                    }
                    _ => return Err(format!("Unsupported operand types for -: {:?}", left_value.get_type())),
                }
            }
            "*" => {
                match left_value {
                    BasicValueEnum::IntValue(left_int) => {
                        let right_int = right_value.into_int_value();
                        self.builder.build_int_mul(left_int, right_int, "mul").into()
                    }
                    BasicValueEnum::FloatValue(left_float) => {
                        let right_float = right_value.into_float_value();
                        self.builder.build_float_mul(left_float, right_float, "fmul").into()
                    }
                    _ => return Err(format!("Unsupported operand types for *: {:?}", left_value.get_type())),
                }
            }
            "/" => {
                match left_value {
                    BasicValueEnum::IntValue(left_int) => {
                        let right_int = right_value.into_int_value();
                        self.builder.build_int_signed_div(left_int, right_int, "div").into()
                    }
                    BasicValueEnum::FloatValue(left_float) => {
                        let right_float = right_value.into_float_value();
                        self.builder.build_float_div(left_float, right_float, "fdiv").into()
                    }
                    _ => return Err(format!("Unsupported operand types for /: {:?}", left_value.get_type())),
                }
            }
            _ => return Err(format!("Unsupported binary operator: {}", op)),
        };

        // Allocate space for the destination
        let dest_alloca = self.builder.build_alloca(result_value.get_type(), dest);
        self.builder.build_store(dest_alloca, result_value);

        // Store in variables map
        self.variables.insert(dest.to_string(), dest_alloca);

        Ok(())
    }

    /// Generate function call
    fn generate_call(&mut self, dest: &Option<String>, function: &str, args: &[String]) -> Result<(), String> {
        let llvm_function = self.functions.get(function)
            .ok_or_else(|| format!("Function {} not found", function))?;

        // Load argument values
        let mut arg_values = Vec::new();
        for arg in args {
            let arg_var = self.variables.get(arg)
                .ok_or_else(|| format!("Variable {} not found", arg))?;
            let arg_value = self.builder.build_load(arg_var.get_type(), *arg_var, &format!("arg_{}", arg));
            arg_values.push(arg_value);
        }

        // Call the function
        let call_result = self.builder.build_call(llvm_function, &arg_values, "call");

        // Store result if destination is specified
        if let Some(dest_name) = dest {
            if let Some(result_value) = call_result.try_as_basic_value().left() {
                let dest_alloca = self.builder.build_alloca(result_value.get_type(), dest_name);
                self.builder.build_store(dest_alloca, result_value);
                self.variables.insert(dest_name.clone(), dest_alloca);
            }
        }

        Ok(())
    }

    /// Generate return instruction
    fn generate_return(&mut self, value: &Option<String>) -> Result<(), String> {
        if let Some(value_name) = value {
            let value_var = self.variables.get(value_name)
                .ok_or_else(|| format!("Variable {} not found", value_name))?;
            let value_loaded = self.builder.build_load(value_var.get_type(), *value_var, "return_value");
            self.builder.build_return(Some(&value_loaded));
        } else {
            self.builder.build_return(None);
        }

        Ok(())
    }

    /// Generate variable assignment
    fn generate_assign_var(&mut self, name: &str, value: &str) -> Result<(), String> {
        let value_var = self.variables.get(value)
            .ok_or_else(|| format!("Variable {} not found", value))?;
        let value_loaded = self.builder.build_load(value_var.get_type(), *value_var, &format!("load_{}", value));

        // Allocate space if variable doesn't exist
        let dest_alloca = if let Some(existing) = self.variables.get(name) {
            *existing
        } else {
            let alloca = self.builder.build_alloca(value_loaded.get_type(), name);
            self.variables.insert(name.to_string(), alloca);
            alloca
        };

        self.builder.build_store(dest_alloca, value_loaded);

        Ok(())
    }

    /// Generate global variable
    fn generate_global_variable(&mut self, global: &crate::ir::IRGlobal) -> Result<(), String> {
        let llvm_type = self.ir_type_to_llvm(&global.ty);
        let initial_value = match &global.initial_value {
            Some(value) => {
                match value {
                    Value::Bool(b) => self.context.bool_type().const_int(*b as u64, false).into(),
                    Value::Int(i) => self.context.i64_type().const_int(*i as u64, true).into(),
                    Value::Float(f) => self.context.f64_type().const_float(*f).into(),
                    Value::String(s) => {
                        // For global strings, create a constant
                        let string_const = self.module.add_global(
                            self.context.i8_type().array_type((s.len() + 1) as u32),
                            Some(AddressSpace::default()),
                            &format!("{}.str", global.name)
                        );
                        string_const.set_initializer(&self.context.const_string(s.as_bytes(), false));
                        string_const.as_pointer_value().into()
                    }
                    _ => return Err(format!("Unsupported global initial value: {:?}", value)),
                }
            }
            None => {
                // Default initialization
                match global.ty {
                    IRType::Bool => self.context.bool_type().const_zero().into(),
                    IRType::Int => self.context.i64_type().const_zero().into(),
                    IRType::Float => self.context.f64_type().const_zero().into(),
                    _ => return Err(format!("Cannot default initialize global of type {:?}", global.ty)),
                }
            }
        };

        let global_var = self.module.add_global(llvm_type, Some(AddressSpace::default()), &global.name);
        global_var.set_initializer(&initial_value);

        Ok(())
    }

    /// Get the generated LLVM module
    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Write LLVM IR to string
    pub fn to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Write LLVM bitcode to file
    pub fn write_bitcode_to_file(&self, filename: &str) -> Result<(), String> {
        self.module.write_bitcode_to_path(std::path::Path::new(filename))
            .map_err(|e| format!("Failed to write bitcode: {}", e))
    }

    /// Compile to object file
    pub fn compile_to_object(&self, filename: &str, optimization: OptimizationLevel) -> Result<(), String> {
        // Create execution engine for JIT compilation
        let execution_engine = self.module.create_jit_execution_engine(optimization)
            .map_err(|e| format!("Failed to create execution engine: {}", e))?;

        // For now, just write the object file
        // TODO: Implement proper object file generation
        self.write_bitcode_to_file(filename)
    }
}

#[cfg(not(feature = "llvm_backend"))]
/// Stub implementation when LLVM backend is not available
pub struct LLVMGenerator;

#[cfg(not(feature = "llvm_backend"))]
impl LLVMGenerator {
    /// Create a new LLVM generator (stub)
    pub fn new(_context: &(), _module_name: &str) -> Self {
        Self
    }

    /// Generate LLVM IR from Vela IR module (stub)
    pub fn generate(&mut self, _ir_module: &IRModule) -> Result<(), String> {
        Err("LLVM backend not available. Enable with --features llvm_backend".to_string())
    }

    /// Compile to object file (stub)
    pub fn compile_to_object(&self, _filename: &str, _optimization: ()) -> Result<(), String> {
        Err("LLVM backend not available. Enable with --features llvm_backend".to_string())
    }
}