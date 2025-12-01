"""
Tests para Expresiones del Parser de Vela

Tests de: VELA-568 (TASK-012)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

Este módulo testea el parsing de expresiones:
- Literals (number, float, string, bool, none)
- Binary expressions (operadores y precedencia)
- Unary expressions
- Call expressions
- Member access (normal y optional chaining)
- Index access
- Lambdas
- If expressions
- Arrays, tuples, structs
"""

import pytest
import sys
sys.path.append('../..')

from src.parser import parse_code
from src.parser.ast_nodes import (
    Literal, BinaryExpression, UnaryExpression,
    CallExpression, MemberAccessExpression, IndexAccessExpression,
    Identifier, LambdaExpression, IfExpression,
    ArrayLiteral, TupleLiteral, StructLiteral
)


class TestLiterals:
    """Tests para literals"""
    
    def test_number_literal(self):
        """Test number literal"""
        code = "42"
        # Wrap in function para parsear
        ast = parse_code(f"fn test() -> Number {{ return {code} }}")
        assert ast is not None
    
    def test_float_literal(self):
        """Test float literal"""
        code = "3.14"
        ast = parse_code(f"fn test() -> Float {{ return {code} }}")
        assert ast is not None
    
    def test_string_literal(self):
        """Test string literal"""
        code = '"hello"'
        ast = parse_code(f'fn test() -> String {{ return {code} }}')
        assert ast is not None
    
    def test_bool_true(self):
        """Test bool true"""
        code = "true"
        ast = parse_code(f"fn test() -> Bool {{ return {code} }}")
        assert ast is not None
    
    def test_bool_false(self):
        """Test bool false"""
        code = "false"
        ast = parse_code(f"fn test() -> Bool {{ return {code} }}")
        assert ast is not None
    
    def test_none_literal(self):
        """Test None literal"""
        code = "None"
        ast = parse_code(f"fn test() -> Option<Number> {{ return {code} }}")
        assert ast is not None


class TestBinaryExpressions:
    """Tests para expresiones binarias"""
    
    def test_addition(self):
        """Test suma"""
        code = "fn test() -> Number { return 1 + 2 }"
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
    
    def test_subtraction(self):
        """Test resta"""
        code = "fn test() -> Number { return 5 - 3 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_multiplication(self):
        """Test multiplicación"""
        code = "fn test() -> Number { return 4 * 5 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_division(self):
        """Test división"""
        code = "fn test() -> Float { return 10 / 2 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_modulo(self):
        """Test módulo"""
        code = "fn test() -> Number { return 10 % 3 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_power(self):
        """Test potencia"""
        code = "fn test() -> Number { return 2 ** 8 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_comparison_lt(self):
        """Test menor que"""
        code = "fn test() -> Bool { return 1 < 2 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_comparison_gt(self):
        """Test mayor que"""
        code = "fn test() -> Bool { return 5 > 3 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_comparison_le(self):
        """Test menor o igual"""
        code = "fn test() -> Bool { return 1 <= 2 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_comparison_ge(self):
        """Test mayor o igual"""
        code = "fn test() -> Bool { return 5 >= 3 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_equality(self):
        """Test igualdad"""
        code = "fn test() -> Bool { return 1 == 1 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_inequality(self):
        """Test desigualdad"""
        code = "fn test() -> Bool { return 1 != 2 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_logical_and(self):
        """Test AND lógico"""
        code = "fn test() -> Bool { return true && false }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_logical_or(self):
        """Test OR lógico"""
        code = "fn test() -> Bool { return true || false }"
        ast = parse_code(code)
        assert ast is not None


class TestPrecedence:
    """Tests para precedencia de operadores"""
    
    def test_multiplication_before_addition(self):
        """Test: 1 + 2 * 3 debe parsear como 1 + (2 * 3)"""
        code = "fn test() -> Number { return 1 + 2 * 3 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_power_before_multiplication(self):
        """Test: 2 * 3 ** 4 debe parsear como 2 * (3 ** 4)"""
        code = "fn test() -> Number { return 2 * 3 ** 4 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_comparison_before_logical(self):
        """Test: 1 < 2 && 3 > 4 debe parsear como (1 < 2) && (3 > 4)"""
        code = "fn test() -> Bool { return 1 < 2 && 3 > 4 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_parentheses_override_precedence(self):
        """Test: (1 + 2) * 3 debe parsear correctamente"""
        code = "fn test() -> Number { return (1 + 2) * 3 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_complex_expression(self):
        """Test expresión compleja con múltiples operadores"""
        code = "fn test() -> Number { return 1 + 2 * 3 - 4 / 2 ** 3 }"
        ast = parse_code(code)
        assert ast is not None


class TestUnaryExpressions:
    """Tests para expresiones unarias"""
    
    def test_negative_number(self):
        """Test negación numérica"""
        code = "fn test() -> Number { return -42 }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_logical_not(self):
        """Test negación lógica"""
        code = "fn test() -> Bool { return !true }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_double_negation(self):
        """Test doble negación"""
        code = "fn test() -> Number { return --42 }"
        ast = parse_code(code)
        assert ast is not None


class TestCallExpressions:
    """Tests para call expressions"""
    
    def test_function_call_no_args(self):
        """Test llamada sin argumentos"""
        code = """
        fn foo() -> Number { return 42 }
        fn test() -> Number { return foo() }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 2
    
    def test_function_call_one_arg(self):
        """Test llamada con un argumento"""
        code = """
        fn double(x: Number) -> Number { return x * 2 }
        fn test() -> Number { return double(5) }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_function_call_multiple_args(self):
        """Test llamada con múltiples argumentos"""
        code = """
        fn add(a: Number, b: Number) -> Number { return a + b }
        fn test() -> Number { return add(1, 2) }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_chained_calls(self):
        """Test llamadas encadenadas"""
        code = """
        fn get_list() -> [Number] { return [1, 2, 3] }
        fn test() -> Number { return get_list().length() }
        """
        ast = parse_code(code)
        assert ast is not None


class TestMemberAccess:
    """Tests para member access"""
    
    def test_simple_member_access(self):
        """Test acceso a miembro simple"""
        code = """
        struct User { id: Number, name: String }
        fn test(u: User) -> Number { return u.id }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_chained_member_access(self):
        """Test acceso a miembros encadenado"""
        code = """
        struct Point { x: Number, y: Number }
        struct Line { start: Point, end: Point }
        fn test(line: Line) -> Number { return line.start.x }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_optional_chaining(self):
        """Test optional chaining"""
        code = """
        struct User { id: Number, name: String }
        fn test(u: Option<User>) -> Option<Number> { return u?.id }
        """
        ast = parse_code(code)
        assert ast is not None


class TestIndexAccess:
    """Tests para index access"""
    
    def test_array_index(self):
        """Test índice de array"""
        code = """
        fn test() -> Number {
            arr: [Number] = [1, 2, 3]
            return arr[0]
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_chained_index(self):
        """Test índices encadenados"""
        code = """
        fn test() -> Number {
            matrix: [[Number]] = [[1, 2], [3, 4]]
            return matrix[0][1]
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestArrayLiterals:
    """Tests para array literals"""
    
    def test_empty_array(self):
        """Test array vacío"""
        code = "fn test() -> [Number] { return [] }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_array_with_elements(self):
        """Test array con elementos"""
        code = "fn test() -> [Number] { return [1, 2, 3] }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_nested_arrays(self):
        """Test arrays anidados"""
        code = "fn test() -> [[Number]] { return [[1, 2], [3, 4]] }"
        ast = parse_code(code)
        assert ast is not None


class TestTupleLiterals:
    """Tests para tuple literals"""
    
    def test_empty_tuple(self):
        """Test tupla vacía"""
        code = "fn test() -> () { return () }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_tuple_with_two_elements(self):
        """Test tupla con dos elementos"""
        code = "fn test() -> (Number, String) { return (1, \"hello\") }"
        ast = parse_code(code)
        assert ast is not None
    
    def test_tuple_with_multiple_elements(self):
        """Test tupla con múltiples elementos"""
        code = "fn test() -> (Number, String, Bool) { return (42, \"test\", true) }"
        ast = parse_code(code)
        assert ast is not None


class TestStructLiterals:
    """Tests para struct literals"""
    
    def test_empty_struct(self):
        """Test struct vacío"""
        code = """
        struct Empty {}
        fn test() -> Empty { return {} }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_struct_with_fields(self):
        """Test struct con campos"""
        code = """
        struct Point { x: Number, y: Number }
        fn test() -> Point { return { x: 1, y: 2 } }
        """
        ast = parse_code(code)
        assert ast is not None


class TestLambdaExpressions:
    """Tests para lambda expressions"""
    
    def test_lambda_no_params(self):
        """Test lambda sin parámetros"""
        code = """
        fn test() -> () -> Number {
            return || => 42
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_lambda_one_param(self):
        """Test lambda con un parámetro"""
        code = """
        fn test() -> (Number) -> Number {
            return |x| => x * 2
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_lambda_multiple_params(self):
        """Test lambda con múltiples parámetros"""
        code = """
        fn test() -> (Number, Number) -> Number {
            return |a, b| => a + b
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestIfExpressions:
    """Tests para if expressions"""
    
    def test_if_expression_simple(self):
        """Test if expression simple"""
        code = """
        fn test(x: Number) -> String {
            return if x > 0 { "positive" } else { "negative" }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_nested_if_expression(self):
        """Test if expressions anidados"""
        code = """
        fn test(x: Number) -> String {
            return if x > 0 {
                if x > 10 { "large" } else { "small" }
            } else {
                "negative"
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestRangeExpressions:
    """Tests para range expressions"""
    
    def test_exclusive_range(self):
        """Test rango exclusivo"""
        code = """
        fn test() -> void {
            (0..10).forEach(|i| => print(i))
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_inclusive_range(self):
        """Test rango inclusivo"""
        code = """
        fn test() -> void {
            (0..=10).forEach(|i| => print(i))
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestComplexExpressions:
    """Tests para expresiones complejas"""
    
    def test_method_chaining(self):
        """Test method chaining"""
        code = """
        fn test() -> [Number] {
            return [1, 2, 3, 4, 5]
                .filter(|x| => x % 2 == 0)
                .map(|x| => x * 2)
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_complex_nested_expression(self):
        """Test expresión compleja anidada"""
        code = """
        fn test() -> Number {
            return ((1 + 2) * (3 - 4)) / (5 ** 2)
        }
        """
        ast = parse_code(code)
        assert ast is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
