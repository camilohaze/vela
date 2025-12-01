"""
Tests unitarios para decorador @pipe (Backend HTTP Pipes)

Fase 1: Backend HTTP Pipes - NestJS-style
Testing de:
- HTTPPipeMetadata validaciones
- Decorador @pipe context-aware para backend
- Validaciones de contexto estrictas
- Parameter pipes

Jira: TASK-035E2 (Fase 1)
Historia: VELA-575
"""

import pytest
from src.runtime.di.pipes import (
    pipe,
    PipeContext,
    HTTPPipeMetadata,
    ParameterPipeMetadata,
    is_http_pipe,
    is_parameter_pipe,
    get_http_pipe_metadata,
    get_pipe_classes,
)


# ============================================================================
# TEST FIXTURES
# ============================================================================


class ValidationPipe:
    """Mock validation pipe."""
    def transform(self, value):
        return value


class TransformPipe:
    """Mock transform pipe."""
    def transform(self, value):
        return str(value)


class ParseIntPipe:
    """Mock parse int pipe."""
    def transform(self, value):
        return int(value)


class MockController:
    """Mock controller class."""
    __controller_metadata__ = {'path': '/api'}


class MockService:
    """Mock service class."""
    __injectable_metadata__ = {'scope': 'singleton'}


# ============================================================================
# TEST HTTPPipeMetadata
# ============================================================================


class TestHTTPPipeMetadata:
    """Suite de tests para HTTPPipeMetadata."""
    
    def test_initialization_default(self):
        """Test inicialización con valores por defecto."""
        metadata = HTTPPipeMetadata(pipe_classes=[ValidationPipe])
        
        assert metadata.pipe_classes == [ValidationPipe]
        assert metadata.target == "input"
        assert metadata.context == PipeContext.HTTP
        assert metadata.options == {}
    
    def test_initialization_custom(self):
        """Test inicialización con valores custom."""
        metadata = HTTPPipeMetadata(
            pipe_classes=[ValidationPipe, TransformPipe],
            target="output",
            options={"strict": True}
        )
        
        assert metadata.pipe_classes == [ValidationPipe, TransformPipe]
        assert metadata.target == "output"
        assert metadata.context == PipeContext.HTTP
        assert metadata.options == {"strict": True}
    
    def test_validation_empty_pipe_classes(self):
        """Test error si pipe_classes está vacío."""
        with pytest.raises(ValueError, match="must have at least one pipe class"):
            HTTPPipeMetadata(pipe_classes=[])
    
    def test_validation_non_type_pipe_classes(self):
        """Test error si pipe_classes contiene no-Type."""
        with pytest.raises(TypeError, match="must be Type classes"):
            HTTPPipeMetadata(pipe_classes=["not a type"])
    
    def test_validation_invalid_target(self):
        """Test error si target es inválido."""
        with pytest.raises(ValueError, match="Invalid target"):
            HTTPPipeMetadata(
                pipe_classes=[ValidationPipe],
                target="invalid"
            )
    
    def test_multiple_pipes(self):
        """Test múltiples pipes."""
        metadata = HTTPPipeMetadata(
            pipe_classes=[ValidationPipe, TransformPipe, ParseIntPipe]
        )
        
        assert len(metadata.pipe_classes) == 3
        assert ValidationPipe in metadata.pipe_classes
        assert TransformPipe in metadata.pipe_classes
        assert ParseIntPipe in metadata.pipe_classes


# ============================================================================
# TEST @pipe DECORATOR - BACKEND HTTP
# ============================================================================


class TestHTTPPipeDecorator:
    """Suite de tests para @pipe decorator en contexto backend HTTP."""
    
    def test_pipe_on_controller_class(self):
        """Test @pipe en clase controller."""
        @pipe(ValidationPipe, TransformPipe)
        class UserController:
            __controller_metadata__ = {'path': '/users'}
        
        assert is_http_pipe(UserController)
        metadata = get_http_pipe_metadata(UserController)
        assert metadata is not None
        assert metadata.pipe_classes == [ValidationPipe, TransformPipe]
        assert metadata.target == "input"
        assert metadata.context == PipeContext.HTTP
    
    def test_pipe_on_service_class(self):
        """Test @pipe en clase service."""
        @pipe(ValidationPipe)
        class UserService:
            __injectable_metadata__ = {'scope': 'singleton'}
        
        assert is_http_pipe(UserService)
        metadata = get_http_pipe_metadata(UserService)
        assert metadata.pipe_classes == [ValidationPipe]
    
    def test_pipe_on_route_handler(self):
        """Test @pipe en route handler."""
        @pipe(ValidationPipe, TransformPipe)
        def create_user(data):
            return data
        
        assert is_http_pipe(create_user)
        metadata = get_http_pipe_metadata(create_user)
        assert metadata.pipe_classes == [ValidationPipe, TransformPipe]
    
    def test_pipe_with_target_option(self):
        """Test @pipe con opción target."""
        @pipe(TransformPipe, target="output")
        class ResponseController:
            __controller_metadata__ = {'path': '/api'}
        
        metadata = get_http_pipe_metadata(ResponseController)
        assert metadata.target == "output"
    
    def test_pipe_with_custom_options(self):
        """Test @pipe con opciones custom."""
        @pipe(ValidationPipe, target="input", strict=True, async_mode=False)
        class StrictController:
            __controller_metadata__ = {'path': '/strict'}
        
        metadata = get_http_pipe_metadata(StrictController)
        assert metadata.options == {"strict": True, "async_mode": False}
    
    def test_single_pipe(self):
        """Test @pipe con un solo pipe."""
        @pipe(ValidationPipe)
        class SimpleController:
            __controller_metadata__ = {'path': '/simple'}
        
        metadata = get_http_pipe_metadata(SimpleController)
        assert len(metadata.pipe_classes) == 1
        assert metadata.pipe_classes[0] == ValidationPipe
    
    def test_error_on_non_controller_non_service(self):
        """Test error si se aplica a clase sin @controller ni @injectable."""
        with pytest.raises(ValueError, match="can only be used with @controller or @injectable"):
            @pipe(ValidationPipe)
            class RegularClass:
                pass
    
    def test_error_on_pipe_class_backend(self):
        """Test error si se usa en clase que termina en 'Pipe'."""
        with pytest.raises(ValueError, match="cannot be used on class"):
            @pipe(ValidationPipe)
            class CustomPipe:
                def transform(self, value):
                    return value


# ============================================================================
# TEST ParameterPipeMetadata
# ============================================================================


class TestParameterPipeMetadata:
    """Suite de tests para ParameterPipeMetadata."""
    
    def test_initialization_default(self):
        """Test inicialización con valores por defecto."""
        metadata = ParameterPipeMetadata(pipe_classes=[ParseIntPipe])
        
        assert metadata.pipe_classes == [ParseIntPipe]
        assert metadata.context == PipeContext.PARAMETER
        assert metadata.options == {}
    
    def test_initialization_custom(self):
        """Test inicialización con opciones custom."""
        metadata = ParameterPipeMetadata(
            pipe_classes=[ParseIntPipe],
            options={"strict": True}
        )
        
        assert metadata.options == {"strict": True}
    
    def test_validation_empty_pipe_classes(self):
        """Test error si pipe_classes está vacío."""
        with pytest.raises(ValueError, match="must have at least one pipe class"):
            ParameterPipeMetadata(pipe_classes=[])
    
    def test_validation_non_type_pipe_classes(self):
        """Test error si pipe_classes contiene no-Type."""
        with pytest.raises(TypeError, match="must be Type classes"):
            ParameterPipeMetadata(pipe_classes=[123])


# ============================================================================
# TEST PARAMETER PIPES
# ============================================================================


class TestParameterPipes:
    """Suite de tests para parameter pipes."""
    
    def test_parameter_pipe_returns_metadata(self):
        """Test que @pipe en parámetro retorna metadata."""
        # Simular uso en parámetro
        decorator = pipe(ParseIntPipe)
        result = decorator(None)  # En parámetros, el target es None
        
        assert is_parameter_pipe(result)
        assert isinstance(result, ParameterPipeMetadata)
        assert result.pipe_classes == [ParseIntPipe]
    
    def test_parameter_pipe_multiple_pipes(self):
        """Test parameter pipe con múltiples pipes."""
        decorator = pipe(ValidationPipe, ParseIntPipe)
        result = decorator(None)
        
        assert isinstance(result, ParameterPipeMetadata)
        assert len(result.pipe_classes) == 2


# ============================================================================
# TEST HELPER FUNCTIONS
# ============================================================================


class TestHelperFunctions:
    """Suite de tests para helper functions."""
    
    def test_is_http_pipe_true(self):
        """Test is_http_pipe retorna True."""
        @pipe(ValidationPipe)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        assert is_http_pipe(TestController) is True
    
    def test_is_http_pipe_false(self):
        """Test is_http_pipe retorna False."""
        class RegularClass:
            pass
        
        assert is_http_pipe(RegularClass) is False
    
    def test_is_parameter_pipe_true(self):
        """Test is_parameter_pipe retorna True."""
        metadata = ParameterPipeMetadata(pipe_classes=[ParseIntPipe])
        assert is_parameter_pipe(metadata) is True
    
    def test_is_parameter_pipe_false(self):
        """Test is_parameter_pipe retorna False."""
        assert is_parameter_pipe("not a pipe") is False
    
    def test_get_http_pipe_metadata_exists(self):
        """Test get_http_pipe_metadata cuando existe."""
        @pipe(ValidationPipe)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        metadata = get_http_pipe_metadata(TestController)
        assert metadata is not None
        assert isinstance(metadata, HTTPPipeMetadata)
    
    def test_get_http_pipe_metadata_none(self):
        """Test get_http_pipe_metadata cuando no existe."""
        class RegularClass:
            pass
        
        metadata = get_http_pipe_metadata(RegularClass)
        assert metadata is None
    
    def test_get_pipe_classes_http(self):
        """Test get_pipe_classes con HTTP pipe."""
        @pipe(ValidationPipe, TransformPipe)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        classes = get_pipe_classes(TestController)
        assert classes == [ValidationPipe, TransformPipe]
    
    def test_get_pipe_classes_parameter(self):
        """Test get_pipe_classes con parameter pipe."""
        metadata = ParameterPipeMetadata(pipe_classes=[ParseIntPipe])
        classes = get_pipe_classes(metadata)
        assert classes == [ParseIntPipe]
    
    def test_get_pipe_classes_none(self):
        """Test get_pipe_classes sin pipes."""
        class RegularClass:
            pass
        
        classes = get_pipe_classes(RegularClass)
        assert classes == []


# ============================================================================
# TEST INTEGRATION
# ============================================================================


class TestIntegration:
    """Tests de integración para HTTP pipes."""
    
    def test_controller_with_multiple_pipes_and_options(self):
        """Test controller con múltiples pipes y opciones."""
        @pipe(ValidationPipe, TransformPipe, ParseIntPipe, target="input", strict=True)
        class ComplexController:
            __controller_metadata__ = {'path': '/complex'}
            
            def create(self, data):
                return data
        
        assert is_http_pipe(ComplexController)
        metadata = get_http_pipe_metadata(ComplexController)
        
        assert len(metadata.pipe_classes) == 3
        assert metadata.target == "input"
        assert metadata.options["strict"] is True
        assert metadata.context == PipeContext.HTTP
    
    def test_service_with_output_pipe(self):
        """Test service con output pipe."""
        @pipe(TransformPipe, target="output")
        class DataService:
            __injectable_metadata__ = {'scope': 'transient'}
            
            def process(self, data):
                return data
        
        metadata = get_http_pipe_metadata(DataService)
        assert metadata.target == "output"
        assert metadata.pipe_classes == [TransformPipe]


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
