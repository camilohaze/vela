//! # Contexto de Tipos
//!
//! Este módulo implementa la gestión del contexto de tipos,
//! incluyendo scopes, variables y tipos genéricos.

use crate::types::{Type, TypeScheme, TypeVar};
use crate::error::{TypeError, Result};
use std::collections::HashMap;

/// Nivel de scope para gestión de variables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeLevel(pub usize);

/// Contexto de tipos que mantiene el estado del sistema de tipos
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Variables en el contexto actual, mapeadas por nombre
    variables: HashMap<String, TypeScheme>,
    /// Tipos genéricos definidos
    generics: HashMap<String, TypeScheme>,
    /// Nivel de scope actual
    current_level: ScopeLevel,
    /// Stack de scopes para manejo de variables locales
    scope_stack: Vec<HashMap<String, TypeScheme>>,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    /// Crear un nuevo contexto de tipos vacío
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            generics: HashMap::new(),
            current_level: ScopeLevel(0),
            scope_stack: vec![HashMap::new()],
        }
    }

    /// Obtener el nivel de scope actual
    pub fn current_level(&self) -> ScopeLevel {
        self.current_level
    }

    /// Entrar en un nuevo scope
    pub fn enter_scope(&mut self) {
        self.current_level.0 += 1;
        self.scope_stack.push(HashMap::new());
    }

    /// Salir del scope actual
    pub fn exit_scope(&mut self) -> Result<()> {
        if self.scope_stack.len() <= 1 {
            return Err(TypeError::internal("Cannot exit global scope"));
        }

        self.scope_stack.pop();
        self.current_level.0 = self.current_level.0.saturating_sub(1);
        Ok(())
    }

    /// Agregar una variable al contexto actual
    pub fn add_variable(&mut self, name: impl Into<String>, scheme: TypeScheme) {
        let name = name.into();
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name, scheme);
        }
    }

    /// Buscar una variable en el contexto
    pub fn lookup_variable(&self, name: &str) -> Result<&TypeScheme> {
        // Buscar en scopes desde el más interno hacia afuera
        for scope in self.scope_stack.iter().rev() {
            if let Some(scheme) = scope.get(name) {
                return Ok(scheme);
            }
        }

        // Buscar en variables globales
        if let Some(scheme) = self.variables.get(name) {
            return Ok(scheme);
        }

        Err(TypeError::variable_not_found(name))
    }

    /// Verificar si una variable existe en el contexto
    pub fn has_variable(&self, name: &str) -> bool {
        self.lookup_variable(name).is_ok()
    }

    /// Agregar un tipo genérico al contexto
    pub fn add_generic(&mut self, name: impl Into<String>, scheme: TypeScheme) {
        self.generics.insert(name.into(), scheme);
    }

    /// Buscar un tipo genérico
    pub fn lookup_generic(&self, name: &str) -> Result<&TypeScheme> {
        self.generics
            .get(name)
            .ok_or_else(|| TypeError::generic_not_found(name))
    }

    /// Verificar si un tipo genérico existe
    pub fn has_generic(&self, name: &str) -> bool {
        self.generics.contains_key(name)
    }

    /// Crear una nueva variable de tipo fresca
    pub fn fresh_type_var(&self) -> TypeVar {
        TypeVar::fresh()
    }

    /// Generalizar un tipo a un esquema de tipo
    /// Cuantifica todas las variables de tipo libres
    pub fn generalize(&self, ty: Type) -> TypeScheme {
        let free_vars = ty.free_vars();

        // Solo cuantificar variables que no están en el contexto actual
        let mut quantified_vars = Vec::new();
        for var in free_vars {
            // Verificar si la variable está libre en el contexto
            let is_free_in_context = self
                .variables
                .values()
                .chain(self.generics.values())
                .any(|scheme| scheme.vars.contains(&var));

            if !is_free_in_context {
                quantified_vars.push(var);
            }
        }

        TypeScheme::poly(quantified_vars, ty)
    }

    /// Instanciar un esquema de tipo creando variables frescas
    pub fn instantiate(&self, scheme: &TypeScheme) -> Type {
        let mut subst = HashMap::new();

        // Crear variables frescas para cada variable cuantificada
        for &var in &scheme.vars {
            subst.insert(var, Type::Var(self.fresh_type_var()));
        }

        let mut ty = scheme.ty.clone();
        ty.apply_subst(&subst);
        ty
    }

    /// Aplicar una sustitución al contexto completo
    pub fn apply_substitution(&mut self, subst: &HashMap<TypeVar, Type>) {
        // Aplicar sustitución a todas las variables
        for scheme in self.variables.values_mut() {
            scheme.ty.apply_subst(subst);
        }

        // Aplicar sustitución a tipos genéricos
        for scheme in self.generics.values_mut() {
            scheme.ty.apply_subst(subst);
        }

        // Aplicar sustitución a scopes
        for scope in &mut self.scope_stack {
            for scheme in scope.values_mut() {
                scheme.ty.apply_subst(subst);
            }
        }
    }

    /// Obtener todas las variables de tipo libres en el contexto
    pub fn free_vars(&self) -> Vec<TypeVar> {
        let mut vars = Vec::new();

        // Recopilar de variables globales
        for scheme in self.variables.values() {
            vars.extend(scheme.ty.free_vars());
        }

        // Recopilar de tipos genéricos
        for scheme in self.generics.values() {
            vars.extend(scheme.ty.free_vars());
        }

        // Recopilar de scopes
        for scope in &self.scope_stack {
            for scheme in scope.values() {
                vars.extend(scheme.ty.free_vars());
            }
        }

        // Eliminar duplicados
        vars.sort_by_key(|v| v.0);
        vars.dedup();
        vars
    }

    /// Crear un snapshot del contexto para backtracking
    pub fn snapshot(&self) -> TypeContextSnapshot {
        TypeContextSnapshot {
            variables: self.variables.clone(),
            generics: self.generics.clone(),
            current_level: self.current_level,
            scope_stack: self.scope_stack.clone(),
        }
    }

    /// Restaurar el contexto desde un snapshot
    pub fn restore(&mut self, snapshot: TypeContextSnapshot) {
        self.variables = snapshot.variables;
        self.generics = snapshot.generics;
        self.current_level = snapshot.current_level;
        self.scope_stack = snapshot.scope_stack;
    }
}

/// Snapshot del contexto para backtracking
#[derive(Debug, Clone)]
pub struct TypeContextSnapshot {
    variables: HashMap<String, TypeScheme>,
    generics: HashMap<String, TypeScheme>,
    current_level: ScopeLevel,
    scope_stack: Vec<HashMap<String, TypeScheme>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;

    #[test]
    fn test_new_context() {
        let ctx = TypeContext::new();
        assert_eq!(ctx.current_level().0, 0);
        assert_eq!(ctx.scope_stack.len(), 1);
    }

    #[test]
    fn test_enter_exit_scope() {
        let mut ctx = TypeContext::new();
        assert_eq!(ctx.current_level().0, 0);

        ctx.enter_scope();
        assert_eq!(ctx.current_level().0, 1);

        ctx.exit_scope().unwrap();
        assert_eq!(ctx.current_level().0, 0);
    }

    #[test]
    fn test_add_lookup_variable() {
        let mut ctx = TypeContext::new();

        let scheme = TypeScheme::mono(Type::Int);
        ctx.add_variable("x", scheme.clone());

        let found = ctx.lookup_variable("x").unwrap();
        assert_eq!(found, &scheme);
    }

    #[test]
    fn test_variable_not_found() {
        let ctx = TypeContext::new();
        let result = ctx.lookup_variable("nonexistent");
        assert!(matches!(result, Err(TypeError::VariableNotFound { .. })));
    }

    #[test]
    fn test_scope_shadowing() {
        let mut ctx = TypeContext::new();

        // Agregar variable global
        ctx.add_variable("x", TypeScheme::mono(Type::Int));

        // Entrar en scope y agregar variable con mismo nombre
        ctx.enter_scope();
        ctx.add_variable("x", TypeScheme::mono(Type::Bool));

        // Debería encontrar la variable del scope interno
        let found = ctx.lookup_variable("x").unwrap();
        assert_eq!(found.ty, Type::Bool);

        // Salir del scope
        ctx.exit_scope().unwrap();

        // Debería encontrar la variable global
        let found = ctx.lookup_variable("x").unwrap();
        assert_eq!(found.ty, Type::Int);
    }

    #[test]
    fn test_generic_types() {
        let mut ctx = TypeContext::new();

        let scheme = TypeScheme::mono(Type::Generic {
            name: "List".to_string(),
            args: vec![Type::Var(TypeVar(0))],
        });
        ctx.add_generic("List", scheme.clone());

        let found = ctx.lookup_generic("List").unwrap();
        assert_eq!(found, &scheme);
    }

    #[test]
    fn test_fresh_type_var() {
        let ctx = TypeContext::new();
        let tv1 = ctx.fresh_type_var();
        let tv2 = ctx.fresh_type_var();
        assert_ne!(tv1, tv2);
    }

    #[test]
    fn test_generalize() {
        let ctx = TypeContext::new();
        let ty = Type::Function {
            params: vec![Type::Var(TypeVar(1))],
            ret: Box::new(Type::Var(TypeVar(2))),
        };

        let scheme = ctx.generalize(ty);
        assert_eq!(scheme.vars.len(), 2);
    }

    #[test]
    fn test_instantiate() {
        let ctx = TypeContext::new();
        let scheme = TypeScheme::poly(
            vec![TypeVar(1), TypeVar(2)],
            Type::Function {
                params: vec![Type::Var(TypeVar(1))],
                ret: Box::new(Type::Var(TypeVar(2))),
            },
        );

        let instance = ctx.instantiate(&scheme);
        match instance {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 1);
                assert!(matches!(params[0], Type::Var(_)));
                assert!(matches!(*ret, Type::Var(_)));
            }
            _ => panic!("Expected function type"),
        }
    }
}