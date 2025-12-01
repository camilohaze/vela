"""
Token Types for Vela Lexer

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-004
Fecha: 2025-11-30
Actualización: 2025-12-01 (Post-Sprint 8)

Este módulo define todos los tipos de tokens que el lexer puede reconocer.
Incluye ~120 keywords del paradigma funcional puro de Vela.

ACTUALIZACIONES 2025-12-01:
- Agregado keyword 'module' (Angular-style modules)
- Agregados keywords 'extension', 'library', 'package'
- Agregado keyword 'memo' (memoized computed)
- Agregados keywords 'actor', 'Channel', 'Worker' (concurrency)
- Agregados keywords 'on', 'emit', 'off' (event system)
- Agregados keywords 'StatefulWidget', 'StatelessWidget'
- Agregados lifecycle hooks 'beforeMount', 'afterMount'
- Agregado keyword 'batch' (reactive batching)
"""

from enum import Enum, auto
from dataclasses import dataclass
from typing import Union, Optional


class TokenKind(Enum):
    """
    Tipos de tokens reconocidos por el lexer.
    
    Organizado en categorías según la especificación del lenguaje.
    """
    
    # ============================================================
    # CONTROL FLOW (Solo funcional, NO loops imperativos)
    # ============================================================
    IF = auto()
    ELSE = auto()
    MATCH = auto()
    RETURN = auto()
    YIELD = auto()
    
    # ============================================================
    # DECLARATIONS (NO let, const, var)
    # ============================================================
    STATE = auto()          # Única forma de mutabilidad (reactiva)
    FN = auto()
    STRUCT = auto()
    ENUM = auto()
    TRAIT = auto()
    IMPL = auto()
    TYPE = auto()
    INTERFACE = auto()
    CLASS = auto()
    ABSTRACT = auto()
    EXTENDS = auto()
    IMPLEMENTS = auto()
    OVERRIDE = auto()
    OVERLOAD = auto()
    CONSTRUCTOR = auto()
    THIS = auto()
    SUPER = auto()
    
    # ============================================================
    # VISIBILITY & MODIFIERS
    # ============================================================
    PUBLIC = auto()
    PRIVATE = auto()
    PROTECTED = auto()
    ASYNC = auto()
    STATIC = auto()
    EXTERN = auto()
    
    # ============================================================
    # DOMAIN-SPECIFIC KEYWORDS (30+)
    # ============================================================
    # UI
    WIDGET = auto()
    COMPONENT = auto()
    STATEFUL_WIDGET = auto()
    STATELESS_WIDGET = auto()
    
    # Architecture
    SERVICE = auto()
    REPOSITORY = auto()
    CONTROLLER = auto()
    USECASE = auto()
    
    # Models
    DTO = auto()
    ENTITY = auto()
    VALUE_OBJECT = auto()
    MODEL = auto()
    
    # Design Patterns
    FACTORY = auto()
    BUILDER = auto()
    STRATEGY = auto()
    OBSERVER = auto()
    SINGLETON = auto()
    ADAPTER = auto()
    DECORATOR = auto()
    
    # Web/API
    GUARD = auto()
    MIDDLEWARE = auto()
    INTERCEPTOR = auto()
    VALIDATOR = auto()
    
    # Utilities
    PIPE_KEYWORD = auto()  # pipe keyword (transformation pipeline)
    TASK = auto()
    HELPER = auto()
    MAPPER = auto()
    SERIALIZER = auto()
    STORE = auto()
    PROVIDER = auto()
    
    # Module System (Angular-style)
    MODULE = auto()         # module keyword (NOT class, NOT namespace)
    EXTENSION = auto()      # extension keyword
    LIBRARY = auto()        # library keyword
    PACKAGE = auto()        # package keyword
    
    # ============================================================
    # REACTIVE SYSTEM (10)
    # ============================================================
    SIGNAL = auto()         # Signal<T> constructor
    COMPUTED = auto()       # Computed property
    MEMO = auto()           # Memoized computed (aggressive cache)
    EFFECT = auto()         # Side effect
    WATCH = auto()          # Watch value changes
    DISPATCH = auto()       # Store dispatch
    PROVIDE = auto()        # DI provide
    INJECT = auto()         # DI inject
    BATCH = auto()          # Batch reactive updates
    
    # ============================================================
    # LIFECYCLE HOOKS (7)
    # ============================================================
    MOUNT = auto()
    BEFORE_MOUNT = auto()
    AFTER_MOUNT = auto()
    UPDATE = auto()
    BEFORE_UPDATE = auto()
    AFTER_UPDATE = auto()
    DESTROY = auto()
    
    # ============================================================
    # TYPES (NO null, undefined, nil)
    # ============================================================
    NUMBER = auto()
    FLOAT = auto()
    STRING = auto()
    BOOL = auto()
    OPTION = auto()         # Option<T> type
    RESULT = auto()         # Result<T, E> type
    VOID = auto()
    NEVER = auto()
    
    # ============================================================
    # VALUES (None en lugar de null)
    # ============================================================
    TRUE = auto()
    FALSE = auto()
    NONE = auto()           # En lugar de null/undefined/nil
    SOME = auto()           # Some(value) constructor
    OK = auto()             # Ok(value) constructor
    ERR = auto()            # Err(error) constructor
    
    # ============================================================
    # ERROR HANDLING
    # ============================================================
    TRY = auto()
    CATCH = auto()
    THROW = auto()
    FINALLY = auto()
    
    # ============================================================
    # EVENT SYSTEM (3)
    # ============================================================
    ON = auto()             # Event listener: on(event, handler)
    EMIT = auto()           # Emit event: emit(event, data)
    OFF = auto()            # Remove listener: off(event, handler)
    
    # ============================================================
    # CONCURRENCY & ASYNC PROGRAMMING
    # ============================================================
    ACTOR = auto()          # Actor system keyword
    CHANNEL = auto()        # Channel<T> for message passing
    WORKER = auto()         # Worker threads
    ASYNC_KW = auto()       # async keyword
    AWAIT = auto()
    
    # ============================================================
    # MODULE SYSTEM (NO export keyword)
    # ============================================================
    IMPORT = auto()
    FROM = auto()
    AS = auto()
    SHOW = auto()           # import 'lib' show { item1, item2 }
    HIDE = auto()           # import 'lib' hide { item1 }
    
    # ============================================================
    # OPERATORS (40+)
    # ============================================================
    # Arithmetic
    PLUS = auto()           # +
    MINUS = auto()          # -
    STAR = auto()           # *
    SLASH = auto()          # /
    PERCENT = auto()        # %
    STAR_STAR = auto()      # ** (exponenciación)
    
    # Comparison
    EQUAL_EQUAL = auto()    # ==
    BANG_EQUAL = auto()     # !=
    LESS = auto()           # <
    LESS_EQUAL = auto()     # <=
    GREATER = auto()        # >
    GREATER_EQUAL = auto()  # >=
    
    # Logical
    AMPERSAND_AMPERSAND = auto()  # &&
    PIPE_PIPE = auto()            # ||
    BANG = auto()                 # !
    
    # Bitwise
    AMPERSAND = auto()      # &
    PIPE = auto()           # |
    CARET = auto()          # ^
    TILDE = auto()          # ~
    LESS_LESS = auto()      # <<
    GREATER_GREATER = auto() # >>
    
    # Assignment
    EQUAL = auto()          # =
    PLUS_EQUAL = auto()     # +=
    MINUS_EQUAL = auto()    # -=
    STAR_EQUAL = auto()     # *=
    SLASH_EQUAL = auto()    # /=
    PERCENT_EQUAL = auto()  # %=
    
    # Special operators
    QUESTION = auto()              # ? (error propagation)
    QUESTION_QUESTION = auto()     # ?? (Option<T> coalescing)
    QUESTION_DOT = auto()          # ?. (optional chaining)
    DOT = auto()                   # .
    ARROW = auto()                 # ->
    FAT_ARROW = auto()             # =>
    AT = auto()                    # @ (decorators/annotations)
    
    # ============================================================
    # DELIMITERS
    # ============================================================
    LEFT_PAREN = auto()     # (
    RIGHT_PAREN = auto()    # )
    LEFT_BRACE = auto()     # {
    RIGHT_BRACE = auto()    # }
    LEFT_BRACKET = auto()   # [
    RIGHT_BRACKET = auto()  # ]
    COMMA = auto()          # ,
    SEMICOLON = auto()      # ;
    COLON = auto()          # :
    DOUBLE_COLON = auto()   # ::
    
    # ============================================================
    # LITERALS
    # ============================================================
    IDENTIFIER = auto()
    NUMBER_LITERAL = auto()
    FLOAT_LITERAL = auto()
    STRING_LITERAL = auto()
    
    # String Interpolation (TASK-005)
    STRING_INTERPOLATION_START = auto()   # "text ${ (inicio de interpolación)
    STRING_INTERPOLATION_MID = auto()     # } text ${ (medio de interpolación)
    STRING_INTERPOLATION_END = auto()     # } text" (fin de interpolación)
    
    # ============================================================
    # SPECIAL
    # ============================================================
    NEWLINE = auto()
    EOF = auto()
    ERROR = auto()


@dataclass
class Position:
    """
    Representa una posición en el código fuente.
    
    Attributes:
        line: Número de línea (1-indexed)
        column: Número de columna (1-indexed)
        offset: Offset absoluto desde inicio del archivo (0-indexed)
    """
    line: int
    column: int
    offset: int
    
    def __str__(self) -> str:
        return f"{self.line}:{self.column}"
    
    def advance(self, char: str) -> None:
        """
        Avanza la posición según el carácter leído.
        
        Args:
            char: Carácter que fue leído
        """
        self.offset += 1
        if char == '\n':
            self.line += 1
            self.column = 1
        else:
            self.column += 1


@dataclass
class Token:
    """
    Representa un token reconocido por el lexer.
    
    Attributes:
        kind: Tipo de token
        lexeme: Texto original del token
        position: Posición en el código fuente
        value: Valor asociado (para literales)
    """
    kind: TokenKind
    lexeme: str
    position: Position
    value: Optional[Union[int, float, str, bool]] = None
    
    def __str__(self) -> str:
        if self.value is not None:
            return f"{self.kind.name}('{self.lexeme}', {self.value}) at {self.position}"
        return f"{self.kind.name}('{self.lexeme}') at {self.position}"
    
    def __repr__(self) -> str:
        return self.__str__()


# Tabla de keywords: mapeo de string → TokenKind
KEYWORDS = {
    # Control Flow (funcional)
    "if": TokenKind.IF,
    "else": TokenKind.ELSE,
    "match": TokenKind.MATCH,
    "return": TokenKind.RETURN,
    "yield": TokenKind.YIELD,
    
    # Declarations
    "state": TokenKind.STATE,
    "fn": TokenKind.FN,
    "struct": TokenKind.STRUCT,
    "enum": TokenKind.ENUM,
    "trait": TokenKind.TRAIT,
    "impl": TokenKind.IMPL,
    "type": TokenKind.TYPE,
    "interface": TokenKind.INTERFACE,
    "class": TokenKind.CLASS,
    "abstract": TokenKind.ABSTRACT,
    "extends": TokenKind.EXTENDS,
    "implements": TokenKind.IMPLEMENTS,
    "override": TokenKind.OVERRIDE,
    "overload": TokenKind.OVERLOAD,
    "constructor": TokenKind.CONSTRUCTOR,
    "this": TokenKind.THIS,
    "super": TokenKind.SUPER,
    
    # Visibility
    "public": TokenKind.PUBLIC,
    "private": TokenKind.PRIVATE,
    "protected": TokenKind.PROTECTED,
    "async": TokenKind.ASYNC_KW,
    "static": TokenKind.STATIC,
    "extern": TokenKind.EXTERN,
    
    # Domain-specific (30+)
    "widget": TokenKind.WIDGET,
    "component": TokenKind.COMPONENT,
    "StatefulWidget": TokenKind.STATEFUL_WIDGET,
    "StatelessWidget": TokenKind.STATELESS_WIDGET,
    "service": TokenKind.SERVICE,
    "repository": TokenKind.REPOSITORY,
    "controller": TokenKind.CONTROLLER,
    "usecase": TokenKind.USECASE,
    "dto": TokenKind.DTO,
    "entity": TokenKind.ENTITY,
    "valueObject": TokenKind.VALUE_OBJECT,
    "model": TokenKind.MODEL,
    "factory": TokenKind.FACTORY,
    "builder": TokenKind.BUILDER,
    "strategy": TokenKind.STRATEGY,
    "observer": TokenKind.OBSERVER,
    "singleton": TokenKind.SINGLETON,
    "adapter": TokenKind.ADAPTER,
    "decorator": TokenKind.DECORATOR,
    "guard": TokenKind.GUARD,
    "middleware": TokenKind.MIDDLEWARE,
    "interceptor": TokenKind.INTERCEPTOR,
    "validator": TokenKind.VALIDATOR,
    "pipe": TokenKind.PIPE_KEYWORD,
    "task": TokenKind.TASK,
    "helper": TokenKind.HELPER,
    "mapper": TokenKind.MAPPER,
    "serializer": TokenKind.SERIALIZER,
    "store": TokenKind.STORE,
    "provider": TokenKind.PROVIDER,
    
    # Module System (Angular-style)
    "module": TokenKind.MODULE,
    "extension": TokenKind.EXTENSION,
    "library": TokenKind.LIBRARY,
    "package": TokenKind.PACKAGE,
    
    # Reactive (10)
    "Signal": TokenKind.SIGNAL,
    "Computed": TokenKind.COMPUTED,
    "memo": TokenKind.MEMO,
    "Effect": TokenKind.EFFECT,
    "Watch": TokenKind.WATCH,
    "dispatch": TokenKind.DISPATCH,
    "provide": TokenKind.PROVIDE,
    "inject": TokenKind.INJECT,
    "batch": TokenKind.BATCH,
    
    # Concurrency
    "actor": TokenKind.ACTOR,
    "Channel": TokenKind.CHANNEL,
    "Worker": TokenKind.WORKER,
    
    # Event System
    "on": TokenKind.ON,
    "emit": TokenKind.EMIT,
    "off": TokenKind.OFF,
    
    # Lifecycle (7)
    "mount": TokenKind.MOUNT,
    "beforeMount": TokenKind.BEFORE_MOUNT,
    "afterMount": TokenKind.AFTER_MOUNT,
    "update": TokenKind.UPDATE,
    "beforeUpdate": TokenKind.BEFORE_UPDATE,
    "afterUpdate": TokenKind.AFTER_UPDATE,
    "destroy": TokenKind.DESTROY,
    
    # Types
    "Number": TokenKind.NUMBER,
    "Float": TokenKind.FLOAT,
    "String": TokenKind.STRING,
    "Bool": TokenKind.BOOL,
    "Option": TokenKind.OPTION,
    "Result": TokenKind.RESULT,
    "void": TokenKind.VOID,
    "never": TokenKind.NEVER,
    
    # Values
    "true": TokenKind.TRUE,
    "false": TokenKind.FALSE,
    "None": TokenKind.NONE,
    "Some": TokenKind.SOME,
    "Ok": TokenKind.OK,
    "Err": TokenKind.ERR,
    
    # Error handling
    "try": TokenKind.TRY,
    "catch": TokenKind.CATCH,
    "throw": TokenKind.THROW,
    "finally": TokenKind.FINALLY,
    
    # Async
    "await": TokenKind.AWAIT,
    
    # Module system
    "import": TokenKind.IMPORT,
    "from": TokenKind.FROM,
    "as": TokenKind.AS,
    "show": TokenKind.SHOW,
    "hide": TokenKind.HIDE,
}


def is_keyword(text: str) -> bool:
    """Verifica si el texto es un keyword."""
    return text in KEYWORDS


def get_keyword_token_kind(text: str) -> Optional[TokenKind]:
    """Obtiene el TokenKind para un keyword, o None si no es keyword."""
    return KEYWORDS.get(text)


if __name__ == "__main__":
    # Demo: crear algunos tokens
    pos = Position(line=1, column=1, offset=0)
    
    # Token de keyword
    token1 = Token(TokenKind.SERVICE, "service", pos)
    print(token1)
    
    # Token de literal
    token2 = Token(TokenKind.NUMBER_LITERAL, "42", pos, value=42)
    print(token2)
    
    # Token de string
    token3 = Token(TokenKind.STRING_LITERAL, '"Hello, Vela!"', pos, value="Hello, Vela!")
    print(token3)
    
    # Verificar keywords
    print(f"\n'service' is keyword: {is_keyword('service')}")
    print(f"'let' is keyword: {is_keyword('let')}")  # False (NO existe)
    print(f"'state' is keyword: {is_keyword('state')}")  # True
    print(f"'module' is keyword: {is_keyword('module')}")  # True (agregado 2025-12-01)
    print(f"'namespace' is keyword: {is_keyword('namespace')}")  # False (NO existe)
    
    print(f"\nTotal keywords: {len(KEYWORDS)}")
    print(f"Total tokens: {len(TokenKind)}")
