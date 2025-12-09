//! # Generic Types
//!
//! This module implements generic types in Vela:
//! - `TypeVar`: Type variables for generics
//! - `TypeConstructor`: Generic type constructors
//! - `TypeConstraint`: Constraints on type variables

use std::collections::HashSet;
use std::fmt;

/// Type variable for generic types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    /// Unique identifier for this type variable
    pub id: u32,

    /// Optional name for debugging/display purposes
    pub name: Option<String>,
}

impl TypeVar {
    /// Create a new type variable with an ID
    pub fn new(id: u32) -> Self {
        Self { id, name: None }
    }

    /// Create a new type variable with an ID and name
    pub fn named(id: u32, name: String) -> Self {
        Self { id, name: Some(name) }
    }

    /// Get the display name of this type variable
    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| format!("T{}", self.id))
    }
}

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Type constructor for generic types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeConstructor {
    /// Name of the type constructor (e.g., "List", "Option", "Result")
    pub name: String,

    /// Type parameters
    pub params: Vec<super::Type>,
}

impl TypeConstructor {
    /// Create a new type constructor
    pub fn new(name: String, params: Vec<super::Type>) -> Self {
        Self { name, params }
    }

    /// Create a type constructor with no parameters
    pub fn nullary(name: String) -> Self {
        Self::new(name, vec![])
    }

    /// Create a type constructor with one parameter
    pub fn unary(name: String, param: super::Type) -> Self {
        Self::new(name, vec![param])
    }

    /// Create a type constructor with two parameters
    pub fn binary(name: String, param1: super::Type, param2: super::Type) -> Self {
        Self::new(name, vec![param1, param2])
    }

    /// Get the arity (number of parameters) of this constructor
    pub fn arity(&self) -> usize {
        self.params.len()
    }

    /// Check if this is a nullary constructor (no parameters)
    pub fn is_nullary(&self) -> bool {
        self.params.is_empty()
    }

    /// Check if this is a unary constructor (one parameter)
    pub fn is_unary(&self) -> bool {
        self.params.len() == 1
    }

    /// Check if this is a binary constructor (two parameters)
    pub fn is_binary(&self) -> bool {
        self.params.len() == 2
    }
}

impl fmt::Display for TypeConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.params.is_empty() {
            write!(f, "{}", self.name)
        } else {
            let params_str = self.params.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "{}<{}>", self.name, params_str)
        }
    }
}

/// Trait bound for type constraints
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TraitBound {
    /// Must implement a specific trait
    Trait(String),

    /// Must be a subtype of another type
    Subtype(Box<super::Type>),

    /// Must be a supertype of another type
    Supertype(Box<super::Type>),
}

impl fmt::Display for TraitBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraitBound::Trait(name) => write!(f, "{}", name),
            TraitBound::Subtype(ty) => write!(f, "<: {}", ty),
            TraitBound::Supertype(ty) => write!(f, ">: {}", ty),
        }
    }
}

/// Type constraint on a type variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeConstraint {
    /// The type variable being constrained
    pub var: TypeVar,

    /// The constraint/bound
    pub bound: TraitBound,
}

impl TypeConstraint {
    /// Create a trait constraint
    pub fn trait_bound(var: TypeVar, trait_name: String) -> Self {
        Self {
            var,
            bound: TraitBound::Trait(trait_name),
        }
    }

    /// Create a subtype constraint
    pub fn subtype_bound(var: TypeVar, supertype: super::Type) -> Self {
        Self {
            var,
            bound: TraitBound::Subtype(Box::new(supertype)),
        }
    }

    /// Create a supertype constraint
    pub fn supertype_bound(var: TypeVar, subtype: super::Type) -> Self {
        Self {
            var,
            bound: TraitBound::Supertype(Box::new(subtype)),
        }
    }
}

impl fmt::Display for TypeConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.var, self.bound)
    }
}

/// Generic type scheme (forall quantification)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    /// Bound type variables
    pub vars: Vec<TypeVar>,

    /// The type with bound variables
    pub ty: Box<super::Type>,
}

impl TypeScheme {
    /// Create a new type scheme
    pub fn new(vars: Vec<TypeVar>, ty: super::Type) -> Self {
        Self {
            vars,
            ty: Box::new(ty),
        }
    }

    /// Create a monomorphic type scheme (no generics)
    pub fn mono(ty: super::Type) -> Self {
        Self::new(vec![], ty)
    }

    /// Instantiate this scheme with fresh type variables
    pub fn instantiate(&self, supply: &mut TypeVarSupply) -> super::Type {
        if self.vars.is_empty() {
            *self.ty.clone()
        } else {
            let fresh_vars: Vec<TypeVar> = (0..self.vars.len())
                .map(|_| supply.fresh())
                .collect();

            let mut subst = std::collections::HashMap::new();
            for (old_var, fresh_var) in self.vars.iter().zip(fresh_vars.iter()) {
                subst.insert(old_var.clone(), super::Type::Variable(fresh_var.clone()));
            }

            self.ty.apply_substitution(&subst)
        }
    }
}

impl fmt::Display for TypeScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "{}", self.ty)
        } else {
            let vars_str = self.vars.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "forall {}. {}", vars_str, self.ty)
        }
    }
}

/// Supply of fresh type variables
#[derive(Debug, Clone)]
pub struct TypeVarSupply {
    next_id: u32,
}

impl TypeVarSupply {
    /// Create a new type variable supply
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    /// Get the next fresh type variable
    pub fn fresh(&mut self) -> TypeVar {
        let id = self.next_id;
        self.next_id += 1;
        TypeVar::new(id)
    }

    /// Get the next fresh type variable with a name
    pub fn fresh_named(&mut self, name: String) -> TypeVar {
        let id = self.next_id;
        self.next_id += 1;
        TypeVar::named(id, name)
    }
}

impl Default for TypeVarSupply {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for common generic types
pub mod helpers {
    use super::*;

    /// Create a List<T> type constructor
    pub fn list_type(element_type: super::Type) -> TypeConstructor {
        TypeConstructor::unary("List".to_string(), element_type)
    }

    /// Create a Dict<K,V> type constructor
    pub fn dict_type(key_type: super::Type, value_type: super::Type) -> TypeConstructor {
        TypeConstructor::binary("Dict".to_string(), key_type, value_type)
    }

    /// Create an Option<T> type constructor
    pub fn option_type(inner_type: super::Type) -> TypeConstructor {
        TypeConstructor::unary("Option".to_string(), inner_type)
    }

    /// Create a Result<T,E> type constructor
    pub fn result_type(ok_type: super::Type, err_type: super::Type) -> TypeConstructor {
        TypeConstructor::binary("Result".to_string(), ok_type, err_type)
    }

    /// Create a function type scheme
    pub fn function_scheme(
        param_types: Vec<super::Type>,
        return_type: super::Type,
        supply: &mut TypeVarSupply
    ) -> TypeScheme {
        let mut vars = HashSet::new();

        // Collect all free variables from parameters and return type
        for param in &param_types {
            vars.extend(param.free_vars());
        }
        vars.extend(return_type.free_vars());

        let vars_vec: Vec<TypeVar> = vars.into_iter().collect();

        let fn_type = super::functions::FunctionType {
            params: param_types.into_iter()
                .enumerate()
                .map(|(i, ty)| super::functions::Parameter {
                    name: format!("arg{}", i),
                    ty,
                    is_mutable: false,
                })
                .collect(),
            return_type: Box::new(return_type),
            is_async: false,
        };

        TypeScheme::new(vars_vec, super::Type::Function(fn_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Type, primitives::PrimitiveType};

    #[test]
    fn test_type_var_creation() {
        let var1 = TypeVar::new(1);
        let var2 = TypeVar::named(2, "T".to_string());

        assert_eq!(var1.id, 1);
        assert_eq!(var1.name, None);
        assert_eq!(var2.id, 2);
        assert_eq!(var2.name, Some("T".to_string()));

        assert_eq!(var1.display_name(), "T1");
        assert_eq!(var2.display_name(), "T");
    }

    #[test]
    fn test_type_constructor() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        // Nullary constructor
        let unit = TypeConstructor::nullary("Unit".to_string());
        assert_eq!(unit.arity(), 0);
        assert!(unit.is_nullary());
        assert_eq!(format!("{}", unit), "Unit");

        // Unary constructor
        let list = TypeConstructor::unary("List".to_string(), int_type.clone());
        assert_eq!(list.arity(), 1);
        assert!(list.is_unary());
        assert_eq!(format!("{}", list), "List<number>");

        // Binary constructor
        let dict = TypeConstructor::binary("Dict".to_string(), string_type, int_type);
        assert_eq!(dict.arity(), 2);
        assert!(dict.is_binary());
        assert_eq!(format!("{}", dict), "Dict<string, number>");
    }

    #[test]
    fn test_type_constraints() {
        let var = TypeVar::named(1, "T".to_string());

        let trait_constraint = TypeConstraint::trait_bound(var.clone(), "Display".to_string());
        assert_eq!(format!("{}", trait_constraint), "T: Display");

        let subtype_constraint = TypeConstraint::subtype_bound(
            var.clone(),
            Type::Primitive(PrimitiveType::Number)
        );
        assert_eq!(format!("{}", subtype_constraint), "T: <: number");
    }

    #[test]
    fn test_type_scheme() {
        let mut supply = TypeVarSupply::new();
        let var_t = supply.fresh_named("T".to_string());
        let var_u = supply.fresh_named("U".to_string());

        let ty = Type::Function(super::functions::FunctionType {
            params: vec![
                super::functions::Parameter {
                    name: "x".to_string(),
                    ty: Type::Variable(var_t.clone()),
                    is_mutable: false,
                }
            ],
            return_type: Box::new(Type::Variable(var_u.clone())),
            is_async: false,
        });

        let scheme = TypeScheme::new(vec![var_t.clone(), var_u.clone()], ty);

        // Test instantiation creates fresh variables
        let instantiated = scheme.instantiate(&mut supply);
        match instantiated {
            Type::Function(ft) => {
                assert_eq!(ft.params.len(), 1);
                // The instantiated type should have fresh variables
                assert!(matches!(ft.params[0].ty, Type::Variable(_)));
                assert!(matches!(*ft.return_type, Type::Variable(_)));
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_type_var_supply() {
        let mut supply = TypeVarSupply::new();

        let var1 = supply.fresh();
        let var2 = supply.fresh();
        let var3 = supply.fresh_named("MyType".to_string());

        assert_eq!(var1.id, 0);
        assert_eq!(var2.id, 1);
        assert_eq!(var3.id, 2);
        assert_eq!(var3.name, Some("MyType".to_string()));
    }

    #[test]
    fn test_helper_functions() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        let list = helpers::list_type(int_type.clone());
        assert_eq!(list.name, "List");
        assert_eq!(list.params, vec![int_type.clone()]);

        let dict = helpers::dict_type(string_type.clone(), int_type.clone());
        assert_eq!(dict.name, "Dict");
        assert_eq!(dict.params, vec![string_type, int_type.clone()]);

        let option = helpers::option_type(int_type.clone());
        assert_eq!(option.name, "Option");
        assert_eq!(option.params, vec![int_type]);

        let result = helpers::result_type(int_type.clone(), string_type);
        assert_eq!(result.name, "Result");
        assert_eq!(result.params.len(), 2);
    }
}