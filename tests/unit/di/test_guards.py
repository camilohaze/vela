"""
Tests unitarios para decorador @guard

Fase 4: Guards - NestJS-style
Testing de:
- GuardMetadata validaciones
- Decorador @guard para backend HTTP
- ExecutionContext interface
- Validaciones de contexto estrictas
- Helper functions

Jira: TASK-035E2 (Fase 4)
Historia: VELA-575
"""

import pytest
from src.runtime.di.guards import (
    guard,
    GuardMetadata,
    ExecutionContext,
    is_guard,
    get_guard_metadata,
    get_guard_classes,
    combine_guards,
    validate_guard_class,
)


# ============================================================================
# TEST FIXTURES
# ============================================================================


class AuthGuard:
    """Mock auth guard."""
    def canActivate(self, context):
        return True


class RolesGuard:
    """Mock roles guard."""
    def canActivate(self, context):
        return True


class AdminGuard:
    """Mock admin guard."""
    def canActivate(self, context):
        return True


# ============================================================================
# TEST ExecutionContext
# ============================================================================


class TestExecutionContext:
    """Suite de tests para ExecutionContext."""
    
    def test_initialization_default(self):
        """Test inicialización con valores por defecto."""
        context = ExecutionContext()
        
        assert context.request is None
        assert context.response is None
        assert context.handler is None
        assert context.metadata == {}
    
    def test_initialization_custom(self):
        """Test inicialización con valores custom."""
        mock_request = {"path": "/users"}
        mock_response = {"status": 200}
        mock_handler = lambda: None
        mock_metadata = {"roles": ["admin"]}
        
        context = ExecutionContext(
            request=mock_request,
            response=mock_response,
            handler=mock_handler,
            metadata=mock_metadata
        )
        
        assert context.request == mock_request
        assert context.response == mock_response
        assert context.handler == mock_handler
        assert context.metadata == mock_metadata
    
    def test_get_handler(self):
        """Test get_handler method."""
        mock_handler = lambda: None
        context = ExecutionContext(handler=mock_handler)
        
        assert context.get_handler() == mock_handler
    
    def test_switch_to_http(self):
        """Test switch_to_http method."""
        context = ExecutionContext()
        http_context = context.switch_to_http()
        
        assert http_context is context


# ============================================================================
# TEST GuardMetadata
# ============================================================================


class TestGuardMetadata:
    """Suite de tests para GuardMetadata."""
    
    def test_initialization_default(self):
        """Test inicialización con valores por defecto."""
        metadata = GuardMetadata(guard_classes=[AuthGuard])
        
        assert metadata.guard_classes == [AuthGuard]
        assert metadata.options == {}
    
    def test_initialization_custom(self):
        """Test inicialización con valores custom."""
        metadata = GuardMetadata(
            guard_classes=[AuthGuard, RolesGuard],
            options={"roles": ["admin", "user"]}
        )
        
        assert metadata.guard_classes == [AuthGuard, RolesGuard]
        assert metadata.options == {"roles": ["admin", "user"]}
    
    def test_validation_empty_guard_classes(self):
        """Test error si guard_classes está vacío."""
        with pytest.raises(ValueError, match="must have at least one guard class"):
            GuardMetadata(guard_classes=[])
    
    def test_validation_non_type_guard_classes(self):
        """Test error si guard_classes contiene no-Type."""
        with pytest.raises(TypeError, match="must be Type classes"):
            GuardMetadata(guard_classes=["not a type"])
    
    def test_multiple_guard_classes(self):
        """Test múltiples guard classes."""
        metadata = GuardMetadata(
            guard_classes=[AuthGuard, RolesGuard, AdminGuard]
        )
        
        assert len(metadata.guard_classes) == 3
        assert AuthGuard in metadata.guard_classes
        assert RolesGuard in metadata.guard_classes
        assert AdminGuard in metadata.guard_classes


# ============================================================================
# TEST @guard DECORATOR
# ============================================================================


class TestGuardDecorator:
    """Suite de tests para @guard decorator."""
    
    def test_guard_on_controller_class(self):
        """Test @guard en clase controller."""
        @guard(AuthGuard, RolesGuard)
        class UserController:
            __controller_metadata__ = {'path': '/users'}
        
        assert is_guard(UserController)
        metadata = get_guard_metadata(UserController)
        assert metadata is not None
        assert metadata.guard_classes == [AuthGuard, RolesGuard]
    
    def test_guard_on_route_handler(self):
        """Test @guard en route handler."""
        @guard(AdminGuard)
        def delete_user(id):
            __route_metadata__ = {'method': 'DELETE', 'path': '/users/:id'}
            return {"deleted": id}
        
        assert is_guard(delete_user)
        metadata = get_guard_metadata(delete_user)
        assert metadata.guard_classes == [AdminGuard]
    
    def test_guard_with_options(self):
        """Test @guard con opciones."""
        @guard(RolesGuard, roles=["admin", "moderator"])
        class AdminController:
            __controller_metadata__ = {'path': '/admin'}
        
        metadata = get_guard_metadata(AdminController)
        assert metadata.options == {"roles": ["admin", "moderator"]}
    
    def test_single_guard(self):
        """Test @guard con un solo guard."""
        @guard(AuthGuard)
        class SimpleController:
            __controller_metadata__ = {'path': '/simple'}
        
        metadata = get_guard_metadata(SimpleController)
        assert len(metadata.guard_classes) == 1
        assert metadata.guard_classes[0] == AuthGuard
    
    def test_error_on_ui_pipe(self):
        """Test error si se usa en UI pipe."""
        with pytest.raises(ValueError, match="cannot be used on UI pipes"):
            @guard(AuthGuard)
            class TestPipe:
                __ui_pipe_metadata__ = {'name': 'test'}
    
    def test_error_on_non_controller_class(self):
        """Test error si se usa en clase sin @controller."""
        with pytest.raises(ValueError, match="can only be used with @controller"):
            @guard(AuthGuard)
            class RegularClass:
                pass
    
    def test_error_on_non_callable(self):
        """Test error si target no es callable ni class."""
        with pytest.raises(ValueError, match="can only be used on classes or functions"):
            decorated = guard(AuthGuard)("not a callable")


# ============================================================================
# TEST HELPER FUNCTIONS
# ============================================================================


class TestHelperFunctions:
    """Suite de tests para helper functions."""
    
    def test_is_guard_true(self):
        """Test is_guard retorna True."""
        @guard(AuthGuard)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        assert is_guard(TestController) is True
    
    def test_is_guard_false(self):
        """Test is_guard retorna False."""
        class RegularClass:
            pass
        
        assert is_guard(RegularClass) is False
    
    def test_get_guard_metadata_exists(self):
        """Test get_guard_metadata cuando existe."""
        @guard(AuthGuard)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        metadata = get_guard_metadata(TestController)
        assert metadata is not None
        assert isinstance(metadata, GuardMetadata)
    
    def test_get_guard_metadata_none(self):
        """Test get_guard_metadata cuando no existe."""
        class RegularClass:
            pass
        
        metadata = get_guard_metadata(RegularClass)
        assert metadata is None
    
    def test_get_guard_classes(self):
        """Test get_guard_classes."""
        @guard(AuthGuard, RolesGuard)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        classes = get_guard_classes(TestController)
        assert classes == [AuthGuard, RolesGuard]
    
    def test_get_guard_classes_none(self):
        """Test get_guard_classes sin guards."""
        class RegularClass:
            pass
        
        classes = get_guard_classes(RegularClass)
        assert classes == []
    
    def test_combine_guards(self):
        """Test combine_guards."""
        @guard(AuthGuard)
        class Controller:
            __controller_metadata__ = {'path': '/api'}
        
        @guard(RolesGuard)
        def handler():
            pass
        
        combined = combine_guards(Controller, handler)
        # Guards del controller van primero
        assert combined == [AuthGuard, RolesGuard]
    
    def test_combine_guards_empty(self):
        """Test combine_guards sin guards."""
        class NoGuard:
            pass
        
        combined = combine_guards(NoGuard)
        assert combined == []
    
    def test_validate_guard_class_valid(self):
        """Test validate_guard_class con clase válida."""
        # No debe lanzar excepción
        validate_guard_class(AuthGuard)
    
    def test_validate_guard_class_missing_canActivate(self):
        """Test validate_guard_class sin método canActivate."""
        class InvalidGuard:
            pass
        
        with pytest.raises(TypeError, match="must implement canActivate"):
            validate_guard_class(InvalidGuard)
    
    def test_validate_guard_class_non_callable_canActivate(self):
        """Test validate_guard_class con canActivate no-callable."""
        class InvalidGuard:
            canActivate = "not a method"
        
        with pytest.raises(TypeError, match="must be a callable method"):
            validate_guard_class(InvalidGuard)


# ============================================================================
# TEST INTEGRATION
# ============================================================================


class TestIntegration:
    """Tests de integración para guards."""
    
    def test_controller_with_multiple_guards(self):
        """Test controller con múltiples guards."""
        @guard(AuthGuard, RolesGuard, AdminGuard)
        class ComplexController:
            __controller_metadata__ = {'path': '/complex'}
        
        assert is_guard(ComplexController)
        metadata = get_guard_metadata(ComplexController)
        
        assert len(metadata.guard_classes) == 3
        assert AuthGuard in metadata.guard_classes
        assert RolesGuard in metadata.guard_classes
        assert AdminGuard in metadata.guard_classes
    
    def test_guard_chain_ordering(self):
        """Test ordenamiento de cadena de guards."""
        @guard(AuthGuard)
        class Controller:
            __controller_metadata__ = {'path': '/api'}
            
            @guard(RolesGuard)
            def secure_endpoint(self):
                pass
            
            @guard(AdminGuard)
            def admin_endpoint(self):
                pass
        
        # Combinar guards de controller y método secure
        combined_secure = combine_guards(Controller, Controller.secure_endpoint)
        assert combined_secure == [AuthGuard, RolesGuard]
        
        # Combinar guards de controller y método admin
        combined_admin = combine_guards(Controller, Controller.admin_endpoint)
        assert combined_admin == [AuthGuard, AdminGuard]


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
