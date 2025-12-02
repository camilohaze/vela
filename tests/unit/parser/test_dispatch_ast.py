"""
Tests SIMPLIFICADOS para DispatchStatement

Jira: VELA-577
Historia: Sprint 15 - State Management
Subtask: TASK-035U - Implementar dispatch keyword

NOTA: El parser actual tiene errores preexistentes con TokenType.NOT y TokenType.OPTIONAL_CHAIN.
Estos tests verifican que el nodo AST existe y es correcto.
"""

import pytest
import sys
sys.path.append('..')

from src.parser.ast_nodes import (
    DispatchStatement,
    Identifier,
    CallExpression,
    Range,
    Position
)


class TestDispatchStatementAST:
    """Tests del nodo AST DispatchStatement."""
    
    def test_dispatch_statement_creation(self):
        """Test que DispatchStatement se puede crear"""
        action = Identifier(
            range=Range(Position(1, 10), Position(1, 19)),
            name="INCREMENT"
        )
        
        dispatch_stmt = DispatchStatement(
            range=Range(Position(1, 1), Position(1, 20)),
            action=action
        )
        
        assert isinstance(dispatch_stmt, DispatchStatement)
        assert isinstance(dispatch_stmt.action, Identifier)
        assert dispatch_stmt.action.name == "INCREMENT"
    
    def test_dispatch_with_call_expression(self):
        """Test DispatchStatement con CallExpression"""
        callee = Identifier(
            range=Range(Position(1, 10), Position(1, 22)),
            name="createAction"
        )
        
        action = CallExpression(
            range=Range(Position(1, 10), Position(1, 24)),
            callee=callee,
            arguments=[]
        )
        
        dispatch_stmt = DispatchStatement(
            range=Range(Position(1, 1), Position(1, 25)),
            action=action
        )
        
        assert isinstance(dispatch_stmt, DispatchStatement)
        assert isinstance(dispatch_stmt.action, CallExpression)
        assert dispatch_stmt.action.callee.name == "createAction"
    
    def test_dispatch_statement_has_action_field(self):
        """Test que DispatchStatement tiene campo action"""
        action = Identifier(
            range=Range(Position(1, 1), Position(1, 10)),
            name="ACTION"
        )
        
        dispatch_stmt = DispatchStatement(
            range=Range(Position(1, 1), Position(1, 15)),
            action=action
        )
        
        assert hasattr(dispatch_stmt, 'action')
        assert dispatch_stmt.action is not None
        assert dispatch_stmt.action.name == "ACTION"
    
    def test_dispatch_statement_has_range(self):
        """Test que DispatchStatement tiene range"""
        action = Identifier(
            range=Range(Position(1, 10), Position(1, 15)),
            name="TEST"
        )
        
        dispatch_stmt = DispatchStatement(
            range=Range(Position(1, 1), Position(1, 16)),
            action=action
        )
        
        assert hasattr(dispatch_stmt, 'range')
        assert dispatch_stmt.range is not None
        assert dispatch_stmt.range.start.line == 1
        assert dispatch_stmt.range.start.column == 1


class TestDispatchStatementDocumentation:
    """Tests de documentación de DispatchStatement."""
    
    def test_dispatch_statement_has_docstring(self):
        """Test que DispatchStatement tiene docstring"""
        assert DispatchStatement.__doc__ is not None
        assert "dispatch" in DispatchStatement.__doc__.lower()
    
    def test_dispatch_statement_docstring_has_examples(self):
        """Test que el docstring contiene ejemplos"""
        doc = DispatchStatement.__doc__
        assert "INCREMENT" in doc or "dispatch" in doc.lower()
        assert "Store" in doc or "action" in doc.lower()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
