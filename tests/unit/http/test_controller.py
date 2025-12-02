"""
Tests for HTTP Controller DI Integration

Test Suite: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02
"""

import pytest
from typing import Optional
from src.runtime.http.controller import (
    ControllerRegistry,
    create_handler_from_method,
    register_controller_routes
)
from src.runtime.http.router import Router
from src.runtime.http.request import Request, HttpMethod
from src.runtime.http.response import Response, ok, not_found
from src.runtime.di.injector import Injector
from src.runtime.di.scopes import Scope


# Mock service for testing
class UserService:
    """Mock user service for testing DI."""
    
    def __init__(self):
        self.users = {
            "123": {"id": "123", "name": "Alice"},
            "456": {"id": "456", "name": "Bob"}
        }
    
    def get_user(self, user_id: str) -> Optional[dict]:
        """Get user by ID."""
        return self.users.get(user_id)
    
    def list_users(self) -> list:
        """List all users."""
        return list(self.users.values())
    
    def create_user(self, name: str) -> dict:
        """Create new user."""
        new_id = str(len(self.users) + 1)
        user = {"id": new_id, "name": name}
        self.users[new_id] = user
        return user


# Mock controller for testing
class UserController:
    """Mock controller for testing."""
    
    def __init__(self, user_service: UserService):
        self.user_service = user_service
    
    def get_user(self, request: Request) -> Response:
        """Get user by ID."""
        user_id = request.get_param('id')
        user = self.user_service.get_user(user_id)
        
        if user:
            return ok(user)
        return not_found(f"User {user_id} not found")
    
    def list_users(self, request: Request) -> Response:
        """List all users."""
        users = self.user_service.list_users()
        return ok({"users": users})
    
    def create_user(self, request: Request) -> Response:
        """Create new user."""
        name = request.body.get("name") if request.body else "Unknown"
        user = self.user_service.create_user(name)
        return ok(user)


class TestControllerRegistry:
    """Test suite for ControllerRegistry."""
    
    def test_registry_creation(self):
        """Test creating controller registry."""
        injector = Injector()
        registry = ControllerRegistry(injector)
        
        assert registry.injector == injector
        assert len(registry.controllers) == 0
    
    def test_register_controller(self):
        """Test registering controller."""
        injector = Injector()
        registry = ControllerRegistry(injector)
        
        registry.register_controller(UserController)
        
        assert UserController in registry.controllers
    
    def test_resolve_controller_without_dependencies(self):
        """Test resolving controller without dependencies."""
        injector = Injector()
        registry = ControllerRegistry(injector)
        
        # Simple controller without dependencies
        class SimpleController:
            def __init__(self):
                self.name = "SimpleController"
        
        registry.register_controller(SimpleController)
        instance = registry.resolve_controller(SimpleController)
        
        assert instance is not None
        assert instance.name == "SimpleController"
    
    def test_resolve_controller_with_dependencies(self):
        """Test resolving controller with dependencies."""
        injector = Injector()
        
        # Register service first
        injector.register(UserService, UserService, Scope.SCOPED)
        
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        # Resolve controller (should auto-inject UserService)
        instance = registry.resolve_controller(UserController)
        
        assert instance is not None
        assert isinstance(instance.user_service, UserService)
    
    def test_resolve_unregistered_controller_raises_error(self):
        """Test resolving unregistered controller raises error."""
        injector = Injector()
        registry = ControllerRegistry(injector)
        
        with pytest.raises(ValueError, match="not registered"):
            registry.resolve_controller(UserController)
    
    def test_resolve_controller_with_request_scope(self):
        """Test resolving controller with request scope."""
        injector = Injector()
        injector.register(UserService, UserService, Scope.SCOPED)
        
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        # Create request scope
        request_scope = {}
        
        # Resolve with scope
        instance1 = registry.resolve_controller(UserController, request_scope)
        instance2 = registry.resolve_controller(UserController, request_scope)
        
        # Should be same instance within same scope (Scoped)
        assert instance1 is instance2
    
    def test_resolve_controller_different_scopes(self):
        """Test resolving controller in different scopes."""
        injector = Injector()
        injector.register(UserService, UserService, Scope.SCOPED)
        
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        # Two different request scopes
        scope1 = {}
        scope2 = {}
        
        instance1 = registry.resolve_controller(UserController, scope1)
        instance2 = registry.resolve_controller(UserController, scope2)
        
        # Should be different instances (different scopes)
        assert instance1 is not instance2


class TestCreateHandlerFromMethod:
    """Test suite for create_handler_from_method."""
    
    def test_create_handler(self):
        """Test creating handler from controller method."""
        injector = Injector()
        injector.register(UserService, UserService, Scope.SCOPED)
        
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        # Create handler for get_user method
        handler = create_handler_from_method(
            UserController,
            "get_user",
            registry
        )
        
        assert callable(handler)
    
    def test_handler_calls_controller_method(self):
        """Test that handler correctly calls controller method."""
        injector = Injector()
        injector.register(UserService, UserService, Scope.SCOPED)
        
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        handler = create_handler_from_method(
            UserController,
            "get_user",
            registry
        )
        
        # Create request
        request = Request(
            method=HttpMethod.GET,
            path="/users/123",
            params={"id": "123"}
        )
        
        # Call handler
        response = handler(request)
        
        assert response.status == 200
        assert response.body["id"] == "123"
        assert response.body["name"] == "Alice"
    
    def test_handler_resolves_controller_per_request(self):
        """Test that handler resolves controller per request."""
        injector = Injector()
        injector.register(UserService, UserService, Scope.SCOPED)
        
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        handler = create_handler_from_method(
            UserController,
            "list_users",
            registry
        )
        
        # First request
        request1 = Request(method=HttpMethod.GET, path="/users")
        response1 = handler(request1)
        
        # Second request
        request2 = Request(method=HttpMethod.GET, path="/users")
        response2 = handler(request2)
        
        # Both should succeed (new controller instance per request)
        assert response1.status == 200
        assert response2.status == 200
    
    def test_handler_with_method_not_found(self):
        """Test handler with non-existent method."""
        injector = Injector()
        registry = ControllerRegistry(injector)
        
        class EmptyController:
            pass
        
        registry.register_controller(EmptyController)
        
        with pytest.raises(AttributeError):
            create_handler_from_method(
                EmptyController,
                "non_existent_method",
                registry
            )


class TestRegisterControllerRoutes:
    """Test suite for register_controller_routes."""
    
    def test_register_controller_routes(self):
        """Test registering controller routes on router."""
        injector = Injector()
        injector.register(UserService, UserService, Scope.SCOPED)
        
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        router = Router()
        
        # Register routes (manual for now, will be automatic via decorators)
        # This would normally be done via decorators like @get("/users/:id")
        handler_get = create_handler_from_method(UserController, "get_user", registry)
        handler_list = create_handler_from_method(UserController, "list_users", registry)
        
        router.get("/users/:id", handler_get)
        router.get("/users", handler_list)
        
        # Test route matching
        match = router.match(HttpMethod.GET, "/users/123")
        assert match is not None
        
        match2 = router.match(HttpMethod.GET, "/users")
        assert match2 is not None


class TestFullIntegration:
    """Test suite for full Router + DI + Controller integration."""
    
    def test_full_integration(self):
        """Test complete flow: Router → DI → Controller → Response."""
        # Setup DI
        injector = Injector()
        injector.register(UserService, UserService, Scope.SCOPED)
        
        # Setup registry
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        # Setup router
        router = Router()
        
        # Create handlers
        handler_get = create_handler_from_method(UserController, "get_user", registry)
        handler_list = create_handler_from_method(UserController, "list_users", registry)
        handler_create = create_handler_from_method(UserController, "create_user", registry)
        
        # Register routes
        router.get("/users/:id", handler_get)
        router.get("/users", handler_list)
        router.post("/users", handler_create)
        
        # Test GET /users/123
        response1 = router.handle(HttpMethod.GET, "/users/123")
        assert response1.status == 200
        assert response1.body["name"] == "Alice"
        
        # Test GET /users
        response2 = router.handle(HttpMethod.GET, "/users")
        assert response2.status == 200
        assert len(response2.body["users"]) == 2
        
        # Test POST /users
        # Note: In real scenario, body would be parsed from request
        # Here we're testing the handler directly
        request_create = Request(
            method=HttpMethod.POST,
            path="/users",
            body={"name": "Charlie"}
        )
        response3 = handler_create(request_create)
        assert response3.status == 200
        assert response3.body["name"] == "Charlie"
    
    def test_integration_with_not_found(self):
        """Test integration with non-existent resource."""
        injector = Injector()
        injector.register(UserService, UserService, Scope.SCOPED)
        
        registry = ControllerRegistry(injector)
        registry.register_controller(UserController)
        
        router = Router()
        handler_get = create_handler_from_method(UserController, "get_user", registry)
        router.get("/users/:id", handler_get)
        
        # Request non-existent user
        response = router.handle(HttpMethod.GET, "/users/999")
        
        assert response.status == 404
        assert "not found" in response.body["error"].lower()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
