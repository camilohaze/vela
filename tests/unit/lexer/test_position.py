"""
Tests para Position Tracking del Lexer de Vela

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-007 (Tests Unitarios)
Fecha: 2025-11-30

Tests para tracking de línea, columna y offset.
"""

import pytest
from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind, Position


class TestPositionBasics:
    """Tests básicos de Position."""
    
    def test_position_initialization(self):
        pos = Position(line=1, column=0, offset=0)
        assert pos.line == 1
        assert pos.column == 0
        assert pos.offset == 0
    
    def test_position_advance_regular_char(self):
        pos = Position(line=1, column=0, offset=0)
        pos.advance('a')
        assert pos.line == 1
        assert pos.column == 1
        assert pos.offset == 1
    
    def test_position_advance_newline(self):
        pos = Position(line=1, column=5, offset=5)
        pos.advance('\n')
        assert pos.line == 2
        assert pos.column == 0
        assert pos.offset == 6


class TestLineTracking:
    """Tests para tracking de líneas."""
    
    def test_single_line_tokens(self):
        code = "age = 30"
        tokens = Lexer(code).tokenize()
        for token in tokens[:-1]:  # Excluir EOF
            assert token.position.line == 1
    
    def test_multiline_tokens(self):
        code = """age = 30
name = "Alice"
isActive = true"""
        tokens = Lexer(code).tokenize()
        
        # age está en línea 1
        age_token = tokens[0]
        assert age_token.position.line == 1
        
        # name está en línea 2
        name_token = tokens[4]
        assert name_token.position.line == 2
        
        # isActive está en línea 3
        isActive_token = tokens[8]
        assert isActive_token.position.line == 3
    
    def test_line_increments_on_newline(self):
        code = "line1\nline2\nline3"
        tokens = Lexer(code).tokenize()
        
        line_numbers = [t.position.line for t in tokens if t.kind != TokenKind.EOF]
        assert 1 in line_numbers
        assert 2 in line_numbers
        assert 3 in line_numbers
    
    def test_empty_lines_counted(self):
        code = "line1\n\n\nline4"
        tokens = Lexer(code).tokenize()
        
        line1_token = tokens[0]
        line4_token = tokens[1]
        
        assert line1_token.position.line == 1
        assert line4_token.position.line == 4


class TestColumnTracking:
    """Tests para tracking de columnas."""
    
    def test_column_starts_at_zero(self):
        code = "age"
        token = Lexer(code).next_token()
        assert token.position.column == 0
    
    def test_column_increments(self):
        code = "  age"  # 2 espacios antes
        token = Lexer(code).next_token()
        assert token.position.column == 2
    
    def test_column_resets_on_newline(self):
        code = "line1\nline2"
        tokens = Lexer(code).tokenize()
        
        line1_token = tokens[0]
        line2_token = tokens[1]
        
        assert line1_token.position.column == 0
        assert line2_token.position.column == 0
    
    def test_column_with_tabs(self):
        """Tabs incrementan columna en 1 (no expanden)."""
        code = "\tage"
        token = Lexer(code).next_token()
        assert token.position.column == 1


class TestOffsetTracking:
    """Tests para tracking de offset absoluto."""
    
    def test_offset_starts_at_zero(self):
        code = "age"
        token = Lexer(code).next_token()
        assert token.position.offset == 0
    
    def test_offset_increments(self):
        code = "age = 30"
        tokens = Lexer(code).tokenize()
        
        # Cada token tiene offset creciente
        offsets = [t.position.offset for t in tokens[:-1]]  # Excluir EOF
        assert offsets == sorted(offsets)
    
    def test_offset_counts_all_chars(self):
        code = "abc"  # 3 chars
        tokens = Lexer(code).tokenize()
        
        # offset de 'abc' es 0 (inicio)
        abc_token = tokens[0]
        assert abc_token.position.offset == 0
        
        # EOF debería estar en offset 3
        eof_token = tokens[1]
        assert eof_token.position.offset == 3
    
    def test_offset_includes_newlines(self):
        code = "a\nb"  # 3 chars: a, \n, b
        tokens = Lexer(code).tokenize()
        
        a_token = tokens[0]
        b_token = tokens[1]
        
        assert a_token.position.offset == 0
        assert b_token.position.offset == 2  # Después de 'a' y '\n'


class TestPositionInComplexCode:
    """Tests de position en código complejo."""
    
    def test_position_in_function(self):
        code = """fn greet(name: String) {
    print("Hello, ${name}!")
}"""
        tokens = Lexer(code).tokenize()
        
        fn_token = tokens[0]
        assert fn_token.position.line == 1
        assert fn_token.position.column == 0
        
        # print está en línea 2, con indentación
        print_tokens = [t for t in tokens if t.lexeme == "print"]
        if print_tokens:
            print_token = print_tokens[0]
            assert print_token.position.line == 2
            assert print_token.position.column > 0  # Indentado
    
    def test_position_across_comments(self):
        code = """age = 30 // Comment
// Full line comment
name = "Alice" """
        tokens = Lexer(code).tokenize()
        
        age_token = tokens[0]
        name_token = [t for t in tokens if t.lexeme == "name"][0]
        
        assert age_token.position.line == 1
        assert name_token.position.line == 3
    
    def test_position_in_string_literal(self):
        code = '"This is a long string"'
        token = Lexer(code).next_token()
        
        # La posición es el inicio del string
        assert token.position.line == 1
        assert token.position.column == 0


class TestPositionEdgeCases:
    """Tests de casos edge con position."""
    
    def test_position_at_eof(self):
        code = "age = 30"
        tokens = Lexer(code).tokenize()
        eof_token = tokens[-1]
        
        assert eof_token.kind == TokenKind.EOF
        assert eof_token.position.offset == len(code)
    
    def test_position_empty_file(self):
        tokens = Lexer("").tokenize()
        eof_token = tokens[0]
        
        assert eof_token.position.line == 1
        assert eof_token.position.column == 0
        assert eof_token.position.offset == 0
    
    def test_position_with_crlf(self):
        """CRLF (\\r\\n) incrementa línea una vez."""
        code = "line1\r\nline2"
        tokens = Lexer(code).tokenize()
        
        line1_token = tokens[0]
        line2_token = tokens[1]
        
        assert line1_token.position.line == 1
        assert line2_token.position.line == 2


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
