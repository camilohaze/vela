/*
AST (Abstract Syntax Tree) Nodes para el lenguaje Vela

Implementación de: TASK-RUST-102 (Migrar AST nodes básicos)
Historia: US-RUST-02 (Compiler Foundation)
Fecha: 2025-12-01

⚠️ IMPORTANTE: Migración del código Python src/parser/ast_nodes.py a Rust

Este módulo define la estructura completa de nodos del AST para el lenguaje Vela.
Cada nodo representa una construcción sintáctica del lenguaje.

Jerarquía de Nodos:
- ASTNode: Nodo base abstracto
  - Program: Nodo raíz del AST
  - Declaration: Declaraciones (funciones, structs, enums, etc.)
  - Statement: Statements (assignments, expressions, control flow)
  - Expression: Expresiones (literales, binarias, llamadas, etc.)
  - Pattern: Patterns para match expressions
  - Type: Anotaciones de tipos
*/

use std::fmt;
use serde::{Deserialize, Serialize};

// ===================================================================
// BASE NODE
// ===================================================================

/// Posición en el código fuente (línea, columna)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Rango de código fuente (inicio, fin)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.start, self.end)
    }
}

/// Nodo base del AST.
/// Todos los nodos del AST heredan de esta clase.
/// Contiene información de posición para error reporting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ASTNode {
    pub range: Range,
}

impl ASTNode {
    pub fn new(range: Range) -> Self {
        Self { range }
    }
}

// ===================================================================
// PROGRAM (Root Node)
// ===================================================================

/// Nodo raíz del AST.
/// Representa un programa completo de Vela.
/// Contiene imports y declaraciones de nivel superior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub node: ASTNode,
    pub imports: Vec<ImportDeclaration>,
    pub declarations: Vec<Declaration>,
}

impl Program {
    pub fn new(range: Range, imports: Vec<ImportDeclaration>, declarations: Vec<Declaration>) -> Self {
        Self {
            node: ASTNode::new(range),
            imports,
            declarations,
        }
    }
}

// ===================================================================
// IMPORTS
// ===================================================================

/// Tipos de imports en Vela
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    System,      // system:io
    Package,     // package:http
    Module,      // module:utils
    Library,     // library:math
    Extension,   // extension:my_ext
    Assets,      // assets:images/logo.png
}

/// Import statement.
/// Ejemplos en Vela:
/// - import 'package:http'
/// - import 'module:utils' show { sort, filter }
/// - import 'library:math' hide { deprecated_fn }
/// - import 'package:long_name' as ln
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportDeclaration {
    pub node: ASTNode,
    pub kind: ImportKind,
    pub path: String,
    pub alias: Option<String>,
    pub show: Option<Vec<String>>,  // Importar solo estos
    pub hide: Option<Vec<String>>,  // Importar todo excepto estos
}

impl ImportDeclaration {
    pub fn new(
        range: Range,
        kind: ImportKind,
        path: String,
        alias: Option<String>,
        show: Option<Vec<String>>,
        hide: Option<Vec<String>>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            kind,
            path,
            alias,
            show,
            hide,
        }
    }
}

// ===================================================================
// DECLARATIONS
// ===================================================================

/// Declaración de nivel superior
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Declaration {
    Function(FunctionDeclaration),
    Struct(StructDeclaration),
    Enum(EnumDeclaration),
    TypeAlias(TypeAliasDeclaration),
    Variable(VariableDeclaration),
    Interface(InterfaceDeclaration),
    Class(ClassDeclaration),
    Service(ServiceDeclaration),
    Repository(RepositoryDeclaration),
    Controller(ControllerDeclaration),
    UseCase(UseCaseDeclaration),
    Entity(EntityDeclaration),
    ValueObject(ValueObjectDeclaration),
    DTO(DTODeclaration),
    Widget(WidgetDeclaration),
    Component(ComponentDeclaration),
    Model(ModelDeclaration),
    Factory(FactoryDeclaration),
    Builder(BuilderDeclaration),
    Strategy(StrategyDeclaration),
    Observer(ObserverDeclaration),
    Singleton(SingletonDeclaration),
    Adapter(AdapterDeclaration),
    Decorator(DecoratorDeclaration),
    Guard(GuardDeclaration),
    Middleware(MiddlewareDeclaration),
    Interceptor(InterceptorDeclaration),
    Validator(ValidatorDeclaration),
    Store(StoreDeclaration),
    Provider(ProviderDeclaration),
    Actor(ActorDeclaration),
    Pipe(PipeDeclaration),
    Task(TaskDeclaration),
    Helper(HelperDeclaration),
    Mapper(MapperDeclaration),
    Serializer(SerializerDeclaration),
    Module(ModuleDeclaration),
}

/// Declaración de función.
/// Ejemplo en Vela:
/// ```vela
/// public fn add(a: Number, b: Number) -> Number {
///   return a + b
/// }
///
/// async fn fetchData() -> Result<String> {
///   data = await http.get("api.com")
///   return Ok(data)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub decorators: Vec<Decorator>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body: BlockStatement,
    pub is_async: bool,
    pub generic_params: Vec<GenericParameter>,
}

impl FunctionDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        decorators: Vec<Decorator>,
        parameters: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: BlockStatement,
        is_async: bool,
        generic_params: Vec<GenericParameter>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            decorators,
            parameters,
            return_type,
            body,
            is_async,
            generic_params,
        }
    }
}

/// Parámetro de función
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub default_value: Option<Expression>,
    pub range: Range,
}

impl Parameter {
    pub fn new(
        name: String,
        type_annotation: Option<TypeAnnotation>,
        default_value: Option<Expression>,
        range: Range,
    ) -> Self {
        Self {
            name,
            type_annotation,
            default_value,
            range,
        }
    }
}

/// Parámetro genérico (ej: T, E)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericParameter {
    pub name: String,
    pub constraints: Vec<TypeAnnotation>,
    pub range: Range,
}

impl GenericParameter {
    pub fn new(name: String, constraints: Vec<TypeAnnotation>, range: Range) -> Self {
        Self {
            name,
            constraints,
            range,
        }
    }
}

/// Declaración de struct.
/// Ejemplo en Vela:
/// ```vela
/// struct User {
///   id: Number
///   name: String
///   email: String
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub decorators: Vec<Decorator>,
    pub fields: Vec<StructField>,
    pub generic_params: Vec<GenericParameter>,
}

impl StructDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        decorators: Vec<Decorator>,
        fields: Vec<StructField>,
        generic_params: Vec<GenericParameter>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            decorators,
            fields,
            generic_params,
        }
    }
}

/// Campo de struct
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub type_annotation: TypeAnnotation,
    pub is_public: bool,
    pub range: Range,
}

impl StructField {
    pub fn new(name: String, type_annotation: TypeAnnotation, is_public: bool, range: Range) -> Self {
        Self {
            name,
            type_annotation,
            is_public,
            range,
        }
    }
}

/// Declaración de enum.
/// Ejemplo en Vela:
/// ```vela
/// enum Color {
///   Red
///   Green
///   Blue
///   Custom(r: Number, g: Number, b: Number)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub generic_params: Vec<GenericParameter>,
}

impl EnumDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        variants: Vec<EnumVariant>,
        generic_params: Vec<GenericParameter>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            variants,
            generic_params,
        }
    }
}

/// Variante de enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<Vec<StructField>>,  // None para variantes sin datos
    pub range: Range,
}

impl EnumVariant {
    pub fn new(name: String, fields: Option<Vec<StructField>>, range: Range) -> Self {
        Self {
            name,
            fields,
            range,
        }
    }
}

/// Alias de tipo.
/// Ejemplo en Vela:
/// ```vela
/// type UserId = Number
/// type Status = "active" | "inactive"
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAliasDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

impl TypeAliasDeclaration {
    pub fn new(range: Range, is_public: bool, name: String, type_annotation: TypeAnnotation) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            type_annotation,
        }
    }
}

/// Declaración de interface.
/// Ejemplo en Vela:
/// ```vela
/// interface Drawable {
///   fn draw() -> void
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<FunctionSignature>,
    pub generic_params: Vec<GenericParameter>,
}

impl InterfaceDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<FunctionSignature>,
        generic_params: Vec<GenericParameter>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
            generic_params,
        }
    }
}

/// Firma de función (sin body)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub is_async: bool,
    pub range: Range,
}

impl FunctionSignature {
    pub fn new(
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        is_async: bool,
        range: Range,
    ) -> Self {
        Self {
            name,
            parameters,
            return_type,
            is_async,
            range,
        }
    }
}

/// Declaración de clase.
/// Ejemplo en Vela:
/// ```vela
/// class Dog extends Animal implements Petable {
///   name: String
///
///   constructor(name: String) {
///     this.name = name
///   }
///
///   fn bark() -> void {
///     print("Woof!")
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub decorators: Vec<Decorator>,
    pub constructor: Option<ConstructorDeclaration>,
    pub fields: Vec<ClassField>,
    pub methods: Vec<MethodDeclaration>,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub generic_params: Vec<GenericParameter>,
}

impl ClassDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        decorators: Vec<Decorator>,
        constructor: Option<ConstructorDeclaration>,
        fields: Vec<ClassField>,
        methods: Vec<MethodDeclaration>,
        extends: Option<String>,
        implements: Vec<String>,
        generic_params: Vec<GenericParameter>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            decorators,
            constructor,
            fields,
            methods,
            extends,
            implements,
            generic_params,
        }
    }
}

/// Campo de clase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassField {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub is_state: bool,  // Si es `state` (mutable y reactivo)
    pub initial_value: Option<Expression>,
    pub is_public: bool,
    pub is_protected: bool,
    pub is_private: bool,
    pub range: Range,
}

impl ClassField {
    pub fn new(
        name: String,
        type_annotation: Option<TypeAnnotation>,
        is_state: bool,
        initial_value: Option<Expression>,
        is_public: bool,
        is_protected: bool,
        is_private: bool,
        range: Range,
    ) -> Self {
        Self {
            name,
            type_annotation,
            is_state,
            initial_value,
            is_public,
            is_protected,
            is_private,
            range,
        }
    }
}

/// Constructor de clase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructorDeclaration {
    pub parameters: Vec<Parameter>,
    pub body: BlockStatement,
    pub range: Range,
}

impl ConstructorDeclaration {
    pub fn new(parameters: Vec<Parameter>, body: BlockStatement, range: Range) -> Self {
        Self {
            parameters,
            body,
            range,
        }
    }
}

/// Método de clase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body: BlockStatement,
    pub range: Range,
    pub is_async: bool,
    pub is_override: bool,
    pub is_public: bool,
    pub is_protected: bool,
    pub is_private: bool,
}

impl MethodDeclaration {
    pub fn new(
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: BlockStatement,
        range: Range,
        is_async: bool,
        is_override: bool,
        is_public: bool,
        is_protected: bool,
        is_private: bool,
    ) -> Self {
        Self {
            name,
            parameters,
            return_type,
            body,
            range,
            is_async,
            is_override,
            is_public,
            is_protected,
            is_private,
        }
    }
}

// Keywords Específicos de Dominio (DDD, Architecture Patterns, etc.)

/// service keyword - Capa de servicio (lógica de negocio)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub decorators: Vec<Decorator>,
    pub methods: Vec<MethodDeclaration>,
    pub dependencies: Vec<Parameter>,
}

impl ServiceDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        decorators: Vec<Decorator>,
        methods: Vec<MethodDeclaration>,
        dependencies: Vec<Parameter>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            decorators,
            methods,
            dependencies,
        }
    }
}

/// repository keyword - Capa de acceso a datos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub entity_type: String,
    pub methods: Vec<MethodDeclaration>,
}

impl RepositoryDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        entity_type: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            entity_type,
            methods,
        }
    }
}

/// controller keyword - Controlador HTTP
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub decorators: Vec<Decorator>,
    pub routes: Vec<RouteDeclaration>,
}

impl ControllerDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        decorators: Vec<Decorator>,
        routes: Vec<RouteDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            decorators,
            routes,
        }
    }
}

/// Ruta HTTP (decorada con @get, @post, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteDeclaration {
    pub method: String,  // "GET", "POST", "PUT", "DELETE", "PATCH"
    pub path: String,
    pub handler: FunctionDeclaration,
    pub range: Range,
}

impl RouteDeclaration {
    pub fn new(method: String, path: String, handler: FunctionDeclaration, range: Range) -> Self {
        Self {
            method,
            path,
            handler,
            range,
        }
    }
}

/// usecase keyword - Caso de uso / interactor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseCaseDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub execute_method: MethodDeclaration,
    pub dependencies: Vec<Parameter>,
}

impl UseCaseDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        execute_method: MethodDeclaration,
        dependencies: Vec<Parameter>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            execute_method,
            dependencies,
        }
    }
}

/// entity keyword - Entidad de dominio
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub id_field: StructField,
    pub fields: Vec<StructField>,
}

impl EntityDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        id_field: StructField,
        fields: Vec<StructField>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            id_field,
            fields,
        }
    }
}

/// valueObject keyword - Value Object inmutable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueObjectDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<StructField>,
}

impl ValueObjectDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        fields: Vec<StructField>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            fields,
        }
    }
}

/// dto keyword - Data Transfer Object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DTODeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<StructField>,
}

impl DTODeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        fields: Vec<StructField>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            fields,
        }
    }
}

// ===================================================================
// UI KEYWORDS
// ===================================================================

/// widget keyword - Stateful widget (UI component)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<ClassField>,
    pub methods: Vec<MethodDeclaration>,
}

impl WidgetDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        fields: Vec<ClassField>,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            fields,
            methods,
        }
    }
}

/// component keyword - UI component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<ClassField>,
    pub methods: Vec<MethodDeclaration>,
}

impl ComponentDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        fields: Vec<ClassField>,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            fields,
            methods,
        }
    }
}

// ===================================================================
// MODEL KEYWORDS
// ===================================================================

/// model keyword - Generic model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<StructField>,
}

impl ModelDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        fields: Vec<StructField>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            fields,
        }
    }
}

// ===================================================================
// DESIGN PATTERN KEYWORDS
// ===================================================================

/// factory keyword - Factory pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactoryDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl FactoryDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// builder keyword - Builder pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuilderDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl BuilderDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// strategy keyword - Strategy pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl StrategyDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// observer keyword - Observer pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObserverDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl ObserverDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// singleton keyword - Singleton pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingletonDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<ClassField>,
    pub methods: Vec<MethodDeclaration>,
}

impl SingletonDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        fields: Vec<ClassField>,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            fields,
            methods,
        }
    }
}

/// adapter keyword - Adapter pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl AdapterDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// decorator keyword - Decorator pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecoratorDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl DecoratorDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

// ===================================================================
// WEB/API KEYWORDS
// ===================================================================

/// guard keyword - Route guard/authorization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl GuardDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// middleware keyword - HTTP middleware
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MiddlewareDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl MiddlewareDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// interceptor keyword - Request/response interceptor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterceptorDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl InterceptorDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// validator keyword - Input validator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatorDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl ValidatorDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

// ===================================================================
// STATE & DI KEYWORDS
// ===================================================================

/// store keyword - State store
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<ClassField>,
    pub methods: Vec<MethodDeclaration>,
}

impl StoreDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        fields: Vec<ClassField>,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            fields,
            methods,
        }
    }
}

/// provider keyword - Dependency provider
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl ProviderDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

// ===================================================================
// CONCURRENCY KEYWORDS
// ===================================================================

/// actor keyword - Actor model concurrency
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActorDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<ClassField>,
    pub methods: Vec<MethodDeclaration>,
}

impl ActorDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        fields: Vec<ClassField>,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            fields,
            methods,
        }
    }
}

// ===================================================================
// UTILITY KEYWORDS
// ===================================================================

/// pipe keyword - Transformation pipeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipeDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl PipeDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// task keyword - Asynchronous task/job
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl TaskDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// helper keyword - Helper utilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HelperDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl HelperDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// mapper keyword - Object mapper
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapperDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl MapperDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

/// serializer keyword - Data serializer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializerDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub methods: Vec<MethodDeclaration>,
}

impl SerializerDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        methods: Vec<MethodDeclaration>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            methods,
        }
    }
}

// ===================================================================
// MODULE SYSTEM (Angular-style)
// ===================================================================

/// Decorator/Annotation.
/// Ejemplos en Vela:
/// ```vela
/// @injectable
/// @controller("/api/users")
/// @get("/profile")
/// @validate
/// @module({
///   declarations: [UserService, LoginWidget, RegisterWidget],
///   exports: [UserService],
///   providers: [UserService, TokenService],
///   imports: [HttpModule, CryptoModule]
/// })
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decorator {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub range: Range,
}

impl Decorator {
    pub fn new(name: String, arguments: Vec<Expression>, range: Range) -> Self {
        Self {
            name,
            arguments,
            range,
        }
    }
}

/// module keyword - Angular-style module (NO instanciable).
/// Ejemplo en Vela:
/// ```vela
/// @module({
///   declarations: [AuthService, LoginWidget, RegisterWidget],
///   exports: [AuthService],
///   providers: [AuthService, TokenService],
///   imports: [HttpModule, CryptoModule]
/// })
/// module AuthModule {
///   # Módulo NO instanciable (NO tiene constructor)
///   # NO se puede hacer: new AuthModule()
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleDeclaration {
    pub node: ASTNode,
    pub is_public: bool,
    pub name: String,
    pub decorators: Vec<Decorator>,
    pub body: Vec<Declaration>,
    // Metadata extraída del decorador @module (parsed desde object literal)
    pub declarations: Vec<Expression>,  // [Service1, Service2]
    pub exports: Vec<Expression>,       // [Service1]
    pub providers: Vec<Expression>,     // [Service1, DatabaseConnection]
    pub imports: Vec<Expression>,       // ['system:http', 'module:auth']
}

impl ModuleDeclaration {
    pub fn new(
        range: Range,
        is_public: bool,
        name: String,
        decorators: Vec<Decorator>,
        body: Vec<Declaration>,
        declarations: Vec<Expression>,
        exports: Vec<Expression>,
        providers: Vec<Expression>,
        imports: Vec<Expression>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            is_public,
            name,
            decorators,
            body,
            declarations,
            exports,
            providers,
            imports,
        }
    }
}

// ===================================================================
// STATEMENTS
// ===================================================================

/// Statement base
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Block(BlockStatement),
    Expression(ExpressionStatement),
    Variable(VariableDeclaration),
    Assignment(AssignmentStatement),
    Return(ReturnStatement),
    If(IfStatement),
    Match(MatchStatement),
    Throw(ThrowStatement),
    Try(TryStatement),
    EventOn(EventOnStatement),
    EventEmit(EventEmitStatement),
    EventOff(EventOffStatement),
    Dispatch(DispatchStatement),
}

/// Bloque de statements.
/// Ejemplo en Vela:
/// ```vela
/// {
///   x = 5
///   y = 10
///   print(x + y)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockStatement {
    pub node: ASTNode,
    pub statements: Vec<Statement>,
}

impl BlockStatement {
    pub fn new(range: Range, statements: Vec<Statement>) -> Self {
        Self {
            node: ASTNode::new(range),
            statements,
        }
    }
}

/// Expression como statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionStatement {
    pub node: ASTNode,
    pub expression: Expression,
}

impl ExpressionStatement {
    pub fn new(range: Range, expression: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            expression,
        }
    }
}

/// Declaración de variable.
/// Ejemplos en Vela:
/// ```vela
/// # Inmutable por defecto (NO se usa let/const)
/// name: String = "Vela"
///
/// # Mutable y reactivo (ÚNICA forma de mutabilidad)
/// state count: Number = 0
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableDeclaration {
    pub node: ASTNode,
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub initializer: Option<Expression>,
    pub is_state: bool,  // Si es `state` (mutable y reactivo)
}

impl VariableDeclaration {
    pub fn new(
        range: Range,
        name: String,
        type_annotation: Option<TypeAnnotation>,
        initializer: Option<Expression>,
        is_state: bool,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            name,
            type_annotation,
            initializer,
            is_state,
        }
    }
}

/// Asignación a variable existente.
/// Solo válido para variables `state`.
/// Ejemplo en Vela:
/// ```vela
/// count = count + 1  # OK solo si count es `state`
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentStatement {
    pub node: ASTNode,
    pub target: Expression,  // Puede ser Identifier, MemberAccess, IndexAccess
    pub value: Expression,
}

impl AssignmentStatement {
    pub fn new(range: Range, target: Expression, value: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            target,
            value,
        }
    }
}

/// Return statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub node: ASTNode,
    pub value: Option<Expression>,
}

impl ReturnStatement {
    pub fn new(range: Range, value: Option<Expression>) -> Self {
        Self {
            node: ASTNode::new(range),
            value,
        }
    }
}

/// If statement.
/// Ejemplo en Vela:
/// ```vela
/// if age >= 18 {
///   print("adult")
/// } else {
///   print("minor")
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStatement {
    pub node: ASTNode,
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

impl IfStatement {
    pub fn new(
        range: Range,
        condition: Expression,
        then_branch: Statement,
        else_branch: Option<Statement>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
}

/// Match statement (pattern matching exhaustivo).
/// Ejemplo en Vela:
/// ```vela
/// match result {
///   Ok(value) => print("Success: ${value}")
///   Err(error) => print("Error: ${error}")
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchStatement {
    pub node: ASTNode,
    pub value: Expression,
    pub arms: Vec<MatchArm>,
}

impl MatchStatement {
    pub fn new(range: Range, value: Expression, arms: Vec<MatchArm>) -> Self {
        Self {
            node: ASTNode::new(range),
            value,
            arms,
        }
    }
}

/// Brazo de match expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,  // if condition
    pub body: Statement,
    pub range: Range,
}

impl MatchArm {
    pub fn new(pattern: Pattern, guard: Option<Expression>, body: Statement, range: Range) -> Self {
        Self {
            pattern,
            guard,
            body,
            range,
        }
    }
}

/// Throw exception
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThrowStatement {
    pub node: ASTNode,
    pub exception: Expression,
}

impl ThrowStatement {
    pub fn new(range: Range, exception: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            exception,
        }
    }
}

/// Try-catch-finally statement.
/// Ejemplo en Vela:
/// ```vela
/// try {
///   riskyOp()
/// } catch (e: MyError) {
///   handle(e)
/// } finally {
///   cleanup()
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TryStatement {
    pub node: ASTNode,
    pub try_block: BlockStatement,
    pub catch_clauses: Vec<CatchClause>,
    pub finally_block: Option<BlockStatement>,
}

impl TryStatement {
    pub fn new(
        range: Range,
        try_block: BlockStatement,
        catch_clauses: Vec<CatchClause>,
        finally_block: Option<BlockStatement>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            try_block,
            catch_clauses,
            finally_block,
        }
    }
}

/// Cláusula catch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchClause {
    pub exception_name: String,
    pub exception_type: Option<TypeAnnotation>,
    pub body: BlockStatement,
    pub range: Range,
}

impl CatchClause {
    pub fn new(
        exception_name: String,
        exception_type: Option<TypeAnnotation>,
        body: BlockStatement,
        range: Range,
    ) -> Self {
        Self {
            exception_name,
            exception_type,
            body,
            range,
        }
    }
}

// ===================================================================
// EVENT SYSTEM (TASK-035M)
// ===================================================================

/// Event listener registration: on(event_type, handler)
/// Sintaxis en Vela:
/// ```vela
/// # Callback inline
/// on("user.created", (event) => {
///   print("User created: ${event.payload.name}")
/// })
///
/// # Callback con nombre
/// on("user.deleted", handleUserDeleted)
///
/// # Con tipo de evento
/// on<UserEvent>("user.updated", handleUserUpdated)
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventOnStatement {
    pub node: ASTNode,
    pub event_type: Expression,  // Nombre del evento (String literal o Expression)
    pub handler: Expression,     // Callback function (Lambda o Identifier)
    pub type_param: Option<TypeAnnotation>,  // on<T>
}

impl EventOnStatement {
    pub fn new(
        range: Range,
        event_type: Expression,
        handler: Expression,
        type_param: Option<TypeAnnotation>,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            event_type,
            handler,
            type_param,
        }
    }
}

/// Event emission: emit(event_type, payload)
/// Sintaxis en Vela:
/// ```vela
/// # Emit simple
/// emit("user.created", user)
///
/// # Emit con datos inline
/// emit("notification", {
///   message: "Hello",
///   level: "info"
/// })
///
/// # Emit sin datos
/// emit("app.started")
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEmitStatement {
    pub node: ASTNode,
    pub event_type: Expression,  // Nombre del evento
    pub payload: Option<Expression>,  // Datos del evento (opcional)
}

impl EventEmitStatement {
    pub fn new(range: Range, event_type: Expression, payload: Option<Expression>) -> Self {
        Self {
            node: ASTNode::new(range),
            event_type,
            payload,
        }
    }
}

/// Event listener removal: off(event_type, handler)
/// Sintaxis en Vela:
/// ```vela
/// # Remover listener específico
/// off("user.created", handleUserCreated)
///
/// # Remover todos los listeners de un evento
/// off("user.created")
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventOffStatement {
    pub node: ASTNode,
    pub event_type: Expression,  // Nombre del evento
    pub handler: Option<Expression>,  // Handler a remover (opcional)
}

impl EventOffStatement {
    pub fn new(range: Range, event_type: Expression, handler: Option<Expression>) -> Self {
        Self {
            node: ASTNode::new(range),
            event_type,
            handler,
        }
    }
}

// ===================================================================
// STATE MANAGEMENT - DISPATCH (TASK-035U)
// ===================================================================

/// Dispatch action to store: dispatch(action)
/// Sintaxis en Vela:
/// ```vela
/// # Dispatch simple action
/// dispatch(INCREMENT)
///
/// # Dispatch con payload
/// dispatch(AddTodo({ title: "Buy milk", completed: false }))
///
/// # Dispatch con action creator
/// dispatch(todoActions.add("Buy milk"))
///
/// # Dispatch async action
/// dispatch(await fetchUser(userId))
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchStatement {
    pub node: ASTNode,
    pub action: Expression,  // Expression que evalúa a un Action
}

impl DispatchStatement {
    pub fn new(range: Range, action: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            action,
        }
    }
}

// ===================================================================
// EXPRESSIONS
// ===================================================================

/// Expression base
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(Identifier),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Call(CallExpression),
    MemberAccess(MemberAccessExpression),
    IndexAccess(IndexAccessExpression),
    ArrayLiteral(ArrayLiteral),
    TupleLiteral(TupleLiteral),
    StructLiteral(StructLiteral),
    Lambda(LambdaExpression),
    If(IfExpression),
    Match(MatchExpression),
    StringInterpolation(StringInterpolation),
    Await(AwaitExpression),
    Computed(ComputedExpression),
    Dispatch(DispatchExpression),
}

/// Valor literal.
/// Ejemplos en Vela:
/// - Números: 42, 3.14
/// - Strings: "hello", 'world'
/// - Booleanos: true, false
/// - Option: None (NO null)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Literal {
    pub node: ASTNode,
    pub value: serde_json::Value,  // Usamos serde_json::Value para valores dinámicos
    pub kind: String,  // "number", "float", "string", "bool", "none"
}

impl Literal {
    pub fn new(range: Range, value: serde_json::Value, kind: String) -> Self {
        Self {
            node: ASTNode::new(range),
            value,
            kind,
        }
    }
}

/// Identificador (nombre de variable, función, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identifier {
    pub node: ASTNode,
    pub name: String,
}

impl Identifier {
    pub fn new(range: Range, name: String) -> Self {
        Self {
            node: ASTNode::new(range),
            name,
        }
    }
}

/// Expresión binaria.
/// Ejemplos en Vela:
/// - Aritméticas: a + b, x * y, base ** exp
/// - Comparación: x == y, a < b
/// - Lógicas: p && q, a || b
/// - None coalescing: value ?? default (NO null)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub node: ASTNode,
    pub left: Box<Expression>,
    pub operator: String,  // "+", "-", "*", "/", "**", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "??", etc.
    pub right: Box<Expression>,
}

impl BinaryExpression {
    pub fn new(range: Range, left: Expression, operator: String, right: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

/// Expresión unaria.
/// Ejemplos en Vela: -x, !flag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryExpression {
    pub node: ASTNode,
    pub operator: String,  // "-", "!"
    pub operand: Box<Expression>,
}

impl UnaryExpression {
    pub fn new(range: Range, operator: String, operand: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            operator,
            operand: Box::new(operand),
        }
    }
}

/// Llamada a función.
/// Ejemplo en Vela:
/// ```vela
/// add(2, 3)
/// user.getName()
/// [1, 2, 3].map(x => x * 2)
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpression {
    pub node: ASTNode,
    pub callee: Box<Expression>,  // Función a llamar
    pub arguments: Vec<Expression>,
}

impl CallExpression {
    pub fn new(range: Range, callee: Expression, arguments: Vec<Expression>) -> Self {
        Self {
            node: ASTNode::new(range),
            callee: Box::new(callee),
            arguments,
        }
    }
}

/// Acceso a miembro (dot notation).
/// Ejemplos en Vela:
/// - user.name
/// - object.method()
/// - Optional chaining: user?.address?.street
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberAccessExpression {
    pub node: ASTNode,
    pub object: Box<Expression>,
    pub member: String,
    pub is_optional: bool,  // Si usa ?. (optional chaining)
}

impl MemberAccessExpression {
    pub fn new(range: Range, object: Expression, member: String, is_optional: bool) -> Self {
        Self {
            node: ASTNode::new(range),
            object: Box::new(object),
            member,
            is_optional,
        }
    }
}

/// Acceso por índice.
/// Ejemplos en Vela:
/// - array[0]
/// - map["key"]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexAccessExpression {
    pub node: ASTNode,
    pub object: Box<Expression>,
    pub index: Box<Expression>,
}

impl IndexAccessExpression {
    pub fn new(range: Range, object: Expression, index: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            object: Box::new(object),
            index: Box::new(index),
        }
    }
}

/// Array literal.
/// Ejemplo en Vela: [1, 2, 3, 4]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayLiteral {
    pub node: ASTNode,
    pub elements: Vec<Expression>,
}

impl ArrayLiteral {
    pub fn new(range: Range, elements: Vec<Expression>) -> Self {
        Self {
            node: ASTNode::new(range),
            elements,
        }
    }
}

/// Tuple literal.
/// Ejemplo en Vela: (1, "hello", true)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleLiteral {
    pub node: ASTNode,
    pub elements: Vec<Expression>,
}

impl TupleLiteral {
    pub fn new(range: Range, elements: Vec<Expression>) -> Self {
        Self {
            node: ASTNode::new(range),
            elements,
        }
    }
}

/// Struct literal.
/// Ejemplo en Vela:
/// ```vela
/// User { id: 1, name: "Alice", email: "alice@example.com" }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructLiteral {
    pub node: ASTNode,
    pub struct_name: String,
    pub fields: Vec<StructLiteralField>,
}

impl StructLiteral {
    pub fn new(range: Range, struct_name: String, fields: Vec<StructLiteralField>) -> Self {
        Self {
            node: ASTNode::new(range),
            struct_name,
            fields,
        }
    }
}

/// Campo de struct literal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructLiteralField {
    pub name: String,
    pub value: Expression,
    pub range: Range,
}

impl StructLiteralField {
    pub fn new(name: String, value: Expression, range: Range) -> Self {
        Self {
            name,
            value,
            range,
        }
    }
}

/// Función anónima (arrow function).
/// Ejemplos en Vela:
/// ```vela
/// (x) => x * 2
/// (a, b) => a + b
/// (data) => {
///   cleaned = data.trim()
///   return cleaned.toUpperCase()
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LambdaExpression {
    pub node: ASTNode,
    pub parameters: Vec<Parameter>,
    pub body: LambdaBody,
}

impl LambdaExpression {
    pub fn new(range: Range, parameters: Vec<Parameter>, body: LambdaBody) -> Self {
        Self {
            node: ASTNode::new(range),
            parameters,
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LambdaBody {
    Expression(Box<Expression>),
    Block(BlockStatement),
}

/// If como expresión (retorna valor).
/// Ejemplo en Vela:
/// ```vela
/// status = if age >= 18 { "adult" } else { "minor" }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfExpression {
    pub node: ASTNode,
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
    pub else_branch: Box<Expression>,
}

impl IfExpression {
    pub fn new(
        range: Range,
        condition: Expression,
        then_branch: Expression,
        else_branch: Expression,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }
    }
}

/// Match como expresión (retorna valor).
/// Ejemplo en Vela:
/// ```vela
/// message = match result {
///   Ok(val) => "Success: ${val}"
///   Err(e) => "Error: ${e}"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchExpression {
    pub node: ASTNode,
    pub value: Box<Expression>,
    pub arms: Vec<MatchExpressionArm>,
}

impl MatchExpression {
    pub fn new(range: Range, value: Expression, arms: Vec<MatchExpressionArm>) -> Self {
        Self {
            node: ASTNode::new(range),
            value: Box::new(value),
            arms,
        }
    }
}

/// Brazo de match expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchExpressionArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,  // if condition
    pub body: Expression,
    pub range: Range,
}

impl MatchExpressionArm {
    pub fn new(
        pattern: Pattern,
        guard: Option<Expression>,
        body: Expression,
        range: Range,
    ) -> Self {
        Self {
            pattern,
            guard,
            body,
            range,
        }
    }
}

/// String interpolation con ${}.
/// Ejemplo en Vela:
/// ```vela
/// "Hello, ${name}!"
/// "Result: ${fn() -> Number { calculate(x, y) }}"
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringInterpolation {
    pub node: ASTNode,
    pub parts: Vec<StringInterpolationPart>,
}

impl StringInterpolation {
    pub fn new(range: Range, parts: Vec<StringInterpolationPart>) -> Self {
        Self {
            node: ASTNode::new(range),
            parts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringInterpolationPart {
    String(String),
    Expression(Box<Expression>),
}

/// Await expression.
/// Ejemplo en Vela:
/// ```vela
/// data = await fetchData()
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwaitExpression {
    pub node: ASTNode,
    pub expression: Box<Expression>,
}

impl AwaitExpression {
    pub fn new(range: Range, expression: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            expression: Box::new(expression),
        }
    }
}

/// Computed value (reactivo, derivado).
/// Ejemplo en Vela:
/// ```vela
/// computed doubled: Number {
///   return this.count * 2
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputedExpression {
    pub node: ASTNode,
    pub body: BlockStatement,
}

impl ComputedExpression {
    pub fn new(range: Range, body: BlockStatement) -> Self {
        Self {
            node: ASTNode::new(range),
            body,
        }
    }
}

/// Expresión dispatch para enviar acciones al store global.
/// Ejemplo en Vela: dispatch(IncrementCounter())
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchExpression {
    pub node: ASTNode,
    pub action: Box<Expression>,  // La expresión que representa la acción
}

impl DispatchExpression {
    pub fn new(range: Range, action: Expression) -> Self {
        Self {
            node: ASTNode::new(range),
            action: Box::new(action),
        }
    }
}

// ===================================================================
// PATTERNS (for match expressions)
// ===================================================================

/// Pattern base para match
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Literal(LiteralPattern),
    Identifier(IdentifierPattern),
    Tuple(TuplePattern),
    Struct(StructPattern),
    Enum(EnumPattern),
    Or(OrPattern),
    Range(RangePattern),
    Wildcard(WildcardPattern),
}

/// Pattern literal: 1, "hello", true
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteralPattern {
    pub node: ASTNode,
    pub value: serde_json::Value,
}

impl LiteralPattern {
    pub fn new(range: Range, value: serde_json::Value) -> Self {
        Self {
            node: ASTNode::new(range),
            value,
        }
    }
}

/// Pattern identifier: bind a variable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentifierPattern {
    pub node: ASTNode,
    pub name: String,
}

impl IdentifierPattern {
    pub fn new(range: Range, name: String) -> Self {
        Self {
            node: ASTNode::new(range),
            name,
        }
    }
}

/// Pattern tuple: (x, y, z)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TuplePattern {
    pub node: ASTNode,
    pub elements: Vec<Pattern>,
}

impl TuplePattern {
    pub fn new(range: Range, elements: Vec<Pattern>) -> Self {
        Self {
            node: ASTNode::new(range),
            elements,
        }
    }
}

/// Pattern struct.
/// Ejemplo: User { id, name }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructPattern {
    pub node: ASTNode,
    pub struct_name: String,
    pub fields: Vec<StructPatternField>,
}

impl StructPattern {
    pub fn new(range: Range, struct_name: String, fields: Vec<StructPatternField>) -> Self {
        Self {
            node: ASTNode::new(range),
            struct_name,
            fields,
        }
    }
}

/// Campo de struct pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructPatternField {
    pub name: String,
    pub pattern: Pattern,
    pub range: Range,
}

impl StructPatternField {
    pub fn new(name: String, pattern: Pattern, range: Range) -> Self {
        Self {
            name,
            pattern,
            range,
        }
    }
}

/// Pattern enum variant.
/// Ejemplo: Ok(value), Err(error)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumPattern {
    pub node: ASTNode,
    pub variant_name: String,
    pub inner_patterns: Option<Vec<Pattern>>,
}

impl EnumPattern {
    pub fn new(range: Range, variant_name: String, inner_patterns: Option<Vec<Pattern>>) -> Self {
        Self {
            node: ASTNode::new(range),
            variant_name,
            inner_patterns,
        }
    }
}

/// Pattern or: pattern1 | pattern2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrPattern {
    pub node: ASTNode,
    pub patterns: Vec<Pattern>,
}

impl OrPattern {
    pub fn new(range: Range, patterns: Vec<Pattern>) -> Self {
        Self {
            node: ASTNode::new(range),
            patterns,
        }
    }
}

/// Pattern range: 1..10, 'a'..='z'
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangePattern {
    pub node: ASTNode,
    pub start: Box<Expression>,
    pub end: Box<Expression>,
    pub is_inclusive: bool,  // True para ..=, False para ..
}

impl RangePattern {
    pub fn new(range: Range, start: Expression, end: Expression, is_inclusive: bool) -> Self {
        Self {
            node: ASTNode::new(range),
            start: Box::new(start),
            end: Box::new(end),
            is_inclusive,
        }
    }
}

/// Pattern wildcard: _ (catch-all)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WildcardPattern {
    pub node: ASTNode,
}

impl WildcardPattern {
    pub fn new(range: Range) -> Self {
        Self {
            node: ASTNode::new(range),
        }
    }
}

// ===================================================================
// TYPE ANNOTATIONS
// ===================================================================

/// Type annotation base
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotation {
    Primitive(PrimitiveType),
    Array(ArrayType),
    Tuple(TupleType),
    Function(FunctionType),
    Generic(GenericType),
    Union(UnionType),
    Named(NamedType),
    Optional(OptionalType),
}

/// Tipo primitivo.
/// En Vela: Number, Float, String, Bool, void, never
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimitiveType {
    pub node: ASTNode,
    pub name: String,  // "Number", "Float", "String", "Bool", "void", "never"
}

impl PrimitiveType {
    pub fn new(range: Range, name: String) -> Self {
        Self {
            node: ASTNode::new(range),
            name,
        }
    }
}

/// Tipo array: List<T>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayType {
    pub node: ASTNode,
    pub element_type: Box<TypeAnnotation>,
}

impl ArrayType {
    pub fn new(range: Range, element_type: TypeAnnotation) -> Self {
        Self {
            node: ASTNode::new(range),
            element_type: Box::new(element_type),
        }
    }
}

/// Tipo tuple: (Number, String, Bool)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleType {
    pub node: ASTNode,
    pub element_types: Vec<TypeAnnotation>,
}

impl TupleType {
    pub fn new(range: Range, element_types: Vec<TypeAnnotation>) -> Self {
        Self {
            node: ASTNode::new(range),
            element_types,
        }
    }
}

/// Tipo función.
/// Ejemplo: (Number, Number) -> Number
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionType {
    pub node: ASTNode,
    pub parameter_types: Vec<TypeAnnotation>,
    pub return_type: Box<TypeAnnotation>,
}

impl FunctionType {
    pub fn new(
        range: Range,
        parameter_types: Vec<TypeAnnotation>,
        return_type: TypeAnnotation,
    ) -> Self {
        Self {
            node: ASTNode::new(range),
            parameter_types,
            return_type: Box::new(return_type),
        }
    }
}

/// Tipo genérico.
/// Ejemplos: Option<T>, Result<T, E>, List<String>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericType {
    pub node: ASTNode,
    pub base_name: String,
    pub type_arguments: Vec<TypeAnnotation>,
}

impl GenericType {
    pub fn new(range: Range, base_name: String, type_arguments: Vec<TypeAnnotation>) -> Self {
        Self {
            node: ASTNode::new(range),
            base_name,
            type_arguments,
        }
    }
}

/// Union type.
/// Ejemplo: "active" | "inactive" | "pending"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnionType {
    pub node: ASTNode,
    pub types: Vec<TypeAnnotation>,
}

impl UnionType {
    pub fn new(range: Range, types: Vec<TypeAnnotation>) -> Self {
        Self {
            node: ASTNode::new(range),
            types,
        }
    }
}

/// Tipo nombrado: User, Config, etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedType {
    pub node: ASTNode,
    pub name: String,
}

impl NamedType {
    pub fn new(range: Range, name: String) -> Self {
        Self {
            node: ASTNode::new(range),
            name,
        }
    }
}

/// Tipo opcional (Option<T>).
/// En Vela: Option<T> (NO T?)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionalType {
    pub node: ASTNode,
    pub inner_type: Box<TypeAnnotation>,
}

impl OptionalType {
    pub fn new(range: Range, inner_type: TypeAnnotation) -> Self {
        Self {
            node: ASTNode::new(range),
            inner_type: Box::new(inner_type),
        }
    }
}

// ===================================================================
// AST VISITOR (for traversal)
// ===================================================================

/// Visitor pattern para traversal del AST.
/// Subclasear este visitor para implementar análisis o transformaciones del AST.
pub trait ASTVisitor<T> {
    fn visit_program(&mut self, node: &Program) -> T;
    fn visit_function_declaration(&mut self, node: &FunctionDeclaration) -> T;
    fn visit_block_statement(&mut self, node: &BlockStatement) -> T;
    fn visit_expression_statement(&mut self, node: &ExpressionStatement) -> T;
    fn visit_variable_declaration(&mut self, node: &VariableDeclaration) -> T;
    fn visit_assignment_statement(&mut self, node: &AssignmentStatement) -> T;
    fn visit_return_statement(&mut self, node: &ReturnStatement) -> T;
    fn visit_if_statement(&mut self, node: &IfStatement) -> T;
    fn visit_match_statement(&mut self, node: &MatchStatement) -> T;
    fn visit_throw_statement(&mut self, node: &ThrowStatement) -> T;
    fn visit_try_statement(&mut self, node: &TryStatement) -> T;
    fn visit_event_on_statement(&mut self, node: &EventOnStatement) -> T;
    fn visit_event_emit_statement(&mut self, node: &EventEmitStatement) -> T;
    fn visit_event_off_statement(&mut self, node: &EventOffStatement) -> T;
    fn visit_dispatch_statement(&mut self, node: &DispatchStatement) -> T;

    fn visit_literal(&mut self, node: &Literal) -> T;
    fn visit_identifier(&mut self, node: &Identifier) -> T;
    fn visit_binary_expression(&mut self, node: &BinaryExpression) -> T;
    fn visit_unary_expression(&mut self, node: &UnaryExpression) -> T;
    fn visit_call_expression(&mut self, node: &CallExpression) -> T;
    fn visit_member_access_expression(&mut self, node: &MemberAccessExpression) -> T;
    fn visit_index_access_expression(&mut self, node: &IndexAccessExpression) -> T;
    fn visit_array_literal(&mut self, node: &ArrayLiteral) -> T;
    fn visit_tuple_literal(&mut self, node: &TupleLiteral) -> T;
    fn visit_struct_literal(&mut self, node: &StructLiteral) -> T;
    fn visit_lambda_expression(&mut self, node: &LambdaExpression) -> T;
    fn visit_if_expression(&mut self, node: &IfExpression) -> T;
    fn visit_match_expression(&mut self, node: &MatchExpression) -> T;
    fn visit_string_interpolation(&mut self, node: &StringInterpolation) -> T;
    fn visit_await_expression(&mut self, node: &AwaitExpression) -> T;
    fn visit_computed_expression(&mut self, node: &ComputedExpression) -> T;

    fn visit_literal_pattern(&mut self, node: &LiteralPattern) -> T;
    fn visit_identifier_pattern(&mut self, node: &IdentifierPattern) -> T;
    fn visit_tuple_pattern(&mut self, node: &TuplePattern) -> T;
    fn visit_struct_pattern(&mut self, node: &StructPattern) -> T;
    fn visit_enum_pattern(&mut self, node: &EnumPattern) -> T;
    fn visit_or_pattern(&mut self, node: &OrPattern) -> T;
    fn visit_range_pattern(&mut self, node: &RangePattern) -> T;
    fn visit_wildcard_pattern(&mut self, node: &WildcardPattern) -> T;

    fn visit_primitive_type(&mut self, node: &PrimitiveType) -> T;
    fn visit_array_type(&mut self, node: &ArrayType) -> T;
    fn visit_tuple_type(&mut self, node: &TupleType) -> T;
    fn visit_function_type(&mut self, node: &FunctionType) -> T;
    fn visit_generic_type(&mut self, node: &GenericType) -> T;
    fn visit_union_type(&mut self, node: &UnionType) -> T;
    fn visit_named_type(&mut self, node: &NamedType) -> T;
    fn visit_optional_type(&mut self, node: &OptionalType) -> T;
}

// ===================================================================
// UTILITY FUNCTIONS
// ===================================================================

/// Helper para crear Position
pub fn create_position(line: usize, column: usize) -> Position {
    Position::new(line, column)
}

/// Helper para crear Range
pub fn create_range(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Range {
    Range::new(
        Position::new(start_line, start_col),
        Position::new(end_line, end_col),
    )
}

/// Verifica si una expresión es válida como statement.
/// Válidas: CallExpression, AssignmentExpression
/// Inválidas: BinaryExpression (a + b como statement no tiene sentido)
pub fn is_expression_statement_valid(expr: &Expression) -> bool {
    matches!(expr, Expression::Call(_) | Expression::Binary(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // ===================================================================
    // BASE NODE TESTS
    // ===================================================================

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(3, 7);
        assert_eq!(format!("{}", pos), "3:7");
    }

    #[test]
    fn test_range_creation() {
        let start = Position::new(1, 1);
        let end = Position::new(1, 10);
        let range = Range::new(start, end);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.column, 1);
        assert_eq!(range.end.line, 1);
        assert_eq!(range.end.column, 10);
    }

    #[test]
    fn test_range_display() {
        let start = Position::new(2, 5);
        let end = Position::new(2, 15);
        let range = Range::new(start, end);
        assert_eq!(format!("{}", range), "2:5 - 2:15");
    }

    #[test]
    fn test_ast_node_creation() {
        let range = create_range(1, 1, 1, 5);
        let node = ASTNode::new(range);
        assert_eq!(node.range.start.line, 1);
        assert_eq!(node.range.start.column, 1);
        assert_eq!(node.range.end.line, 1);
        assert_eq!(node.range.end.column, 5);
    }

    // ===================================================================
    // PROGRAM TESTS
    // ===================================================================

    #[test]
    fn test_program_creation() {
        let range = create_range(1, 1, 10, 1);
        let program = Program::new(range, vec![], vec![]);
        assert_eq!(program.imports.len(), 0);
        assert_eq!(program.declarations.len(), 0);
        assert_eq!(program.node.range.start.line, 1);
    }

    #[test]
    fn test_program_with_imports_and_declarations() {
        let range = create_range(1, 1, 20, 1);

        // Crear import
        let import_range = create_range(1, 1, 1, 25);
        let import = ImportDeclaration::new(
            import_range,
            ImportKind::Package,
            "http".to_string(),
            Some("IO".to_string()),
            None,
            None,
        );

        // Crear declaración de función
        let func_range = create_range(3, 1, 5, 2);
        let body = BlockStatement::new(create_range(5, 1, 5, 2), vec![]);
        let func = FunctionDeclaration::new(
            func_range,
            true,
            "main".to_string(),
            vec![],
            vec![],
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(3, 15, 3, 19),
                "void".to_string(),
            ))),
            body,
            false,
            vec![],
        );

        let program = Program::new(
            range,
            vec![import],
            vec![Declaration::Function(func)],
        );

        assert_eq!(program.imports.len(), 1);
        assert_eq!(program.declarations.len(), 1);
    }

    // ===================================================================
    // IMPORT TESTS
    // ===================================================================

    #[test]
    fn test_import_declaration_creation() {
        let range = create_range(1, 1, 1, 30);
        let import = ImportDeclaration::new(
            range,
            ImportKind::System,
            "io".to_string(),
            Some("IO".to_string()),
            None,
            None,
        );

        assert_eq!(import.kind, ImportKind::System);
        assert_eq!(import.path, "io");
        assert_eq!(import.alias, Some("IO".to_string()));
        assert!(import.show.is_none());
        assert!(import.hide.is_none());
    }

    #[test]
    fn test_import_with_show_and_hide() {
        let range = create_range(1, 1, 1, 50);
        let import = ImportDeclaration::new(
            range,
            ImportKind::Library,
            "utils".to_string(),
            None,
            Some(vec!["sort".to_string(), "filter".to_string()]),
            Some(vec!["deprecated".to_string()]),
        );

        assert_eq!(import.kind, ImportKind::Library);
        assert_eq!(import.show.as_ref().unwrap().len(), 2);
        assert_eq!(import.hide.as_ref().unwrap().len(), 1);
    }

    // ===================================================================
    // DECLARATION TESTS
    // ===================================================================

    #[test]
    fn test_function_declaration_creation() {
        let range = create_range(1, 1, 5, 2);
        let body = BlockStatement::new(create_range(4, 1, 5, 2), vec![]);

        let param_range = create_range(1, 10, 1, 15);
        let param = Parameter::new(
            "x".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                param_range.clone(),
                "Number".to_string(),
            ))),
            None,
            param_range.clone(),
        );

        let func = FunctionDeclaration::new(
            range,
            true,
            "add".to_string(),
            vec![],
            vec![param],
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 20, 1, 26),
                "Number".to_string(),
            ))),
            body,
            false,
            vec![],
        );

        assert_eq!(func.name, "add");
        assert!(func.is_public);
        assert!(!func.is_async);
        assert_eq!(func.parameters.len(), 1);
        assert!(func.return_type.is_some());
        assert!(func.generic_params.is_empty());
    }

    #[test]
    fn test_async_function_declaration() {
        let range = create_range(1, 1, 5, 2);
        let body = BlockStatement::new(create_range(4, 1, 5, 2), vec![]);

        let func = FunctionDeclaration::new(
            range,
            false,
            "fetchData".to_string(),
            vec![],
            vec![],
            Some(TypeAnnotation::Generic(GenericType::new(
                create_range(1, 20, 1, 35),
                "Result".to_string(),
                vec![TypeAnnotation::Primitive(PrimitiveType::new(
                    create_range(1, 27, 1, 33),
                    "String".to_string(),
                ))],
            ))),
            body,
            true, // async
            vec![],
        );

        assert_eq!(func.name, "fetchData");
        assert!(!func.is_public);
        assert!(func.is_async);
    }

    #[test]
    fn test_struct_declaration_creation() {
        let range = create_range(1, 1, 5, 2);

        let field_range = create_range(2, 5, 2, 15);
        let field = StructField::new(
            "name".to_string(),
            TypeAnnotation::Primitive(PrimitiveType::new(
                field_range.clone(),
                "String".to_string(),
            )),
            true,
            field_range.clone(),
        );

        let struct_decl = StructDeclaration::new(
            range,
            true,
            "User".to_string(),
            vec![],
            vec![field],
            vec![],
        );

        assert_eq!(struct_decl.name, "User");
        assert!(struct_decl.is_public);
        assert_eq!(struct_decl.fields.len(), 1);
        assert!(struct_decl.generic_params.is_empty());
    }

    #[test]
    fn test_enum_declaration_creation() {
        let range = create_range(1, 1, 8, 2);

        let variant1 = EnumVariant::new("Red".to_string(), None, create_range(2, 5, 2, 8));
        let variant2 = EnumVariant::new(
            "Custom".to_string(),
            Some(vec![
                StructField::new(
                    "r".to_string(),
                    TypeAnnotation::Primitive(PrimitiveType::new(
                        create_range(4, 10, 4, 16),
                        "Number".to_string(),
                    )),
                    false,
                    create_range(4, 10, 4, 16),
                ),
                StructField::new(
                    "g".to_string(),
                    TypeAnnotation::Primitive(PrimitiveType::new(
                        create_range(4, 18, 4, 24),
                        "Number".to_string(),
                    )),
                    false,
                    create_range(4, 18, 4, 24),
                ),
            ]),
            create_range(4, 5, 4, 25),
        );

        let enum_decl = EnumDeclaration::new(
            range,
            true,
            "Color".to_string(),
            vec![variant1, variant2],
            vec![],
        );

        assert_eq!(enum_decl.name, "Color");
        assert!(enum_decl.is_public);
        assert_eq!(enum_decl.variants.len(), 2);
        assert!(enum_decl.variants[0].fields.is_none());
        assert!(enum_decl.variants[1].fields.is_some());
    }

    #[test]
    fn test_type_alias_declaration() {
        let range = create_range(1, 1, 1, 20);
        let type_alias = TypeAliasDeclaration::new(
            range,
            true,
            "UserId".to_string(),
            TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 15, 1, 21),
                "Number".to_string(),
            )),
        );

        assert_eq!(type_alias.name, "UserId");
        assert!(type_alias.is_public);
    }

    // ===================================================================
    // STATEMENT TESTS
    // ===================================================================

    #[test]
    fn test_block_statement_creation() {
        let range = create_range(1, 1, 5, 2);
        let block = BlockStatement::new(range.clone(), vec![]);
        assert_eq!(block.statements.len(), 0);
        assert_eq!(block.node.range, range);
    }

    #[test]
    fn test_expression_statement_creation() {
        let range = create_range(1, 1, 1, 10);
        let expr = Expression::Literal(Literal::new(
            range.clone(),
            serde_json::json!(42),
            "number".to_string(),
        ));
        let stmt = ExpressionStatement::new(range.clone(), expr);

        assert_eq!(stmt.node.range, range);
        match &stmt.expression {
            Expression::Literal(lit) => assert_eq!(lit.value, serde_json::json!(42)),
            _ => panic!("Expected literal expression"),
        }
    }

    #[test]
    fn test_variable_declaration() {
        let range = create_range(1, 1, 1, 15);
        let initializer = Expression::Literal(Literal::new(
            create_range(1, 10, 1, 15),
            serde_json::json!("hello"),
            "string".to_string(),
        ));

        let var_decl = VariableDeclaration::new(
            range,
            "message".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 5, 1, 11),
                "String".to_string(),
            ))),
            Some(initializer),
            false, // not state
        );

        assert_eq!(var_decl.name, "message");
        assert!(!var_decl.is_state);
        assert!(var_decl.type_annotation.is_some());
        assert!(var_decl.initializer.is_some());
    }

    #[test]
    fn test_state_variable_declaration() {
        let range = create_range(1, 1, 1, 20);
        let state_var = VariableDeclaration::new(
            range,
            "count".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 10, 1, 16),
                "Number".to_string(),
            ))),
            Some(Expression::Literal(Literal::new(
                create_range(1, 19, 1, 20),
                serde_json::json!(0),
                "number".to_string(),
            ))),
            true, // is state
        );

        assert_eq!(state_var.name, "count");
        assert!(state_var.is_state);
    }

    #[test]
    fn test_assignment_statement() {
        let range = create_range(1, 1, 1, 10);
        let target = Expression::Identifier(Identifier::new(
            create_range(1, 1, 1, 5),
            "count".to_string(),
        ));
        let value = Expression::Literal(Literal::new(
            create_range(1, 9, 1, 10),
            serde_json::json!(5),
            "number".to_string(),
        ));

        let assignment = AssignmentStatement::new(range.clone(), target, value);
        assert_eq!(assignment.node.range, range);
    }

    #[test]
    fn test_return_statement() {
        let range = create_range(1, 1, 1, 15);
        let value = Expression::Literal(Literal::new(
            create_range(1, 8, 1, 15),
            serde_json::json!(42),
            "number".to_string(),
        ));

        let return_stmt = ReturnStatement::new(range, Some(value));
        assert!(return_stmt.value.is_some());
    }

    #[test]
    fn test_if_statement() {
        let range = create_range(1, 1, 5, 2);
        let condition = Expression::Identifier(Identifier::new(
            create_range(1, 5, 1, 9),
            "flag".to_string(),
        ));
        let then_branch = Statement::Block(BlockStatement::new(
            create_range(2, 1, 3, 2),
            vec![],
        ));

        let if_stmt = IfStatement::new(range, condition, then_branch, None);
        assert!(if_stmt.else_branch.is_none());
    }

    #[test]
    fn test_if_else_statement() {
        let range = create_range(1, 1, 7, 2);
        let condition = Expression::Binary(BinaryExpression::new(
            create_range(1, 5, 1, 12),
            Expression::Identifier(Identifier::new(create_range(1, 5, 1, 6), "x".to_string())),
            ">".to_string(),
            Expression::Literal(Literal::new(create_range(1, 10, 1, 12), serde_json::json!(0), "number".to_string())),
        ));
        let then_branch = Statement::Return(ReturnStatement::new(
            create_range(2, 5, 2, 12),
            Some(Expression::Literal(Literal::new(create_range(2, 12, 2, 12), serde_json::json!(1), "number".to_string()))),
        ));
        let else_branch = Statement::Return(ReturnStatement::new(
            create_range(4, 5, 4, 12),
            Some(Expression::Literal(Literal::new(create_range(4, 12, 4, 12), serde_json::json!(0), "number".to_string()))),
        ));

        let if_stmt = IfStatement::new(range, condition, then_branch, Some(else_branch));
        assert!(if_stmt.else_branch.is_some());
    }

    // ===================================================================
    // EXPRESSION TESTS
    // ===================================================================

    #[test]
    fn test_literal_expression() {
        let range = create_range(1, 1, 1, 4);
        let literal = Literal::new(range, serde_json::json!(42), "number".to_string());
        assert_eq!(literal.kind, "number");
        assert_eq!(literal.value, serde_json::json!(42));
    }

    #[test]
    fn test_none_literal() {
        let range = create_range(1, 1, 1, 4);
        let none_literal = Literal::new(range, serde_json::json!(null), "none".to_string());
        assert_eq!(none_literal.kind, "none");
        assert_eq!(none_literal.value, serde_json::json!(null));
    }

    #[test]
    fn test_identifier_expression() {
        let range = create_range(1, 1, 1, 5);
        let ident = Identifier::new(range.clone(), "variable".to_string());
        assert_eq!(ident.name, "variable");
        assert_eq!(ident.node.range, range);
    }

    #[test]
    fn test_binary_expression() {
        let range = create_range(1, 1, 1, 5);
        let left = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 1), "a".to_string()));
        let right = Expression::Identifier(Identifier::new(create_range(1, 5, 1, 5), "b".to_string()));
        let binary = BinaryExpression::new(range.clone(), left, "+".to_string(), right);

        assert_eq!(binary.operator, "+");
        assert_eq!(binary.node.range, range);
    }

    #[test]
    fn test_unary_expression() {
        let range = create_range(1, 1, 1, 3);
        let operand = Expression::Identifier(Identifier::new(create_range(1, 2, 1, 3), "x".to_string()));
        let unary = UnaryExpression::new(range, "-".to_string(), operand);

        assert_eq!(unary.operator, "-");
    }

    #[test]
    fn test_call_expression() {
        let range = create_range(1, 1, 1, 10);
        let callee = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 3), "add".to_string()));
        let args = vec![
            Expression::Literal(Literal::new(create_range(1, 5, 1, 5), serde_json::json!(2), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 8, 1, 8), serde_json::json!(3), "number".to_string())),
        ];

        let call = CallExpression::new(range, callee, args);
        assert_eq!(call.arguments.len(), 2);
    }

    #[test]
    fn test_member_access_expression() {
        let range = create_range(1, 1, 1, 8);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 4), "user".to_string()));
        let member_access = MemberAccessExpression::new(range, object, "name".to_string(), false);

        assert_eq!(member_access.member, "name");
        assert!(!member_access.is_optional);
    }

    #[test]
    fn test_optional_member_access() {
        let range = create_range(1, 1, 1, 12);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 4), "user".to_string()));
        let optional_access = MemberAccessExpression::new(range, object, "address".to_string(), true);

        assert_eq!(optional_access.member, "address");
        assert!(optional_access.is_optional);
    }

    #[test]
    fn test_index_access_expression() {
        let range = create_range(1, 1, 1, 8);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 5), "array".to_string()));
        let index = Expression::Literal(Literal::new(create_range(1, 7, 1, 7), serde_json::json!(0), "number".to_string()));
        let index_access = IndexAccessExpression::new(range.clone(), object, index);

        assert_eq!(index_access.node.range, range);
    }

    #[test]
    fn test_array_literal() {
        let range = create_range(1, 1, 1, 10);
        let elements = vec![
            Expression::Literal(Literal::new(create_range(1, 2, 1, 2), serde_json::json!(1), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 5, 1, 5), serde_json::json!(2), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 8, 1, 8), serde_json::json!(3), "number".to_string())),
        ];

        let array = ArrayLiteral::new(range, elements);
        assert_eq!(array.elements.len(), 3);
    }

    #[test]
    fn test_tuple_literal() {
        let range = create_range(1, 1, 1, 12);
        let elements = vec![
            Expression::Literal(Literal::new(create_range(1, 2, 1, 2), serde_json::json!(1), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 5, 1, 9), serde_json::json!("hello"), "string".to_string())),
        ];

        let tuple = TupleLiteral::new(range, elements);
        assert_eq!(tuple.elements.len(), 2);
    }

    #[test]
    fn test_struct_literal() {
        let range = create_range(1, 1, 1, 25);
        let fields = vec![
            StructLiteralField::new(
                "id".to_string(),
                Expression::Literal(Literal::new(create_range(1, 8, 1, 8), serde_json::json!(1), "number".to_string())),
                create_range(1, 5, 1, 9),
            ),
            StructLiteralField::new(
                "name".to_string(),
                Expression::Literal(Literal::new(create_range(1, 18, 1, 24), serde_json::json!("Alice"), "string".to_string())),
                create_range(1, 12, 1, 25),
            ),
        ];

        let struct_lit = StructLiteral::new(range, "User".to_string(), fields);
        assert_eq!(struct_lit.struct_name, "User");
        assert_eq!(struct_lit.fields.len(), 2);
    }

    #[test]
    fn test_lambda_expression() {
        let range = create_range(1, 1, 1, 15);
        let parameters = vec![
            Parameter::new(
                "x".to_string(),
                Some(TypeAnnotation::Primitive(PrimitiveType::new(
                    create_range(1, 2, 1, 8),
                    "Number".to_string(),
                ))),
                None,
                create_range(1, 2, 1, 8),
            ),
        ];
        let body = LambdaBody::Expression(Box::new(Expression::Binary(BinaryExpression::new(
            create_range(1, 12, 1, 15),
            Expression::Identifier(Identifier::new(create_range(1, 12, 1, 13), "x".to_string())),
            "*".to_string(),
            Expression::Literal(Literal::new(create_range(1, 15, 1, 15), serde_json::json!(2), "number".to_string())),
        ))));

        let lambda = LambdaExpression::new(range, parameters, body);
        assert_eq!(lambda.parameters.len(), 1);
    }

    #[test]
    fn test_if_expression() {
        let range = create_range(1, 1, 1, 25);
        let condition = Expression::Identifier(Identifier::new(create_range(1, 5, 1, 9), "flag".to_string()));
        let then_branch = Expression::Literal(Literal::new(create_range(1, 13, 1, 17), serde_json::json!("yes"), "string".to_string()));
        let else_branch = Expression::Literal(Literal::new(create_range(1, 21, 1, 25), serde_json::json!("no"), "string".to_string()));

        let if_expr = IfExpression::new(range.clone(), condition, then_branch, else_branch);
        assert_eq!(if_expr.node.range, range);
    }

    #[test]
    fn test_await_expression() {
        let range = create_range(1, 1, 1, 15);
        let inner_expr = Expression::Call(CallExpression::new(
            create_range(1, 7, 1, 15),
            Expression::Identifier(Identifier::new(create_range(1, 7, 1, 12), "fetch".to_string())),
            vec![],
        ));

        let await_expr = AwaitExpression::new(range.clone(), inner_expr);
        assert_eq!(await_expr.node.range, range);
    }

    // ===================================================================
    // PATTERN TESTS
    // ===================================================================

    #[test]
    fn test_literal_pattern() {
        let range = create_range(1, 1, 1, 3);
        let pattern = LiteralPattern::new(range, serde_json::json!(42));
        assert_eq!(pattern.value, serde_json::json!(42));
    }

    #[test]
    fn test_identifier_pattern() {
        let range = create_range(1, 1, 1, 5);
        let pattern = IdentifierPattern::new(range, "value".to_string());
        assert_eq!(pattern.name, "value");
    }

    #[test]
    fn test_tuple_pattern() {
        let range = create_range(1, 1, 1, 8);
        let elements = vec![
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 2, 1, 2), "x".to_string())),
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 5, 1, 5), "y".to_string())),
        ];

        let tuple_pattern = TuplePattern::new(range, elements);
        assert_eq!(tuple_pattern.elements.len(), 2);
    }

    #[test]
    fn test_struct_pattern() {
        let range = create_range(1, 1, 1, 15);
        let fields = vec![
            StructPatternField::new(
                "id".to_string(),
                Pattern::Identifier(IdentifierPattern::new(create_range(1, 8, 1, 8), "id".to_string())),
                create_range(1, 5, 1, 9),
            ),
            StructPatternField::new(
                "name".to_string(),
                Pattern::Identifier(IdentifierPattern::new(create_range(1, 14, 1, 14), "name".to_string())),
                create_range(1, 12, 1, 15),
            ),
        ];

        let struct_pattern = StructPattern::new(range, "User".to_string(), fields);
        assert_eq!(struct_pattern.struct_name, "User");
        assert_eq!(struct_pattern.fields.len(), 2);
    }

    #[test]
    fn test_enum_pattern() {
        let range = create_range(1, 1, 1, 10);
        let inner_patterns = Some(vec![
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 6, 1, 6), "value".to_string())),
        ]);

        let enum_pattern = EnumPattern::new(range, "Some".to_string(), inner_patterns);
        assert_eq!(enum_pattern.variant_name, "Some");
        assert!(enum_pattern.inner_patterns.is_some());
        assert_eq!(enum_pattern.inner_patterns.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_wildcard_pattern() {
        let range = create_range(1, 1, 1, 1);
        let wildcard = WildcardPattern::new(range.clone());
        assert_eq!(wildcard.node.range, range);
    }

    // ===================================================================
    // TYPE ANNOTATION TESTS
    // ===================================================================

    #[test]
    fn test_primitive_type() {
        let range = create_range(1, 1, 1, 6);
        let primitive = PrimitiveType::new(range, "Number".to_string());
        assert_eq!(primitive.name, "Number");
    }

    #[test]
    fn test_array_type() {
        let range = create_range(1, 1, 1, 10);
        let element_type = TypeAnnotation::Primitive(PrimitiveType::new(
            create_range(1, 6, 1, 10),
            "String".to_string(),
        ));
        let array_type = ArrayType::new(range.clone(), element_type);
        assert_eq!(array_type.node.range, range);
    }

    #[test]
    fn test_tuple_type() {
        let range = create_range(1, 1, 1, 15);
        let element_types = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 2, 1, 8), "Number".to_string())),
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 10, 1, 15), "String".to_string())),
        ];
        let tuple_type = TupleType::new(range, element_types);
        assert_eq!(tuple_type.element_types.len(), 2);
    }

    #[test]
    fn test_function_type() {
        let range = create_range(1, 1, 1, 20);
        let param_types = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 2, 1, 8), "Number".to_string())),
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 10, 1, 16), "Number".to_string())),
        ];
        let return_type = TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 18, 1, 24), "Number".to_string()));
        let func_type = FunctionType::new(range, param_types, return_type);
        assert_eq!(func_type.parameter_types.len(), 2);
    }

    #[test]
    fn test_generic_type() {
        let range = create_range(1, 1, 1, 15);
        let type_args = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 8, 1, 14), "String".to_string())),
        ];
        let generic_type = GenericType::new(range, "Option".to_string(), type_args);
        assert_eq!(generic_type.base_name, "Option");
        assert_eq!(generic_type.type_arguments.len(), 1);
    }

    #[test]
    fn test_union_type() {
        let range = create_range(1, 1, 1, 20);
        let types = vec![
            TypeAnnotation::Named(NamedType::new(create_range(1, 1, 1, 6), "active".to_string())),
            TypeAnnotation::Named(NamedType::new(create_range(1, 10, 1, 20), "inactive".to_string())),
        ];
        let union_type = UnionType::new(range, types);
        assert_eq!(union_type.types.len(), 2);
    }

    #[test]
    fn test_named_type() {
        let range = create_range(1, 1, 1, 4);
        let named_type = NamedType::new(range, "User".to_string());
        assert_eq!(named_type.name, "User");
    }

    #[test]
    fn test_optional_type() {
        let range = create_range(1, 1, 1, 8);
        let inner_type = TypeAnnotation::Primitive(PrimitiveType::new(
            create_range(1, 1, 1, 6),
            "String".to_string(),
        ));
        let optional_type = OptionalType::new(range.clone(), inner_type);
        assert_eq!(optional_type.node.range, range);
    }

    // ===================================================================
    // UTILITY FUNCTIONS TESTS
    // ===================================================================

    #[test]
    fn test_create_position_utility() {
        let pos = create_position(10, 20);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 20);
    }

    #[test]
    fn test_create_range_utility() {
        let range = create_range(1, 5, 2, 10);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.column, 5);
        assert_eq!(range.end.line, 2);
        assert_eq!(range.end.column, 10);
    }

    #[test]
    fn test_is_expression_statement_valid() {
        // Valid: Call expression
        let call_expr = Expression::Call(CallExpression::new(
            create_range(1, 1, 1, 8),
            Expression::Identifier(Identifier::new(create_range(1, 1, 1, 6), "print".to_string())),
            vec![],
        ));
        assert!(is_expression_statement_valid(&call_expr));

        // Valid: Binary expression (assignment-like)
        let binary_expr = Expression::Binary(BinaryExpression::new(
            create_range(1, 1, 1, 5),
            Expression::Identifier(Identifier::new(create_range(1, 1, 1, 1), "a".to_string())),
            "=".to_string(),
            Expression::Literal(Literal::new(create_range(1, 5, 1, 5), serde_json::json!(5), "number".to_string())),
        ));
        assert!(is_expression_statement_valid(&binary_expr));

        // Invalid: Just a literal
        let literal_expr = Expression::Literal(Literal::new(
            create_range(1, 1, 1, 3),
            serde_json::json!(42),
            "number".to_string(),
        ));
        assert!(!is_expression_statement_valid(&literal_expr));
    }

    // ===================================================================
    // EVENT SYSTEM TESTS
    // ===================================================================

    #[test]
    fn test_event_on_statement() {
        let range = create_range(1, 1, 1, 25);
        let event_type = Expression::Literal(Literal::new(
            create_range(1, 5, 1, 18),
            serde_json::json!("user.created"),
            "string".to_string(),
        ));
        let handler = Expression::Identifier(Identifier::new(
            create_range(1, 20, 1, 25),
            "callback".to_string(),
        ));

        let event_on = EventOnStatement::new(range, event_type, handler, None);
        assert!(event_on.type_param.is_none());
    }

    #[test]
    fn test_event_emit_statement() {
        let range = create_range(1, 1, 1, 20);
        let event_type = Expression::Literal(Literal::new(
            create_range(1, 7, 1, 20),
            serde_json::json!("user.updated"),
            "string".to_string(),
        ));
        let payload = Some(Expression::StructLiteral(StructLiteral::new(
            create_range(1, 22, 1, 35),
            "User".to_string(),
            vec![],
        )));

        let event_emit = EventEmitStatement::new(range, event_type, payload);
        assert!(event_emit.payload.is_some());
    }

    #[test]
    fn test_event_off_statement() {
        let range = create_range(1, 1, 1, 25);
        let event_type = Expression::Literal(Literal::new(
            create_range(1, 5, 1, 18),
            serde_json::json!("user.created"),
            "string".to_string(),
        ));
        let handler = Some(Expression::Identifier(Identifier::new(
            create_range(1, 20, 1, 25),
            "callback".to_string(),
        )));

        let event_off = EventOffStatement::new(range, event_type, handler);
        assert!(event_off.handler.is_some());
    }

    // ===================================================================
    // DISPATCH TESTS
    // ===================================================================

    #[test]
    fn test_dispatch_statement() {
        let range = create_range(1, 1, 1, 15);
        let action = Expression::Call(CallExpression::new(
            create_range(1, 9, 1, 15),
            Expression::Identifier(Identifier::new(create_range(1, 9, 1, 14), "INCREMENT".to_string())),
            vec![],
        ));

        let dispatch = DispatchStatement::new(range.clone(), action);
        assert_eq!(dispatch.node.range, range);
    }

    // ===================================================================
    // SERIALIZATION TESTS
    // ===================================================================

    #[test]
    fn test_ast_node_serialization() {
        let range = create_range(1, 1, 1, 5);
        let node = ASTNode::new(range);

        let serialized = serde_json::to_string(&node).unwrap();
        let deserialized: ASTNode = serde_json::from_str(&serialized).unwrap();

        assert_eq!(node.range.start.line, deserialized.range.start.line);
        assert_eq!(node.range.start.column, deserialized.range.start.column);
        assert_eq!(node.range.end.line, deserialized.range.end.line);
        assert_eq!(node.range.end.column, deserialized.range.end.column);
    }

    #[test]
    fn test_literal_serialization() {
        let range = create_range(1, 1, 1, 4);
        let literal = Literal::new(range, serde_json::json!(42), "number".to_string());

        let serialized = serde_json::to_string(&literal).unwrap();
        let deserialized: Literal = serde_json::from_str(&serialized).unwrap();

        assert_eq!(literal.value, deserialized.value);
        assert_eq!(literal.kind, deserialized.kind);
    }

    #[test]
    fn test_function_declaration_serialization() {
        let range = create_range(1, 1, 5, 2);
        let body = BlockStatement::new(create_range(4, 1, 5, 2), vec![]);
        let func = FunctionDeclaration::new(
            range,
            true,
            "add".to_string(),
            vec![],
            vec![],
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 20, 1, 26),
                "Number".to_string(),
            ))),
            body,
            false,
            vec![],
        );

        let serialized = serde_json::to_string(&func).unwrap();
        let deserialized: FunctionDeclaration = serde_json::from_str(&serialized).unwrap();

        assert_eq!(func.name, deserialized.name);
        assert_eq!(func.is_public, deserialized.is_public);
        assert_eq!(func.is_async, deserialized.is_async);
    }
}