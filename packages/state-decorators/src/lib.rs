//! # Vela State Management Decorators System
//!
//! Sistema de decoradores para state management en Vela que implementa
//! los decoradores `@connect`, `@select` y `@persistent`.
//!
//! Este crate proporciona:
//! - `@connect`: Conecta componentes/widgets a stores globales
//! - `@select`: Selección optimizada de estado del store
//! - `@persistent`: Persistencia automática de estado

pub mod connect;
pub mod select;
pub mod persistent;

pub use connect::*;
pub use select::*;
pub use persistent::*;