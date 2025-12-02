"""
Pratt Parser para Expresiones del lenguaje Vela

Implementación de: VELA-568 (TASK-009)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

⚠️ IMPORTANTE: Este es código Python del compilador de Vela

Este módulo implementa Pratt Parsing (Top-Down Operator Precedence Parsing)
para manejar correctamente la precedencia y asociatividad de operadores
en expresiones de Vela.

Ventajas del Pratt Parsing:
- Simple y elegante
- Fácil de extender con nuevos operadores
- Manejo natural de precedencia
- Soporte para operadores prefix, infix, postfix

Referencias:
- "Top Down Operator Precedence" - Vaughan Pratt (1973)
- "Pratt Parsers: Expression Parsing Made Easy" - Bob Nystrom

Ejemplo de uso:
```python
from lexer import Lexer
from pratt_parser import PrattParser

code = "1 + 2 * 3 ** 4"
lexer = Lexer(code)
tokens = lexer.tokenize()
parser = PrattParser(tokens)
ast = parser.parse_expression()
print(ast)  # BinaryExpr(+, 1, BinaryExpr(*, 2, BinaryExpr(**, 3, 4)))
```
"""

from typing import Optional, Callable, Dict
from enum import IntEnum
import sys

# Import del lexer (con fallback para tests)
try:
    # Try relative import (when running from src/)
    from ..lexer.token import Token, TokenType
except ImportError:
    # Fallback to absolute import (when running tests)
    sys.path.append('..')
    from src.lexer.token import Token, TokenType

from .ast_nodes import (
    Expression, BinaryExpression, UnaryExpression,
    CallExpression, MemberAccessExpression, IndexAccessExpression,
    Literal, Identifier, LambdaExpression, IfExpression, MatchExpression,
    ArrayLiteral, TupleLiteral, StructLiteral, StringInterpolation,
    AwaitExpression, Range, Position, create_position, Parameter
)


# ===================================================================
# PRATT PARSER ERRORS
# ===================================================================

class PrattParserError(Exception):
    """Error base del Pratt Parser"""
    pass


# ===================================================================
# PRECEDENCE LEVELS (PRATT PARSER)
# ===================================================================

class Precedence(IntEnum):
    """
    Niveles de precedencia para operadores de Vela.
    
    Orden de precedencia (menor a mayor):
    1. NONE: sin precedencia
    2. ASSIGNMENT: =
    3. OR: ||
    4. AND: &&
    5. EQUALITY: ==, !=
    6. COMPARISON: <, >, <=, >=
    7. COALESCING: ??
    8. ADDITIVE: +, -
    9. MULTIPLICATIVE: *, /, %
    10. POWER: **
    11. UNARY: -, !, not
    12. POSTFIX: ., ?., [], ()
    13. PRIMARY: literals, identifiers
    """
    NONE = 0
    ASSIGNMENT = 1     # =
    OR = 2             # ||
    AND = 3            # &&
    EQUALITY = 4       # ==, !=
    COMPARISON = 5     # <, >, <=, >=
    COALESCING = 6     # ??
    ADDITIVE = 7       # +, -
    MULTIPLICATIVE = 8  # *, /, %
    POWER = 9          # **
    UNARY = 10         # -, !, not
    POSTFIX = 11       # ., ?., [], ()
    PRIMARY = 12       # literals, identifiers


# ===================================================================
# PRATT PARSER
# ===================================================================

class PrattParser:
    """
    Pratt Parser para expresiones de Vela.
    
    Implementa Top-Down Operator Precedence Parsing.
    
    El parser mantiene dos tablas:
    - prefix_parsers: Para operadores prefix (ej: -, !, not)
    - infix_parsers: Para operadores infix (ej: +, -, *, /)
    
    Cada token tiene asociado:
    - Un binding power (precedencia)
    - Un parser function (cómo parsearlo)
    """
    
    def __init__(self, tokens: list[Token], start_index: int = 0):
        """
        Inicializa el Pratt Parser.
        
        Args:
            tokens: Lista de tokens
            start_index: Índice inicial en la lista de tokens
        """
        self.tokens = tokens
        self.current = start_index
        
        # Tablas de parsers
        self.prefix_parsers: Dict[TokenType, Callable] = {}
        self.infix_parsers: Dict[TokenType, Callable] = {}
        self.precedences: Dict[TokenType, Precedence] = {}
        
        # Registrar todos los parsers
        self._register_parsers()
    
    def _register_parsers(self):
        """Registra todos los parsers de prefix e infix"""
        
        # ===== PREFIX PARSERS =====
        
        # Literals
        self.register_prefix(TokenType.NUMBER, self.parse_number_literal)
        self.register_prefix(TokenType.FLOAT, self.parse_float_literal)
        self.register_prefix(TokenType.STRING, self.parse_string_literal)
        self.register_prefix(TokenType.TRUE, self.parse_bool_literal)
        self.register_prefix(TokenType.FALSE, self.parse_bool_literal)
        self.register_prefix(TokenType.NONE, self.parse_none_literal)
        
        # Identifier
        self.register_prefix(TokenType.IDENTIFIER, self.parse_identifier)
        
        # Unary operators
        self.register_prefix(TokenType.MINUS, self.parse_unary_expression)
        self.register_prefix(TokenType.NOT, self.parse_unary_expression)
        
        # Grouping
        self.register_prefix(TokenType.LPAREN, self.parse_grouped_or_tuple)
        
        # Array literal
        self.register_prefix(TokenType.LBRACKET, self.parse_array_literal)
        
        # Struct literal
        self.register_prefix(TokenType.LBRACE, self.parse_struct_literal)
        
        # Lambda expression
        self.register_prefix(TokenType.PIPE, self.parse_lambda_expression)
        
        # If expression
        self.register_prefix(TokenType.IF, self.parse_if_expression)
        
        # Match expression
        self.register_prefix(TokenType.MATCH, self.parse_match_expression)
        
        # Await expression
        self.register_prefix(TokenType.AWAIT, self.parse_await_expression)
        
        # ===== INFIX PARSERS =====
        
        # Binary operators
        self.register_infix(TokenType.PLUS, Precedence.ADDITIVE, self.parse_binary_expression)
        self.register_infix(TokenType.MINUS, Precedence.ADDITIVE, self.parse_binary_expression)
        self.register_infix(TokenType.STAR, Precedence.MULTIPLICATIVE, self.parse_binary_expression)
        self.register_infix(TokenType.SLASH, Precedence.MULTIPLICATIVE, self.parse_binary_expression)
        self.register_infix(TokenType.PERCENT, Precedence.MULTIPLICATIVE, self.parse_binary_expression)
        self.register_infix(TokenType.POWER, Precedence.POWER, self.parse_binary_expression)
        
        # Comparison operators
        self.register_infix(TokenType.LT, Precedence.COMPARISON, self.parse_binary_expression)
        self.register_infix(TokenType.GT, Precedence.COMPARISON, self.parse_binary_expression)
        self.register_infix(TokenType.LE, Precedence.COMPARISON, self.parse_binary_expression)
        self.register_infix(TokenType.GE, Precedence.COMPARISON, self.parse_binary_expression)
        
        # Equality operators
        self.register_infix(TokenType.EQ, Precedence.EQUALITY, self.parse_binary_expression)
        self.register_infix(TokenType.NE, Precedence.EQUALITY, self.parse_binary_expression)
        
        # Logical operators
        self.register_infix(TokenType.AND, Precedence.AND, self.parse_binary_expression)
        self.register_infix(TokenType.OR, Precedence.OR, self.parse_binary_expression)
        
        # None coalescing
        self.register_infix(TokenType.COALESCE, Precedence.COALESCING, self.parse_binary_expression)
        
        # Postfix operators
        self.register_infix(TokenType.LPAREN, Precedence.POSTFIX, self.parse_call_expression)
        self.register_infix(TokenType.LBRACKET, Precedence.POSTFIX, self.parse_index_expression)
        self.register_infix(TokenType.DOT, Precedence.POSTFIX, self.parse_member_access)
        self.register_infix(TokenType.OPTIONAL_CHAIN, Precedence.POSTFIX, self.parse_optional_chain)
        
        # Range operator
        self.register_infix(TokenType.RANGE, Precedence.COMPARISON, self.parse_range_expression)
        self.register_infix(TokenType.RANGE_INCLUSIVE, Precedence.COMPARISON, self.parse_range_expression)
    
    def register_prefix(self, token_type: TokenType, parser: Callable):
        """Registra un parser prefix"""
        self.prefix_parsers[token_type] = parser
    
    def register_infix(self, token_type: TokenType, precedence: Precedence, parser: Callable):
        """Registra un parser infix con su precedencia"""
        self.infix_parsers[token_type] = parser
        self.precedences[token_type] = precedence
    
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
        """Consume y retorna el token si es del tipo esperado"""
        token = self.peek()
        if not token or token.type != token_type:
            raise PrattParserError(f"Expected {message or token_type.value}, found {token.type.value if token else 'EOF'}")
        return self.advance()
    
    def get_precedence(self) -> Precedence:
        """Obtiene la precedencia del token actual"""
        token = self.peek()
        if not token:
            return Precedence.NONE
        return self.precedences.get(token.type, Precedence.NONE)
    
    def create_range_from_token(self, token: Token) -> Range:
        """Crea Range desde un token"""
        return Range(
            start=Position(token.line, token.column),
            end=Position(token.line, token.column + len(token.value))
        )
    
    def create_range_from_tokens(self, start: Token, end: Token) -> Range:
        """Crea Range desde dos tokens"""
        return Range(
            start=Position(start.line, start.column),
            end=Position(end.line, end.column + len(end.value))
        )
    
    # ===============================================================
    # MAIN PARSING METHOD
    # ===============================================================
    
    def parse_expression(self, precedence: Precedence = Precedence.NONE) -> Expression:
        """
        Parsea una expresión usando Pratt parsing.
        
        Este es el corazón del Pratt Parser.
        
        Algorithm:
        1. Parse prefix expression (literal, identifier, unary, etc.)
        2. Mientras haya un infix operator con mayor precedencia:
           a. Parse el operador
           b. Parse el right operand con precedence climbing
        
        Args:
            precedence: Precedencia mínima para parsing
        
        Returns:
            Expression: AST node de la expresión
        """
        # 1. Parse prefix
        token = self.peek()
        if not token:
            raise PrattParserError("Unexpected end of input")
        
        prefix_parser = self.prefix_parsers.get(token.type)
        if not prefix_parser:
            raise PrattParserError(f"No prefix parser for {token.type.value}")
        
        left = prefix_parser()
        
        # 2. Parse infix (precedence climbing)
        while precedence < self.get_precedence():
            token = self.peek()
            infix_parser = self.infix_parsers.get(token.type)
            if not infix_parser:
                return left
            
            left = infix_parser(left)
        
        return left
    
    # ===============================================================
    # PREFIX PARSERS
    # ===============================================================
    
    def parse_number_literal(self) -> Literal:
        """Parse number literal"""
        token = self.advance()
        return Literal(
            range=self.create_range_from_token(token),
            value=int(token.value),
            kind="number"
        )
    
    def parse_float_literal(self) -> Literal:
        """Parse float literal"""
        token = self.advance()
        return Literal(
            range=self.create_range_from_token(token),
            value=float(token.value),
            kind="float"
        )
    
    def parse_string_literal(self) -> Expression:
        """Parse string literal (con soporte para interpolación)"""
        token = self.advance()
        value = token.value.strip('"\'')
        
        # NOTA: String interpolation ${...} se maneja en el lexer
        # El lexer ya tokenizó la interpolación, aquí solo parseamos el literal simple
        return Literal(
            range=self.create_range_from_token(token),
            value=value,
            kind="string"
        )
    
    def parse_bool_literal(self) -> Literal:
        """Parse bool literal"""
        token = self.advance()
        return Literal(
            range=self.create_range_from_token(token),
            value=token.type == TokenType.TRUE,
            kind="bool"
        )
    
    def parse_none_literal(self) -> Literal:
        """Parse None literal"""
        token = self.advance()
        return Literal(
            range=self.create_range_from_token(token),
            value=None,
            kind="none"
        )
    
    def parse_identifier(self) -> Identifier:
        """Parse identifier"""
        token = self.advance()
        return Identifier(
            range=self.create_range_from_token(token),
            name=token.value
        )
    
    def parse_unary_expression(self) -> UnaryExpression:
        """Parse unary expression"""
        start = self.advance()
        operator = start.value
        
        # Parse operand with UNARY precedence
        operand = self.parse_expression(Precedence.UNARY)
        
        end = self.peek(-1)
        
        return UnaryExpression(
            range=self.create_range_from_tokens(start, end or start),
            operator=operator,
            operand=operand
        )
    
    def parse_grouped_or_tuple(self) -> Expression:
        """
        Parse grouped expression o tuple literal.
        
        - (expr) -> grouped
        - (expr1, expr2) -> tuple
        """
        start = self.advance()  # consume '('
        
        # Empty tuple
        if self.check(TokenType.RPAREN):
            end = self.advance()
            return TupleLiteral(
                range=self.create_range_from_tokens(start, end),
                elements=[]
            )
        
        # Parse first expression
        first = self.parse_expression()
        
        # Check if tuple
        if self.match(TokenType.COMMA):
            elements = [first]
            
            # Parse remaining elements
            if not self.check(TokenType.RPAREN):
                elements.append(self.parse_expression())
                while self.match(TokenType.COMMA):
                    if self.check(TokenType.RPAREN):
                        break
                    elements.append(self.parse_expression())
            
            end = self.expect(TokenType.RPAREN)
            return TupleLiteral(
                range=self.create_range_from_tokens(start, end),
                elements=elements
            )
        
        # Grouped expression
        self.expect(TokenType.RPAREN)
        return first
    
    def parse_array_literal(self) -> ArrayLiteral:
        """Parse array literal [1, 2, 3]"""
        start = self.advance()  # consume '['
        
        elements = []
        if not self.check(TokenType.RBRACKET):
            elements.append(self.parse_expression())
            while self.match(TokenType.COMMA):
                if self.check(TokenType.RBRACKET):
                    break
                elements.append(self.parse_expression())
        
        end = self.expect(TokenType.RBRACKET)
        
        return ArrayLiteral(
            range=self.create_range_from_tokens(start, end),
            elements=elements
        )
    
    def parse_struct_literal(self) -> StructLiteral:
        """Parse struct literal { x: 1, y: 2 }"""
        start = self.advance()  # consume '{'
        
        fields = []
        if not self.check(TokenType.RBRACE):
            fields.append(self.parse_struct_field())
            while self.match(TokenType.COMMA):
                if self.check(TokenType.RBRACE):
                    break
                fields.append(self.parse_struct_field())
        
        end = self.expect(TokenType.RBRACE)
        
        return StructLiteral(
            range=self.create_range_from_tokens(start, end),
            fields=fields
        )
    
    def parse_struct_field(self) -> tuple[str, Expression]:
        """Parse campo de struct literal"""
        name = self.expect(TokenType.IDENTIFIER).value
        self.expect(TokenType.COLON)
        value = self.parse_expression()
        return (name, value)
    
    def parse_lambda_expression(self) -> LambdaExpression:
        """
        Parse lambda expression.
        
        Syntax: |param1, param2| => expression
        """
        start = self.advance()  # consume '|'
        
        # Parse parameters (simplificado para lambdas)
        parameters = []
        if not self.check(TokenType.PIPE):
            param_token = self.expect(TokenType.IDENTIFIER)
            name = param_token.value
            # Lambdas no requieren type annotation obligatoria
            parameters.append(Parameter(
                name=name,
                type_annotation=None,
                default_value=None,
                range=self.create_range_from_token(param_token)
            ))
            
            while self.match(TokenType.COMMA):
                param_token = self.expect(TokenType.IDENTIFIER)
                name = param_token.value
                parameters.append(Parameter(
                    name=name,
                    type_annotation=None,
                    default_value=None,
                    range=self.create_range_from_token(param_token)
                ))
        
        self.expect(TokenType.PIPE)
        self.expect(TokenType.ARROW)
        
        # Parse body
        body = self.parse_expression()
        
        end = self.peek(-1)
        
        return LambdaExpression(
            range=self.create_range_from_tokens(start, end or start),
            parameters=parameters,
            body=body
        )
    
    def parse_if_expression(self) -> IfExpression:
        """
        Parse if expression.
        
        Syntax: if condition { then_expr } else { else_expr }
        """
        start = self.advance()  # consume 'if'
        
        condition = self.parse_expression()
        
        # Parse then branch (puede ser expression o block)
        if self.check(TokenType.LBRACE):
            # Block expression
            self.advance()
            then_branch = self.parse_expression()
            self.expect(TokenType.RBRACE)
        else:
            then_branch = self.parse_expression()
        
        # Parse else branch
        self.expect(TokenType.ELSE)
        
        if self.check(TokenType.LBRACE):
            self.advance()
            else_branch = self.parse_expression()
            self.expect(TokenType.RBRACE)
        else:
            else_branch = self.parse_expression()
        
        end = self.peek(-1)
        
        return IfExpression(
            range=self.create_range_from_tokens(start, end or start),
            condition=condition,
            then_branch=then_branch,
            else_branch=else_branch
        )
    
    def parse_match_expression(self) -> MatchExpression:
        """
        Parse match expression.
        
        Syntax:
        match value {
          pattern1 => expr1,
          pattern2 => expr2
        }
        """
        start = self.advance()  # consume 'match'
        
        value = self.parse_expression()
        
        self.expect(TokenType.LBRACE)
        
        arms = []
        while not self.check(TokenType.RBRACE):
            # NOTA: Match arms se parsean en el parser principal (parser.py)
            # porque requieren pattern matching complejo
            # Aquí solo manejamos match expressions simples
            self.match(TokenType.COMMA)
        
        end = self.expect(TokenType.RBRACE)
        
        return MatchExpression(
            range=self.create_range_from_tokens(start, end),
            value=value,
            arms=arms
        )
    
    def parse_await_expression(self) -> AwaitExpression:
        """Parse await expression"""
        start = self.advance()  # consume 'await'
        
        expression = self.parse_expression(Precedence.UNARY)
        
        end = self.peek(-1)
        
        return AwaitExpression(
            range=self.create_range_from_tokens(start, end or start),
            expression=expression
        )
    
    # ===============================================================
    # INFIX PARSERS
    # ===============================================================
    
    def parse_binary_expression(self, left: Expression) -> BinaryExpression:
        """
        Parse binary expression.
        
        Maneja asociatividad:
        - Left-associative: +, -, *, /, %, <, >, etc.
        - Right-associative: **
        """
        start_token = self.peek()
        operator_token = self.advance()
        operator = operator_token.value
        
        precedence = self.precedences[operator_token.type]
        
        # Right-associative para **
        if operator == "**":
            right = self.parse_expression(precedence)
        else:
            # Left-associative
            right = self.parse_expression(Precedence(precedence + 1))
        
        end = self.peek(-1)
        
        return BinaryExpression(
            range=self.create_range_from_tokens(
                self.tokens[self.current - 3] if self.current >= 3 else start_token,
                end or operator_token
            ),
            left=left,
            operator=operator,
            right=right
        )
    
    def parse_call_expression(self, callee: Expression) -> CallExpression:
        """Parse call expression"""
        start_range = callee.range
        self.advance()  # consume '('
        
        arguments = []
        if not self.check(TokenType.RPAREN):
            arguments.append(self.parse_expression())
            while self.match(TokenType.COMMA):
                if self.check(TokenType.RPAREN):
                    break
                arguments.append(self.parse_expression())
        
        end = self.expect(TokenType.RPAREN)
        
        return CallExpression(
            range=Range(
                start=start_range.start,
                end=Position(end.line, end.column + 1)
            ),
            callee=callee,
            arguments=arguments
        )
    
    def parse_index_expression(self, object_expr: Expression) -> IndexAccessExpression:
        """Parse index expression obj[index]"""
        start_range = object_expr.range
        self.advance()  # consume '['
        
        index = self.parse_expression()
        
        end = self.expect(TokenType.RBRACKET)
        
        return IndexAccessExpression(
            range=Range(
                start=start_range.start,
                end=Position(end.line, end.column + 1)
            ),
            object=object_expr,
            index=index
        )
    
    def parse_member_access(self, object_expr: Expression) -> MemberAccessExpression:
        """Parse member access obj.member"""
        start_range = object_expr.range
        self.advance()  # consume '.'
        
        member = self.expect(TokenType.IDENTIFIER).value
        
        end = self.peek(-1)
        
        return MemberAccessExpression(
            range=Range(
                start=start_range.start,
                end=Position(end.line, end.column + len(member))
            ),
            object=object_expr,
            member=member,
            is_optional=False
        )
    
    def parse_optional_chain(self, object_expr: Expression) -> MemberAccessExpression:
        """Parse optional chaining obj?.member"""
        start_range = object_expr.range
        self.advance()  # consume '?.'
        
        member = self.expect(TokenType.IDENTIFIER).value
        
        end = self.peek(-1)
        
        return MemberAccessExpression(
            range=Range(
                start=start_range.start,
                end=Position(end.line, end.column + len(member))
            ),
            object=object_expr,
            member=member,
            is_optional=True
        )
    
    def parse_range_expression(self, left: Expression) -> BinaryExpression:
        """
        Parse range expression.
        
        - a..b (exclusive)
        - a..=b (inclusive)
        """
        operator_token = self.advance()
        operator = operator_token.value
        
        right = self.parse_expression(Precedence.COMPARISON)
        
        end = self.peek(-1)
        
        return BinaryExpression(
            range=Range(
                start=left.range.start,
                end=end.line if end else operator_token.line
            ),
            left=left,
            operator=operator,
            right=right
        )


# ===================================================================
# CONVENIENCE FUNCTIONS
# ===================================================================

def parse_expression_from_tokens(tokens: list[Token], start_index: int = 0) -> Expression:
    """
    Parsea una expresión desde una lista de tokens.
    
    Args:
        tokens: Lista de tokens
        start_index: Índice inicial
    
    Returns:
        Expression: AST node de la expresión
    """
    parser = PrattParser(tokens, start_index)
    return parser.parse_expression()


if __name__ == "__main__":
    # Ejemplo de uso
    from lexer import Lexer
    
    code = "1 + 2 * 3 ** 4"
    
    lexer = Lexer(code)
    tokens = lexer.tokenize()
    
    parser = PrattParser(tokens)
    ast = parser.parse_expression()
    
    print(f"✅ Parsed: {code}")
    print(f"Result: {ast}")
