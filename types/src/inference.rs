//! # Inferencia de Tipos
//!
//! Este módulo implementa el algoritmo de inferencia de tipos
//! basado en Hindley-Milner con unificación.

use crate::types::{Type, TypeVar, TypeScheme};
use crate::context::TypeContext;
use crate::error::{TypeError, Result};
use std::collections::HashMap;
use vela_compiler::ast::Expression;

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

    /// Obtener la sustitución mutable (para tests)
    pub fn substitution_mut(&mut self) -> &mut Substitution {
        &mut self.substitution
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

    /// Inferir el tipo de una expresión usando el algoritmo W de Hindley-Milner
    pub fn infer_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(lit) => self.infer_literal(lit),
            Expression::Identifier(ident) => self.infer_identifier(ident),
            Expression::Binary(binary) => self.infer_binary(binary),
            Expression::Unary(unary) => self.infer_unary(unary),
            Expression::Call(call) => self.infer_call(call),
            Expression::MemberAccess(member) => self.infer_member_access(member),
            Expression::IndexAccess(index) => self.infer_index_access(index),
            Expression::ArrayLiteral(array) => self.infer_array_literal(array),
            Expression::TupleLiteral(tuple) => self.infer_tuple_literal(tuple),
            Expression::StructLiteral(struct_lit) => self.infer_struct_literal(struct_lit),
            Expression::Lambda(lambda) => self.infer_lambda(lambda),
            Expression::If(if_expr) => self.infer_if(if_expr),
            Expression::Match(match_expr) => self.infer_match(match_expr),
            Expression::StringInterpolation(interp) => self.infer_string_interpolation(interp),
            Expression::Await(await_expr) => self.infer_await(await_expr),
            Expression::Computed(computed) => self.infer_computed(computed),
        }
    }

    /// Inferir tipo de literal
    fn infer_literal(&mut self, lit: &vela_compiler::ast::Literal) -> Result<Type> {
        match lit.kind.as_str() {
            "number" => Ok(Type::Int), // Simplificado, asumimos Int por ahora
            "float" => Ok(Type::Float),
            "string" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            "none" => Ok(Type::Unit), // None en Vela es Unit
            _ => Err(TypeError::internal(format!("Unknown literal kind: {}", lit.kind))),
        }
    }

    /// Inferir tipo de identificador
    fn infer_identifier(&mut self, ident: &vela_compiler::ast::Identifier) -> Result<Type> {
        // Buscar en el contexto de tipos
        match self.context.lookup_variable(&ident.name) {
            Ok(scheme) => Ok(self.context.instantiate(scheme)),
            Err(_) => Err(TypeError::variable_not_found(ident.name.clone())),
        }
    }

    /// Inferir tipo de expresión binaria
    fn infer_binary(&mut self, binary: &vela_compiler::ast::BinaryExpression) -> Result<Type> {
        let left_ty = self.infer_expression(&binary.left)?;
        let right_ty = self.infer_expression(&binary.right)?;

        match binary.operator.as_str() {
            // Operadores aritméticos
            "+" | "-" | "*" | "/" | "%" => {
                self.unify(&left_ty, &Type::Int)?;
                self.unify(&right_ty, &Type::Int)?;
                Ok(Type::Int)
            }
            // Operadores de comparación
            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                self.unify(&left_ty, &right_ty)?;
                Ok(Type::Bool)
            }
            // Operadores lógicos
            "&&" | "||" => {
                self.unify(&left_ty, &Type::Bool)?;
                self.unify(&right_ty, &Type::Bool)?;
                Ok(Type::Bool)
            }
            // Operador de coalescencia (??)
            "??" => {
                // left debe ser Option<T>, right debe ser T
                let elem_ty = self.fresh_type_var();
                self.unify(&left_ty, &Type::Option(Box::new(elem_ty.clone())))?;
                self.unify(&right_ty, &elem_ty)?;
                Ok(elem_ty)
            }
            _ => Err(TypeError::internal(format!("Unknown binary operator: {}", binary.operator))),
        }
    }

    /// Inferir tipo de expresión unaria
    fn infer_unary(&mut self, unary: &vela_compiler::ast::UnaryExpression) -> Result<Type> {
        let operand_ty = self.infer_expression(&unary.operand)?;

        match unary.operator.as_str() {
            "-" => {
                self.unify(&operand_ty, &Type::Int)?;
                Ok(Type::Int)
            }
            "!" => {
                self.unify(&operand_ty, &Type::Bool)?;
                Ok(Type::Bool)
            }
            _ => Err(TypeError::internal(format!("Unknown unary operator: {}", unary.operator))),
        }
    }

    /// Inferir tipo de llamada a función
    fn infer_call(&mut self, call: &vela_compiler::ast::CallExpression) -> Result<Type> {
        let func_ty = self.infer_expression(&call.callee)?;

        // Inferir tipos de argumentos
        let arg_types: Result<Vec<Type>> = call.arguments.iter()
            .map(|arg| self.infer_expression(arg))
            .collect();
        let arg_types = arg_types?;

        // Crear variable de tipo fresca para el resultado
        let ret_ty = self.fresh_type_var();

        // Unificar con tipo función esperado
        let expected_func_ty = Type::Function {
            params: arg_types,
            ret: Box::new(ret_ty.clone()),
        };

        self.unify(&func_ty, &expected_func_ty)?;
        Ok(ret_ty)
    }

    /// Inferir tipo de acceso a miembro
    fn infer_member_access(&mut self, member: &vela_compiler::ast::MemberAccessExpression) -> Result<Type> {
        let object_ty = self.infer_expression(&member.object)?;

        // Para simplificar, asumimos que el objeto es un record/struct
        // En una implementación completa, esto requeriría información de tipos
        if let Type::Record(fields) = &object_ty {
            if let Some(field_ty) = fields.get(&member.member) {
                Ok(field_ty.clone())
            } else {
                Err(TypeError::field_not_found(member.member.clone(), object_ty))
            }
        } else {
            // Para otros tipos, crear una variable de tipo
            // En la práctica, esto requeriría más información del contexto
            Ok(self.fresh_type_var())
        }
    }

    /// Inferir tipo de acceso por índice
    fn infer_index_access(&mut self, index: &vela_compiler::ast::IndexAccessExpression) -> Result<Type> {
        let object_ty = self.infer_expression(&index.object)?;
        let index_ty = self.infer_expression(&index.index)?;

        // El índice debe ser Int
        self.unify(&index_ty, &Type::Int)?;

        // El objeto debe ser Array<T> o similar
        let elem_ty = self.fresh_type_var();
        self.unify(&object_ty, &Type::Array(Box::new(elem_ty.clone())))?;

        Ok(elem_ty)
    }

    /// Inferir tipo de array literal
    fn infer_array_literal(&mut self, array: &vela_compiler::ast::ArrayLiteral) -> Result<Type> {
        if array.elements.is_empty() {
            // Array vacío: crear variable de tipo fresca
            let elem_ty = self.fresh_type_var();
            Ok(Type::Array(Box::new(elem_ty)))
        } else {
            // Inferir tipo del primer elemento
            let elem_ty = self.infer_expression(&array.elements[0])?;

            // Unificar todos los elementos con el mismo tipo
            for elem in &array.elements[1..] {
                let elem_ty_curr = self.infer_expression(elem)?;
                self.unify(&elem_ty, &elem_ty_curr)?;
            }

            Ok(Type::Array(Box::new(elem_ty)))
        }
    }

    /// Inferir tipo de tuple literal
    fn infer_tuple_literal(&mut self, tuple: &vela_compiler::ast::TupleLiteral) -> Result<Type> {
        let elem_types: Result<Vec<Type>> = tuple.elements.iter()
            .map(|elem| self.infer_expression(elem))
            .collect();

        Ok(Type::Tuple(elem_types?))
    }

    /// Inferir tipo de struct literal
    fn infer_struct_literal(&mut self, struct_lit: &vela_compiler::ast::StructLiteral) -> Result<Type> {
        // Para simplificar, asumimos que el struct existe en el contexto
        // En una implementación completa, esto requeriría lookup de tipos
        let mut fields = HashMap::new();

        for field in &struct_lit.fields {
            let field_ty = self.infer_expression(&field.value)?;
            fields.insert(field.name.clone(), field_ty);
        }

        Ok(Type::Record(fields))
    }

    /// Inferir tipo de lambda expression
    fn infer_lambda(&mut self, lambda: &vela_compiler::ast::LambdaExpression) -> Result<Type> {
        // Crear variables de tipo frescas para parámetros
        let param_types: Vec<Type> = lambda.parameters.iter()
            .map(|_| self.fresh_type_var())
            .collect();

        // Extender contexto con tipos de parámetros
        let mut extended_context = self.context.clone();
        for (param, param_ty) in lambda.parameters.iter().zip(param_types.iter()) {
            let scheme = TypeScheme::mono(param_ty.clone());
            extended_context.add_variable(param.name.clone(), scheme);
        }

        // Crear nuevo motor de inferencia para el cuerpo
        let mut body_inference = TypeInference::new(extended_context);

        // Inferir tipo del cuerpo
        let ret_ty = match &lambda.body {
            vela_compiler::ast::LambdaBody::Expression(expr) => body_inference.infer_expression(expr)?,
            vela_compiler::ast::LambdaBody::Block(block) => {
                // Para simplificar, asumir que el bloque retorna el último statement
                // En una implementación completa, esto sería más complejo
                Type::Unit
            }
        };

        // Unificar sustituciones
        for (var, ty) in body_inference.substitution {
            self.substitution.insert(var, ty);
        }

        Ok(Type::Function {
            params: param_types,
            ret: Box::new(ret_ty),
        })
    }

    /// Inferir tipo de if expression
    fn infer_if(&mut self, if_expr: &vela_compiler::ast::IfExpression) -> Result<Type> {
        // Condición debe ser Bool
        let cond_ty = self.infer_expression(&if_expr.condition)?;
        self.unify(&cond_ty, &Type::Bool)?;

        // Ambos branches deben tener el mismo tipo
        let then_ty = self.infer_expression(&if_expr.then_branch)?;
        let else_ty = self.infer_expression(&if_expr.else_branch)?;

        self.unify(&then_ty, &else_ty)?;
        Ok(then_ty)
    }

    /// Inferir tipo de match expression
    fn infer_match(&mut self, match_expr: &vela_compiler::ast::MatchExpression) -> Result<Type> {
        // Inferir tipo del valor a matchear
        let _value_ty = self.infer_expression(&match_expr.value)?;

        // Para simplificar, asumir que todos los arms tienen el mismo tipo de retorno
        // En una implementación completa, esto sería más complejo
        if let Some(first_arm) = match_expr.arms.first() {
            self.infer_expression(&first_arm.body)
        } else {
            Ok(Type::Unit)
        }
    }

    /// Inferir tipo de string interpolation
    fn infer_string_interpolation(&mut self, interp: &vela_compiler::ast::StringInterpolation) -> Result<Type> {
        // Verificar que todas las expresiones sean válidas
        for part in &interp.parts {
            if let vela_compiler::ast::StringInterpolationPart::Expression(expr) = part {
                self.infer_expression(expr)?;
            }
        }
        Ok(Type::String)
    }

    /// Inferir tipo de await expression
    fn infer_await(&mut self, await_expr: &vela_compiler::ast::AwaitExpression) -> Result<Type> {
        let inner_ty = self.infer_expression(&await_expr.expression)?;

        // Await unwraps futures/promises
        // Para simplificar, asumimos que el tipo interno es el resultado
        // En una implementación completa, esto manejaría tipos Future<T>
        Ok(inner_ty)
    }

    /// Inferir tipo de computed expression
    fn infer_computed(&mut self, computed: &vela_compiler::ast::ComputedExpression) -> Result<Type> {
        // Computed expressions son reactivas, pero para inferencia de tipos
        // tratamos el bloque como una expresión normal
        // En una implementación completa, esto sería más complejo
        Ok(Type::Unit)
    }

    /// Crear una nueva variable de tipo fresca
    pub fn fresh_type_var(&mut self) -> Type {
        // En una implementación real, esto debería generar variables únicas
        // Por simplicidad, usamos un contador global
        static mut COUNTER: usize = 0;
        unsafe {
            COUNTER += 1;
            Type::Var(TypeVar(COUNTER))
        }
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

/// Algoritmo W de Hindley-Milner simplificado
pub fn algorithm_w(context: &TypeContext, expr: &vela_compiler::ast::Expression) -> Result<(Type, Substitution)> {
    let mut inference = TypeInference::new(context.clone());

    let ty = inference.infer_expression(expr)?;
    let final_ty = inference.apply_subst(ty);
    Ok((final_ty, inference.substitution))
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