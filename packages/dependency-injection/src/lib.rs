//! # Vela Dependency Injection System
//!
//! Sistema de inyección de dependencias para Vela que implementa
//! los decoradores `@injectable` e `@inject`.
//!
//! Este crate proporciona:
//! - `@injectable`: Marca clases como inyectables en el contenedor DI
//! - `@inject`: Marca parámetros/propiedades para inyección automática

pub mod injectable;
pub mod injector;
pub mod scope;

pub use injectable::*;
pub use injector::*;
pub use scope::*;