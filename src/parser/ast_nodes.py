"""
AST (Abstract Syntax Tree) Nodes para el lenguaje Vela

Implementación de: VELA-568 (TASK-010)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

⚠️ IMPORTANTE: Este es código Python del compilador de Vela

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
"""

from dataclasses import dataclass, field
from typing import List, Optional, Union, Any
from enum import Enum


# ===================================================================
# BASE NODE
# ===================================================================

@dataclass
class Position:
    """Posición en el código fuente (línea, columna)"""
    line: int
    column: int
    
    def __str__(self) -> str:
        return f"{self.line}:{self.column}"


@dataclass
class Range:
    """Rango de código fuente (inicio, fin)"""
    start: Position
    end: Position
    
    def __str__(self) -> str:
        return f"{self.start} - {self.end}"


@dataclass
class ASTNode:
    """
    Nodo base del AST.
    
    Todos los nodos del AST heredan de esta clase.
    Contiene información de posición para error reporting.
    """
    range: Range
    
    def accept(self, visitor: 'ASTVisitor') -> Any:
        """Visitor pattern para traversal del AST"""
        method_name = f'visit_{self.__class__.__name__}'
        visitor_method = getattr(visitor, method_name, visitor.generic_visit)
        return visitor_method(self)


# ===================================================================
# PROGRAM (Root Node)
# ===================================================================

@dataclass
class Program(ASTNode):
    """
    Nodo raíz del AST.
    
    Representa un programa completo de Vela.
    Contiene imports y declaraciones de nivel superior.
    """
    imports: List['ImportDeclaration'] = field(default_factory=list)
    declarations: List['Declaration'] = field(default_factory=list)


# ===================================================================
# IMPORTS
# ===================================================================

class ImportKind(Enum):
    """Tipos de imports en Vela"""
    SYSTEM = "system"      # system:io
    PACKAGE = "package"    # package:http
    MODULE = "module"      # module:utils
    LIBRARY = "library"    # library:math
    EXTENSION = "extension"  # extension:my_ext
    ASSETS = "assets"      # assets:images/logo.png


@dataclass
class ImportDeclaration(ASTNode):
    """
    Import statement.
    
    Ejemplos en Vela:
    - import 'package:http'
    - import 'module:utils' show { sort, filter }
    - import 'library:math' hide { deprecated_fn }
    - import 'package:long_name' as ln
    """
    kind: ImportKind
    path: str
    alias: Optional[str] = None
    show: Optional[List[str]] = None  # Importar solo estos
    hide: Optional[List[str]] = None  # Importar todo excepto estos


# ===================================================================
# DECLARATIONS
# ===================================================================

@dataclass
class Declaration(ASTNode):
    """Declaración de nivel superior"""
    is_public: bool = field(default=False, kw_only=True)  # Modificador public (keyword-only)


@dataclass
class FunctionDeclaration(Declaration):
    """
    Declaración de función.
    
    Ejemplo en Vela:
    ```vela
    public fn add(a: Number, b: Number) -> Number {
      return a + b
    }
    
    async fn fetchData() -> Result<String> {
      data = await http.get("api.com")
      return Ok(data)
    }
    ```
    """
    name: str
    parameters: List['Parameter']
    return_type: Optional['TypeAnnotation']
    body: 'BlockStatement'
    is_async: bool = False
    generic_params: List['GenericParameter'] = field(default_factory=list)


@dataclass
class Parameter:
    """Parámetro de función"""
    name: str
    type_annotation: Optional['TypeAnnotation']
    default_value: Optional['Expression'] = None
    range: Range = field(default=None)


@dataclass
class GenericParameter:
    """Parámetro genérico (ej: T, E)"""
    name: str
    constraints: List['TypeAnnotation'] = field(default_factory=list)
    range: Range = field(default=None)


@dataclass
class StructDeclaration(Declaration):
    """
    Declaración de struct.
    
    Ejemplo en Vela:
    ```vela
    struct User {
      id: Number
      name: String
      email: String
    }
    ```
    """
    name: str
    fields: List['StructField']
    generic_params: List['GenericParameter'] = field(default_factory=list)


@dataclass
class StructField:
    """Campo de struct"""
    name: str
    type_annotation: 'TypeAnnotation'
    is_public: bool = True
    range: Range = field(default=None)


@dataclass
class EnumDeclaration(Declaration):
    """
    Declaración de enum.
    
    Ejemplo en Vela:
    ```vela
    enum Color {
      Red
      Green
      Blue
      Custom(r: Number, g: Number, b: Number)
    }
    ```
    """
    name: str
    variants: List['EnumVariant']
    generic_params: List['GenericParameter'] = field(default_factory=list)


@dataclass
class EnumVariant:
    """Variante de enum"""
    name: str
    fields: Optional[List['StructField']] = None  # None para variantes sin datos
    range: Range = field(default=None)


@dataclass
class TypeAliasDeclaration(Declaration):
    """
    Alias de tipo.
    
    Ejemplo en Vela:
    ```vela
    type UserId = Number
    type Status = "active" | "inactive"
    ```
    """
    name: str
    type_annotation: 'TypeAnnotation'


@dataclass
class InterfaceDeclaration(Declaration):
    """
    Declaración de interface.
    
    Ejemplo en Vela:
    ```vela
    interface Drawable {
      fn draw() -> void
    }
    ```
    """
    name: str
    methods: List['FunctionSignature']
    generic_params: List['GenericParameter'] = field(default_factory=list)


@dataclass
class FunctionSignature:
    """Firma de función (sin body)"""
    name: str
    parameters: List['Parameter']
    return_type: Optional['TypeAnnotation']
    is_async: bool = False
    range: Range = field(default=None)


@dataclass
class ClassDeclaration(Declaration):
    """
    Declaración de clase.
    
    Ejemplo en Vela:
    ```vela
    class Dog extends Animal implements Petable {
      name: String
      
      constructor(name: String) {
        this.name = name
      }
      
      fn bark() -> void {
        print("Woof!")
      }
    }
    ```
    """
    name: str
    constructor: Optional['ConstructorDeclaration']
    fields: List['ClassField']
    methods: List['MethodDeclaration']
    extends: Optional[str] = None
    implements: List[str] = field(default_factory=list)
    generic_params: List['GenericParameter'] = field(default_factory=list)


@dataclass
class ClassField:
    """Campo de clase"""
    name: str
    type_annotation: Optional['TypeAnnotation']
    is_state: bool = False  # Si es `state` (mutable y reactivo)
    initial_value: Optional['Expression'] = None
    is_public: bool = True
    is_protected: bool = False
    is_private: bool = False
    range: Range = field(default=None)


@dataclass
class ConstructorDeclaration:
    """Constructor de clase"""
    parameters: List['Parameter']
    body: 'BlockStatement'
    range: Range


@dataclass
class MethodDeclaration:
    """Método de clase"""
    name: str
    parameters: List['Parameter']
    return_type: Optional['TypeAnnotation']
    body: 'BlockStatement'
    range: Range
    is_async: bool = False
    is_override: bool = False
    is_public: bool = True
    is_protected: bool = False
    is_private: bool = False


# Keywords Específicos de Dominio (DDD, Architecture Patterns, etc.)

@dataclass
class ServiceDeclaration(Declaration):
    """service keyword - Capa de servicio (lógica de negocio)"""
    name: str
    methods: List['MethodDeclaration']
    dependencies: List['Parameter'] = field(default_factory=list)


@dataclass
class RepositoryDeclaration(Declaration):
    """repository keyword - Capa de acceso a datos"""
    name: str
    entity_type: str
    methods: List['MethodDeclaration']


@dataclass
class ControllerDeclaration(Declaration):
    """controller keyword - Controlador HTTP"""
    name: str
    routes: List['RouteDeclaration']


@dataclass
class RouteDeclaration:
    """Ruta HTTP (decorada con @get, @post, etc.)"""
    method: str  # "GET", "POST", "PUT", "DELETE", "PATCH"
    path: str
    handler: 'FunctionDeclaration'
    range: Range


@dataclass
class UseCaseDeclaration(Declaration):
    """usecase keyword - Caso de uso / interactor"""
    name: str
    execute_method: 'MethodDeclaration'
    dependencies: List['Parameter'] = field(default_factory=list)


@dataclass
class EntityDeclaration(Declaration):
    """entity keyword - Entidad de dominio"""
    name: str
    id_field: 'StructField'
    fields: List['StructField']


@dataclass
class ValueObjectDeclaration(Declaration):
    """valueObject keyword - Value Object inmutable"""
    name: str
    fields: List['StructField']


@dataclass
class DTODeclaration(Declaration):
    """dto keyword - Data Transfer Object"""
    name: str
    fields: List['StructField']


# ===================================================================
# UI KEYWORDS
# ===================================================================

@dataclass
class WidgetDeclaration(Declaration):
    """widget keyword - Stateful widget (UI component)"""
    name: str
    fields: List['ClassField']
    methods: List['MethodDeclaration']


@dataclass
class ComponentDeclaration(Declaration):
    """component keyword - UI component"""
    name: str
    fields: List['ClassField']
    methods: List['MethodDeclaration']


# ===================================================================
# MODEL KEYWORDS
# ===================================================================

@dataclass
class ModelDeclaration(Declaration):
    """model keyword - Generic model"""
    name: str
    fields: List['StructField']


# ===================================================================
# DESIGN PATTERN KEYWORDS
# ===================================================================

@dataclass
class FactoryDeclaration(Declaration):
    """factory keyword - Factory pattern"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class BuilderDeclaration(Declaration):
    """builder keyword - Builder pattern"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class StrategyDeclaration(Declaration):
    """strategy keyword - Strategy pattern"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class ObserverDeclaration(Declaration):
    """observer keyword - Observer pattern"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class SingletonDeclaration(Declaration):
    """singleton keyword - Singleton pattern"""
    name: str
    fields: List['ClassField']
    methods: List['MethodDeclaration']


@dataclass
class AdapterDeclaration(Declaration):
    """adapter keyword - Adapter pattern"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class DecoratorDeclaration(Declaration):
    """decorator keyword - Decorator pattern"""
    name: str
    methods: List['MethodDeclaration']


# ===================================================================
# WEB/API KEYWORDS
# ===================================================================

@dataclass
class GuardDeclaration(Declaration):
    """guard keyword - Route guard/authorization"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class MiddlewareDeclaration(Declaration):
    """middleware keyword - HTTP middleware"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class InterceptorDeclaration(Declaration):
    """interceptor keyword - Request/response interceptor"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class ValidatorDeclaration(Declaration):
    """validator keyword - Input validator"""
    name: str
    methods: List['MethodDeclaration']


# ===================================================================
# STATE & DI KEYWORDS
# ===================================================================

@dataclass
class StoreDeclaration(Declaration):
    """store keyword - State store"""
    name: str
    fields: List['ClassField']
    methods: List['MethodDeclaration']


@dataclass
class ProviderDeclaration(Declaration):
    """provider keyword - Dependency provider"""
    name: str
    methods: List['MethodDeclaration']


# ===================================================================
# CONCURRENCY KEYWORDS
# ===================================================================

@dataclass
class ActorDeclaration(Declaration):
    """actor keyword - Actor model concurrency"""
    name: str
    fields: List['ClassField']
    methods: List['MethodDeclaration']


# ===================================================================
# UTILITY KEYWORDS
# ===================================================================

@dataclass
class PipeDeclaration(Declaration):
    """pipe keyword - Transformation pipeline"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class TaskDeclaration(Declaration):
    """task keyword - Asynchronous task/job"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class HelperDeclaration(Declaration):
    """helper keyword - Helper utilities"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class MapperDeclaration(Declaration):
    """mapper keyword - Object mapper"""
    name: str
    methods: List['MethodDeclaration']


@dataclass
class SerializerDeclaration(Declaration):
    """serializer keyword - Data serializer"""
    name: str
    methods: List['MethodDeclaration']


# ===================================================================
# MODULE SYSTEM (Angular-style)
# ===================================================================

@dataclass
class Decorator:
    """
    Decorator/Annotation.
    
    Ejemplos en Vela:
    ```vela
    @injectable
    @controller("/api/users")
    @get("/profile")
    @validate
    @module({
      declarations: [UserService, UserController],
      exports: [UserService],
      providers: [UserService],
      imports: [HttpModule]
    })
    ```
    
    Tipos de decoradores:
    - DI: @injectable, @inject, @container, @provides, @singleton
    - HTTP/REST: @controller, @get, @post, @put, @delete, @patch
    - Guards/Middleware: @middleware, @guard
    - Validación: @validate, @required, @email, @min, @max, @length, @regex
    - Module System: @module, @extension, @library, @package
    """
    name: str
    arguments: List['Expression'] = field(default_factory=list)
    range: Range = field(default=None)


@dataclass
class ModuleDeclaration(Declaration):
    """
    module keyword - Angular-style module (NO instanciable).
    
    Ejemplo en Vela:
    ```vela
    @module({
      declarations: [AuthService, LoginWidget, RegisterWidget],
      exports: [AuthService],
      providers: [AuthService, TokenService],
      imports: [HttpModule, CryptoModule]
    })
    module AuthModule {
      # Módulo NO instanciable (NO tiene constructor)
      # NO se puede hacer: new AuthModule()
    }
    ```
    
    Reglas Obligatorias:
    1. DEBE tener decorador @module({ ... })
    2. declarations DEBE contener todas las clases del módulo
    3. exports DEBE ser subconjunto de declarations (exports ⊆ declarations)
    4. providers DEBE ser subconjunto de declarations
    5. imports DEBE referenciar otros módulos válidos
    6. NO es instanciable (NO tiene constructor, NO se puede hacer new ModuleX())
    7. NO puede tener métodos de instancia
    8. Solo puede contener declaraciones estáticas
    
    Sistema de Imports con Prefijos:
    - import 'system:ui'      → APIs internas de Vela (Container, Column, Text, etc.)
    - import 'package:http'   → Dependencias externas (npm, pub, etc.)
    - import 'module:auth'    → Módulos del proyecto (@module)
    - import 'library:utils'  → Librerías internas (@library)
    - import 'extension:charts' → Extensiones internas (@extension)
    - import 'assets:images'  → Assets del proyecto
    """
    name: str
    decorators: List[Decorator] = field(default_factory=list)
    body: List[Declaration] = field(default_factory=list)  # Declaraciones dentro del módulo
    
    # Metadata extraída del decorador @module (parsed desde object literal)
    # Estos son Expressions (Identifier para clases, Literal para strings de imports)
    # La validación semántica (exports ⊆ declarations) se hace en semantic analyzer
    declarations: List['Expression'] = field(default_factory=list)  # [Service1, Service2]
    exports: List['Expression'] = field(default_factory=list)       # [Service1]
    providers: List['Expression'] = field(default_factory=list)     # [Service1, DatabaseConnection]
    imports: List['Expression'] = field(default_factory=list)       # ['system:http', 'module:auth']


# ===================================================================
# STATEMENTS
# ===================================================================

@dataclass
class Statement(ASTNode):
    """Statement base"""
    pass


@dataclass
class BlockStatement(Statement):
    """
    Bloque de statements.
    
    Ejemplo en Vela:
    ```vela
    {
      x = 5
      y = 10
      print(x + y)
    }
    ```
    """
    statements: List[Statement] = field(default_factory=list)


@dataclass
class ExpressionStatement(Statement):
    """Expression como statement"""
    expression: 'Expression'


@dataclass
class VariableDeclaration(Statement):
    """
    Declaración de variable.
    
    Ejemplos en Vela:
    ```vela
    # Inmutable por defecto (NO se usa let/const)
    name: String = "Vela"
    
    # Mutable y reactivo (ÚNICA forma de mutabilidad)
    state count: Number = 0
    ```
    """
    name: str
    type_annotation: Optional['TypeAnnotation']
    initializer: Optional['Expression']
    is_state: bool = False  # Si es `state` (mutable y reactivo)


@dataclass
class AssignmentStatement(Statement):
    """
    Asignación a variable existente.
    
    Solo válido para variables `state`.
    Ejemplo en Vela:
    ```vela
    count = count + 1  # OK solo si count es `state`
    ```
    """
    target: 'Expression'  # Puede ser Identifier, MemberAccess, IndexAccess
    value: 'Expression'


@dataclass
class ReturnStatement(Statement):
    """Return statement"""
    value: Optional['Expression'] = None


@dataclass
class IfStatement(Statement):
    """
    If statement.
    
    Ejemplo en Vela:
    ```vela
    if age >= 18 {
      print("adult")
    } else {
      print("minor")
    }
    ```
    """
    condition: 'Expression'
    then_branch: Statement
    else_branch: Optional[Statement] = None


@dataclass
class MatchStatement(Statement):
    """
    Match statement (pattern matching exhaustivo).
    
    Ejemplo en Vela:
    ```vela
    match result {
      Ok(value) => print("Success: ${value}")
      Err(error) => print("Error: ${error}")
    }
    ```
    """
    value: 'Expression'
    arms: List['MatchArm']


@dataclass
class MatchArm:
    """Brazo de match expression"""
    pattern: 'Pattern'
    guard: Optional['Expression'] = None  # if condition
    body: Statement = None
    range: Range = field(default=None)


@dataclass
class ThrowStatement(Statement):
    """Throw exception"""
    exception: 'Expression'


@dataclass
class TryStatement(Statement):
    """
    Try-catch-finally statement.
    
    Ejemplo en Vela:
    ```vela
    try {
      riskyOp()
    } catch (e: MyError) {
      handle(e)
    } finally {
      cleanup()
    }
    ```
    """
    try_block: BlockStatement
    catch_clauses: List['CatchClause']
    finally_block: Optional[BlockStatement] = None


@dataclass
class CatchClause:
    """Cláusula catch"""
    exception_name: str
    exception_type: Optional['TypeAnnotation']
    body: BlockStatement
    range: Range


# ===================================================================
# EVENT SYSTEM (TASK-035M)
# ===================================================================

@dataclass
class EventOnStatement(Statement):
    """
    Event listener registration: on(event_type, handler)
    
    Sintaxis en Vela:
    ```vela
    # Callback inline
    on("user.created", (event) => {
      print("User created: ${event.payload.name}")
    })
    
    # Callback con nombre
    on("user.deleted", handleUserDeleted)
    
    # Con tipo de evento
    on<UserEvent>("user.updated", handleUserUpdated)
    ```
    
    Generará:
    ```python
    bus.on("user.created", lambda event: ...)
    ```
    """
    event_type: 'Expression'  # Nombre del evento (String literal o Expression)
    handler: 'Expression'     # Callback function (Lambda o Identifier)
    type_param: Optional['TypeAnnotation'] = None  # on<T>


@dataclass
class EventEmitStatement(Statement):
    """
    Event emission: emit(event_type, payload)
    
    Sintaxis en Vela:
    ```vela
    # Emit simple
    emit("user.created", user)
    
    # Emit con datos inline
    emit("notification", {
      message: "Hello",
      level: "info"
    })
    
    # Emit sin datos
    emit("app.started")
    ```
    
    Generará:
    ```python
    bus.emit("user.created", user)
    ```
    """
    event_type: 'Expression'  # Nombre del evento
    payload: Optional['Expression'] = None  # Datos del evento (opcional)


@dataclass
class EventOffStatement(Statement):
    """
    Event listener removal: off(event_type, handler)
    
    Sintaxis en Vela:
    ```vela
    # Remover listener específico
    off("user.created", handleUserCreated)
    
    # Remover todos los listeners de un evento
    off("user.created")
    ```
    
    Generará:
    ```python
    bus.off("user.created", handleUserCreated)
    # o
    bus.clear("user.created")
    ```
    """
    event_type: 'Expression'  # Nombre del evento
    handler: Optional['Expression'] = None  # Handler a remover (opcional)


# ===================================================================
# EXPRESSIONS
# ===================================================================

@dataclass
class Expression(ASTNode):
    """Expression base"""
    pass


@dataclass
class Literal(Expression):
    """
    Valor literal.
    
    Ejemplos en Vela:
    - Números: 42, 3.14
    - Strings: "hello", 'world'
    - Booleanos: true, false
    - Option: None (NO null)
    """
    value: Any  # int, float, str, bool, None
    kind: str   # "number", "float", "string", "bool", "none"


@dataclass
class Identifier(Expression):
    """Identificador (nombre de variable, función, etc.)"""
    name: str


@dataclass
class BinaryExpression(Expression):
    """
    Expresión binaria.
    
    Ejemplos en Vela:
    - Aritméticas: a + b, x * y, base ** exp
    - Comparación: x == y, a < b
    - Lógicas: p && q, a || b
    - None coalescing: value ?? default (NO null)
    """
    left: Expression
    operator: str  # "+", "-", "*", "/", "**", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "??", etc.
    right: Expression


@dataclass
class UnaryExpression(Expression):
    """
    Expresión unaria.
    
    Ejemplos en Vela: -x, !flag
    """
    operator: str  # "-", "!"
    operand: Expression


@dataclass
class CallExpression(Expression):
    """
    Llamada a función.
    
    Ejemplo en Vela:
    ```vela
    add(2, 3)
    user.getName()
    [1, 2, 3].map(x => x * 2)
    ```
    """
    callee: Expression  # Función a llamar
    arguments: List[Expression]


@dataclass
class MemberAccessExpression(Expression):
    """
    Acceso a miembro (dot notation).
    
    Ejemplos en Vela:
    - user.name
    - object.method()
    - Optional chaining: user?.address?.street
    """
    object: Expression
    member: str
    is_optional: bool = False  # Si usa ?. (optional chaining)


@dataclass
class IndexAccessExpression(Expression):
    """
    Acceso por índice.
    
    Ejemplos en Vela:
    - array[0]
    - map["key"]
    """
    object: Expression
    index: Expression


@dataclass
class ArrayLiteral(Expression):
    """
    Array literal.
    
    Ejemplo en Vela: [1, 2, 3, 4]
    """
    elements: List[Expression]


@dataclass
class TupleLiteral(Expression):
    """
    Tuple literal.
    
    Ejemplo en Vela: (1, "hello", true)
    """
    elements: List[Expression]


@dataclass
class StructLiteral(Expression):
    """
    Struct literal.
    
    Ejemplo en Vela:
    ```vela
    User { id: 1, name: "Alice", email: "alice@example.com" }
    ```
    """
    struct_name: str
    fields: List['StructLiteralField']


@dataclass
class StructLiteralField:
    """Campo de struct literal"""
    name: str
    value: Expression
    range: Range


@dataclass
class LambdaExpression(Expression):
    """
    Función anónima (arrow function).
    
    Ejemplos en Vela:
    ```vela
    (x) => x * 2
    (a, b) => a + b
    (data) => {
      cleaned = data.trim()
      return cleaned.toUpperCase()
    }
    ```
    """
    parameters: List[Parameter]
    body: Union[Expression, BlockStatement]  # Expression para arrow, Block para cuerpo


@dataclass
class IfExpression(Expression):
    """
    If como expresión (retorna valor).
    
    Ejemplo en Vela:
    ```vela
    status = if age >= 18 { "adult" } else { "minor" }
    ```
    """
    condition: Expression
    then_branch: Expression
    else_branch: Expression


@dataclass
class MatchExpression(Expression):
    """
    Match como expresión (retorna valor).
    
    Ejemplo en Vela:
    ```vela
    message = match result {
      Ok(val) => "Success: ${val}"
      Err(e) => "Error: ${e}"
    }
    ```
    """
    value: Expression
    arms: List['MatchExpressionArm']


@dataclass
class MatchExpressionArm:
    """Brazo de match expression"""
    pattern: 'Pattern'
    guard: Optional[Expression] = None
    body: Expression = None
    range: Range = field(default=None)


@dataclass
class StringInterpolation(Expression):
    """
    String interpolation con ${}.
    
    Ejemplo en Vela:
    ```vela
    "Hello, ${name}!"
    "Result: ${fn() -> Number { calculate(x, y) }}"
    ```
    """
    parts: List[Union[str, Expression]]  # Alterna strings y expresiones


@dataclass
class AwaitExpression(Expression):
    """
    Await expression.
    
    Ejemplo en Vela:
    ```vela
    data = await fetchData()
    ```
    """
    expression: Expression


@dataclass
class ComputedExpression(Expression):
    """
    Computed value (reactivo, derivado).
    
    Ejemplo en Vela:
    ```vela
    computed doubled: Number {
      return this.count * 2
    }
    ```
    """
    body: BlockStatement


# ===================================================================
# PATTERNS (for match expressions)
# ===================================================================

@dataclass
class Pattern(ASTNode):
    """Pattern base para match"""
    pass


@dataclass
class LiteralPattern(Pattern):
    """Pattern literal: 1, "hello", true"""
    value: Any


@dataclass
class IdentifierPattern(Pattern):
    """Pattern identifier: bind a variable"""
    name: str


@dataclass
class TuplePattern(Pattern):
    """Pattern tuple: (x, y, z)"""
    elements: List[Pattern]


@dataclass
class StructPattern(Pattern):
    """
    Pattern struct.
    
    Ejemplo: User { id, name }
    """
    struct_name: str
    fields: List['StructPatternField']


@dataclass
class StructPatternField:
    """Campo de struct pattern"""
    name: str
    pattern: Pattern
    range: Range


@dataclass
class EnumPattern(Pattern):
    """
    Pattern enum variant.
    
    Ejemplo: Ok(value), Err(error)
    """
    variant_name: str
    inner_patterns: Optional[List[Pattern]] = None


@dataclass
class OrPattern(Pattern):
    """Pattern or: pattern1 | pattern2"""
    patterns: List[Pattern]


@dataclass
class RangePattern(Pattern):
    """Pattern range: 1..10, 'a'..='z'"""
    start: Expression
    end: Expression
    is_inclusive: bool = False  # True para ..=, False para ..


@dataclass
class WildcardPattern(Pattern):
    """Pattern wildcard: _ (catch-all)"""
    pass


# ===================================================================
# TYPE ANNOTATIONS
# ===================================================================

@dataclass
class TypeAnnotation(ASTNode):
    """Type annotation base"""
    pass


@dataclass
class PrimitiveType(TypeAnnotation):
    """
    Tipo primitivo.
    
    En Vela: Number, Float, String, Bool, void, never
    """
    name: str  # "Number", "Float", "String", "Bool", "void", "never"


@dataclass
class ArrayType(TypeAnnotation):
    """Tipo array: List<T>"""
    element_type: TypeAnnotation


@dataclass
class TupleType(TypeAnnotation):
    """Tipo tuple: (Number, String, Bool)"""
    element_types: List[TypeAnnotation]


@dataclass
class FunctionType(TypeAnnotation):
    """
    Tipo función.
    
    Ejemplo: (Number, Number) -> Number
    """
    parameter_types: List[TypeAnnotation]
    return_type: TypeAnnotation


@dataclass
class GenericType(TypeAnnotation):
    """
    Tipo genérico.
    
    Ejemplos: Option<T>, Result<T, E>, List<String>
    """
    base_name: str
    type_arguments: List[TypeAnnotation]


@dataclass
class UnionType(TypeAnnotation):
    """
    Union type.
    
    Ejemplo: "active" | "inactive" | "pending"
    """
    types: List[TypeAnnotation]


@dataclass
class NamedType(TypeAnnotation):
    """Tipo nombrado: User, Config, etc."""
    name: str


@dataclass
class OptionalType(TypeAnnotation):
    """
    Tipo opcional (Option<T>).
    
    En Vela: Option<T> (NO T?)
    """
    inner_type: TypeAnnotation


# ===================================================================
# AST VISITOR (for traversal)
# ===================================================================

class ASTVisitor:
    """
    Visitor pattern para traversal del AST.
    
    Subclasear este visitor para implementar análisis o transformaciones del AST.
    """
    
    def generic_visit(self, node: ASTNode) -> Any:
        """Visita genérica (fallback)"""
        raise NotImplementedError(f"No visit method for {node.__class__.__name__}")
    
    def visit_Program(self, node: Program) -> Any:
        for imp in node.imports:
            imp.accept(self)
        for decl in node.declarations:
            decl.accept(self)
    
    def visit_FunctionDeclaration(self, node: FunctionDeclaration) -> Any:
        node.body.accept(self)
    
    def visit_BlockStatement(self, node: BlockStatement) -> Any:
        for stmt in node.statements:
            stmt.accept(self)
    
    # ... más visit methods según se necesiten


# ===================================================================
# UTILITY FUNCTIONS
# ===================================================================

def create_position(line: int, column: int) -> Position:
    """Helper para crear Position"""
    return Position(line=line, column=column)


def create_range(start_line: int, start_col: int, end_line: int, end_col: int) -> Range:
    """Helper para crear Range"""
    return Range(
        start=Position(start_line, start_col),
        end=Position(end_line, end_col)
    )


def is_expression_statement_valid(expr: Expression) -> bool:
    """
    Verifica si una expresión es válida como statement.
    
    Válidas: CallExpression, AssignmentExpression
    Inválidas: BinaryExpression (a + b como statement no tiene sentido)
    """
    return isinstance(expr, (CallExpression, AssignmentStatement))


if __name__ == "__main__":
    # Ejemplo de uso: crear un AST simple
    
    # fn add(a: Number, b: Number) -> Number { return a + b }
    
    add_function = FunctionDeclaration(
        range=create_range(1, 1, 1, 50),
        is_public=True,
        name="add",
        parameters=[
            Parameter(name="a", type_annotation=PrimitiveType(range=create_range(1, 11, 1, 17), name="Number")),
            Parameter(name="b", type_annotation=PrimitiveType(range=create_range(1, 19, 1, 25), name="Number"))
        ],
        return_type=PrimitiveType(range=create_range(1, 30, 1, 36), name="Number"),
        body=BlockStatement(
            range=create_range(1, 38, 1, 50),
            statements=[
                ReturnStatement(
                    range=create_range(1, 40, 1, 48),
                    value=BinaryExpression(
                        range=create_range(1, 47, 1, 48),
                        left=Identifier(range=create_range(1, 47, 1, 48), name="a"),
                        operator="+",
                        right=Identifier(range=create_range(1, 47, 1, 48), name="b")
                    )
                )
            ]
        )
    )
    
    program = Program(
        range=create_range(1, 1, 1, 50),
        imports=[],
        declarations=[add_function]
    )
    
    print("✅ AST Node Structure created successfully")
    print(f"Program with {len(program.declarations)} declarations")
    print(f"Function: {add_function.name} with {len(add_function.parameters)} parameters")
