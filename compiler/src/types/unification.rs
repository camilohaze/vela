//! # Unification Algorithm
//!
//! This module implements the unification algorithm for Vela's type system.
//! Unification finds a substitution that makes two types equal.
//!
//! The algorithm handles:
//! - Type variables
//! - Primitive types
//! - Function types
//! - Generic types
//! - Compound types (struct, enum, union)
//! - Special types (Option, Result, List, etc.)

use std::collections::{HashMap, HashSet};
use super::{Type, TypeError, generics::TypeVar};

/// A substitution mapping type variables to types
pub type Substitution = HashMap<TypeVar, Type>;

/// Result of unification
pub type UnificationResult = Result<Substitution, TypeError>;

/// Unification context for tracking state during unification
#[derive(Debug, Clone)]
pub struct UnificationContext {
    /// Current substitution
    pub substitution: Substitution,

    /// Type variable supply for fresh variables
    pub supply: super::generics::TypeVarSupply,
}

impl UnificationContext {
    /// Create a new unification context
    pub fn new() -> Self {
        Self {
            substitution: HashMap::new(),
            supply: super::generics::TypeVarSupply::new(),
        }
    }

    /// Apply the current substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        ty.apply_substitution(&self.substitution)
    }

    /// Compose this substitution with another
    pub fn compose(&mut self, other: Substitution) {
        // Apply current substitution to the values of the other substitution
        let composed: Substitution = other.into_iter()
            .map(|(var, ty)| (var, ty.apply_substitution(&self.substitution)))
            .collect();

        // Add the composed substitution to our current one
        for (var, ty) in composed {
            self.substitution.insert(var, ty);
        }
    }

    /// Get a fresh type variable
    pub fn fresh_var(&mut self) -> TypeVar {
        self.supply.fresh()
    }

    /// Get a fresh type variable with a name
    pub fn fresh_var_named(&mut self, name: String) -> TypeVar {
        self.supply.fresh_named(name)
    }
}

impl Default for UnificationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main unification function
pub fn unify(left: &Type, right: &Type) -> UnificationResult {
    let mut context = UnificationContext::new();
    unify_types(&mut context, left, right)?;
    Ok(context.substitution)
}

/// Unify two types within a context
pub fn unify_types(context: &mut UnificationContext, left: &Type, right: &Type) -> Result<(), TypeError> {
    let left = context.apply(left);
    let right = context.apply(right);

    match (&left, &right) {
        // Same types unify trivially
        (Type::Primitive(l), Type::Primitive(r)) if l == r => Ok(()),

        // Type variables
        (Type::Variable(var), ty) | (ty, Type::Variable(var)) => {
            unify_variable(context, var, ty)
        }

        // Function types
        (Type::Function(l), Type::Function(r)) => {
            unify_functions(context, l, r)
        }

        // Constructor types (generics)
        (Type::Constructor(l), Type::Constructor(r)) => {
            unify_constructors(context, l, r)
        }

        // Struct types
        (Type::Struct(l), Type::Struct(r)) => {
            unify_structs(context, l, r)
        }

        // Enum types
        (Type::Enum(l), Type::Enum(r)) => {
            unify_enums(context, l, r)
        }

        // Union types
        (Type::Union(l), Type::Union(r)) => {
            unify_unions(context, l, r)
        }

        // Intersection types
        (Type::Intersection(l), Type::Intersection(r)) => {
            unify_intersections(context, l, r)
        }

        // Tuple types
        (Type::Tuple(l), Type::Tuple(r)) => {
            unify_tuples(context, l, r)
        }

        // List types
        (Type::List(l), Type::List(r)) => {
            unify_types(context, &l.element_type, &r.element_type)
        }

        // Dict types
        (Type::Dict(l), Type::Dict(r)) => {
            unify_types(context, &l.key_type, &r.key_type)?;
            unify_types(context, &l.value_type, &r.value_type)
        }

        // Set types
        (Type::Set(l), Type::Set(r)) => {
            unify_types(context, &l.element_type, &r.element_type)
        }

        // Option types
        (Type::Option(l), Type::Option(r)) => {
            unify_option_variants(context, l, r)
        }

        // Result types
        (Type::Result(l), Type::Result(r)) => {
            unify_result_variants(context, l, r)
        }

        // Promise types
        (Type::Promise(l), Type::Promise(r)) => {
            unify_types(context, &l.result_type, &r.result_type)
        }

        // Stream types
        (Type::Stream(l), Type::Stream(r)) => {
            unify_types(context, &l.element_type, &r.element_type)
        }

        // Iterator types
        (Type::Iterator(l), Type::Iterator(r)) => {
            unify_types(context, &l.element_type, &r.element_type)
        }

        // Range types
        (Type::Range(l), Type::Range(r)) => {
            if l.inclusive != r.inclusive {
                return Err(TypeError::UnificationError {
                    left: left.clone(),
                    right: right.clone(),
                    reason: "Range inclusivity mismatch".to_string(),
                });
            }
            unify_types(context, &l.element_type, &r.element_type)
        }

        // Different types cannot unify
        _ => Err(TypeError::UnificationError {
            left: left.clone(),
            right: right.clone(),
            reason: "Type mismatch".to_string(),
        }),
    }
}

/// Unify a type variable with a type
fn unify_variable(context: &mut UnificationContext, var: &TypeVar, ty: &Type) -> Result<(), TypeError> {
    // If the variable is already bound
    if let Some(bound_type) = context.substitution.get(var) {
        return unify_types(context, bound_type, ty);
    }

    // Cannot bind variable to itself
    if let Type::Variable(other_var) = ty {
        if var == other_var {
            return Ok(());
        }
    }

    // Occurs check - variable cannot appear in the type it's being bound to
    if ty.free_vars().contains(var) {
        return Err(TypeError::UnificationError {
            left: Type::Variable(var.clone()),
            right: ty.clone(),
            reason: "Occurs check failed - circular type".to_string(),
        });
    }

    // Bind the variable
    context.substitution.insert(var.clone(), ty.clone());
    Ok(())
}

/// Unify function types
fn unify_functions(
    context: &mut UnificationContext,
    left: &super::functions::FunctionType,
    right: &super::functions::FunctionType
) -> Result<(), TypeError> {
    // Must have same arity
    if left.params.len() != right.params.len() {
        return Err(TypeError::UnificationError {
            left: Type::Function(left.clone()),
            right: Type::Function(right.clone()),
            reason: format!("Function arity mismatch: {} vs {}", left.params.len(), right.params.len()),
        });
    }

    // Must have same async-ness
    if left.is_async != right.is_async {
        return Err(TypeError::UnificationError {
            left: Type::Function(left.clone()),
            right: Type::Function(right.clone()),
            reason: "Function async mismatch".to_string(),
        });
    }

    // Unify parameters
    for (l_param, r_param) in left.params.iter().zip(right.params.iter()) {
        unify_types(context, &l_param.ty, &r_param.ty)?;
    }

    // Unify return types
    unify_types(context, &left.return_type, &right.return_type)
}

/// Unify generic type constructors
fn unify_constructors(
    context: &mut UnificationContext,
    left: &super::generics::TypeConstructor,
    right: &super::generics::TypeConstructor
) -> Result<(), TypeError> {
    // Must have same name
    if left.name != right.name {
        return Err(TypeError::UnificationError {
            left: Type::Constructor(left.clone()),
            right: Type::Constructor(right.clone()),
            reason: format!("Constructor name mismatch: {} vs {}", left.name, right.name),
        });
    }

    // Must have same arity
    if left.params.len() != right.params.len() {
        return Err(TypeError::UnificationError {
            left: Type::Constructor(left.clone()),
            right: Type::Constructor(right.clone()),
            reason: format!("Constructor arity mismatch: {} vs {}", left.params.len(), right.params.len()),
        });
    }

    // Unify type parameters
    for (l_param, r_param) in left.params.iter().zip(right.params.iter()) {
        unify_types(context, l_param, r_param)?;
    }

    Ok(())
}

/// Unify struct types
fn unify_structs(
    context: &mut UnificationContext,
    left: &super::compounds::StructType,
    right: &super::compounds::StructType
) -> Result<(), TypeError> {
    // Must have same name
    if left.name != right.name {
        return Err(TypeError::UnificationError {
            left: Type::Struct(left.clone()),
            right: Type::Struct(right.clone()),
            reason: format!("Struct name mismatch: {} vs {}", left.name, right.name),
        });
    }

    // Must have same fields (name and type)
    if left.fields.len() != right.fields.len() {
        return Err(TypeError::UnificationError {
            left: Type::Struct(left.clone()),
            right: Type::Struct(right.clone()),
            reason: format!("Struct field count mismatch: {} vs {}", left.fields.len(), right.fields.len()),
        });
    }

    for (l_field, r_field) in left.fields.iter().zip(right.fields.iter()) {
        if l_field.name != r_field.name {
            return Err(TypeError::UnificationError {
                left: Type::Struct(left.clone()),
                right: Type::Struct(right.clone()),
                reason: format!("Struct field name mismatch: {} vs {}", l_field.name, r_field.name),
            });
        }
        unify_types(context, &l_field.ty, &r_field.ty)?;
    }

    Ok(())
}

/// Unify enum types
fn unify_enums(
    context: &mut UnificationContext,
    left: &super::compounds::EnumType,
    right: &super::compounds::EnumType
) -> Result<(), TypeError> {
    // Must have same name
    if left.name != right.name {
        return Err(TypeError::UnificationError {
            left: Type::Enum(left.clone()),
            right: Type::Enum(right.clone()),
            reason: format!("Enum name mismatch: {} vs {}", left.name, right.name),
        });
    }

    // Must have same variants
    if left.variants.len() != right.variants.len() {
        return Err(TypeError::UnificationError {
            left: Type::Enum(left.clone()),
            right: Type::Enum(right.clone()),
            reason: format!("Enum variant count mismatch: {} vs {}", left.variants.len(), right.variants.len()),
        });
    }

    for (l_variant, r_variant) in left.variants.iter().zip(right.variants.iter()) {
        unify_enum_variants(context, l_variant, r_variant)?;
    }

    Ok(())
}

/// Unify enum variants
fn unify_enum_variants(
    context: &mut UnificationContext,
    left: &super::compounds::EnumVariant,
    right: &super::compounds::EnumVariant
) -> Result<(), TypeError> {
    match (left, right) {
        (super::compounds::EnumVariant::Unit(l_name), super::compounds::EnumVariant::Unit(r_name)) => {
            if l_name != r_name {
                return Err(TypeError::UnificationError {
                    left: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![left.clone()],
                    }),
                    right: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![right.clone()],
                    }),
                    reason: format!("Enum variant name mismatch: {} vs {}", l_name, r_name),
                });
            }
            Ok(())
        }

        (super::compounds::EnumVariant::Tuple(l_name, l_types), super::compounds::EnumVariant::Tuple(r_name, r_types)) => {
            if l_name != r_name || l_types.len() != r_types.len() {
                return Err(TypeError::UnificationError {
                    left: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![left.clone()],
                    }),
                    right: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![right.clone()],
                    }),
                    reason: "Enum tuple variant mismatch".to_string(),
                });
            }

            for (l_ty, r_ty) in l_types.iter().zip(r_types.iter()) {
                unify_types(context, l_ty, r_ty)?;
            }
            Ok(())
        }

        (super::compounds::EnumVariant::Struct(l_name, l_fields), super::compounds::EnumVariant::Struct(r_name, r_fields)) => {
            if l_name != r_name || l_fields.len() != r_fields.len() {
                return Err(TypeError::UnificationError {
                    left: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![left.clone()],
                    }),
                    right: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![right.clone()],
                    }),
                    reason: "Enum struct variant mismatch".to_string(),
                });
            }

            for (l_field, r_field) in l_fields.iter().zip(r_fields.iter()) {
                if l_field.name != r_field.name {
                    return Err(TypeError::UnificationError {
                        left: Type::Enum(super::compounds::EnumType {
                            name: "enum".to_string(),
                            variants: vec![left.clone()],
                        }),
                        right: Type::Enum(super::compounds::EnumType {
                            name: "enum".to_string(),
                            variants: vec![right.clone()],
                        }),
                        reason: format!("Enum struct field name mismatch: {} vs {}", l_field.name, r_field.name),
                    });
                }
                unify_types(context, &l_field.ty, &r_field.ty)?;
            }
            Ok(())
        }

        _ => Err(TypeError::UnificationError {
            left: Type::Enum(super::compounds::EnumType {
                name: "enum".to_string(),
                variants: vec![left.clone()],
            }),
            right: Type::Enum(super::compounds::EnumType {
                name: "enum".to_string(),
                variants: vec![right.clone()],
            }),
            reason: "Enum variant type mismatch".to_string(),
        }),
    }
}

/// Unify union types
fn unify_unions(
    context: &mut UnificationContext,
    left: &super::special::UnionType,
    right: &super::special::UnionType
) -> Result<(), TypeError> {
    // For now, require exact match of union variants
    // More sophisticated union unification could be added later
    if left.variants.len() != right.variants.len() {
        return Err(TypeError::UnificationError {
            left: Type::Union(left.clone()),
            right: Type::Union(right.clone()),
            reason: "Union variant count mismatch".to_string(),
        });
    }

    for (l_ty, r_ty) in left.variants.iter().zip(right.variants.iter()) {
        unify_types(context, l_ty, r_ty)?;
    }

    Ok(())
}

/// Unify intersection types
fn unify_intersections(
    context: &mut UnificationContext,
    left: &super::special::IntersectionType,
    right: &super::special::IntersectionType
) -> Result<(), TypeError> {
    // For now, require exact match of intersection types
    if left.types.len() != right.types.len() {
        return Err(TypeError::UnificationError {
            left: Type::Intersection(left.clone()),
            right: Type::Intersection(right.clone()),
            reason: "Intersection type count mismatch".to_string(),
        });
    }

    for (l_ty, r_ty) in left.types.iter().zip(right.types.iter()) {
        unify_types(context, l_ty, r_ty)?;
    }

    Ok(())
}

/// Unify tuple types
fn unify_tuples(
    context: &mut UnificationContext,
    left: &super::special::TupleType,
    right: &super::special::TupleType
) -> Result<(), TypeError> {
    if left.elements.len() != right.elements.len() {
        return Err(TypeError::UnificationError {
            left: Type::Tuple(left.clone()),
            right: Type::Tuple(right.clone()),
            reason: format!("Tuple arity mismatch: {} vs {}", left.elements.len(), right.elements.len()),
        });
    }

    for (l_elem, r_elem) in left.elements.iter().zip(right.elements.iter()) {
        unify_types(context, l_elem, r_elem)?;
    }

    Ok(())
}

/// Unify Option variants
fn unify_option_variants(
    context: &mut UnificationContext,
    left: &super::special::OptionVariant,
    right: &super::special::OptionVariant
) -> Result<(), TypeError> {
    match (left, right) {
        (super::special::OptionVariant::Some(l_ty), super::special::OptionVariant::Some(r_ty)) => {
            unify_types(context, l_ty, r_ty)
        }
        (super::special::OptionVariant::None, super::special::OptionVariant::None) => Ok(()),
        _ => Err(TypeError::UnificationError {
            left: Type::Option(left.clone()),
            right: Type::Option(right.clone()),
            reason: "Option variant mismatch".to_string(),
        }),
    }
}

/// Unify Result variants
fn unify_result_variants(
    context: &mut UnificationContext,
    left: &super::special::ResultVariant,
    right: &super::special::ResultVariant
) -> Result<(), TypeError> {
    match (left, right) {
        (super::special::ResultVariant::Ok(l_ty), super::special::ResultVariant::Ok(r_ty)) => {
            unify_types(context, l_ty, r_ty)
        }
        (super::special::ResultVariant::Err(l_ty), super::special::ResultVariant::Err(r_ty)) => {
            unify_types(context, l_ty, r_ty)
        }
        _ => Err(TypeError::UnificationError {
            left: Type::Result(left.clone()),
            right: Type::Result(right.clone()),
            reason: "Result variant mismatch".to_string(),
        }),
    }
}

/// Most general unifier (MGU) - finds the most general substitution
pub fn mgu(left: &Type, right: &Type) -> UnificationResult {
    unify(left, right)
}

/// Check if two types can be unified
pub fn can_unify(left: &Type, right: &Type) -> bool {
    unify(left, right).is_ok()
}

/// Get the principal type of a type (after unification)
pub fn principal_type(ty: &Type) -> Type {
    // For now, just return the type as-is
    // In a full implementation, this would normalize the type
    ty.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{primitives::PrimitiveType, generics::TypeVar};

    #[test]
    fn test_unify_primitives() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        // Same types unify
        assert!(unify(&int_type, &int_type).is_ok());

        // Different primitives don't unify
        assert!(unify(&int_type, &string_type).is_err());
    }

    #[test]
    fn test_unify_variables() {
        let mut supply = super::generics::TypeVarSupply::new();
        let var1 = supply.fresh();
        let var2 = supply.fresh();
        let int_type = Type::Primitive(PrimitiveType::Number);

        // Variable with itself
        assert!(unify(&Type::Variable(var1.clone()), &Type::Variable(var1.clone())).is_ok());

        // Variable with concrete type
        let subst = unify(&Type::Variable(var1.clone()), &int_type).unwrap();
        assert_eq!(subst.len(), 1);
        assert_eq!(subst.get(&var1), Some(&int_type));

        // Two different variables
        let subst = unify(&Type::Variable(var1), &Type::Variable(var2)).unwrap();
        assert_eq!(subst.len(), 1);
    }

    #[test]
    fn test_unify_functions() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        let fn1 = Type::Function(super::functions::FunctionType::new(
            vec![
                super::functions::Parameter::new("x".to_string(), int_type.clone()),
            ],
            string_type.clone()
        ));

        let fn2 = Type::Function(super::functions::FunctionType::new(
            vec![
                super::functions::Parameter::new("y".to_string(), int_type.clone()),
            ],
            string_type.clone()
        ));

        // Compatible function types unify
        assert!(unify(&fn1, &fn2).is_ok());

        // Different parameter types don't unify
        let fn3 = Type::Function(super::functions::FunctionType::new(
            vec![
                super::functions::Parameter::new("z".to_string(), string_type.clone()),
            ],
            string_type.clone()
        ));
        assert!(unify(&fn1, &fn3).is_err());

        // Different return types don't unify
        let fn4 = Type::Function(super::functions::FunctionType::new(
            vec![
                super::functions::Parameter::new("w".to_string(), int_type.clone()),
            ],
            int_type.clone()
        ));
        assert!(unify(&fn1, &fn4).is_err());
    }

    #[test]
    fn test_unify_constructors() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        let list1 = Type::Constructor(super::generics::TypeConstructor::unary(
            "List".to_string(),
            int_type.clone()
        ));

        let list2 = Type::Constructor(super::generics::TypeConstructor::unary(
            "List".to_string(),
            int_type.clone()
        ));

        let list3 = Type::Constructor(super::generics::TypeConstructor::unary(
            "List".to_string(),
            string_type.clone()
        ));

        // Same constructors unify
        assert!(unify(&list1, &list2).is_ok());

        // Different parameter types don't unify
        assert!(unify(&list1, &list3).is_err());

        // Different constructor names don't unify
        let set = Type::Constructor(super::generics::TypeConstructor::unary(
            "Set".to_string(),
            int_type.clone()
        ));
        assert!(unify(&list1, &set).is_err());
    }

    #[test]
    fn test_occurs_check() {
        let mut supply = super::generics::TypeVarSupply::new();
        let var = supply.fresh();

        // Create a recursive type: T = List<T>
        let recursive_type = Type::Constructor(super::generics::TypeConstructor::unary(
            "List".to_string(),
            Type::Variable(var.clone())
        ));

        // This should fail the occurs check
        assert!(unify(&Type::Variable(var), &recursive_type).is_err());
    }

    #[test]
    fn test_unify_tuples() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        let tuple1 = Type::Tuple(super::special::TupleType::new(vec![
            int_type.clone(),
            string_type.clone(),
        ]));

        let tuple2 = Type::Tuple(super::special::TupleType::new(vec![
            int_type.clone(),
            string_type.clone(),
        ]));

        let tuple3 = Type::Tuple(super::special::TupleType::new(vec![
            int_type.clone(),
        ]));

        // Same tuples unify
        assert!(unify(&tuple1, &tuple2).is_ok());

        // Different arity tuples don't unify
        assert!(unify(&tuple1, &tuple3).is_err());
    }

    #[test]
    fn test_can_unify() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        assert!(can_unify(&int_type, &int_type));
        assert!(!can_unify(&int_type, &string_type));
    }
}