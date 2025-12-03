//! # Inferencia de Tipos
//!
//! Este módulo implementa el algoritmo de inferencia de tipos
//! basado en Hindley-Milner con unificación.

use crate::types::{Type, TypeVar};
use crate::context::TypeContext;
use crate::error::{TypeError, Result};
use std::collections::HashMap;

/// Sustitución de tipos (mapeo de variables de tipo a tipos)
pub type Substitution = HashMap<TypeVar, Type>;

/// Motor de inferencia de tipos
#[derive(Debug)]
pub struct TypeInference {
    /// Contexto de tipos
    context: TypeContext,
    /// Sustitución acumulada
    substitution: Substitution,
}

impl TypeInference {
    /// Crear un nuevo motor de inferencia
    pub fn new(context: TypeContext) -> Self {
        Self {
            context,
            substitution: HashMap::new(),
        }
    }

    /// Obtener el contexto mutable
    pub fn context_mut(&mut self) -> &mut TypeContext {
        &mut self.context
    }

    /// Obtener el contexto inmutable
    pub fn context(&self) -> &TypeContext {
        &self.context
    }

    /// Obtener la sustitución actual
    pub fn substitution(&self) -> &Substitution {
        &self.substitution
    }

    /// Aplicar la sustitución acumulada a un tipo
    pub fn apply_subst(&self, mut ty: Type) -> Type {
        ty.apply_subst(&self.substitution);
        ty
    }

    /// Unificar dos tipos
    pub fn unify(&mut self, lhs: &Type, rhs: &Type) -> Result<()> {
        let lhs = self.apply_subst(lhs.clone());
        let rhs = self.apply_subst(rhs.clone());

        match (&lhs, &rhs) {
            // Tipos idénticos
            (a, b) if a == b => Ok(()),

            // Variable de tipo con cualquier tipo
            (Type::Var(var), ty) | (ty, Type::Var(var)) => {
                self.unify_var(*var, ty)
            }

            // Funciones
            (
                Type::Function { params: params1, ret: ret1 },
                Type::Function { params: params2, ret: ret2 },
            ) => {
                if params1.len() != params2.len() {
                    return Err(TypeError::wrong_args(params2.len(), params1.len()));
                }

                // Unificar parámetros
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(p1, p2)?;
                }

                // Unificar retorno
                self.unify(ret1, ret2)
            }

            // Arreglos
            (Type::Array(ty1), Type::Array(ty2)) => self.unify(ty1, ty2),

            // Tuplas
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(TypeError::unification(lhs, rhs));
                }

                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    self.unify(t1, t2)?;
                }
                Ok(())
            }

            // Registros
            (Type::Record(fields1), Type::Record(fields2)) => {
                // Verificar que tengan los mismos campos
                if fields1.len() != fields2.len() {
                    return Err(TypeError::unification(lhs, rhs));
                }

                for (name, ty1) in fields1 {
                    if let Some(ty2) = fields2.get(name) {
                        self.unify(ty1, ty2)?;
                    } else {
                        return Err(TypeError::unification(lhs, rhs));
                    }
                }
                Ok(())
            }

            // Variantes
            (Type::Variant(variants1), Type::Variant(variants2)) => {
                if variants1.len() != variants2.len() {
                    return Err(TypeError::unification(lhs, rhs));
                }

                for (name, ty1) in variants1 {
                    if let Some(ty2) = variants2.get(name) {
                        self.unify(ty1, ty2)?;
                    } else {
                        return Err(TypeError::unification(lhs, rhs));
                    }
                }
                Ok(())
            }

            // Option
            (Type::Option(ty1), Type::Option(ty2)) => self.unify(ty1, ty2),

            // Result
            (
                Type::Result { ok: ok1, err: err1 },
                Type::Result { ok: ok2, err: err2 },
            ) => {
                self.unify(ok1, ok2)?;
                self.unify(err1, err2)
            }

            // Genéricos
            (
                Type::Generic { name: name1, args: args1 },
                Type::Generic { name: name2, args: args2 },
            ) => {
                if name1 != name2 || args1.len() != args2.len() {
                    return Err(TypeError::unification(lhs, rhs));
                }

                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    self.unify(arg1, arg2)?;
                }
                Ok(())
            }

            // Tipos incompatibles
            _ => Err(TypeError::unification(lhs, rhs)),
        }
    }

    /// Unificar una variable de tipo con un tipo
    fn unify_var(&mut self, var: TypeVar, ty: &Type) -> Result<()> {
        // Verificar recursión infinita
        if let Type::Var(other_var) = ty {
            if var == *other_var {
                return Ok(()); // Misma variable
            }
        }

        // Verificar si la variable ocurre en el tipo
        if self.occurs_check(var, ty) {
            return Err(TypeError::infinite_type(format!("'{}", var.0), ty.clone()));
        }

        // Aplicar sustitución
        self.substitution.insert(var, ty.clone());
        Ok(())
    }

    /// Verificar si una variable ocurre en un tipo (occurs check)
    fn occurs_check(&self, var: TypeVar, ty: &Type) -> bool {
        match ty {
            Type::Var(other_var) => var == *other_var,
            Type::Array(elem_ty) => self.occurs_check(var, elem_ty),
            Type::Function { params, ret } => {
                params.iter().any(|p| self.occurs_check(var, p)) || self.occurs_check(var, ret)
            }
            Type::Tuple(types) => types.iter().any(|t| self.occurs_check(var, t)),
            Type::Record(fields) => fields.values().any(|t| self.occurs_check(var, t)),
            Type::Variant(variants) => variants.values().any(|t| self.occurs_check(var, t)),
            Type::Option(inner_ty) => self.occurs_check(var, inner_ty),
            Type::Result { ok, err } => {
                self.occurs_check(var, ok) || self.occurs_check(var, err)
            }
            Type::Generic { args, .. } => args.iter().any(|a| self.occurs_check(var, a)),
            _ => false,
        }
    }

    /// Inferir el tipo de una expresión (placeholder para futuras implementaciones)
    pub fn infer_expression(&mut self, _expr: &Expression) -> Result<Type> {
        // TODO: Implementar inferencia de expresiones
        // Este método será implementado cuando integremos con el AST
        Err(TypeError::internal("Expression inference not yet implemented"))
    }

    /// Verificar que un tipo sea válido en el contexto actual
    pub fn check_type(&self, ty: &Type) -> Result<()> {
        match ty {
            Type::Var(var) => {
                // Verificar que la variable esté en la sustitución
                if !self.substitution.contains_key(var) {
                    return Err(TypeError::infinite_type(
                        format!("'{}", var.0),
                        ty.clone(),
                    ));
                }
                Ok(())
            }
            Type::Array(elem_ty) => self.check_type(elem_ty),
            Type::Function { params, ret } => {
                for param in params {
                    self.check_type(param)?;
                }
                self.check_type(ret)
            }
            Type::Tuple(types) => {
                for ty in types {
                    self.check_type(ty)?;
                }
                Ok(())
            }
            Type::Record(fields) => {
                for ty in fields.values() {
                    self.check_type(ty)?;
                }
                Ok(())
            }
            Type::Variant(variants) => {
                for ty in variants.values() {
                    self.check_type(ty)?;
                }
                Ok(())
            }
            Type::Option(inner_ty) => self.check_type(inner_ty),
            Type::Result { ok, err } => {
                self.check_type(ok)?;
                self.check_type(err)
            }
            Type::Generic { args, .. } => {
                for arg in args {
                    self.check_type(arg)?;
                }
                Ok(())
            }
            _ => Ok(()), // Tipos primitivos son siempre válidos
        }
    }
}

/// Placeholder para expresiones (será definido en el AST)
#[derive(Debug)]
pub struct Expression;

/// Algoritmo W de Hindley-Milner simplificado
pub fn algorithm_w(context: &TypeContext, expr: &Expression) -> Result<(Type, Substitution)> {
    let mut inference = TypeInference::new(context.clone());

    // TODO: Implementar algoritmo W completo
    // Por ahora retornamos un error
    let _ty = inference.infer_expression(expr)?;

    Ok((Type::Unit, inference.substitution))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;

    fn create_inference() -> TypeInference {
        TypeInference::new(TypeContext::new())
    }

    #[test]
    fn test_unify_identical_types() {
        let mut inf = create_inference();
        assert!(inf.unify(&Type::Int, &Type::Int).is_ok());
        assert!(inf.unify(&Type::Bool, &Type::Bool).is_ok());
    }

    #[test]
    fn test_unify_different_primitive_types() {
        let mut inf = create_inference();
        let result = inf.unify(&Type::Int, &Type::Bool);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));
    }

    #[test]
    fn test_unify_with_type_var() {
        let mut inf = create_inference();
        let tv = TypeVar(1);

        inf.unify(&Type::Var(tv), &Type::Int).unwrap();

        assert_eq!(inf.substitution.get(&tv), Some(&Type::Int));
    }

    #[test]
    fn test_unify_arrays() {
        let mut inf = create_inference();

        let arr1 = Type::Array(Box::new(Type::Int));
        let arr2 = Type::Array(Box::new(Type::Int));

        assert!(inf.unify(&arr1, &arr2).is_ok());
    }

    #[test]
    fn test_unify_arrays_different_elem_types() {
        let mut inf = create_inference();

        let arr1 = Type::Array(Box::new(Type::Int));
        let arr2 = Type::Array(Box::new(Type::Bool));

        let result = inf.unify(&arr1, &arr2);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));
    }

    #[test]
    fn test_unify_functions() {
        let mut inf = create_inference();

        let func1 = Type::Function {
            params: vec![Type::Int, Type::Bool],
            ret: Box::new(Type::String),
        };

        let func2 = Type::Function {
            params: vec![Type::Int, Type::Bool],
            ret: Box::new(Type::String),
        };

        assert!(inf.unify(&func1, &func2).is_ok());
    }

    #[test]
    fn test_unify_functions_wrong_param_count() {
        let mut inf = create_inference();

        let func1 = Type::Function {
            params: vec![Type::Int],
            ret: Box::new(Type::String),
        };

        let func2 = Type::Function {
            params: vec![Type::Int, Type::Bool],
            ret: Box::new(Type::String),
        };

        let result = inf.unify(&func1, &func2);
        assert!(matches!(result, Err(TypeError::WrongNumberOfArguments { .. })));
    }

    #[test]
    fn test_unify_tuples() {
        let mut inf = create_inference();

        let tuple1 = Type::Tuple(vec![Type::Int, Type::Bool]);
        let tuple2 = Type::Tuple(vec![Type::Int, Type::Bool]);

        assert!(inf.unify(&tuple1, &tuple2).is_ok());
    }

    #[test]
    fn test_unify_tuples_different_lengths() {
        let mut inf = create_inference();

        let tuple1 = Type::Tuple(vec![Type::Int]);
        let tuple2 = Type::Tuple(vec![Type::Int, Type::Bool]);

        let result = inf.unify(&tuple1, &tuple2);
        assert!(matches!(result, Err(TypeError::UnificationError { .. })));
    }

    #[test]
    fn test_occurs_check() {
        let inf = create_inference();
        let tv = TypeVar(1);

        // Var ocurre en sí misma
        assert!(inf.occurs_check(tv, &Type::Var(tv)));

        // Var ocurre en función que la contiene
        let func_ty = Type::Function {
            params: vec![Type::Var(tv)],
            ret: Box::new(Type::Int),
        };
        assert!(inf.occurs_check(tv, &func_ty));

        // Var no ocurre en tipo diferente
        assert!(!inf.occurs_check(tv, &Type::Int));
    }

    #[test]
    fn test_infinite_type_prevention() {
        let mut inf = create_inference();
        let tv = TypeVar(1);

        // Intentar unificar X con (X -> Int) debería fallar
        let func_ty = Type::Function {
            params: vec![Type::Var(tv)],
            ret: Box::new(Type::Int),
        };

        let result = inf.unify(&Type::Var(tv), &func_ty);
        assert!(matches!(result, Err(TypeError::InfiniteType { .. })));
    }
}