# TASK-008: Parser Recursive Descent

## 📋 Información General
- **Historia:** VELA-568 (Parser que genere AST válido)
- **Estimación:** 120 horas
- **Estado:** ✅ Completada
- **Fecha:** 2025-11-30
- **Commit:** 0be9b08
- **Archivo:** `src/parser/parser.py` (1,850+ líneas)

## 🎯 Objetivo

Implementar un parser completo usando la técnica de **Recursive Descent** que transforme tokens en un AST válido. El parser debe manejar todas las construcciones sintácticas de Vela:
- Imports con 6 tipos
- Declarations (standard, domain-specific, patterns)
- Statements (variables, control flow)
- Integración con Pratt Parser para expresiones

## 🔨 Implementación

### Arquitectura del Parser

```
Parser (Recursive Descent)
├── parse_program()           → Program
│   ├── parse_import()*       → ImportDeclaration
│   └── parse_declaration()*  → Declaration
│
├── parse_import()
│   ├── system:, package:, module:, library:, extension:, assets:
│   └── clauses: show, hide, as
│
├── parse_declaration()
│   ├── parse_function()      → FunctionDeclaration
│   ├── parse_struct()        → StructDeclaration
│   ├── parse_enum()          → EnumDeclaration
│   ├── parse_class()         → ClassDeclaration
│   ├── parse_interface()     → InterfaceDeclaration
│   ├── parse_type_alias()    → TypeAliasDeclaration
│   └── parse_domain_specific() → Service, Repository, etc.
│
├── parse_statement()
│   ├── parse_variable()      → VariableStatement
│   ├── parse_if()            → IfStatement
│   ├── parse_match()         → MatchStatement
│   ├── parse_try()           → TryStatement
│   ├── parse_return()        → ReturnStatement
│   └── parse_throw()         → ThrowStatement
│
├── parse_expression()        → Delega a PrattParser
│
├── parse_pattern()           → Pattern (para match)
│   ├── Literal, Identifier, Wildcard
│   ├── Tuple, Struct, Enum
│   └── Or, Range patterns
│
└── parse_type_annotation()   → TypeAnnotation
    ├── Primitive, Array, Tuple, Function
    └── Generic, Union, Option
```

### Métodos Principales

#### 1. **parse_program()**
Entry point del parser:

```python
def parse_program(self) -> Program:
    """
    program = import_declaration* declaration*
    """
    imports: List[ImportDeclaration] = []
    declarations: List[Declaration] = []
    
    # Parse imports
    while self.current_token.type == 'IMPORT':
        imports.append(self.parse_import())
    
    # Parse declarations
    while not self.is_at_end():
        declarations.append(self.parse_declaration())
    
    return Program(imports=imports, declarations=declarations)
```

#### 2. **parse_import()**
Maneja 6 tipos de imports:

```python
def parse_import(self) -> ImportDeclaration:
    """
    import_declaration = 
        'import' string_literal (import_clause)*
    
    import_clause =
        'show' '{' identifier (',' identifier)* '}'
      | 'hide' '{' identifier (',' identifier)* '}'
      | 'as' identifier
    """
    self.consume('IMPORT')
    
    # Parsear path: 'system:io', 'package:http', etc.
    path = self.consume('STRING').value
    
    # Detectar tipo de import por prefijo
    if path.startswith('system:'):
        import_type = 'system'
    elif path.startswith('package:'):
        import_type = 'package'
    # ... otros tipos
    
    # Parsear clauses opcionales
    show_list = []
    hide_list = []
    alias = None
    
    if self.match('SHOW'):
        show_list = self.parse_identifier_list()
    if self.match('HIDE'):
        hide_list = self.parse_identifier_list()
    if self.match('AS'):
        alias = self.consume('IDENTIFIER').value
    
    return ImportDeclaration(
        path=path,
        import_type=import_type,
        show_list=show_list,
        hide_list=hide_list,
        alias=alias
    )
```

#### 3. **parse_declaration()**
Router para diferentes tipos de declarations:

```python
def parse_declaration(self) -> Declaration:
    """Parsea declaration basándose en keyword actual"""
    
    # Check modifiers
    is_public = self.match('PUBLIC')
    is_async = self.match('ASYNC')
    
    # Route por keyword
    if self.match('FN'):
        return self.parse_function(is_public, is_async)
    elif self.match('STRUCT'):
        return self.parse_struct(is_public)
    elif self.match('ENUM'):
        return self.parse_enum(is_public)
    elif self.match('CLASS'):
        return self.parse_class(is_public)
    elif self.match('INTERFACE'):
        return self.parse_interface(is_public)
    elif self.match('TYPE'):
        return self.parse_type_alias(is_public)
    
    # Domain-specific
    elif self.match('SERVICE'):
        return self.parse_service(is_public)
    elif self.match('REPOSITORY'):
        return self.parse_repository(is_public)
    # ... otros domain-specific
    
    else:
        raise ParserError(f"Unexpected token: {self.current_token}")
```

#### 4. **parse_function()**
Functions con generics, async, params:

```python
def parse_function(self, is_public: bool, is_async: bool) -> FunctionDeclaration:
    """
    function_declaration =
        modifiers? 'async'? 'fn' identifier generic_params? 
        '(' parameters? ')' ('->' type_annotation)? block
    """
    name = self.consume('IDENTIFIER').value
    
    # Generics
    generic_params = []
    if self.match('LESS'):
        generic_params = self.parse_generic_params()
    
    # Parameters
    self.consume('LPAREN')
    parameters = []
    if not self.check('RPAREN'):
        parameters = self.parse_parameters()
    self.consume('RPAREN')
    
    # Return type
    return_type = None
    if self.match('ARROW'):
        return_type = self.parse_type_annotation()
    
    # Body
    body = self.parse_block()
    
    return FunctionDeclaration(
        name=name,
        is_public=is_public,
        is_async=is_async,
        generic_params=generic_params,
        parameters=parameters,
        return_type=return_type,
        body=body
    )
```

#### 5. **parse_statement()**
Statements con control flow:

```python
def parse_statement(self) -> Statement:
    """Parsea statement basándose en keyword"""
    
    # Variable declaration
    if self.check_type_annotation() or self.match('STATE'):
        return self.parse_variable()
    
    # Control flow
    if self.match('IF'):
        return self.parse_if()
    if self.match('MATCH'):
        return self.parse_match()
    if self.match('TRY'):
        return self.parse_try()
    
    # Return/throw
    if self.match('RETURN'):
        return self.parse_return()
    if self.match('THROW'):
        return self.parse_throw()
    
    # Expression statement
    return self.parse_expression_statement()
```

#### 6. **parse_variable()**
Variables con state para mutabilidad:

```python
def parse_variable(self) -> VariableStatement:
    """
    variable_statement =
        'state'? identifier ':' type_annotation ('=' expression)?
    """
    # Detectar 'state' keyword
    is_mutable = self.match('STATE')
    
    name = self.consume('IDENTIFIER').value
    
    # Type annotation (opcional)
    type_annotation = None
    if self.match('COLON'):
        type_annotation = self.parse_type_annotation()
    
    # Initializer (opcional)
    initializer = None
    if self.match('EQUALS'):
        initializer = self.parse_expression()
    
    return VariableStatement(
        is_mutable=is_mutable,
        name=name,
        type_annotation=type_annotation,
        initializer=initializer
    )
```

#### 7. **parse_match()**
Match con patterns exhaustivos:

```python
def parse_match(self) -> MatchStatement:
    """
    match_statement =
        'match' expression '{' match_case+ '}'
    
    match_case =
        pattern ('if' expression)? '=>' (expression | block)
    """
    value = self.parse_expression()
    
    self.consume('LBRACE')
    cases = []
    
    while not self.check('RBRACE'):
        pattern = self.parse_pattern()
        
        # Guard opcional
        guard = None
        if self.match('IF'):
            guard = self.parse_expression()
        
        self.consume('ARROW')
        
        # Body puede ser expression o block
        if self.check('LBRACE'):
            body = self.parse_block()
        else:
            body = self.parse_expression()
        
        cases.append(MatchCase(
            pattern=pattern,
            guard=guard,
            body=body
        ))
    
    self.consume('RBRACE')
    
    return MatchStatement(value=value, cases=cases)
```

#### 8. **parse_pattern()**
Patterns para match:

```python
def parse_pattern(self) -> Pattern:
    """
    pattern =
        literal_pattern
      | identifier_pattern
      | wildcard_pattern '_'
      | tuple_pattern '(' pattern (',' pattern)* ')'
      | struct_pattern identifier '{' field_patterns '}'
      | enum_pattern identifier '(' patterns ')'
      | or_pattern pattern '|' pattern
      | range_pattern expression '..' expression
    """
    # Wildcard
    if self.match('UNDERSCORE'):
        return WildcardPattern()
    
    # Literal
    if self.is_literal():
        return LiteralPattern(value=self.parse_literal())
    
    # Tuple
    if self.check('LPAREN'):
        return self.parse_tuple_pattern()
    
    # Identifier (puede ser enum o simple binding)
    if self.check('IDENTIFIER'):
        identifier = self.consume('IDENTIFIER').value
        
        # Enum pattern: Some(x)
        if self.check('LPAREN'):
            return self.parse_enum_pattern(identifier)
        
        # Struct pattern: User { id, name }
        if self.check('LBRACE'):
            return self.parse_struct_pattern(identifier)
        
        # Simple identifier binding
        return IdentifierPattern(name=identifier)
    
    raise ParserError("Expected pattern")
```

#### 9. **parse_expression()**
Delega a Pratt Parser:

```python
def parse_expression(self) -> Expression:
    """
    Delega parsing de expresiones al Pratt Parser
    para manejar precedencia correctamente
    """
    from .pratt_parser import PrattParser
    
    pratt = PrattParser(self.tokens, self.current)
    return pratt.parse_expression()
```

#### 10. **parse_type_annotation()**
Type annotations con generics:

```python
def parse_type_annotation(self) -> TypeAnnotation:
    """
    type_annotation =
        primitive_type
      | array_type '[' type_annotation ']'
      | tuple_type '(' type_annotation (',' type_annotation)* ')'
      | function_type '(' types ')' '->' type_annotation
      | generic_type identifier '<' type_args '>'
      | union_type type '|' type
      | 'Option' '<' type '>'
    """
    # Primitive
    if self.match_any(['NUMBER', 'FLOAT', 'STRING', 'BOOL', 'VOID']):
        return PrimitiveType(name=self.previous().value)
    
    # Array: [T]
    if self.match('LBRACKET'):
        element_type = self.parse_type_annotation()
        self.consume('RBRACKET')
        return ArrayType(element_type=element_type)
    
    # Tuple: (T, U, V)
    if self.check('LPAREN'):
        return self.parse_tuple_type()
    
    # Generic: Option<T>, Result<T, E>
    if self.check('IDENTIFIER'):
        name = self.consume('IDENTIFIER').value
        
        if self.match('LESS'):
            type_args = self.parse_type_arguments()
            return GenericType(name=name, type_args=type_args)
        
        return PrimitiveType(name=name)
    
    raise ParserError("Expected type annotation")
```

### Error Handling

El parser incluye manejo básico de errores:

```python
class ParserError(Exception):
    """Error durante parsing"""
    def __init__(self, message: str, token: Optional[Token] = None):
        self.message = message
        self.token = token
        super().__init__(self.message)

def consume(self, token_type: str, message: Optional[str] = None) -> Token:
    """Consume token esperado o lanza error"""
    if self.check(token_type):
        return self.advance()
    
    msg = message or f"Expected {token_type}, got {self.current_token.type}"
    raise ParserError(msg, self.current_token)

def synchronize(self):
    """Sincronizar después de error (panic mode)"""
    self.advance()
    
    while not self.is_at_end():
        # Sync en statement boundaries
        if self.previous().type == 'SEMICOLON':
            return
        
        # Sync en keywords de declaration
        if self.current_token.type in ['FN', 'STRUCT', 'CLASS', 'ENUM']:
            return
        
        self.advance()
```

## 📊 Cobertura de Construcciones

### ✅ Imports (6 tipos)
- `system:` - Sistema operativo y IO
- `package:` - Paquetes externos
- `module:` - Módulos internos
- `library:` - Librerías
- `extension:` - Extensiones
- `assets:` - Assets estáticos

### ✅ Declarations (20+ tipos)

**Standard:**
- Functions (normales, async, genéricas)
- Structs (con fields, genéricos)
- Enums (con datos asociados)
- Classes (con herencia, interfaces)
- Interfaces
- Type aliases

**Domain-Specific:**
- Service, Repository, Controller
- Entity, DTO, UseCase, ValueObject
- Model

**Design Patterns:**
- Factory, Builder, Strategy
- Observer, Singleton, Adapter, Decorator

### ✅ Statements (15+ tipos)
- Variables (inmutables, con state)
- Assignments
- If-else
- Match con patterns
- Try-catch-finally
- Return, Throw
- Expression statements

### ✅ Patterns (8 tipos)
- Literal, Identifier, Wildcard
- Tuple, Struct, Enum
- Or patterns, Range patterns

### ✅ Type Annotations (9 tipos)
- Primitivos: Number, Float, String, Bool
- Colecciones: Array, Tuple
- Funciones: (args) -> return
- Genéricos: Option<T>, Result<T, E>
- Union types: A | B

## 📁 Ubicación de Archivos

```
src/parser/parser.py             # Implementación (1,850+ líneas)
src/parser/__init__.py           # Exports (incluye Parser)
```

## ✅ Criterios de Aceptación

- [x] Parser completo con recursive descent
- [x] parse_program() como entry point
- [x] 6 tipos de imports con clauses
- [x] 20+ tipos de declarations
- [x] Variables con detección de 'state'
- [x] Control flow (if, match, try-catch)
- [x] Patterns para match (8 tipos)
- [x] Type annotations con generics
- [x] Integración con Pratt Parser para expresiones
- [x] Error handling básico
- [x] Sincronización en errores
- [x] Código committeado y versionado

## 🎓 Decisiones de Diseño

### 1. **Recursive Descent para Estructuras**
- Natural para gramáticas LL(1)
- Fácil de entender y mantener
- Un método por regla gramatical

### 2. **Delegación a Pratt Parser**
- Expresiones con precedencia → Pratt
- Estructuras de alto nivel → Recursive Descent
- Separación de concerns

### 3. **Detección de Keywords Prohibidas**
```python
# Parser NO reconoce estos keywords:
# - let, const, var
# - null, undefined, nil
# - for, while, loop
# - switch, case
# - export

# Estos se manejan en error_recovery.py
```

### 4. **Modificadores como Flags**
```python
is_public: bool   # public modifier
is_async: bool    # async modifier
is_mutable: bool  # state keyword
```

### 5. **Patterns Exhaustivos**
Match debe cubrir todos los casos:
```python
# Parser valida que último case sea:
# - Wildcard pattern (_)
# - O un pattern que cubra todos los casos
```

## 📊 Métricas

- **Total líneas:** 1,850+
- **Métodos principales:** ~30
- **Construcciones soportadas:** 50+
- **Commit:** 0be9b08

## 🔗 Referencias

- **Jira:** [VELA-568](https://velalang.atlassian.net/browse/VELA-568)
- **Historia:** [Sprint 6](../README.md)
- **Archivo:** `src/parser/parser.py`
- **Anterior:** [TASK-010: Estructura AST](./TASK-010.md)
- **Siguiente:** [TASK-009: Pratt Parser](./TASK-009.md)

---

**Autor:** GitHub Copilot Agent  
**Fecha:** 2025-11-30  
**Commit:** 0be9b08
