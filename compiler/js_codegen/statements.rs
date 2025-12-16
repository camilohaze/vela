//! Statement Code Generation for JavaScript
//!
//! Handles generation of JavaScript statements from Vela IR instructions.

use crate::ir::{IRInstruction, IRExpr, IRType};

/// Generate JavaScript code for statements/instructions
pub struct StatementGenerator {
    indent_level: usize,
}

impl StatementGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    /// Generate JavaScript code for a list of IR instructions
    pub fn generate_instructions(&mut self, instructions: &[IRInstruction]) -> String {
        let mut output = String::new();

        for instruction in instructions {
            output.push_str(&self.generate_instruction(instruction));
        }

        output
    }

    /// Generate JavaScript code for a single IR instruction
    pub fn generate_instruction(&mut self, instruction: &IRInstruction) -> String {
        let indent = self.get_indent();

        match instruction {
            IRInstruction::DeclareVar { name, ty } => {
                format!("{}let {}: {};\n", indent, name, Self::type_to_js(ty))
            }

            IRInstruction::AssignVar { name, value } => {
                format!("{}{} = {};\n", indent, name, Self::generate_expression(value))
            }

            IRInstruction::LoadConst(_) => {
                // In a stack-based VM, this would push to stack
                // For direct code generation, we handle this contextually
                String::new()
            }

            IRInstruction::LoadVar(_) => {
                // Similar to LoadConst
                String::new()
            }

            IRInstruction::StoreVar(_) => {
                // Similar to LoadConst
                String::new()
            }

            IRInstruction::BinaryOp(_) => {
                // Would be handled in expression context
                String::new()
            }

            IRInstruction::UnaryOp(_) => {
                // Would be handled in expression context
                String::new()
            }

            IRInstruction::Call { function, arg_count: _ } => {
                format!("{}{}();\n", indent, function)
            }

            IRInstruction::Return => {
                format!("{}return;\n", indent)
            }

            IRInstruction::Jump(label) => {
                // JavaScript doesn't have goto, so we'd restructure control flow
                // For now, this is a simplified implementation
                format!("{}// goto {}\n", indent, label)
            }

            IRInstruction::JumpIf(label) => {
                // Similar to Jump - would need restructuring
                format!("{}// conditional jump to {}\n", indent, label)
            }

            IRInstruction::Label(label) => {
                format!("{}{}:\n", indent, label)
            }

            IRInstruction::CreateArray { element_type, size } => {
                let type_str = Self::type_to_js(element_type);
                format!("{}new Array({}); // {}[]\n", indent, size, type_str)
            }

            IRInstruction::ArrayAccess => {
                // Would be handled in expression context
                String::new()
            }

            IRInstruction::ArrayStore => {
                // Would be handled in expression context
                String::new()
            }

            IRInstruction::CreateObject(type_name) => {
                format!("{}{} {{}}; // {}\n", indent, "{}".to_string(), type_name)
            }

            IRInstruction::PropertyAccess(_) => {
                // Would be handled in expression context
                String::new()
            }

            IRInstruction::PropertyStore(_) => {
                // Would be handled in expression context
                String::new()
            }
        }
    }

    /// Generate JavaScript code for an expression (delegate to ExpressionGenerator)
    fn generate_expression(expr: &IRExpr) -> String {
        crate::js_codegen::expressions::ExpressionGenerator::generate(expr)
    }

    /// Convert IR type to JavaScript type annotation
    fn type_to_js(ty: &IRType) -> String {
        match ty {
            IRType::Void => "void".to_string(),
            IRType::Bool => "boolean".to_string(),
            IRType::Int => "number".to_string(),
            IRType::Float => "number".to_string(),
            IRType::String => "string".to_string(),
            IRType::Array(element_type) => format!("{}[]", Self::type_to_js(element_type)),
            IRType::Object(name) => name.clone(),
        }
    }

    /// Get current indentation string
    fn get_indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    /// Increase indentation level
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IRInstruction, IRExpr, IRType, Value};

    #[test]
    fn test_generate_variable_declaration() {
        let mut generator = StatementGenerator::new();

        let instr = IRInstruction::DeclareVar {
            name: "x".to_string(),
            ty: IRType::Int,
        };

        let js = generator.generate_instruction(&instr);
        assert_eq!(js, "let x: number;\n");
    }

    #[test]
    fn test_generate_variable_assignment() {
        let mut generator = StatementGenerator::new();

        let instr = IRInstruction::AssignVar {
            name: "x".to_string(),
            value: IRExpr::Const(Value::Int(42)),
        };

        let js = generator.generate_instruction(&instr);
        assert_eq!(js, "x = 42;\n");
    }

    #[test]
    fn test_generate_function_call() {
        let mut generator = StatementGenerator::new();

        let instr = IRInstruction::Call {
            function: "println".to_string(),
            arg_count: 1,
        };

        let js = generator.generate_instruction(&instr);
        assert_eq!(js, "println();\n");
    }

    #[test]
    fn test_generate_return() {
        let mut generator = StatementGenerator::new();

        let instr = IRInstruction::Return;
        let js = generator.generate_instruction(&instr);
        assert_eq!(js, "return;\n");
    }

    #[test]
    fn test_generate_array_creation() {
        let mut generator = StatementGenerator::new();

        let instr = IRInstruction::CreateArray {
            element_type: IRType::String,
            size: 10,
        };

        let js = generator.generate_instruction(&instr);
        assert_eq!(js, "new Array(10); // string[]\n");
    }

    #[test]
    fn test_generate_object_creation() {
        let mut generator = StatementGenerator::new();

        let instr = IRInstruction::CreateObject("User".to_string());
        let js = generator.generate_instruction(&instr);
        assert_eq!(js, "{} {}; // User\n");
    }

    #[test]
    fn test_type_conversion() {
        assert_eq!(StatementGenerator::type_to_js(&IRType::Int), "number");
        assert_eq!(StatementGenerator::type_to_js(&IRType::String), "string");
        assert_eq!(StatementGenerator::type_to_js(&IRType::Bool), "boolean");
        assert_eq!(StatementGenerator::type_to_js(&IRType::Void), "void");
        assert_eq!(StatementGenerator::type_to_js(&IRType::Array(Box::new(IRType::Int))), "number[]");
        assert_eq!(StatementGenerator::type_to_js(&IRType::Object("User".to_string())), "User");
    }
}