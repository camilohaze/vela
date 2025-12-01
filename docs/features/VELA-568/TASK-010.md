# TASK-010: Estructura AST completa

## 📋 Información General
- **Historia:** VELA-568 (Parser que genere AST válido)
- **Estimación:** 32 horas
- **Estado:** ✅ Completada
- **Fecha:** 2025-11-30
- **Commit:** ba5d178
- **Archivo:** `src/parser/ast_nodes.py` (1,100+ líneas)

## 🎯 Objetivo

Definir una estructura completa de nodos del Abstract Syntax Tree (AST) que represente todas las construcciones sintácticas del lenguaje Vela, siguiendo los principios de:
- **Programación funcional pura**
- **Inmutabilidad por defecto**
- **Reactividad integrada**
- **UI declarativa**

## 🔨 Implementación

### Estructura del AST

El AST de Vela está compuesto por 60+ tipos de nodos organizados en las siguientes categorías:

#### 1. **Base Nodes** (Nodos fundamentales)

```python
class ASTNode:
    """Nodo base del AST"""
    position: Position         # Línea y columna del código fuente
    range: Range              # Rango de caracteres
    
class Position:
    line: int                 # Línea (1-indexed)
    column: int               # Columna (0-indexed)
    
class Range:
    start: Position           # Inicio del rango
    end: Position             # Fin del rango
    
class Program:
    """Nodo raíz del AST"""
    imports: List[ImportDeclaration]
    declarations: List[Declaration]
```

#### 2. **Import Declarations** (6 tipos)

```python
# Import types:
- SystemImport     # import 'system:io'
- PackageImport    # import 'package:http'
- ModuleImport     # import 'module:utils'
- LibraryImport    # import 'library:db'
- ExtensionImport  # import 'extension:vscode'
- AssetsImport     # import 'assets:logo.png'

# Import clauses:
- show: List[str]  # show { sin, cos, tan }
- hide: List[str]  # hide { deprecated_fn }
- alias: str       # as math
```

#### 3. **Declarations** (20+ tipos)

**Standard Declarations:**
- `FunctionDeclaration` - Funciones (normales, async, genéricas)
- `StructDeclaration` - Estructuras de datos
- `EnumDeclaration` - Enumeraciones (con datos asociados)
- `ClassDeclaration` - Clases (con herencia, interfaces)
- `InterfaceDeclaration` - Interfaces (contratos)
- `TypeAliasDeclaration` - Aliases de tipos

**Domain-Specific Declarations:**
- `ServiceDeclaration` - Servicios (lógica de negocio)
- `RepositoryDeclaration` - Repositorios (acceso a datos)
- `ControllerDeclaration` - Controladores (HTTP, etc.)
- `EntityDeclaration` - Entidades de dominio
- `DTODeclaration` - Data Transfer Objects
- `UseCaseDeclaration` - Casos de uso
- `ValueObjectDeclaration` - Value Objects
- `ModelDeclaration` - Modelos genéricos

**Design Pattern Declarations:**
- `FactoryDeclaration` - Factory pattern
- `BuilderDeclaration` - Builder pattern
- `StrategyDeclaration` - Strategy pattern
- `ObserverDeclaration` - Observer pattern
- `SingletonDeclaration` - Singleton pattern
- `AdapterDeclaration` - Adapter pattern
- `DecoratorDeclaration` - Decorator pattern

#### 4. **Statements** (15+ tipos)

```python
# Variable statements
- VariableStatement      # x: Number = 42
  - is_mutable: bool     # true si usa 'state'
  - name: str
  - type_annotation: Optional[TypeAnnotation]
  - initializer: Optional[Expression]

# Control flow
- IfStatement            # if condition { } else { }
  - condition: Expression
  - then_body: Block
  - else_body: Optional[Block]

- MatchStatement         # match value { patterns }
  - value: Expression
  - cases: List[MatchCase]

- TryStatement           # try { } catch (e) { } finally { }
  - try_body: Block
  - catch_clauses: List[CatchClause]
  - finally_body: Optional[Block]

# Other statements
- ReturnStatement        # return value
- ThrowStatement         # throw Error("msg")
- AssignmentStatement    # x = value
- ExpressionStatement    # call()
```

#### 5. **Expressions** (20+ tipos)

```python
# Literals
- NumberLiteral          # 42
- FloatLiteral           # 3.14
- StringLiteral          # "hello"
- BoolLiteral            # true/false
- NoneLiteral            # None

# Operators
- BinaryExpression       # a + b, x * y, etc.
  - left: Expression
  - operator: str        # +, -, *, /, %, **, <, >, <=, >=, ==, !=, &&, ||, ??
  - right: Expression

- UnaryExpression        # -x, !flag
  - operator: str        # -, !
  - operand: Expression

# Complex expressions
- CallExpression         # func(arg1, arg2)
  - callee: Expression
  - arguments: List[Expression]

- MemberExpression       # obj.field, obj?.field
  - object: Expression
  - member: str
  - is_optional_chain: bool  # true si usa ?.

- IndexExpression        # arr[0]
  - object: Expression
  - index: Expression

- LambdaExpression       # (x, y) => x + y
  - parameters: List[Parameter]
  - body: Expression | Block

- IfExpression           # if x > 0 { 1 } else { 0 }
  - condition: Expression
  - then_expr: Expression
  - else_expr: Expression

- MatchExpression        # match value { patterns }
  - value: Expression
  - cases: List[MatchCase]

- RangeExpression        # 0..10 (exclusivo), 0..=10 (inclusivo)
  - start: Expression
  - end: Expression
  - is_inclusive: bool

# Collection literals
- ArrayLiteral           # [1, 2, 3]
- TupleLiteral           # (1, "hello", true)
- StructLiteral          # Point { x: 10, y: 20 }
```

#### 6. **Patterns** (8 tipos)

Usados en `match` expressions:

```python
- LiteralPattern         # 1, "hello", true
- IdentifierPattern      # x, value
- WildcardPattern        # _
- TuplePattern           # (x, y)
- StructPattern          # User { id, name }
- EnumPattern            # Some(x), None
- OrPattern              # 1 | 2 | 3
- RangePattern           # 0..18, 18..65
```

#### 7. **Type Annotations** (9 tipos)

```python
- PrimitiveType          # Number, Float, String, Bool
- ArrayType              # [Number], [String]
- TupleType              # (Number, String, Bool)
- FunctionType           # (Number, Number) -> Number
- GenericType            # Option<T>, Result<T, E>
- UnionType              # "active" | "inactive"
- IntersectionType       # A & B
- OptionalType           # Option<T>
- NeverType              # never (función nunca retorna)
```

### Visitor Pattern

Para recorrer el AST:

```python
class ASTVisitor:
    """Base class for AST visitors"""
    
    def visit(self, node: ASTNode) -> Any:
        method_name = f'visit_{node.__class__.__name__}'
        method = getattr(self, method_name, self.generic_visit)
        return method(node)
    
    def generic_visit(self, node: ASTNode) -> Any:
        """Called if no explicit visitor method exists for a node"""
        pass
    
    # Visitor methods para cada tipo de nodo:
    def visit_Program(self, node: Program) -> Any: pass
    def visit_FunctionDeclaration(self, node: FunctionDeclaration) -> Any: pass
    # ... y así para todos los nodos
```

## 📁 Ubicación de Archivos

```
src/parser/ast_nodes.py          # Implementación completa (1,100+ líneas)
```

## ✅ Criterios de Aceptación

- [x] 60+ tipos de nodos AST definidos
- [x] Todos los nodos heredan de `ASTNode`
- [x] Position y Range en todos los nodos
- [x] Imports con 6 tipos (system, package, module, library, extension, assets)
- [x] Declarations: standard, domain-specific, patterns
- [x] Statements: variables (con state), control flow, etc.
- [x] Expressions: literals, operators, complex
- [x] Patterns: para match expressions
- [x] Type annotations: primitivos, genéricos, union
- [x] Visitor pattern implementado
- [x] Código committeado y versionado

## 🎓 Decisiones de Diseño

### 1. **Inmutabilidad por Defecto**

Variables sin keyword → inmutables:
```python
class VariableStatement:
    is_mutable: bool = False  # Por defecto inmutable
    # Solo True si usa 'state'
```

### 2. **`state` para Mutabilidad Reactiva**

```python
# Código Vela:
state counter: Number = 0  # Mutable y reactivo

# AST:
VariableStatement(
    is_mutable=True,         # ← Detecta 'state'
    name="counter",
    type_annotation=PrimitiveType("Number"),
    initializer=NumberLiteral(0)
)
```

### 3. **`Option<T>` en lugar de null**

No hay `NullLiteral` en el AST:
```python
# ❌ PROHIBIDO: null
# ✅ PERMITIDO: None (member de Option<T>)
NoneLiteral  # Representa 'None' de Option<T>
```

### 4. **Sin Loops Imperativos**

No hay nodos para `for`, `while`, `loop`:
```python
# ❌ NO EXISTEN:
# - ForStatement
# - WhileStatement
# - LoopStatement

# ✅ USAR EXPRESIONES FUNCIONALES:
# (0..10).forEach(i => print(i))
# Representado como:
CallExpression(
    callee=MemberExpression(
        object=RangeExpression(0, 10),
        member="forEach"
    ),
    arguments=[LambdaExpression(...)]
)
```

### 5. **`match` en lugar de switch**

```python
# ✅ EXISTE:
MatchStatement
MatchExpression

# ❌ NO EXISTE:
# SwitchStatement
```

### 6. **Modificador `public` en lugar de `export`**

```python
class FunctionDeclaration:
    is_public: bool = False   # true = accesible desde otros módulos
    # NO hay keyword 'export'
```

### 7. **Domain-Specific Declarations**

Para DDD y arquitectura limpia:
```python
# Código Vela:
service UserService {
    fn createUser(name: String) -> Result<User> { }
}

# AST:
ServiceDeclaration(
    name="UserService",
    methods=[FunctionDeclaration(...)]
)
```

## 📊 Métricas

- **Total líneas:** 1,100+
- **Tipos de nodos:** 60+
- **Categorías:** 7 (base, imports, declarations, statements, expressions, patterns, types)
- **Commit:** ba5d178

## 🔗 Referencias

- **Jira:** [VELA-568](https://velalang.atlassian.net/browse/VELA-568)
- **Historia:** [Sprint 6](../README.md)
- **Archivo:** `src/parser/ast_nodes.py`
- **Siguiente:** [TASK-008: Parser Recursive Descent](./TASK-008.md)

---

**Autor:** GitHub Copilot Agent  
**Fecha:** 2025-11-30  
**Commit:** ba5d178
