/*
VelaVM - Virtual Machine Implementation

This module implements the core virtual machine that executes Vela bytecode.

TODO: Complete refactoring according to ADR-801:
- Implement CallFrame struct
- Implement fetch-decode-execute loop
- Support all 40+ instructions from ADR-801
- Add exception handling
*/

// Temporarily commented out old implementation
// Will be replaced with ADR-801 design

/*
use crate::bytecode::{Bytecode, Instruction, Value};
use std::collections::HashMap;

/// Virtual Machine for executing Vela bytecode
pub struct VM {
    stack: Vec<Value>,
    environment: HashMap<String, Value>,
    ip: usize, // Instruction pointer
}

impl VM {
    /// Create a new VM instance
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            environment: HashMap::new(),
            ip: 0,
        }
    }

    /// Execute bytecode
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Value, String> {
        self.ip = 0;

        while self.ip < bytecode.instructions.len() {
            let instruction = &bytecode.instructions[self.ip];
            self.execute_instruction(instruction, bytecode)?;
            self.ip += 1;
        }

        // Return top of stack or None
        Ok(self.stack.pop().unwrap_or(Value::None))
    }

    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: &Instruction, bytecode: &Bytecode) -> Result<(), String> {
        match instruction {
            Instruction::Push(val) => self.stack.push(Value::Int(*val)),
            Instruction::PushFloat(val) => self.stack.push(Value::Float(*val)),
            Instruction::PushString(val) => self.stack.push(Value::String(val.clone())),
            Instruction::PushBool(val) => self.stack.push(Value::Bool(*val)),
            Instruction::Pop => { self.stack.pop(); },
            Instruction::Add => self.binary_op(|a, b| a + b)?,
            Instruction::Sub => self.binary_op(|a, b| a - b)?,
            Instruction::Mul => self.binary_op(|a, b| a * b)?,
            Instruction::Div => self.binary_op(|a, b| a / b)?,
            Instruction::Return => return Ok(()), // End execution
            // TODO: Implementar resto de instrucciones
            _ => return Err(format!("Instruction not implemented: {:?}", instruction)),
        }
        Ok(())
    }

    /// Execute binary operation
    fn binary_op<F>(&mut self, op: F) -> Result<(), String>
    where
        F: Fn(i64, i64) -> i64,
    {
        let b = self.stack.pop().ok_or("Stack underflow")?;
        let a = self.stack.pop().ok_or("Stack underflow")?;

        match (a, b) {
            (Value::Int(a_val), Value::Int(b_val)) => {
                self.stack.push(Value::Int(op(a_val, b_val)));
                Ok(())
            }
            _ => Err("Type error in binary operation".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = VM::new();
        assert!(vm.stack.is_empty());
        assert!(vm.environment.is_empty());
    }

    #[test]
    fn test_simple_execution() {
        let mut vm = VM::new();
        let mut bytecode = Bytecode::new();

        bytecode.push_instruction(Instruction::Push(5));
        bytecode.push_instruction(Instruction::Push(3));
        bytecode.push_instruction(Instruction::Add);
        bytecode.push_instruction(Instruction::Return);

        let result = vm.execute(&bytecode);
        assert!(result.is_ok());
        // TODO: Verificar resultado cuando esté implementado
    }
}
*/