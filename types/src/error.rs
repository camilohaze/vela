//! # Sistema de Errores de Tipos
//!
//! Este módulo define los errores que pueden ocurrir durante
//! la inferencia y verificación de tipos.

use crate::types::Type;
use thiserror::Error;

/// Errores que pueden ocurrir durante la verificación de tipos
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TypeError {
    /// Error de unificación: dos tipos no pueden unificarse
    #[error("Cannot unify types {lhs} and {rhs}")]
    UnificationError {
        /// Tipo izquierdo
        lhs: Type,
        /// Tipo derecho
        rhs: Type,
    },

    /// Variable de tipo infinita (ocurre-cuando check)
    #[error("Infinite type: {var} occurs in {ty}")]
    InfiniteType {
        /// Variable de tipo
        var: String,
        /// Tipo donde ocurre
        ty: Type,
    },

    /// Variable no encontrada en el contexto
    #[error("Variable '{name}' not found in scope")]
    VariableNotFound {
        /// Nombre de la variable
        name: String,
    },

    /// Error de aplicación de función
    #[error("Expected function type, got {actual}")]
    NotAFunction {
        /// Tipo actual encontrado
        actual: Type,
    },

    /// Número incorrecto de argumentos
    #[error("Expected {expected} arguments, got {actual}")]
    WrongNumberOfArguments {
        /// Número esperado
        expected: usize,
        /// Número actual
        actual: usize,
    },

    /// Error de acceso a campo
    #[error("Field '{field}' not found in type {ty}")]
    FieldNotFound {
        /// Nombre del campo
        field: String,
        /// Tipo del registro
        ty: Type,
    },

    /// Error de acceso a variante
    #[error("Variant '{variant}' not found in type {ty}")]
    VariantNotFound {
        /// Nombre de la variante
        variant: String,
        /// Tipo de la variante
        ty: Type,
    },

    /// Error de tipo esperado
    #[error("Expected type {expected}, got {actual}")]
    TypeMismatch {
        /// Tipo esperado
        expected: Type,
        /// Tipo actual
        actual: Type,
    },

    /// Error de tipo genérico
    #[error("Generic type '{name}' not found")]
    GenericTypeNotFound {
        /// Nombre del tipo genérico
        name: String,
    },

    /// Error de argumentos genéricos
    #[error("Wrong number of generic arguments for '{name}': expected {expected}, got {actual}")]
    WrongGenericArgs {
        /// Nombre del tipo genérico
        name: String,
        /// Número esperado de argumentos
        expected: usize,
        /// Número actual de argumentos
        actual: usize,
    },

    /// Error de recursión infinita en tipos
    #[error("Infinite recursion detected in type definition")]
    RecursiveType,

    /// Error de tipo never usado incorrectamente
    #[error("Never type used in invalid context")]
    InvalidNeverUsage,

    /// Error de conversión implícita no permitida
    #[error("Implicit conversion from {from} to {to} not allowed")]
    ImplicitConversionNotAllowed {
        /// Tipo origen
        from: Type,
        /// Tipo destino
        to: Type,
    },

    /// Error de tipo desconocido
    #[error("Unknown type: {name}")]
    UnknownType {
        /// Nombre del tipo desconocido
        name: String,
    },

    /// Error interno del sistema de tipos
    #[error("Internal type system error: {message}")]
    InternalError {
        /// Mensaje de error interno
        message: String,
    },
}

impl TypeError {
    /// Crear un error de unificación
    pub fn unification(lhs: Type, rhs: Type) -> Self {
        Self::UnificationError { lhs, rhs }
    }

    /// Crear un error de tipo infinito
    pub fn infinite_type(var: impl Into<String>, ty: Type) -> Self {
        Self::InfiniteType {
            var: var.into(),
            ty,
        }
    }

    /// Crear un error de variable no encontrada
    pub fn variable_not_found(name: impl Into<String>) -> Self {
        Self::VariableNotFound { name: name.into() }
    }

    /// Crear un error de función esperada
    pub fn not_a_function(actual: Type) -> Self {
        Self::NotAFunction { actual }
    }

    /// Crear un error de número de argumentos
    pub fn wrong_args(expected: usize, actual: usize) -> Self {
        Self::WrongNumberOfArguments { expected, actual }
    }

    /// Crear un error de campo no encontrado
    pub fn field_not_found(field: impl Into<String>, ty: Type) -> Self {
        Self::FieldNotFound {
            field: field.into(),
            ty,
        }
    }

    /// Crear un error de variante no encontrada
    pub fn variant_not_found(variant: impl Into<String>, ty: Type) -> Self {
        Self::VariantNotFound {
            variant: variant.into(),
            ty,
        }
    }

    /// Crear un error de tipo esperado
    pub fn type_mismatch(expected: Type, actual: Type) -> Self {
        Self::TypeMismatch { expected, actual }
    }

    /// Crear un error de tipo genérico no encontrado
    pub fn generic_not_found(name: impl Into<String>) -> Self {
        Self::GenericTypeNotFound { name: name.into() }
    }

    /// Crear un error de argumentos genéricos incorrectos
    pub fn wrong_generic_args(name: impl Into<String>, expected: usize, actual: usize) -> Self {
        Self::WrongGenericArgs {
            name: name.into(),
            expected,
            actual,
        }
    }

    /// Crear un error interno
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }
}

/// Resultado de operaciones de tipos con errores
pub type Result<T> = std::result::Result<T, TypeError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;

    #[test]
    fn test_unification_error_display() {
        let error = TypeError::unification(Type::Int, Type::Bool);
        let message = error.to_string();
        assert!(message.contains("Cannot unify types"));
        assert!(message.contains("Int"));
        assert!(message.contains("Bool"));
    }

    #[test]
    fn test_variable_not_found_error() {
        let error = TypeError::variable_not_found("x");
        assert_eq!(error.to_string(), "Variable 'x' not found in scope");
    }

    #[test]
    fn test_not_a_function_error() {
        let error = TypeError::not_a_function(Type::Int);
        let message = error.to_string();
        assert!(message.contains("Expected function type"));
        assert!(message.contains("Int"));
    }

    #[test]
    fn test_field_not_found_error() {
        let record_ty = Type::Record(std::collections::HashMap::new());
        let error = TypeError::field_not_found("name", record_ty);
        let message = error.to_string();
        assert!(message.contains("Field 'name' not found"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let error = TypeError::type_mismatch(Type::Int, Type::Bool);
        let message = error.to_string();
        assert!(message.contains("Expected type"));
        assert!(message.contains("got"));
    }

    #[test]
    fn test_internal_error() {
        let error = TypeError::internal("Something went wrong");
        assert_eq!(error.to_string(), "Internal type system error: Something went wrong");
    }
}