//! DAP (Debug Adapter Protocol) implementation for Vela
//!
//! This module implements the Debug Adapter Protocol as specified by Microsoft,
//! allowing Vela programs to be debugged in editors like VS Code.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use vela_vm::{VirtualMachine, vm::{Breakpoint, DebuggerCallback, DebuggerAction, CallFrame}, bytecode::Value as VmValue};
use vela_vm::vm::DebuggerState;

use vela_compiler::debug_info::DebugInfo;
use vela_compiler::Compiler;

/// DAP Server for Vela debugging
pub struct DapServer {
    vm: Arc<Mutex<VirtualMachine>>,
    breakpoints: HashMap<String, Vec<Breakpoint>>,
    next_seq: u32,
    initialized: bool,
    debug_info: Option<DebugInfo>,
}

impl DapServer {
    /// Create a new DAP server
    pub fn new() -> Self {
        Self {
            vm: Arc::new(Mutex::new(VirtualMachine::new())),
            breakpoints: HashMap::new(),
            next_seq: 1,
            initialized: false,
            debug_info: None,
        }
    }

    /// Start the DAP server, reading from stdin and writing to stdout
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let mut reader = BufReader::new(stdin);

        loop {
            let mut line = String::new();
            reader.read_line(&mut line)?;

            if line.trim().is_empty() {
                continue;
            }

            // Parse DAP message
            let message: DapMessage = serde_json::from_str(&line)?;

            // Handle the message
            let response = self.handle_message(message)?;

            // Send response
            if let Some(response) = response {
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    /// Handle incoming DAP message
    fn handle_message(&mut self, message: DapMessage) -> Result<Option<DapMessage>, Box<dyn std::error::Error>> {
        match message {
            DapMessage::Request(request) => {
                let response = self.handle_request(request)?;
                Ok(Some(DapMessage::Response(response)))
            }
            DapMessage::Event(_) => {
                // Events are sent by the server, not received
                Ok(None)
            }
            DapMessage::Response(_) => {
                // Responses are sent by the server, not received
                Ok(None)
            }
        }
    }

    /// Handle DAP request
    pub fn handle_request(&mut self, request: Request) -> Result<Response, Box<dyn std::error::Error>> {
        let seq = self.next_seq;
        self.next_seq += 1;

        match request.command.as_str() {
            "initialize" => {
                self.initialized = true;
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: Some(serde_json::json!({
                        "supportsConfigurationDoneRequest": true,
                        "supportsSetVariable": true,
                        "supportsEvaluateForHovers": true,
                        "supportsStepBack": false,
                        "supportsRestartFrame": false,
                        "supportsConditionalBreakpoints": true,
                        "supportsHitConditionalBreakpoints": true,
                        "supportsLogPoints": true,
                        "supportsFunctionBreakpoints": false,
                        "supportsExceptionInfoRequest": true,
                        "supportsExceptionOptions": false,
                        "supportsCompletionsRequest": false,
                        "supportsModulesRequest": false,
                        "supportsRestartRequest": false,
                        "supportsSetExpression": false,
                        "supportsTerminateRequest": true,
                        "supportsGotoTargetsRequest": false,
                        "supportsStepInTargetsRequest": false,
                        "supportsReadMemoryRequest": false,
                        "supportsWriteMemoryRequest": false,
                        "supportsDisassembleRequest": false,
                        "supportsCancelRequest": false,
                        "supportsBreakpointLocationsRequest": false,
                        "supportsClipboardContext": false,
                        "supportsSteppingGranularity": false,
                        "supportsInstructionBreakpoints": false,
                        "supportsExceptionFilterOptions": false,
                        "exceptionBreakpointFilters": []
                    })),
                    message: None,
                })
            }

            "launch" => {
                // Launch the Vela program
                let args: LaunchRequestArguments = serde_json::from_value(request.arguments.unwrap_or(Value::Null))?;

                // Load and execute the Vela program
                let program_path = args.program.clone();
                let source_code = std::fs::read_to_string(&program_path)
                    .map_err(|e| format!("Failed to read program file {}: {}", program_path, e))?;

                // Compile the Vela source code
                let mut compiler = vela_compiler::Compiler::default();
                let bytecode_bytes = compiler.compile_string(&source_code, &program_path)
                    .map_err(|e| format!("Compilation failed: {:?}", e))?;

                // Deserialize bytecode
                let bytecode = vela_vm::bytecode::Bytecode::from_bytes(&bytecode_bytes)
                    .map_err(|e| format!("Failed to deserialize bytecode: {}", e))?;

                // Store debug info for breakpoint mapping
                // TODO: Get debug info from compiler
                // self.debug_info = compiler.get_debug_info().cloned();

                // Execute the program synchronously for now
                // TODO: Implement proper async execution with debugger integration
                {
                    let mut vm = self.vm.lock().unwrap();
                    if let Err(e) = vm.execute(&bytecode) {
                        eprintln!("VM execution error: {:?}", e);
                    }
                }

                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: None,
                    message: None,
                })
            }

            "attach" => {
                // Attach to a running Vela process
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: None,
                    message: None,
                })
            }

            "setBreakpoints" => {
                let args: SetBreakpointsArguments = serde_json::from_value(request.arguments.unwrap_or(Value::Null))?;
                let breakpoints = self.set_breakpoints(args)?;

                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: Some(serde_json::json!({
                        "breakpoints": breakpoints
                    })),
                    message: None,
                })
            }

            "configurationDone" => {
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: None,
                    message: None,
                })
            }

            "threads" => {
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: Some(serde_json::json!({
                        "threads": [
                            {
                                "id": 1,
                                "name": "main"
                            }
                        ]
                    })),
                    message: None,
                })
            }

            "stackTrace" => {
                let stack_frames = self.get_stack_trace()?;
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: Some(serde_json::json!({
                        "stackFrames": stack_frames
                    })),
                    message: None,
                })
            }

            "scopes" => {
                let args: ScopesArguments = serde_json::from_value(request.arguments.unwrap_or(Value::Null))?;
                let scopes = self.get_scopes(args.frame_id)?;

                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: Some(serde_json::json!({
                        "scopes": scopes
                    })),
                    message: None,
                })
            }

            "variables" => {
                let args: VariablesArguments = serde_json::from_value(request.arguments.unwrap_or(Value::Null))?;
                let variables = self.get_variables(args.variables_reference)?;

                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: Some(serde_json::json!({
                        "variables": variables
                    })),
                    message: None,
                })
            }

            "continue" => {
                self.continue_execution()?;
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: Some(serde_json::json!({
                        "allThreadsContinued": true
                    })),
                    message: None,
                })
            }

            "next" => {
                self.step_over()?;
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: None,
                    message: None,
                })
            }

            "stepIn" => {
                self.step_in()?;
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: None,
                    message: None,
                })
            }

            "stepOut" => {
                self.step_out()?;
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: None,
                    message: None,
                })
            }

            "disconnect" => {
                Ok(Response {
                    request_seq: request.seq,
                    success: true,
                    command: request.command,
                    body: None,
                    message: None,
                })
            }

            _ => {
                let command = request.command.clone();
                Ok(Response {
                    request_seq: request.seq,
                    success: false,
                    command: command,
                    body: None,
                    message: Some(format!("Unknown command: {}", request.command)),
                })
            }
        }
    }

    /// Map bytecode offset back to source line using debug info
    fn map_offset_to_line(&self, code_object_name: &str, offset: usize) -> (String, i32) {
        if let Some(debug_info) = &self.debug_info {
            // Try to find the source file for this code object
            for (source_path, source_map) in &debug_info.source_maps {
                // Find the closest line mapping for this offset
                let mut closest_line = 1;
                for mapping in &source_map.line_mappings {
                    if mapping.bytecode_offset <= offset {
                        closest_line = mapping.source_line as i32;
                    } else {
                        break;
                    }
                }
                return (source_path.clone(), closest_line);
            }
        }
        // Fallback: use code object name as source and offset as line
        (code_object_name.to_string(), offset as i32)
    }

    /// Map source line to bytecode offset using debug info
    fn map_line_to_offset(&self, source_path: &str, line: usize) -> usize {
        if let Some(debug_info) = &self.debug_info {
            if let Some(source_map) = debug_info.source_maps.get(source_path) {
                // Find the closest bytecode offset for this line
                for mapping in &source_map.line_mappings {
                    if mapping.source_line >= line {
                        return mapping.bytecode_offset;
                    }
                }
                // If no exact match, return the last offset
                if let Some(last_mapping) = source_map.line_mappings.last() {
                    return last_mapping.bytecode_offset;
                }
            }
        }
        // Fallback: use line number as offset
        line
    }

    /// Set breakpoints
    fn set_breakpoints(&mut self, args: SetBreakpointsArguments) -> Result<Vec<BreakpointInfo>, Box<dyn std::error::Error>> {
        let source_path = args.source.path.clone().unwrap_or_else(|| "unknown".to_string());

        // Clear existing breakpoints for this file
        self.breakpoints.remove(&source_path);

        let mut breakpoint_infos = Vec::new();

        for bp_args in args.breakpoints {
            // Map source line to bytecode offset
            let bytecode_offset = self.map_line_to_offset(&source_path, bp_args.line as usize);

            let breakpoint = Breakpoint {
                code_object_idx: 0, // TODO: Map to actual code object
                offset: bytecode_offset,
                condition: bp_args.condition,
                enabled: true,
            };

            let id = {
                let mut vm = self.vm.lock().unwrap();
                vm.add_breakpoint(breakpoint)
            };

            breakpoint_infos.push(BreakpointInfo {
                id: Some(id as i32),
                verified: true,
                message: None,
                source: Some(args.source.clone()),
                line: Some(bp_args.line),
                column: Some(1),
                end_line: None,
                end_column: None,
            });
        }

        Ok(breakpoint_infos)
    }

    /// Get current stack trace
    fn get_stack_trace(&self) -> Result<Vec<StackFrame>, Box<dyn std::error::Error>> {
        let vm = self.vm.lock().unwrap();
        let frames = vm.get_call_frames();

        let mut stack_frames = Vec::new();
        for (i, frame) in frames.iter().enumerate() {
            // Map bytecode offset back to source line
            let (source_path, line) = self.map_offset_to_line(&format!("function_{}", frame.code.name), frame.ip);

            stack_frames.push(StackFrame {
                id: i as i32,
                name: format!("function_{}", frame.code.name),
                source: Some(Source {
                    name: Some(format!("function_{}", frame.code.name)),
                    path: Some(source_path),
                    source_reference: None,
                    presentation_hint: None,
                    origin: None,
                    sources: None,
                    adapter_data: None,
                    checksums: None,
                }),
                line,
                column: 1,
                end_line: None,
                end_column: None,
                can_restart: Some(false),
                instruction_pointer_reference: None,
                module_id: None,
                presentation_hint: None,
            });
        }

        Ok(stack_frames)
    }

    /// Get scopes for a frame
    fn get_scopes(&self, frame_id: i32) -> Result<Vec<Scope>, Box<dyn std::error::Error>> {
        let vm = self.vm.lock().unwrap();
        let frames = vm.get_call_frames();

        if frame_id < 0 || frame_id as usize >= frames.len() {
            return Ok(vec![]);
        }

        let frame = &frames[frame_id as usize];
        let local_count = frame.locals.len() as i32;

        Ok(vec![
            Scope {
                name: "Locals".to_string(),
                variables_reference: frame_id as i32 + 1, // Use frame_id + 1 as reference
                named_variables: Some(local_count),
                indexed_variables: None,
                expensive: false,
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            }
        ])
    }

    /// Get variables for a scope
    fn get_variables(&self, variables_reference: i32) -> Result<Vec<Variable>, Box<dyn std::error::Error>> {
        let vm = self.vm.lock().unwrap();
        let frames = vm.get_call_frames();

        // variables_reference = frame_id + 1, so frame_id = variables_reference - 1
        let frame_id = variables_reference - 1;
        if frame_id < 0 || frame_id as usize >= frames.len() {
            return Ok(vec![]);
        }

        let frame = &frames[frame_id as usize];
        let mut variables = Vec::new();

        // Get variable names from the code object
        for (i, value) in frame.locals.iter().enumerate() {
            if i < frame.code.names.len() {
                let name_idx = frame.code.names[i];
                if let Ok(name) = vm.get_string(name_idx) {
                    let var_value = self.format_value(value);
                    let var_type = self.get_value_type(value);

                    variables.push(Variable {
                        name: name.to_string(),
                        value: var_value,
                        type_: Some(var_type),
                        presentation_hint: None,
                        evaluate_name: None,
                        variables_reference: 0, // No nested variables for now
                        named_variables: None,
                        indexed_variables: None,
                        memory_reference: None,
                    });
                }
            }
        }

        Ok(variables)
    }

    /// Continue execution
    fn continue_execution(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut vm = self.vm.lock().unwrap();
        vm.set_debugger_state(DebuggerState::Running);
        Ok(())
    }

    /// Step over
    fn step_over(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut vm = self.vm.lock().unwrap();
        vm.set_debugger_state(DebuggerState::Stepping);
        Ok(())
    }

    /// Step in
    fn step_in(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut vm = self.vm.lock().unwrap();
        vm.set_debugger_state(DebuggerState::Stepping);
        Ok(())
    }

    /// Step out
    fn step_out(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut vm = self.vm.lock().unwrap();
        vm.set_debugger_state(DebuggerState::Stepping);
        Ok(())
    }

    /// Format a VM value for DAP display
    fn format_value(&self, value: &vela_vm::bytecode::Value) -> String {
        if value.is_null() {
            "null".to_string()
        } else if let Some(n) = value.as_int() {
            format!("{}", n)
        } else if let Some(f) = value.as_float() {
            format!("{}", f)
        } else if let Some(b) = value.as_bool() {
            format!("{}", b)
        } else if value.is_ptr() {
            "<object>".to_string()
        } else {
            "<unknown>".to_string()
        }
    }

    /// Get the type name of a VM value for DAP
    fn get_value_type(&self, value: &vela_vm::bytecode::Value) -> String {
        if value.is_null() {
            "null".to_string()
        } else if value.is_int() {
            "number".to_string()
        } else if value.is_float() {
            "float".to_string()
        } else if value.is_bool() {
            "boolean".to_string()
        } else if value.is_ptr() {
            "object".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

// DAP Message Types

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DapMessage {
    #[serde(rename = "request")]
    Request(Request),
    #[serde(rename = "response")]
    Response(Response),
    #[serde(rename = "event")]
    Event(Event),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub seq: u32,
    pub command: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub request_seq: u32,
    pub success: bool,
    pub command: String,
    #[serde(default)]
    pub body: Option<Value>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub seq: u32,
    pub event: String,
    #[serde(default)]
    pub body: Option<Value>,
}

// Request Arguments

#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchRequestArguments {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetBreakpointsArguments {
    pub source: Source,
    pub breakpoints: Vec<SourceBreakpoint>,
    #[serde(default)]
    pub lines: Option<Vec<i32>>,
    #[serde(default)]
    pub source_modified: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceBreakpoint {
    pub line: i32,
    #[serde(default)]
    pub column: Option<i32>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub hit_condition: Option<String>,
    #[serde(default)]
    pub log_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScopesArguments {
    pub frame_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariablesArguments {
    pub variables_reference: i32,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub start: Option<i32>,
    #[serde(default)]
    pub count: Option<i32>,
}

// Response Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub source_reference: Option<i32>,
    #[serde(default)]
    pub presentation_hint: Option<String>,
    #[serde(default)]
    pub origin: Option<String>,
    #[serde(default)]
    pub sources: Option<Vec<Source>>,
    #[serde(default)]
    pub adapter_data: Option<Value>,
    #[serde(default)]
    pub checksums: Option<Vec<Checksum>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checksum {
    pub algorithm: String,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BreakpointInfo {
    #[serde(default)]
    pub id: Option<i32>,
    pub verified: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub source: Option<Source>,
    #[serde(default)]
    pub line: Option<i32>,
    #[serde(default)]
    pub column: Option<i32>,
    #[serde(default)]
    pub end_line: Option<i32>,
    #[serde(default)]
    pub end_column: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub source: Option<Source>,
    pub line: i32,
    pub column: i32,
    #[serde(default)]
    pub end_line: Option<i32>,
    #[serde(default)]
    pub end_column: Option<i32>,
    #[serde(default)]
    pub can_restart: Option<bool>,
    #[serde(default)]
    pub instruction_pointer_reference: Option<String>,
    #[serde(default)]
    pub module_id: Option<Value>,
    #[serde(default)]
    pub presentation_hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scope {
    pub name: String,
    pub variables_reference: i32,
    #[serde(default)]
    pub named_variables: Option<i32>,
    #[serde(default)]
    pub indexed_variables: Option<i32>,
    pub expensive: bool,
    #[serde(default)]
    pub source: Option<Source>,
    #[serde(default)]
    pub line: Option<i32>,
    #[serde(default)]
    pub column: Option<i32>,
    #[serde(default)]
    pub end_line: Option<i32>,
    #[serde(default)]
    pub end_column: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub presentation_hint: Option<VariablePresentationHint>,
    #[serde(default)]
    pub evaluate_name: Option<String>,
    #[serde(default)]
    pub variables_reference: i32,
    #[serde(default)]
    pub named_variables: Option<i32>,
    #[serde(default)]
    pub indexed_variables: Option<i32>,
    #[serde(default)]
    pub memory_reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariablePresentationHint {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub attributes: Option<Vec<String>>,
    #[serde(default)]
    pub visibility: Option<String>,
}
