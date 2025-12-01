// Vela Language Prototypes (Phase 0)
//
// Sprint 4 (VELA-565): Prototype & Validation
//
// Este módulo contiene proof of concept implementations para validar
// decisiones arquitectónicas críticas:
//
// - TASK-000V: Lexer prototype (state machine validation)
// - TASK-000W: Parser prototype (recursive descent + AST structure)
// - TASK-000X: Toolchain validation (CI integration)
// - TASK-000Y: Benchmarking framework (performance baseline)
//
// IMPORTANTE: Este NO es código de producción.
// Es código de validación técnica para Phase 0.

pub mod lexer;
pub mod parser;

// Re-export main types for convenience
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{parse_source, Expr, Parser, Program, Stmt};
