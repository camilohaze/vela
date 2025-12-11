//! # Vela Type System
//!
//! This module implements the complete type system for the Vela programming language.
//! It provides:
//! - Type representation (primitives, compounds, generics, functions)
//! - Type unification algorithm (Robinson's algorithm)
//! - Type inference foundations (Hindley-Milner preparation)
//!
//! ## Architecture
//!
//! The type system is organized into several submodules:
//! - `primitives`: Basic types (Number, String, Bool, Void)
//! - `compounds`: Complex types (Struct, Enum, Union)
//! - `generics`: Generic types and constraints
//! - `functions`: Function types and signatures
//! - `special`: Special types (Option<T>, Result<T,E>, etc.)
//! - `unification`: Type unification algorithm

pub mod primitives;
pub mod compounds;
pub mod generics;
pub mod functions;
pub mod special;
pub mod unification;

use std::collections::HashSet;
use std::fmt;

/// Core type representation for Vela
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Primitive types (Number, String, Bool, Void)
    Primitive(primitives::PrimitiveType),

    /// Type variables for generics
    Variable(generics::TypeVar),

    /// Generic type constructors
    Constructor(generics::TypeConstructor),

    /// Function types
    Function(functions::FunctionType),

    /// Method types (functions associated with types)
    Method(functions::MethodType),

    /// Closure types (anonymous functions with captures)
    Closure(functions::ClosureType),

    /// Struct types
    Struct(compounds::StructType),

    /// Enum types
    Enum(compounds::EnumType),

    /// Union types
    Union(special::UnionType),

    /// Intersection types
    Intersection(special::IntersectionType),

    /// Tuple types
    Tuple(special::TupleType),

    /// List types
    List(special::ListType),

    /// Dictionary types
    Dict(special::DictType),

    /// Set types
    Set(special::SetType),

    /// Option types (Some/None variants)
    Option(special::OptionVariant),

    /// Result types (Ok/Err variants)
    Result(special::ResultVariant),

    /// Promise types (async computations)
    Promise(special::PromiseType),

    /// Stream types (async sequences)
    Stream(special::StreamType),

    /// Iterator types
    Iterator(special::IteratorType),

    /// Range types
    Range(special::RangeType),
}

impl Type {
    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Primitive(_))
    }

    /// Check if this type contains generics
    pub fn is_generic(&self) -> bool {
        !self.free_vars().is_empty()
    }

    /// Get all free type variables in this type
    pub fn free_vars(&self) -> HashSet<generics::TypeVar> {
        match self {
            Type::Variable(var) => {
                let mut set = HashSet::new();
                set.insert(var.clone());
                set
            }
            Type::Constructor(generics::TypeConstructor { params, .. }) => {
                params.iter().flat_map(|t| t.free_vars()).collect()
            }
            Type::Function(functions::FunctionType { params, return_type, .. }) => {
                let mut vars = return_type.free_vars();
                for param in params {
                    vars.extend(param.ty.free_vars());
                }
                vars
            }
            Type::Method(functions::MethodType { self_type, params, return_type, .. }) => {
                let mut vars = self_type.free_vars();
                vars.extend(return_type.free_vars());
                for param in params {
                    vars.extend(param.ty.free_vars());
                }
                vars
            }
            Type::Closure(functions::ClosureType { captures, function_type }) => {
                let mut vars = function_type.type_vars();
                for (_, ty) in captures {
                    vars.extend(ty.free_vars());
                }
                vars
            }
            Type::Struct(compounds::StructType { fields, .. }) => {
                fields.values().flat_map(|ty| ty.free_vars()).collect()
            }
            Type::Enum(compounds::EnumType { variants, .. }) => {
                variants.iter().flat_map(|v| v.free_vars()).collect()
            }
            Type::Union(special::UnionType { variants }) => {
                variants.iter().flat_map(|t| t.free_vars()).collect()
            }
            Type::Intersection(special::IntersectionType { types }) => {
                types.iter().flat_map(|t| t.free_vars()).collect()
            }
            Type::Tuple(special::TupleType { elements }) => {
                elements.iter().flat_map(|t| t.free_vars()).collect()
            }
            Type::List(special::ListType { element_type }) => {
                element_type.free_vars()
            }
            Type::Dict(special::DictType { key_type, value_type }) => {
                let mut vars = key_type.free_vars();
                vars.extend(value_type.free_vars());
                vars
            }
            Type::Set(special::SetType { element_type }) => {
                element_type.free_vars()
            }
            Type::Option(special::OptionVariant::Some(ty)) => {
                ty.free_vars()
            }
            Type::Result(special::ResultVariant::Ok(ty) | special::ResultVariant::Err(ty)) => {
                ty.free_vars()
            }
            Type::Promise(special::PromiseType { result_type }) => {
                result_type.free_vars()
            }
            Type::Stream(special::StreamType { element_type }) => {
                element_type.free_vars()
            }
            Type::Iterator(special::IteratorType { element_type }) => {
                element_type.free_vars()
            }
            Type::Range(special::RangeType { element_type, .. }) => {
                element_type.free_vars()
            }
            Type::Option(special::OptionVariant::None) | Type::Primitive(_) => {
                HashSet::new()
            }
        }
    }

    /// Apply a substitution to this type
    pub fn apply_substitution(&self, subst: &unification::Substitution) -> Type {
        match self {
            Type::Variable(var) => subst.get(var).cloned().unwrap_or_else(|| self.clone()),
            Type::Constructor(generics::TypeConstructor { name, params }) => {
                let new_params = params.iter()
                    .map(|p| p.apply_substitution(subst))
                    .collect();
                Type::Constructor(generics::TypeConstructor {
                    name: name.clone(),
                    params: new_params,
                })
            }
            Type::Function(functions::FunctionType { params, return_type, is_async }) => {
                let new_params = params.iter()
                    .map(|p| functions::Parameter {
                        name: p.name.clone(),
                        ty: p.ty.apply_substitution(subst),
                        is_mutable: p.is_mutable,
                    })
                    .collect();
                let new_return = return_type.apply_substitution(subst);
                Type::Function(functions::FunctionType {
                    params: new_params,
                    return_type: Box::new(new_return),
                    is_async: *is_async,
                })
            }
            Type::Method(functions::MethodType { self_type, name, params, return_type, is_static, is_async }) => {
                let new_self = self_type.apply_substitution(subst);
                let new_params = params.iter()
                    .map(|p| functions::Parameter {
                        name: p.name.clone(),
                        ty: p.ty.apply_substitution(subst),
                        is_mutable: p.is_mutable,
                    })
                    .collect();
                let new_return = return_type.apply_substitution(subst);
                Type::Method(functions::MethodType {
                    self_type: Box::new(new_self),
                    name: name.clone(),
                    params: new_params,
                    return_type: Box::new(new_return),
                    is_static: *is_static,
                    is_async: *is_async,
                })
            }
            Type::Closure(functions::ClosureType { captures, function_type }) => {
                let new_captures = captures.iter()
                    .map(|(name, ty)| (name.clone(), ty.apply_substitution(subst)))
                    .collect();
                let new_fn_type = functions::FunctionType {
                    params: function_type.params.iter()
                        .map(|p| functions::Parameter {
                            name: p.name.clone(),
                            ty: p.ty.apply_substitution(subst),
                            is_mutable: p.is_mutable,
                        })
                        .collect(),
                    return_type: Box::new(function_type.return_type.apply_substitution(subst)),
                    is_async: function_type.is_async,
                };
                Type::Closure(functions::ClosureType {
                    captures: new_captures,
                    function_type: new_fn_type,
                })
            }
            Type::Struct(compounds::StructType { name, fields, methods }) => {
                let new_fields = fields.iter()
                    .map(|(name, ty)| (name.clone(), ty.apply_substitution(subst)))
                    .collect();
                let new_methods = methods.clone(); // TODO: apply substitution to methods
                Type::Struct(compounds::StructType {
                    name: name.clone(),
                    fields: new_fields,
                    methods: new_methods,
                })
            }
            Type::Enum(compounds::EnumType { name, variants }) => {
                let new_variants = variants.iter()
                    .map(|v| v.apply_substitution(subst))
                    .collect();
                Type::Enum(compounds::EnumType {
                    name: name.clone(),
                    variants: new_variants,
                })
            }
            Type::Union(special::UnionType { variants }) => {
                let new_variants = variants.iter()
                    .map(|t| t.apply_substitution(subst))
                    .collect();
                Type::Union(special::UnionType {
                    variants: new_variants,
                })
            }
            Type::Intersection(special::IntersectionType { types }) => {
                let new_types = types.iter()
                    .map(|t| t.apply_substitution(subst))
                    .collect();
                Type::Intersection(special::IntersectionType {
                    types: new_types,
                })
            }
            Type::Tuple(special::TupleType { elements }) => {
                let new_elements = elements.iter()
                    .map(|t| t.apply_substitution(subst))
                    .collect();
                Type::Tuple(special::TupleType {
                    elements: new_elements,
                })
            }
            Type::List(special::ListType { element_type }) => {
                Type::List(special::ListType {
                    element_type: Box::new(element_type.apply_substitution(subst)),
                })
            }
            Type::Dict(special::DictType { key_type, value_type }) => {
                Type::Dict(special::DictType {
                    key_type: Box::new(key_type.apply_substitution(subst)),
                    value_type: Box::new(value_type.apply_substitution(subst)),
                })
            }
            Type::Set(special::SetType { element_type }) => {
                Type::Set(special::SetType {
                    element_type: Box::new(element_type.apply_substitution(subst)),
                })
            }
            Type::Option(special::OptionVariant::Some(ty)) => {
                Type::Option(special::OptionVariant::Some(
                    Box::new(ty.apply_substitution(subst))
                ))
            }
            Type::Result(special::ResultVariant::Ok(ty)) => {
                Type::Result(special::ResultVariant::Ok(
                    Box::new(ty.apply_substitution(subst))
                ))
            }
            Type::Result(special::ResultVariant::Err(ty)) => {
                Type::Result(special::ResultVariant::Err(
                    Box::new(ty.apply_substitution(subst))
                ))
            }
            Type::Promise(special::PromiseType { result_type }) => {
                Type::Promise(special::PromiseType {
                    result_type: Box::new(result_type.apply_substitution(subst)),
                })
            }
            Type::Stream(special::StreamType { element_type }) => {
                Type::Stream(special::StreamType {
                    element_type: Box::new(element_type.apply_substitution(subst)),
                })
            }
            Type::Iterator(special::IteratorType { element_type }) => {
                Type::Iterator(special::IteratorType {
                    element_type: Box::new(element_type.apply_substitution(subst)),
                })
            }
            Type::Range(special::RangeType { element_type, inclusive }) => {
                Type::Range(special::RangeType {
                    element_type: Box::new(element_type.apply_substitution(subst)),
                    inclusive: *inclusive,
                })
            }
            // Types that don't contain other types
            Type::Primitive(_) | Type::Option(special::OptionVariant::None) => self.clone(),
        }
    }

    /// Get a human-readable string representation
    pub fn display(&self) -> String {
        match self {
            Type::Primitive(p) => format!("{:?}", p).to_lowercase(),
            Type::Variable(var) => var.display_name(),
            Type::Constructor(ctor) => ctor.to_string(),
            Type::Function(ft) => ft.to_string(),
            Type::Method(mt) => mt.to_string(),
            Type::Closure(ct) => ct.to_string(),
            Type::Struct(st) => st.to_string(),
            Type::Enum(et) => et.to_string(),
            Type::Union(ut) => ut.to_string(),
            Type::Intersection(it) => it.to_string(),
            Type::Tuple(tt) => tt.to_string(),
            Type::List(lt) => lt.to_string(),
            Type::Dict(dt) => dt.to_string(),
            Type::Set(st) => st.to_string(),
            Type::Option(ov) => ov.to_string(),
            Type::Result(rv) => rv.to_string(),
            Type::Promise(pt) => pt.to_string(),
            Type::Stream(st) => st.to_string(),
            Type::Iterator(it) => it.to_string(),
            Type::Range(rt) => rt.to_string(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Type environment for tracking variable types
#[derive(Debug, Clone, Default)]
pub struct TypeEnvironment {
    bindings: std::collections::HashMap<String, Type>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, name: String, ty: Type) {
        self.bindings.insert(name, ty);
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.bindings.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    pub fn extend(&self, bindings: std::collections::HashMap<String, Type>) -> TypeEnvironment {
        let mut new_env = self.clone();
        new_env.bindings.extend(bindings);
        new_env
    }
}

/// Errors that can occur during type operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    UnboundVariable(String),
    TypeMismatch { expected: Type, actual: Type },
    OccursCheck,
    InfiniteType,
    InvalidOperation { operation: String, operand: Type },
    MissingField { struct_name: String, field: String },
    DuplicateField(String),
    InvalidEnumVariant { enum_name: String, variant: String },
    UnificationError { expected: Type, actual: Type, context: String },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::UnboundVariable(var) => {
                write!(f, "Unbound variable: {}", var)
            }
            TypeError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            TypeError::OccursCheck => {
                write!(f, "Occurs check failed (infinite type)")
            }
            TypeError::InfiniteType => {
                write!(f, "Infinite type detected")
            }
            TypeError::InvalidOperation { operation, operand } => {
                write!(f, "Invalid operation '{}' on type {}", operation, operand)
            }
            TypeError::MissingField { struct_name, field } => {
                write!(f, "Missing field '{}' in struct '{}'", field, struct_name)
            }
            TypeError::DuplicateField(field) => {
                write!(f, "Duplicate field: {}", field)
            }
            TypeError::InvalidEnumVariant { enum_name, variant } => {
                write!(f, "Invalid enum variant '{}' for enum '{}'", variant, enum_name)
            }
            TypeError::UnificationError { expected, actual, context } => {
                write!(f, "Unification failed in {}: expected {}, got {}", context, expected, actual)
            }
        }
    }
}

impl std::error::Error for TypeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        let num_type = Type::Primitive(primitives::PrimitiveType::Number);
        assert!(num_type.is_primitive());
        assert!(!num_type.is_generic());
        assert_eq!(num_type.free_vars().len(), 0);
        assert_eq!(num_type.display(), "number");
    }

    #[test]
    fn test_type_variables() {
        let var = generics::TypeVar { id: 1, name: Some("T".to_string()) };
        let var_type = Type::Variable(var);
        assert!(!var_type.is_primitive());
        assert!(var_type.is_generic());
        assert_eq!(var_type.free_vars().len(), 1);
        assert_eq!(var_type.display(), "T");
    }

    #[test]
    fn test_generic_constructor() {
        let params = vec![Type::Variable(generics::TypeVar { id: 1, name: Some("T".to_string()) })];
        let constructor = generics::TypeConstructor {
            name: "List".to_string(),
            params,
        };
        let list_type = Type::Constructor(constructor);
        assert!(!list_type.is_primitive());
        assert!(list_type.is_generic());
        assert_eq!(list_type.free_vars().len(), 1);
        assert_eq!(list_type.display(), "List<T>");
    }

    #[test]
    fn test_option_type() {
        let inner = Type::Primitive(primitives::PrimitiveType::String);
        let option_type = Type::Constructor(generics::TypeConstructor::unary("Option".to_string(), inner));
        assert!(!option_type.is_primitive());
        assert!(!option_type.is_generic());
        assert_eq!(option_type.free_vars().len(), 0);
        assert_eq!(option_type.display(), "Option<string>");
    }

    #[test]
    fn test_union_type() {
        let types = vec![
            Type::Primitive(primitives::PrimitiveType::Number),
            Type::Primitive(primitives::PrimitiveType::String),
        ];
        let union_type = Type::Union(special::UnionType::new(types));
        assert!(!union_type.is_primitive());
        assert!(!union_type.is_generic());
        assert_eq!(union_type.free_vars().len(), 0);
        assert_eq!(union_type.display(), "number | string");
    }
}