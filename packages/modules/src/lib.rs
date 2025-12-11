//! # Vela Modules System
//!
//! Sistema de módulos para Vela que implementa el patrón de módulos
//! funcionales con decoradores `@package`, `@library` y `@module`.
//!
//! Este crate proporciona:
//! - `@package`: Define paquetes publicables
//! - `@library`: Define bibliotecas internas reutilizables
//! - `@module`: Define módulos funcionales (NO instanciables)

pub mod package;
pub mod library;
pub mod module;
pub mod registry;

pub use package::*;
pub use library::*;
pub use module::*;
pub use registry::*;