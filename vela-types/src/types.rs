//! # Definiciones de Tipos
//!
//! Este módulo contiene las definiciones principales del sistema de tipos
//! incluyendo el enum `Type` y estructuras relacionadas.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identificador único para variables de tipo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeVar(pub usize);

impl TypeVar {
    /// Crear una nueva variable de tipo con ID único
    pub fn fresh() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Esquema de tipo (tipo cuantificado)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeScheme {
    /// Variables de tipo cuantificadas
    pub vars: Vec<TypeVar>,
    /// Tipo base
    pub ty: Type,
}

impl TypeScheme {
    /// Crear un esquema sin cuantificación
    pub fn mono(ty: Type) -> Self {
        Self { vars: vec![], ty }
    }

    /// Crear un esquema cuantificado
    pub fn poly(vars: Vec<TypeVar>, ty: Type) -> Self {
        Self { vars, ty }
    }
}

/// Representación de tipos en el sistema de tipos de Vela
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    /// Tipo unit (sin valor)
    Unit,
    /// Tipo booleano
    Bool,
    /// Tipo entero (64-bit)
    Int,
    /// Tipo flotante (64-bit)
    Float,
    /// Tipo cadena de texto
    String,
    /// Tipo carácter
    Char,
    /// Tipo arreglo homogéneo
    Array(Box<Type>),
    /// Tipo función
    Function {
        /// Tipos de parámetros
        params: Vec<Type>,
        /// Tipo de retorno
        ret: Box<Type>,
    },
    /// Tipo tupla
    Tuple(Vec<Type>),
    /// Tipo registro/struct
    Record(HashMap<String, Type>),
    /// Tipo variante/enum
    Variant(HashMap<String, Type>),
    /// Tipo opción (Option<T>)
    Option(Box<Type>),
    /// Tipo resultado (Result<T, E>)
    Result {
        /// Tipo de éxito
        ok: Box<Type>,
        /// Tipo de error
        err: Box<Type>,
    },
    /// Variable de tipo (para inferencia)
    Var(TypeVar),
    /// Tipo genérico parametrizado
    Generic {
        /// Nombre del tipo genérico
        name: String,
        /// Parámetros de tipo
        args: Vec<Type>,
    },
    /// Tipo never (nunca retorna)
    Never,
}

impl Type {
    /// Verificar si el tipo es monomórfico (sin variables de tipo)
    pub fn is_mono(&self) -> bool {
        match self {
            Type::Var(_) => false,
            Type::Array(ty) => ty.is_mono(),
            Type::Function { params, ret } => {
                params.iter().all(|p| p.is_mono()) && ret.is_mono()
            }
            Type::Tuple(types) => types.iter().all(|t| t.is_mono()),
            Type::Record(fields) => fields.values().all(|t| t.is_mono()),
            Type::Variant(variants) => variants.values().all(|t| t.is_mono()),
            Type::Option(ty) => ty.is_mono(),
            Type::Result { ok, err } => ok.is_mono() && err.is_mono(),
            Type::Generic { args, .. } => args.iter().all(|a| a.is_mono()),
            _ => true,
        }
    }

    /// Obtener todas las variables de tipo libres en el tipo
    pub fn free_vars(&self) -> Vec<TypeVar> {
        let mut vars = Vec::new();
        self.collect_free_vars(&mut vars);
        vars
    }

    fn collect_free_vars(&self, vars: &mut Vec<TypeVar>) {
        match self {
            Type::Var(tv) => {
                if !vars.contains(tv) {
                    vars.push(*tv);
                }
            }
            Type::Array(ty) => ty.collect_free_vars(vars),
            Type::Function { params, ret } => {
                for param in params {
                    param.collect_free_vars(vars);
                }
                ret.collect_free_vars(vars);
            }
            Type::Tuple(types) => {
                for ty in types {
                    ty.collect_free_vars(vars);
                }
            }
            Type::Record(fields) => {
                for ty in fields.values() {
                    ty.collect_free_vars(vars);
                }
            }
            Type::Variant(variants) => {
                for ty in variants.values() {
                    ty.collect_free_vars(vars);
                }
            }
            Type::Option(ty) => ty.collect_free_vars(vars),
            Type::Result { ok, err } => {
                ok.collect_free_vars(vars);
                err.collect_free_vars(vars);
            }
            Type::Generic { args, .. } => {
                for arg in args {
                    arg.collect_free_vars(vars);
                }
            }
            _ => {} // Otros tipos no tienen variables libres
        }
    }

    /// Aplicar sustitución de tipos
    pub fn apply_subst(&mut self, subst: &HashMap<TypeVar, Type>) {
        match self {
            Type::Var(tv) => {
                if let Some(ty) = subst.get(tv) {
                    *self = ty.clone();
                }
            }
            Type::Array(ty) => ty.apply_subst(subst),
            Type::Function { params, ret } => {
                for param in params {
                    param.apply_subst(subst);
                }
                ret.apply_subst(subst);
            }
            Type::Tuple(types) => {
                for ty in types {
                    ty.apply_subst(subst);
                }
            }
            Type::Record(fields) => {
                for ty in fields.values_mut() {
                    ty.apply_subst(subst);
                }
            }
            Type::Variant(variants) => {
                for ty in variants.values_mut() {
                    ty.apply_subst(subst);
                }
            }
            Type::Option(ty) => ty.apply_subst(subst),
            Type::Result { ok, err } => {
                ok.apply_subst(subst);
                err.apply_subst(subst);
            }
            Type::Generic { args, .. } => {
                for arg in args {
                    arg.apply_subst(subst);
                }
            }
            _ => {} // Otros tipos no necesitan sustitución
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Bool => write!(f, "Bool"),
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
            Type::Char => write!(f, "Char"),
            Type::Array(ty) => write!(f, "[{}]", ty),
            Type::Function { params, ret } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Type::Record(fields) => {
                write!(f, "{{")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, "}}")
            }
            Type::Variant(variants) => {
                write!(f, "enum {{")?;
                for (i, (name, ty)) in variants.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, "}}")
            }
            Type::Option(ty) => write!(f, "Option<{}>", ty),
            Type::Result { ok, err } => write!(f, "Result<{}, {}>", ok, err),
            Type::Var(tv) => write!(f, "'{}", tv.0),
            Type::Generic { name, args } => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Never => write!(f, "Never"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(Type::Unit.to_string(), "()");
        assert_eq!(Type::Bool.to_string(), "Bool");
        assert_eq!(Type::Int.to_string(), "Int");
        assert_eq!(Type::Float.to_string(), "Float");
        assert_eq!(Type::String.to_string(), "String");
        assert_eq!(Type::Char.to_string(), "Char");
    }

    #[test]
    fn test_array_type_display() {
        let array_ty = Type::Array(Box::new(Type::Int));
        assert_eq!(array_ty.to_string(), "[Int]");
    }

    #[test]
    fn test_function_type_display() {
        let func_ty = Type::Function {
            params: vec![Type::Int, Type::String],
            ret: Box::new(Type::Bool),
        };
        assert_eq!(func_ty.to_string(), "(Int, String) -> Bool");
    }

    #[test]
    fn test_tuple_type_display() {
        let tuple_ty = Type::Tuple(vec![Type::Int, Type::String, Type::Bool]);
        assert_eq!(tuple_ty.to_string(), "(Int, String, Bool)");
    }

    #[test]
    fn test_free_vars() {
        let tv1 = TypeVar(1);
        let tv2 = TypeVar(2);

        let ty = Type::Function {
            params: vec![Type::Var(tv1), Type::Int],
            ret: Box::new(Type::Var(tv2)),
        };

        let free_vars = ty.free_vars();
        assert_eq!(free_vars.len(), 2);
        assert!(free_vars.contains(&tv1));
        assert!(free_vars.contains(&tv2));
    }

    #[test]
    fn test_is_mono() {
        assert!(Type::Int.is_mono());
        assert!(Type::Bool.is_mono());
        assert!(!Type::Var(TypeVar(1)).is_mono());

        let func_ty = Type::Function {
            params: vec![Type::Int, Type::Var(TypeVar(1))],
            ret: Box::new(Type::Bool),
        };
        assert!(!func_ty.is_mono());
    }
}