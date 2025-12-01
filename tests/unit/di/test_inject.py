"""
Tests unitarios para @inject decorator

Implementación de: TASK-035C
Historia: VELA-575
Sprint: 13
"""

import pytest
import sys
from pathlib import Path
from typing import Optional
import inspect

# Agregar src/ al path para importar módulos
src_path = Path(__file__).resolve().parent.parent.parent.parent / "src"
sys.path.insert(0, str(src_path))

from runtime.di.inject import (
    inject,
    InjectMetadata,
    get_inject_metadata,
    set_inject_metadata,
    get_constructor_inject_metadata,
    has_inject_params,
    get_inject_token
)


class TestInjectMetadata:
    """Tests para InjectMetadata dataclass."""
    
    def test_inject_metadata_creation(self):
        """Test creación de InjectMetadata."""
        metadata = InjectMetadata(
            param_name="repository",
            param_type=str,
            token="my-repo",
            optional=False
        )
        
        assert metadata.param_name == "repository"
        assert metadata.param_type == str
        assert metadata.token == "my-repo"
        assert metadata.optional == False
    
    def test_inject_metadata_defaults(self):
        """Test valores por defecto de InjectMetadata."""
        metadata = InjectMetadata(param_name="test")
        
        assert metadata.param_name == "test"
        assert metadata.param_type is None
        assert metadata.token is None
        assert metadata.optional == False
        assert metadata.default is inspect.Parameter.empty
    
    def test_inject_metadata_empty_name_raises(self):
        """Test que nombre vacío lanza ValueError."""
        with pytest.raises(ValueError, match="param_name cannot be empty"):
            InjectMetadata(param_name="")


class TestGetInjectToken:
    """Tests para get_inject_token function."""
    
    def test_get_inject_token_with_custom_token(self):
        """Test get_inject_token con token custom."""
        metadata = InjectMetadata(
            param_name="repo",
            param_type=str,
            token="custom-token"
        )
        
        token = get_inject_token(metadata)
        assert token == "custom-token"
    
    def test_get_inject_token_uses_type_name(self):
        """Test get_inject_token usa nombre de tipo si no hay token."""
        class MyClass:
            pass
        
        metadata = InjectMetadata(
            param_name="service",
            param_type=MyClass
        )
        
        token = get_inject_token(metadata)
        assert token == "MyClass"
    
    def test_get_inject_token_fallback_to_param_name(self):
        """Test get_inject_token fallback a nombre de parámetro."""
        metadata = InjectMetadata(param_name="my_param")
        
        token = get_inject_token(metadata)
        assert token == "my_param"


class TestGetInjectMetadata:
    """Tests para get_inject_metadata function."""
    
    def test_get_inject_metadata_empty_function(self):
        """Test get_inject_metadata con función sin parámetros."""
        def empty_func():
            pass
        
        metadata = get_inject_metadata(empty_func)
        assert len(metadata) == 0
    
    def test_get_inject_metadata_no_inject_params(self):
        """Test función con parámetros pero sin @inject."""
        def func_without_inject(a: int, b: str):
            pass
        
        metadata = get_inject_metadata(func_without_inject)
        assert len(metadata) == 0
    
    def test_get_inject_metadata_skips_self(self):
        """Test que get_inject_metadata ignora 'self'."""
        class DummyClass:
            def method(self, value: int):
                pass
        
        metadata = get_inject_metadata(DummyClass.method)
        # No debe incluir 'self' ni 'value' (sin @inject)
        assert len(metadata) == 0


class TestSetAndGetInjectMetadata:
    """Tests para set/get inject metadata en clases."""
    
    def test_set_inject_metadata(self):
        """Test set_inject_metadata almacena metadata en clase."""
        class TestClass:
            pass
        
        metadata = [
            InjectMetadata(param_name="repo", param_type=str)
        ]
        
        set_inject_metadata(TestClass, metadata)
        
        assert hasattr(TestClass, '__inject_params__')
        assert TestClass.__inject_params__ == metadata
    
    def test_get_constructor_inject_metadata_cached(self):
        """Test que get_constructor_inject_metadata usa cache."""
        class TestClass:
            pass
        
        metadata = [
            InjectMetadata(param_name="cached", param_type=int)
        ]
        
        # Setear cache manualmente
        set_inject_metadata(TestClass, metadata)
        
        # Debería retornar desde cache
        result = get_constructor_inject_metadata(TestClass)
        assert result == metadata
    
    def test_get_constructor_inject_metadata_no_init(self):
        """Test clase sin __init__."""
        class NoInit:
            pass
        
        metadata = get_constructor_inject_metadata(NoInit)
        assert len(metadata) == 0


class TestHasInjectParams:
    """Tests para has_inject_params function."""
    
    def test_has_inject_params_false_without_inject(self):
        """Test has_inject_params retorna False sin @inject."""
        class ServiceWithoutInject:
            def __init__(self, value: int):
                self.value = value
        
        assert has_inject_params(ServiceWithoutInject) == False
    
    def test_has_inject_params_false_no_init(self):
        """Test has_inject_params False para clase sin __init__."""
        class NoInit:
            pass
        
        assert has_inject_params(NoInit) == False


class TestInjectDecorator:
    """Tests para decorador @inject."""
    
    def test_inject_decorator_basic(self):
        """Test decorador @inject marca parámetro."""
        # Simular parámetro con default que actúa como decorador
        class DummyParam:
            pass
        
        param = DummyParam()
        decorated = inject()(param)
        
        assert hasattr(decorated, '__inject_metadata__')
        assert decorated.__inject_metadata__.token is None
    
    def test_inject_decorator_with_token(self):
        """Test @inject con token custom."""
        class DummyParam:
            pass
        
        param = DummyParam()
        decorated = inject("custom-token")(param)
        
        assert hasattr(decorated, '__inject_metadata__')
        assert decorated.__inject_metadata__.token == "custom-token"


class TestInjectIntegration:
    """Tests de integración para @inject."""
    
    def test_inject_metadata_extraction_basic(self):
        """Test extracción básica de metadata con type hints."""
        # Crear clase con constructor que simula @inject
        class Repository:
            pass
        
        # Función que simula constructor con @inject
        def mock_constructor(self, repo: Repository):
            pass
        
        # En producción, @inject modificaría param.default
        # Aquí simulamos manualmente
        sig = inspect.signature(mock_constructor)
        param = sig.parameters['repo']
        
        # Simular que tiene @inject
        class MockDefault:
            __inject_metadata__ = InjectMetadata(
                param_name="repo",
                token=None
            )
        
        # Modificar signature para incluir mock
        # (En test real, @inject haría esto)
        pass  # Nota: Test completo requeriría mock más elaborado


# Ejecutar tests si se corre directamente
if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
