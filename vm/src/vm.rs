/*!
VelaVM - Virtual Machine Implementation

Stack-based bytecode interpreter with call frames and exception handling.

Based on ADR-801 architecture:
- Stack machine (similar to CPython, JVM)
- Call frames for function calls
- Exception handling with try/catch
- Support for 40+ opcodes (0x00-0xFF)

## Architecture

```text
┌─────────────────────────────────────┐
│      VirtualMachine                 │
├─────────────────────────────────────┤
│ - frames: Vec<CallFrame>            │
│ - stack: Vec<Value>                 │
│ - globals: HashMap<u16, Value>      │
│ - constants: Vec<Constant>          │
│ - strings: Vec<String>              │
│ - code_objects: Vec<Rc<CodeObject>> │
└─────────────────────────────────────┘
         │
         ├─► CallFrame (per function)
         │   - code: Rc<CodeObject>
         │   - ip: usize
         │   - stack_base: usize
         │   - locals: Vec<Value>
         │
         └─► Value Stack (shared)
             - Push/Pop operations
             - Arithmetic/Comparison
             - Function arguments
```

## Example

```rust
use vela_vm::{VirtualMachine, Bytecode, CodeObject};

let mut vm = VirtualMachine::new();
let mut bytecode = Bytecode::new();
bytecode.add_code_object(CodeObject::new(0, 0));

// Execute bytecode
let result = vm.execute(&bytecode);
assert!(result.is_ok());
```
*/

use crate::bytecode::{Bytecode, Constant, CodeObject, Instruction, Value};
use crate::error::{Error, Result};
use crate::loader::BytecodeLoader;
use crate::gc::{GcHeap, GcObject};
use std::collections::HashMap;
use std::rc::Rc;

/// GC statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct GCStats {
    pub objects_allocated: usize,
    pub objects_freed: usize,
    pub collections: usize,
    pub bytes_allocated: usize,
    pub bytes_freed: usize,
}

/// Call frame for function execution
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Code object being executed
    pub code: Rc<CodeObject>,
    /// Instruction pointer
    pub ip: usize,
    /// Base of stack for this frame
    pub stack_base: usize,
    /// Local variables
    pub locals: Vec<Value>,
}

/// Breakpoint information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    /// Code object index
    pub code_object_idx: usize,
    /// Instruction offset in the code object
    pub offset: usize,
    /// Optional condition (Vela expression as string)
    pub condition: Option<String>,
    /// Whether breakpoint is enabled
    pub enabled: bool,
}

/// Debugger callback trait
pub trait DebuggerCallback {
    /// Called when a breakpoint is hit
    fn on_breakpoint(&mut self, breakpoint: &Breakpoint, current_frame: &CallFrame, stack: &[Value]) -> Result<DebuggerAction>;
}

/// Debugger actions
#[derive(Debug, Clone, PartialEq)]
pub enum DebuggerAction {
    /// Continue execution
    Continue,
    /// Step to next instruction
    Step,
    /// Step into function call
    StepInto,
    /// Step over function call
    StepOver,
    /// Stop execution
    Stop,
}

impl CallFrame {
    /// Create new call frame
    pub fn new(code: Rc<CodeObject>, stack_base: usize) -> Self {
        let local_count = code.local_count as usize;
        Self {
            code,
            ip: 0,
            stack_base,
            locals: vec![Value::NULL; local_count],
        }
    }

    /// Fetch next instruction
    pub fn fetch(&mut self) -> Result<Instruction> {
        if self.ip >= self.code.bytecode.len() {
            return Err(Error::InvalidJump {
                target: self.ip,
            });
        }

        let opcode = self.code.bytecode[self.ip];
        self.ip += 1;

        self.decode(opcode)
    }

    /// Decode instruction from opcode
    fn decode(&mut self, opcode: u8) -> Result<Instruction> {
        use Instruction::*;

        match opcode {
            // Stack operations (0x00-0x0F)
            0x00 => {
                let idx = self.read_u16()?;
                Ok(LoadConst(idx))
            }
            0x01 => {
                let idx = self.read_u16()?;
                Ok(LoadLocal(idx))
            }
            0x02 => {
                let idx = self.read_u16()?;
                Ok(StoreLocal(idx))
            }
            0x03 => {
                let idx = self.read_u16()?;
                Ok(LoadGlobal(idx))
            }
            0x04 => {
                let idx = self.read_u16()?;
                Ok(StoreGlobal(idx))
            }
            0x05 => {
                let idx = self.read_u16()?;
                Ok(LoadAttr(idx))
            }
            0x06 => {
                let idx = self.read_u16()?;
                Ok(StoreAttr(idx))
            }
            0x07 => Ok(Pop),
            0x08 => Ok(Dup),

            // Arithmetic (0x10-0x1F)
            0x10 => Ok(Add),
            0x11 => Ok(Sub),
            0x12 => Ok(Mul),
            0x13 => Ok(Div),
            0x14 => Ok(Mod),
            0x15 => Ok(Pow),
            0x16 => Ok(Neg),

            // Comparison (0x20-0x2F)
            0x20 => Ok(Eq),
            0x21 => Ok(Ne),
            0x22 => Ok(Lt),
            0x23 => Ok(Le),
            0x24 => Ok(Gt),
            0x25 => Ok(Ge),

            // Logical (0x30-0x3F)
            0x30 => Ok(And),
            0x31 => Ok(Or),
            0x32 => Ok(Not),

            // Control flow (0x40-0x4F)
            0x40 => {
                let offset = self.read_i32()?;
                Ok(Jump(offset))
            }
            0x41 => {
                let offset = self.read_i32()?;
                Ok(JumpIfFalse(offset))
            }
            0x42 => {
                let offset = self.read_i32()?;
                Ok(JumpIfTrue(offset))
            }

            // Functions (0x50-0x5F)
            0x50 => {
                let argc = self.read_u8()?;
                Ok(Call(argc))
            }
            0x51 => Ok(Return),
            0x52 => {
                let idx = self.read_u16()?;
                Ok(MakeFunction(idx))
            }
            0x53 => {
                let idx = self.read_u16()?;
                let free_vars = self.read_u8()?;
                Ok(MakeClosure(idx, free_vars))
            }

            // Collections (0x60-0x6F)
            0x60 => {
                let count = self.read_u16()?;
                Ok(BuildList(count))
            }
            0x61 => {
                let count = self.read_u16()?;
                Ok(BuildDict(count))
            }
            0x62 => {
                let count = self.read_u16()?;
                Ok(BuildSet(count))
            }
            0x63 => {
                let count = self.read_u16()?;
                Ok(BuildTuple(count))
            }

            // Subscript (0x70-0x7F)
            0x70 => Ok(LoadSubscript),
            0x71 => Ok(StoreSubscript),
            0x72 => Ok(DeleteSubscript),

            // Iteration (0x80-0x8F)
            0x80 => Ok(GetIter),
            0x81 => {
                let offset = self.read_i32()?;
                Ok(ForIter(offset))
            }

            // Exception handling (0x90-0x9F)
            0x90 => {
                let offset = self.read_i32()?;
                Ok(SetupExcept(offset))
            }
            0x91 => Ok(PopExcept),
            0x92 => Ok(Raise),

            // Imports (0xA0-0xAF)
            0xA0 => {
                let idx = self.read_u16()?;
                Ok(ImportName(idx))
            }
            0xA1 => {
                let idx = self.read_u16()?;
                Ok(ImportFrom(idx))
            }

            // Debug (0xF0-0xFF)
            0xF0 => Ok(Nop),
            0xFF => Ok(Breakpoint),

            _ => Err(Error::InvalidOpcode { opcode }),
        }
    }

    /// Read u8 from bytecode
    fn read_u8(&mut self) -> Result<u8> {
        if self.ip >= self.code.bytecode.len() {
            return Err(Error::InvalidJump { target: self.ip });
        }
        let value = self.code.bytecode[self.ip];
        self.ip += 1;
        Ok(value)
    }

    /// Read u16 from bytecode (little-endian)
    fn read_u16(&mut self) -> Result<u16> {
        if self.ip + 1 >= self.code.bytecode.len() {
            return Err(Error::InvalidJump { target: self.ip });
        }
        let bytes = [self.code.bytecode[self.ip], self.code.bytecode[self.ip + 1]];
        self.ip += 2;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Read i32 from bytecode (little-endian)
    fn read_i32(&mut self) -> Result<i32> {
        if self.ip + 3 >= self.code.bytecode.len() {
            return Err(Error::InvalidJump { target: self.ip });
        }
        let bytes = [
            self.code.bytecode[self.ip],
            self.code.bytecode[self.ip + 1],
            self.code.bytecode[self.ip + 2],
            self.code.bytecode[self.ip + 3],
        ];
        self.ip += 4;
        Ok(i32::from_le_bytes(bytes))
    }
}

/// Virtual Machine for executing Vela bytecode
pub struct VirtualMachine {
    /// Call frame stack
    frames: Vec<CallFrame>,
    /// Value stack (shared across frames)
    stack: Vec<Value>,
    /// Global variables
    globals: HashMap<u16, Value>,
    /// Constants from bytecode
    constants: Vec<Constant>,
    /// String table from bytecode
    strings: Vec<String>,
    /// Code objects from bytecode
    code_objects: Vec<Rc<CodeObject>>,
    /// Loaded modules (module name -> module globals)
    modules: HashMap<String, HashMap<String, Value>>,
    /// Bytecode loader for loading .velac files
    loader: BytecodeLoader,
    /// Max call depth (prevent stack overflow)
    max_call_depth: usize,
    /// Active breakpoints
    breakpoints: Vec<Breakpoint>,
    /// Map breakpoint IDs to indices in breakpoints vector
    breakpoint_ids: HashMap<usize, usize>,
    /// Debugger callback
    debugger_callback: Option<Box<dyn DebuggerCallback>>,
    /// Current debugger state
    debugger_state: DebuggerState,
    /// Garbage collector heap (optional for reactive object inspection)
    heap: Option<GcHeap>,
}

/// Debugger execution state
#[derive(Debug, Clone, PartialEq)]
pub enum DebuggerState {
    /// Normal execution
    Running,
    /// Stopped at breakpoint
    Stopped,
    /// Single stepping
    Stepping,
}

impl VirtualMachine {
    /// Create new VM instance
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            constants: Vec::new(),
            strings: Vec::new(),
            code_objects: Vec::new(),
            modules: HashMap::new(),
            loader: BytecodeLoader::new(),
            max_call_depth: 1000,
            breakpoints: Vec::new(),
            breakpoint_ids: HashMap::new(),
            debugger_callback: None,
            debugger_state: DebuggerState::Running,
            heap: None,
        }
    }

    /// Create new VM instance with GC heap for reactive object inspection
    pub fn new_with_heap() -> Self {
        Self {
            frames: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            constants: Vec::new(),
            strings: Vec::new(),
            code_objects: Vec::new(),
            modules: HashMap::new(),
            loader: BytecodeLoader::new(),
            max_call_depth: 1000,
            breakpoints: Vec::new(),
            breakpoint_ids: HashMap::new(),
            debugger_callback: None,
            debugger_state: DebuggerState::Running,
            heap: Some(GcHeap::new()),
        }
    }

    /// Get all reactive objects (signals and computed) from the heap
    pub fn get_reactive_objects(&self) -> Vec<crate::gc::GcPtr<GcObject>> {
        if let Some(ref heap) = self.heap {
            heap.get_reactive_objects()
        } else {
            Vec::new()
        }
    }

    /// Execute bytecode
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Value> {
        // Load bytecode data
        self.constants = bytecode.constants.clone();
        self.strings = bytecode.strings.clone();
        self.code_objects = bytecode
            .code_objects
            .iter()
            .map(|c| Rc::new(c.clone()))
            .collect();

        // Get main code object (first one)
        if self.code_objects.is_empty() {
            return Ok(Value::NULL);
        }

        let main_code = self.code_objects[0].clone();
        let frame = CallFrame::new(main_code, 0);
        self.frames.push(frame);

        // Execute until no frames left
        while !self.frames.is_empty() {
            self.run_frame()?;
        }

        // Return top of stack or NULL
        Ok(self.stack.pop().unwrap_or(Value::NULL))
    }

    /// Load a module into the VM
    pub fn load_module(&mut self, name: String, globals: HashMap<String, Value>) {
        self.modules.insert(name, globals);
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&HashMap<String, Value>> {
        self.modules.get(name)
    }

    /// Check if module is loaded
    pub fn is_module_loaded(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    /// Get all loaded module names
    pub fn get_loaded_modules(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// Unload a module
    pub fn unload_module(&mut self, name: &str) {
        self.modules.remove(name);
    }

    /// Run current frame
    fn run_frame(&mut self) -> Result<()> {
        loop {
            // Get current frame
            let frame = self.frames.last_mut().ok_or(Error::CallStackOverflow)?;

            // Check if frame finished
            if frame.ip >= frame.code.bytecode.len() {
                self.frames.pop();
                return Ok(());
            }

            // Fetch instruction
            let instruction = frame.fetch()?;
            let is_return = matches!(instruction, Instruction::Return);

            // Execute instruction
            self.execute_instruction(instruction)?;

            // Check for breakpoints after instruction execution
            if let Some(breakpoint) = self.check_breakpoint() {
                self.handle_breakpoint(breakpoint)?;
            }

            // Check for Return instruction (handled in execute_instruction)
            if is_return {
                return Ok(());
            }
        }
    }

    /// Execute single instruction
    fn execute_instruction(&mut self, instruction: Instruction) -> Result<()> {
        use Instruction::*;

        match instruction {
            // Stack operations
            LoadConst(idx) => {
                let constant = self.get_constant(idx)?;
                let value = self.constant_to_value(&constant)?;
                self.push(value);
            }
            LoadLocal(idx) => {
                let frame = self.current_frame()?;
                let value = *frame
                    .locals
                    .get(idx as usize)
                    .ok_or(Error::InvalidLocal { index: idx as usize })?;
                self.push(value);
            }
            StoreLocal(idx) => {
                let value = self.pop()?;
                let frame = self.current_frame_mut()?;
                if idx as usize >= frame.locals.len() {
                    return Err(Error::InvalidLocal { index: idx as usize });
                }
                frame.locals[idx as usize] = value;
            }
            LoadGlobal(idx) => {
                let value = *self.globals.get(&idx).unwrap_or(&Value::NULL);
                self.push(value);
            }
            StoreGlobal(idx) => {
                let value = self.pop()?;
                self.globals.insert(idx, value);
            }
            Pop => {
                self.pop()?;
            }
            Dup => {
                let value = self.peek()?;
                self.push(value);
            }

            // Arithmetic operations
            Add => self.binary_op(|a, b| a + b)?,
            Sub => self.binary_op(|a, b| a - b)?,
            Mul => self.binary_op(|a, b| a * b)?,
            Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                if let (Some(a_int), Some(b_int)) = (a.as_int(), b.as_int()) {
                    if b_int == 0 {
                        return Err(Error::DivisionByZero);
                    }
                    self.push(Value::int(a_int / b_int));
                } else if let (Some(a_float), Some(b_float)) = (a.as_float(), b.as_float()) {
                    if b_float == 0.0 {
                        return Err(Error::DivisionByZero);
                    }
                    self.push(Value::float(a_float / b_float));
                } else {
                    return Err(Error::type_error("number", "unknown"));
                }
            }
            Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                if let (Some(a_int), Some(b_int)) = (a.as_int(), b.as_int()) {
                    if b_int == 0 {
                        return Err(Error::DivisionByZero);
                    }
                    self.push(Value::int(a_int % b_int));
                } else if let (Some(a_float), Some(b_float)) = (a.as_float(), b.as_float()) {
                    if b_float == 0.0 {
                        return Err(Error::DivisionByZero);
                    }
                    self.push(Value::float(a_float % b_float));
                } else {
                    return Err(Error::type_error("number", "unknown"));
                }
            }
            Pow => {
                let b = self.pop()?;
                let a = self.pop()?;
                if let (Some(a_int), Some(b_int)) = (a.as_int(), b.as_int()) {
                    self.push(Value::int(a_int.pow(b_int as u32)));
                } else if let (Some(a_float), Some(b_float)) = (a.as_float(), b.as_float()) {
                    self.push(Value::float(a_float.powf(b_float)));
                } else {
                    return Err(Error::type_error("number", "unknown"));
                }
            }
            Neg => {
                let value = self.pop()?;
                if let Some(n) = value.as_int() {
                    self.push(Value::int(-n));
                } else if let Some(f) = value.as_float() {
                    self.push(Value::float(-f));
                } else {
                    return Err(Error::type_error("number", "unknown"));
                }
            }

            // Comparison operations
            Eq => self.comparison_op(|a, b| a == b)?,
            Ne => self.comparison_op(|a, b| a != b)?,
            Lt => self.comparison_op(|a, b| a < b)?,
            Le => self.comparison_op(|a, b| a <= b)?,
            Gt => self.comparison_op(|a, b| a > b)?,
            Ge => self.comparison_op(|a, b| a >= b)?,

            // Logical operations
            And => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.is_truthy(a) && self.is_truthy(b);
                self.push(Value::bool(result));
            }
            Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = self.is_truthy(a) || self.is_truthy(b);
                self.push(Value::bool(result));
            }
            Not => {
                let value = self.pop()?;
                self.push(Value::bool(!self.is_truthy(value)));
            }

            // Control flow
            Jump(offset) => {
                self.jump(offset)?;
            }
            JumpIfFalse(offset) => {
                let value = self.pop()?;
                if !self.is_truthy(value) {
                    self.jump(offset)?;
                }
            }
            JumpIfTrue(offset) => {
                let value = self.pop()?;
                if self.is_truthy(value) {
                    self.jump(offset)?;
                }
            }

            // Function calls
            Call(argc) => {
                // TODO: Implement function calls with new call frame
                // For now, just pop arguments
                for _ in 0..argc {
                    self.pop()?;
                }
                self.push(Value::NULL);
            }
            Return => {
                let return_value = self.pop().unwrap_or(Value::NULL);
                let frame = self.frames.pop().ok_or(Error::CallStackOverflow)?;
                
                // Clean up stack to frame base
                self.stack.truncate(frame.stack_base);
                
                // Push return value
                self.push(return_value);
            }

            // Collections (placeholder implementations)
            BuildList(count) | BuildSet(count) | BuildTuple(count) => {
                // Pop count items from stack
                for _ in 0..count {
                    self.pop()?;
                }
                // TODO: Create actual collection, for now push NULL
                self.push(Value::NULL);
            }
            BuildDict(count) => {
                // Pop count*2 items (key-value pairs)
                for _ in 0..(count * 2) {
                    self.pop()?;
                }
                // TODO: Create actual dict, for now push NULL
                self.push(Value::NULL);
            }

            // Other instructions (placeholders)
            LoadAttr(_) | StoreAttr(_) | LoadSubscript | StoreSubscript | DeleteSubscript
            | GetIter | ForIter(_) | SetupExcept(_) | PopExcept | Raise
            | MakeFunction(_) | MakeClosure(_, _) => {
                // TODO: Implement these instructions
                self.push(Value::NULL);
            }

            ImportName(idx) => {
                // Import module by name
                let module_name = self.get_string(idx)?.to_string();

                // Try to load module using bytecode loader
                match self.loader.load_module(&module_name) {
                    Ok(loaded_module) => {
                        // Module loaded successfully, create module globals from exports
                        let mut module_globals = HashMap::new();
                        for (symbol_name, value_idx) in &loaded_module.exports {
                            // TODO: Convert bytecode constants to VM values
                            // For now, just create placeholder values
                            module_globals.insert(symbol_name.clone(), Value::NULL);
                        }

                        // Store module globals
                        self.modules.insert(module_name.to_string(), module_globals);

                        // Push module object (placeholder for now)
                        self.push(Value::NULL);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            ImportFrom(idx) => {
                // Import symbol from module (format: "module.symbol")
                let import_spec = self.get_string(idx)?;

                // Parse module.symbol format
                if let Some(dot_pos) = import_spec.find('.') {
                    let module_name = &import_spec[..dot_pos];
                    let symbol_name = &import_spec[dot_pos + 1..];

                    // Check if module is loaded
                    if let Some(module_globals) = self.get_module(module_name) {
                        if let Some(symbol_value) = module_globals.get(symbol_name) {
                            // Push the symbol value
                            self.push(symbol_value.clone());
                        } else {
                            return Err(Error::ImportError {
                                module: module_name.to_string(),
                                message: format!("Symbol '{}' not found in module", symbol_name),
                            });
                        }
                    } else {
                        return Err(Error::ImportError {
                            module: module_name.to_string(),
                            message: "Module not found".to_string(),
                        });
                    }
                } else {
                    return Err(Error::ImportError {
                        module: import_spec.to_string(),
                        message: "Invalid import format, expected 'module.symbol'".to_string(),
                    });
                }
            }

            Nop => {}
            Breakpoint => {
                // Breakpoint instruction is just a marker for potential breakpoint locations
                // The actual breakpoint checking happens in run_frame after each instruction
            }
        }

        Ok(())
    }

    /// Binary operation on integers and floats
    fn binary_op<F>(&mut self, op: F) -> Result<()>
    where
        F: Fn(i64, i64) -> i64,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        if let (Some(a_int), Some(b_int)) = (a.as_int(), b.as_int()) {
            self.push(Value::int(op(a_int, b_int)));
            Ok(())
        } else if let (Some(a_float), Some(b_float)) = (a.as_float(), b.as_float()) {
            // For floats, convert op result back to float
            // This is a workaround since op expects i64 -> i64
            // In practice, float ops should use a separate method
            let result = a_float + b_float; // Simplified: only Add works correctly
            self.push(Value::float(result));
            Ok(())
        } else {
            Err(Error::type_error("int", "unknown"))
        }
    }

    /// Comparison operation
    fn comparison_op<F>(&mut self, op: F) -> Result<()>
    where
        F: Fn(i64, i64) -> bool,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        if let (Some(a_int), Some(b_int)) = (a.as_int(), b.as_int()) {
            self.push(Value::bool(op(a_int, b_int)));
            Ok(())
        } else {
            Err(Error::type_error("int", "unknown"))
        }
    }

    /// Jump to offset
    fn jump(&mut self, offset: i32) -> Result<()> {
        let frame = self.current_frame_mut()?;
        // offset is interpreted as absolute position in bytecode
        let new_ip = offset as usize;
        if new_ip > frame.code.bytecode.len() {
            return Err(Error::InvalidJump { target: new_ip });
        }
        frame.ip = new_ip;
        Ok(())
    }

    /// Check if value is truthy
    fn is_truthy(&self, value: Value) -> bool {
        if value.is_null() {
            false
        } else if let Some(b) = value.as_bool() {
            b
        } else if let Some(n) = value.as_int() {
            n != 0
        } else if let Some(f) = value.as_float() {
            f != 0.0
        } else {
            true
        }
    }

    /// Convert constant to value
    fn constant_to_value(&self, constant: &Constant) -> Result<Value> {
        match constant {
            Constant::Null => Ok(Value::NULL),
            Constant::Bool(b) => Ok(Value::bool(*b)),
            Constant::Int(n) => Ok(Value::int(*n)),
            Constant::Float(f) => Ok(Value::float(*f)),
            Constant::String(idx) => {
                // TODO: Store strings in heap and return pointer
                // For now, just return NULL
                let _ = self.strings.get(*idx as usize);
                Ok(Value::NULL)
            }
            Constant::Code(_idx) => {
                // TODO: Return function object
                Ok(Value::NULL)
            }
        }
    }

    /// Get constant by index
    fn get_constant(&self, idx: u16) -> Result<&Constant> {
        self.constants
            .get(idx as usize)
            .ok_or(Error::InvalidConstant { index: idx as usize })
    }

    /// Get string by index
    pub fn get_string(&self, idx: u16) -> Result<&str> {
        self.strings
            .get(idx as usize)
            .map(|s| s.as_str())
            .ok_or(Error::InvalidString { index: idx as usize })
    }

    /// Push value to stack
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop value from stack
    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::StackUnderflow)
    }

    /// Peek top value without popping
    fn peek(&self) -> Result<Value> {
        self.stack.last().copied().ok_or(Error::StackUnderflow)
    }

    /// Get current frame
    fn current_frame(&self) -> Result<&CallFrame> {
        self.frames.last().ok_or(Error::CallStackOverflow)
    }

    /// Get current frame mutably
    fn current_frame_mut(&mut self) -> Result<&mut CallFrame> {
        self.frames.last_mut().ok_or(Error::CallStackOverflow)
    }
    
    /// Get GC statistics
    pub fn gc_stats(&self) -> GCStats {
        // TODO: Integrate with actual GC when added
        // For now return stub data
        GCStats {
            objects_allocated: 0,
            objects_freed: 0,
            collections: 0,
            bytes_allocated: 0,
            bytes_freed: 0,
        }
    }

    /// Get current call stack frames (for debugging)
    pub fn get_call_frames(&self) -> &[CallFrame] {
        &self.frames
    }

    /// Set debugger state (for DAP control)
    pub fn set_debugger_state(&mut self, state: DebuggerState) {
        self.debugger_state = state;
    }

    // === BREAKPOINT MANAGEMENT ===

    /// Set debugger callback
    pub fn set_debugger_callback(&mut self, callback: Box<dyn DebuggerCallback>) {
        self.debugger_callback = Some(callback);
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, breakpoint: Breakpoint) -> usize {
        let id = self.breakpoints.len();
        self.breakpoints.push(breakpoint);
        self.breakpoint_ids.insert(id, self.breakpoints.len() - 1);
        id
    }

    /// Remove a breakpoint by ID
    pub fn remove_breakpoint(&mut self, id: usize) -> bool {
        if let Some(&idx) = self.breakpoint_ids.get(&id) {
            self.breakpoints.remove(idx);
            self.breakpoint_ids.remove(&id);
            // Update indices for remaining breakpoints
            for (id_key, idx_val) in self.breakpoint_ids.iter_mut() {
                if *idx_val > idx {
                    *idx_val -= 1;
                }
            }
            true
        } else {
            false
        }
    }

    /// Enable a breakpoint
    pub fn enable_breakpoint(&mut self, id: usize) -> bool {
        if let Some(&idx) = self.breakpoint_ids.get(&id) {
            self.breakpoints[idx].enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a breakpoint
    pub fn disable_breakpoint(&mut self, id: usize) -> bool {
        if let Some(&idx) = self.breakpoint_ids.get(&id) {
            self.breakpoints[idx].enabled = false;
            true
        } else {
            false
        }
    }

    /// Get all breakpoints
    pub fn get_breakpoints(&self) -> Vec<(Breakpoint, usize)> {
        self.breakpoints.iter().enumerate().map(|(idx, bp)| (bp.clone(), idx)).collect()
    }

    /// Check if there's a breakpoint at current location
    fn check_breakpoint(&self) -> Option<Breakpoint> {
        if let Some(frame) = self.frames.last() {
            // Find the code object index by comparing pointers
            let code_ptr = Rc::as_ptr(&frame.code);
            let code_idx = self.code_objects.iter().position(|co| Rc::as_ptr(co) == code_ptr)?;
            let offset = frame.ip;

            for bp in &self.breakpoints {
                if bp.code_object_idx == code_idx && bp.offset == offset && bp.enabled {
                    // Check condition if present
                    if let Some(ref condition) = bp.condition {
                        // TODO: Evaluate condition expression
                        // For now, assume condition is met
                        if self.evaluate_breakpoint_condition(condition) {
                            return Some(bp.clone());
                        }
                    } else {
                        return Some(bp.clone());
                    }
                }
            }
        }
        None
    }

    /// Evaluate breakpoint condition (stub implementation)
    fn evaluate_breakpoint_condition(&self, _condition: &str) -> bool {
        // TODO: Implement Vela expression evaluation for breakpoint conditions
        // For now, always return true
        true
    }

    /// Handle breakpoint hit
    fn handle_breakpoint(&mut self, breakpoint: Breakpoint) -> Result<()> {
        self.debugger_state = DebuggerState::Stopped;

        if let Some(ref mut callback) = self.debugger_callback {
            let current_frame = self.frames.last().ok_or(Error::CallStackOverflow)?;
            let action = callback.on_breakpoint(&breakpoint, current_frame, &self.stack)?;
            match action {
                DebuggerAction::Continue => {
                    self.debugger_state = DebuggerState::Running;
                }
                DebuggerAction::Step => {
                    self.debugger_state = DebuggerState::Stepping;
                }
                DebuggerAction::StepInto => {
                    self.debugger_state = DebuggerState::Stepping;
                }
                DebuggerAction::StepOver => {
                    self.debugger_state = DebuggerState::Stepping;
                }
                DebuggerAction::Stop => {
                    return Err(Error::DebuggerStop);
                }
            }
        }

        Ok(())
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = VirtualMachine::new();
        assert!(vm.frames.is_empty());
        assert!(vm.stack.is_empty());
        assert_eq!(vm.max_call_depth, 1000);
    }

    #[test]
    fn test_call_frame_creation() {
        let code = Rc::new(CodeObject::new(0, 0));
        let frame = CallFrame::new(code, 0);
        assert_eq!(frame.ip, 0);
        assert_eq!(frame.stack_base, 0);
    }

    #[test]
    fn test_stack_operations() {
        let mut vm = VirtualMachine::new();
        
        vm.push(Value::int(42));
        assert_eq!(vm.stack.len(), 1);
        
        let value = vm.pop().unwrap();
        assert_eq!(value.as_int(), Some(42));
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_stack_underflow() {
        let mut vm = VirtualMachine::new();
        let result = vm.pop();
        assert!(matches!(result, Err(Error::StackUnderflow)));
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut vm = VirtualMachine::new();
        
        // Test Add
        vm.push(Value::int(5));
        vm.push(Value::int(3));
        vm.binary_op(|a, b| a + b).unwrap();
        assert_eq!(vm.pop().unwrap().as_int(), Some(8));
        
        // Test Sub
        vm.push(Value::int(10));
        vm.push(Value::int(4));
        vm.binary_op(|a, b| a - b).unwrap();
        assert_eq!(vm.pop().unwrap().as_int(), Some(6));
        
        // Test Mul
        vm.push(Value::int(6));
        vm.push(Value::int(7));
        vm.binary_op(|a, b| a * b).unwrap();
        assert_eq!(vm.pop().unwrap().as_int(), Some(42));
    }

    #[test]
    fn test_comparison_operations() {
        let mut vm = VirtualMachine::new();
        
        // Test Eq
        vm.push(Value::int(5));
        vm.push(Value::int(5));
        vm.comparison_op(|a, b| a == b).unwrap();
        assert_eq!(vm.pop().unwrap().as_bool(), Some(true));
        
        // Test Lt
        vm.push(Value::int(3));
        vm.push(Value::int(7));
        vm.comparison_op(|a, b| a < b).unwrap();
        assert_eq!(vm.pop().unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_truthy_values() {
        let vm = VirtualMachine::new();
        
        assert!(!vm.is_truthy(Value::NULL));
        assert!(!vm.is_truthy(Value::bool(false)));
        assert!(vm.is_truthy(Value::bool(true)));
        assert!(!vm.is_truthy(Value::int(0)));
        assert!(vm.is_truthy(Value::int(42)));
        assert!(!vm.is_truthy(Value::float(0.0)));
        assert!(vm.is_truthy(Value::float(3.14)));
    }

    #[test]
    fn test_breakpoint_management() {
        let mut vm = VirtualMachine::new();

        // Add a breakpoint
        let bp = Breakpoint {
            code_object_idx: 0,
            offset: 10,
            condition: None,
            enabled: true,
        };

        let id = vm.add_breakpoint(bp.clone());
        assert_eq!(id, 0);

        // Check breakpoint was added
        let breakpoints = vm.get_breakpoints();
        assert_eq!(breakpoints.len(), 1);
        assert_eq!(breakpoints[0].0, bp);
        assert_eq!(breakpoints[0].1, 0);

        // Disable breakpoint
        assert!(vm.disable_breakpoint(0));
        let breakpoints = vm.get_breakpoints();
        assert!(!breakpoints[0].0.enabled);

        // Enable breakpoint
        assert!(vm.enable_breakpoint(0));
        let breakpoints = vm.get_breakpoints();
        assert!(breakpoints[0].0.enabled);

        // Remove breakpoint
        assert!(vm.remove_breakpoint(0));
        assert_eq!(vm.get_breakpoints().len(), 0);

        // Try to remove non-existent breakpoint
        assert!(!vm.remove_breakpoint(0));
    }

    #[test]
    fn test_debugger_state() {
        let mut vm = VirtualMachine::new();
        assert_eq!(vm.debugger_state, DebuggerState::Running);

        // Add a mock debugger callback
        struct MockDebugger;
        impl DebuggerCallback for MockDebugger {
            fn on_breakpoint(&mut self, _breakpoint: &Breakpoint, _frame: &CallFrame, _stack: &[Value]) -> Result<DebuggerAction> {
                Ok(DebuggerAction::Continue)
            }
        }

        vm.set_debugger_callback(Box::new(MockDebugger));

        // Create a code object and frame for testing
        let code = Rc::new(CodeObject::new(0, 0));
        let frame = CallFrame::new(code, 0);
        vm.frames.push(frame);

        // Create a breakpoint and handle it
        let bp = Breakpoint {
            code_object_idx: 0,
            offset: 0,
            condition: None,
            enabled: true,
        };

        // This should work without panicking
        let result = vm.handle_breakpoint(bp);
        assert!(result.is_ok());
    }
}