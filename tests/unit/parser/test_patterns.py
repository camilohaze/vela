"""
Tests para Pattern Matching del Parser de Vela

Tests de: VELA-568 (TASK-012)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

Este módulo testea el parsing de patterns en match expressions:
- Literal patterns
- Identifier patterns
- Wildcard patterns
- Tuple patterns
- Struct patterns
- Enum patterns
- Or patterns
- Range patterns
"""

import pytest
import sys
sys.path.append('../..')

from src.parser import parse_code
from src.parser.ast_nodes import (
    LiteralPattern, IdentifierPattern, WildcardPattern,
    TuplePattern, StructPattern, EnumPattern, OrPattern, RangePattern
)


class TestLiteralPatterns:
    """Tests para literal patterns"""
    
    def test_number_pattern(self):
        """Test pattern number literal"""
        code = """
        fn test() -> void {
            match value {
                0 => print("zero")
                1 => print("one")
                _ => print("other")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_string_pattern(self):
        """Test pattern string literal"""
        code = """
        fn test() -> void {
            match status {
                "active" => print("Active")
                "inactive" => print("Inactive")
                _ => print("Unknown")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_bool_pattern(self):
        """Test pattern bool literal"""
        code = """
        fn test() -> void {
            match flag {
                true => print("Enabled")
                false => print("Disabled")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestIdentifierPatterns:
    """Tests para identifier patterns (binding)"""
    
    def test_simple_binding(self):
        """Test binding simple"""
        code = """
        fn test() -> void {
            match value {
                x => print("Got: ${x}")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_binding_with_type(self):
        """Test binding con tipo"""
        code = """
        fn test() -> void {
            match value {
                x: Number => print("Number: ${x}")
                s: String => print("String: ${s}")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestWildcardPattern:
    """Tests para wildcard pattern (_)"""
    
    def test_simple_wildcard(self):
        """Test wildcard simple"""
        code = """
        fn test() -> void {
            match value {
                1 => print("one")
                _ => print("anything else")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_wildcard_as_catch_all(self):
        """Test wildcard como catch-all (obligatorio)"""
        code = """
        fn test() -> void {
            match value {
                Some(_) => print("has value")
                None => print("no value")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestTuplePatterns:
    """Tests para tuple patterns"""
    
    def test_simple_tuple_pattern(self):
        """Test tuple pattern simple"""
        code = """
        fn test() -> void {
            match point {
                (0, 0) => print("origin")
                (x, y) => print("point at (${x}, ${y})")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_nested_tuple_pattern(self):
        """Test tuple pattern anidado"""
        code = """
        fn test() -> void {
            match data {
                ((x, y), z) => print("nested")
                _ => print("other")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_tuple_with_wildcard(self):
        """Test tuple con wildcard"""
        code = """
        fn test() -> void {
            match pair {
                (x, _) => print("first: ${x}")
                _ => print("other")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestStructPatterns:
    """Tests para struct patterns"""
    
    def test_simple_struct_pattern(self):
        """Test struct pattern simple"""
        code = """
        fn test() -> void {
            match user {
                User { id: 0, name } => print("Default user")
                User { id, name } => print("User ${id}: ${name}")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_nested_struct_pattern(self):
        """Test struct pattern anidado"""
        code = """
        fn test() -> void {
            match request {
                Request { user: User { id, name }, path } => {
                    print("Request from ${name} to ${path}")
                }
                _ => print("other")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_struct_with_rest(self):
        """Test struct pattern con rest (..)"""
        code = """
        fn test() -> void {
            match user {
                User { id, .. } => print("User ${id}")
                _ => print("other")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestEnumPatterns:
    """Tests para enum patterns"""
    
    def test_simple_enum_pattern(self):
        """Test enum pattern simple"""
        code = """
        fn test() -> void {
            match color {
                Color::Red => print("Red")
                Color::Green => print("Green")
                Color::Blue => print("Blue")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_enum_with_data(self):
        """Test enum pattern con datos"""
        code = """
        fn test() -> void {
            match result {
                Ok(value) => print("Success: ${value}")
                Err(error) => print("Error: ${error}")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_option_enum_pattern(self):
        """Test Option<T> pattern"""
        code = """
        fn test() -> void {
            match maybeValue {
                Some(x) => print("Has value: ${x}")
                None => print("No value")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestOrPatterns:
    """Tests para or patterns (|)"""
    
    def test_simple_or_pattern(self):
        """Test or pattern simple"""
        code = """
        fn test() -> void {
            match value {
                1 | 2 | 3 => print("one, two, or three")
                _ => print("other")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_or_pattern_with_strings(self):
        """Test or pattern con strings"""
        code = """
        fn test() -> void {
            match status {
                "active" | "running" => print("Active")
                "inactive" | "stopped" => print("Inactive")
                _ => print("Unknown")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestRangePatterns:
    """Tests para range patterns"""
    
    def test_exclusive_range_pattern(self):
        """Test range exclusivo (..)"""
        code = """
        fn test() -> void {
            match age {
                0..18 => print("minor")
                18..65 => print("adult")
                _ => print("senior")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_inclusive_range_pattern(self):
        """Test range inclusivo (..=)"""
        code = """
        fn test() -> void {
            match score {
                0..=59 => print("F")
                60..=69 => print("D")
                70..=79 => print("C")
                80..=89 => print("B")
                90..=100 => print("A")
                _ => print("Invalid")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestPatternGuards:
    """Tests para pattern guards (if clause)"""
    
    def test_simple_guard(self):
        """Test guard simple"""
        code = """
        fn test() -> void {
            match number {
                n if n < 0 => print("negative")
                n if n == 0 => print("zero")
                n if n > 0 => print("positive")
                _ => print("unreachable")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_guard_with_complex_condition(self):
        """Test guard con condición compleja"""
        code = """
        fn test() -> void {
            match user {
                User { age, .. } if age >= 18 && age < 65 => print("adult")
                User { age, .. } if age < 18 => print("minor")
                _ => print("senior")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestComplexPatterns:
    """Tests para patterns complejos"""
    
    def test_nested_enum_struct_pattern(self):
        """Test pattern anidado: enum + struct"""
        code = """
        fn test() -> void {
            match response {
                Ok(Response { status: 200, body }) => {
                    print("Success: ${body}")
                }
                Ok(Response { status, .. }) => {
                    print("Status: ${status}")
                }
                Err(error) => print("Error: ${error}")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_pattern_with_or_and_guard(self):
        """Test pattern con OR y guard combinados"""
        code = """
        fn test() -> void {
            match value {
                x | y if x > 10 || y > 10 => print("large")
                _ => print("small")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_deeply_nested_pattern(self):
        """Test pattern profundamente anidado"""
        code = """
        fn test() -> void {
            match data {
                Some(Ok(User { profile: Profile { name, age }, .. })) => {
                    print("User: ${name}, ${age}")
                }
                Some(Err(e)) => print("Error: ${e}")
                None => print("No data")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
