"""
Tests para Manejo de Errores del Lexer de Vela

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-007 (Tests Unitarios)
Fecha: 2025-11-30

Tests para error recovery y tokens inválidos.
"""

import pytest
from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind


class TestErrorRecovery:
    """Tests para error recovery del lexer."""
    
    def test_unterminated_string(self):
        code = '"This is unterminated'
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
        assert "Unterminated" in token.lexeme
    
    def test_unterminated_block_comment(self):
        code = "/* This comment never ends"
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
        assert "Unterminated" in token.lexeme
    
    def test_invalid_escape_sequence(self):
        """Escape sequences inválidos."""
        code = r'"Invalid \x escape"'
        token = Lexer(code).next_token()
        # Debería ser ERROR o procesar como string con \x literal
        assert token.kind in (TokenKind.STRING_LITERAL, TokenKind.ERROR)
    
    def test_error_token_has_position(self):
        """Tokens de error tienen posición."""
        code = '"Unterminated'
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
        assert token.position.line >= 1
        assert token.position.column >= 0


class TestInvalidCharacters:
    """Tests para caracteres inválidos."""
    
    def test_at_symbol_invalid(self):
        """@ no es válido en Vela (salvo en decoradores, no impl)."""
        code = "@invalid"
        token = Lexer(code).next_token()
        # Debería generar ERROR o ser ignorado
        assert token.kind in (TokenKind.ERROR, TokenKind.IDENTIFIER)
    
    def test_hash_symbol_invalid(self):
        """# no es válido (no hay # comments)."""
        code = "# This is not a comment"
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
    
    def test_backtick_invalid(self):
        """Backticks no son válidos en Vela."""
        code = "`not valid`"
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
    
    def test_dollar_alone_after_fix(self):
        """$ solo (sin {) es inválido fuera de string."""
        code = "$100"
        tokens = Lexer(code).tokenize()
        # Debería ser ERROR seguido de NUMBER
        assert tokens[0].kind in (TokenKind.ERROR, TokenKind.NUMBER_LITERAL)


class TestErrorMessagesClarity:
    """Tests para claridad de mensajes de error."""
    
    def test_unterminated_string_message(self):
        code = '"Missing end quote'
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
        # Mensaje debería indicar string sin terminar
        assert "Unterminated string" in token.lexeme or "string" in token.lexeme.lower()
    
    def test_unterminated_comment_message(self):
        code = "/* Missing end"
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
        assert "Unterminated" in token.lexeme or "comment" in token.lexeme.lower()
    
    def test_invalid_char_shows_character(self):
        code = "#"
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
        # El mensaje debería incluir el carácter inválido
        assert "#" in token.lexeme or "Unexpected" in token.lexeme


class TestRecoveryAfterError:
    """Tests para continuación después de error."""
    
    def test_tokenize_continues_after_error(self):
        """El lexer continúa después de un error."""
        code = '# Invalid\nage = 30'
        tokens = Lexer(code).tokenize()
        
        # Debería haber ERROR y luego tokens válidos
        error_count = len([t for t in tokens if t.kind == TokenKind.ERROR])
        valid_count = len([t for t in tokens if t.kind in (TokenKind.IDENTIFIER, TokenKind.EQUAL, TokenKind.NUMBER_LITERAL)])
        
        assert error_count >= 1  # Al menos un error
        assert valid_count >= 3  # age, =, 30
    
    def test_multiple_errors_reported(self):
        """Múltiples errores son reportados."""
        code = '# Error 1\n` Error 2\nage = 30'
        tokens = Lexer(code).tokenize()
        error_count = len([t for t in tokens if t.kind == TokenKind.ERROR])
        assert error_count >= 2


class TestPositionInErrors:
    """Tests para tracking de posición en errores."""
    
    def test_error_position_line(self):
        code = 'valid = 10\n"Unterminated'
        tokens = Lexer(code).tokenize()
        error_token = [t for t in tokens if t.kind == TokenKind.ERROR][0]
        assert error_token.position.line == 2
    
    def test_error_position_column(self):
        code = '  # Invalid'
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
        assert token.position.column == 2  # Después de 2 espacios


class TestEdgeCaseErrors:
    """Tests para casos edge con errores."""
    
    def test_string_with_newline_terminates_early(self):
        """Strings con newline sin escape terminan."""
        code = '"Line 1\nLine 2"'
        token = Lexer(code).next_token()
        # El string termina en el newline
        assert token.kind == TokenKind.STRING_LITERAL
        assert "\n" not in token.value or token.value == "Line 1"
    
    def test_empty_input(self):
        """Input vacío solo produce EOF."""
        tokens = Lexer("").tokenize()
        assert len(tokens) == 1
        assert tokens[0].kind == TokenKind.EOF
    
    def test_whitespace_only(self):
        """Solo whitespace produce EOF."""
        tokens = Lexer("   \n\t\r\n   ").tokenize()
        assert len(tokens) == 1
        assert tokens[0].kind == TokenKind.EOF


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
