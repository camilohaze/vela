"""
Tests para Comentarios del Lexer de Vela

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-007 (Tests Unitarios)
Fecha: 2025-11-30

Tests para // line comments y /* block comments */.
"""

import pytest
from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind


class TestLineComments:
    """Tests para line comments con //."""
    
    def test_line_comment_skipped(self):
        """Line comments son ignorados por el lexer."""
        code = "// This is a comment\n42"
        tokens = Lexer(code).tokenize()
        # Solo debería haber NUMBER y EOF
        assert len([t for t in tokens if t.kind != TokenKind.EOF]) == 1
        assert tokens[0].kind == TokenKind.NUMBER_LITERAL
        assert tokens[0].value == 42
    
    def test_line_comment_at_end(self):
        code = "age = 30 // Set initial age"
        tokens = Lexer(code).tokenize()
        # No debe aparecer el comentario
        assert all(t.kind != TokenKind.ERROR for t in tokens)
        assert tokens[0].kind == TokenKind.IDENTIFIER  # age
        assert tokens[1].kind == TokenKind.EQUAL
        assert tokens[2].kind == TokenKind.NUMBER_LITERAL
    
    def test_line_comment_only(self):
        code = "// Just a comment"
        tokens = Lexer(code).tokenize()
        assert len(tokens) == 1
        assert tokens[0].kind == TokenKind.EOF
    
    def test_multiple_line_comments(self):
        code = """// Comment 1
// Comment 2
// Comment 3
42"""
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.NUMBER_LITERAL
        assert tokens[0].value == 42
    
    def test_line_comment_with_special_chars(self):
        code = "// @#$%^&*() special chars\n10"
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.NUMBER_LITERAL


class TestBlockComments:
    """Tests para block comments con /* */."""
    
    def test_block_comment_single_line(self):
        code = "/* This is a block comment */ 42"
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.NUMBER_LITERAL
        assert tokens[0].value == 42
    
    def test_block_comment_multiline(self):
        code = """/* This is a
multiline
block comment */
age = 30"""
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.IDENTIFIER  # age
        assert tokens[1].kind == TokenKind.EQUAL
        assert tokens[2].kind == TokenKind.NUMBER_LITERAL
    
    def test_block_comment_with_code_inside(self):
        """Comentarios de bloque ignoran código dentro."""
        code = "/* let x = 42; */ state y = 10"
        tokens = Lexer(code).tokenize()
        # Solo debería aparecer state y = 10
        assert tokens[0].kind == TokenKind.STATE
        assert tokens[1].kind == TokenKind.IDENTIFIER  # y
    
    def test_block_comment_at_end(self):
        code = "fn add() { return 42 /* TODO: implement */ }"
        tokens = Lexer(code).tokenize()
        # Verificar que el comentario fue ignorado
        keyword_tokens = [t for t in tokens if t.kind == TokenKind.FN]
        assert len(keyword_tokens) == 1
    
    def test_empty_block_comment(self):
        code = "/**/ 42"
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.NUMBER_LITERAL


class TestCommentEdgeCases:
    """Tests de casos edge con comentarios."""
    
    def test_unterminated_block_comment(self):
        """Block comment sin cerrar genera ERROR."""
        code = "/* Unterminated comment"
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.ERROR
        assert "Unterminated" in token.lexeme
    
    def test_slash_alone_is_operator(self):
        """/ solo es el operador DIVIDE."""
        code = "x / y"
        tokens = Lexer(code).tokenize()
        assert tokens[1].kind == TokenKind.DIVIDE
    
    def test_star_slash_in_string_not_comment_end(self):
        """*/ dentro de string NO termina comentario."""
        code = '"This */ is a string"'
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert "*/" in token.value
    
    def test_double_slash_in_string_not_comment(self):
        """// dentro de string NO es comentario."""
        code = '"URL: http://example.com"'
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        assert "http://example.com" == token.value
    
    def test_comment_markers_in_interpolation(self):
        """Comentarios dentro de ${} en strings."""
        code = '"Value: ${x /* comment */ + y}"'
        token = Lexer(code).next_token()
        assert token.kind == TokenKind.STRING_LITERAL
        # El comentario queda como texto crudo (parser lo procesa)
        assert "/*" in token.value


class TestCommentsInContext:
    """Tests de comentarios en código real."""
    
    def test_comments_around_function(self):
        code = """
// Define function
fn greet(name: String) {
    // Print greeting
    print("Hello, ${name}!")  // End of function
}
"""
        tokens = Lexer(code).tokenize()
        # Verificar que los comentarios no afectan el código
        fn_tokens = [t for t in tokens if t.kind == TokenKind.FN]
        assert len(fn_tokens) == 1
    
    def test_inline_comments_between_tokens(self):
        code = "age /* initial value */ = /* set to */ 30"
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.IDENTIFIER  # age
        assert tokens[1].kind == TokenKind.EQUAL
        assert tokens[2].kind == TokenKind.NUMBER_LITERAL
    
    def test_comment_preserves_line_numbers(self):
        code = """// Line 1
age = 30  // Line 2
name = "Alice"  // Line 3"""
        tokens = Lexer(code).tokenize()
        age_token = tokens[0]
        name_token = tokens[4]
        assert age_token.position.line == 2
        assert name_token.position.line == 3
    
    def test_doc_comment_style(self):
        """Doc comments también con // o /* */."""
        code = """/**
 * Function documentation
 * @param name User name
 */
fn greet(name: String) { }"""
        tokens = Lexer(code).tokenize()
        assert tokens[0].kind == TokenKind.FN


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
