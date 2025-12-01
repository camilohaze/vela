"""
Tests unitarios para decorador @pipe (Frontend UI Pipes)

Fase 2: Frontend UI Pipes - Angular-style
Testing de:
- UIPipeMetadata validaciones
- Decorador @pipe context-aware para frontend
- Context detection (auto-detección frontend vs backend)
- Validaciones de contexto estrictas
- PipeTransform interface

Jira: TASK-035E2 (Fase 2)
Historia: VELA-575
"""

import pytest
from src.runtime.di.pipes import (
    pipe,
    PipeContext,
    UIPipeMetadata,
    HTTPPipeMetadata,
    is_ui_pipe,
    is_http_pipe,
    get_ui_pipe_metadata,
    get_pipe_name,
    validate_pipe_class,
)


# ============================================================================
# TEST FIXTURES
# ============================================================================


class ValidationPipe:
    """Mock backend validation pipe."""
    def transform(self, value):
        return value


# ============================================================================
# TEST UIPipeMetadata
# ============================================================================


class TestUIPipeMetadata:
    """Suite de tests para UIPipeMetadata."""
    
    def test_initialization_default(self):
        """Test inicialización con valores por defecto."""
        metadata = UIPipeMetadata(name="currency")
        
        assert metadata.name == "currency"
        assert metadata.pure is True
        assert metadata.standalone is False
        assert metadata.context == PipeContext.UI
    
    def test_initialization_custom(self):
        """Test inicialización con valores custom."""
        metadata = UIPipeMetadata(
            name="uppercase",
            pure=False,
            standalone=True
        )
        
        assert metadata.name == "uppercase"
        assert metadata.pure is False
        assert metadata.standalone is True
        assert metadata.context == PipeContext.UI
    
    def test_validation_empty_name(self):
        """Test error si name está vacío."""
        with pytest.raises(ValueError, match="name cannot be empty"):
            UIPipeMetadata(name="")
    
    def test_validation_whitespace_name(self):
        """Test error si name solo tiene espacios."""
        with pytest.raises(ValueError, match="name cannot be empty"):
            UIPipeMetadata(name="   ")
    
    def test_validation_invalid_identifier(self):
        """Test error si name no es identificador válido."""
        with pytest.raises(ValueError, match="not a valid identifier"):
            UIPipeMetadata(name="invalid-name")
    
    def test_name_trimming(self):
        """Test que name se trimea automáticamente."""
        metadata = UIPipeMetadata(name="  currency  ")
        assert metadata.name == "currency"
    
    def test_valid_identifiers(self):
        """Test nombres válidos."""
        valid_names = ["currency", "uppercase", "date_format", "camelCase", "snake_case"]
        
        for name in valid_names:
            metadata = UIPipeMetadata(name=name)
            assert metadata.name == name


# ============================================================================
# TEST @pipe DECORATOR - FRONTEND UI
# ============================================================================


class TestUIFrontendPipes:
    """Suite de tests para @pipe decorator en contexto frontend UI."""
    
    def test_pipe_on_ui_pipe_class(self):
        """Test @pipe en clase que termina en 'Pipe'."""
        @pipe(name="currency", pure=True)
        class CurrencyPipe:
            def transform(self, value):
                return f"${value:.2f}"
        
        assert is_ui_pipe(CurrencyPipe)
        metadata = get_ui_pipe_metadata(CurrencyPipe)
        assert metadata is not None
        assert metadata.name == "currency"
        assert metadata.pure is True
        assert metadata.context == PipeContext.UI
    
    def test_pipe_with_pure_false(self):
        """Test @pipe con pure=False (impure pipe)."""
        @pipe(name="async_data", pure=False)
        class AsyncDataPipe:
            def transform(self, value):
                return value
        
        metadata = get_ui_pipe_metadata(AsyncDataPipe)
        assert metadata.pure is False
    
    def test_pipe_with_standalone(self):
        """Test @pipe con standalone=True."""
        @pipe(name="standalone_pipe", standalone=True)
        class StandalonePipe:
            def transform(self, value):
                return value
        
        metadata = get_ui_pipe_metadata(StandalonePipe)
        assert metadata.standalone is True
    
    def test_error_on_non_pipe_class(self):
        """Test error si clase no termina en 'Pipe'."""
        with pytest.raises(ValueError, match="can only be used on pipe classes"):
            @pipe(name="currency")
            class CurrencyTransformer:
                def transform(self, value):
                    return value
    
    def test_error_on_controller_class(self):
        """Test error si se usa con @controller."""
        with pytest.raises(ValueError, match="cannot be used with @controller"):
            @pipe(name="currency")
            class CurrencyPipe:
                __controller_metadata__ = {'path': '/api'}
                
                def transform(self, value):
                    return value
    
    def test_error_on_http_pipe_metadata_conflict(self):
        """Test error si ya tiene HTTP pipe metadata."""
        with pytest.raises(ValueError, match="cannot be used with backend HTTP pipes"):
            @pipe(name="currency")
            class CurrencyPipe:
                __http_pipe_metadata__ = HTTPPipeMetadata(pipe_classes=[ValidationPipe])
                
                def transform(self, value):
                    return value
    
    def test_error_on_backend_injectable(self):
        """Test error si se usa con @injectable backend."""
        with pytest.raises(ValueError, match="cannot be used with @injectable"):
            @pipe(name="currency")
            class CurrencyPipe:
                __injectable_metadata__ = {'context': 'backend'}
                
                def transform(self, value):
                    return value
    
    def test_multiple_ui_pipes(self):
        """Test múltiples UI pipes."""
        @pipe(name="uppercase")
        class UppercasePipe:
            def transform(self, value):
                return value.upper()
        
        @pipe(name="lowercase")
        class LowercasePipe:
            def transform(self, value):
                return value.lower()
        
        @pipe(name="titlecase")
        class TitlecasePipe:
            def transform(self, value):
                return value.title()
        
        assert get_pipe_name(UppercasePipe) == "uppercase"
        assert get_pipe_name(LowercasePipe) == "lowercase"
        assert get_pipe_name(TitlecasePipe) == "titlecase"


# ============================================================================
# TEST CONTEXT DETECTION
# ============================================================================


class TestContextDetection:
    """Suite de tests para auto-detección de contexto."""
    
    def test_frontend_detection_by_kwargs(self):
        """Test que kwargs con 'name' detecta frontend."""
        @pipe(name="currency")
        class CurrencyPipe:
            def transform(self, value):
                return value
        
        assert is_ui_pipe(CurrencyPipe)
        assert not is_http_pipe(CurrencyPipe)
    
    def test_backend_detection_by_args(self):
        """Test que args con Type detecta backend."""
        @pipe(ValidationPipe)
        class TestController:
            __controller_metadata__ = {'path': '/test'}
        
        assert is_http_pipe(TestController)
        assert not is_ui_pipe(TestController)
    
    def test_error_on_invalid_syntax(self):
        """Test error si sintaxis no es frontend ni backend."""
        with pytest.raises(ValueError, match="Invalid @pipe usage"):
            @pipe("invalid_string_arg")
            class SomePipe:
                pass
    
    def test_error_on_empty_args_kwargs(self):
        """Test error si no hay args ni kwargs."""
        with pytest.raises(ValueError, match="Invalid @pipe usage"):
            @pipe()
            class SomeClass:
                pass
    
    def test_frontend_ignores_extra_kwargs(self):
        """Test que frontend acepta kwargs adicionales."""
        @pipe(name="custom", pure=False, custom_option=True)
        class CustomPipe:
            def transform(self, value):
                return value
        
        metadata = get_ui_pipe_metadata(CustomPipe)
        assert metadata.name == "custom"
        assert metadata.pure is False
    
    def test_backend_ignores_extra_kwargs(self):
        """Test que backend acepta kwargs adicionales (options)."""
        @pipe(ValidationPipe, strict=True, async_mode=False)
        class StrictController:
            __controller_metadata__ = {'path': '/strict'}
        
        metadata = get_ui_pipe_metadata(StrictController)
        assert metadata is None  # No tiene UI metadata
        
        http_metadata = get_ui_pipe_metadata(StrictController)
        assert http_metadata is None  # Solo HTTP metadata


# ============================================================================
# TEST CONTEXT VALIDATION (ERRORES ESPERADOS)
# ============================================================================


class TestContextValidation:
    """Suite de tests para validaciones de contexto estrictas."""
    
    def test_ui_pipe_cannot_be_controller(self):
        """Test que UI pipe no puede ser controller."""
        with pytest.raises(ValueError, match="cannot be used with @controller"):
            @pipe(name="test")
            class TestPipe:
                __controller_metadata__ = {'path': '/test'}
    
    def test_ui_pipe_requires_pipe_suffix(self):
        """Test que UI pipe requiere sufijo 'Pipe'."""
        with pytest.raises(ValueError, match="Class name must end with 'Pipe'"):
            @pipe(name="test")
            class TestTransformer:
                pass
    
    def test_http_pipe_cannot_be_on_pipe_class(self):
        """Test que HTTP pipe no puede estar en clase con sufijo 'Pipe'."""
        with pytest.raises(ValueError, match="cannot be used on class"):
            @pipe(ValidationPipe)
            class CustomPipe:
                pass
    
    def test_ui_pipe_cannot_have_http_metadata(self):
        """Test que UI pipe no puede tener HTTP metadata."""
        with pytest.raises(ValueError, match="cannot be used with backend HTTP pipes"):
            @pipe(name="test")
            class TestPipe:
                __http_pipe_metadata__ = HTTPPipeMetadata(pipe_classes=[ValidationPipe])
    
    def test_http_pipe_cannot_have_ui_metadata(self):
        """Test que HTTP pipe no puede tener UI metadata."""
        # Crear clase con UI metadata primero
        class TestPipe:
            __ui_pipe_metadata__ = UIPipeMetadata(name="test")
        
        # Intentar agregar HTTP pipe debe fallar
        with pytest.raises(ValueError, match="cannot be used on UI pipe classes"):
            decorated = pipe(ValidationPipe)(TestPipe)
    
    def test_ui_pipe_cannot_be_backend_injectable(self):
        """Test que UI pipe no puede ser @injectable backend."""
        with pytest.raises(ValueError, match="cannot be used with @injectable"):
            @pipe(name="test")
            class TestPipe:
                __injectable_metadata__ = {'context': 'backend'}
    
    def test_http_pipe_requires_controller_or_service(self):
        """Test que HTTP pipe requiere @controller o @injectable."""
        with pytest.raises(ValueError, match="can only be used with @controller or @injectable"):
            @pipe(ValidationPipe)
            class RegularClass:
                pass
    
    def test_mixed_args_kwargs_with_name_uses_frontend(self):
        """Test que 'name' en kwargs fuerza frontend incluso con args."""
        @pipe(name="currency")
        class CurrencyPipe:
            def transform(self, value):
                return value
        
        # Debe ser UI pipe, no HTTP
        assert is_ui_pipe(CurrencyPipe)
        assert not is_http_pipe(CurrencyPipe)


# ============================================================================
# TEST HELPER FUNCTIONS - UI
# ============================================================================


class TestUIHelperFunctions:
    """Suite de tests para helper functions de UI pipes."""
    
    def test_is_ui_pipe_true(self):
        """Test is_ui_pipe retorna True."""
        @pipe(name="test")
        class TestPipe:
            def transform(self, value):
                return value
        
        assert is_ui_pipe(TestPipe) is True
    
    def test_is_ui_pipe_false(self):
        """Test is_ui_pipe retorna False."""
        class RegularClass:
            pass
        
        assert is_ui_pipe(RegularClass) is False
    
    def test_get_ui_pipe_metadata_exists(self):
        """Test get_ui_pipe_metadata cuando existe."""
        @pipe(name="currency")
        class CurrencyPipe:
            def transform(self, value):
                return value
        
        metadata = get_ui_pipe_metadata(CurrencyPipe)
        assert metadata is not None
        assert isinstance(metadata, UIPipeMetadata)
        assert metadata.name == "currency"
    
    def test_get_ui_pipe_metadata_none(self):
        """Test get_ui_pipe_metadata cuando no existe."""
        class RegularClass:
            pass
        
        metadata = get_ui_pipe_metadata(RegularClass)
        assert metadata is None
    
    def test_get_pipe_name_exists(self):
        """Test get_pipe_name cuando existe."""
        @pipe(name="uppercase")
        class UppercasePipe:
            def transform(self, value):
                return value
        
        name = get_pipe_name(UppercasePipe)
        assert name == "uppercase"
    
    def test_get_pipe_name_none(self):
        """Test get_pipe_name cuando no existe."""
        class RegularClass:
            pass
        
        name = get_pipe_name(RegularClass)
        assert name is None
    
    def test_validate_pipe_class_valid(self):
        """Test validate_pipe_class con clase válida."""
        @pipe(name="test")
        class TestPipe:
            def transform(self, value):
                return value
        
        # No debe lanzar excepción
        validate_pipe_class(TestPipe)
    
    def test_validate_pipe_class_missing_transform(self):
        """Test validate_pipe_class sin método transform."""
        @pipe(name="test")
        class InvalidPipe:
            pass
        
        with pytest.raises(TypeError, match="must implement transform"):
            validate_pipe_class(InvalidPipe)
    
    def test_validate_pipe_class_non_callable_transform(self):
        """Test validate_pipe_class con transform no-callable."""
        @pipe(name="test")
        class InvalidPipe:
            transform = "not a method"
        
        with pytest.raises(TypeError, match="must be a callable method"):
            validate_pipe_class(InvalidPipe)


# ============================================================================
# TEST INTEGRATION - UI PIPES
# ============================================================================


class TestUIIntegration:
    """Tests de integración para UI pipes."""
    
    def test_multiple_ui_pipes_with_different_configs(self):
        """Test múltiples UI pipes con configs diferentes."""
        @pipe(name="currency", pure=True)
        class CurrencyPipe:
            def transform(self, value):
                return f"${value:.2f}"
        
        @pipe(name="async_data", pure=False)
        class AsyncDataPipe:
            def transform(self, value):
                return value
        
        @pipe(name="standalone", standalone=True)
        class StandalonePipe:
            def transform(self, value):
                return value
        
        # Verificar cada pipe
        assert get_pipe_name(CurrencyPipe) == "currency"
        assert get_ui_pipe_metadata(CurrencyPipe).pure is True
        
        assert get_pipe_name(AsyncDataPipe) == "async_data"
        assert get_ui_pipe_metadata(AsyncDataPipe).pure is False
        
        assert get_pipe_name(StandalonePipe) == "standalone"
        assert get_ui_pipe_metadata(StandalonePipe).standalone is True
    
    def test_pipe_transform_validation(self):
        """Test validación de método transform."""
        @pipe(name="uppercase")
        class UppercasePipe:
            def transform(self, value: str) -> str:
                return value.upper()
        
        # Validar que tiene transform
        validate_pipe_class(UppercasePipe)
        
        # Crear instancia y probar
        pipe_instance = UppercasePipe()
        result = pipe_instance.transform("hello")
        assert result == "HELLO"


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
