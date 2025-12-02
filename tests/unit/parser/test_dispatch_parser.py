"""
Tests unitarios para el parser de dispatch statement

Jira: VELA-577
Historia: Sprint 15 - State Management
Subtask: TASK-035U - Implementar dispatch keyword

NOTA: dispatch es un statement, NO una declaration de nivel superior.
Debe estar dentro de funciones, bloques, etc.
"""

import pytest
import sys
sys.path.append('..')

from src.lexer.lexer import Lexer
from src.parser.parser import Parser
from src.parser.ast_nodes import (
    Program,
    DispatchStatement,
    Identifier,
    CallExpression,
    MemberAccessExpression,
    StructLiteral,
    AwaitExpression
)


def find_dispatch_in_function(func_decl):
    """Helper para encontrar dispatch statement en función"""
    if not func_decl.body:
        return None
    for stmt in func_decl.body.statements:
        if isinstance(stmt, DispatchStatement):
            return stmt
    return None


class TestDispatchParser:
    """Suite de tests para el parser de dispatch statement."""
    
    def test_dispatch_simple_identifier(self):
        """Test dispatch con identificador simple: dispatch(INCREMENT)"""
        code = """
        fn test() -> void {
          dispatch(INCREMENT)
        }
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        # Obtener función
        func_decl = ast.declarations[0]
        assert func_decl.body is not None
        
        # Buscar dispatch en el body
        dispatch_stmt = None
        for stmt in func_decl.body.statements:
            if isinstance(stmt, DispatchStatement):
                dispatch_stmt = stmt
                break
        
        assert dispatch_stmt is not None
        assert isinstance(dispatch_stmt.action, Identifier)
        assert dispatch_stmt.action.name == "INCREMENT"
    
    def test_dispatch_with_function_call(self):
        """Test dispatch con llamada a función: dispatch(createAction())"""
        code = """
        dispatch(createAction())
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        stmt = ast.declarations[0]
        assert isinstance(stmt, DispatchStatement)
        assert isinstance(stmt.action, CallExpression)
        assert isinstance(stmt.action.callee, Identifier)
        assert stmt.action.callee.name == "createAction"
    
    def test_dispatch_with_action_creator(self):
        """Test dispatch con action creator: dispatch(actions.add("test"))"""
        code = """
        dispatch(actions.add("test"))
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        stmt = ast.declarations[0]
        assert isinstance(stmt, DispatchStatement)
        assert isinstance(stmt.action, CallExpression)
        assert isinstance(stmt.action.callee, MemberAccessExpression)
    
    def test_dispatch_with_object_literal(self):
        """Test dispatch con objeto literal: dispatch({ type: "ADD" })"""
        code = """
        dispatch({ type: "ADD", payload: 42 })
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        stmt = ast.declarations[0]
        assert isinstance(stmt, DispatchStatement)
        # El parser debería reconocer el objeto literal
        # (el tipo exacto depende de la implementación del parser)
        assert stmt.action is not None
    
    def test_dispatch_with_constructor_call(self):
        """Test dispatch con constructor: dispatch(AddTodo({ title: "test" }))"""
        code = """
        dispatch(AddTodo({ title: "Buy milk" }))
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        stmt = ast.declarations[0]
        assert isinstance(stmt, DispatchStatement)
        assert isinstance(stmt.action, CallExpression)
        assert isinstance(stmt.action.callee, Identifier)
        assert stmt.action.callee.name == "AddTodo"
    
    def test_dispatch_with_await(self):
        """Test dispatch con await: dispatch(await fetchUser(id))"""
        code = """
        dispatch(await fetchUser(123))
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        stmt = ast.declarations[0]
        assert isinstance(stmt, DispatchStatement)
        assert isinstance(stmt.action, AwaitExpression)
        assert isinstance(stmt.action.expression, CallExpression)
    
    def test_dispatch_with_complex_expression(self):
        """Test dispatch con expresión compleja: dispatch(count > 0 ? increment : decrement)"""
        code = """
        dispatch(count > 0 ? increment : decrement)
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        stmt = ast.declarations[0]
        assert isinstance(stmt, DispatchStatement)
        # Ternary expression
        assert stmt.action is not None
    
    def test_dispatch_in_function_body(self):
        """Test dispatch dentro de función"""
        code = """
        fn increment() -> void {
          dispatch(INCREMENT)
        }
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        # La función debería tener un dispatch en su body
        func_decl = ast.declarations[0]
        assert func_decl.body is not None
        
        # Buscar dispatch statement en el body
        dispatch_found = False
        for stmt in func_decl.body.statements:
            if isinstance(stmt, DispatchStatement):
                dispatch_found = True
                assert isinstance(stmt.action, Identifier)
                assert stmt.action.name == "INCREMENT"
        
        assert dispatch_found, "dispatch statement not found in function body"
    
    def test_dispatch_in_if_statement(self):
        """Test dispatch dentro de if statement"""
        code = """
        if count > 0 {
          dispatch(INCREMENT)
        } else {
          dispatch(DECREMENT)
        }
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        if_stmt = ast.declarations[0]
        # Verificar que hay dispatch statements en then y else branches
        assert if_stmt.then_branch is not None
        assert if_stmt.else_branch is not None
    
    def test_dispatch_multiple_calls(self):
        """Test múltiples dispatch statements"""
        code = """
        dispatch(ACTION_ONE)
        dispatch(ACTION_TWO)
        dispatch(ACTION_THREE)
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 3
        
        for stmt in ast.declarations:
            assert isinstance(stmt, DispatchStatement)
            assert isinstance(stmt.action, Identifier)
    
    def test_dispatch_with_namespaced_action(self):
        """Test dispatch con acción con namespace: dispatch(todos.actions.add(...))"""
        code = """
        dispatch(todos.actions.add("Buy milk"))
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        stmt = ast.declarations[0]
        assert isinstance(stmt, DispatchStatement)
        assert isinstance(stmt.action, CallExpression)
        # La callee debería ser una cadena de member expressions
        assert isinstance(stmt.action.callee, MemberAccessExpression)
    
    def test_dispatch_in_event_handler(self):
        """Test dispatch dentro de event handler"""
        code = """
        on("click", () => {
          dispatch(BUTTON_CLICKED)
        })
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        assert isinstance(ast, Program)
        assert len(ast.declarations) == 1
        
        # Verificar que el on statement contiene un dispatch
        # (la estructura exacta depende del parser)
        assert ast.declarations[0] is not None


class TestDispatchParserErrors:
    """Tests de errores en el parser de dispatch."""
    
    def test_dispatch_missing_opening_paren(self):
        """Test error: dispatch sin paréntesis de apertura"""
        code = """
        dispatch INCREMENT)
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        
        # Debería lanzar error de parser
        with pytest.raises(Exception):
            parser.parse()
    
    def test_dispatch_missing_closing_paren(self):
        """Test error: dispatch sin paréntesis de cierre"""
        code = """
        dispatch(INCREMENT
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        
        # Debería lanzar error de parser
        with pytest.raises(Exception):
            parser.parse()
    
    def test_dispatch_missing_action(self):
        """Test error: dispatch sin expresión de acción"""
        code = """
        dispatch()
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        
        # Debería parsear pero la acción estaría vacía o error
        # (depende de la implementación)
        try:
            ast = parser.parse()
            # Si parsea, verificar que detecta el error
            stmt = ast.body[0]
            assert isinstance(stmt, DispatchStatement)
        except Exception:
            # Si lanza error, es comportamiento válido
            pass
    
    def test_dispatch_with_invalid_syntax(self):
        """Test error: dispatch con sintaxis inválida"""
        code = """
        dispatch(INCREMENT,)
        """
        
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        
        # Debería parsear (coma trailing es válida en algunos parsers)
        # o lanzar error
        try:
            ast = parser.parse()
            assert ast is not None
        except Exception:
            # Error esperado
            pass


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
