"""
Parser Recursive Descent para el lenguaje Vela

Implementación de: VELA-568 (TASK-008)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

⚠️ IMPORTANTE: Este es código Python del compilador de Vela

Este módulo implementa un parser recursive descent completo para Vela.
Consume tokens del lexer y genera un AST (Abstract Syntax Tree).

Arquitectura:
- Recursive Descent: cada regla gramatical = método
- Pratt Parsing: para expresiones con precedencia (ver pratt_parser.py)
- Error Recovery: estrategias de recuperación (ver error_recovery.py)

Ejemplo de uso:
```python
from lexer import Lexer
from parser import Parser

code = '''
fn add(a: Number, b: Number) -> Number {
  return a + b
}
'''

lexer = Lexer(code)
tokens = lexer.tokenize()
parser = Parser(tokens)
ast = parser.parse()
print(ast)
```
"""

from typing import List, Optional, Union, Callable
from dataclasses import dataclass
import sys

# Import del lexer (necesitamos los tokens)
try:
    # Try relative import (when running from src/)
    from ..lexer.token import Token, TokenType
    from ..lexer.lexer import Lexer
except ImportError:
    # Fallback to absolute import (when running tests)
    import sys
    sys.path.append('..')
    from src.lexer.token import Token, TokenType
    from src.lexer.lexer import Lexer

# Import de los nodos del AST
from .ast_nodes import *


# ===================================================================
# PARSER ERRORS
# ===================================================================

class ParserError(Exception):
    """Error base del parser"""
    def __init__(self, message: str, position: Position):
        self.message = message
        self.position = position
        super().__init__(f"{message} at {position}")


class UnexpectedTokenError(ParserError):
    """Token inesperado"""
    def __init__(self, expected: str, found: Token):
        message = f"Expected {expected}, found {found.type.value}"
        super().__init__(message, create_position(found.line, found.column))
        self.expected = expected
        self.found = found


class UnexpectedEOFError(ParserError):
    """EOF inesperado"""
    def __init__(self, expected: str):
        message = f"Unexpected end of file, expected {expected}"
        super().__init__(message, create_position(0, 0))


# ===================================================================
# PARSER
# ===================================================================

class Parser:
    """
    Parser Recursive Descent para Vela.
    
    Convierte una lista de tokens en un AST.
    
    Métodos principales:
    - parse() -> Program: Parsea un programa completo
    - parse_declaration() -> Declaration: Parsea una declaración
    - parse_statement() -> Statement: Parsea un statement
    - parse_expression() -> Expression: Parsea una expresión
    """
    
    def __init__(self, tokens: List[Token]):
        """
        Inicializa el parser.
        
        Args:
            tokens: Lista de tokens del lexer
        """
        self.tokens = tokens
        self.current = 0
        self.errors: List[ParserError] = []
    
    # ===============================================================
    # UTILITIES
    # ===============================================================
    
    def peek(self, offset: int = 0) -> Optional[Token]:
        """Obtiene el token actual + offset sin consumirlo"""
        index = self.current + offset
        if index < len(self.tokens):
            return self.tokens[index]
        return None
    
    def advance(self) -> Token:
        """Consume y retorna el token actual"""
        token = self.peek()
        if token:
            self.current += 1
        return token
    
    def check(self, *types: TokenType) -> bool:
        """Verifica si el token actual es de alguno de los tipos dados"""
        token = self.peek()
        if not token:
            return False
        return token.type in types
    
    def match(self, *types: TokenType) -> bool:
        """Si el token actual es de algún tipo dado, lo consume y retorna True"""
        if self.check(*types):
            self.advance()
            return True
        return False
    
    def expect(self, token_type: TokenType, message: str = None) -> Token:
        """
        Consume y retorna el token si es del tipo esperado.
        Si no, lanza error.
        """
        token = self.peek()
        if not token:
            raise UnexpectedEOFError(message or token_type.value)
        if token.type != token_type:
            raise UnexpectedTokenError(message or token_type.value, token)
        return self.advance()
    
    def is_at_end(self) -> bool:
        """Verifica si llegamos al final"""
        return self.peek() is None or self.check(TokenType.EOF)
    
    def synchronize(self):
        """
        Sincroniza el parser tras un error.
        Busca el próximo punto de sincronización (;, }, declaración, etc.)
        """
        self.advance()
        
        while not self.is_at_end():
            # Si encontramos punto y coma, estamos sincronizados
            if self.peek(-1).type == TokenType.SEMICOLON:
                return
            
            # Si encontramos keyword de declaración, estamos sincronizados
            if self.check(
                TokenType.FN, TokenType.STRUCT, TokenType.ENUM,
                TokenType.CLASS, TokenType.INTERFACE, TokenType.TYPE,
                TokenType.SERVICE, TokenType.REPOSITORY, TokenType.CONTROLLER
            ):
                return
            
            self.advance()
    
    def create_range_from_token(self, token: Token) -> Range:
        """Crea Range desde un token"""
        # Use lexeme length (always a string with len())
        token_length = len(token.lexeme)
        return Range(
            start=Position(token.line, token.column),
            end=Position(token.line, token.column + token_length)
        )
    
    def create_range_from_tokens(self, start: Token, end: Token) -> Range:
        """Crea Range desde dos tokens"""
        # Use lexeme length (always a string with len())
        end_length = len(end.lexeme)
        return Range(
            start=Position(start.line, start.column),
            end=Position(end.line, end.column + end_length)
        )
    
    # ===============================================================
    # MAIN PARSE METHOD
    # ===============================================================
    
    def parse(self) -> Program:
        """
        Parsea un programa completo.
        
        Grammar:
        ```
        program = import_declaration* declaration*
        ```
        
        Returns:
            Program: Nodo raíz del AST
        """
        start_token = self.peek() or Token(TokenType.EOF, "", 1, 1)
        imports: List[ImportDeclaration] = []
        declarations: List[Declaration] = []
        
        try:
            # Parse imports
            while self.check(TokenType.IMPORT):
                imports.append(self.parse_import())
            
            # Parse declarations
            while not self.is_at_end():
                try:
                    decl = self.parse_declaration()
                    if decl:
                        declarations.append(decl)
                except ParserError as e:
                    self.errors.append(e)
                    self.synchronize()
        
        except Exception as e:
            # Error inesperado
            print(f"Unexpected error during parsing: {e}")
            raise
        
        end_token = self.peek(-1) or start_token
        
        return Program(
            range=self.create_range_from_tokens(start_token, end_token),
            imports=imports,
            declarations=declarations
        )
    
    # ===============================================================
    # IMPORTS
    # ===============================================================
    
    def parse_import(self) -> ImportDeclaration:
        """
        Parsea import declaration.
        
        Grammar:
        ```
        import = 'import' STRING ('show' '{' identifier_list '}')? 
                                ('hide' '{' identifier_list '}')?
                                ('as' IDENTIFIER)?
        ```
        
        Examples:
        - import 'package:http'
        - import 'module:utils' show { sort, filter }
        - import 'library:math' as m
        """
        start = self.expect(TokenType.IMPORT)
        
        # Parse path string
        path_token = self.expect(TokenType.STRING, "import path")
        path = path_token.value.strip('"\'')
        
        # Determinar el kind desde el prefijo
        kind = ImportKind.MODULE  # Default
        if path.startswith("system:"):
            kind = ImportKind.SYSTEM
        elif path.startswith("package:"):
            kind = ImportKind.PACKAGE
        elif path.startswith("module:"):
            kind = ImportKind.MODULE
        elif path.startswith("library:"):
            kind = ImportKind.LIBRARY
        elif path.startswith("extension:"):
            kind = ImportKind.EXTENSION
        elif path.startswith("assets:"):
            kind = ImportKind.ASSETS
        
        # Parse 'show' clause
        show = None
        if self.match(TokenType.IDENTIFIER):
            if self.peek(-1).value == "show":
                self.expect(TokenType.LBRACE)
                show = self.parse_identifier_list()
                self.expect(TokenType.RBRACE)
        
        # Parse 'hide' clause
        hide = None
        if self.check(TokenType.IDENTIFIER) and self.peek().value == "hide":
            self.advance()
            self.expect(TokenType.LBRACE)
            hide = self.parse_identifier_list()
            self.expect(TokenType.RBRACE)
        
        # Parse 'as' alias
        alias = None
        if self.match(TokenType.AS):
            alias_token = self.expect(TokenType.IDENTIFIER)
            alias = alias_token.value
        
        end = self.peek(-1)
        
        return ImportDeclaration(
            range=self.create_range_from_tokens(start, end),
            kind=kind,
            path=path,
            alias=alias,
            show=show,
            hide=hide
        )
    
    def parse_identifier_list(self) -> List[str]:
        """Parse lista de identificadores separados por coma"""
        identifiers = []
        
        identifiers.append(self.expect(TokenType.IDENTIFIER).value)
        
        while self.match(TokenType.COMMA):
            identifiers.append(self.expect(TokenType.IDENTIFIER).value)
        
        return identifiers
    
    # ===============================================================
    # DECLARATIONS
    # ===============================================================
    
    def parse_decorators(self) -> List[Decorator]:
        """
        Parsea decoradores/annotations.
        
        Grammar:
        ```
        decorators = decorator+
        decorator = '@' IDENTIFIER ('(' arguments? ')')?
        ```
        
        Ejemplos en Vela:
        ```vela
        @injectable
        @controller("/api/users")
        @get("/profile")
        @module({
          declarations: [UserService],
          exports: [UserService]
        })
        ```
        """
        decorators = []
        
        while self.check(TokenType.AT):
            start = self.advance()  # Consume '@'
            
            # Expect identifier (decorator name)
            name_token = self.expect(TokenType.IDENTIFIER, "decorator name")
            name = name_token.value
            
            # Parse arguments if present
            arguments = []
            if self.match(TokenType.LPAREN):
                # Parse arguments (expressions)
                if not self.check(TokenType.RPAREN):
                    arguments.append(self.parse_expression())
                    
                    while self.match(TokenType.COMMA):
                        arguments.append(self.parse_expression())
                
                self.expect(TokenType.RPAREN, "')'")
            
            end = self.peek(-1)
            decorator = Decorator(
                name=name,
                arguments=arguments,
                range=self.create_range_from_tokens(start, end)
            )
            decorators.append(decorator)
        
        return decorators
    
    def parse_object_literal(self) -> StructLiteral:
        """
        Parsea object literal para metadata de decoradores.
        
        Grammar:
        ```
        object_literal = '{' (property (',' property)*)? '}'
        property = IDENTIFIER ':' expression
        ```
        
        Ejemplos en Vela:
        ```vela
        { declarations: [UserService, ProductService] }
        { path: "/api/users", method: "GET" }
        { scope: "singleton" }
        { min: 5, max: 100 }
        ```
        
        Returns:
            StructLiteral con fields representando las propiedades del objeto
        """
        start = self.expect(TokenType.LBRACE, "'{'")
        
        fields = []
        
        # Parse properties
        if not self.check(TokenType.RBRACE):
            # First property
            prop_name = self.expect(TokenType.IDENTIFIER, "property name").value
            self.expect(TokenType.COLON, "':'")
            prop_value = self.parse_expression()
            
            fields.append(StructLiteralField(
                name=prop_name,
                value=prop_value,
                range=self.create_range_from_tokens(start, self.peek(-1))
            ))
            
            # Additional properties
            while self.match(TokenType.COMMA):
                # Allow trailing comma
                if self.check(TokenType.RBRACE):
                    break
                
                prop_name = self.expect(TokenType.IDENTIFIER, "property name").value
                self.expect(TokenType.COLON, "':'")
                prop_value = self.parse_expression()
                
                fields.append(StructLiteralField(
                    name=prop_name,
                    value=prop_value,
                    range=self.create_range_from_tokens(start, self.peek(-1))
                ))
        
        end = self.expect(TokenType.RBRACE, "'}'")
        
        return StructLiteral(
            struct_name="",  # Anonymous object literal
            fields=fields,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_declaration(self) -> Optional[Declaration]:
        """
        Parsea una declaración de nivel superior.
        
        Puede ser: function, struct, enum, class, interface, type alias,
        o keywords específicos (service, repository, controller, etc.)
        """
        # Parse decorators if present
        decorators = self.parse_decorators()
        
        # Check for 'public' modifier
        is_public = self.match(TokenType.PUBLIC)
        
        # Function
        if self.check(TokenType.FN) or self.check(TokenType.ASYNC):
            return self.parse_function_declaration(is_public)
        
        # Struct
        if self.check(TokenType.STRUCT):
            return self.parse_struct_declaration(is_public)
        
        # Enum
        if self.check(TokenType.ENUM):
            return self.parse_enum_declaration(is_public)
        
        # Class
        if self.check(TokenType.CLASS):
            return self.parse_class_declaration(is_public)
        
        # Interface
        if self.check(TokenType.INTERFACE):
            return self.parse_interface_declaration(is_public)
        
        # Type Alias
        if self.check(TokenType.TYPE):
            return self.parse_type_alias_declaration(is_public)
        
        # Domain-Specific Keywords
        if self.check(TokenType.SERVICE):
            return self.parse_service_declaration(is_public)
        
        if self.check(TokenType.REPOSITORY):
            return self.parse_repository_declaration(is_public)
        
        if self.check(TokenType.CONTROLLER):
            return self.parse_controller_declaration(is_public)
        
        if self.check(TokenType.USECASE):
            return self.parse_usecase_declaration(is_public)
        
        if self.check(TokenType.ENTITY):
            return self.parse_entity_declaration(is_public)
        
        if self.check(TokenType.VALUE_OBJECT):
            return self.parse_value_object_declaration(is_public)
        
        if self.check(TokenType.DTO):
            return self.parse_dto_declaration(is_public)
        
        # UI Keywords
        if self.check(TokenType.WIDGET):
            return self.parse_widget_declaration(is_public)
        
        if self.check(TokenType.COMPONENT):
            return self.parse_component_declaration(is_public)
        
        # Models
        if self.check(TokenType.MODEL):
            return self.parse_model_declaration(is_public)
        
        # Design Patterns
        if self.check(TokenType.FACTORY):
            return self.parse_factory_declaration(is_public)
        
        if self.check(TokenType.BUILDER):
            return self.parse_builder_declaration(is_public)
        
        if self.check(TokenType.STRATEGY):
            return self.parse_strategy_declaration(is_public)
        
        if self.check(TokenType.OBSERVER):
            return self.parse_observer_declaration(is_public)
        
        if self.check(TokenType.SINGLETON):
            return self.parse_singleton_declaration(is_public)
        
        if self.check(TokenType.ADAPTER):
            return self.parse_adapter_declaration(is_public)
        
        if self.check(TokenType.DECORATOR):
            return self.parse_decorator_declaration(is_public)
        
        # Web/API
        if self.check(TokenType.GUARD):
            return self.parse_guard_declaration(is_public)
        
        if self.check(TokenType.MIDDLEWARE):
            return self.parse_middleware_declaration(is_public)
        
        if self.check(TokenType.INTERCEPTOR):
            return self.parse_interceptor_declaration(is_public)
        
        if self.check(TokenType.VALIDATOR):
            return self.parse_validator_declaration(is_public)
        
        # State & DI
        if self.check(TokenType.STORE):
            return self.parse_store_declaration(is_public)
        
        if self.check(TokenType.PROVIDER):
            return self.parse_provider_declaration(is_public)
        
        # Concurrency
        if self.check(TokenType.ACTOR):
            return self.parse_actor_declaration(is_public)
        
        # Utilities
        if self.check(TokenType.PIPE_KEYWORD):
            return self.parse_pipe_declaration(is_public)
        
        if self.check(TokenType.TASK):
            return self.parse_task_declaration(is_public)
        
        if self.check(TokenType.HELPER):
            return self.parse_helper_declaration(is_public)
        
        if self.check(TokenType.MAPPER):
            return self.parse_mapper_declaration(is_public)
        
        if self.check(TokenType.SERIALIZER):
            return self.parse_serializer_declaration(is_public)
        
        # Module System (Angular-style)
        if self.check(TokenType.MODULE):
            return self.parse_module_declaration(is_public, decorators)
        
        # Unknown
        token = self.peek()
        if token:
            raise UnexpectedTokenError("declaration", token)
        return None
    
    def parse_function_declaration(self, is_public: bool = False) -> FunctionDeclaration:
        """
        Parsea function declaration.
        
        Grammar:
        ```
        function = 'async'? 'fn' IDENTIFIER generic_params? '(' parameters? ')' 
                   ('->' type_annotation)? block
        ```
        """
        start = self.peek()
        is_async = self.match(TokenType.ASYNC)
        
        self.expect(TokenType.FN)
        name_token = self.expect(TokenType.IDENTIFIER)
        name = name_token.value
        
        # Generic parameters
        generic_params = []
        if self.match(TokenType.LT):
            generic_params = self.parse_generic_parameters()
            self.expect(TokenType.GT)
        
        # Parameters
        self.expect(TokenType.LPAREN)
        parameters = []
        if not self.check(TokenType.RPAREN):
            parameters = self.parse_parameters()
        self.expect(TokenType.RPAREN)
        
        # Return type
        return_type = None
        if self.match(TokenType.ARROW):
            return_type = self.parse_type_annotation()
        
        # Body
        body = self.parse_block_statement()
        
        end = self.peek(-1)
        
        return FunctionDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            parameters=parameters,
            return_type=return_type,
            body=body,
            is_async=is_async,
            generic_params=generic_params
        )
    
    def parse_parameters(self) -> List[Parameter]:
        """Parse lista de parámetros"""
        parameters = []
        
        parameters.append(self.parse_parameter())
        
        while self.match(TokenType.COMMA):
            parameters.append(self.parse_parameter())
        
        return parameters
    
    def parse_parameter(self) -> Parameter:
        """Parse un parámetro"""
        start = self.peek()
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Type annotation
        type_annotation = None
        if self.match(TokenType.COLON):
            type_annotation = self.parse_type_annotation()
        
        # Default value
        default_value = None
        if self.match(TokenType.ASSIGN):
            default_value = self.parse_expression()
        
        end = self.peek(-1)
        
        return Parameter(
            name=name,
            type_annotation=type_annotation,
            default_value=default_value,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_generic_parameters(self) -> List[GenericParameter]:
        """Parse lista de parámetros genéricos"""
        params = []
        
        params.append(self.parse_generic_parameter())
        
        while self.match(TokenType.COMMA):
            params.append(self.parse_generic_parameter())
        
        return params
    
    def parse_generic_parameter(self) -> GenericParameter:
        """Parse un parámetro genérico"""
        start = self.peek()
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Constraints (T: Trait1 + Trait2)
        constraints = []
        if self.match(TokenType.COLON):
            constraints.append(self.parse_type_annotation())
            while self.match(TokenType.PLUS):
                constraints.append(self.parse_type_annotation())
        
        end = self.peek(-1)
        
        return GenericParameter(
            name=name,
            constraints=constraints,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_struct_declaration(self, is_public: bool = False) -> StructDeclaration:
        """
        Parsea struct declaration.
        
        Grammar:
        ```
        struct = 'struct' IDENTIFIER generic_params? '{' struct_field* '}'
        ```
        """
        start = self.expect(TokenType.STRUCT)
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Generic parameters
        generic_params = []
        if self.match(TokenType.LT):
            generic_params = self.parse_generic_parameters()
            self.expect(TokenType.GT)
        
        # Fields
        self.expect(TokenType.LBRACE)
        fields = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            fields.append(self.parse_struct_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return StructDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields,
            generic_params=generic_params
        )
    
    def parse_struct_field(self) -> StructField:
        """Parse campo de struct"""
        start = self.peek()
        
        # Check for 'public' modifier
        is_public = True
        if self.match(TokenType.PRIVATE):
            is_public = False
        
        name = self.expect(TokenType.IDENTIFIER).value
        self.expect(TokenType.COLON)
        type_annotation = self.parse_type_annotation()
        
        end = self.peek(-1)
        
        return StructField(
            name=name,
            type_annotation=type_annotation,
            is_public=is_public,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_enum_declaration(self, is_public: bool = False) -> EnumDeclaration:
        """Parsea enum declaration"""
        start = self.expect(TokenType.ENUM)
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Generic parameters
        generic_params = []
        if self.match(TokenType.LT):
            generic_params = self.parse_generic_parameters()
            self.expect(TokenType.GT)
        
        # Variants
        self.expect(TokenType.LBRACE)
        variants = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            variants.append(self.parse_enum_variant())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return EnumDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            variants=variants,
            generic_params=generic_params
        )
    
    def parse_enum_variant(self) -> EnumVariant:
        """Parse variante de enum"""
        start = self.peek()
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Associated data
        fields = None
        if self.match(TokenType.LPAREN):
            fields = []
            if not self.check(TokenType.RPAREN):
                fields = [self.parse_struct_field()]
                while self.match(TokenType.COMMA):
                    fields.append(self.parse_struct_field())
            self.expect(TokenType.RPAREN)
        
        end = self.peek(-1)
        
        return EnumVariant(
            name=name,
            fields=fields,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_class_declaration(self, is_public: bool = False) -> ClassDeclaration:
        """Parsea class declaration"""
        start = self.expect(TokenType.CLASS)
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Generic parameters
        generic_params = []
        if self.match(TokenType.LT):
            generic_params = self.parse_generic_parameters()
            self.expect(TokenType.GT)
        
        # Extends
        extends = None
        if self.match(TokenType.EXTENDS):
            extends = self.expect(TokenType.IDENTIFIER).value
        
        # Implements
        implements = []
        if self.match(TokenType.IMPLEMENTS):
            implements.append(self.expect(TokenType.IDENTIFIER).value)
            while self.match(TokenType.COMMA):
                implements.append(self.expect(TokenType.IDENTIFIER).value)
        
        # Body
        self.expect(TokenType.LBRACE)
        
        constructor = None
        fields = []
        methods = []
        
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            # Constructor
            if self.check(TokenType.CONSTRUCTOR):
                constructor = self.parse_constructor()
            # Method or field
            else:
                # Try to parse as method or field
                member_start = self.current
                try:
                    if self.check(TokenType.FN) or self.check(TokenType.ASYNC):
                        methods.append(self.parse_method())
                    else:
                        fields.append(self.parse_class_field())
                except:
                    # Si falla, intentar continuar
                    self.current = member_start + 1
        
        self.expect(TokenType.RBRACE)
        end = self.peek(-1)
        
        return ClassDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            constructor=constructor,
            fields=fields,
            methods=methods,
            extends=extends,
            implements=implements,
            generic_params=generic_params
        )
    
    def parse_constructor(self) -> ConstructorDeclaration:
        """Parse constructor"""
        start = self.expect(TokenType.CONSTRUCTOR)
        
        self.expect(TokenType.LPAREN)
        parameters = []
        if not self.check(TokenType.RPAREN):
            parameters = self.parse_parameters()
        self.expect(TokenType.RPAREN)
        
        body = self.parse_block_statement()
        end = self.peek(-1)
        
        return ConstructorDeclaration(
            parameters=parameters,
            body=body,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_class_field(self) -> ClassField:
        """Parse campo de clase"""
        start = self.peek()
        
        # Modifiers
        is_public = True
        is_protected = False
        is_private = False
        is_state = False
        
        if self.match(TokenType.PUBLIC):
            is_public = True
        elif self.match(TokenType.PROTECTED):
            is_protected = True
        elif self.match(TokenType.PRIVATE):
            is_private = True
        
        if self.match(TokenType.STATE):
            is_state = True
        
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Type annotation
        type_annotation = None
        if self.match(TokenType.COLON):
            type_annotation = self.parse_type_annotation()
        
        # Initial value
        initial_value = None
        if self.match(TokenType.ASSIGN):
            initial_value = self.parse_expression()
        
        end = self.peek(-1)
        
        return ClassField(
            name=name,
            type_annotation=type_annotation,
            is_state=is_state,
            initial_value=initial_value,
            is_public=is_public,
            is_protected=is_protected,
            is_private=is_private,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_method(self) -> MethodDeclaration:
        """Parse método de clase"""
        start = self.peek()
        
        # Modifiers
        is_public = True
        is_protected = False
        is_private = False
        is_override = False
        is_async = False
        
        if self.match(TokenType.PUBLIC):
            is_public = True
        elif self.match(TokenType.PROTECTED):
            is_protected = True
        elif self.match(TokenType.PRIVATE):
            is_private = True
        
        if self.match(TokenType.OVERRIDE):
            is_override = True
        
        if self.match(TokenType.ASYNC):
            is_async = True
        
        self.expect(TokenType.FN)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LPAREN)
        parameters = []
        if not self.check(TokenType.RPAREN):
            parameters = self.parse_parameters()
        self.expect(TokenType.RPAREN)
        
        # Return type
        return_type = None
        if self.match(TokenType.ARROW):
            return_type = self.parse_type_annotation()
        
        body = self.parse_block_statement()
        end = self.peek(-1)
        
        return MethodDeclaration(
            name=name,
            parameters=parameters,
            return_type=return_type,
            body=body,
            is_async=is_async,
            is_override=is_override,
            is_public=is_public,
            is_protected=is_protected,
            is_private=is_private,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_interface_declaration(self, is_public: bool = False) -> InterfaceDeclaration:
        """Parsea interface declaration"""
        start = self.expect(TokenType.INTERFACE)
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Generic parameters
        generic_params = []
        if self.match(TokenType.LT):
            generic_params = self.parse_generic_parameters()
            self.expect(TokenType.GT)
        
        # Methods
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_function_signature())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return InterfaceDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods,
            generic_params=generic_params
        )
    
    def parse_function_signature(self) -> FunctionSignature:
        """Parse firma de función (sin body)"""
        start = self.peek()
        is_async = self.match(TokenType.ASYNC)
        
        self.expect(TokenType.FN)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LPAREN)
        parameters = []
        if not self.check(TokenType.RPAREN):
            parameters = self.parse_parameters()
        self.expect(TokenType.RPAREN)
        
        # Return type
        return_type = None
        if self.match(TokenType.ARROW):
            return_type = self.parse_type_annotation()
        
        end = self.peek(-1)
        
        return FunctionSignature(
            name=name,
            parameters=parameters,
            return_type=return_type,
            is_async=is_async,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_type_alias_declaration(self, is_public: bool = False) -> TypeAliasDeclaration:
        """Parsea type alias"""
        start = self.expect(TokenType.TYPE)
        name = self.expect(TokenType.IDENTIFIER).value
        self.expect(TokenType.ASSIGN)
        type_annotation = self.parse_type_annotation()
        
        end = self.peek(-1)
        
        return TypeAliasDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            type_annotation=type_annotation
        )
    
    # Domain-Specific Declarations (simplificados para este parser)
    
    def parse_service_declaration(self, is_public: bool = False) -> ServiceDeclaration:
        """Parsea service declaration"""
        start = self.expect(TokenType.SERVICE)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ServiceDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_repository_declaration(self, is_public: bool = False) -> RepositoryDeclaration:
        """Parsea repository declaration"""
        start = self.expect(TokenType.REPOSITORY)
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Entity type
        entity_type = "Entity"
        if self.match(TokenType.LT):
            entity_type = self.expect(TokenType.IDENTIFIER).value
            self.expect(TokenType.GT)
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return RepositoryDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            entity_type=entity_type,
            methods=methods
        )
    
    def parse_controller_declaration(self, is_public: bool = False) -> ControllerDeclaration:
        """Parsea controller declaration"""
        start = self.expect(TokenType.CONTROLLER)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        routes = []
        # Simplificado: parsear métodos como routes
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ControllerDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            routes=routes
        )
    
    def parse_usecase_declaration(self, is_public: bool = False) -> UseCaseDeclaration:
        """Parsea usecase declaration"""
        start = self.expect(TokenType.USECASE)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        # Buscar método execute
        execute_method = None
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            method = self.parse_method()
            if method.name == "execute":
                execute_method = method
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return UseCaseDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            execute_method=execute_method
        )
    
    def parse_entity_declaration(self, is_public: bool = False) -> EntityDeclaration:
        """Parsea entity declaration"""
        start = self.expect(TokenType.ENTITY)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        id_field = None
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            field = self.parse_struct_field()
            fields.append(field)
            if field.name == "id":
                id_field = field
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return EntityDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            id_field=id_field,
            fields=fields
        )
    
    def parse_value_object_declaration(self, is_public: bool = False) -> ValueObjectDeclaration:
        """Parsea valueObject declaration"""
        start = self.expect(TokenType.VALUE_OBJECT)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            fields.append(self.parse_struct_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ValueObjectDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields
        )
    
    def parse_dto_declaration(self, is_public: bool = False) -> DTODeclaration:
        """Parsea dto declaration"""
        start = self.expect(TokenType.DTO)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            fields.append(self.parse_struct_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return DTODeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields
        )
    
    # ===============================================================
    # UI KEYWORDS
    # ===============================================================
    
    def parse_widget_declaration(self, is_public: bool = False) -> WidgetDeclaration:
        """Parsea widget declaration (StatefulWidget)"""
        start = self.expect(TokenType.WIDGET)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            if self.check(TokenType.FN) or self.check(TokenType.ASYNC):
                methods.append(self.parse_method())
            else:
                fields.append(self.parse_class_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return WidgetDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields,
            methods=methods
        )
    
    def parse_component_declaration(self, is_public: bool = False) -> ComponentDeclaration:
        """Parsea component declaration"""
        start = self.expect(TokenType.COMPONENT)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            if self.check(TokenType.FN) or self.check(TokenType.ASYNC):
                methods.append(self.parse_method())
            else:
                fields.append(self.parse_class_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ComponentDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields,
            methods=methods
        )
    
    # ===============================================================
    # MODEL KEYWORDS
    # ===============================================================
    
    def parse_model_declaration(self, is_public: bool = False) -> ModelDeclaration:
        """Parsea model declaration"""
        start = self.expect(TokenType.MODEL)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            fields.append(self.parse_struct_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ModelDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields
        )
    
    # ===============================================================
    # DESIGN PATTERN KEYWORDS
    # ===============================================================
    
    def parse_factory_declaration(self, is_public: bool = False) -> FactoryDeclaration:
        """Parsea factory declaration"""
        start = self.expect(TokenType.FACTORY)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return FactoryDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_builder_declaration(self, is_public: bool = False) -> BuilderDeclaration:
        """Parsea builder declaration"""
        start = self.expect(TokenType.BUILDER)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return BuilderDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_strategy_declaration(self, is_public: bool = False) -> StrategyDeclaration:
        """Parsea strategy declaration"""
        start = self.expect(TokenType.STRATEGY)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return StrategyDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_observer_declaration(self, is_public: bool = False) -> ObserverDeclaration:
        """Parsea observer declaration"""
        start = self.expect(TokenType.OBSERVER)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ObserverDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_singleton_declaration(self, is_public: bool = False) -> SingletonDeclaration:
        """Parsea singleton declaration"""
        start = self.expect(TokenType.SINGLETON)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            if self.check(TokenType.FN) or self.check(TokenType.ASYNC):
                methods.append(self.parse_method())
            else:
                fields.append(self.parse_class_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return SingletonDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields,
            methods=methods
        )
    
    def parse_adapter_declaration(self, is_public: bool = False) -> AdapterDeclaration:
        """Parsea adapter declaration"""
        start = self.expect(TokenType.ADAPTER)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return AdapterDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_decorator_declaration(self, is_public: bool = False) -> DecoratorDeclaration:
        """Parsea decorator declaration"""
        start = self.expect(TokenType.DECORATOR)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return DecoratorDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    # ===============================================================
    # WEB/API KEYWORDS
    # ===============================================================
    
    def parse_guard_declaration(self, is_public: bool = False) -> GuardDeclaration:
        """Parsea guard declaration"""
        start = self.expect(TokenType.GUARD)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return GuardDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_middleware_declaration(self, is_public: bool = False) -> MiddlewareDeclaration:
        """Parsea middleware declaration"""
        start = self.expect(TokenType.MIDDLEWARE)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return MiddlewareDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_interceptor_declaration(self, is_public: bool = False) -> InterceptorDeclaration:
        """Parsea interceptor declaration"""
        start = self.expect(TokenType.INTERCEPTOR)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return InterceptorDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_validator_declaration(self, is_public: bool = False) -> ValidatorDeclaration:
        """Parsea validator declaration"""
        start = self.expect(TokenType.VALIDATOR)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ValidatorDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    # ===============================================================
    # STATE & DI KEYWORDS
    # ===============================================================
    
    def parse_store_declaration(self, is_public: bool = False) -> StoreDeclaration:
        """Parsea store declaration"""
        start = self.expect(TokenType.STORE)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            if self.check(TokenType.FN) or self.check(TokenType.ASYNC):
                methods.append(self.parse_method())
            else:
                fields.append(self.parse_class_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return StoreDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields,
            methods=methods
        )
    
    def parse_provider_declaration(self, is_public: bool = False) -> ProviderDeclaration:
        """Parsea provider declaration"""
        start = self.expect(TokenType.PROVIDER)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ProviderDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    # ===============================================================
    # CONCURRENCY KEYWORDS
    # ===============================================================
    
    def parse_actor_declaration(self, is_public: bool = False) -> ActorDeclaration:
        """Parsea actor declaration"""
        start = self.expect(TokenType.ACTOR)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        fields = []
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            if self.check(TokenType.FN) or self.check(TokenType.ASYNC):
                methods.append(self.parse_method())
            else:
                fields.append(self.parse_class_field())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return ActorDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            fields=fields,
            methods=methods
        )
    
    # ===============================================================
    # UTILITY KEYWORDS
    # ===============================================================
    
    def parse_pipe_declaration(self, is_public: bool = False) -> PipeDeclaration:
        """Parsea pipe declaration"""
        start = self.expect(TokenType.PIPE_KEYWORD)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return PipeDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_task_declaration(self, is_public: bool = False) -> TaskDeclaration:
        """Parsea task declaration"""
        start = self.expect(TokenType.TASK)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return TaskDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_helper_declaration(self, is_public: bool = False) -> HelperDeclaration:
        """Parsea helper declaration"""
        start = self.expect(TokenType.HELPER)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return HelperDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_mapper_declaration(self, is_public: bool = False) -> MapperDeclaration:
        """Parsea mapper declaration"""
        start = self.expect(TokenType.MAPPER)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return MapperDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_serializer_declaration(self, is_public: bool = False) -> SerializerDeclaration:
        """Parsea serializer declaration"""
        start = self.expect(TokenType.SERIALIZER)
        name = self.expect(TokenType.IDENTIFIER).value
        
        self.expect(TokenType.LBRACE)
        methods = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            methods.append(self.parse_method())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return SerializerDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            methods=methods
        )
    
    def parse_module_declaration(self, is_public: bool = False, decorators: List[Decorator] = None) -> ModuleDeclaration:
        """
        Parsea module declaration (Angular-style).
        
        Grammar:
        ```
        module = decorator* 'module' IDENTIFIER '{' declaration* '}'
        decorator = '@module' '(' metadata_object ')'
        ```
        
        Ejemplo en Vela:
        ```vela
        @module({
          declarations: [AuthService, LoginWidget],
          exports: [AuthService],
          providers: [AuthService, TokenService],
          imports: [HttpModule, CryptoModule]
        })
        module AuthModule {
          # Módulo NO instanciable (NO constructor, NO new AuthModule())
        }
        ```
        
        Reglas de Validación (se validan en semantic analysis):
        1. DEBE tener decorador @module({ ... })
        2. exports ⊆ declarations (exports debe ser subconjunto de declarations)
        3. providers ⊆ declarations
        4. NO instanciable (NO constructor, NO new)
        """
        start = self.expect(TokenType.MODULE)
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Parse body (declarations dentro del módulo)
        self.expect(TokenType.LBRACE)
        body = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            decl = self.parse_declaration()
            if decl:
                body.append(decl)
        end = self.expect(TokenType.RBRACE)
        
        # Extraer metadata del decorador @module (si existe)
        # La validación completa (exports ⊆ declarations) se hace en semantic analysis
        declarations_list = []
        exports_list = []
        providers_list = []
        imports_list = []
        
        if decorators:
            for decorator in decorators:
                if decorator.name == "module" and len(decorator.arguments) > 0:
                    # Extraer metadata del object literal
                    metadata_obj = decorator.arguments[0]
                    
                    if isinstance(metadata_obj, StructLiteral):
                        for field in metadata_obj.fields:
                            if field.name == "declarations" and isinstance(field.value, ArrayLiteral):
                                declarations_list = field.value.elements
                            elif field.name == "exports" and isinstance(field.value, ArrayLiteral):
                                exports_list = field.value.elements
                            elif field.name == "providers" and isinstance(field.value, ArrayLiteral):
                                providers_list = field.value.elements
                            elif field.name == "imports" and isinstance(field.value, ArrayLiteral):
                                imports_list = field.value.elements
        
        return ModuleDeclaration(
            range=self.create_range_from_tokens(start, end),
            is_public=is_public,
            name=name,
            decorators=decorators or [],
            body=body,
            declarations=declarations_list,
            exports=exports_list,
            providers=providers_list,
            imports=imports_list
        )
    
    # ===============================================================
    # STATEMENTS
    # ===============================================================
    
    def parse_statement(self) -> Statement:
        """
        Parsea un statement.
        
        Puede ser: block, return, if, match, try, variable declaration, etc.
        """
        # Block
        if self.check(TokenType.LBRACE):
            return self.parse_block_statement()
        
        # Return
        if self.check(TokenType.RETURN):
            return self.parse_return_statement()
        
        # If
        if self.check(TokenType.IF):
            return self.parse_if_statement()
        
        # Match
        if self.check(TokenType.MATCH):
            return self.parse_match_statement()
        
        # Try
        if self.check(TokenType.TRY):
            return self.parse_try_statement()
        
        # Throw
        if self.check(TokenType.THROW):
            return self.parse_throw_statement()
        
        # Event System (TASK-035M)
        if self.check(TokenType.ON):
            return self.parse_on_statement()
        
        if self.check(TokenType.EMIT):
            return self.parse_emit_statement()
        
        if self.check(TokenType.OFF):
            return self.parse_off_statement()
        
        # Variable declaration o assignment
        if self.check(TokenType.STATE) or (self.check(TokenType.IDENTIFIER) and self.peek(1) and self.peek(1).type == TokenType.COLON):
            return self.parse_variable_declaration()
        
        # Assignment o expression statement
        return self.parse_expression_statement()
    
    def parse_block_statement(self) -> BlockStatement:
        """Parsea block statement"""
        start = self.expect(TokenType.LBRACE)
        
        statements = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            statements.append(self.parse_statement())
        
        end = self.expect(TokenType.RBRACE)
        
        return BlockStatement(
            range=self.create_range_from_tokens(start, end),
            statements=statements
        )
    
    def parse_return_statement(self) -> ReturnStatement:
        """Parsea return statement"""
        start = self.expect(TokenType.RETURN)
        
        value = None
        if not self.check(TokenType.SEMICOLON) and not self.check(TokenType.RBRACE):
            value = self.parse_expression()
        
        end = self.peek(-1)
        
        return ReturnStatement(
            range=self.create_range_from_tokens(start, end),
            value=value
        )
    
    def parse_if_statement(self) -> IfStatement:
        """Parsea if statement"""
        start = self.expect(TokenType.IF)
        
        condition = self.parse_expression()
        then_branch = self.parse_statement()
        
        else_branch = None
        if self.match(TokenType.ELSE):
            else_branch = self.parse_statement()
        
        end = self.peek(-1)
        
        return IfStatement(
            range=self.create_range_from_tokens(start, end),
            condition=condition,
            then_branch=then_branch,
            else_branch=else_branch
        )
    
    def parse_match_statement(self) -> MatchStatement:
        """Parsea match statement"""
        start = self.expect(TokenType.MATCH)
        
        value = self.parse_expression()
        
        self.expect(TokenType.LBRACE)
        arms = []
        while not self.check(TokenType.RBRACE) and not self.is_at_end():
            arms.append(self.parse_match_arm())
        self.expect(TokenType.RBRACE)
        
        end = self.peek(-1)
        
        return MatchStatement(
            range=self.create_range_from_tokens(start, end),
            value=value,
            arms=arms
        )
    
    def parse_match_arm(self) -> MatchArm:
        """Parse match arm"""
        start = self.peek()
        
        pattern = self.parse_pattern()
        
        # Guard
        guard = None
        if self.check(TokenType.IF):
            self.advance()
            guard = self.parse_expression()
        
        self.expect(TokenType.ARROW)  # =>
        body = self.parse_statement()
        
        end = self.peek(-1)
        
        return MatchArm(
            pattern=pattern,
            guard=guard,
            body=body,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_try_statement(self) -> TryStatement:
        """Parsea try statement"""
        start = self.expect(TokenType.TRY)
        
        try_block = self.parse_block_statement()
        
        # Catch clauses
        catch_clauses = []
        while self.match(TokenType.CATCH):
            catch_clauses.append(self.parse_catch_clause())
        
        # Finally
        finally_block = None
        if self.match(TokenType.FINALLY):
            finally_block = self.parse_block_statement()
        
        end = self.peek(-1)
        
        return TryStatement(
            range=self.create_range_from_tokens(start, end),
            try_block=try_block,
            catch_clauses=catch_clauses,
            finally_block=finally_block
        )
    
    def parse_catch_clause(self) -> CatchClause:
        """Parse catch clause"""
        start = self.peek()
        
        self.expect(TokenType.LPAREN)
        exception_name = self.expect(TokenType.IDENTIFIER).value
        
        exception_type = None
        if self.match(TokenType.COLON):
            exception_type = self.parse_type_annotation()
        
        self.expect(TokenType.RPAREN)
        
        body = self.parse_block_statement()
        end = self.peek(-1)
        
        return CatchClause(
            exception_name=exception_name,
            exception_type=exception_type,
            body=body,
            range=self.create_range_from_tokens(start, end)
        )
    
    def parse_throw_statement(self) -> ThrowStatement:
        """Parsea throw statement"""
        start = self.expect(TokenType.THROW)
        exception = self.parse_expression()
        end = self.peek(-1)
        
        return ThrowStatement(
            range=self.create_range_from_tokens(start, end),
            exception=exception
        )
    
    # ===============================================================
    # EVENT SYSTEM STATEMENTS (TASK-035M)
    # ===============================================================
    
    def parse_on_statement(self) -> EventOnStatement:
        """
        Parsea on statement: on(event_type, handler) o on<T>(event_type, handler)
        
        Sintaxis:
        ```vela
        on("user.created", handleUserCreated)
        on<UserEvent>("user.updated", (event) => { ... })
        ```
        """
        start = self.expect(TokenType.ON)
        
        # Type parameter opcional: on<T>(...)
        type_param = None
        if self.match(TokenType.LESS):
            type_param = self.parse_type_annotation()
            self.expect(TokenType.GREATER)
        
        # Expect opening paren
        self.expect(TokenType.LPAREN)
        
        # Event type expression (usualmente string literal)
        event_type = self.parse_expression()
        
        # Expect comma
        self.expect(TokenType.COMMA)
        
        # Handler expression (function reference o lambda)
        handler = self.parse_expression()
        
        # Expect closing paren
        self.expect(TokenType.RPAREN)
        end = self.peek(-1)
        
        return EventOnStatement(
            range=self.create_range_from_tokens(start, end),
            event_type=event_type,
            handler=handler,
            type_param=type_param
        )
    
    def parse_emit_statement(self) -> EventEmitStatement:
        """
        Parsea emit statement: emit(event_type) o emit(event_type, payload)
        
        Sintaxis:
        ```vela
        emit("app.started")
        emit("user.created", user)
        emit("notification", { message: "Hello", level: "info" })
        ```
        """
        start = self.expect(TokenType.EMIT)
        
        # Expect opening paren
        self.expect(TokenType.LPAREN)
        
        # Event type expression
        event_type = self.parse_expression()
        
        # Payload opcional
        payload = None
        if self.match(TokenType.COMMA):
            payload = self.parse_expression()
        
        # Expect closing paren
        self.expect(TokenType.RPAREN)
        end = self.peek(-1)
        
        return EventEmitStatement(
            range=self.create_range_from_tokens(start, end),
            event_type=event_type,
            payload=payload
        )
    
    def parse_off_statement(self) -> EventOffStatement:
        """
        Parsea off statement: off(event_type) o off(event_type, handler)
        
        Sintaxis:
        ```vela
        off("user.created")  # Remover todos los listeners
        off("user.created", handleUserCreated)  # Remover listener específico
        ```
        """
        start = self.expect(TokenType.OFF)
        
        # Expect opening paren
        self.expect(TokenType.LPAREN)
        
        # Event type expression
        event_type = self.parse_expression()
        
        # Handler opcional
        handler = None
        if self.match(TokenType.COMMA):
            handler = self.parse_expression()
        
        # Expect closing paren
        self.expect(TokenType.RPAREN)
        end = self.peek(-1)
        
        return EventOffStatement(
            range=self.create_range_from_tokens(start, end),
            event_type=event_type,
            handler=handler
        )
    
    def parse_variable_declaration(self) -> VariableDeclaration:
        """Parsea variable declaration"""
        start = self.peek()
        
        is_state = self.match(TokenType.STATE)
        
        name = self.expect(TokenType.IDENTIFIER).value
        
        # Type annotation
        type_annotation = None
        if self.match(TokenType.COLON):
            type_annotation = self.parse_type_annotation()
        
        # Initializer
        initializer = None
        if self.match(TokenType.ASSIGN):
            initializer = self.parse_expression()
        
        end = self.peek(-1)
        
        return VariableDeclaration(
            range=self.create_range_from_tokens(start, end),
            name=name,
            type_annotation=type_annotation,
            initializer=initializer,
            is_state=is_state
        )
    
    def parse_expression_statement(self) -> Union[ExpressionStatement, AssignmentStatement]:
        """Parsea expression statement o assignment"""
        start = self.peek()
        
        expr = self.parse_expression()
        
        # Check si es assignment
        if self.match(TokenType.ASSIGN):
            value = self.parse_expression()
            end = self.peek(-1)
            
            return AssignmentStatement(
                range=self.create_range_from_tokens(start, end),
                target=expr,
                value=value
            )
        
        end = self.peek(-1)
        
        return ExpressionStatement(
            range=self.create_range_from_tokens(start, end),
            expression=expr
        )
    
    # ===============================================================
    # EXPRESSIONS (delegado a Pratt Parser en producción)
    # ===============================================================
    
    def parse_expression(self) -> Expression:
        """
        Parsea una expresión.
        
        NOTA: En producción, esto debería delegarse al Pratt Parser
        para manejo correcto de precedencia.
        
        Por ahora, implementación simplificada.
        """
        return self.parse_assignment_expression()
    
    def parse_assignment_expression(self) -> Expression:
        """Parse assignment expression"""
        expr = self.parse_ternary_expression()
        
        if self.match(TokenType.ASSIGN):
            value = self.parse_assignment_expression()
            # Esto debería ser AssignmentExpression, pero lo manejamos en statement
            pass
        
        return expr
    
    def parse_ternary_expression(self) -> Expression:
        """Parse ternary expression (if como expression)"""
        expr = self.parse_logical_or_expression()
        
        # Vela usa if-else como expression
        # Esto se maneja en parse_primary
        
        return expr
    
    def parse_logical_or_expression(self) -> Expression:
        """Parse logical OR expression"""
        start = self.peek()
        left = self.parse_logical_and_expression()
        
        while self.match(TokenType.OR):
            operator = "||"
            right = self.parse_logical_and_expression()
            end = self.peek(-1)
            left = BinaryExpression(
                range=self.create_range_from_tokens(start, end),
                left=left,
                operator=operator,
                right=right
            )
        
        return left
    
    def parse_logical_and_expression(self) -> Expression:
        """Parse logical AND expression"""
        start = self.peek()
        left = self.parse_equality_expression()
        
        while self.match(TokenType.AND):
            operator = "&&"
            right = self.parse_equality_expression()
            end = self.peek(-1)
            left = BinaryExpression(
                range=self.create_range_from_tokens(start, end),
                left=left,
                operator=operator,
                right=right
            )
        
        return left
    
    def parse_equality_expression(self) -> Expression:
        """Parse equality expression"""
        start = self.peek()
        left = self.parse_comparison_expression()
        
        while self.match(TokenType.EQ, TokenType.NE):
            operator = self.peek(-1).value
            right = self.parse_comparison_expression()
            end = self.peek(-1)
            left = BinaryExpression(
                range=self.create_range_from_tokens(start, end),
                left=left,
                operator=operator,
                right=right
            )
        
        return left
    
    def parse_comparison_expression(self) -> Expression:
        """Parse comparison expression"""
        start = self.peek()
        left = self.parse_additive_expression()
        
        while self.match(TokenType.LT, TokenType.GT, TokenType.LE, TokenType.GE):
            operator = self.peek(-1).value
            right = self.parse_additive_expression()
            end = self.peek(-1)
            left = BinaryExpression(
                range=self.create_range_from_tokens(start, end),
                left=left,
                operator=operator,
                right=right
            )
        
        return left
    
    def parse_additive_expression(self) -> Expression:
        """Parse additive expression"""
        start = self.peek()
        left = self.parse_multiplicative_expression()
        
        while self.match(TokenType.PLUS, TokenType.MINUS):
            operator = self.peek(-1).value
            right = self.parse_multiplicative_expression()
            end = self.peek(-1)
            left = BinaryExpression(
                range=self.create_range_from_tokens(start, end),
                left=left,
                operator=operator,
                right=right
            )
        
        return left
    
    def parse_multiplicative_expression(self) -> Expression:
        """Parse multiplicative expression"""
        start = self.peek()
        left = self.parse_unary_expression()
        
        while self.match(TokenType.STAR, TokenType.SLASH, TokenType.PERCENT):
            operator = self.peek(-1).value
            right = self.parse_unary_expression()
            end = self.peek(-1)
            left = BinaryExpression(
                range=self.create_range_from_tokens(start, end),
                left=left,
                operator=operator,
                right=right
            )
        
        return left
    
    def parse_unary_expression(self) -> Expression:
        """Parse unary expression"""
        if self.match(TokenType.MINUS, TokenType.NOT):
            start = self.peek(-1)
            operator = start.value
            operand = self.parse_unary_expression()
            end = self.peek(-1)
            
            return UnaryExpression(
                range=self.create_range_from_tokens(start, end),
                operator=operator,
                operand=operand
            )
        
        return self.parse_postfix_expression()
    
    def parse_postfix_expression(self) -> Expression:
        """Parse postfix expression (call, member access, index)"""
        start = self.peek()
        expr = self.parse_primary_expression()
        
        while True:
            if self.match(TokenType.LPAREN):
                # Call expression
                arguments = []
                if not self.check(TokenType.RPAREN):
                    arguments.append(self.parse_expression())
                    while self.match(TokenType.COMMA):
                        arguments.append(self.parse_expression())
                end = self.expect(TokenType.RPAREN)
                
                expr = CallExpression(
                    range=self.create_range_from_tokens(start, end),
                    callee=expr,
                    arguments=arguments
                )
            
            elif self.match(TokenType.DOT) or self.match(TokenType.OPTIONAL_CHAIN):
                # Member access
                is_optional = self.peek(-1).type == TokenType.OPTIONAL_CHAIN
                member_name = self.expect(TokenType.IDENTIFIER).value
                end = self.peek(-1)
                
                expr = MemberAccessExpression(
                    range=self.create_range_from_tokens(start, end),
                    object=expr,
                    member=member_name,
                    is_optional=is_optional
                )
            
            elif self.match(TokenType.LBRACKET):
                # Index access
                index = self.parse_expression()
                end = self.expect(TokenType.RBRACKET)
                
                expr = IndexAccessExpression(
                    range=self.create_range_from_tokens(start, end),
                    object=expr,
                    index=index
                )
            
            else:
                break
        
        return expr
    
    def parse_primary_expression(self) -> Expression:
        """Parse primary expression"""
        start = self.peek()
        
        # Literals
        if self.check(TokenType.NUMBER):
            token = self.advance()
            return Literal(
                range=self.create_range_from_token(token),
                value=int(token.value),
                kind="number"
            )
        
        if self.check(TokenType.FLOAT):
            token = self.advance()
            return Literal(
                range=self.create_range_from_token(token),
                value=float(token.value),
                kind="float"
            )
        
        if self.check(TokenType.STRING):
            token = self.advance()
            # TODO: parse string interpolation
            return Literal(
                range=self.create_range_from_token(token),
                value=token.value.strip('"\''),
                kind="string"
            )
        
        if self.match(TokenType.TRUE):
            return Literal(
                range=self.create_range_from_token(self.peek(-1)),
                value=True,
                kind="bool"
            )
        
        if self.match(TokenType.FALSE):
            return Literal(
                range=self.create_range_from_token(self.peek(-1)),
                value=False,
                kind="bool"
            )
        
        if self.match(TokenType.NONE):
            return Literal(
                range=self.create_range_from_token(self.peek(-1)),
                value=None,
                kind="none"
            )
        
        # Identifier
        if self.check(TokenType.IDENTIFIER):
            token = self.advance()
            return Identifier(
                range=self.create_range_from_token(token),
                name=token.value
            )
        
        # Parenthesized expression
        if self.match(TokenType.LPAREN):
            expr = self.parse_expression()
            self.expect(TokenType.RPAREN)
            return expr
        
        # Array literal
        if self.match(TokenType.LBRACKET):
            elements = []
            if not self.check(TokenType.RBRACKET):
                elements.append(self.parse_expression())
                while self.match(TokenType.COMMA):
                    elements.append(self.parse_expression())
            end = self.expect(TokenType.RBRACKET)
            
            return ArrayLiteral(
                range=self.create_range_from_tokens(start, end),
                elements=elements
            )
        
        # Object literal
        if self.check(TokenType.LBRACE):
            return self.parse_object_literal()
        
        # Lambda expression
        if self.match(TokenType.PIPE):
            # |params| => expr
            parameters = []
            if not self.check(TokenType.PIPE):
                # Parse parameters
                pass
            self.expect(TokenType.PIPE)
            self.expect(TokenType.ARROW)
            body = self.parse_expression()
            end = self.peek(-1)
            
            return LambdaExpression(
                range=self.create_range_from_tokens(start, end),
                parameters=parameters,
                body=body
            )
        
        # If expression
        if self.match(TokenType.IF):
            condition = self.parse_expression()
            then_branch = self.parse_expression()
            self.expect(TokenType.ELSE)
            else_branch = self.parse_expression()
            end = self.peek(-1)
            
            return IfExpression(
                range=self.create_range_from_tokens(start, end),
                condition=condition,
                then_branch=then_branch,
                else_branch=else_branch
            )
        
        # Match expression
        if self.match(TokenType.MATCH):
            value = self.parse_expression()
            self.expect(TokenType.LBRACE)
            arms = []
            # Parse match arms
            self.expect(TokenType.RBRACE)
            end = self.peek(-1)
            
            return MatchExpression(
                range=self.create_range_from_tokens(start, end),
                value=value,
                arms=arms
            )
        
        # Error: unexpected token
        token = self.peek()
        if token:
            raise UnexpectedTokenError("expression", token)
        raise UnexpectedEOFError("expression")
    
    # ===============================================================
    # PATTERNS
    # ===============================================================
    
    def parse_pattern(self) -> Pattern:
        """Parse pattern for match"""
        start = self.peek()
        
        # Wildcard
        if self.match(TokenType.UNDERSCORE):
            return WildcardPattern(
                range=self.create_range_from_token(self.peek(-1))
            )
        
        # Literal
        if self.check(TokenType.NUMBER, TokenType.STRING, TokenType.TRUE, TokenType.FALSE):
            token = self.advance()
            value = token.value
            if token.type == TokenType.NUMBER:
                value = int(value)
            elif token.type == TokenType.TRUE:
                value = True
            elif token.type == TokenType.FALSE:
                value = False
            
            return LiteralPattern(
                range=self.create_range_from_token(token),
                value=value
            )
        
        # Identifier or Struct/Enum pattern
        if self.check(TokenType.IDENTIFIER):
            name = self.advance().value
            
            # Struct pattern: User { id, name }
            if self.match(TokenType.LBRACE):
                fields = []
                # Parse struct pattern fields
                self.expect(TokenType.RBRACE)
                end = self.peek(-1)
                
                return StructPattern(
                    range=self.create_range_from_tokens(start, end),
                    struct_name=name,
                    fields=fields
                )
            
            # Enum pattern: Some(value)
            elif self.match(TokenType.LPAREN):
                inner = []
                if not self.check(TokenType.RPAREN):
                    inner.append(self.parse_pattern())
                    while self.match(TokenType.COMMA):
                        inner.append(self.parse_pattern())
                self.expect(TokenType.RPAREN)
                end = self.peek(-1)
                
                return EnumPattern(
                    range=self.create_range_from_tokens(start, end),
                    variant_name=name,
                    inner_patterns=inner
                )
            
            # Just identifier
            else:
                return IdentifierPattern(
                    range=self.create_range_from_token(self.peek(-1)),
                    name=name
                )
        
        # Tuple pattern
        if self.match(TokenType.LPAREN):
            elements = []
            if not self.check(TokenType.RPAREN):
                elements.append(self.parse_pattern())
                while self.match(TokenType.COMMA):
                    elements.append(self.parse_pattern())
            end = self.expect(TokenType.RPAREN)
            
            return TuplePattern(
                range=self.create_range_from_tokens(start, end),
                elements=elements
            )
        
        raise UnexpectedTokenError("pattern", self.peek())
    
    # ===============================================================
    # TYPE ANNOTATIONS
    # ===============================================================
    
    def parse_type_annotation(self) -> TypeAnnotation:
        """Parse type annotation"""
        return self.parse_union_type()
    
    def parse_union_type(self) -> TypeAnnotation:
        """Parse union type (A | B | C)"""
        start = self.peek()
        types = [self.parse_primary_type()]
        
        while self.match(TokenType.PIPE):
            types.append(self.parse_primary_type())
        
        if len(types) == 1:
            return types[0]
        
        end = self.peek(-1)
        return UnionType(
            range=self.create_range_from_tokens(start, end),
            types=types
        )
    
    def parse_primary_type(self) -> TypeAnnotation:
        """Parse primary type"""
        start = self.peek()
        
        # Primitive types
        if self.check(TokenType.IDENTIFIER):
            name = self.advance().value
            
            # Generic type: List<T>
            if self.match(TokenType.LT):
                type_args = []
                type_args.append(self.parse_type_annotation())
                while self.match(TokenType.COMMA):
                    type_args.append(self.parse_type_annotation())
                end = self.expect(TokenType.GT)
                
                return GenericType(
                    range=self.create_range_from_tokens(start, end),
                    base_name=name,
                    type_arguments=type_args
                )
            
            # Check if primitive
            if name in ["Number", "Float", "String", "Bool", "void", "never"]:
                return PrimitiveType(
                    range=self.create_range_from_token(self.peek(-1)),
                    name=name
                )
            
            # Named type
            return NamedType(
                range=self.create_range_from_token(self.peek(-1)),
                name=name
            )
        
        # Function type: (A, B) -> C
        if self.match(TokenType.LPAREN):
            param_types = []
            if not self.check(TokenType.RPAREN):
                param_types.append(self.parse_type_annotation())
                while self.match(TokenType.COMMA):
                    param_types.append(self.parse_type_annotation())
            self.expect(TokenType.RPAREN)
            self.expect(TokenType.ARROW)
            return_type = self.parse_type_annotation()
            end = self.peek(-1)
            
            return FunctionType(
                range=self.create_range_from_tokens(start, end),
                parameter_types=param_types,
                return_type=return_type
            )
        
        # Array type: [T]
        if self.match(TokenType.LBRACKET):
            element_type = self.parse_type_annotation()
            end = self.expect(TokenType.RBRACKET)
            
            return ArrayType(
                range=self.create_range_from_tokens(start, end),
                element_type=element_type
            )
        
        raise UnexpectedTokenError("type annotation", self.peek())


# ===================================================================
# CONVENIENCE FUNCTIONS
# ===================================================================

def parse_code(code: str) -> Program:
    """
    Parsea código Vela y retorna el AST.
    
    Args:
        code: Código fuente en Vela
    
    Returns:
        Program: AST del programa
    """
    lexer = Lexer(code)
    tokens = lexer.tokenize()
    parser = Parser(tokens)
    return parser.parse()


if __name__ == "__main__":
    # Ejemplo de uso
    code = """
    fn add(a: Number, b: Number) -> Number {
        return a + b
    }
    
    struct User {
        id: Number
        name: String
    }
    """
    
    try:
        ast = parse_code(code)
        print("✅ Parsing successful!")
        print(f"Program with {len(ast.declarations)} declarations")
        for decl in ast.declarations:
            print(f"  - {decl.__class__.__name__}: {getattr(decl, 'name', 'N/A')}")
    except ParserError as e:
        print(f"❌ Parse error: {e}")
