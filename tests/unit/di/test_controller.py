"""
Tests unitarios para @controller decorator

Jira: TASK-035D2
Historia: VELA-575
"""

import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../..')))

import pytest
from src.runtime.di.controller import (
    controller,
    ControllerMetadata,
    is_controller,
    get_controller_metadata,
    get_controller_base_path,
    get_controller_full_path,
    get_controller_tags,
    register_controller,
    get_controller,
    get_all_controllers,
    clear_controller_registry,
    find_controller_by_path,
    get_controllers_by_tag,
)


# ============================================================================
# FIXTURES
# ============================================================================

@pytest.fixture(autouse=True)
def clear_registry():
    """Limpiar registry antes y después de cada test."""
    clear_controller_registry()
    yield
    clear_controller_registry()


# ============================================================================
# TEST: ControllerMetadata
# ============================================================================

class TestControllerMetadata:
    """Tests para ControllerMetadata dataclass."""
    
    def test_create_empty_metadata(self):
        """Test crear metadata con defaults."""
        metadata = ControllerMetadata()
        
        assert metadata.base_path == "/"
        assert metadata.prefix is None
        assert metadata.tags == []
        assert metadata.description is None
    
    def test_create_metadata_with_base_path(self):
        """Test crear metadata con base_path."""
        metadata = ControllerMetadata(base_path="/users")
        
        assert metadata.base_path == "/users"
    
    def test_normalize_base_path_without_leading_slash(self):
        """Test normalizar base_path sin slash inicial."""
        metadata = ControllerMetadata(base_path="users")
        
        assert metadata.base_path == "/users"
    
    def test_normalize_base_path_removes_trailing_slash(self):
        """Test normalizar base_path removiendo trailing slash."""
        metadata = ControllerMetadata(base_path="/users/")
        
        assert metadata.base_path == "/users"
    
    def test_normalize_base_path_keeps_root_slash(self):
        """Test mantener "/" como ruta root."""
        metadata = ControllerMetadata(base_path="/")
        
        assert metadata.base_path == "/"
    
    def test_get_full_path_without_prefix(self):
        """Test get_full_path sin prefix."""
        metadata = ControllerMetadata(base_path="/users")
        
        assert metadata.get_full_path() == "/users"
    
    def test_get_full_path_with_prefix(self):
        """Test get_full_path con prefix."""
        metadata = ControllerMetadata(base_path="/users", prefix="/api")
        
        assert metadata.get_full_path() == "/api/users"
    
    def test_get_full_path_with_prefix_no_slash(self):
        """Test get_full_path con prefix sin slash inicial."""
        metadata = ControllerMetadata(base_path="/users", prefix="api")
        
        assert metadata.get_full_path() == "/api/users"
    
    def test_get_full_path_root_with_prefix(self):
        """Test get_full_path para root con prefix."""
        metadata = ControllerMetadata(base_path="/", prefix="/api")
        
        assert metadata.get_full_path() == "/api"
    
    def test_tags_as_list(self):
        """Test tags como lista."""
        metadata = ControllerMetadata(tags=["Users", "Admin"])
        
        assert metadata.tags == ["Users", "Admin"]
    
    def test_tags_as_string_converts_to_list(self):
        """Test tags como string se convierte a lista."""
        metadata = ControllerMetadata(tags="Users")
        
        assert metadata.tags == ["Users"]
    
    def test_tags_none_converts_to_empty_list(self):
        """Test tags None se convierte a lista vacía."""
        metadata = ControllerMetadata(tags=None)
        
        assert metadata.tags == []
    
    def test_description(self):
        """Test description."""
        metadata = ControllerMetadata(description="User management controller")
        
        assert metadata.description == "User management controller"


# ============================================================================
# TEST: @controller decorator
# ============================================================================

class TestControllerDecorator:
    """Tests para decorador @controller."""
    
    def test_basic_controller(self):
        """Test crear controller básico."""
        @controller("/users")
        class UserController:
            pass
        
        assert is_controller(UserController) == True
        metadata = get_controller_metadata(UserController)
        assert metadata is not None
        assert metadata.base_path == "/users"
    
    def test_controller_with_prefix(self):
        """Test controller con prefix."""
        @controller("/products", prefix="/api")
        class ProductController:
            pass
        
        metadata = get_controller_metadata(ProductController)
        assert metadata.prefix == "/api"
        assert metadata.get_full_path() == "/api/products"
    
    def test_controller_with_tags(self):
        """Test controller con tags."""
        @controller("/orders", tags=["Orders", "Payments"])
        class OrderController:
            pass
        
        tags = get_controller_tags(OrderController)
        assert "Orders" in tags
        assert "Payments" in tags
    
    def test_controller_with_description(self):
        """Test controller con description."""
        @controller("/admin", description="Admin panel controller")
        class AdminController:
            pass
        
        metadata = get_controller_metadata(AdminController)
        assert metadata.description == "Admin panel controller"
    
    def test_controller_auto_registers(self):
        """Test controller se auto-registra en registry."""
        @controller("/test")
        class TestController:
            pass
        
        # Verificar que está en registry
        registered = get_controller(TestController)
        assert registered is not None
        assert registered.base_path == "/test"
    
    def test_controller_default_path(self):
        """Test controller con path por defecto."""
        @controller()
        class RootController:
            pass
        
        metadata = get_controller_metadata(RootController)
        assert metadata.base_path == "/"


# ============================================================================
# TEST: Helper Functions
# ============================================================================

class TestControllerHelpers:
    """Tests para helper functions."""
    
    def test_is_controller_true(self):
        """Test is_controller retorna True para controllers."""
        @controller("/users")
        class UserController:
            pass
        
        assert is_controller(UserController) == True
    
    def test_is_controller_false(self):
        """Test is_controller retorna False para no-controllers."""
        class NotAController:
            pass
        
        assert is_controller(NotAController) == False
    
    def test_get_controller_metadata_returns_metadata(self):
        """Test get_controller_metadata retorna metadata."""
        @controller("/users")
        class UserController:
            pass
        
        metadata = get_controller_metadata(UserController)
        assert metadata is not None
        assert isinstance(metadata, ControllerMetadata)
        assert metadata.base_path == "/users"
    
    def test_get_controller_metadata_returns_none(self):
        """Test get_controller_metadata retorna None para no-controllers."""
        class NotAController:
            pass
        
        assert get_controller_metadata(NotAController) is None
    
    def test_get_controller_base_path(self):
        """Test get_controller_base_path."""
        @controller("/products")
        class ProductController:
            pass
        
        base_path = get_controller_base_path(ProductController)
        assert base_path == "/products"
    
    def test_get_controller_full_path(self):
        """Test get_controller_full_path."""
        @controller("/products", prefix="/api")
        class ProductController:
            pass
        
        full_path = get_controller_full_path(ProductController)
        assert full_path == "/api/products"
    
    def test_get_controller_tags(self):
        """Test get_controller_tags."""
        @controller("/orders", tags=["Orders", "Admin"])
        class OrderController:
            pass
        
        tags = get_controller_tags(OrderController)
        assert "Orders" in tags
        assert "Admin" in tags
    
    def test_get_controller_tags_empty_for_non_controller(self):
        """Test get_controller_tags retorna lista vacía para no-controller."""
        class NotAController:
            pass
        
        tags = get_controller_tags(NotAController)
        assert tags == []


# ============================================================================
# TEST: Controller Registry
# ============================================================================

class TestControllerRegistry:
    """Tests para controller registry."""
    
    def test_register_controller(self):
        """Test registrar controller manualmente."""
        class TestController:
            pass
        
        metadata = ControllerMetadata(base_path="/test")
        register_controller(TestController, metadata)
        
        registered = get_controller(TestController)
        assert registered is not None
        assert registered == metadata
    
    def test_get_controller_returns_metadata(self):
        """Test get_controller retorna metadata."""
        @controller("/users")
        class UserController:
            pass
        
        registered = get_controller(UserController)
        assert registered is not None
        assert registered.base_path == "/users"
    
    def test_get_controller_returns_none_for_unregistered(self):
        """Test get_controller retorna None para no registrados."""
        class NotRegistered:
            pass
        
        assert get_controller(NotRegistered) is None
    
    def test_get_all_controllers(self):
        """Test get_all_controllers retorna todos los controllers."""
        @controller("/users")
        class UserController:
            pass
        
        @controller("/products")
        class ProductController:
            pass
        
        all_controllers = get_all_controllers()
        
        assert UserController in all_controllers
        assert ProductController in all_controllers
        assert len(all_controllers) == 2
    
    def test_clear_controller_registry(self):
        """Test clear_controller_registry limpia registry."""
        @controller("/test")
        class TestController:
            pass
        
        assert len(get_all_controllers()) == 1
        
        clear_controller_registry()
        
        assert len(get_all_controllers()) == 0
    
    def test_find_controller_by_path_exact_match(self):
        """Test find_controller_by_path con match exacto."""
        @controller("/users")
        class UserController:
            pass
        
        found = find_controller_by_path("/users")
        
        assert found == UserController
    
    def test_find_controller_by_path_prefix_match(self):
        """Test find_controller_by_path con prefix match."""
        @controller("/api/users", prefix="/v1")
        class UserController:
            pass
        
        # Full path es "/v1/api/users"
        found = find_controller_by_path("/v1/api/users/123")
        
        assert found == UserController
    
    def test_find_controller_by_path_returns_none(self):
        """Test find_controller_by_path retorna None si no encuentra."""
        found = find_controller_by_path("/nonexistent")
        
        assert found is None
    
    def test_find_controller_by_path_root_controller(self):
        """Test find_controller_by_path con root controller."""
        @controller("/")
        class RootController:
            pass
        
        found = find_controller_by_path("/anything")
        
        assert found == RootController
    
    def test_get_controllers_by_tag(self):
        """Test get_controllers_by_tag encuentra controllers."""
        @controller("/users", tags=["Users", "Admin"])
        class UserController:
            pass
        
        @controller("/products", tags=["Products"])
        class ProductController:
            pass
        
        @controller("/orders", tags=["Orders", "Admin"])
        class OrderController:
            pass
        
        admin_controllers = get_controllers_by_tag("Admin")
        
        assert UserController in admin_controllers
        assert OrderController in admin_controllers
        assert ProductController not in admin_controllers
        assert len(admin_controllers) == 2
    
    def test_get_controllers_by_tag_returns_empty_list(self):
        """Test get_controllers_by_tag retorna lista vacía si no encuentra."""
        controllers = get_controllers_by_tag("NonExistent")
        
        assert controllers == []


# ============================================================================
# TEST: Edge Cases
# ============================================================================

class TestControllerEdgeCases:
    """Tests para casos edge."""
    
    def test_controller_with_empty_string_path(self):
        """Test controller con path string vacío."""
        @controller("")
        class EmptyPathController:
            pass
        
        metadata = get_controller_metadata(EmptyPathController)
        # Se normaliza a "/"
        assert metadata.base_path == "/"
    
    def test_controller_with_complex_path(self):
        """Test controller con path complejo."""
        @controller("/api/v2/users/management")
        class ComplexController:
            pass
        
        metadata = get_controller_metadata(ComplexController)
        assert metadata.base_path == "/api/v2/users/management"
    
    def test_multiple_controllers_same_path_different_prefix(self):
        """Test múltiples controllers con mismo path pero diferente prefix."""
        @controller("/users", prefix="/v1")
        class UserV1Controller:
            pass
        
        @controller("/users", prefix="/v2")
        class UserV2Controller:
            pass
        
        metadata_v1 = get_controller_metadata(UserV1Controller)
        metadata_v2 = get_controller_metadata(UserV2Controller)
        
        assert metadata_v1.get_full_path() == "/v1/users"
        assert metadata_v2.get_full_path() == "/v2/users"
    
    def test_find_controller_by_path_prefers_specific_over_general(self):
        """Test find_controller_by_path prefiere paths específicos."""
        @controller("/")
        class RootController:
            pass
        
        @controller("/api/users")
        class UserController:
            pass
        
        # Debe encontrar UserController, no RootController
        found = find_controller_by_path("/api/users/123")
        
        assert found == UserController


# ============================================================================
# TEST: Integration
# ============================================================================

class TestControllerIntegration:
    """Tests de integración de @controller."""
    
    def test_complete_controller_scenario(self):
        """Test escenario completo de controller REST."""
        @controller("/api/users", prefix="/v1", tags=["Users", "API"], description="User management")
        class UserController:
            pass
        
        # Verificar metadata
        metadata = get_controller_metadata(UserController)
        assert metadata.base_path == "/api/users"
        assert metadata.prefix == "/v1"
        assert "Users" in metadata.tags
        assert "API" in metadata.tags
        assert metadata.description == "User management"
        assert metadata.get_full_path() == "/v1/api/users"
        
        # Verificar registry
        assert get_controller(UserController) == metadata
        
        # Verificar helpers
        assert is_controller(UserController) == True
        assert get_controller_base_path(UserController) == "/api/users"
        assert get_controller_full_path(UserController) == "/v1/api/users"
        assert "Users" in get_controller_tags(UserController)
        
        # Verificar búsqueda
        assert find_controller_by_path("/v1/api/users/123") == UserController
        assert UserController in get_controllers_by_tag("Users")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
