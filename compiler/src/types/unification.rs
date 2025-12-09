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
pub fn unify(expected: &Type, actual: &Type) -> UnificationResult {
    let mut context = UnificationContext::new();
    unify_types(&mut context, expected, actual)?;
    Ok(context.substitution)
}

/// Unify two types within a context
pub fn unify_types(context: &mut UnificationContext, expected: &Type, actual: &Type) -> Result<(), TypeError> {
    let expected = context.apply(expected);
    let actual = context.apply(actual);

    match (&expected, &actual) {
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
                    expected: expected.clone(),
                    actual: actual.clone(),
                    context: "range inclusivity".to_string(),
                });
            }
            unify_types(context, &l.element_type, &r.element_type)
        }

        // Different types cannot unify
        _ => Err(TypeError::UnificationError {
            expected: expected.clone(),
            actual: actual.clone(),
            context: "type mismatch".to_string(),
        }),
    }
}

/// Unify a type variable with a type
fn unify_variable(context: &mut UnificationContext, var: &TypeVar, ty: &Type) -> Result<(), TypeError> {
    // If the variable is already bound
    if let Some(bound_type) = context.substitution.get(var) {
        let bound_type = bound_type.clone();
        return unify_types(context, &bound_type, ty);
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
            expected: Type::Variable(var.clone()),
            actual: ty.clone(),
            context: "Occurs check failed - circular type".to_string(),
        });
    }

    // Bind the variable
    context.substitution.insert(var.clone(), ty.clone());
    Ok(())
}

/// Unify function types
fn unify_functions(
    context: &mut UnificationContext,
    expected: &super::functions::FunctionType,
    actual: &super::functions::FunctionType
) -> Result<(), TypeError> {
    // Must have same arity
    if expected.params.len() != actual.params.len() {
        return Err(TypeError::UnificationError {
            expected: Type::Function(expected.clone()),
            actual: Type::Function(actual.clone()),
            context: format!("Function arity mismatch: {} vs {}", expected.params.len(), actual.params.len()),
        });
    }

    // Must have same async-ness
    if expected.is_async != actual.is_async {
        return Err(TypeError::UnificationError {
            expected: Type::Function(expected.clone()),
            actual: Type::Function(actual.clone()),
            context: "Function async mismatch".to_string(),
        });
    }

    // Unify parameters
    for (l_param, r_param) in expected.params.iter().zip(actual.params.iter()) {
        unify_types(context, &l_param.ty, &r_param.ty)?;
    }

    // Unify return types
    unify_types(context, &expected.return_type, &actual.return_type)
}

/// Unify generic type constructors
fn unify_constructors(
    context: &mut UnificationContext,
    expected: &super::generics::TypeConstructor,
    actual: &super::generics::TypeConstructor
) -> Result<(), TypeError> {
    // Must have same name
    if expected.name != actual.name {
        return Err(TypeError::UnificationError {
            expected: Type::Constructor(expected.clone()),
            actual: Type::Constructor(actual.clone()),
            context: format!("Constructor name mismatch: {} vs {}", expected.name, actual.name),
        });
    }

    // Must have same arity
    if expected.params.len() != actual.params.len() {
        return Err(TypeError::UnificationError {
            expected: Type::Constructor(expected.clone()),
            actual: Type::Constructor(actual.clone()),
            context: format!("Constructor arity mismatch: {} vs {}", expected.params.len(), actual.params.len()),
        });
    }

    // Unify type parameters
    for (l_param, r_param) in expected.params.iter().zip(actual.params.iter()) {
        unify_types(context, l_param, r_param)?;
    }

    Ok(())
}

/// Unify struct types
fn unify_structs(
    context: &mut UnificationContext,
    expected: &super::compounds::StructType,
    actual: &super::compounds::StructType
) -> Result<(), TypeError> {
    // Must have same name
    if expected.name != actual.name {
        return Err(TypeError::UnificationError {
            expected: Type::Struct(expected.clone()),
            actual: Type::Struct(actual.clone()),
            context: format!("Struct name mismatch: {} vs {}", expected.name, actual.name),
        });
    }

    // Must have same fields (name and type)
    if expected.fields.len() != actual.fields.len() {
        return Err(TypeError::UnificationError {
            expected: Type::Struct(expected.clone()),
            actual: Type::Struct(actual.clone()),
            context: "struct field count".to_string(),
        });
    }

    for (field_name, expected_ty) in &expected.fields {
        match actual.fields.get(field_name) {
            Some(actual_ty) => {
                unify_types(context, expected_ty, actual_ty)?;
            }
            None => {
                return Err(TypeError::UnificationError {
                    expected: Type::Struct(expected.clone()),
                    actual: Type::Struct(actual.clone()),
                    context: format!("missing field '{}'", field_name),
                });
            }
        }
    }

    Ok(())
}

/// Unify enum types
fn unify_enums(
    context: &mut UnificationContext,
    expected: &super::compounds::EnumType,
    actual: &super::compounds::EnumType
) -> Result<(), TypeError> {
    // Must have same name
    if expected.name != actual.name {
        return Err(TypeError::UnificationError {
            expected: Type::Enum(expected.clone()),
            actual: Type::Enum(actual.clone()),
            context: format!("Enum name mismatch: {} vs {}", expected.name, actual.name),
        });
    }

    // Must have same variants
    if expected.variants.len() != actual.variants.len() {
        return Err(TypeError::UnificationError {
            expected: Type::Enum(expected.clone()),
            actual: Type::Enum(actual.clone()),
            context: format!("Enum variant count mismatch: {} vs {}", expected.variants.len(), actual.variants.len()),
        });
    }

    for (l_variant, r_variant) in expected.variants.iter().zip(actual.variants.iter()) {
        unify_enum_variants(context, l_variant, r_variant)?;
    }

    Ok(())
}

/// Unify enum variants
fn unify_enum_variants(
    context: &mut UnificationContext,
    expected: &super::compounds::EnumVariant,
    actual: &super::compounds::EnumVariant
) -> Result<(), TypeError> {
    match (expected, actual) {
        (super::compounds::EnumVariant::Unit(l_name), super::compounds::EnumVariant::Unit(r_name)) => {
            if l_name != r_name {
                return Err(TypeError::UnificationError {
                    expected: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![expected.clone()],
                    }),
                    actual: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![actual.clone()],
                    }),
                    context: format!("Enum variant name mismatch: {} vs {}", l_name, r_name),
                });
            }
            Ok(())
        }

        (super::compounds::EnumVariant::Tuple(l_name, l_types), super::compounds::EnumVariant::Tuple(r_name, r_types)) => {
            if l_name != r_name || l_types.len() != r_types.len() {
                return Err(TypeError::UnificationError {
                    expected: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![expected.clone()],
                    }),
                    actual: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![actual.clone()],
                    }),
                    context: "Enum tuple variant mismatch".to_string(),
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
                    expected: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![expected.clone()],
                    }),
                    actual: Type::Enum(super::compounds::EnumType {
                        name: "enum".to_string(),
                        variants: vec![actual.clone()],
                    }),
                    context: "Enum struct variant mismatch".to_string(),
                });
            }

            for (l_field, r_field) in l_fields.iter().zip(r_fields.iter()) {
                if l_field.0 != r_field.0 {
                    return Err(TypeError::UnificationError {
                        expected: Type::Enum(super::compounds::EnumType {
                            name: "enum".to_string(),
                            variants: vec![expected.clone()],
                        }),
                        actual: Type::Enum(super::compounds::EnumType {
                            name: "enum".to_string(),
                            variants: vec![actual.clone()],
                        }),
                        context: format!("Enum struct field name mismatch: {} vs {}", l_field.0, r_field.0),
                    });
                }
                unify_types(context, &l_field.1, &r_field.1)?;
            }
            Ok(())
        }

        _ => Err(TypeError::UnificationError {
            expected: Type::Enum(super::compounds::EnumType {
                name: "enum".to_string(),
                variants: vec![expected.clone()],
            }),
            actual: Type::Enum(super::compounds::EnumType {
                name: "enum".to_string(),
                variants: vec![actual.clone()],
            }),
            context: "Enum variant type mismatch".to_string(),
        }),
    }
}

/// Unify union types
fn unify_unions(
    context: &mut UnificationContext,
    expected: &super::special::UnionType,
    actual: &super::special::UnionType
) -> Result<(), TypeError> {
    // For now, require exact match of union variants
    // More sophisticated union unification could be added later
    if expected.variants.len() != actual.variants.len() {
        return Err(TypeError::UnificationError {
            expected: Type::Union(expected.clone()),
            actual: Type::Union(actual.clone()),
            context: "Union variant count mismatch".to_string(),
        });
    }

    for (l_ty, r_ty) in expected.variants.iter().zip(actual.variants.iter()) {
        unify_types(context, l_ty, r_ty)?;
    }

    Ok(())
}

/// Unify intersection types
fn unify_intersections(
    context: &mut UnificationContext,
    expected: &super::special::IntersectionType,
    actual: &super::special::IntersectionType
) -> Result<(), TypeError> {
    // For now, require exact match of intersection types
    if expected.types.len() != actual.types.len() {
        return Err(TypeError::UnificationError {
            expected: Type::Intersection(expected.clone()),
            actual: Type::Intersection(actual.clone()),
            context: "Intersection type count mismatch".to_string(),
        });
    }

    for (l_ty, r_ty) in expected.types.iter().zip(actual.types.iter()) {
        unify_types(context, l_ty, r_ty)?;
    }

    Ok(())
}

/// Unify tuple types
fn unify_tuples(
    context: &mut UnificationContext,
    expected: &super::special::TupleType,
    actual: &super::special::TupleType
) -> Result<(), TypeError> {
    if expected.elements.len() != actual.elements.len() {
        return Err(TypeError::UnificationError {
            expected: Type::Tuple(expected.clone()),
            actual: Type::Tuple(actual.clone()),
            context: format!("Tuple arity mismatch: {} vs {}", expected.elements.len(), actual.elements.len()),
        });
    }

    for (l_elem, r_elem) in expected.elements.iter().zip(actual.elements.iter()) {
        unify_types(context, l_elem, r_elem)?;
    }

    Ok(())
}

/// Unify Option variants
fn unify_option_variants(
    context: &mut UnificationContext,
    expected: &super::special::OptionVariant,
    actual: &super::special::OptionVariant
) -> Result<(), TypeError> {
    match (expected, actual) {
        (super::special::OptionVariant::Some(l_ty), super::special::OptionVariant::Some(r_ty)) => {
            unify_types(context, &l_ty, &r_ty)
        }
        (super::special::OptionVariant::None, super::special::OptionVariant::None) => Ok(()),
        _ => Err(TypeError::UnificationError {
            expected: Type::Option(expected.clone()),
            actual: Type::Option(actual.clone()),
            context: "Option variant mismatch".to_string(),
        }),
    }
}

/// Unify Result variants
fn unify_result_variants(
    context: &mut UnificationContext,
    expected: &super::special::ResultVariant,
    actual: &super::special::ResultVariant
) -> Result<(), TypeError> {
    match (expected, actual) {
        (super::special::ResultVariant::Ok(l_ty), super::special::ResultVariant::Ok(r_ty)) => {
            unify_types(context, &l_ty, &r_ty)
        }
        (super::special::ResultVariant::Err(l_ty), super::special::ResultVariant::Err(r_ty)) => {
            unify_types(context, &l_ty, &r_ty)
        }
        _ => Err(TypeError::UnificationError {
            expected: Type::Result(expected.clone()),
            actual: Type::Result(actual.clone()),
            context: "Result variant mismatch".to_string(),
        }),
    }
}

/// Most general unifier (MGU) - finds the most general substitution
pub fn mgu(expected: &Type, actual: &Type) -> UnificationResult {
    unify(expected, actual)
}

/// Check if two types can be unified
pub fn can_unify(expected: &Type, actual: &Type) -> bool {
    unify(expected, actual).is_ok()
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
