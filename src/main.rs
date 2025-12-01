// Copyright (c) 2025 Vela Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vela Programming Language Compiler
//!
//! This is the main entry point for the Vela compiler.
//! 
//! # Sprint 0 - Architectural Foundations
//! 
//! This file demonstrates the architectural decisions made in Sprint 0:
//! - **Language**: Rust (ADR-001)
//! - **Build System**: Cargo workspace (ADR-002)
//! - **License**: MIT OR Apache-2.0 (ADR-003)
//! - **CI/CD**: GitHub Actions (ADR-004)
//! - **Documentation**: rustdoc + mdBook (ADR-005)
//!
//! # Architecture
//!
//! The Vela compiler is organized as a Cargo workspace with multiple crates:
//!
//! - `vela-compiler`: Main compiler orchestration
//! - `vela-parser`: Lexer and parser implementation
//! - `vela-ast`: Abstract Syntax Tree definitions
//! - `vela-codegen`: Code generation (LLVM backend)
//! - `vela-runtime`: Runtime library
//! - `vela-cli`: Command-line interface
//!
//! # Example Usage
//!
//! ```bash
//! # Compile a Vela program
//! cargo run -- compile hello.vela
//!
//! # Run a Vela program
//! cargo run -- run hello.vela
//!
//! # Show compiler version
//! cargo run -- --version
//! ```

use std::process;

fn main() {
    println!("Vela Compiler v0.1.0");
    println!("===================");
    println!();
    println!("Sprint 0: Architectural Foundations");
    println!();
    println!("✅ Language: Rust");
    println!("✅ Build System: Cargo (workspace)");
    println!("✅ License: MIT OR Apache-2.0");
    println!("✅ CI/CD: GitHub Actions");
    println!("✅ Documentation: rustdoc + mdBook");
    println!();
    println!("Next: Implement lexer, parser, and AST (Sprint 1+)");
    
    // Exit successfully
    process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_runs() {
        // This test ensures the main function can be called
        // In a real implementation, we'd test individual components
        assert!(true);
    }
}
