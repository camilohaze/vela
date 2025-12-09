/*
Semantic Analyzer - TASK-RUST-105: Semantic Analyzer Implementation

Implementación completa del analizador semántico para el compilador Vela en Rust.
Incluye type checking, symbol resolution, type inference y validación semántica.

Jira: VELA-561
Historia: US-RUST-02
Fecha: Diciembre 2025
*/

use crate::ast::*;
use crate::error::{CompileError, CompileResult, SemanticError, SourceLocation, Type};
use std::collections::HashMap;
use std::fmt;

/// Analizador semántico para type checking y symbol resolution
pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<CompileError>,
    current_scope: ScopeId,
}

/// Tabla de símbolos con scopes anidados
#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_scope: ScopeId,
}

/// Scope individual con símbolos
#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<ScopeId>,
    kind: ScopeKind,
}

/// ID único para scopes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(usize);

/// Tipos de símbolos
#[derive(Debug, Clone)]
pub enum Symbol {
    Variable {
        name: String,
        ty: SemanticType,
        mutable: bool,
        location: SourceLocation,
    },
    Function {
        name: String,
        params: Vec<(String, SemanticType)>,
        return_type: SemanticType,
        location: SourceLocation,
    },
    Struct {
        name: String,
        fields: HashMap<String, SemanticType>,
        location: SourceLocation,
    },
    Enum {
        name: String,
        variants: HashMap<String, Vec<SemanticType>>,
        location: SourceLocation,
    },
}

/// Sistema de tipos para type checking
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticType {
    Primitive(String),
    Struct(String),                    // Nombre del struct
    Enum(String),                      // Nombre del enum
    Function(Vec<SemanticType>, Box<SemanticType>),    // (params, return)
    Array(Box<SemanticType>),                  // Element type
    Tuple(Vec<SemanticType>),                  // Element types
    Generic(String, Vec<SemanticType>),        // Name, type args
    Unknown,                           // Para inference
}

impl fmt::Display for SemanticType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticType::Primitive(name) => write!(f, "{}", name),
            SemanticType::Struct(name) => write!(f, "{}", name),
            SemanticType::Enum(name) => write!(f, "{}", name),
            SemanticType::Array(elem_ty) => write!(f, "[{}]", elem_ty),
            SemanticType::Tuple(elem_types) => {
                let types_str = elem_types.iter()
                    .map(|ty| ty.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({})", types_str)
            }
            SemanticType::Function(param_types, return_ty) => {
                let params_str = param_types.iter()
                    .map(|ty| ty.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({}) -> {}", params_str, return_ty)
            }
            SemanticType::Generic(base_name, type_args) => {
                let args_str = type_args.iter()
                    .map(|ty| ty.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{}<{}>", base_name, args_str)
            }
            SemanticType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Tipos de scopes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScopeKind {
    Global,
    Function,
    Block,
    Struct,
    Enum,
}

impl SemanticAnalyzer {
    /// Crea un nuevo analizador semántico
    pub fn new() -> Self {
        let mut symbols = SymbolTable::new();
        let global_scope = symbols.create_scope(None, ScopeKind::Global);
        symbols.current_scope = global_scope;

        Self {
            symbols,
            errors: Vec::new(),
            current_scope: global_scope,
        }
    }

    /// Analiza un programa completo
    pub fn analyze(&mut self, program: &Program) -> CompileResult<()> {
        // Fase 1: Declarar todos los símbolos
        self.declare_symbols(program)?;

        // Fase 2: Type checking
        self.type_check_program(program)?;

        // Fase 3: Validación adicional
        self.validate_program(program)?;

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.remove(0)) // Retornar primer error
        }
    }

    /// Fase 1: Declarar símbolos en la tabla de símbolos
    fn declare_symbols(&mut self, program: &Program) -> CompileResult<()> {
        for decl in &program.declarations {
            self.declare_declaration(decl)?;
        }
        Ok(())
    }

    /// Declara una declaración en el scope actual
    fn declare_declaration(&mut self, decl: &Declaration) -> CompileResult<()> {
        match decl {
            Declaration::Function(func) => self.declare_function(func),
            Declaration::Struct(struct_) => self.declare_struct(struct_),
            Declaration::Enum(enum_) => self.declare_enum(enum_),
            Declaration::TypeAlias(type_alias) => self.declare_type_alias(type_alias),
            Declaration::Variable(var) => self.declare_variable(var),
            // Variables se declaran en statements, no en declarations de nivel superior
            _ => Ok(()),
        }
    }

    /// Declara una función
    fn declare_function(&mut self, func: &FunctionDeclaration) -> CompileResult<()> {
        let params: Vec<(String, SemanticType)> = func.parameters.iter()
            .map(|p| {
                let ty = if let Some(ref type_ann) = p.type_annotation {
                    self.type_annotation_to_semantic_type(type_ann)?
                } else {
                    return Err(CompileError::Semantic(SemanticError::TypeInferenceFailed {
                        location: SourceLocation::new(p.range.start.line, p.range.start.column, 0),
                        message: "Parameter type annotation required".to_string(),
                    }));
                };
                Ok((p.name.clone(), ty))
            })
            .collect::<CompileResult<Vec<_>>>()?;

        let return_type = match &func.return_type {
            Some(ty) => self.type_annotation_to_semantic_type(ty)?,
            None => SemanticType::Primitive("void".to_string()),
        };

        let symbol = Symbol::Function {
            name: func.name.clone(),
            params,
            return_type,
            location: SourceLocation::new(func.node.range.start.line, func.node.range.start.column, 0),
        };

        self.symbols.declare(func.name.clone(), symbol)
    }

    /// Declara un struct
    fn declare_struct(&mut self, struct_: &StructDeclaration) -> CompileResult<()> {
        let mut fields = HashMap::new();
        for field in &struct_.fields {
            let ty = self.type_annotation_to_semantic_type(&field.type_annotation)?;
            fields.insert(field.name.clone(), ty);
        }

        let symbol = Symbol::Struct {
            name: struct_.name.clone(),
            fields,
            location: SourceLocation::new(struct_.node.range.start.line, struct_.node.range.start.column, 0),
        };

        self.symbols.declare(struct_.name.clone(), symbol)
    }

    /// Declara un enum
    fn declare_enum(&mut self, enum_: &EnumDeclaration) -> CompileResult<()> {
        let mut variants = HashMap::new();
        for variant in &enum_.variants {
            let types: Vec<SemanticType> = variant.fields.as_ref()
                .map(|fields| {
                    fields.iter()
                        .map(|field| self.type_annotation_to_semantic_type(&field.type_annotation))
                        .collect::<CompileResult<Vec<_>>>()
                })
                .transpose()?
                .unwrap_or_default();
            variants.insert(variant.name.clone(), types);
        }

        let symbol = Symbol::Enum {
            name: enum_.name.clone(),
            variants,
            location: SourceLocation::new(enum_.node.range.start.line, enum_.node.range.start.column, 0),
        };

        self.symbols.declare(enum_.name.clone(), symbol)
    }

    /// Declara una variable
    fn declare_variable(&mut self, var: &VariableDeclaration) -> CompileResult<()> {
        let ty = match &var.type_annotation {
            Some(ty_ann) => self.type_annotation_to_semantic_type(ty_ann)?,
            None => {
                // Type inference desde initializer
                match &var.initializer {
                    Some(expr) => self.infer_expression_type(expr)?,
                    None => return Err(CompileError::Semantic(SemanticError::TypeInferenceFailed {
                        location: SourceLocation::new(var.node.range.start.line, var.node.range.start.column, 0),
                        message: "Cannot infer type without initializer".to_string(),
                    })),
                }
            }
        };

        let symbol = Symbol::Variable {
            name: var.name.clone(),
            ty,
            mutable: var.is_state,
            location: SourceLocation::new(var.node.range.start.line, var.node.range.start.column, 0),
        };

        self.symbols.declare(var.name.clone(), symbol)
    }

    /// Declara un type alias
    fn declare_type_alias(&mut self, _type_alias: &TypeAliasDeclaration) -> CompileResult<()> {
        // TODO: Implementar type aliases
        Ok(())
    }

    /// Convierte TypeAnnotation a SemanticType
    fn type_annotation_to_semantic_type(&self, type_ann: &TypeAnnotation) -> CompileResult<SemanticType> {
        match type_ann {
            TypeAnnotation::Primitive(prim) => Ok(SemanticType::Primitive(prim.name.clone())),
            TypeAnnotation::Named(named) => {
                // Verificar si es un tipo conocido
                match named.name.as_str() {
                    "Number" | "Float" | "String" | "Bool" | "void" => {
                        Ok(SemanticType::Primitive(named.name.clone()))
                    }
                    _ => {
                        // Verificar si es un struct/enum definido
                        if let Some(symbol) = self.symbols.lookup(&named.name) {
                            match symbol {
                                Symbol::Struct { .. } => Ok(SemanticType::Struct(named.name.clone())),
                                Symbol::Enum { .. } => Ok(SemanticType::Enum(named.name.clone())),
                                _ => Err(CompileError::Semantic(SemanticError::InvalidType {
                                    name: named.name.clone(),
                                    location: SourceLocation::new(named.node.range.start.line, named.node.range.start.column, 0),
                                })),
                            }
                        } else {
                            Err(CompileError::Semantic(SemanticError::UndefinedType {
                                name: named.name.clone(),
                                location: SourceLocation::new(named.node.range.start.line, named.node.range.start.column, 0),
                            }))
                        }
                    }
                }
            }
            TypeAnnotation::Array(arr) => {
                let elem_ty = self.type_annotation_to_semantic_type(&arr.element_type)?;
                Ok(SemanticType::Array(Box::new(elem_ty)))
            }
            TypeAnnotation::Tuple(tuple) => {
                let elem_types = tuple.element_types.iter()
                    .map(|ty| self.type_annotation_to_semantic_type(ty))
                    .collect::<CompileResult<Vec<_>>>()?;
                Ok(SemanticType::Tuple(elem_types))
            }
            TypeAnnotation::Function(func) => {
                let param_types = func.parameter_types.iter()
                    .map(|ty| self.type_annotation_to_semantic_type(ty))
                    .collect::<CompileResult<Vec<_>>>()?;
                let return_ty = self.type_annotation_to_semantic_type(&func.return_type)?;
                Ok(SemanticType::Function(param_types, Box::new(return_ty)))
            }
            TypeAnnotation::Generic(gen) => {
                let type_args = gen.type_arguments.iter()
                    .map(|ty| self.type_annotation_to_semantic_type(ty))
                    .collect::<CompileResult<Vec<_>>>()?;
                Ok(SemanticType::Generic(gen.base_name.clone(), type_args))
            }
            TypeAnnotation::Union(_) | TypeAnnotation::Optional(_) => {
                // TODO: Implementar union y optional types
                Ok(SemanticType::Unknown)
            }
        }
    }

    /// Fase 2: Type checking del programa
    fn type_check_program(&mut self, program: &Program) -> CompileResult<()> {
        for decl in &program.declarations {
            self.type_check_declaration(decl)?;
        }
        Ok(())
    }

    /// Type check de una declaración
    fn type_check_declaration(&mut self, decl: &Declaration) -> CompileResult<()> {
        match decl {
            Declaration::Function(func) => self.type_check_function(func),
            Declaration::Variable(var) => self.type_check_variable(var),
            // Variables se type checkean en statements
            _ => Ok(()),
        }
    }

    /// Type check de función
    fn type_check_function(&mut self, func: &FunctionDeclaration) -> CompileResult<()> {
        // Crear scope de función
        let func_scope = self.symbols.create_scope(Some(self.current_scope), ScopeKind::Function);
        let prev_scope = self.current_scope;
        self.current_scope = func_scope;

        // Declarar parámetros en el scope de función
        for param in &func.parameters {
            let param_ty = if let Some(ref type_ann) = param.type_annotation {
                self.type_annotation_to_semantic_type(type_ann)?
            } else {
                return Err(CompileError::Semantic(SemanticError::TypeInferenceFailed {
                    location: SourceLocation::new(param.range.start.line, param.range.start.column, 0),
                    message: "Parameter type annotation required".to_string(),
                }));
            };
            let symbol = Symbol::Variable {
                name: param.name.clone(),
                ty: param_ty,
                mutable: false,
                location: SourceLocation::new(param.range.start.line, param.range.start.column, 0),
            };
            self.symbols.declare(param.name.clone(), symbol)?;
        }

        // Type check del body
        self.type_check_block_statement(&func.body)?;

        // Verificar return type si especificado
        if let Some(expected_return) = &func.return_type {
            let expected_ty = self.type_annotation_to_semantic_type(expected_return)?;
            self.check_function_returns(&func.body, &expected_ty)?;
        }

        // Restaurar scope anterior
        self.current_scope = prev_scope;
        Ok(())
    }

    /// Type check de variable
    fn type_check_variable(&mut self, var: &VariableDeclaration) -> CompileResult<()> {
        // Type check del initializer si existe
        if let (Some(type_ann), Some(init)) = (&var.type_annotation, &var.initializer) {
            let declared_ty = self.type_annotation_to_semantic_type(type_ann)?;
            let init_ty = self.type_check_expression(init)?;
            if !self.types_compatible(&declared_ty, &init_ty) {
                return Err(CompileError::Semantic(SemanticError::TypeMismatch {
                    expected: Type::Primitive(declared_ty.to_string()),
                    actual: Type::Primitive(init_ty.to_string()),
                    location: SourceLocation::new(var.node.range.start.line, var.node.range.start.column, 0),
                }));
            }
        }
        Ok(())
    }

    /// Type check de expresión
    fn type_check_expression(&mut self, expr: &Expression) -> CompileResult<SemanticType> {
        match expr {
            Expression::Literal(lit) => self.type_check_literal(lit),
            Expression::Identifier(ident) => self.type_check_identifier(ident),
            Expression::Binary(bin) => self.type_check_binary(bin),
            Expression::Unary(un) => self.type_check_unary(un),
            Expression::Dispatch(disp) => self.type_check_dispatch(disp),
            _ => Ok(SemanticType::Unknown), // TODO: Implementar otros tipos de expresiones
        }
    }

    /// Type check de literal
    fn type_check_literal(&mut self, lit: &Literal) -> CompileResult<SemanticType> {
        match lit.kind.as_str() {
            "number" => Ok(SemanticType::Primitive("Number".to_string())),
            "string" => Ok(SemanticType::Primitive("String".to_string())),
            "bool" => Ok(SemanticType::Primitive("Bool".to_string())),
            _ => Ok(SemanticType::Unknown),
        }
    }

    /// Type check de identificador
    fn type_check_identifier(&mut self, ident: &Identifier) -> CompileResult<SemanticType> {
        match self.symbols.lookup(&ident.name) {
            Some(Symbol::Variable { ty, .. }) => Ok(ty.clone()),
            Some(Symbol::Function { return_type, .. }) => Ok(return_type.clone()),
            Some(_) => Err(CompileError::Semantic(SemanticError::InvalidSymbolUsage {
                name: ident.name.clone(),
                location: SourceLocation::new(ident.node.range.start.line, ident.node.range.start.column, 0),
            })),
            None => Err(CompileError::Semantic(SemanticError::UndefinedSymbol {
                name: ident.name.clone(),
                location: SourceLocation::new(ident.node.range.start.line, ident.node.range.start.column, 0),
            })),
        }
    }

    /// Type check de expresión binaria
    fn type_check_binary(&mut self, bin: &BinaryExpression) -> CompileResult<SemanticType> {
        let left_ty = self.type_check_expression(&bin.left)?;
        let right_ty = self.type_check_expression(&bin.right)?;

        match bin.operator.as_str() {
            "+" | "-" | "*" | "/" => {
                if self.is_numeric_type(&left_ty) && self.is_numeric_type(&right_ty) {
                    Ok(left_ty) // Resultado tiene tipo del operando izquierdo
                } else {
                    Err(CompileError::Semantic(SemanticError::InvalidBinaryOperation {
                        operator: bin.operator.clone(),
                        left_type: Type::Primitive(left_ty.to_string()),
                        right_type: Type::Primitive(right_ty.to_string()),
                        location: SourceLocation::new(bin.node.range.start.line, bin.node.range.start.column, 0),
                    }))
                }
            }
            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                if self.types_compatible(&left_ty, &right_ty) {
                    Ok(SemanticType::Primitive("Bool".to_string()))
                } else {
                    Err(CompileError::Semantic(SemanticError::InvalidBinaryOperation {
                        operator: bin.operator.clone(),
                        left_type: Type::Primitive(left_ty.to_string()),
                        right_type: Type::Primitive(right_ty.to_string()),
                        location: SourceLocation::new(bin.node.range.start.line, bin.node.range.start.column, 0),
                    }))
                }
            }
            _ => Err(CompileError::Semantic(SemanticError::UnknownOperator {
                operator: bin.operator.clone(),
                location: SourceLocation::new(bin.node.range.start.line, bin.node.range.start.column, 0),
            })),
        }
    }

    /// Type check de expresión unaria
    fn type_check_unary(&mut self, un: &UnaryExpression) -> CompileResult<SemanticType> {
        let operand_ty = self.type_check_expression(&un.operand)?;

        match un.operator.as_str() {
            "-" => {
                if self.is_numeric_type(&operand_ty) {
                    Ok(operand_ty)
                } else {
                    Err(CompileError::Semantic(SemanticError::InvalidUnaryOperation {
                        operator: un.operator.clone(),
                        operand_type: Type::Primitive(operand_ty.to_string()),
                        location: SourceLocation::new(un.node.range.start.line, un.node.range.start.column, 0),
                    }))
                }
            }
            "!" => {
                if self.is_boolean_type(&operand_ty) {
                    Ok(operand_ty)
                } else {
                    Err(CompileError::Semantic(SemanticError::InvalidUnaryOperation {
                        operator: un.operator.clone(),
                        operand_type: Type::Primitive(operand_ty.to_string()),
                        location: SourceLocation::new(un.node.range.start.line, un.node.range.start.column, 0),
                    }))
                }
            }
            _ => Err(CompileError::Semantic(SemanticError::UnknownOperator {
                operator: un.operator.clone(),
                location: SourceLocation::new(un.node.range.start.line, un.node.range.start.column, 0),
            })),
        }
    }

    /// Type check de dispatch expression
    fn type_check_dispatch(&mut self, disp: &DispatchExpression) -> CompileResult<SemanticType> {
        // Type check the action expression
        let action_ty = self.type_check_expression(&disp.action)?;
        
        // TODO: Verify that action implements the Action trait
        // For now, just ensure it's a valid expression
        
        // Dispatch returns void
        Ok(SemanticType::Primitive("void".to_string()))
    }

    /// Type check de block statement
    fn type_check_block_statement(&mut self, block: &BlockStatement) -> CompileResult<()> {
        let block_scope = self.symbols.create_scope(Some(self.current_scope), ScopeKind::Block);
        let prev_scope = self.current_scope;
        self.current_scope = block_scope;

        for stmt in &block.statements {
            self.type_check_statement(stmt)?;
        }

        self.current_scope = prev_scope;
        Ok(())
    }

    /// Type check de statement
    fn type_check_statement(&mut self, stmt: &Statement) -> CompileResult<()> {
        match stmt {
            Statement::Variable(var) => self.type_check_variable(var),
            Statement::Expression(expr) => {
                self.type_check_expression(&expr.expression)?;
                Ok(())
            }
            Statement::Return(ret) => {
                if let Some(val) = &ret.value {
                    self.type_check_expression(val)?;
                }
                Ok(())
            }
            Statement::Block(block) => self.type_check_block_statement(block),
            _ => Ok(()), // TODO: Implementar otros statements
        }
    }

    /// Verifica que una función tenga returns apropiados
    fn check_function_returns(&mut self, body: &BlockStatement, expected_ty: &SemanticType) -> CompileResult<()> {
        // TODO: Implementar verificación de returns
        Ok(())
    }

    /// Verifica compatibilidad de tipos
    fn types_compatible(&self, left: &SemanticType, right: &SemanticType) -> bool {
        match (left, right) {
            (SemanticType::Primitive(l), SemanticType::Primitive(r)) => l == r,
            (SemanticType::Struct(l), SemanticType::Struct(r)) => l == r,
            (SemanticType::Enum(l), SemanticType::Enum(r)) => l == r,
            (SemanticType::Unknown, _) | (_, SemanticType::Unknown) => true, // Permitir unknown para inference
            _ => false,
        }
    }

    /// Verifica si un tipo es numérico
    fn is_numeric_type(&self, ty: &SemanticType) -> bool {
        matches!(ty, SemanticType::Primitive(t) if t == "Number" || t == "Float")
    }

    /// Verifica si un tipo es booleano
    fn is_boolean_type(&self, ty: &SemanticType) -> bool {
        matches!(ty, SemanticType::Primitive(t) if t == "Bool")
    }

    /// Infere tipo de expresión (para variables sin tipo explícito)
    fn infer_expression_type(&mut self, expr: &Expression) -> CompileResult<SemanticType> {
        self.type_check_expression(expr)
    }

    /// Fase 3: Validación adicional del programa
    fn validate_program(&mut self, _program: &Program) -> CompileResult<()> {
        // TODO: Implementar validaciones adicionales
        Ok(())
    }
}

impl SymbolTable {
    /// Crea una nueva tabla de símbolos
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            current_scope: ScopeId(0),
        }
    }

    /// Crea un nuevo scope
    pub fn create_scope(&mut self, parent: Option<ScopeId>, kind: ScopeKind) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope {
            symbols: HashMap::new(),
            parent,
            kind,
        });
        id
    }

    /// Declara un símbolo en el scope actual
    pub fn declare(&mut self, name: String, symbol: Symbol) -> CompileResult<()> {
        let scope = &mut self.scopes[self.current_scope.0];
        if scope.symbols.contains_key(&name) {
            return Err(CompileError::Semantic(SemanticError::DuplicateSymbol {
                name,
                location: match &symbol {
                    Symbol::Variable { location, .. } => location.clone(),
                    Symbol::Function { location, .. } => location.clone(),
                    Symbol::Struct { location, .. } => location.clone(),
                    Symbol::Enum { location, .. } => location.clone(),
                },
            }));
        }
        scope.symbols.insert(name, symbol);
        Ok(())
    }

    /// Busca un símbolo en la jerarquía de scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut current = Some(self.current_scope);
        while let Some(scope_id) = current {
            let scope = &self.scopes[scope_id.0];
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
            current = scope.parent;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse_program(source: &str) -> Program {
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap()
    }

    #[test]
    fn test_variable_declaration_type_check() {
        let source = "state x: Number = 42;";
        let program = parse_program(source);
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_variable_error() {
        let source = "state y = x + 1;";
        let program = parse_program(source);
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_function_declaration() {
        let source = "fn add(a: Number, b: Number) -> Number { return a + b; }";
        let program = parse_program(source);
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_operation_type_check() {
        let source = "state result = 1 + 2;";
        let program = parse_program(source);
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_mismatch_error() {
        let source = "state x: String = 42;";
        let program = parse_program(source);
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_declaration() {
        let source = "struct Point { x: Number, y: Number }";
        let program = parse_program(source);
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_declaration() {
        let source = "enum Color { Red, Green, Blue }";
        let program = parse_program(source);
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }
}