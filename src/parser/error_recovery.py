"""
Error Recovery para el Parser de Vela

Implementación de: VELA-568 (TASK-011)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

⚠️ IMPORTANTE: Este es código Python del compilador de Vela

Este módulo implementa estrategias de recuperación de errores para el parser,
permitiendo reportar múltiples errores en una sola pasada y continuar parseando
después de encontrar errores.

Estrategias implementadas:
1. Panic Mode: Skip hasta sync point (;, }, keyword)
2. Phrase-Level Recovery: Insert/delete tokens sugeridos
3. Error Productions: Catch common mistakes
4. Multiple Error Reporting: Acumular y reportar todos los errores

Referencias:
- "Engineering a Compiler" - Cooper & Torczon
- "Crafting Interpreters" - Bob Nystrom (error recovery chapter)
- "Modern Compiler Implementation" - Andrew Appel

Ejemplo de uso:
```python
from parser import Parser
from error_recovery import ErrorRecoveryParser

code = '''
fn bad_function(  # Missing closing paren
    return 42     # Missing braces
}
'''

parser = ErrorRecoveryParser(code)
ast, errors = parser.parse_with_recovery()

for error in errors:
    print(f"Error at line {error.line}: {error.message}")
```
"""

from typing import List, Optional, Set, Tuple, Dict
from dataclasses import dataclass
from enum import Enum
import sys

# Import del lexer (con fallback para tests)
try:
    # Try relative import (when running from src/)
    from ..lexer.token import Token, TokenType
    from ..lexer.lexer import Lexer
except ImportError:
    # Fallback to absolute import (when running tests)
    sys.path.append('..')
    from src.lexer.token import Token, TokenType
    from src.lexer.lexer import Lexer

from .ast_nodes import (
    Program, Declaration, Statement, Expression,
    Position, Range, create_position
)
from .parser import Parser, ParserError


# ===================================================================
# ERROR TYPES
# ===================================================================

class ErrorSeverity(Enum):
    """Severidad de errores"""
    ERROR = "error"       # Error crítico
    WARNING = "warning"   # Advertencia
    INFO = "info"         # Información


@dataclass
class ParseError:
    """
    Representa un error de parsing.
    
    Incluye información detallada para ayudar al desarrollador.
    """
    severity: ErrorSeverity
    message: str
    position: Position
    token: Optional[Token]
    expected: Optional[List[str]] = None
    suggestion: Optional[str] = None
    
    def __str__(self) -> str:
        """String representation del error"""
        result = f"{self.severity.value.upper()} at line {self.position.line}, column {self.position.column}: {self.message}"
        
        if self.expected:
            result += f"\n  Expected: {', '.join(self.expected)}"
        
        if self.token:
            result += f"\n  Found: {self.token.type.value} '{self.token.value}'"
        
        if self.suggestion:
            result += f"\n  Suggestion: {self.suggestion}"
        
        return result


# ===================================================================
# SYNC POINTS
# ===================================================================

# Tokens donde el parser puede sincronizar tras un error
SYNC_TOKENS: Set[TokenType] = {
    TokenType.SEMICOLON,
    TokenType.RBRACE,
    TokenType.FN,
    TokenType.STRUCT,
    TokenType.ENUM,
    TokenType.CLASS,
    TokenType.INTERFACE,
    TokenType.TYPE,
    TokenType.SERVICE,
    TokenType.REPOSITORY,
    TokenType.CONTROLLER,
    TokenType.ENTITY,
    TokenType.DTO,
    TokenType.IMPORT,
    TokenType.RETURN,
    TokenType.IF,
    TokenType.MATCH,
    TokenType.TRY,
}

# Tokens que indican inicio de declaración
DECLARATION_START_TOKENS: Set[TokenType] = {
    TokenType.FN,
    TokenType.ASYNC,
    TokenType.STRUCT,
    TokenType.ENUM,
    TokenType.CLASS,
    TokenType.INTERFACE,
    TokenType.TYPE,
    TokenType.SERVICE,
    TokenType.REPOSITORY,
    TokenType.CONTROLLER,
    TokenType.ENTITY,
    TokenType.DTO,
    TokenType.USECASE,
    TokenType.VALUE_OBJECT,
}

# Tokens que indican inicio de statement
STATEMENT_START_TOKENS: Set[TokenType] = {
    TokenType.RETURN,
    TokenType.IF,
    TokenType.MATCH,
    TokenType.TRY,
    TokenType.THROW,
    TokenType.STATE,
    TokenType.LBRACE,
}


# ===================================================================
# ERROR RECOVERY PARSER
# ===================================================================

class ErrorRecoveryParser(Parser):
    """
    Parser con capacidades de error recovery.
    
    Extiende el Parser base con estrategias de recuperación:
    - Panic mode recovery
    - Error accumulation
    - Smart synchronization
    - Common error detection
    """
    
    def __init__(self, code: str):
        """
        Inicializa el parser con error recovery.
        
        Args:
            code: Código fuente a parsear
        """
        # Tokenizar código
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        # Inicializar parser base
        super().__init__(tokens)
        
        # Error tracking
        self.parse_errors: List[ParseError] = []
        self.warnings: List[ParseError] = []
        self.recovery_count = 0
        self.max_recovery_attempts = 100  # Evitar loops infinitos
    
    # ===============================================================
    # ERROR REPORTING
    # ===============================================================
    
    def report_error(
        self,
        message: str,
        position: Optional[Position] = None,
        token: Optional[Token] = None,
        expected: Optional[List[str]] = None,
        suggestion: Optional[str] = None,
        severity: ErrorSeverity = ErrorSeverity.ERROR
    ):
        """
        Reporta un error de parsing.
        
        Args:
            message: Mensaje de error
            position: Posición del error
            token: Token que causó el error
            expected: Lista de tokens esperados
            suggestion: Sugerencia de corrección
            severity: Severidad del error
        """
        if position is None and token:
            position = create_position(token.line, token.column)
        elif position is None:
            position = create_position(0, 0)
        
        error = ParseError(
            severity=severity,
            message=message,
            position=position,
            token=token,
            expected=expected,
            suggestion=suggestion
        )
        
        if severity == ErrorSeverity.ERROR:
            self.parse_errors.append(error)
        else:
            self.warnings.append(error)
    
    def report_unexpected_token(
        self,
        expected: List[str],
        found: Optional[Token] = None
    ):
        """Reporta un token inesperado"""
        if found is None:
            found = self.peek()
        
        if found is None:
            self.report_error(
                "Unexpected end of file",
                expected=expected,
                suggestion="Add the missing tokens"
            )
        else:
            self.report_error(
                f"Unexpected token '{found.value}'",
                token=found,
                expected=expected,
                suggestion=self._suggest_fix(expected, found)
            )
    
    def _suggest_fix(self, expected: List[str], found: Token) -> Optional[str]:
        """Genera sugerencia de corrección"""
        if not expected:
            return None
        
        # Common mistakes
        common_mistakes = {
            (")", "]"): "Did you mean to use ')' instead of ']'?",
            ("]", ")"): "Did you mean to use ']' instead of ')'?",
            ("}", "]"): "Did you mean to use '}' instead of ']'?",
            (";", ","): "Did you mean to use ';' instead of ','?",
        }
        
        for (exp, fnd), suggestion in common_mistakes.items():
            if exp in expected and found.value == fnd:
                return suggestion
        
        # Si solo hay un expected, sugerir usarlo
        if len(expected) == 1:
            return f"Try adding '{expected[0]}'"
        
        return None
    
    # ===============================================================
    # PANIC MODE RECOVERY
    # ===============================================================
    
    def synchronize(self):
        """
        Sincroniza el parser tras un error (Panic Mode).
        
        Avanza hasta encontrar un sync point seguro donde continuar parseando.
        """
        if self.recovery_count >= self.max_recovery_attempts:
            raise Exception("Too many recovery attempts, aborting parse")
        
        self.recovery_count += 1
        
        # Avanzar al menos un token
        if not self.is_at_end():
            self.advance()
        
        # Buscar sync point
        while not self.is_at_end():
            # Si el token anterior era un sync point, estamos sincronizados
            prev_token = self.peek(-1)
            if prev_token and prev_token.type in SYNC_TOKENS:
                return
            
            # Si encontramos inicio de declaración, estamos sincronizados
            current = self.peek()
            if current and current.type in DECLARATION_START_TOKENS:
                return
            
            # Si encontramos inicio de statement en contexto apropiado
            if current and current.type in STATEMENT_START_TOKENS:
                return
            
            self.advance()
    
    def synchronize_to_declaration(self):
        """Sincroniza hasta el inicio de la próxima declaración"""
        while not self.is_at_end():
            token = self.peek()
            if token and token.type in DECLARATION_START_TOKENS:
                return
            self.advance()
    
    def synchronize_to_statement(self):
        """Sincroniza hasta el inicio del próximo statement"""
        while not self.is_at_end():
            token = self.peek()
            if token and (token.type in STATEMENT_START_TOKENS or token.type in SYNC_TOKENS):
                return
            self.advance()
    
    # ===============================================================
    # PHRASE-LEVEL RECOVERY
    # ===============================================================
    
    def try_insert_token(self, token_type: TokenType) -> bool:
        """
        Intenta recuperar insertando un token faltante.
        
        Returns:
            True si la inserción parece razonable
        """
        # Solo insertar tokens comunes que frecuentemente se olvidan
        insertable_tokens = {
            TokenType.SEMICOLON,
            TokenType.COMMA,
            TokenType.RPAREN,
            TokenType.RBRACKET,
            TokenType.RBRACE,
        }
        
        if token_type not in insertable_tokens:
            return False
        
        # Reportar como warning
        self.report_error(
            f"Missing '{token_type.value}'",
            token=self.peek(),
            suggestion=f"Add '{token_type.value}' before this token",
            severity=ErrorSeverity.WARNING
        )
        
        return True
    
    def try_delete_token(self) -> bool:
        """
        Intenta recuperar eliminando el token actual.
        
        Returns:
            True si la eliminación parece razonable
        """
        token = self.peek()
        if not token:
            return False
        
        # Solo eliminar tokens que claramente están de más
        deletable_tokens = {
            TokenType.COMMA,
            TokenType.SEMICOLON,
        }
        
        if token.type not in deletable_tokens:
            return False
        
        self.report_error(
            f"Unexpected '{token.value}'",
            token=token,
            suggestion="Remove this token",
            severity=ErrorSeverity.WARNING
        )
        
        self.advance()  # Skip token
        return True
    
    # ===============================================================
    # ERROR PRODUCTIONS (COMMON MISTAKES)
    # ===============================================================
    
    def check_common_mistakes(self) -> bool:
        """
        Detecta y reporta errores comunes.
        
        Returns:
            True si se detectó un error común
        """
        token = self.peek()
        if not token:
            return False
        
        # Mistake 1: usar 'let' o 'const' en lugar de nada o 'state'
        if token.type == TokenType.IDENTIFIER and token.value in ["let", "const", "var"]:
            self.report_error(
                f"'{token.value}' is not a keyword in Vela",
                token=token,
                suggestion="Remove 'let/const/var' (variables are immutable by default) or use 'state' for mutable reactive variables",
                severity=ErrorSeverity.ERROR
            )
            self.advance()
            return True
        
        # Mistake 2: usar 'null' en lugar de 'None'
        if token.type == TokenType.IDENTIFIER and token.value in ["null", "undefined", "nil"]:
            self.report_error(
                f"'{token.value}' does not exist in Vela",
                token=token,
                suggestion="Use 'None' or 'Option<T>' type instead",
                severity=ErrorSeverity.ERROR
            )
            self.advance()
            return True
        
        # Mistake 3: usar 'for' o 'while' loops
        if token.type == TokenType.IDENTIFIER and token.value in ["for", "while", "loop"]:
            self.report_error(
                f"'{token.value}' loops are not allowed in Vela",
                token=token,
                suggestion="Use functional methods (.map(), .filter(), .forEach()) or recursion instead",
                severity=ErrorSeverity.ERROR
            )
            self.advance()
            return True
        
        # Mistake 4: usar 'switch' en lugar de 'match'
        if token.type == TokenType.IDENTIFIER and token.value == "switch":
            self.report_error(
                "'switch' is not a keyword in Vela",
                token=token,
                suggestion="Use 'match' with pattern matching instead",
                severity=ErrorSeverity.ERROR
            )
            self.advance()
            return True
        
        # Mistake 5: usar 'export' en lugar de 'public'
        if token.type == TokenType.IDENTIFIER and token.value == "export":
            self.report_error(
                "'export' is not a keyword in Vela",
                token=token,
                suggestion="Use 'public' modifier instead",
                severity=ErrorSeverity.ERROR
            )
            self.advance()
            return True
        
        return False
    
    # ===============================================================
    # OVERRIDE PARSING METHODS CON ERROR RECOVERY
    # ===============================================================
    
    def parse_with_recovery(self) -> Tuple[Optional[Program], List[ParseError]]:
        """
        Parsea el código con error recovery.
        
        Returns:
            Tuple de (AST, lista de errores)
        """
        try:
            ast = self.parse()
            return (ast, self.parse_errors)
        except Exception as e:
            # Error inesperado durante parsing
            self.report_error(
                f"Fatal parsing error: {str(e)}",
                severity=ErrorSeverity.ERROR
            )
            return (None, self.parse_errors)
    
    def parse_declaration(self) -> Optional[Declaration]:
        """Parse declaration con error recovery"""
        try:
            # Check common mistakes primero
            if self.check_common_mistakes():
                return None
            
            return super().parse_declaration()
        
        except ParserError as e:
            # Reportar error
            self.report_error(
                e.message,
                position=e.position,
                suggestion="Check the syntax and try again"
            )
            
            # Sincronizar a próxima declaración
            self.synchronize_to_declaration()
            return None
        
        except Exception as e:
            # Error inesperado
            self.report_error(
                f"Unexpected error in declaration: {str(e)}",
                token=self.peek()
            )
            self.synchronize_to_declaration()
            return None
    
    def parse_statement(self) -> Optional[Statement]:
        """Parse statement con error recovery"""
        try:
            return super().parse_statement()
        
        except ParserError as e:
            self.report_error(
                e.message,
                position=e.position,
                suggestion="Check the statement syntax"
            )
            
            self.synchronize_to_statement()
            return None
        
        except Exception as e:
            self.report_error(
                f"Unexpected error in statement: {str(e)}",
                token=self.peek()
            )
            self.synchronize_to_statement()
            return None
    
    def parse_expression(self) -> Optional[Expression]:
        """Parse expression con error recovery"""
        try:
            return super().parse_expression()
        
        except ParserError as e:
            self.report_error(
                e.message,
                position=e.position,
                suggestion="Check the expression syntax"
            )
            
            # Para expresiones, sincronizar a operador binario o fin de expresión
            while not self.is_at_end():
                token = self.peek()
                if token and token.type in [
                    TokenType.SEMICOLON, TokenType.COMMA,
                    TokenType.RPAREN, TokenType.RBRACKET, TokenType.RBRACE
                ]:
                    break
                self.advance()
            
            return None
        
        except Exception as e:
            self.report_error(
                f"Unexpected error in expression: {str(e)}",
                token=self.peek()
            )
            return None


# ===================================================================
# ERROR STATISTICS
# ===================================================================

@dataclass
class ErrorStatistics:
    """Estadísticas de errores de parsing"""
    total_errors: int
    total_warnings: int
    syntax_errors: int
    semantic_errors: int
    recovery_attempts: int
    
    def __str__(self) -> str:
        return f"""
Error Statistics:
  Total Errors: {self.total_errors}
  Total Warnings: {self.total_warnings}
  Syntax Errors: {self.syntax_errors}
  Recovery Attempts: {self.recovery_attempts}
"""


def collect_error_statistics(errors: List[ParseError], recovery_count: int) -> ErrorStatistics:
    """Recopila estadísticas de errores"""
    total_errors = sum(1 for e in errors if e.severity == ErrorSeverity.ERROR)
    total_warnings = sum(1 for e in errors if e.severity == ErrorSeverity.WARNING)
    
    return ErrorStatistics(
        total_errors=total_errors,
        total_warnings=total_warnings,
        syntax_errors=total_errors,  # Por ahora todos son syntax errors
        semantic_errors=0,
        recovery_attempts=recovery_count
    )


# ===================================================================
# CONVENIENCE FUNCTIONS
# ===================================================================

def parse_with_error_recovery(code: str) -> Tuple[Optional[Program], List[ParseError], ErrorStatistics]:
    """
    Parsea código con error recovery completo.
    
    Args:
        code: Código fuente
    
    Returns:
        Tuple de (AST, errores, estadísticas)
    """
    parser = ErrorRecoveryParser(code)
    ast, errors = parser.parse_with_recovery()
    stats = collect_error_statistics(errors, parser.recovery_count)
    
    return (ast, errors, stats)


def format_errors(errors: List[ParseError]) -> str:
    """Formatea errores para display"""
    if not errors:
        return "✅ No errors found"
    
    result = []
    for i, error in enumerate(errors, 1):
        result.append(f"{i}. {error}")
    
    return "\n\n".join(result)


if __name__ == "__main__":
    # Ejemplo de uso
    code = """
    fn bad_function(
        return 42
    }
    
    let x = 5  # Error: 'let' no existe
    
    for i in 0..10 {  # Error: 'for' no existe
        print(i)
    }
    """
    
    ast, errors, stats = parse_with_error_recovery(code)
    
    print("=" * 60)
    print("PARSE RESULTS")
    print("=" * 60)
    
    if ast:
        print(f"\n✅ AST generated (with errors)")
    else:
        print(f"\n❌ Failed to generate AST")
    
    print("\n" + "=" * 60)
    print("ERRORS")
    print("=" * 60)
    print(format_errors(errors))
    
    print("\n" + "=" * 60)
    print(stats)
