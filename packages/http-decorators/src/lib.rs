//! # Vela HTTP Decorators System
//!
//! Sistema de decoradores HTTP para Vela que implementa
//! los decoradores REST: `@controller`, `@get`, `@post`, etc.
//!
//! Este crate proporciona:
//! - `@controller`: Define controladores REST con ruta base
//! - `@get`, `@post`, `@put`, `@patch`, `@delete`: Endpoints HTTP
//! - `@middleware`: Middleware HTTP
//! - `@guard`: Guards de autorización

pub mod controller;
pub mod routes;
pub mod middleware;
pub mod guard;

pub use controller::*;
pub use routes::*;
pub use middleware::*;
pub use guard::*;