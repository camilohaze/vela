//! Sistema de Validación para Vela
//!
//! Este módulo implementa el sistema de validación declarativa y programática
//! definido en ADR-113F. Proporciona validadores built-in, schema builder API,
//! y integración con el ecosistema Vela (DTOs, controllers, UI).
//!
//! # Arquitectura
//!
//! El sistema se divide en tres capas:
//! 1. **Validadores**: Decoradores y funciones de validación básicas
//! 2. **Esquemas**: Construcción programática de reglas de validación
//! 3. **Integración**: Conexión con DTOs, controllers y UI
//!
//! # Ejemplo de Uso
//!
//! ```rust
//! use vela_validation::*;
//!
//! // Validación declarativa con decoradores
//! #[validate]
//! struct CreateUserDTO {
//!     #[required]
//!     #[length(min = 2, max = 50)]
//!     name: String,
//!
//!     #[required]
//!     #[email]
//!     email: String,
//!
//!     #[min(18)]
//!     #[max(120)]
//!     age: Option<i32>,
//! }
//!
//! // Validación programática con schema builder
//! let user_schema = Schema::new()
//!     .field("name", string().required().min(2).max(50))
//!     .field("email", string().required().email())
//!     .field("age", number().min(18).max(120));
//! ```

pub mod error;
pub mod validators;
pub mod schema;
pub mod decorator;
pub mod errors;
pub mod integration;

pub use error::{ValidationError, ValidationResult};
pub use validators::*;
pub use schema::*;
pub use decorator::*;
pub use errors::*;
pub use integration::*;