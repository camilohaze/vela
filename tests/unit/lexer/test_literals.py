"""
Tests para Literales del Lexer de Vela

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-007 (Tests Unitarios)
Fecha: 2025-11-30

Tests para números, floats, strings y booleanos.
"""

import pytest
from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind


class TestNumberLiterals:
    """Tests para literales numéricos enteros."""
    
    def test_single_digit_number(self):
        token = Lexer("5").next_token()
        assert token.kind == TokenKind.NUMBER_LITERAL
        assert token.value == 5
        assert token.lexeme == "5"
    
    def test_multiple_digit_number(self):
        token = Lexer("42").next_token()
        assert token.kind == TokenKind.NUMBER_LITERAL
        assert token.value == 42
    
    def test_large_number(self):
        token = Lexer("123456789").next_token()
        assert token.kind == TokenKind.NUMBER_LITERAL
        assert token.value == 123456789
    
    def test_zero(self):
        token = Lexer("0").next_token()
        assert token.kind == TokenKind.NUMBER_LITERAL
        assert token.value == 0
    
    def test_number_followed_by_identifier(self):
        """Números seguidos de letras son dos tokens."""
        tokens = Lexer("42answer").tokenize()
        assert tokens[0].kind == TokenKind.NUMBER_LITERAL
        assert tokens[0].value == 42
        assert tokens[1].kind == TokenKind.IDENTIFIER
        assert tokens[1].lexeme == "answer"


class TestFloatLiterals:
    """Tests para literales de punto flotante."""
    
    def test_simple_float(self):
        token = Lexer("3.14").next_token()
        assert token.kind == TokenKind.FLOAT_LITERAL
        assert token.value == 3.14
        assert token.lexeme == "3.14"
    
    def test_float_starting_with_zero(self):
        token = Lexer("0.5").next_token()
        assert token.kind == TokenKind.FLOAT_LITERAL
        assert token.value == 0.5
    
    def test_float_with_many_decimals(self):
        token = Lexer("3.141592653589793").next_token()
        assert token.kind == TokenKind.FLOAT_LITERAL
        assert abs(token.value - 3.141592653589793) < 0.0000001
    
    def test_number_dot_without_digits_is_not_float(self):
        """3. sin dígitos después NO es float."""
        tokens = Lexer("3.").tokenize()
        # Debería ser NUMBER seguido de DOT
        assert tokens[0].kind == TokenKind.NUMBER_LITERAL
        assert tokens[0].value == 3
        assert tokens[1].kind == TokenKind.DOT
    
    def test_float_in_expression(self):
        code = "price = 19.99"
        tokens = Lexer(code).tokenize()
        float_token = [t for t in tokens if t.kind == TokenKind.FLOAT_LITERAL][0]
        assert float_token.value == 19.99


class TestStringLiterals:
    """Tests para literales de string."""
    
    def test_empty_string(self):
        token = Lexer('""').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == ""
    
    def test_simple_string(self):
        token = Lexer('"Hello"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Hello"
    
    def test_string_with_spaces(self):
        token = Lexer('"Hello World"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Hello World"
    
    def test_string_with_numbers(self):
        token = Lexer('"Code 123"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Code 123"
    
    def test_string_with_special_chars(self):
        token = Lexer('"Hello, World! @#$"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert "Hello, World! @#$" == token.value


class TestStringEscapeSequences:
    """Tests para escape sequences en strings."""
    
    def test_newline_escape(self):
        token = Lexer(r'"Line 1\nLine 2"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Line 1\nLine 2"
    
    def test_tab_escape(self):
        token = Lexer(r'"Col1\tCol2"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Col1\tCol2"
    
    def test_backslash_escape(self):
        token = Lexer(r'"Path: C:\\Users"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Path: C:\\Users"
    
    def test_quote_escape(self):
        token = Lexer(r'"Say \"Hello\""').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == 'Say "Hello"'
    
    def test_carriage_return_escape(self):
        token = Lexer(r'"Line\rReturn"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Line\rReturn"
    
    def test_null_char_escape(self):
        token = Lexer(r'"Null\0Char"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Null\0Char"
    
    def test_multiple_escapes(self):
        token = Lexer(r'"Line 1\n\tLine 2\n\tLine 3"').next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert "\n\t" in token.value


class TestBooleanLiterals:
    """Tests para literales booleanos."""
    
    def test_true_literal(self):
        token = Lexer("true").next_token()
        assert token.kind == TokenKind.TRUE
        assert token.lexeme == "true"
    
    def test_false_literal(self):
        token = Lexer("false").next_token()
        assert token.kind == TokenKind.FALSE
        assert token.lexeme == "false"
    
    def test_boolean_in_expression(self):
        code = "isActive = true"
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.IDENTIFIER
        assert tokens[2].kind == TokenKind.TRUE


class TestLiteralsInContext:
    """Tests de literales en contextos reales."""
    
    def test_variable_assignment_with_number(self):
        code = "age = 30"
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.IDENTIFIER  # age
        assert tokens[1].kind == TokenKind.EQUAL
        assert tokens[2].kind == TokenKind.NUMBER_LITERAL
        assert tokens[2].value == 30
    
    def test_variable_assignment_with_float(self):
        code = "pi = 3.14159"
        tokens = Lexer(code).tokenize()
        assert tokens[2].kind == TokenKind.FLOAT_LITERAL
        assert abs(tokens[2].value - 3.14159) < 0.00001
    
    def test_variable_assignment_with_string(self):
        code = 'name = "Alice"'
        tokens = Lexer(code).tokenize()
        assert tokens[2].kind == TokenKind.STRING_LITERAL
        assert tokens[2].value == "Alice"
    
    def test_function_call_with_literals(self):
        code = 'print("Age:", 30, true)'
        tokens = Lexer(code).tokenize()
        
        string_tokens = [t for t in tokens if t.kind == TokenKind.STRING_LITERAL]
        assert len(string_tokens) == 1
        assert string_tokens[0].value == "Age:"
        
        number_tokens = [t for t in tokens if t.kind == TokenKind.NUMBER_LITERAL]
        assert len(number_tokens) == 1
        assert number_tokens[0].value == 30
        
        bool_tokens = [t for t in tokens if t.kind == TokenKind.TRUE]
        assert len(bool_tokens) == 1
    
    def test_array_with_mixed_literals(self):
        code = '[1, "two", 3.0, true]'
        tokens = Lexer(code).tokenize()
        
        assert tokens[0].kind == TokenKind.LEFT_BRACKET
        assert tokens[1].kind == TokenKind.NUMBER_LITERAL
        assert tokens[1].value == 1
        assert tokens[3].kind == TokenKind.STRING_LITERAL
        assert tokens[3].value == "two"
        assert tokens[5].kind == TokenKind.FLOAT_LITERAL
        assert tokens[5].value == 3.0
        assert tokens[7].kind == TokenKind.TRUE


class TestLiteralEdgeCases:
    """Tests de casos edge con literales."""
    
    def test_unterminated_string(self):
        token = Lexer('"unterminated').next_token()
        assert token.kind == TokenKind.ERROR
        assert "Unterminated" in token.lexeme
    
    def test_string_with_newline_terminates(self):
        """Strings no pueden contener newlines sin escape."""
        code = '"Line 1\nLine 2"'
        token = Lexer(code).next_token()
        # El string termina en el newline
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Line 1"
    
    def test_negative_number_is_minus_and_number(self):
        """Números negativos son dos tokens: MINUS y NUMBER."""
        tokens = Lexer("-42").tokenize()
        assert tokens[0].kind == TokenKind.MINUS
        assert tokens[1].kind == TokenKind.NUMBER_LITERAL
        assert tokens[1].value == 42
    
    def test_float_without_integer_part(self):
        """.5 sin parte entera NO es válido (debería ser 0.5)."""
        tokens = Lexer(".5").tokenize()
        # Debería ser DOT seguido de NUMBER
        assert tokens[0].kind == TokenKind.DOT
        assert tokens[1].kind == TokenKind.NUMBER_LITERAL
    
    def test_multiple_dots_in_number(self):
        """1.2.3 no es un número válido."""
        tokens = Lexer("1.2.3").tokenize()
        assert tokens[0].kind == TokenKind.FLOAT_LITERAL
        assert tokens[0].value == 1.2
        assert tokens[1].kind == TokenKind.DOT
        assert tokens[2].kind == TokenKind.NUMBER_LITERAL
        assert tokens[2].value == 3


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
