"""
Tests para Statements del Parser de Vela

Tests de: VELA-568 (TASK-012)
Historia: Sprint 6 - Parser que genere AST válido
Fecha: 2025-12-01

Este módulo testea el parsing de statements:
- Variables (inmutables, con state)
- Assignments
- If statements
- Match statements  
- Try-catch
- Return/throw
- Blocks
"""

import pytest
import sys
sys.path.append('../..')

from src.parser import parse_code
from src.parser.ast_nodes import (
    VariableStatement, AssignmentStatement, IfStatement,
    MatchStatement, TryStatement, ReturnStatement, ThrowStatement
)


class TestVariableStatements:
    """Tests para variable statements"""
    
    def test_simple_variable(self):
        """Test variable inmutable simple"""
        code = """
        fn test() -> void {
            x: Number = 42
        }
        """
        ast = parse_code(code)
        assert ast is not None
        func = ast.declarations[0]
        assert len(func.body.statements) == 1
        assert isinstance(func.body.statements[0], VariableStatement)
    
    def test_variable_without_type(self):
        """Test variable sin anotación de tipo"""
        code = """
        fn test() -> void {
            name = "Vela"
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_state_variable(self):
        """Test variable mutable con state"""
        code = """
        fn test() -> void {
            state counter: Number = 0
        }
        """
        ast = parse_code(code)
        assert ast is not None
        var = ast.declarations[0].body.statements[0]
        assert var.is_mutable == True
    
    def test_multiple_variables(self):
        """Test múltiples variables"""
        code = """
        fn test() -> void {
            x: Number = 1
            y: Number = 2
            z: Number = x + y
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations[0].body.statements) == 3


class TestAssignmentStatements:
    """Tests para assignments"""
    
    def test_simple_assignment(self):
        """Test assignment simple"""
        code = """
        fn test() -> void {
            state x: Number = 0
            x = 42
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0].body.statements[1], AssignmentStatement)
    
    def test_member_assignment(self):
        """Test assignment a member"""
        code = """
        fn test() -> void {
            obj.field = 10
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_index_assignment(self):
        """Test assignment a índice"""
        code = """
        fn test() -> void {
            arr[0] = 100
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestIfStatements:
    """Tests para if statements"""
    
    def test_simple_if(self):
        """Test if simple"""
        code = """
        fn test() -> void {
            if x > 0 {
                print("positive")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0].body.statements[0], IfStatement)
    
    def test_if_else(self):
        """Test if-else"""
        code = """
        fn test() -> void {
            if x > 0 {
                print("positive")
            } else {
                print("negative or zero")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        if_stmt = ast.declarations[0].body.statements[0]
        assert if_stmt.else_body is not None
    
    def test_if_elif_else(self):
        """Test if-elif-else"""
        code = """
        fn test() -> void {
            if x > 0 {
                print("positive")
            } else if x < 0 {
                print("negative")
            } else {
                print("zero")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_nested_if(self):
        """Test if anidados"""
        code = """
        fn test() -> void {
            if x > 0 {
                if y > 0 {
                    print("both positive")
                }
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestMatchStatements:
    """Tests para match statements"""
    
    def test_simple_match(self):
        """Test match simple"""
        code = """
        fn test() -> void {
            match value {
                1 => print("one")
                2 => print("two")
                _ => print("other")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0].body.statements[0], MatchStatement)
    
    def test_match_with_destructuring(self):
        """Test match con destructuring"""
        code = """
        fn test() -> void {
            match point {
                { x: 0, y: 0 } => print("origin")
                { x, y } => print("point at (${x}, ${y})")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_match_enum_variants(self):
        """Test match con enum variants"""
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
    
    def test_match_with_guards(self):
        """Test match con guards"""
        code = """
        fn test() -> void {
            match number {
                n if n < 0 => print("negative")
                n if n == 0 => print("zero")
                n => print("positive")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestTryStatements:
    """Tests para try-catch"""
    
    def test_simple_try_catch(self):
        """Test try-catch simple"""
        code = """
        fn test() -> void {
            try {
                riskyOperation()
            } catch (e) {
                print("Error: ${e}")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0].body.statements[0], TryStatement)
    
    def test_try_catch_finally(self):
        """Test try-catch-finally"""
        code = """
        fn test() -> void {
            try {
                riskyOperation()
            } catch (e) {
                print("Error")
            } finally {
                cleanup()
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        try_stmt = ast.declarations[0].body.statements[0]
        assert try_stmt.finally_body is not None
    
    def test_multiple_catch_blocks(self):
        """Test múltiples catch"""
        code = """
        fn test() -> void {
            try {
                riskyOperation()
            } catch (e: NetworkError) {
                print("Network error")
            } catch (e: ParseError) {
                print("Parse error")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestReturnStatements:
    """Tests para return statements"""
    
    def test_return_with_value(self):
        """Test return con valor"""
        code = """
        fn add(a: Number, b: Number) -> Number {
            return a + b
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0].body.statements[0], ReturnStatement)
    
    def test_return_without_value(self):
        """Test return sin valor"""
        code = """
        fn log() -> void {
            return
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_early_return(self):
        """Test early return"""
        code = """
        fn process(x: Number) -> Number {
            if x < 0 {
                return 0
            }
            return x * 2
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestThrowStatements:
    """Tests para throw statements"""
    
    def test_simple_throw(self):
        """Test throw simple"""
        code = """
        fn fail() -> never {
            throw Error("failed")
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert isinstance(ast.declarations[0].body.statements[0], ThrowStatement)
    
    def test_throw_custom_error(self):
        """Test throw error custom"""
        code = """
        fn validate() -> void {
            if !isValid {
                throw ValidationError("invalid input")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestBlockStatements:
    """Tests para blocks"""
    
    def test_empty_block(self):
        """Test block vacío"""
        code = """
        fn test() -> void {
            {}
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_nested_blocks(self):
        """Test blocks anidados"""
        code = """
        fn test() -> void {
            {
                x: Number = 1
                {
                    y: Number = 2
                }
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


class TestComplexStatements:
    """Tests para statements complejos"""
    
    def test_loop_with_functional(self):
        """Test usar métodos funcionales (NO for loops)"""
        code = """
        fn test() -> void {
            (0..10).forEach(i => {
                print(i)
            })
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_match_with_complex_patterns(self):
        """Test match con patterns complejos"""
        code = """
        fn test() -> void {
            match value {
                Some(User { id, name }) => print("User: ${name}")
                Some(_) => print("Some other value")
                None => print("Nothing")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
    
    def test_nested_control_flow(self):
        """Test control flow anidado"""
        code = """
        fn test() -> void {
            if condition {
                match result {
                    Ok(val) => {
                        try {
                            process(val)
                        } catch (e) {
                            print("Error")
                        }
                    }
                    Err(e) => print("Failed")
                }
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
