"""
Type System Module - Vela Language

Implementación de: VELA-570 (TASK-013)
Sprint: Sprint 8
Fecha: 2025-12-01

Este módulo contiene el sistema de tipos completo de Vela:
- Representación interna de tipos
- Type inference (Hindley-Milner)
- Type checking
- Generics support
- Option<T> safety
"""

pub mod types;
pub mod inference;
pub mod checker;
pub mod env;

pub use types::*;
pub use inference::*;
pub use checker::*;
pub use env::*;
