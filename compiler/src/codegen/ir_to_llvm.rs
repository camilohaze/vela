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
    stack: Vec<BasicValueEnum<'ctx>>,
    labels: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>>,
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
            stack: Vec::new(),
            labels: HashMap::new(),
        }
    }

    /// Generate LLVM IR from Vela IR module
    pub fn generate(&mut self, ir_module: &IRModule) -> Result<(), String> {
        // Declare runtime library functions
        self.declare_runtime_functions()?;

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

    /// Declare runtime library functions
    fn declare_runtime_functions(&mut self) -> Result<(), String> {
        // Runtime initialization/shutdown
        self.declare_runtime_function("vela_init_runtime", &[], &IRType::Void)?;
        self.declare_runtime_function("vela_shutdown_runtime", &[], &IRType::Void)?;

        // Garbage collector functions
        self.declare_runtime_function("vela_gc_alloc", &[IRType::Int], &IRType::Object(vec![]))?;
        self.declare_runtime_function("vela_gc_collect", &[], &IRType::Void)?;
        self.declare_runtime_function("vela_gc_add_root", &[IRType::Object(vec![])], &IRType::Void)?;
        self.declare_runtime_function("vela_gc_remove_root", &[IRType::Object(vec![])], &IRType::Void)?;

        // Signal functions
        self.declare_runtime_function("vela_signal_create", &[IRType::Object(vec![])], &IRType::Object(vec![]))?;
        self.declare_runtime_function("vela_signal_destroy", &[IRType::Object(vec![])], &IRType::Void)?;
        self.declare_runtime_function("vela_signal_set", &[IRType::Object(vec![]), IRType::Object(vec![])], &IRType::Void)?;
        self.declare_runtime_function("vela_signal_get", &[IRType::Object(vec![])], &IRType::Object(vec![]))?;

        // Actor functions
        self.declare_runtime_function("vela_actor_create", &[IRType::Object(vec![]), IRType::Object(vec![])], &IRType::Object(vec![]))?;
        self.declare_runtime_function("vela_actor_destroy", &[IRType::Object(vec![])], &IRType::Void)?;
        self.declare_runtime_function("vela_actor_send", &[IRType::Object(vec![]), IRType::Object(vec![])], &IRType::Int)?;
        self.declare_runtime_function("vela_actor_get_state", &[IRType::Object(vec![])], &IRType::Object(vec![]))?;

        // Vela object functions
        self.declare_runtime_function("vela_array_create", &[IRType::Int, IRType::Int], &IRType::Object(vec![]))?;
        self.declare_runtime_function("vela_array_get", &[IRType::Object(vec![]), IRType::Int], &IRType::Object(vec![]))?;
        self.declare_runtime_function("vela_array_set", &[IRType::Object(vec![]), IRType::Int, IRType::Object(vec![])], &IRType::Int)?;
        self.declare_runtime_function("vela_array_length", &[IRType::Object(vec![])], &IRType::Int)?;

        self.declare_runtime_function("vela_string_create", &[IRType::String], &IRType::Object(vec![]))?;
        self.declare_runtime_function("vela_string_get", &[IRType::Object(vec![])], &IRType::String)?;
        self.declare_runtime_function("vela_string_length", &[IRType::Object(vec![])], &IRType::Int)?;

        self.declare_runtime_function("vela_object_create", &[], &IRType::Object(vec![]))?;
        self.declare_runtime_function("vela_object_set", &[IRType::Object(vec![]), IRType::String, IRType::Object(vec![])], &IRType::Int)?;
        self.declare_runtime_function("vela_object_get", &[IRType::Object(vec![]), IRType::String], &IRType::Object(vec![]))?;

        Ok(())
    }

    /// Helper to declare a runtime function
    fn declare_runtime_function(&mut self, name: &str, param_types: &[IRType], return_type: &IRType) -> Result<(), String> {
        let llvm_param_types: Vec<BasicTypeEnum> = param_types
            .iter()
            .map(|ty| self.ir_type_to_llvm(ty))
            .collect();

        let llvm_return_type = match return_type {
            IRType::Void => None,
            _ => Some(self.ir_type_to_llvm(return_type)),
        };

        let fn_type = match llvm_return_type {
            Some(ret_type) => ret_type.fn_type(&llvm_param_types, false),
            None => self.context.void_type().fn_type(&llvm_param_types, false),
        };

        let function = self.module.add_function(name, fn_type, None);
        self.functions.insert(name.to_string(), function);

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

        // Initialize runtime for main function
        if function.name == "main" {
            let vela_init_runtime_fn = self.get_runtime_function("vela_init_runtime")?;
            self.builder.build_call(vela_init_runtime_fn, &[], "init_runtime");
        }

        // Clear local variables and stack for this function
        self.variables.clear();
        self.stack.clear();
        self.labels.clear();

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
            // Shutdown runtime for main function before returning
            if function.name == "main" {
                let vela_shutdown_runtime_fn = self.get_runtime_function("vela_shutdown_runtime")?;
                self.builder.build_call(vela_shutdown_runtime_fn, &[], "shutdown_runtime");
            }
            self.builder.build_return(None);
        }

        Ok(())
    }

    /// Generate LLVM instruction from IR instruction
    fn generate_instruction(&mut self, instruction: &IRInstruction) -> Result<(), String> {
        match instruction {
            IRInstruction::DeclareVar { name, ty } => {
                self.generate_declare_var(name, ty)
            }
            IRInstruction::AssignVar { name, value } => {
                self.generate_assign_var_expr(name, value)
            }
            IRInstruction::LoadConst(value) => {
                self.generate_load_const_stack(value)
            }
            IRInstruction::LoadVar(name) => {
                self.generate_load_var_stack(name)
            }
            IRInstruction::StoreVar(name) => {
                self.generate_store_var_stack(name)
            }
            IRInstruction::BinaryOp(op) => {
                self.generate_binary_op_stack(op)
            }
            IRInstruction::UnaryOp(op) => {
                self.generate_unary_op_stack(op)
            }
            IRInstruction::Call { function, arg_count } => {
                self.generate_call_stack(function, *arg_count)
            }
            IRInstruction::Return => {
                self.generate_return_stack()
            }
            IRInstruction::Jump(label) => {
                self.generate_jump(label)
            }
            IRInstruction::JumpIf(label) => {
                self.generate_jump_if(label)
            }
            IRInstruction::Label(label) => {
                self.generate_label(label)
            }
            IRInstruction::CreateArray { element_type, size } => {
                self.generate_create_array(element_type, *size)
            }
            IRInstruction::ArrayAccess => {
                self.generate_array_access()
            }
            IRInstruction::ArrayStore => {
                self.generate_array_store()
            }
            IRInstruction::CreateObject(type_name) => {
                self.generate_create_object(type_name)
            }
            IRInstruction::PropertyAccess(prop_name) => {
                self.generate_property_access(prop_name)
            }
            IRInstruction::PropertyStore(prop_name) => {
                self.generate_property_store(prop_name)
            }
        }
    }

    /// Generate declare variable instruction
    fn generate_declare_var(&mut self, name: &str, ty: &IRType) -> Result<(), String> {
        let llvm_type = self.ir_type_to_llvm(ty);
        let alloca = self.builder.build_alloca(llvm_type, name);
        self.variables.insert(name.to_string(), alloca);
        Ok(())
    }

    /// Generate assign variable with expression
    fn generate_assign_var_expr(&mut self, name: &str, value: &IRExpr) -> Result<(), String> {
        let value_llvm = self.generate_expression(value)?;
        let var_alloca = if let Some(existing) = self.variables.get(name) {
            *existing
        } else {
            let llvm_type = value_llvm.get_type();
            let alloca = self.builder.build_alloca(llvm_type, name);
            self.variables.insert(name.to_string(), alloca);
            alloca
        };
        self.builder.build_store(var_alloca, value_llvm);
        Ok(())
    }

    /// Generate load constant to stack
    fn generate_load_const_stack(&mut self, value: &Value) -> Result<(), String> {
        let llvm_value = match value {
            Value::Bool(b) => self.context.bool_type().const_int(*b as u64, false).into(),
            Value::Int(i) => self.context.i64_type().const_int(*i as u64, true).into(),
            Value::Float(f) => self.context.f64_type().const_float(*f).into(),
            Value::String(s) => {
                // Create string struct { length, data }
                let string_struct_type = self.ir_type_to_llvm(&IRType::String);
                let string_alloca = self.builder.build_alloca(string_struct_type, "string_const");

                // Store length
                let length_ptr = self.builder.build_struct_gep(string_struct_type, string_alloca, 0, "length_ptr")
                    .map_err(|_| "Failed to get length field")?;
                let length_value = self.context.i32_type().const_int(s.len() as u64, false);
                self.builder.build_store(length_ptr, length_value);

                // Store data pointer
                let data_ptr = self.builder.build_struct_gep(string_struct_type, string_alloca, 1, "data_ptr")
                    .map_err(|_| "Failed to get data field")?;
                let string_const = self.builder.build_global_string_ptr(s, ".str");
                self.builder.build_store(data_ptr, string_const);

                self.builder.build_load(string_struct_type, string_alloca, "string_load")
            }
        };
        self.stack.push(llvm_value);
        Ok(())
    }

    /// Generate load variable to stack
    fn generate_load_var_stack(&mut self, name: &str) -> Result<(), String> {
        let var_alloca = self.variables.get(name)
            .ok_or_else(|| format!("Variable {} not found", name))?;
        let loaded_value = self.builder.build_load(var_alloca.get_type(), *var_alloca, &format!("load_{}", name));
        self.stack.push(loaded_value);
        Ok(())
    }

    /// Generate store variable from stack
    fn generate_store_var_stack(&mut self, name: &str) -> Result<(), String> {
        let value = self.stack.pop()
            .ok_or_else(|| "Stack underflow in store_var".to_string())?;
        let var_alloca = self.variables.get(name)
            .ok_or_else(|| format!("Variable {} not found", name))?;
        self.builder.build_store(*var_alloca, value);
        Ok(())
    }

    /// Generate binary operation on stack
    fn generate_binary_op_stack(&mut self, op: &BinaryOp) -> Result<(), String> {
        let right = self.stack.pop()
            .ok_or_else(|| "Stack underflow in binary op (right)".to_string())?;
        let left = self.stack.pop()
            .ok_or_else(|| "Stack underflow in binary op (left)".to_string())?;

        let result = match op {
            BinaryOp::Add => self.generate_add(left, right)?,
            BinaryOp::Sub => self.generate_sub(left, right)?,
            BinaryOp::Mul => self.generate_mul(left, right)?,
            BinaryOp::Div => self.generate_div(left, right)?,
            BinaryOp::Mod => self.generate_mod(left, right)?,
            BinaryOp::Eq => self.generate_eq(left, right)?,
            BinaryOp::Ne => self.generate_ne(left, right)?,
            BinaryOp::Lt => self.generate_lt(left, right)?,
            BinaryOp::Le => self.generate_le(left, right)?,
            BinaryOp::Gt => self.generate_gt(left, right)?,
            BinaryOp::Ge => self.generate_ge(left, right)?,
            BinaryOp::And => self.generate_and(left, right)?,
            BinaryOp::Or => self.generate_or(left, right)?,
        };
        self.stack.push(result);
        Ok(())
    }

    /// Generate unary operation on stack
    fn generate_unary_op_stack(&mut self, op: &UnaryOp) -> Result<(), String> {
        let value = self.stack.pop()
            .ok_or_else(|| "Stack underflow in unary op".to_string())?;

        let result = match op {
            UnaryOp::Neg => self.generate_neg(value)?,
            UnaryOp::Not => self.generate_not(value)?,
        };
        self.stack.push(result);
        Ok(())
    }

    /// Generate function call on stack
    fn generate_call_stack(&mut self, function: &str, arg_count: usize) -> Result<(), String> {
        if self.stack.len() < arg_count {
            return Err(format!("Stack underflow in call: need {} args, have {}", arg_count, self.stack.len()));
        }

        let mut args = Vec::new();
        for _ in 0..arg_count {
            args.push(self.stack.pop().unwrap());
        }
        args.reverse(); // Arguments were popped in reverse order

        let llvm_function = self.functions.get(function)
            .ok_or_else(|| format!("Function {} not found", function))?;

        let call_result = self.builder.build_call(llvm_function, &args, "call");
        if let Some(result_value) = call_result.try_as_basic_value().left() {
            self.stack.push(result_value);
        }
        Ok(())
    }

    /// Generate return from stack
    fn generate_return_stack(&mut self) -> Result<(), String> {
        if let Some(value) = self.stack.pop() {
            self.builder.build_return(Some(&value));
        } else {
            self.builder.build_return(None);
        }
        Ok(())
    }

    /// Generate unconditional jump
    fn generate_jump(&mut self, label: &Label) -> Result<(), String> {
        let block = self.labels.get(&label.0)
            .ok_or_else(|| format!("Label {} not found", label.0))?;
        self.builder.build_unconditional_branch(*block);
        Ok(())
    }

    /// Generate conditional jump
    fn generate_jump_if(&mut self, label: &Label) -> Result<(), String> {
        let condition = self.stack.pop()
            .ok_or_else(|| "Stack underflow in jump_if".to_string())?;
        let then_block = self.labels.get(&label.0)
            .ok_or_else(|| format!("Label {} not found", label.0))?;

        // Create else block
        let else_block = self.context.append_basic_block(
            self.builder.get_insert_block().get_parent().unwrap(),
            &format!("else_{}", label.0)
        );

        self.builder.build_conditional_branch(condition.into_int_value(), *then_block, else_block);
        self.builder.position_at_end(else_block);
        Ok(())
    }

    /// Generate label
    fn generate_label(&mut self, label: &Label) -> Result<(), String> {
        let current_function = self.builder.get_insert_block().get_parent().unwrap();
        let block = self.context.append_basic_block(current_function, &label.0);
        self.labels.insert(label.0.clone(), block);
        self.builder.position_at_end(block);
        Ok(())
    }

    /// Generate create array
    fn generate_create_array(&mut self, element_type: &IRType, size: usize) -> Result<(), String> {
        // Get the vela_create_array function
        let create_array_fn = *self.functions.get("vela_create_array")
            .ok_or("vela_create_array function not declared")?;

        // Calculate element size
        let element_size = match element_type {
            IRType::Bool | IRType::Int => 8, // i64 is 8 bytes
            IRType::Float => 8, // f64 is 8 bytes
            IRType::String => 16, // String struct is 16 bytes (length + pointer)
            IRType::Array(_) => 16, // Array struct is 16 bytes (length + pointer)
            IRType::Object(_) => 8, // Object pointer is 8 bytes
            IRType::Void => return Err("Cannot create array of void type".to_string()),
        };

        // Push arguments: element_count, element_size
        let size_value = self.context.i64_type().const_int(size as u64, false);
        let element_size_value = self.context.i64_type().const_int(element_size, false);

        let args = &[size_value.into(), element_size_value.into()];

        // Call vela_create_array
        let array_ptr = self.builder.build_call(create_array_fn, args, "array")
            .map_err(|_| "Failed to call vela_create_array")?;

        self.stack.push(array_ptr);
        Ok(())
    }

    /// Generate array access
    fn generate_array_access(&mut self) -> Result<(), String> {
        let index = self.stack.pop()
            .ok_or_else(|| "Stack underflow in array_access (index)".to_string())?;
        let array = self.stack.pop()
            .ok_or_else(|| "Stack underflow in array_access (array)".to_string())?;

        // Call runtime function vela_array_get
        let vela_array_get_fn = self.get_runtime_function("vela_array_get")?;
        let result = self.builder.build_call(
            vela_array_get_fn,
            &[array.into(), index.into()],
            "array_get_result"
        );

        // Cast result to appropriate type (assuming void* for now)
        let element_value = self.builder.build_bitcast(
            result,
            self.context.i64_type().ptr_type(inkwell::AddressSpace::Generic),
            "element_value"
        );

        self.stack.push(element_value);
        Ok(())
    }

    /// Generate array store
    fn generate_array_store(&mut self) -> Result<(), String> {
        let value = self.stack.pop()
            .ok_or_else(|| "Stack underflow in array_store (value)".to_string())?;
        let index = self.stack.pop()
            .ok_or_else(|| "Stack underflow in array_store (index)".to_string())?;
        let array = self.stack.pop()
            .ok_or_else(|| "Stack underflow in array_store (array)".to_string())?;

        // Call runtime function vela_array_set
        let vela_array_set_fn = self.get_runtime_function("vela_array_set")?;
        self.builder.build_call(
            vela_array_set_fn,
            &[array.into(), index.into(), value.into()],
            "array_set_result"
        );

        Ok(())
    }

    /// Generate create object
    fn generate_create_object(&mut self, type_name: &str) -> Result<(), String> {
        // Call runtime function vela_object_create
        let vela_object_create_fn = self.get_runtime_function("vela_object_create")?;
        let object = self.builder.build_call(
            vela_object_create_fn,
            &[],
            &format!("object_{}", type_name)
        );

        self.stack.push(object);
        Ok(())
    }

    /// Generate property access
    fn generate_property_access(&mut self, prop_name: &str) -> Result<(), String> {
        let object = self.stack.pop()
            .ok_or_else(|| "Stack underflow in property_access".to_string())?;

        // Call runtime function vela_object_get
        let vela_object_get_fn = self.get_runtime_function("vela_object_get")?;
        let prop_name_const = self.builder.build_global_string_ptr(prop_name, "prop_name");
        let result = self.builder.build_call(
            vela_object_get_fn,
            &[object.into(), prop_name_const.as_pointer_value().into()],
            &format!("prop_{}_value", prop_name)
        );

        // Cast result to appropriate type (assuming void* for now)
        let property_value = self.builder.build_bitcast(
            result,
            self.context.i64_type().ptr_type(inkwell::AddressSpace::Generic),
            "property_value"
        );

        self.stack.push(property_value);
        Ok(())
    }

    /// Generate property store
    fn generate_property_store(&mut self, prop_name: &str) -> Result<(), String> {
        let value = self.stack.pop()
            .ok_or_else(|| "Stack underflow in property_store (value)".to_string())?;
        let object = self.stack.pop()
            .ok_or_else(|| "Stack underflow in property_store (object)".to_string())?;

        // Call runtime function vela_object_set
        let vela_object_set_fn = self.get_runtime_function("vela_object_set")?;
        let prop_name_const = self.builder.build_global_string_ptr(prop_name, "prop_name");
        self.builder.build_call(
            vela_object_set_fn,
            &[object.into(), prop_name_const.as_pointer_value().into(), value.into()],
            "property_set_result"
        );

        Ok(())
    }
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

    /// Generate expression
    fn generate_expression(&mut self, expr: &IRExpr) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            IRExpr::Const(value) => self.generate_const_expr(value),
            IRExpr::Var(name) => {
                let var_alloca = self.variables.get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                Ok(self.builder.build_load(var_alloca.get_type(), *var_alloca, &format!("load_{}", name)))
            }
            IRExpr::Binary(op, left, right) => {
                let left_val = self.generate_expression(left)?;
                let right_val = self.generate_expression(right)?;
                self.generate_binary_op_values(op, left_val, right_val)
            }
            IRExpr::Unary(op, operand) => {
                let operand_val = self.generate_expression(operand)?;
                self.generate_unary_op_value(op, operand_val)
            }
            IRExpr::Call(function, args) => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.generate_expression(arg)?);
                }
                let llvm_function = self.functions.get(function)
                    .ok_or_else(|| format!("Function {} not found", function))?;
                let call_result = self.builder.build_call(llvm_function, &arg_values, "call");
                call_result.try_as_basic_value().left()
                    .ok_or_else(|| "Function call did not return a value".to_string())
            }
        }
    }

    /// Generate constant expression
    fn generate_const_expr(&self, value: &Value) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            Value::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Value::Int(i) => Ok(self.context.i64_type().const_int(*i as u64, true).into()),
            Value::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            Value::String(s) => {
                // Create global string constant
                let string_const = self.module.add_global(
                    self.context.i8_type().array_type((s.len() + 1) as u32),
                    Some(AddressSpace::default()),
                    ".str"
                );
                string_const.set_initializer(&self.context.const_string(s.as_bytes(), true));
                Ok(string_const.as_pointer_value().into())
            }
        }
    }

    /// Generate binary operation on values
    fn generate_binary_op_values(&self, op: &BinaryOp, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match op {
            BinaryOp::Add => self.generate_add(left, right),
            BinaryOp::Sub => self.generate_sub(left, right),
            BinaryOp::Mul => self.generate_mul(left, right),
            BinaryOp::Div => self.generate_div(left, right),
            BinaryOp::Mod => self.generate_mod(left, right),
            BinaryOp::Eq => self.generate_eq(left, right),
            BinaryOp::Ne => self.generate_ne(left, right),
            BinaryOp::Lt => self.generate_lt(left, right),
            BinaryOp::Le => self.generate_le(left, right),
            BinaryOp::Gt => self.generate_gt(left, right),
            BinaryOp::Ge => self.generate_ge(left, right),
            BinaryOp::And => self.generate_and(left, right),
            BinaryOp::Or => self.generate_or(left, right),
        }
    }

    /// Generate unary operation on value
    fn generate_unary_op_value(&self, op: &UnaryOp, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match op {
            UnaryOp::Neg => self.generate_neg(value),
            UnaryOp::Not => self.generate_not(value),
        }
    }

    /// Generate add operation
    fn generate_add(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_add(left_int, right_int, "add").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_add(left_float, right_float, "fadd").into())
            }
            _ => Err(format!("Unsupported operand types for +: {:?}", left.get_type())),
        }
    }

    /// Generate subtract operation
    fn generate_sub(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_sub(left_int, right_int, "sub").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_sub(left_float, right_float, "fsub").into())
            }
            _ => Err(format!("Unsupported operand types for -: {:?}", left.get_type())),
        }
    }

    /// Generate multiply operation
    fn generate_mul(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_mul(left_int, right_int, "mul").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_mul(left_float, right_float, "fmul").into())
            }
            _ => Err(format!("Unsupported operand types for *: {:?}", left.get_type())),
        }
    }

    /// Generate divide operation
    fn generate_div(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_signed_div(left_int, right_int, "div").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_div(left_float, right_float, "fdiv").into())
            }
            _ => Err(format!("Unsupported operand types for /: {:?}", left.get_type())),
        }
    }

    /// Generate modulo operation
    fn generate_mod(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_signed_rem(left_int, right_int, "mod").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_rem(left_float, right_float, "fmod").into())
            }
            _ => Err(format!("Unsupported operand types for %: {:?}", left.get_type())),
        }
    }

    /// Generate equality comparison
    fn generate_eq(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::EQ, left_int, right_int, "eq").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, left_float, right_float, "feq").into())
            }
            _ => Err(format!("Unsupported operand types for ==: {:?}", left.get_type())),
        }
    }

    /// Generate inequality comparison
    fn generate_ne(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::NE, left_int, right_int, "ne").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::ONE, left_float, right_float, "fne").into())
            }
            _ => Err(format!("Unsupported operand types for !=: {:?}", left.get_type())),
        }
    }

    /// Generate less than comparison
    fn generate_lt(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::SLT, left_int, right_int, "lt").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OLT, left_float, right_float, "flt").into())
            }
            _ => Err(format!("Unsupported operand types for <: {:?}", left.get_type())),
        }
    }

    /// Generate less or equal comparison
    fn generate_le(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::SLE, left_int, right_int, "le").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OLE, left_float, right_float, "fle").into())
            }
            _ => Err(format!("Unsupported operand types for <=: {:?}", left.get_type())),
        }
    }

    /// Generate greater than comparison
    fn generate_gt(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::SGT, left_int, right_int, "gt").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OGT, left_float, right_float, "fgt").into())
            }
            _ => Err(format!("Unsupported operand types for >: {:?}", left.get_type())),
        }
    }

    /// Generate greater or equal comparison
    fn generate_ge(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match left {
            BasicValueEnum::IntValue(left_int) => {
                let right_int = right.into_int_value();
                Ok(self.builder.build_int_compare(inkwell::IntPredicate::SGE, left_int, right_int, "ge").into())
            }
            BasicValueEnum::FloatValue(left_float) => {
                let right_float = right.into_float_value();
                Ok(self.builder.build_float_compare(inkwell::FloatPredicate::OGE, left_float, right_float, "fge").into())
            }
            _ => Err(format!("Unsupported operand types for >=: {:?}", left.get_type())),
        }
    }

    /// Generate logical and
    fn generate_and(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        let left_bool = left.into_int_value();
        let right_bool = right.into_int_value();
        Ok(self.builder.build_and(left_bool, right_bool, "and").into())
    }

    /// Generate logical or
    fn generate_or(&self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        let left_bool = left.into_int_value();
        let right_bool = right.into_int_value();
        Ok(self.builder.build_or(left_bool, right_bool, "or").into())
    }

    /// Generate unary negation
    fn generate_neg(&self, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            BasicValueEnum::IntValue(int_val) => {
                Ok(self.builder.build_int_neg(int_val, "neg").into())
            }
            BasicValueEnum::FloatValue(float_val) => {
                Ok(self.builder.build_float_neg(float_val, "fneg").into())
            }
            _ => Err(format!("Unsupported operand type for unary -: {:?}", value.get_type())),
        }
    }

    /// Generate logical not
    fn generate_not(&self, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        let bool_val = value.into_int_value();
        Ok(self.builder.build_not(bool_val, "not").into())
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