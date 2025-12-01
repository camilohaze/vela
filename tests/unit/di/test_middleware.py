"""
Tests unitarios para decorador @middleware

Fase 3: Middleware - NestJS-style
Testing de:
- MiddlewareMetadata validaciones
- Decorador @middleware para backend HTTP
- Validaciones de contexto estrictas
- Orden de ejecución
- Helper functions

Jira: TASK-035E2 (Fase 3)
Historia: VELA-575
"""

import pytest
from src.runtime.di.middleware import (
    middleware,
    MiddlewareMetadata,
    is_middleware,
    get_middleware_metadata,
    get_middleware_classes,
    get_middleware_order,
    combine_middleware,
    validate_middleware_class,
)


# ============================================================================
# TEST FIXTURES
# ============================================================================


class LoggingMiddleware:
    """Mock logging middleware."""
    def use(self, request, response, next):
        print("Logging request")
        next()


class RateLimitMiddleware:
    """Mock rate limit middleware."""
    def use(self, request, response, next):
        print("Checking rate limit")
        next()


class AuthMiddleware:
    """Mock auth middleware."""
    def use(self, request, response, next):
        print("Checking auth")
        next()


# ============================================================================
# TEST MiddlewareMetadata
# ============================================================================


class TestMiddlewareMetadata:
    """Suite de tests para MiddlewareMetadata."""
    
    def test_initialization_default(self):
        """Test inicialización con valores por defecto."""
        metadata = MiddlewareMetadata(middleware_classes=[LoggingMiddleware])
        
        assert metadata.middleware_classes == [LoggingMiddleware]
        assert metadata.order == 0
        assert metadata.options == {}
    
    def test_initialization_custom(self):
        """Test inicialización con valores custom."""
        metadata = MiddlewareMetadata(
            middleware_classes=[LoggingMiddleware, RateLimitMiddleware],
            order=5,
            options={"strict": True}
        )
        
        assert metadata.middleware_classes == [LoggingMiddleware, RateLimitMiddleware]
        assert metadata.order == 5
        assert metadata.options == {"strict": True}
    
    def test_validation_empty_middleware_classes(self):
        """Test error si middleware_classes está vacío."""
        with pytest.raises(ValueError, match="must have at least one middleware class"):
            MiddlewareMetadata(middleware_classes=[])
    
    def test_validation_non_type_middleware_classes(self):
        """Test error si middleware_classes contiene no-Type."""
        with pytest.raises(TypeError, match="must be Type classes"):
            MiddlewareMetadata(middleware_classes=["not a type"])
    
    def test_multiple_middleware_classes(self):
        """Test múltiples middleware classes."""
        metadata = MiddlewareMetadata(
            middleware_classes=[LoggingMiddleware, RateLimitMiddleware, AuthMiddleware]
        )
        
        assert len(metadata.middleware_classes) == 3
        assert LoggingMiddleware in metadata.middleware_classes
        assert RateLimitMiddleware in metadata.middleware_classes
        assert AuthMiddleware in metadata.middleware_classes


# ============================================================================
# TEST @middleware DECORATOR
# ============================================================================


class TestMiddlewareDecorator:
    """Suite de tests para @middleware decorator."""
    
    def test_middleware_on_controller_class(self):
        """Test @middleware en clase controller."""
        @middleware(LoggingMiddleware, RateLimitMiddleware)
        class UserController:
            __controller_metadata__ = {'path': '/users'}
        
        assert is_middleware(UserController)
        metadata = get_middleware_metadata(UserController)
        assert metadata is not None
        assert metadata.middleware_classes == [LoggingMiddleware, RateLimitMiddleware]
        assert metadata.order == 0
    
    def test_middleware_on_route_handler(self):
        """Test @middleware en route handler."""
        @middleware(AuthMiddleware)
        def create_user(data):
            __route_metadata__ = {'method': 'POST', 'path': '/users'}
            return data
        
        assert is_middleware(create_user)
        metadata = get_middleware_metadata(create_user)
        assert metadata.middleware_classes == [AuthMiddleware]
    
    def test_middleware_with_order(self):
        """Test @middleware con orden de ejecución."""
        @middleware(LoggingMiddleware, order=5)
        class OrderedController:
            __controller_metadata__ = {'path': '/api'}
        
        metadata = get_middleware_metadata(OrderedController)
        assert metadata.order == 5
    
    def test_middleware_with_custom_options(self):
        """Test @middleware con opciones custom."""
        @middleware(RateLimitMiddleware, order=1, max_requests=100, window_ms=60000)
        class RateLimitedController:
            __controller_metadata__ = {'path': '/api'}
        
        metadata = get_middleware_metadata(RateLimitedController)
        assert metadata.order == 1
        assert metadata.options == {"max_requests": 100, "window_ms": 60000}
    
    def test_single_middleware(self):
        """Test @middleware con un solo middleware."""
        @middleware(LoggingMiddleware)
        class SimpleController:
            __controller_metadata__ = {'path': '/simple'}
        
        metadata = get_middleware_metadata(SimpleController)
        assert len(metadata.middleware_classes) == 1
        assert metadata.middleware_classes[0] == LoggingMiddleware
    
    def test_error_on_ui_pipe(self):
        """Test error si se usa en UI pipe."""
        with pytest.raises(ValueError, match="cannot be used on UI pipes"):
            @middleware(LoggingMiddleware)
            class TestPipe:
                __ui_pipe_metadata__ = {'name': 'test'}
    
    def test_error_on_non_controller_class(self):
        """Test error si se usa en clase sin @controller."""
        with pytest.raises(ValueError, match="can only be used with @controller"):
            @middleware(LoggingMiddleware)
            class RegularClass:
                pass
    
    def test_error_on_non_callable(self):
        """Test error si target no es callable ni class."""
        with pytest.raises(ValueError, match="can only be used on classes or functions"):
            decorated = middleware(LoggingMiddleware)("not a callable")


# ============================================================================
# TEST HELPER FUNCTIONS
# ============================================================================


class TestHelperFunctions:
    """Suite de tests para helper functions."""
    
    def test_is_middleware_true(self):
        """Test is_middleware retorna True."""
        @middleware(LoggingMiddleware)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        assert is_middleware(TestController) is True
    
    def test_is_middleware_false(self):
        """Test is_middleware retorna False."""
        class RegularClass:
            pass
        
        assert is_middleware(RegularClass) is False
    
    def test_get_middleware_metadata_exists(self):
        """Test get_middleware_metadata cuando existe."""
        @middleware(LoggingMiddleware)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        metadata = get_middleware_metadata(TestController)
        assert metadata is not None
        assert isinstance(metadata, MiddlewareMetadata)
    
    def test_get_middleware_metadata_none(self):
        """Test get_middleware_metadata cuando no existe."""
        class RegularClass:
            pass
        
        metadata = get_middleware_metadata(RegularClass)
        assert metadata is None
    
    def test_get_middleware_classes(self):
        """Test get_middleware_classes."""
        @middleware(LoggingMiddleware, RateLimitMiddleware)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        classes = get_middleware_classes(TestController)
        assert classes == [LoggingMiddleware, RateLimitMiddleware]
    
    def test_get_middleware_classes_none(self):
        """Test get_middleware_classes sin middleware."""
        class RegularClass:
            pass
        
        classes = get_middleware_classes(RegularClass)
        assert classes == []
    
    def test_get_middleware_order(self):
        """Test get_middleware_order."""
        @middleware(LoggingMiddleware, order=10)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        order = get_middleware_order(TestController)
        assert order == 10
    
    def test_get_middleware_order_default(self):
        """Test get_middleware_order sin middleware."""
        class RegularClass:
            pass
        
        order = get_middleware_order(RegularClass)
        assert order == 0
    
    def test_combine_middleware(self):
        """Test combine_middleware."""
        @middleware(LoggingMiddleware, order=2)
        class Controller:
            __controller_metadata__ = {'path': '/api'}
        
        @middleware(AuthMiddleware, order=1)
        def handler():
            pass
        
        combined = combine_middleware(Controller, handler)
        # AuthMiddleware (order=1) debe ir primero que LoggingMiddleware (order=2)
        assert combined == [AuthMiddleware, LoggingMiddleware]
    
    def test_combine_middleware_empty(self):
        """Test combine_middleware sin middleware."""
        class NoMiddleware:
            pass
        
        combined = combine_middleware(NoMiddleware)
        assert combined == []
    
    def test_validate_middleware_class_valid(self):
        """Test validate_middleware_class con clase válida."""
        # No debe lanzar excepción
        validate_middleware_class(LoggingMiddleware)
    
    def test_validate_middleware_class_missing_use(self):
        """Test validate_middleware_class sin método use."""
        class InvalidMiddleware:
            pass
        
        with pytest.raises(TypeError, match="must implement use"):
            validate_middleware_class(InvalidMiddleware)
    
    def test_validate_middleware_class_non_callable_use(self):
        """Test validate_middleware_class con use no-callable."""
        class InvalidMiddleware:
            use = "not a method"
        
        with pytest.raises(TypeError, match="must be a callable method"):
            validate_middleware_class(InvalidMiddleware)


# ============================================================================
# TEST INTEGRATION
# ============================================================================


class TestIntegration:
    """Tests de integración para middleware."""
    
    def test_controller_with_multiple_middleware_and_order(self):
        """Test controller con múltiples middleware y orden."""
        @middleware(LoggingMiddleware, RateLimitMiddleware, AuthMiddleware, order=5)
        class ComplexController:
            __controller_metadata__ = {'path': '/complex'}
        
        assert is_middleware(ComplexController)
        metadata = get_middleware_metadata(ComplexController)
        
        assert len(metadata.middleware_classes) == 3
        assert metadata.order == 5
        assert LoggingMiddleware in metadata.middleware_classes
        assert RateLimitMiddleware in metadata.middleware_classes
        assert AuthMiddleware in metadata.middleware_classes
    
    def test_middleware_chain_ordering(self):
        """Test ordenamiento de cadena de middleware."""
        @middleware(LoggingMiddleware, order=3)
        class Controller:
            __controller_metadata__ = {'path': '/api'}
            
            @middleware(AuthMiddleware, order=1)
            def secure_endpoint(self):
                pass
            
            @middleware(RateLimitMiddleware, order=2)
            def limited_endpoint(self):
                pass
        
        # Combinar middleware de controller y método secure
        combined_secure = combine_middleware(Controller, Controller.secure_endpoint)
        assert combined_secure == [AuthMiddleware, LoggingMiddleware]
        
        # Combinar middleware de controller y método limited
        combined_limited = combine_middleware(Controller, Controller.limited_endpoint)
        assert combined_limited == [RateLimitMiddleware, LoggingMiddleware]


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
