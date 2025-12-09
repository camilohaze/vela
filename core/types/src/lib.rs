//! # Vela Types
//!
//! Sistema de tipos estático para el lenguaje Vela.
//!
//! Este crate implementa el sistema de tipos híbrido que combina
//! tipado estático con inferencia de tipos (estilo Hindley-Milner).
//!
//! ## Arquitectura
//!
//! El sistema de tipos está dividido en módulos especializados:
//!
//! - `types`: Definiciones de tipos y operaciones básicas
//! - `context`: Gestión de contexto y scopes
//! - `error`: Sistema de errores de tipos
//! - `inference`: Algoritmo de inferencia de tipos
//! - `checker`: Verificación de tipos

pub mod checker;
pub mod context;
pub mod error;
pub mod inference;
pub mod types;

/// Resultado común para operaciones de tipos
pub type TypeResult<T> = Result<T, error::TypeError>;

/// Re-export de tipos principales para conveniencia
pub use types::{Type, TypeVar, TypeScheme};
pub use context::TypeContext;
pub use error::TypeError;
pub use checker::TypeChecker;