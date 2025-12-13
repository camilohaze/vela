//! JavaScript Code Generator for Vela
//!
//! This module implements the JavaScript code generation backend for Vela.
//! It transforms Vela's Intermediate Representation (IR) into valid JavaScript code.

pub mod codegen;
pub mod expressions;
pub mod statements;
pub mod types;
pub mod runtime;

use crate::ir::{Module, Function, Expression, Statement, Type};
use codegen::JSCodegen;

/// Main entry point for JavaScript code generation
pub struct JSGenerator {
    codegen: JSCodegen,
}

impl JSGenerator {
    pub fn new() -> Self {
        Self {
            codegen: JSCodegen::new(),
        }
    }

    /// Generate JavaScript code from a Vela module
    pub fn generate_module(&mut self, module: &Module) -> String {
        self.codegen.generate_module(module)
    }

    /// Generate JavaScript code from a single function
    pub fn generate_function(&mut self, function: &Function) -> String {
        self.codegen.generate_function(function)
    }

    /// Generate JavaScript code from an expression
    pub fn generate_expression(&mut self, expr: &Expression) -> String {
        self.codegen.generate_expression(expr)
    }

    /// Generate JavaScript code from a statement
    pub fn generate_statement(&mut self, stmt: &Statement) -> String {
        self.codegen.generate_statement(stmt)
    }

    /// Generate JavaScript type annotation
    pub fn generate_type(&mut self, ty: &Type) -> String {
        self.codegen.generate_type(ty)
    }
}

impl Default for JSGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Module, Function, Expression, Statement, Type};

    #[test]
    fn test_basic_generation() {
        let mut generator = JSGenerator::new();

        // Test simple expression
        let expr = Expression::Literal(crate::ir::Literal::Number(42.0));
        let js_code = generator.generate_expression(&expr);
        assert_eq!(js_code, "42");

        // Test string literal
        let expr = Expression::Literal(crate::ir::Literal::String("hello".to_string()));
        let js_code = generator.generate_expression(&expr);
        assert_eq!(js_code, "\"hello\"");
    }
}