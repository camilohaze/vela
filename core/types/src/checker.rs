//! # Type Checker
//!
//! Este módulo implementa el type checker principal que verifica
//! tipos en el AST de Vela usando inferencia Hindley-Milner.

use crate::types::{Type, TypeScheme, TypeVar};
use crate::context::TypeContext;
use crate::inference::{TypeInference, Substitution};
use crate::error::{TypeError, Result};
use std::collections::HashMap;

/// Resultado del type checking
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Tipo inferido
    pub ty: Type,
    /// Sustitución generada durante la inferencia
    pub substitution: Substitution,
    /// Variables de tipo libres en el resultado
    pub free_vars: Vec<TypeVar>,
}

/// Type checker principal para el lenguaje Vela
#[derive(Debug)]
pub struct TypeChecker {
    /// Contexto de tipos
    context: TypeContext,
    /// Motor de inferencia
    inference: TypeInference,
}

impl TypeChecker {
    /// Crear un nuevo type checker
    pub fn new() -> Self {
        let context = TypeContext::new();
        let inference = TypeInference::new(context.clone());

        Self {
            context,
            inference,
        }
    }

    /// Crear un type checker con contexto personalizado
    pub fn with_context(context: TypeContext) -> Self {
        let inference = TypeInference::new(context.clone());

        Self {
            context,
            inference,
        }
    }

    /// Obtener el contexto de tipos (inmutable)
    pub fn context(&self) -> &TypeContext {
        &self.context
    }

    /// Obtener el contexto de tipos (mutable)
    pub fn context_mut(&mut self) -> &mut TypeContext {
        &mut self.context
    }

    /// Verificar tipos en una expresión
    pub fn check_expression(&mut self, expr: &Expression) -> Result<TypeCheckResult> {
        let ty = self.infer_expression(expr)?;
        let substitution = self.inference.substitution().clone();
        let free_vars = ty.free_vars();

        Ok(TypeCheckResult {
            ty,
            substitution,
            free_vars,
        })
    }

    /// Verificar tipos en un statement
    pub fn check_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Expression(expr_stmt) => {
                self.check_expression_statement(expr_stmt)?;
            }
            Statement::Variable(var_decl) => {
                self.check_variable_declaration(var_decl)?;
            }
            Statement::Assignment(assign_stmt) => {
                self.check_assignment_statement(assign_stmt)?;
            }
            Statement::Return(ret_stmt) => {
                self.check_return_statement(ret_stmt)?;
            }
            Statement::If(if_stmt) => {
                self.check_if_statement(if_stmt)?;
            }
            Statement::Match(match_stmt) => {
                self.check_match_statement(match_stmt)?;
            }
            Statement::Block(block_stmt) => {
                self.check_block_statement(block_stmt)?;
            }
            Statement::Throw(throw_stmt) => {
                self.check_throw_statement(throw_stmt)?;
            }
            Statement::Try(try_stmt) => {
                self.check_try_statement(try_stmt)?;
            }
            Statement::EventOn(event_on) => {
                self.check_event_on_statement(event_on)?;
            }
            Statement::EventEmit(event_emit) => {
                self.check_event_emit_statement(event_emit)?;
            }
            Statement::EventOff(event_off) => {
                self.check_event_off_statement(event_off)?;
            }
            Statement::Dispatch(dispatch_stmt) => {
                self.check_dispatch_statement(dispatch_stmt)?;
            }
        }
        Ok(())
    }

    /// Verificar tipos en una declaración
    pub fn check_declaration(&mut self, decl: &Declaration) -> Result<()> {
        match decl {
            Declaration::Function(func_decl) => {
                self.check_function_declaration(func_decl);
            }
            Declaration::Struct(struct_decl) => {
                self.check_struct_declaration(struct_decl);
            }
            Declaration::Enum(enum_decl) => {
                self.check_enum_declaration(enum_decl);
            }
            Declaration::TypeAlias(type_alias) => {
                self.check_type_alias_declaration(type_alias);
            }
            Declaration::Variable(var_decl) => {
                self.check_variable_declaration(var_decl);
            }
            Declaration::Interface(interface_decl) => {
                self.check_interface_declaration(interface_decl);
            }
            Declaration::Class(class_decl) => {
                self.check_class_declaration(class_decl);
            }
            Declaration::Service(service_decl) => {
                self.check_service_declaration(service_decl);
            }
            Declaration::Repository(repo_decl) => {
                self.check_repository_declaration(repo_decl);
            }
            Declaration::Controller(ctrl_decl) => {
                self.check_controller_declaration(ctrl_decl);
            }
            Declaration::UseCase(usecase_decl) => {
                self.check_usecase_declaration(usecase_decl);
            }
            Declaration::Entity(entity_decl) => {
                self.check_entity_declaration(entity_decl);
            }
            Declaration::ValueObject(vo_decl) => {
                self.check_valueobject_declaration(vo_decl);
            }
            Declaration::DTO(dto_decl) => {
                self.check_dto_declaration(dto_decl);
            }
            Declaration::Widget(widget_decl) => {
                self.check_widget_declaration(widget_decl);
            }
            Declaration::Component(comp_decl) => {
                self.check_component_declaration(comp_decl);
            }
            Declaration::Model(model_decl) => {
                self.check_model_declaration(model_decl);
            }
            Declaration::Factory(factory_decl) => {
                self.check_factory_declaration(factory_decl);
            }
            Declaration::Builder(builder_decl) => {
                self.check_builder_declaration(builder_decl);
            }
            Declaration::Strategy(strategy_decl) => {
                self.check_strategy_declaration(strategy_decl);
            }
            Declaration::Observer(observer_decl) => {
                self.check_observer_declaration(observer_decl);
            }
            Declaration::Singleton(singleton_decl) => {
                self.check_singleton_declaration(singleton_decl);
            }
            Declaration::Adapter(adapter_decl) => {
                self.check_adapter_declaration(adapter_decl);
            }
            Declaration::Decorator(decorator_decl) => {
                self.check_decorator_declaration(decorator_decl);
            }
            Declaration::Guard(guard_decl) => {
                self.check_guard_declaration(guard_decl);
            }
            Declaration::Middleware(mw_decl) => {
                self.check_middleware_declaration(mw_decl);
            }
            Declaration::Interceptor(interceptor_decl) => {
                self.check_interceptor_declaration(interceptor_decl);
            }
            Declaration::Validator(validator_decl) => {
                self.check_validator_declaration(validator_decl);
            }
            Declaration::Store(store_decl) => {
                self.check_store_declaration(store_decl);
            }
            Declaration::Provider(provider_decl) => {
                self.check_provider_declaration(provider_decl);
            }
            Declaration::Actor(actor_decl) => {
                self.check_actor_declaration(actor_decl);
            }
            Declaration::Pipe(pipe_decl) => {
                self.check_pipe_declaration(pipe_decl);
            }
            Declaration::Task(task_decl) => {
                self.check_task_declaration(task_decl);
            }
            Declaration::Helper(helper_decl) => {
                self.check_helper_declaration(helper_decl);
            }
            Declaration::Mapper(mapper_decl) => {
                self.check_mapper_declaration(mapper_decl);
            }
            Declaration::Serializer(serializer_decl) => {
                self.check_serializer_declaration(serializer_decl);
            }
            Declaration::Module(module_decl) => {
                self.check_module_declaration(module_decl);
            }
        }
        Ok(())
    }

    /// Inferir el tipo de una expresión
    fn infer_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(lit) => self.infer_literal(lit),
            Expression::Identifier(ident) => self.infer_identifier(ident),
            Expression::Binary(binary) => self.infer_binary_expression(binary),
            Expression::Unary(unary) => self.infer_unary_expression(unary),
            Expression::Call(call) => self.infer_call_expression(call),
            Expression::MemberAccess(member) => self.infer_member_access(member),
            Expression::IndexAccess(index) => self.infer_index_access(index),
            Expression::ArrayLiteral(array) => self.infer_array_literal(array),
            Expression::TupleLiteral(tuple) => self.infer_tuple_literal(tuple),
            Expression::StructLiteral(struct_lit) => self.infer_struct_literal(struct_lit),
            Expression::Lambda(lambda) => self.infer_lambda_expression(lambda),
            Expression::If(if_expr) => self.infer_if_expression(if_expr),
            Expression::Match(match_expr) => self.infer_match_expression(match_expr),
            Expression::StringInterpolation(interp) => self.infer_string_interpolation(interp),
            Expression::Await(await_expr) => self.infer_await_expression(await_expr),
            Expression::Computed(computed) => self.infer_computed_expression(computed),
        }
    }

    /// Inferir tipo de literal
    fn infer_literal(&mut self, lit: &Literal) -> Result<Type> {
        match lit.kind.as_str() {
            "number" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "string" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            "none" => Ok(Type::Unit), // None se trata como Unit
            _ => Err(TypeError::internal(format!("Unknown literal kind: {}", lit.kind))),
        }
    }

    /// Inferir tipo de identificador
    fn infer_identifier(&mut self, ident: &Identifier) -> Result<Type> {
        match self.context.lookup_variable(&ident.name) {
            Ok(scheme) => {
                let ty = self.context.instantiate(scheme);
                Ok(ty)
            }
            Err(_) => Err(TypeError::variable_not_found(&ident.name)),
        }
    }

    /// Inferir tipo de expresión binaria
    fn infer_binary_expression(&mut self, binary: &BinaryExpression) -> Result<Type> {
        let left_ty = self.infer_expression(&binary.left)?;
        let right_ty = self.infer_expression(&binary.right)?;

        match binary.operator.as_str() {
            // Operadores aritméticos
            "+" | "-" | "*" | "/" | "%" => {
                self.inference.unify(&left_ty, &Type::Int)?;
                self.inference.unify(&right_ty, &Type::Int)?;
                Ok(Type::Int)
            }
            "**" => {
                self.inference.unify(&left_ty, &Type::Float)?;
                self.inference.unify(&right_ty, &Type::Float)?;
                Ok(Type::Float)
            }
            // Operadores de comparación
            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                self.inference.unify(&left_ty, &right_ty)?;
                Ok(Type::Bool)
            }
            // Operadores lógicos
            "&&" | "||" => {
                self.inference.unify(&left_ty, &Type::Bool)?;
                self.inference.unify(&right_ty, &Type::Bool)?;
                Ok(Type::Bool)
            }
            // Operador de coalescencia
            "??" => {
                // left debe ser Option<T>, right debe ser T
                let elem_ty = self.context.fresh_type_var();
                let option_ty = Type::Option(Box::new(Type::Var(elem_ty)));
                self.inference.unify(&left_ty, &option_ty)?;
                self.inference.unify(&right_ty, &Type::Var(elem_ty))?;
                Ok(Type::Var(elem_ty))
            }
            // Operador de asignación
            "=" => {
                self.inference.unify(&left_ty, &right_ty)?;
                Ok(left_ty)
            }
            _ => Err(TypeError::internal(format!("Unknown binary operator: {}", binary.operator))),
        }
    }

    /// Inferir tipo de expresión unaria
    fn infer_unary_expression(&mut self, unary: &UnaryExpression) -> Result<Type> {
        let operand_ty = self.infer_expression(&unary.operand)?;

        match unary.operator.as_str() {
            "-" => {
                self.inference.unify(&operand_ty, &Type::Int)?;
                Ok(Type::Int)
            }
            "!" => {
                self.inference.unify(&operand_ty, &Type::Bool)?;
                Ok(Type::Bool)
            }
            _ => Err(TypeError::internal(format!("Unknown unary operator: {}", unary.operator))),
        }
    }

    /// Inferir tipo de llamada a función
    fn infer_call_expression(&mut self, call: &CallExpression) -> Result<Type> {
        let callee_ty = self.infer_expression(&call.callee)?;

        // Crear tipos para los argumentos
        let arg_types: Vec<Type> = call.arguments
            .iter()
            .map(|arg| self.infer_expression(arg))
            .collect::<Result<Vec<_>>>()?;

        // Crear variable de tipo para el retorno
        let ret_ty = Type::Var(self.context.fresh_type_var());

        // El tipo de la función debe ser: (arg_types...) -> ret_ty
        let func_ty = Type::Function {
            params: arg_types,
            ret: Box::new(ret_ty.clone()),
        };

        self.inference.unify(&callee_ty, &func_ty)?;

        // Aplicar la sustitución al tipo de retorno
        let result_ty = self.inference.apply_subst(ret_ty);

        Ok(result_ty)
    }

    /// Inferir tipo de acceso a miembro
    fn infer_member_access(&mut self, member: &MemberAccessExpression) -> Result<Type> {
        let object_ty = self.infer_expression(&member.object)?;

        // El objeto debe ser un record
        match object_ty {
            Type::Record(fields) => {
                // Buscar el campo en el record
                match fields.get(&member.member) {
                    Some(field_ty) => Ok(field_ty.clone()),
                    None => Err(TypeError::FieldNotFound {
                        field: member.member.clone(),
                        ty: Type::Record(fields),
                    }),
                }
            }
            _ => Err(TypeError::UnificationError {
                lhs: object_ty,
                rhs: Type::Record(HashMap::new()), // Placeholder - should be more specific
            }),
        }
    }

    /// Inferir tipo de acceso por índice
    fn infer_index_access(&mut self, index: &IndexAccessExpression) -> Result<Type> {
        let object_ty = self.infer_expression(&index.object)?;
        let index_ty = self.infer_expression(&index.index)?;

        // El índice debe ser un entero
        self.inference.unify(&index_ty, &Type::Int)?;

        // El objeto debe ser un array
        let elem_ty = self.context.fresh_type_var();
        let array_ty = Type::Array(Box::new(Type::Var(elem_ty)));
        self.inference.unify(&object_ty, &array_ty)?;

        Ok(Type::Var(elem_ty))
    }

    /// Inferir tipo de array literal
    fn infer_array_literal(&mut self, array: &ArrayLiteral) -> Result<Type> {
        if array.elements.is_empty() {
            // Array vacío - tipo genérico
            let elem_ty = self.context.fresh_type_var();
            return Ok(Type::Array(Box::new(Type::Var(elem_ty))));
        }

        // Inferir tipo del primer elemento
        let first_ty = self.infer_expression(&array.elements[0])?;

        // Todos los elementos deben tener el mismo tipo
        for elem in &array.elements[1..] {
            let elem_ty = self.infer_expression(elem)?;
            self.inference.unify(&first_ty, &elem_ty)?;
        }

        Ok(Type::Array(Box::new(first_ty)))
    }

    /// Inferir tipo de tuple literal
    fn infer_tuple_literal(&mut self, tuple: &TupleLiteral) -> Result<Type> {
        let elem_types: Vec<Type> = tuple.elements
            .iter()
            .map(|elem| self.infer_expression(elem))
            .collect::<Result<Vec<_>>>()?;

        Ok(Type::Tuple(elem_types))
    }

    /// Inferir tipo de struct literal
    fn infer_struct_literal(&mut self, struct_lit: &StructLiteral) -> Result<Type> {
        let mut fields = HashMap::new();

        for field in &struct_lit.fields {
            let field_ty = self.infer_expression(&field.value)?;
            fields.insert(field.name.clone(), field_ty);
        }

        Ok(Type::Record(fields))
    }

    /// Inferir tipo de lambda expression
    fn infer_lambda_expression(&mut self, lambda: &LambdaExpression) -> Result<Type> {
        // Entrar en nuevo scope para parámetros
        self.context.enter_scope();

        // Agregar parámetros al contexto
        let mut param_types = Vec::new();
        for param in &lambda.parameters {
            let param_ty = if let Some(type_ann) = &param.type_annotation {
                self.convert_type_annotation(type_ann)?
            } else {
                // Parámetro sin anotación - crear variable de tipo
                Type::Var(self.context.fresh_type_var())
            };

            let param_scheme = TypeScheme::mono(param_ty.clone());
            self.context.add_variable(&param.name, param_scheme);
            param_types.push(param_ty);
        }

        // Inferir tipo del cuerpo
        let ret_ty = match &lambda.body {
            LambdaBody::Expression(expr) => self.infer_expression(expr)?,
            LambdaBody::Block(block) => {
                self.check_block_statement(block)?;
                // TODO: Inferir tipo de retorno del bloque
                Type::Unit // Placeholder
            }
        };

        // Salir del scope
        self.context.exit_scope()?;

        Ok(Type::Function {
            params: param_types,
            ret: Box::new(ret_ty),
        })
    }

    /// Inferir tipo de if expression
    fn infer_if_expression(&mut self, if_expr: &IfExpression) -> Result<Type> {
        // La condición debe ser booleana
        let cond_ty = self.infer_expression(&if_expr.condition)?;
        self.inference.unify(&cond_ty, &Type::Bool)?;

        // Ambos branches deben tener el mismo tipo
        let then_ty = self.infer_expression(&if_expr.then_branch)?;
        let else_ty = self.infer_expression(&if_expr.else_branch)?;

        self.inference.unify(&then_ty, &else_ty)?;

        Ok(then_ty)
    }

    /// Inferir tipo de match expression
    fn infer_match_expression(&mut self, match_expr: &MatchExpression) -> Result<Type> {
        let _value_ty = self.infer_expression(&match_expr.value)?;

        if match_expr.arms.is_empty() {
            return Err(TypeError::internal("Match expression must have at least one arm"));
        }

        // Inferir tipo del primer brazo
        let first_arm_ty = self.infer_expression(&match_expr.arms[0].body)?;

        // Todos los brazos deben tener el mismo tipo
        for arm in &match_expr.arms[1..] {
            let arm_ty = self.infer_expression(&arm.body)?;
            self.inference.unify(&first_arm_ty, &arm_ty)?;
        }

        Ok(first_arm_ty)
    }

    /// Inferir tipo de string interpolation
    fn infer_string_interpolation(&mut self, _interp: &StringInterpolation) -> Result<Type> {
        // String interpolation siempre produce String
        Ok(Type::String)
    }

    /// Inferir tipo de await expression
    fn infer_await_expression(&mut self, await_expr: &AwaitExpression) -> Result<Type> {
        let inner_ty = self.infer_expression(&await_expr.expression)?;

        // Await quita el wrapper Future/Promise
        // Por simplicidad, asumimos que el tipo interior es el resultado
        // TODO: Implementar tipos Future más sofisticados
        Ok(inner_ty)
    }

    /// Inferir tipo de computed expression
    fn infer_computed_expression(&mut self, computed: &ComputedExpression) -> Result<Type> {
        // Computed expressions son como funciones sin parámetros
        self.check_block_statement(&computed.body)?;
        // TODO: Inferir tipo de retorno del bloque
        Ok(Type::Unit) // Placeholder
    }

    /// Convertir anotación de tipo del AST a nuestro Type
    fn convert_type_annotation(&self, type_ann: &TypeAnnotation) -> Result<Type> {
        // Implementation remains the same
        match type_ann {
            TypeAnnotation::Named(name) => {
                // For now, create a type variable for named types
                // TODO: Look up in context or predefined types
                Ok(Type::Var(self.context.fresh_type_var()))
            }
            TypeAnnotation::Function(func) => {
                let param_types = func.parameter_types.iter()
                    .map(|ty| self.convert_type_annotation(ty))
                    .collect::<Result<Vec<_>>>()?;
                let ret_ty = self.convert_type_annotation(&*func.return_type)?;
                Ok(Type::Function {
                    params: param_types,
                    ret: Box::new(ret_ty),
                })
            }
            TypeAnnotation::Array(arr) => {
                let elem_ty = self.convert_type_annotation(&arr.element_type)?;
                Ok(Type::Array(Box::new(elem_ty)))
            }
            TypeAnnotation::Tuple(tuple) => {
                let types = tuple.element_types.iter()
                    .map(|ty| self.convert_type_annotation(ty))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Tuple(types))
            }
            TypeAnnotation::Optional(opt) => {
                let inner_ty = self.convert_type_annotation(&opt.inner_type)?;
                Ok(Type::Option(Box::new(inner_ty)))
            }
            TypeAnnotation::Primitive(prim) => {
                self.convert_type_annotation_inner(type_ann)
            }
            TypeAnnotation::Generic(generic) => {
                self.convert_type_annotation_inner(type_ann)
            }
            TypeAnnotation::Union(_union) => {
                // TODO: Implementar tipos union
                Err(TypeError::internal("Union types not yet implemented"))
            }
        }
    }

    /// Helper method to convert optional type annotations
    fn convert_optional_type_annotation(&self, type_ann: Option<&TypeAnnotation>) -> Result<Type> {
        match type_ann {
            Some(ty) => self.convert_type_annotation(ty),
            None => Ok(Type::Var(self.context.fresh_type_var())),
        }
    }

    fn convert_type_annotation_inner(&self, type_ann: &TypeAnnotation) -> Result<Type> {
        match type_ann {
            TypeAnnotation::Primitive(prim) => match prim.name.as_str() {
                "Number" => Ok(Type::Int),
                "Float" => Ok(Type::Float),
                "String" => Ok(Type::String),
                "Bool" => Ok(Type::Bool),
                "void" => Ok(Type::Unit),
                "never" => Ok(Type::Never),
                _ => Err(TypeError::internal(format!("Unknown primitive type: {}", prim.name))),
            },
            TypeAnnotation::Array(arr) => {
                let elem_ty = self.convert_type_annotation(&arr.element_type)?;
                Ok(Type::Array(Box::new(elem_ty)))
            }
            TypeAnnotation::Tuple(tuple) => {
                let elem_types: Vec<Type> = tuple.element_types
                    .iter()
                    .map(|ty| self.convert_type_annotation(ty))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Tuple(elem_types))
            }
            TypeAnnotation::Function(func) => {
                let param_types: Vec<Type> = func.parameter_types
                    .iter()
                    .map(|ty| self.convert_type_annotation(ty))
                    .collect::<Result<Vec<_>>>()?;
                let ret_ty = self.convert_type_annotation(&*func.return_type)?;
                Ok(Type::Function {
                    params: param_types,
                    ret: Box::new(ret_ty),
                })
            }
            TypeAnnotation::Generic(generic) => {
                let args: Vec<Type> = generic.type_arguments
                    .iter()
                    .map(|ty| self.convert_type_annotation(ty))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Generic {
                    name: generic.base_name.clone(),
                    args,
                })
            }
            TypeAnnotation::Union(_union) => {
                // TODO: Implementar tipos union
                Err(TypeError::internal("Union types not yet implemented"))
            }
            TypeAnnotation::Named(named) => {
                // TODO: Resolver tipos nombrados
                Err(TypeError::internal(format!("Named types not yet implemented: {}", named.name)))
            }
            TypeAnnotation::Optional(opt) => {
                let inner_ty = self.convert_type_annotation(&opt.inner_type)?;
                Ok(Type::Option(Box::new(inner_ty)))
            }
        }
    }

    // Métodos para verificar statements
    fn check_expression_statement(&mut self, expr_stmt: &ExpressionStatement) -> Result<()> {
        let _ty = self.infer_expression(&expr_stmt.expression)?;
        // Expression statements no retornan valores, solo verificamos que sean válidos
        Ok(())
    }

    fn check_variable_declaration(&mut self, var_decl: &VariableDeclaration) -> Result<()> {
        let inferred_ty = if let Some(init) = &var_decl.initializer {
            self.infer_expression(init)?
        } else {
            // Variable sin inicializador - requiere anotación de tipo
            return Err(TypeError::internal("Variable declaration without initializer requires type annotation"));
        };

        // Si hay anotación de tipo, verificar compatibilidad
        if let Some(type_ann) = &var_decl.type_annotation {
            let expected_ty = self.convert_type_annotation(type_ann)?;
            self.inference.unify(&inferred_ty, &expected_ty)?;
        }

        // Agregar al contexto
        let scheme = self.context.generalize(inferred_ty);
        self.context.add_variable(&var_decl.name, scheme);

        Ok(())
    }

    fn check_assignment_statement(&mut self, assign_stmt: &AssignmentStatement) -> Result<()> {
        let target_ty = self.infer_expression(&assign_stmt.target)?;
        let value_ty = self.infer_expression(&assign_stmt.value)?;

        self.inference.unify(&target_ty, &value_ty)?;
        Ok(())
    }

    fn check_return_statement(&mut self, _ret_stmt: &ReturnStatement) -> Result<()> {
        // TODO: Verificar que el tipo de retorno coincida con la función
        Ok(())
    }

    fn check_if_statement(&mut self, if_stmt: &IfStatement) -> Result<()> {
        // Condición debe ser booleana
        let cond_ty = self.infer_expression(&if_stmt.condition)?;
        self.inference.unify(&cond_ty, &Type::Bool)?;

        // Verificar then branch
        self.check_statement(&if_stmt.then_branch)?;

        // Verificar else branch si existe
        if let Some(else_branch) = &if_stmt.else_branch {
            self.check_statement(else_branch)?;
        }

        Ok(())
    }

    fn check_match_statement(&mut self, _match_stmt: &MatchStatement) -> Result<()> {
        // TODO: Implementar verificación de match statements
        Ok(())
    }

    fn check_block_statement(&mut self, block_stmt: &BlockStatement) -> Result<()> {
        self.context.enter_scope();

        for stmt in &block_stmt.statements {
            self.check_statement(stmt)?;
        }

        self.context.exit_scope()?;
        Ok(())
    }

    fn check_throw_statement(&mut self, _throw_stmt: &ThrowStatement) -> Result<()> {
        // TODO: Verificar que se lance un tipo válido
        Ok(())
    }

    fn check_try_statement(&mut self, _try_stmt: &TryStatement) -> Result<()> {
        // TODO: Implementar verificación de try-catch
        Ok(())
    }

    fn check_event_on_statement(&mut self, _event_on: &EventOnStatement) -> Result<()> {
        // TODO: Verificar tipos de eventos
        Ok(())
    }

    fn check_event_emit_statement(&mut self, _event_emit: &EventEmitStatement) -> Result<()> {
        // TODO: Verificar tipos de eventos
        Ok(())
    }

    fn check_event_off_statement(&mut self, _event_off: &EventOffStatement) -> Result<()> {
        // TODO: Verificar tipos de eventos
        Ok(())
    }

    fn check_dispatch_statement(&mut self, _dispatch_stmt: &DispatchStatement) -> Result<()> {
        // TODO: Verificar tipos de dispatch
        Ok(())
    }

    // Métodos para verificar declarations
    fn check_function_declaration(&mut self, func_decl: &FunctionDeclaration) -> Result<()> {
        self.context.enter_scope();

        // Agregar parámetros al contexto
        for param in &func_decl.parameters {
            let param_ty = if let Some(type_ann) = &param.type_annotation {
                self.convert_type_annotation(type_ann)?
            } else {
                Type::Var(self.context.fresh_type_var())
            };

            let param_scheme = TypeScheme::mono(param_ty.clone());
            self.context.add_variable(&param.name, param_scheme);
        }

        // Verificar el cuerpo
        self.check_block_statement(&func_decl.body)?;

        // Verificar tipo de retorno si está anotado
        if let Some(_return_ann) = &func_decl.return_type {
            // TODO: Verificar que el cuerpo retorna el tipo correcto
        }

        self.context.exit_scope()?;

        // Agregar función al contexto global
        let func_ty = self.build_function_type(func_decl)?;
        let func_scheme = TypeScheme::mono(func_ty);
        self.context.add_variable(&func_decl.name, func_scheme);

        Ok(())
    }

    fn check_struct_declaration(&mut self, _struct_decl: &StructDeclaration) -> Result<()> {
        // TODO: Implementar verificación de structs
        Ok(())
    }

    fn check_enum_declaration(&mut self, _enum_decl: &EnumDeclaration) -> Result<()> {
        // TODO: Implementar verificación de enums
        Ok(())
    }

    fn check_type_alias_declaration(&mut self, type_alias: &TypeAliasDeclaration) -> Result<()> {
        // Convert the type annotation to internal type
        let aliased_type = self.convert_type_annotation_inner(&type_alias.type_annotation)?;
        
        // Add the type alias to the context
        self.context.add_variable(type_alias.name.clone(), TypeScheme::mono(aliased_type));
        
        Ok(())
    }

    fn check_interface_declaration(&mut self, interface_decl: &InterfaceDeclaration) -> Result<()> {
        // Create interface type as a record with method signatures
        let mut methods = HashMap::new();
        
        for method in &interface_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let interface_type = Type::Record(methods);
        self.context.add_variable(interface_decl.name.clone(), TypeScheme::mono(interface_type));
        Ok(())
    }

    fn check_class_declaration(&mut self, class_decl: &ClassDeclaration) -> Result<()> {
        // Create class type with fields and methods
        let mut fields = HashMap::new();
        let mut methods = HashMap::new();
        
        // Process fields
        for field in &class_decl.fields {
            if let Some(ref type_ann) = field.type_annotation {
                let field_type = self.convert_type_annotation(type_ann)?;
                fields.insert(field.name.clone(), field_type);
            }
        }
        
        // Process methods
        for method in &class_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        // Combine fields and methods into a single record type
        let mut class_record = fields;
        for (method_name, method_type) in methods {
            class_record.insert(method_name, method_type);
        }
        
        let class_type = Type::Record(class_record);
        self.context.add_variable(class_decl.name.clone(), TypeScheme::mono(class_type));
        Ok(())
    }

    fn check_service_declaration(&mut self, service_decl: &ServiceDeclaration) -> Result<()> {
        // Services are similar to classes - treat as record with methods
        let mut methods = HashMap::new();
        
        for method in &service_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let service_type = Type::Record(methods);
        self.context.add_variable(service_decl.name.clone(), TypeScheme::mono(service_type));
        Ok(())
    }

    fn check_repository_declaration(&mut self, repo_decl: &RepositoryDeclaration) -> Result<()> {
        // Repositories are similar to services
        let mut methods = HashMap::new();
        
        for method in &repo_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let repo_type = Type::Record(methods);
        self.context.add_variable(repo_decl.name.clone(), TypeScheme::mono(repo_type));
        Ok(())
    }

    fn check_controller_declaration(&mut self, ctrl_decl: &ControllerDeclaration) -> Result<()> {
        // Controllers have routes - treat as record with route handlers
        let mut routes = HashMap::new();
        
        for route in &ctrl_decl.routes {
            let param_types = route.handler.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(route.handler.return_type.as_ref())?;
            routes.insert(route.path.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let controller_type = Type::Record(routes);
        self.context.add_variable(ctrl_decl.name.clone(), TypeScheme::mono(controller_type));
        Ok(())
    }

    fn check_usecase_declaration(&mut self, usecase_decl: &UseCaseDeclaration) -> Result<()> {
        // Use cases have an execute method
        let param_types = usecase_decl.execute_method.parameters.iter()
            .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
            .collect::<Result<Vec<_>>>()?;
        let return_type = self.convert_optional_type_annotation(usecase_decl.execute_method.return_type.as_ref())?;
        
        let execute_type = Type::Function {
            params: param_types,
            ret: Box::new(return_type),
        };
        
        let mut methods = HashMap::new();
        methods.insert("execute".to_string(), execute_type);
        
        let usecase_type = Type::Record(methods);
        self.context.add_variable(usecase_decl.name.clone(), TypeScheme::mono(usecase_type));
        Ok(())
    }

    fn check_entity_declaration(&mut self, entity_decl: &EntityDeclaration) -> Result<()> {
        // Entities are records with fields
        let mut fields = HashMap::new();
        
        // Add ID field
        let id_type = self.convert_type_annotation(&entity_decl.id_field.type_annotation)?;
        fields.insert(entity_decl.id_field.name.clone(), id_type);
        
        // Add other fields
        for field in &entity_decl.fields {
            let field_type = self.convert_type_annotation(&field.type_annotation)?;
            fields.insert(field.name.clone(), field_type);
        }
        
        let entity_type = Type::Record(fields);
        self.context.add_variable(entity_decl.name.clone(), TypeScheme::mono(entity_type));
        Ok(())
    }

    fn check_valueobject_declaration(&mut self, vo_decl: &ValueObjectDeclaration) -> Result<()> {
        // Value objects are records with fields
        let mut fields = HashMap::new();
        
        for field in &vo_decl.fields {
            let field_type = self.convert_type_annotation(&field.type_annotation)?;
            fields.insert(field.name.clone(), field_type);
        }
        
        let vo_type = Type::Record(fields);
        self.context.add_variable(vo_decl.name.clone(), TypeScheme::mono(vo_type));
        Ok(())
    }

    fn check_dto_declaration(&mut self, dto_decl: &DTODeclaration) -> Result<()> {
        // DTOs are records with fields
        let mut fields = HashMap::new();
        
        for field in &dto_decl.fields {
            let field_type = self.convert_type_annotation(&field.type_annotation)?;
            fields.insert(field.name.clone(), field_type);
        }
        
        let dto_type = Type::Record(fields);
        self.context.add_variable(dto_decl.name.clone(), TypeScheme::mono(dto_type));
        Ok(())
    }

    fn check_widget_declaration(&mut self, widget_decl: &WidgetDeclaration) -> Result<()> {
        // Widgets are similar to classes
        let mut fields = HashMap::new();
        let mut methods = HashMap::new();
        
        // Process fields
        for field in &widget_decl.fields {
            if let Some(ref type_ann) = field.type_annotation {
                let field_type = self.convert_type_annotation(type_ann)?;
                fields.insert(field.name.clone(), field_type);
            }
        }
        
        // Process methods
        for method in &widget_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        // Combine fields and methods
        let mut widget_record = fields;
        for (method_name, method_type) in methods {
            widget_record.insert(method_name, method_type);
        }
        
        let widget_type = Type::Record(widget_record);
        self.context.add_variable(widget_decl.name.clone(), TypeScheme::mono(widget_type));
        Ok(())
    }

    fn check_component_declaration(&mut self, comp_decl: &ComponentDeclaration) -> Result<()> {
        // Components are similar to widgets
        let mut fields = HashMap::new();
        let mut methods = HashMap::new();
        
        // Process fields
        for field in &comp_decl.fields {
            if let Some(ref type_ann) = field.type_annotation {
                let field_type = self.convert_type_annotation(type_ann)?;
                fields.insert(field.name.clone(), field_type);
            }
        }
        
        // Process methods
        for method in &comp_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        // Combine fields and methods
        let mut component_record = fields;
        for (method_name, method_type) in methods {
            component_record.insert(method_name, method_type);
        }
        
        let component_type = Type::Record(component_record);
        self.context.add_variable(comp_decl.name.clone(), TypeScheme::mono(component_type));
        Ok(())
    }

    fn check_model_declaration(&mut self, model_decl: &ModelDeclaration) -> Result<()> {
        // Models are records with fields
        let mut fields = HashMap::new();
        
        for field in &model_decl.fields {
            let field_type = self.convert_type_annotation(&field.type_annotation)?;
            fields.insert(field.name.clone(), field_type);
        }
        
        let model_type = Type::Record(fields);
        self.context.add_variable(model_decl.name.clone(), TypeScheme::mono(model_type));
        Ok(())
    }

    fn check_factory_declaration(&mut self, factory_decl: &FactoryDeclaration) -> Result<()> {
        // Factories have methods
        let mut methods = HashMap::new();
        
        for method in &factory_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let factory_type = Type::Record(methods);
        self.context.add_variable(factory_decl.name.clone(), TypeScheme::mono(factory_type));
        Ok(())
    }

    fn check_builder_declaration(&mut self, builder_decl: &BuilderDeclaration) -> Result<()> {
        // Builders have methods
        let mut methods = HashMap::new();
        
        for method in &builder_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let builder_type = Type::Record(methods);
        self.context.add_variable(builder_decl.name.clone(), TypeScheme::mono(builder_type));
        Ok(())
    }

    fn check_strategy_declaration(&mut self, strategy_decl: &StrategyDeclaration) -> Result<()> {
        // Strategies have methods
        let mut methods = HashMap::new();
        
        for method in &strategy_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let strategy_type = Type::Record(methods);
        self.context.add_variable(strategy_decl.name.clone(), TypeScheme::mono(strategy_type));
        Ok(())
    }

    fn check_observer_declaration(&mut self, observer_decl: &ObserverDeclaration) -> Result<()> {
        // Observers have methods
        let mut methods = HashMap::new();
        
        for method in &observer_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let observer_type = Type::Record(methods);
        self.context.add_variable(observer_decl.name.clone(), TypeScheme::mono(observer_type));
        Ok(())
    }

    fn check_singleton_declaration(&mut self, singleton_decl: &SingletonDeclaration) -> Result<()> {
        // Singletons have fields and methods
        let mut fields = HashMap::new();
        let mut methods = HashMap::new();
        
        // Process fields
        for field in &singleton_decl.fields {
            if let Some(ref type_ann) = field.type_annotation {
                let field_type = self.convert_type_annotation(type_ann)?;
                fields.insert(field.name.clone(), field_type);
            }
        }
        
        // Process methods
        for method in &singleton_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        // Combine fields and methods
        let mut singleton_record = fields;
        for (method_name, method_type) in methods {
            singleton_record.insert(method_name, method_type);
        }
        
        let singleton_type = Type::Record(singleton_record);
        self.context.add_variable(singleton_decl.name.clone(), TypeScheme::mono(singleton_type));
        Ok(())
    }

    fn check_adapter_declaration(&mut self, adapter_decl: &AdapterDeclaration) -> Result<()> {
        // Adapters have methods
        let mut methods = HashMap::new();
        
        for method in &adapter_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let adapter_type = Type::Record(methods);
        self.context.add_variable(adapter_decl.name.clone(), TypeScheme::mono(adapter_type));
        Ok(())
    }

    fn check_decorator_declaration(&mut self, decorator_decl: &DecoratorDeclaration) -> Result<()> {
        // Decorators have methods
        let mut methods = HashMap::new();
        
        for method in &decorator_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let decorator_type = Type::Record(methods);
        self.context.add_variable(decorator_decl.name.clone(), TypeScheme::mono(decorator_type));
        Ok(())
    }

    fn check_guard_declaration(&mut self, guard_decl: &GuardDeclaration) -> Result<()> {
        // Guards have methods
        let mut methods = HashMap::new();
        
        for method in &guard_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let guard_type = Type::Record(methods);
        self.context.add_variable(guard_decl.name.clone(), TypeScheme::mono(guard_type));
        Ok(())
    }

    fn check_middleware_declaration(&mut self, mw_decl: &MiddlewareDeclaration) -> Result<()> {
        // Middleware have methods
        let mut methods = HashMap::new();
        
        for method in &mw_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let mw_type = Type::Record(methods);
        self.context.add_variable(mw_decl.name.clone(), TypeScheme::mono(mw_type));
        Ok(())
    }

    fn check_interceptor_declaration(&mut self, interceptor_decl: &InterceptorDeclaration) -> Result<()> {
        // Interceptors have methods
        let mut methods = HashMap::new();
        
        for method in &interceptor_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let interceptor_type = Type::Record(methods);
        self.context.add_variable(interceptor_decl.name.clone(), TypeScheme::mono(interceptor_type));
        Ok(())
    }

    fn check_validator_declaration(&mut self, validator_decl: &ValidatorDeclaration) -> Result<()> {
        // Validators have methods
        let mut methods = HashMap::new();
        
        for method in &validator_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let validator_type = Type::Record(methods);
        self.context.add_variable(validator_decl.name.clone(), TypeScheme::mono(validator_type));
        Ok(())
    }

    fn check_store_declaration(&mut self, store_decl: &StoreDeclaration) -> Result<()> {
        // Stores have fields and methods
        let mut fields = HashMap::new();
        let mut methods = HashMap::new();
        
        // Process fields
        for field in &store_decl.fields {
            if let Some(ref type_ann) = field.type_annotation {
                let field_type = self.convert_type_annotation(type_ann)?;
                fields.insert(field.name.clone(), field_type);
            }
        }
        
        // Process methods
        for method in &store_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        // Combine fields and methods
        let mut store_record = fields;
        for (method_name, method_type) in methods {
            store_record.insert(method_name, method_type);
        }
        
        let store_type = Type::Record(store_record);
        self.context.add_variable(store_decl.name.clone(), TypeScheme::mono(store_type));
        Ok(())
    }

    fn check_provider_declaration(&mut self, provider_decl: &ProviderDeclaration) -> Result<()> {
        // Providers have methods
        let mut methods = HashMap::new();
        
        for method in &provider_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let provider_type = Type::Record(methods);
        self.context.add_variable(provider_decl.name.clone(), TypeScheme::mono(provider_type));
        Ok(())
    }

    fn check_actor_declaration(&mut self, actor_decl: &ActorDeclaration) -> Result<()> {
        // Actors have fields and methods
        let mut fields = HashMap::new();
        let mut methods = HashMap::new();
        
        // Process fields
        for field in &actor_decl.fields {
            if let Some(ref type_ann) = field.type_annotation {
                let field_type = self.convert_type_annotation(type_ann)?;
                fields.insert(field.name.clone(), field_type);
            }
        }
        
        // Process methods
        for method in &actor_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        // Combine fields and methods
        let mut actor_record = fields;
        for (method_name, method_type) in methods {
            actor_record.insert(method_name, method_type);
        }
        
        let actor_type = Type::Record(actor_record);
        self.context.add_variable(actor_decl.name.clone(), TypeScheme::mono(actor_type));
        Ok(())
    }

    fn check_pipe_declaration(&mut self, pipe_decl: &PipeDeclaration) -> Result<()> {
        // Pipes have methods
        let mut methods = HashMap::new();
        
        for method in &pipe_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let pipe_type = Type::Record(methods);
        self.context.add_variable(pipe_decl.name.clone(), TypeScheme::mono(pipe_type));
        Ok(())
    }

    fn check_task_declaration(&mut self, task_decl: &TaskDeclaration) -> Result<()> {
        // Tasks have methods
        let mut methods = HashMap::new();
        
        for method in &task_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let task_type = Type::Record(methods);
        self.context.add_variable(task_decl.name.clone(), TypeScheme::mono(task_type));
        Ok(())
    }

    fn check_helper_declaration(&mut self, helper_decl: &HelperDeclaration) -> Result<()> {
        // Helpers have methods
        let mut methods = HashMap::new();
        
        for method in &helper_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let helper_type = Type::Record(methods);
        self.context.add_variable(helper_decl.name.clone(), TypeScheme::mono(helper_type));
        Ok(())
    }

    fn check_mapper_declaration(&mut self, mapper_decl: &MapperDeclaration) -> Result<()> {
        // Mappers have methods
        let mut methods = HashMap::new();
        
        for method in &mapper_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let mapper_type = Type::Record(methods);
        self.context.add_variable(mapper_decl.name.clone(), TypeScheme::mono(mapper_type));
        Ok(())
    }

    fn check_serializer_declaration(&mut self, serializer_decl: &SerializerDeclaration) -> Result<()> {
        // Serializers have methods
        let mut methods = HashMap::new();
        
        for method in &serializer_decl.methods {
            let param_types = method.parameters.iter()
                .map(|param| self.convert_optional_type_annotation(param.type_annotation.as_ref()))
                .collect::<Result<Vec<_>>>()?;
            let return_type = self.convert_optional_type_annotation(method.return_type.as_ref())?;
            methods.insert(method.name.clone(), Type::Function {
                params: param_types,
                ret: Box::new(return_type),
            });
        }
        
        let serializer_type = Type::Record(methods);
        self.context.add_variable(serializer_decl.name.clone(), TypeScheme::mono(serializer_type));
        Ok(())
    }

    fn check_module_declaration(&mut self, _module_decl: &ModuleDeclaration) -> Result<()> {
        // Modules are organizational units, not types
        // We could validate module structure here if needed
        Ok(())
    }

    /// Construir el tipo de función a partir de la declaración
    fn build_function_type(&self, func_decl: &FunctionDeclaration) -> Result<Type> {
        let param_types: Vec<Type> = func_decl.parameters
            .iter()
            .map(|param| {
                if let Some(type_ann) = &param.type_annotation {
                    self.convert_type_annotation(type_ann)
                } else {
                    Ok(Type::Var(self.context.fresh_type_var()))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let ret_ty = if let Some(return_ann) = &func_decl.return_type {
            self.convert_type_annotation(return_ann)?
        } else {
            Type::Unit
        };

        Ok(Type::Function {
            params: param_types,
            ret: Box::new(ret_ty),
        })
    }
}

// Re-exportar tipos del AST para conveniencia
pub use vela_compiler::ast::*;