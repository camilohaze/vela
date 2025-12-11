//! # Function Types
//!
//! This module implements function types in Vela:
//! - `FunctionType`: Function signatures with parameters and return types
//! - `Parameter`: Function parameters with types and mutability
//! - `MethodType`: Methods on types
//! - `ClosureType`: Anonymous function types

use std::fmt;
use crate::types::Type;
use crate::types::primitives::PrimitiveType;
use crate::types::generics::helpers;

/// Function parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub ty: Type,

    /// Whether this parameter is mutable
    pub is_mutable: bool,
}

impl Parameter {
    /// Create a new parameter
    pub fn new(name: String, ty: Type) -> Self {
        Self {
            name,
            ty,
            is_mutable: false,
        }
    }

    /// Create a mutable parameter
    pub fn mutable(name: String, ty: Type) -> Self {
        Self {
            name,
            ty,
            is_mutable: true,
        }
    }

    /// Check if this parameter is a type variable
    pub fn is_type_var(&self) -> bool {
        matches!(self.ty, Type::Variable(_))
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_mutable {
            write!(f, "mut {}: {}", self.name, self.ty)
        } else {
            write!(f, "{}: {}", self.name, self.ty)
        }
    }
}

/// Function type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    /// Function parameters
    pub params: Vec<Parameter>,

    /// Return type
    pub return_type: Box<Type>,

    /// Whether this is an async function
    pub is_async: bool,
}

impl FunctionType {
    /// Create a new function type
    pub fn new(params: Vec<Parameter>, return_type: Type) -> Self {
        Self {
            params,
            return_type: Box::new(return_type),
            is_async: false,
        }
    }

    /// Create an async function type
    pub fn async_fn(params: Vec<Parameter>, return_type: Type) -> Self {
        Self {
            params,
            return_type: Box::new(return_type),
            is_async: true,
        }
    }

    /// Create a function type from parameter types only
    pub fn from_types(param_types: Vec<Type>, return_type: Type) -> Self {
        let params = param_types.into_iter()
            .enumerate()
            .map(|(i, ty)| Parameter::new(format!("arg{}", i), ty))
            .collect();
        Self::new(params, return_type)
    }

    /// Get the arity (number of parameters) of this function
    pub fn arity(&self) -> usize {
        self.params.len()
    }

    /// Check if this function takes no parameters
    pub fn is_nullary(&self) -> bool {
        self.params.is_empty()
    }

    /// Check if this function takes one parameter
    pub fn is_unary(&self) -> bool {
        self.params.len() == 1
    }

    /// Check if this function takes two parameters
    pub fn is_binary(&self) -> bool {
        self.params.len() == 2
    }

    /// Get the parameter types
    pub fn param_types(&self) -> Vec<&Type> {
        self.params.iter().map(|p| &p.ty).collect()
    }

    /// Check if this function has mutable parameters
    pub fn has_mutable_params(&self) -> bool {
        self.params.iter().any(|p| p.is_mutable)
    }

    /// Get all type variables in this function type
    pub fn type_vars(&self) -> std::collections::HashSet<super::generics::TypeVar> {
        let mut vars = std::collections::HashSet::new();

        for param in &self.params {
            vars.extend(param.ty.free_vars());
        }
        vars.extend(self.return_type.free_vars());

        vars
    }
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let async_prefix = if self.is_async { "async " } else { "" };
        let params_str = self.params.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{}fn({}) -> {}", async_prefix, params_str, self.return_type)
    }
}

/// Method type (function associated with a type)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodType {
    /// The type this method belongs to
    pub self_type: Box<Type>,

    /// Method name
    pub name: String,

    /// Method parameters (excluding self)
    pub params: Vec<Parameter>,

    /// Return type
    pub return_type: Box<Type>,

    /// Whether this is a static method
    pub is_static: bool,

    /// Whether this is an async method
    pub is_async: bool,
}

impl MethodType {
    /// Create a new instance method
    pub fn instance(
        self_type: Type,
        name: String,
        params: Vec<Parameter>,
        return_type: Type
    ) -> Self {
        Self {
            self_type: Box::new(self_type),
            name,
            params,
            return_type: Box::new(return_type),
            is_static: false,
            is_async: false,
        }
    }

    /// Create a new static method
    pub fn static_method(
        self_type: Type,
        name: String,
        params: Vec<Parameter>,
        return_type: Type
    ) -> Self {
        Self {
            self_type: Box::new(self_type),
            name,
            params,
            return_type: Box::new(return_type),
            is_static: true,
            is_async: false,
        }
    }

    /// Create an async method
    pub fn async_method(
        self_type: Type,
        name: String,
        params: Vec<Parameter>,
        return_type: Type,
        is_static: bool
    ) -> Self {
        Self {
            self_type: Box::new(self_type),
            name,
            params,
            return_type: Box::new(return_type),
            is_static,
            is_async: true,
        }
    }

    /// Get the full function signature including self parameter
    pub fn full_signature(&self) -> FunctionType {
        let mut all_params = vec![];

        if !self.is_static {
            // Add self parameter
            all_params.push(Parameter {
                name: "self".to_string(),
                ty: *self.self_type.clone(),
                is_mutable: false, // self is typically immutable
            });
        }

        all_params.extend(self.params.clone());

        FunctionType {
            params: all_params,
            return_type: self.return_type.clone(),
            is_async: self.is_async,
        }
    }

    /// Get all type variables in this method type
    pub fn type_vars(&self) -> std::collections::HashSet<super::generics::TypeVar> {
        let mut vars = std::collections::HashSet::new();

        vars.extend(self.self_type.free_vars());

        for param in &self.params {
            vars.extend(param.ty.free_vars());
        }
        vars.extend(self.return_type.free_vars());

        vars
    }
}

impl fmt::Display for MethodType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prefix = String::new();
        if self.is_static {
            prefix.push_str("static ");
        }
        if self.is_async {
            prefix.push_str("async ");
        }

        let params_str = self.params.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        if prefix.is_empty() {
            write!(f, "{}::{}({}) -> {}",
                   self.self_type, self.name, params_str, self.return_type)
        } else {
            write!(f, "{} {}::{}({}) -> {}",
                   prefix.trim_end(), self.self_type, self.name, params_str, self.return_type)
        }
    }
}

/// Closure type (anonymous function)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureType {
    /// Captured variables and their types
    pub captures: Vec<(String, Type)>,

    /// Function type of the closure
    pub function_type: FunctionType,
}

impl ClosureType {
    /// Create a new closure type
    pub fn new(captures: Vec<(String, Type)>, function_type: FunctionType) -> Self {
        Self {
            captures,
            function_type,
        }
    }

    /// Create a closure with no captures
    pub fn pure(function_type: FunctionType) -> Self {
        Self::new(vec![], function_type)
    }

    /// Check if this closure captures variables
    pub fn has_captures(&self) -> bool {
        !self.captures.is_empty()
    }

    /// Get the number of captured variables
    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    /// Get all type variables in this closure type
    pub fn type_vars(&self) -> std::collections::HashSet<super::generics::TypeVar> {
        let mut vars = self.function_type.type_vars();

        for (_, ty) in &self.captures {
            vars.extend(ty.free_vars());
        }

        vars
    }
}

impl fmt::Display for ClosureType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.captures.is_empty() {
            write!(f, "{}", self.function_type)
        } else {
            let captures_str = self.captures.iter()
                .map(|(name, ty)| format!("{}: {}", name, ty))
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "[{}] {}", captures_str, self.function_type)
        }
    }
}

/// Higher-order function type constructors
pub mod higher_order {
    use super::*;

    /// Create a map function type: (T) -> U => List<T> -> List<U>
    pub fn map_function(
        input_type: Type,
        output_type: Type
    ) -> FunctionType {
        let list_input = helpers::list_type(input_type.clone());
        let list_output = helpers::list_type(output_type.clone());

        FunctionType::new(
            vec![
                Parameter::new("list".to_string(), Type::Constructor(list_input)),
                Parameter::new("f".to_string(), Type::Function(FunctionType::new(
                    vec![Parameter::new("x".to_string(), input_type)],
                    output_type
                ))),
            ],
            Type::Constructor(list_output)
        )
    }

    /// Create a filter function type: (T) -> Bool => List<T> -> List<T>
    pub fn filter_function(element_type: Type) -> FunctionType {
        let list_type = helpers::list_type(element_type.clone());

        FunctionType::new(
            vec![
                Parameter::new("list".to_string(), Type::Constructor(list_type.clone())),
                Parameter::new("predicate".to_string(), Type::Function(FunctionType::new(
                    vec![Parameter::new("x".to_string(), element_type)],
                    Type::Primitive(PrimitiveType::Bool)
                ))),
            ],
            Type::Constructor(list_type)
        )
    }

    /// Create a fold/reduce function type: (U, T) -> U => List<T> -> U -> U
    pub fn fold_function(
        element_type: Type,
        accumulator_type: Type
    ) -> FunctionType {
        let list_type = helpers::list_type(element_type.clone());

        FunctionType::new(
            vec![
                Parameter::new("list".to_string(), Type::Constructor(list_type)),
                Parameter::new("initial".to_string(), accumulator_type.clone()),
                Parameter::new("f".to_string(), Type::Function(FunctionType::new(
                    vec![
                        Parameter::new("acc".to_string(), accumulator_type.clone()),
                        Parameter::new("x".to_string(), element_type),
                    ],
                    accumulator_type.clone()
                ))),
            ],
            accumulator_type
        )
    }

    /// Create a compose function type: (B -> C) -> (A -> B) -> (A -> C)
    pub fn compose_function(
        a: Type,
        b: Type,
        c: Type
    ) -> FunctionType {
        let f1_type = FunctionType::new(
            vec![Parameter::new("x".to_string(), b.clone())],
            c.clone()
        );
        let f2_type = FunctionType::new(
            vec![Parameter::new("x".to_string(), a.clone())],
            b
        );
        let composed_type = FunctionType::new(
            vec![Parameter::new("x".to_string(), a)],
            c
        );

        FunctionType::new(
            vec![
                Parameter::new("f".to_string(), Type::Function(f1_type)),
                Parameter::new("g".to_string(), Type::Function(f2_type)),
            ],
            Type::Function(composed_type)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Type, primitives::PrimitiveType, generics::TypeVar};

    #[test]
    fn test_parameter_creation() {
        let param1 = Parameter::new("x".to_string(), Type::Primitive(PrimitiveType::Number));
        let param2 = Parameter::mutable("y".to_string(), Type::Primitive(PrimitiveType::String));

        assert_eq!(param1.name, "x");
        assert!(!param1.is_mutable);
        assert_eq!(param2.name, "y");
        assert!(param2.is_mutable);

        assert_eq!(format!("{}", param1), "x: number");
        assert_eq!(format!("{}", param2), "mut y: string");
    }

    #[test]
    fn test_function_type() {
        let params = vec![
            Parameter::new("x".to_string(), Type::Primitive(PrimitiveType::Number)),
            Parameter::new("y".to_string(), Type::Primitive(PrimitiveType::String)),
        ];
        let return_type = Type::Primitive(PrimitiveType::Bool);

        let fn_type = FunctionType::new(params, return_type);

        assert_eq!(fn_type.arity(), 2);
        assert!(!fn_type.is_async);
        assert!(!fn_type.has_mutable_params());

        assert_eq!(format!("{}", fn_type), "fn(x: number, y: string) -> bool");

        // Test async function
        let async_fn = FunctionType::async_fn(
            vec![Parameter::new("data".to_string(), Type::Primitive(PrimitiveType::String))],
            Type::Primitive(PrimitiveType::Number)
        );
        assert!(async_fn.is_async);
        assert_eq!(format!("{}", async_fn), "async fn(data: string) -> number");
    }

    #[test]
    fn test_method_type() {
        let self_type = Type::Primitive(PrimitiveType::Number);
        let params = vec![
            Parameter::new("other".to_string(), Type::Primitive(PrimitiveType::Number)),
        ];
        let return_type = Type::Primitive(PrimitiveType::Number);

        let method = MethodType::instance(
            self_type.clone(),
            "add".to_string(),
            params,
            return_type.clone()
        );

        assert!(!method.is_static);
        assert!(!method.is_async);
        assert_eq!(method.name, "add");

        let full_sig = method.full_signature();
        assert_eq!(full_sig.params.len(), 2); // self + other
        assert_eq!(full_sig.params[0].name, "self");

        assert_eq!(format!("{}", method), "number::add(other: number) -> number");

        // Test static method
        let static_method = MethodType::static_method(
            self_type,
            "parse".to_string(),
            vec![Parameter::new("s".to_string(), Type::Primitive(PrimitiveType::String))],
            return_type
        );

        assert!(static_method.is_static);
        assert_eq!(format!("{}", static_method), "static number::parse(s: string) -> number");
    }

    #[test]
    fn test_closure_type() {
        let captures = vec![
            ("x".to_string(), Type::Primitive(PrimitiveType::Number)),
            ("y".to_string(), Type::Primitive(PrimitiveType::String)),
        ];

        let fn_type = FunctionType::new(
            vec![Parameter::new("z".to_string(), Type::Primitive(PrimitiveType::Bool))],
            Type::Primitive(PrimitiveType::Void)
        );

        let closure = ClosureType::new(captures, fn_type);

        assert!(closure.has_captures());
        assert_eq!(closure.capture_count(), 2);
        assert_eq!(format!("{}", closure), "[x: number, y: string] fn(z: bool) -> void");

        // Test pure closure (no captures)
        let pure_closure = ClosureType::pure(FunctionType::new(
            vec![],
            Type::Primitive(PrimitiveType::Number)
        ));

        assert!(!pure_closure.has_captures());
        assert_eq!(format!("{}", pure_closure), "fn() -> number");
    }

    #[test]
    fn test_higher_order_functions() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);
        let bool_type = Type::Primitive(PrimitiveType::Bool);

        // Test map function
        let map_fn = higher_order::map_function(int_type.clone(), string_type.clone());
        assert_eq!(map_fn.arity(), 2);
        assert_eq!(map_fn.params[0].name, "list");
        assert_eq!(map_fn.params[1].name, "f");

        // Test filter function
        let filter_fn = higher_order::filter_function(int_type.clone());
        assert_eq!(filter_fn.arity(), 2);
        assert_eq!(filter_fn.params[0].name, "list");
        assert_eq!(filter_fn.params[1].name, "predicate");

        // Test fold function
        let fold_fn = higher_order::fold_function(int_type.clone(), string_type.clone());
        assert_eq!(fold_fn.arity(), 3);
        assert_eq!(fold_fn.params[0].name, "list");
        assert_eq!(fold_fn.params[1].name, "initial");
        assert_eq!(fold_fn.params[2].name, "f");

        // Test compose function
        let compose_fn = higher_order::compose_function(
            int_type,
            string_type.clone(),
            bool_type
        );
        assert_eq!(compose_fn.arity(), 2);
        assert_eq!(compose_fn.params[0].name, "f");
        assert_eq!(compose_fn.params[1].name, "g");
    }

    #[test]
    fn test_type_vars_in_functions() {
        let mut supply = super::super::generics::TypeVarSupply::new();
        let var_t = supply.fresh_named("T".to_string());
        let var_u = supply.fresh_named("U".to_string());

        let fn_type = FunctionType::new(
            vec![
                Parameter::new("x".to_string(), Type::Variable(var_t.clone())),
            ],
            Type::Variable(var_u.clone())
        );

        let vars = fn_type.type_vars();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&var_t));
        assert!(vars.contains(&var_u));
    }
}