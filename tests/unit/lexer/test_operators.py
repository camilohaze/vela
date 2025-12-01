"""
Tests para Operadores del Lexer de Vela

Implementación de: VELA-567 (Sprint 5)
Subtask: TASK-007 (Tests Unitarios)
Fecha: 2025-11-30

Tests exhaustivos para todos los ~45 operadores del lenguaje Vela.
"""

import pytest
from src.lexer.lexer import Lexer
from src.lexer.token import TokenKind


class TestArithmeticOperators:
    """Tests para operadores aritméticos."""
    
    def test_plus_operator(self):
        token = Lexer("+").next_token()
        assert token.kind == TokenKind.PLUS
        assert token.lexeme == "+"
    
    def test_minus_operator(self):
        token = Lexer("-").next_token()
        assert token.kind == TokenKind.MINUS
        assert token.lexeme == "-"
    
    def test_star_operator(self):
        token = Lexer("*").next_token()
        assert token.kind == TokenKind.STAR
        assert token.lexeme == "*"
    
    def test_slash_operator(self):
        token = Lexer("/").next_token()
        assert token.kind == TokenKind.SLASH
        assert token.lexeme == "/"
    
    def test_percent_operator(self):
        token = Lexer("%").next_token()
        assert token.kind == TokenKind.PERCENT
        assert token.lexeme == "%"
    
    def test_star_star_operator(self):
        """Exponenciación **"""
        token = Lexer("**").next_token()
        assert token.kind == TokenKind.STAR_STAR
        assert token.lexeme == "**"
    
    def test_star_star_not_two_stars(self):
        """** es un token, no dos *"""
        tokens = Lexer("**").tokenize()
        assert len(tokens) == 2  # ** y EOF
        assert tokens[0].kind == TokenKind.STAR_STAR


class TestComparisonOperators:
    """Tests para operadores de comparación."""
    
    def test_equal_equal_operator(self):
        token = Lexer("==").next_token()
        assert token.kind == TokenKind.EQUAL_EQUAL
        assert token.lexeme == "=="
    
    def test_bang_equal_operator(self):
        token = Lexer("!=").next_token()
        assert token.kind == TokenKind.BANG_EQUAL
        assert token.lexeme == "!="
    
    def test_less_operator(self):
        token = Lexer("<").next_token()
        assert token.kind == TokenKind.LESS
        assert token.lexeme == "<"
    
    def test_less_equal_operator(self):
        token = Lexer("<=").next_token()
        assert token.kind == TokenKind.LESS_EQUAL
        assert token.lexeme == "<="
    
    def test_greater_operator(self):
        token = Lexer(">").next_token()
        assert token.kind == TokenKind.GREATER
        assert token.lexeme == ">"
    
    def test_greater_equal_operator(self):
        token = Lexer(">=").next_token()
        assert token.kind == TokenKind.GREATER_EQUAL
        assert token.lexeme == ">="


class TestLogicalOperators:
    """Tests para operadores lógicos."""
    
    def test_ampersand_ampersand_operator(self):
        """Operador AND lógico &&"""
        token = Lexer("&&").next_token()
        assert token.kind == TokenKind.AMPERSAND_AMPERSAND
        assert token.lexeme == "&&"
    
    def test_pipe_pipe_operator(self):
        """Operador OR lógico ||"""
        token = Lexer("||").next_token()
        assert token.kind == TokenKind.PIPE_PIPE
        assert token.lexeme == "||"
    
    def test_bang_operator(self):
        """Operador NOT lógico !"""
        token = Lexer("!").next_token()
        assert token.kind == TokenKind.BANG
        assert token.lexeme == "!"


class TestBitwiseOperators:
    """Tests para operadores bitwise."""
    
    def test_ampersand_operator(self):
        """Operador AND bitwise &"""
        token = Lexer("&").next_token()
        assert token.kind == TokenKind.AMPERSAND
        assert token.lexeme == "&"
    
    def test_pipe_operator(self):
        """Operador OR bitwise |"""
        token = Lexer("|").next_token()
        assert token.kind == TokenKind.PIPE
        assert token.lexeme == "|"
    
    def test_caret_operator(self):
        """Operador XOR bitwise ^"""
        token = Lexer("^").next_token()
        assert token.kind == TokenKind.CARET
        assert token.lexeme == "^"
    
    def test_tilde_operator(self):
        """Operador NOT bitwise ~"""
        token = Lexer("~").next_token()
        assert token.kind == TokenKind.TILDE
        assert token.lexeme == "~"
    
    def test_less_less_operator(self):
        """Operador shift left <<"""
        token = Lexer("<<").next_token()
        assert token.kind == TokenKind.LESS_LESS
        assert token.lexeme == "<<"
    
    def test_greater_greater_operator(self):
        """Operador shift right >>"""
        token = Lexer(">>").next_token()
        assert token.kind == TokenKind.GREATER_GREATER
        assert token.lexeme == ">>"


class TestAssignmentOperators:
    """Tests para operadores de asignación."""
    
    def test_equal_operator(self):
        token = Lexer("=").next_token()
        assert token.kind == TokenKind.EQUAL
        assert token.lexeme == "="
    
    def test_plus_equal_operator(self):
        token = Lexer("+=").next_token()
        assert token.kind == TokenKind.PLUS_EQUAL
        assert token.lexeme == "+="
    
    def test_minus_equal_operator(self):
        token = Lexer("-=").next_token()
        assert token.kind == TokenKind.MINUS_EQUAL
        assert token.lexeme == "-="
    
    def test_star_equal_operator(self):
        token = Lexer("*=").next_token()
        assert token.kind == TokenKind.STAR_EQUAL
        assert token.lexeme == "*="
    
    def test_slash_equal_operator(self):
        token = Lexer("/=").next_token()
        assert token.kind == TokenKind.SLASH_EQUAL
        assert token.lexeme == "/="
    
    def test_percent_equal_operator(self):
        token = Lexer("%=").next_token()
        assert token.kind == TokenKind.PERCENT_EQUAL
        assert token.lexeme == "%="


class TestSpecialOperators:
    """Tests para operadores especiales de Vela."""
    
    def test_question_operator(self):
        """Operador ? (error propagation)"""
        token = Lexer("?").next_token()
        assert token.kind == TokenKind.QUESTION
        assert token.lexeme == "?"
    
    def test_question_question_operator(self):
        """Operador ?? (Option<T> coalescing)"""
        token = Lexer("??").next_token()
        assert token.kind == TokenKind.QUESTION_QUESTION
        assert token.lexeme == "??"
    
    def test_question_dot_operator(self):
        """Operador ?. (optional chaining)"""
        token = Lexer("?.").next_token()
        assert token.kind == TokenKind.QUESTION_DOT
        assert token.lexeme == "?."
    
    def test_dot_operator(self):
        token = Lexer(".").next_token()
        assert token.kind == TokenKind.DOT
        assert token.lexeme == "."
    
    def test_arrow_operator(self):
        """Operador -> (return type)"""
        token = Lexer("->").next_token()
        assert token.kind == TokenKind.ARROW
        assert token.lexeme == "->"
    
    def test_fat_arrow_operator(self):
        """Operador => (lambda)"""
        token = Lexer("=>").next_token()
        assert token.kind == TokenKind.FAT_ARROW
        assert token.lexeme == "=>"


class TestDelimiters:
    """Tests para delimitadores."""
    
    def test_left_paren(self):
        token = Lexer("(").next_token()
        assert token.kind == TokenKind.LEFT_PAREN
        assert token.lexeme == "("
    
    def test_right_paren(self):
        token = Lexer(")").next_token()
        assert token.kind == TokenKind.RIGHT_PAREN
        assert token.lexeme == ")"
    
    def test_left_brace(self):
        token = Lexer("{").next_token()
        assert token.kind == TokenKind.LEFT_BRACE
        assert token.lexeme == "{"
    
    def test_right_brace(self):
        token = Lexer("}").next_token()
        assert token.kind == TokenKind.RIGHT_BRACE
        assert token.lexeme == "}"
    
    def test_left_bracket(self):
        token = Lexer("[").next_token()
        assert token.kind == TokenKind.LEFT_BRACKET
        assert token.lexeme == "["
    
    def test_right_bracket(self):
        token = Lexer("]").next_token()
        assert token.kind == TokenKind.RIGHT_BRACKET
        assert token.lexeme == "]"
    
    def test_comma(self):
        token = Lexer(",").next_token()
        assert token.kind == TokenKind.COMMA
        assert token.lexeme == ","
    
    def test_semicolon(self):
        token = Lexer(";").next_token()
        assert token.kind == TokenKind.SEMICOLON
        assert token.lexeme == ";"
    
    def test_colon(self):
        token = Lexer(":").next_token()
        assert token.kind == TokenKind.COLON
        assert token.lexeme == ":"
    
    def test_double_colon(self):
        token = Lexer("::").next_token()
        assert token.kind == TokenKind.DOUBLE_COLON
        assert token.lexeme == "::"


class TestOperatorPrecedence:
    """Tests para verificar tokenización correcta de operadores compuestos."""
    
    def test_distinguish_arrow_from_minus_greater(self):
        """-> es un token, no - y >"""
        tokens = Lexer("->").tokenize()
        assert len(tokens) == 2  # -> y EOF
        assert tokens[0].kind == TokenKind.ARROW
    
    def test_distinguish_fat_arrow_from_equal_greater(self):
        """=> es un token, no = y >"""
        tokens = Lexer("=>").tokenize()
        assert len(tokens) == 2  # => y EOF
        assert tokens[0].kind == TokenKind.FAT_ARROW
    
    def test_distinguish_equal_equal_from_two_equals(self):
        """== es un token, no dos ="""
        tokens = Lexer("==").tokenize()
        assert len(tokens) == 2  # == y EOF
        assert tokens[0].kind == TokenKind.EQUAL_EQUAL
    
    def test_three_equals_is_error(self):
        """=== no es válido en Vela"""
        tokens = Lexer("===").tokenize()
        # Debería ser == y =, o error
        assert tokens[0].kind == TokenKind.EQUAL_EQUAL
        assert tokens[1].kind == TokenKind.EQUAL
    
    def test_question_question_vs_two_questions(self):
        """?? es un token único"""
        tokens = Lexer("??").tokenize()
        assert len(tokens) == 2  # ?? y EOF
        assert tokens[0].kind == TokenKind.QUESTION_QUESTION
    
    def test_question_dot_vs_question_and_dot(self):
        """?. es un token único"""
        tokens = Lexer("?.").tokenize()
        assert len(tokens) == 2  # ?. y EOF
        assert tokens[0].kind == TokenKind.QUESTION_DOT


class TestOperatorsInExpressions:
    """Tests de operadores en expresiones reales."""
    
    def test_arithmetic_expression(self):
        code = "x + y * z"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.IDENTIFIER  # x
        assert tokens[1].kind == TokenKind.PLUS
        assert tokens[2].kind == TokenKind.IDENTIFIER  # y
        assert tokens[3].kind == TokenKind.STAR
        assert tokens[4].kind == TokenKind.IDENTIFIER  # z
    
    def test_comparison_expression(self):
        code = "age >= 18"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.IDENTIFIER  # age
        assert tokens[1].kind == TokenKind.GREATER_EQUAL
        assert tokens[2].kind == TokenKind.NUMBER_LITERAL  # 18
    
    def test_logical_expression(self):
        code = "x > 0 && y < 10"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[1].kind == TokenKind.GREATER
        assert tokens[3].kind == TokenKind.AMPERSAND_AMPERSAND
        assert tokens[5].kind == TokenKind.LESS
    
    def test_optional_chaining_expression(self):
        code = "user?.profile?.name"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.IDENTIFIER  # user
        assert tokens[1].kind == TokenKind.QUESTION_DOT
        assert tokens[2].kind == TokenKind.IDENTIFIER  # profile
        assert tokens[3].kind == TokenKind.QUESTION_DOT
        assert tokens[4].kind == TokenKind.IDENTIFIER  # name
    
    def test_lambda_expression(self):
        code = "(x, y) => x + y"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.LEFT_PAREN
        assert tokens[4].kind == TokenKind.RIGHT_PAREN
        assert tokens[5].kind == TokenKind.FAT_ARROW
    
    def test_function_return_type(self):
        code = "fn add(a: Number, b: Number) -> Number"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        # Buscar el ->
        arrow_token = [t for t in tokens if t.kind == TokenKind.ARROW]
        assert len(arrow_token) == 1
        assert arrow_token[0].lexeme == "->"
    
    def test_null_coalescing_expression(self):
        code = "value ?? default"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.IDENTIFIER  # value
        assert tokens[1].kind == TokenKind.QUESTION_QUESTION
        assert tokens[2].kind == TokenKind.IDENTIFIER  # default
    
    def test_assignment_with_operators(self):
        code = "count += 1"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.IDENTIFIER  # count
        assert tokens[1].kind == TokenKind.PLUS_EQUAL
        assert tokens[2].kind == TokenKind.NUMBER_LITERAL  # 1


class TestOperatorEdgeCases:
    """Tests de casos edge con operadores."""
    
    def test_operators_without_whitespace(self):
        """Operadores sin espacios deben tokenizarse correctamente."""
        code = "a+b*c/d-e"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.IDENTIFIER  # a
        assert tokens[1].kind == TokenKind.PLUS
        assert tokens[2].kind == TokenKind.IDENTIFIER  # b
        assert tokens[3].kind == TokenKind.STAR
        assert tokens[4].kind == TokenKind.IDENTIFIER  # c
        assert tokens[5].kind == TokenKind.SLASH
        assert tokens[6].kind == TokenKind.IDENTIFIER  # d
        assert tokens[7].kind == TokenKind.MINUS
        assert tokens[8].kind == TokenKind.IDENTIFIER  # e
    
    def test_minus_vs_arrow(self):
        """- seguido de > con espacio NO es ->"""
        code = "- >"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.MINUS
        assert tokens[1].kind == TokenKind.GREATER
    
    def test_equal_vs_equal_equal_vs_fat_arrow(self):
        """Distinguir =, ==, y =>"""
        code = "= == =>"
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.EQUAL
        assert tokens[1].kind == TokenKind.EQUAL_EQUAL
        assert tokens[2].kind == TokenKind.FAT_ARROW
    
    def test_star_vs_star_star_vs_star_equal(self):
        """Distinguir *, **, y *="""
        code = "* ** *="
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        
        assert tokens[0].kind == TokenKind.STAR
        assert tokens[1].kind == TokenKind.STAR_STAR
        assert tokens[2].kind == TokenKind.STAR_EQUAL


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
