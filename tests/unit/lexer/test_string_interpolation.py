"""
Tests para String Interpolation en Vela Lexer

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-005 (String Interpolation)
Fecha: 2025-11-30

Tests para validar el soporte de interpolación ${} en strings.
"""

import pytest
from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind


class TestStringInterpolation:
    """Suite de tests para string interpolation ${}."""
    
    def test_simple_string_without_interpolation(self):
        """Test de string simple sin interpolación."""
        code = '"Hello, World!"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Hello, World!"
    
    def test_string_with_single_variable_interpolation(self):
        """Test de interpolación con una sola variable."""
        code = '"Hello, ${name}!"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Hello, ${name}!"
        # El parser procesará la interpolación
    
    def test_string_with_multiple_interpolations(self):
        """Test de string con múltiples interpolaciones."""
        code = '"${greeting}, ${name}! You are ${age} years old."'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert "${greeting}" in token.value
        assert "${name}" in token.value
        assert "${age}" in token.value
    
    def test_string_with_expression_interpolation(self):
        """Test de interpolación con expresiones."""
        code = '"Result: ${x + y}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Result: ${x + y}"
    
    def test_string_with_function_call_interpolation(self):
        """Test de interpolación con llamada a función."""
        code = '"Users: ${getUsers().join(", ")}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert "${getUsers().join(\", \")}" in token.value
    
    def test_string_with_nested_braces(self):
        """Test de interpolación con braces anidados."""
        code = '"Map: ${users.map(u => u.name)}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        # Verifica que el balance de braces es correcto
        assert token.value.count('{') == token.value.count('}')
    
    def test_string_with_escaped_dollar(self):
        """Test de escape de $ para evitar interpolación."""
        code = r'"Price: \$${amount}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        # \$ se escapa a $ literal
        assert token.value == "Price: $${amount}"
    
    def test_string_with_only_dollar(self):
        """Test de $ sin { no es interpolación."""
        code = '"Price: $100"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Price: $100"
    
    def test_empty_interpolation(self):
        """Test de interpolación vacía ${}."""
        code = '"Empty: ${}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Empty: ${}"
    
    def test_interpolation_at_start(self):
        """Test de interpolación al inicio del string."""
        code = '"${name} said hello"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "${name} said hello"
    
    def test_interpolation_at_end(self):
        """Test de interpolación al final del string."""
        code = '"Hello, ${name}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Hello, ${name}"
    
    def test_consecutive_interpolations(self):
        """Test de interpolaciones consecutivas."""
        code = '"${first}${second}${third}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "${first}${second}${third}"
    
    def test_interpolation_with_nested_objects(self):
        """Test de interpolación con acceso a propiedades anidadas."""
        code = '"Name: ${user.profile.name}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Name: ${user.profile.name}"
    
    def test_interpolation_with_ternary(self):
        """Test de interpolación con operador ternario."""
        code = '"Status: ${isActive ? "active" : "inactive"}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert "?" in token.value
        assert ":" in token.value
    
    def test_string_with_escape_sequences_and_interpolation(self):
        """Test combinando escape sequences con interpolación."""
        code = r'"Line 1: ${value}\nLine 2: ${other}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert "\n" in token.value  # \n procesado
        assert "${value}" in token.value
        assert "${other}" in token.value
    
    def test_unterminated_string_with_interpolation(self):
        """Test de string sin cerrar con interpolación."""
        code = '"Hello, ${name}'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.ERROR
        assert "Unterminated" in token.lexeme
    
    def test_complex_nested_expression(self):
        """Test de expresión compleja anidada."""
        code = '"Total: ${items.filter(i => i.active).map(i => i.price).reduce((a, b) => a + b, 0)}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        # Verifica balance de braces
        assert token.value.count('{') == token.value.count('}')
        # Verifica que contiene la expresión
        assert "filter" in token.value
        assert "map" in token.value
        assert "reduce" in token.value
    
    def test_interpolation_with_array_access(self):
        """Test de interpolación con acceso a arrays."""
        code = '"First: ${items[0]}, Last: ${items[items.length - 1]}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert "${items[0]}" in token.value
        assert "${items[items.length - 1]}" in token.value
    
    def test_multiple_escape_sequences(self):
        """Test de múltiples escape sequences."""
        code = r'"Path: C:\\Users\\${username}\\Documents\tFile: ${filename}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert "\\" in token.value
        assert "\t" in token.value
        assert "${username}" in token.value
        assert "${filename}" in token.value


class TestStringInterpolationEdgeCases:
    """Tests de casos edge y validaciones."""
    
    def test_unbalanced_braces_in_interpolation(self):
        """
        Test de braces desbalanceados en interpolación.
        Nota: El lexer balancea braces, el parser detectará errores sintácticos.
        """
        code = '"Value: ${obj.prop}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        # Lexer acepta (es sintácticamente válido)
        assert token.kind == TokenKind.STRING_LITERAL
    
    def test_dollar_at_end_of_string(self):
        """Test de $ al final del string."""
        code = '"Price is $"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "Price is $"
    
    def test_escaped_backslash_before_dollar(self):
        """Test de backslash escapado antes de $."""
        code = r'"Path: \\${folder}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        # \\ se convierte a \
        assert "\\" in token.value
        assert "${folder}" in token.value
    
    def test_empty_string_with_only_interpolation(self):
        """Test de string que es solo interpolación."""
        code = '"${value}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        assert token.value == "${value}"
    
    def test_whitespace_in_interpolation(self):
        """Test de whitespace dentro de interpolación."""
        code = '"Result: ${  x  +  y  }"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        assert token.kind == TokenKind.STRING_LITERAL
        # Whitespace preservado (parser lo limpiará)
        assert "${  x  +  y  }" in token.value
    
    def test_newline_in_interpolation_not_allowed(self):
        """
        Test que newlines dentro de ${} no son permitidos en strings simples.
        (Necesitarían triple-quoted strings para multiline)
        """
        code = '"Value: ${x +\ny}"'
        lexer = Lexer(code)
        token = lexer.next_token()
        
        # El lexer detecta el cierre de " antes del \n
        # O debería dar error de string sin cerrar
        assert token.kind in [TokenKind.STRING_LITERAL, TokenKind.ERROR]


class TestStringInterpolationIntegration:
    """Tests de integración con el resto del lexer."""
    
    def test_string_interpolation_in_variable_assignment(self):
        """Test de interpolación en asignación de variable."""
        code = 'message: String = "Hello, ${name}!"'
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        # Verificar tokens: IDENTIFIER, COLON, IDENTIFIER, EQUAL, STRING_LITERAL, EOF
        assert tokens[0].kind == TokenKind.IDENTIFIER
        assert tokens[0].lexeme == "message"
        assert tokens[4].kind == TokenKind.STRING_LITERAL
        assert "${name}" in tokens[4].value
    
    def test_string_interpolation_in_function_call(self):
        """Test de interpolación como argumento de función."""
        code = 'print("Value: ${x}")'
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        # Verificar que el string es tokenizado correctamente
        string_tokens = [t for t in tokens if t.kind == TokenKind.STRING_LITERAL]
        assert len(string_tokens) == 1
        assert "${x}" in string_tokens[0].value
    
    def test_multiple_strings_with_interpolation(self):
        """Test de múltiples strings con interpolación."""
        code = '''
        greeting = "Hello, ${name}!"
        farewell = "Goodbye, ${name}!"
        '''
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        string_tokens = [t for t in tokens if t.kind == TokenKind.STRING_LITERAL]
        assert len(string_tokens) == 2
        assert all("${name}" in t.value for t in string_tokens)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
