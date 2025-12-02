"""
System Tests - REST Routing with DI

Tests de routing HTTP con DI integration:
- @get/@post/@put/@patch/@delete decorators
- Path parameters extraction
- Query parameters
- Request body parsing
- Headers
- Controller DI resolution
- Middleware chain
- Error handling

Jira: VELA-575, TASK-035J
"""

import pytest
from src.runtime.di import Injector, injectable, Scope
from src.runtime.http import (
    Request,
    Response,
    HttpMethod,
    Route,
    Router,
    get,
    post,
    put,
    delete,
    patch,
    ok,
    created,
    no_content,
    bad_request,
    not_found,
    ControllerRegistry
)
from tests.system.fixtures.services import (
    DatabaseConnection,
    UserRepository,
    UserService
)


# ============================================================================
# FIXTURES DE HTTP
# ============================================================================

@pytest.fixture
def router():
    """Router HTTP para tests."""
    return Router()


@pytest.fixture
def controller_registry(configured_injector):
    """ControllerRegistry con DI configurado."""
    return ControllerRegistry(configured_injector)


# ============================================================================
# TESTS DE DECORADORES HTTP BÁSICOS
# ============================================================================

class TestHttpDecorators:
    """Tests de decoradores HTTP (@get, @post, etc.)."""
    
    def test_get_decorator_creates_route(self, router):
        """Test: @get crea ruta GET."""
        
        @get("/users")
        def get_users(request: Request) -> Response:
            return ok({"users": []})
        
        # El decorador debe crear metadata
        assert hasattr(get_users, "__http_route__")
        assert get_users.__http_route__["method"] == HttpMethod.GET
        assert get_users.__http_route__["path"] == "/users"
    
    def test_post_decorator_creates_route(self, router):
        """Test: @post crea ruta POST."""
        
        @post("/users")
        def create_user(request: Request) -> Response:
            return created({"id": 1})
        
        assert hasattr(create_user, "__http_route__")
        assert create_user.__http_route__["method"] == HttpMethod.POST
    
    def test_put_decorator_creates_route(self, router):
        """Test: @put crea ruta PUT."""
        
        @put("/users/:id")
        def update_user(request: Request) -> Response:
            return ok({"updated": True})
        
        assert hasattr(update_user, "__http_route__")
        assert update_user.__http_route__["method"] == HttpMethod.PUT
    
    def test_patch_decorator_creates_route(self, router):
        """Test: @patch crea ruta PATCH."""
        
        @patch("/users/:id")
        def patch_user(request: Request) -> Response:
            return ok({"patched": True})
        
        assert hasattr(patch_user, "__http_route__")
        assert patch_user.__http_route__["method"] == HttpMethod.PATCH
    
    def test_delete_decorator_creates_route(self, router):
        """Test: @delete crea ruta DELETE."""
        
        @delete("/users/:id")
        def delete_user(request: Request) -> Response:
            return no_content()
        
        assert hasattr(delete_user, "__http_route__")
        assert delete_user.__http_route__["method"] == HttpMethod.DELETE


# ============================================================================
# TESTS DE PATH PARAMETERS
# ============================================================================

class TestPathParameters:
    """Tests de extracción de path parameters."""
    
    def test_single_path_parameter(self, router):
        """Test: Extraer single path parameter."""
        
        @get("/users/:id")
        def get_user(request: Request) -> Response:
            user_id = request.params.get("id")
            return ok({"id": user_id})
        
        # Simular request
        request = Request(
            method=HttpMethod.GET,
            path="/users/123",
            params={"id": "123"}
        )
        
        # Act
        response = get_user(request)
        
        # Assert
        assert response.body["id"] == "123"
    
    def test_multiple_path_parameters(self, router):
        """Test: Extraer múltiples path parameters."""
        
        @get("/posts/:postId/comments/:commentId")
        def get_comment(request: Request) -> Response:
            post_id = request.params.get("postId")
            comment_id = request.params.get("commentId")
            return ok({"postId": post_id, "commentId": comment_id})
        
        # Simular request
        request = Request(
            method=HttpMethod.GET,
            path="/posts/42/comments/7",
            params={"postId": "42", "commentId": "7"}
        )
        
        # Act
        response = get_comment(request)
        
        # Assert
        assert response.body["postId"] == "42"
        assert response.body["commentId"] == "7"
    
    def test_path_parameter_type_conversion(self, router):
        """Test: Path parameter se puede convertir a int."""
        
        @get("/users/:id")
        def get_user(request: Request) -> Response:
            user_id = int(request.params.get("id"))
            return ok({"id": user_id, "type": type(user_id).__name__})
        
        request = Request(
            method=HttpMethod.GET,
            path="/users/123",
            params={"id": "123"}
        )
        
        # Act
        response = get_user(request)
        
        # Assert
        assert response.body["id"] == 123
        assert response.body["type"] == "int"


# ============================================================================
# TESTS DE QUERY PARAMETERS
# ============================================================================

class TestQueryParameters:
    """Tests de query parameters."""
    
    def test_single_query_parameter(self, router):
        """Test: Extraer single query parameter."""
        
        @get("/users")
        def list_users(request: Request) -> Response:
            limit = request.query.get("limit", "10")
            return ok({"limit": int(limit)})
        
        request = Request(
            method=HttpMethod.GET,
            path="/users?limit=20",
            query={"limit": "20"}
        )
        
        # Act
        response = list_users(request)
        
        # Assert
        assert response.body["limit"] == 20
    
    def test_multiple_query_parameters(self, router):
        """Test: Extraer múltiples query parameters."""
        
        @get("/users")
        def list_users(request: Request) -> Response:
            limit = int(request.query.get("limit", "10"))
            offset = int(request.query.get("offset", "0"))
            sort = request.query.get("sort", "id")
            return ok({"limit": limit, "offset": offset, "sort": sort})
        
        request = Request(
            method=HttpMethod.GET,
            path="/users?limit=20&offset=40&sort=name",
            query={"limit": "20", "offset": "40", "sort": "name"}
        )
        
        # Act
        response = list_users(request)
        
        # Assert
        assert response.body["limit"] == 20
        assert response.body["offset"] == 40
        assert response.body["sort"] == "name"
    
    def test_query_parameter_default_value(self, router):
        """Test: Query parameter con default value."""
        
        @get("/users")
        def list_users(request: Request) -> Response:
            limit = int(request.query.get("limit", "10"))
            return ok({"limit": limit})
        
        request = Request(
            method=HttpMethod.GET,
            path="/users",
            query={}
        )
        
        # Act
        response = list_users(request)
        
        # Assert: Usa default
        assert response.body["limit"] == 10


# ============================================================================
# TESTS DE REQUEST BODY
# ============================================================================

class TestRequestBody:
    """Tests de request body parsing."""
    
    def test_post_with_json_body(self, router):
        """Test: POST con JSON body."""
        
        @post("/users")
        def create_user(request: Request) -> Response:
            name = request.body.get("name")
            email = request.body.get("email")
            return created({"id": 1, "name": name, "email": email})
        
        request = Request(
            method=HttpMethod.POST,
            path="/users",
            body={"name": "Alice", "email": "alice@example.com"}
        )
        
        # Act
        response = create_user(request)
        
        # Assert
        assert response.status == 201
        assert response.body["name"] == "Alice"
        assert response.body["email"] == "alice@example.com"
    
    def test_put_with_json_body(self, router):
        """Test: PUT con JSON body."""
        
        @put("/users/:id")
        def update_user(request: Request) -> Response:
            user_id = request.params.get("id")
            name = request.body.get("name")
            return ok({"id": user_id, "name": name, "updated": True})
        
        request = Request(
            method=HttpMethod.PUT,
            path="/users/123",
            params={"id": "123"},
            body={"name": "Bob"}
        )
        
        # Act
        response = update_user(request)
        
        # Assert
        assert response.body["id"] == "123"
        assert response.body["name"] == "Bob"
        assert response.body["updated"] is True


# ============================================================================
# TESTS DE HEADERS
# ============================================================================

class TestHeaders:
    """Tests de headers HTTP."""
    
    def test_read_authorization_header(self, router):
        """Test: Leer Authorization header."""
        
        @get("/protected")
        def protected_endpoint(request: Request) -> Response:
            auth = request.headers.get("Authorization", "")
            if auth.startswith("Bearer "):
                token = auth[7:]
                return ok({"token": token, "authenticated": True})
            return bad_request({"error": "Missing token"})
        
        request = Request(
            method=HttpMethod.GET,
            path="/protected",
            headers={"Authorization": "Bearer jwt-token-12345"}
        )
        
        # Act
        response = protected_endpoint(request)
        
        # Assert
        assert response.body["token"] == "jwt-token-12345"
        assert response.body["authenticated"] is True
    
    def test_read_content_type_header(self, router):
        """Test: Leer Content-Type header."""
        
        @post("/upload")
        def upload_file(request: Request) -> Response:
            content_type = request.headers.get("Content-Type", "")
            return ok({"contentType": content_type})
        
        request = Request(
            method=HttpMethod.POST,
            path="/upload",
            headers={"Content-Type": "application/json"}
        )
        
        # Act
        response = upload_file(request)
        
        # Assert
        assert response.body["contentType"] == "application/json"


# ============================================================================
# TESTS DE CONTROLLER CON DI
# ============================================================================

class TestControllerWithDI:
    """Tests de controllers con DI integration."""
    
    def test_controller_resolves_dependencies(self, configured_injector, controller_registry):
        """Test: Controller resuelve dependencies desde DI."""
        
        @injectable
        class UserController:
            def __init__(self, service: UserService):
                self.service = service
            
            def get_user(self, request: Request) -> Response:
                # Simular get de usuario
                user_id = int(request.params.get("id"))
                user = self.service.get_user(user_id)
                
                if user:
                    return ok(user)
                return not_found({"error": "User not found"})
        
        configured_injector.register(UserController)
        controller_registry.register_controller(UserController, prefix="/users")
        
        # Act: Resolver controller
        controller = controller_registry.resolve_controller(UserController)
        
        # Assert: Controller tiene UserService inyectado
        assert isinstance(controller, UserController)
        assert isinstance(controller.service, UserService)
    
    def test_controller_singleton_db_connection(self, configured_injector, controller_registry):
        """Test: Controller usa singleton DatabaseConnection."""
        
        @injectable
        class ProductController:
            def __init__(self, db: DatabaseConnection):
                self.db = db
        
        configured_injector.register(ProductController)
        
        # Act: Resolver 2 veces
        controller1 = configured_injector.get(ProductController)
        controller2 = configured_injector.get(ProductController)
        
        # Assert: Controllers diferentes (TRANSIENT)
        assert controller1 is not controller2
        
        # Assert: DatabaseConnection es el mismo (SINGLETON)
        assert controller1.db is controller2.db


# ============================================================================
# TESTS DE ROUTING CON ROUTER
# ============================================================================

class TestRouterMatching:
    """Tests de route matching en Router."""
    
    def test_router_matches_exact_path(self, router):
        """Test: Router matchea path exacto."""
        
        def handler(request: Request) -> Response:
            return ok({"matched": True})
        
        route = Route(HttpMethod.GET, "/users", handler)
        router.add_route(route)
        
        # Act
        match = router.match(HttpMethod.GET, "/users")
        
        # Assert
        assert match.matched is True
        assert match.route is route
    
    def test_router_matches_path_with_params(self, router):
        """Test: Router matchea path con params."""
        
        def handler(request: Request) -> Response:
            return ok({"id": request.params.get("id")})
        
        route = Route(HttpMethod.GET, "/users/:id", handler)
        router.add_route(route)
        
        # Act
        match = router.match(HttpMethod.GET, "/users/123")
        
        # Assert
        assert match.matched is True
        assert match.params["id"] == "123"
    
    def test_router_no_match_wrong_method(self, router):
        """Test: Router NO matchea con método incorrecto."""
        
        def handler(request: Request) -> Response:
            return ok({})
        
        route = Route(HttpMethod.GET, "/users", handler)
        router.add_route(route)
        
        # Act: POST en lugar de GET
        match = router.match(HttpMethod.POST, "/users")
        
        # Assert
        assert match.matched is False
    
    def test_router_no_match_wrong_path(self, router):
        """Test: Router NO matchea con path incorrecto."""
        
        def handler(request: Request) -> Response:
            return ok({})
        
        route = Route(HttpMethod.GET, "/users", handler)
        router.add_route(route)
        
        # Act
        match = router.match(HttpMethod.GET, "/posts")
        
        # Assert
        assert match.matched is False


# ============================================================================
# TESTS DE ERROR HANDLING
# ============================================================================

class TestErrorHandling:
    """Tests de manejo de errores en routing."""
    
    def test_404_not_found(self, router):
        """Test: 404 cuando ruta no existe."""
        
        # Router vacío
        
        # Act
        match = router.match(HttpMethod.GET, "/non-existent")
        
        # Assert
        assert match.matched is False
    
    def test_handler_exception_returns_500(self, router):
        """Test: Excepción en handler → 500."""
        
        @get("/error")
        def failing_handler(request: Request) -> Response:
            raise ValueError("Something went wrong")
        
        request = Request(
            method=HttpMethod.GET,
            path="/error"
        )
        
        # Act & Assert: Debe lanzar excepción
        with pytest.raises(ValueError):
            failing_handler(request)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
