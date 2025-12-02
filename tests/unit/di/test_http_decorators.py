"""
Tests Unitarios para HTTP Decorators

Suite completa de tests para validar:
- HTTPMethod enum
- ParameterType enum
- RouteMetadata dataclass
- Decoradores HTTP (@get, @post, @put, @patch, @delete, @head, @options)
- Decoradores de parámetros (@param, @query, @body, @header, @cookie, @request, @response)
- Helper functions
- Integración con controllers

Author: Vela Team
Version: 0.2.0  # Completada con @cookie, @request, @response
Created: 2025-12-01
Updated: 2025-12-01 (agregados @cookie, @request, @response)
"""

import pytest
from dataclasses import FrozenInstanceError
from src.runtime.di.http_decorators import (
    HTTPMethod,
    ParameterType,
    RouteMetadata,
    ParameterMetadata,
    ParameterMarker,
    get, post, put, patch, delete, head, options,
    param, query, body, header, cookie, request, response,
    is_route_handler,
    get_route_metadata,
    get_all_routes,
    get_routes_by_method,
    get_route_by_path,
)


# ================================
# Tests HTTPMethod Enum
# ================================

class TestHTTPMethod:
    """Tests para HTTPMethod enum."""
    
    def test_http_method_values(self):
        """Test que HTTPMethod tiene todos los valores correctos."""
        assert HTTPMethod.GET.value == "GET"
        assert HTTPMethod.POST.value == "POST"
        assert HTTPMethod.PUT.value == "PUT"
        assert HTTPMethod.PATCH.value == "PATCH"
        assert HTTPMethod.DELETE.value == "DELETE"
        assert HTTPMethod.HEAD.value == "HEAD"
        assert HTTPMethod.OPTIONS.value == "OPTIONS"
    
    def test_http_method_string_conversion(self):
        """Test que HTTPMethod se convierte a string correctamente."""
        assert str(HTTPMethod.GET) == "GET"
        assert str(HTTPMethod.POST) == "POST"
    
    def test_http_method_comparison(self):
        """Test que HTTPMethod se puede comparar."""
        assert HTTPMethod.GET == HTTPMethod.GET
        assert HTTPMethod.GET != HTTPMethod.POST
    
    def test_http_method_is_string(self):
        """Test que HTTPMethod hereda de str."""
        method = HTTPMethod.GET
        assert isinstance(method, str)


# ================================
# Tests ParameterType Enum
# ================================

class TestParameterType:
    """Tests para ParameterType enum."""
    
    def test_parameter_type_values(self):
        """Test que ParameterType tiene todos los valores."""
        assert ParameterType.PATH.value == "path"
        assert ParameterType.QUERY.value == "query"
        assert ParameterType.BODY.value == "body"
        assert ParameterType.HEADER.value == "header"
        assert ParameterType.COOKIE.value == "cookie"


# ================================
# Tests RouteMetadata
# ================================

class TestRouteMetadata:
    """Tests para RouteMetadata dataclass."""
    
    def test_create_minimal_metadata(self):
        """Test crear metadata con valores mínimos."""
        metadata = RouteMetadata(method=HTTPMethod.GET)
        
        assert metadata.method == HTTPMethod.GET
        assert metadata.path == "/"
        assert metadata.path_params == {}
        assert metadata.query_params == {}
        assert metadata.body_type is None
        assert metadata.headers == {}
        assert metadata.middleware == []
        assert metadata.description is None
        assert metadata.tags == []
    
    def test_normalize_path_without_leading_slash(self):
        """Test que path sin / inicial se normaliza."""
        metadata = RouteMetadata(method=HTTPMethod.GET, path="users")
        assert metadata.path == "/users"
    
    def test_path_with_leading_slash_unchanged(self):
        """Test que path con / inicial no cambia."""
        metadata = RouteMetadata(method=HTTPMethod.GET, path="/users")
        assert metadata.path == "/users"
    
    def test_tags_as_string_converts_to_list(self):
        """Test que tags como string se convierte a lista."""
        metadata = RouteMetadata(method=HTTPMethod.GET, tags="Users")
        assert metadata.tags == ["Users"]
    
    def test_tags_as_list(self):
        """Test que tags como lista se mantiene."""
        metadata = RouteMetadata(method=HTTPMethod.GET, tags=["Users", "Admin"])
        assert metadata.tags == ["Users", "Admin"]
    
    def test_tags_none_converts_to_empty_list(self):
        """Test que tags=None se convierte a []."""
        metadata = RouteMetadata(method=HTTPMethod.GET, tags=None)
        assert metadata.tags == []
    
    def test_get_full_route_with_root_path(self):
        """Test get_full_route con path='/'."""
        metadata = RouteMetadata(method=HTTPMethod.GET, path="/")
        full_route = metadata.get_full_route("/api/users")
        assert full_route == "/api/users"
    
    def test_get_full_route_with_subpath(self):
        """Test get_full_route con subpath."""
        metadata = RouteMetadata(method=HTTPMethod.GET, path="/:id")
        full_route = metadata.get_full_route("/api/users")
        assert full_route == "/api/users/:id"
    
    def test_get_full_route_removes_trailing_slash(self):
        """Test que get_full_route remueve trailing slash del controller."""
        metadata = RouteMetadata(method=HTTPMethod.GET, path="/:id")
        full_route = metadata.get_full_route("/api/users/")
        assert full_route == "/api/users/:id"
    
    def test_with_all_fields(self):
        """Test crear metadata con todos los campos."""
        metadata = RouteMetadata(
            method=HTTPMethod.POST,
            path="/create",
            path_params={"id": int},
            query_params={"page": int},
            body_type=dict,
            headers={"Content-Type": "application/json"},
            middleware=[lambda: None],
            description="Create user",
            tags=["Users"]
        )
        
        assert metadata.method == HTTPMethod.POST
        assert metadata.path == "/create"
        assert metadata.path_params == {"id": int}
        assert metadata.query_params == {"page": int}
        assert metadata.body_type == dict
        assert metadata.headers["Content-Type"] == "application/json"
        assert len(metadata.middleware) == 1
        assert metadata.description == "Create user"
        assert metadata.tags == ["Users"]


# ================================
# Tests ParameterMetadata
# ================================

class TestParameterMetadata:
    """Tests para ParameterMetadata dataclass."""
    
    def test_create_required_parameter(self):
        """Test crear parámetro requerido."""
        param_meta = ParameterMetadata(
            name="id",
            param_type=ParameterType.PATH,
            expected_type=int,
            required=True
        )
        
        assert param_meta.name == "id"
        assert param_meta.param_type == ParameterType.PATH
        assert param_meta.expected_type == int
        assert param_meta.required is True
        assert param_meta.default is None
    
    def test_create_optional_parameter_with_default(self):
        """Test crear parámetro opcional con default."""
        param_meta = ParameterMetadata(
            name="page",
            param_type=ParameterType.QUERY,
            expected_type=int,
            default=1
        )
        
        assert param_meta.default == 1
        assert param_meta.required is False  # Auto-set en __post_init__
    
    def test_parameter_with_description(self):
        """Test parámetro con descripción."""
        param_meta = ParameterMetadata(
            name="token",
            param_type=ParameterType.HEADER,
            expected_type=str,
            description="Auth token"
        )
        
        assert param_meta.description == "Auth token"


# ================================
# Tests HTTP Method Decorators
# ================================

class TestHTTPMethodDecorators:
    """Tests para decoradores HTTP (@get, @post, etc.)."""
    
    def test_get_decorator_basic(self):
        """Test @get decorator básico."""
        @get("/users")
        def list_users():
            return []
        
        assert is_route_handler(list_users)
        metadata = get_route_metadata(list_users)
        assert metadata.method == HTTPMethod.GET
        assert metadata.path == "/users"
    
    def test_get_decorator_with_path_param(self):
        """Test @get con parámetro de ruta."""
        @get("/:id")
        def get_user(id: int):
            return {"id": id}
        
        metadata = get_route_metadata(get_user)
        assert metadata.method == HTTPMethod.GET
        assert metadata.path == "/:id"
        assert "id" in metadata.path_params
        assert metadata.path_params["id"] == int
    
    def test_get_decorator_with_description(self):
        """Test @get con descripción."""
        @get("/users", description="List all users")
        def list_users():
            return []
        
        metadata = get_route_metadata(list_users)
        assert metadata.description == "List all users"
    
    def test_get_decorator_with_tags(self):
        """Test @get con tags."""
        @get("/users", tags=["Users", "Public"])
        def list_users():
            return []
        
        metadata = get_route_metadata(list_users)
        assert metadata.tags == ["Users", "Public"]
    
    def test_post_decorator(self):
        """Test @post decorator."""
        @post("/users")
        def create_user():
            return {"created": True}
        
        metadata = get_route_metadata(create_user)
        assert metadata.method == HTTPMethod.POST
        assert metadata.path == "/users"
    
    def test_put_decorator(self):
        """Test @put decorator."""
        @put("/:id")
        def update_user(id: int):
            return {"updated": True}
        
        metadata = get_route_metadata(update_user)
        assert metadata.method == HTTPMethod.PUT
    
    def test_patch_decorator(self):
        """Test @patch decorator."""
        @patch("/:id")
        def partial_update(id: int):
            return {"patched": True}
        
        metadata = get_route_metadata(partial_update)
        assert metadata.method == HTTPMethod.PATCH
    
    def test_delete_decorator(self):
        """Test @delete decorator."""
        @delete("/:id")
        def delete_user(id: int):
            return {"deleted": True}
        
        metadata = get_route_metadata(delete_user)
        assert metadata.method == HTTPMethod.DELETE
    
    def test_head_decorator(self):
        """Test @head decorator."""
        @head("/status")
        def check_status():
            return None
        
        metadata = get_route_metadata(check_status)
        assert metadata.method == HTTPMethod.HEAD
    
    def test_options_decorator(self):
        """Test @options decorator."""
        @options("/")
        def get_options():
            return {"methods": ["GET", "POST"]}
        
        metadata = get_route_metadata(get_options)  # Corregido: usar get_options, no options
        assert metadata.method == HTTPMethod.OPTIONS
    
    def test_decorator_with_middleware(self):
        """Test decorator con middleware."""
        def auth_middleware():
            pass
        
        @get("/protected", middleware=[auth_middleware])
        def protected_route():
            return {"data": "secret"}
        
        metadata = get_route_metadata(protected_route)
        assert len(metadata.middleware) == 1
        assert metadata.middleware[0] == auth_middleware
    
    def test_function_remains_callable(self):
        """Test que función decorada sigue siendo ejecutable."""
        @get("/test")
        def test_func():
            return "result"
        
        result = test_func()
        assert result == "result"


# ================================
# Tests Parameter Decorators
# ================================

class TestParameterDecorators:
    """Tests para decoradores de parámetros."""
    
    def test_param_decorator_basic(self):
        """Test @param decorator básico."""
        marker = param(param_type=int)
        
        assert isinstance(marker, ParameterMarker)
        assert marker.__parameter_metadata__.param_type == ParameterType.PATH
        assert marker.__parameter_metadata__.expected_type == int
    
    def test_param_decorator_with_name(self):
        """Test @param con nombre explícito."""
        marker = param(name="userId", param_type=int)
        
        assert marker.__parameter_metadata__.name == "userId"
    
    def test_query_decorator_basic(self):
        """Test @query decorator básico."""
        marker = query(param_type=int, default=1)
        
        assert marker.__parameter_metadata__.param_type == ParameterType.QUERY
        assert marker.__parameter_metadata__.default == 1
        assert marker.__parameter_metadata__.required is False
    
    def test_query_decorator_required(self):
        """Test @query con required=True."""
        marker = query(param_type=str, required=True)
        
        assert marker.__parameter_metadata__.required is True
    
    def test_body_decorator(self):
        """Test @body decorator."""
        marker = body(dict, description="User DTO")
        
        assert marker.__parameter_metadata__.param_type == ParameterType.BODY
        assert marker.__parameter_metadata__.expected_type == dict
        assert marker.__parameter_metadata__.description == "User DTO"
    
    def test_header_decorator(self):
        """Test @header decorator."""
        marker = header("Authorization", param_type=str, required=True)
        
        assert marker.__parameter_metadata__.param_type == ParameterType.HEADER
        assert marker.__parameter_metadata__.name == "Authorization"
        assert marker.__parameter_metadata__.required is True
    
    def test_parameter_with_body_decorator(self):
        """Test función con @body decorator."""
        @post("/users")
        def create_user(dto=body(dict)):
            return {"created": True}
        
        metadata = get_route_metadata(create_user)
        assert metadata.body_type == dict
    
    def test_cookie_decorator(self):
        """Test @cookie decorator."""
        marker = cookie("session_id", param_type=str, required=False, default="")
        
        assert marker.__parameter_metadata__.param_type == ParameterType.COOKIE
        assert marker.__parameter_metadata__.name == "session_id"
        assert marker.__parameter_metadata__.required is False
        assert marker.__parameter_metadata__.default == ""
    
    def test_cookie_decorator_required(self):
        """Test @cookie decorator con required=True."""
        marker = cookie("auth_token", required=True)
        
        assert marker.__parameter_metadata__.param_type == ParameterType.COOKIE
        assert marker.__parameter_metadata__.name == "auth_token"
        assert marker.__parameter_metadata__.required is True
    
    def test_request_decorator(self):
        """Test @request decorator para inyectar Request object."""
        marker = request(description="Request object")
        
        assert marker.__parameter_metadata__.name == "__request__"
        assert marker.__parameter_metadata__.required is True
        assert marker.__parameter_metadata__.description == "Request object"
        assert marker.__parameter_metadata__.expected_type == object
    
    def test_response_decorator(self):
        """Test @response decorator para inyectar Response object."""
        marker = response(description="Response object")
        
        assert marker.__parameter_metadata__.name == "__response__"
        assert marker.__parameter_metadata__.required is True
        assert marker.__parameter_metadata__.description == "Response object"
        assert marker.__parameter_metadata__.expected_type == object
    
    def test_function_with_cookie_decorator(self):
        """Test función con @cookie decorator."""
        @get("/profile")
        def get_profile(session_id=cookie("session_id", default="anonymous")):
            return {"session": session_id}
        
        metadata = get_route_metadata(get_profile)
        assert metadata is not None
        # Cookie params NO se agregan a query_params ni path_params
        # Se procesan en runtime desde req.cookies


# ================================
# Tests Helper Functions
# ================================

class TestHelperFunctions:
    """Tests para helper functions."""
    
    def test_is_route_handler_true(self):
        """Test is_route_handler retorna True para función decorada."""
        @get("/test")
        def handler():
            pass
        
        assert is_route_handler(handler) is True
    
    def test_is_route_handler_false(self):
        """Test is_route_handler retorna False para función no decorada."""
        def regular_function():
            pass
        
        assert is_route_handler(regular_function) is False
    
    def test_get_route_metadata_returns_metadata(self):
        """Test get_route_metadata retorna metadata."""
        @get("/test")
        def handler():
            pass
        
        metadata = get_route_metadata(handler)
        assert isinstance(metadata, RouteMetadata)
        assert metadata.method == HTTPMethod.GET
    
    def test_get_route_metadata_returns_none(self):
        """Test get_route_metadata retorna None si no hay metadata."""
        def regular_function():
            pass
        
        metadata = get_route_metadata(regular_function)
        assert metadata is None
    
    def test_get_all_routes_from_controller(self):
        """Test get_all_routes obtiene todas las rutas."""
        class TestController:
            @get("/")
            def list(self):
                return []
            
            @get("/:id")
            def get(self, id: int):
                return {}
            
            @post("/")
            def create(self):
                return {}
            
            def not_a_route(self):
                # Método sin decorator
                pass
        
        routes = get_all_routes(TestController)
        
        assert len(routes) == 3
        assert "list" in routes
        assert "get" in routes
        assert "create" in routes
        assert "not_a_route" not in routes
    
    def test_get_all_routes_empty_controller(self):
        """Test get_all_routes con controller sin rutas."""
        class EmptyController:
            def not_a_route(self):
                pass
        
        routes = get_all_routes(EmptyController)
        assert routes == {}
    
    def test_get_routes_by_method(self):
        """Test get_routes_by_method filtra por método HTTP."""
        class TestController:
            @get("/")
            def list(self):
                return []
            
            @get("/:id")
            def get(self, id: int):
                return {}
            
            @post("/")
            def create(self):
                return {}
        
        get_routes = get_routes_by_method(TestController, HTTPMethod.GET)
        post_routes = get_routes_by_method(TestController, HTTPMethod.POST)
        
        assert len(get_routes) == 2
        assert len(post_routes) == 1
        assert "list" in get_routes
        assert "get" in get_routes
        assert "create" in post_routes
    
    def test_get_routes_by_method_no_matches(self):
        """Test get_routes_by_method sin matches."""
        class TestController:
            @get("/")
            def list(self):
                return []
        
        delete_routes = get_routes_by_method(TestController, HTTPMethod.DELETE)
        assert delete_routes == {}
    
    def test_get_route_by_path_exact_match(self):
        """Test get_route_by_path encuentra ruta exacta."""
        class TestController:
            @get("/users")
            def list_users(self):
                return []
            
            @get("/:id")
            def get_user(self, id: int):
                return {}
        
        result = get_route_by_path(TestController, "/users")
        
        assert result is not None
        method_name, metadata = result
        assert method_name == "list_users"
        assert metadata.path == "/users"
    
    def test_get_route_by_path_not_found(self):
        """Test get_route_by_path retorna None si no encuentra."""
        class TestController:
            @get("/users")
            def list_users(self):
                return []
        
        result = get_route_by_path(TestController, "/posts")
        assert result is None


# ================================
# Tests Integration
# ================================

class TestIntegration:
    """Tests de integración entre decoradores y controllers."""
    
    def test_complete_crud_controller(self):
        """Test controller CRUD completo."""
        class UserController:
            @get("/")
            def list(self):
                return []
            
            @get("/:id")
            def get(self, id: int):
                return {"id": id}
            
            @post("/")
            def create(self, dto=body(dict)):
                return {"created": True}
            
            @put("/:id")
            def update(self, id: int, dto=body(dict)):
                return {"updated": True}
            
            @delete("/:id")
            def delete(self, id: int):
                return {"deleted": True}
        
        routes = get_all_routes(UserController)
        assert len(routes) == 5
        
        # Verificar métodos HTTP
        get_routes = get_routes_by_method(UserController, HTTPMethod.GET)
        assert len(get_routes) == 2
        
        post_routes = get_routes_by_method(UserController, HTTPMethod.POST)
        assert len(post_routes) == 1
        
        # Verificar metadata específica
        create_meta = get_route_metadata(UserController.create)
        assert create_meta.body_type == dict
        
        delete_meta = get_route_metadata(UserController.delete)
        assert delete_meta.method == HTTPMethod.DELETE
    
    def test_controller_with_multiple_parameters(self):
        """Test controller con múltiples tipos de parámetros."""
        class AdvancedController:
            @get("/search")
            def search(
                self,
                q=query("query", str, required=True),
                page=query("page", int, default=1),
                limit=query("limit", int, default=10)
            ):
                return []
            
            @post("/upload")
            def upload(
                self,
                file=body(bytes),
                token=header("Authorization", str, required=True)
            ):
                return {"uploaded": True}
        
        search_meta = get_route_metadata(AdvancedController.search)
        assert "q" in search_meta.query_params or "page" in search_meta.query_params
        
        upload_meta = get_route_metadata(AdvancedController.upload)
        assert upload_meta.body_type == bytes
    
    def test_routes_with_tags_and_descriptions(self):
        """Test rutas con tags y descripciones."""
        class DocumentedController:
            @get("/", tags=["Public"], description="List all items")
            def list(self):
                return []
            
            @post("/", tags=["Admin"], description="Create new item")
            def create(self):
                return {}
        
        list_meta = get_route_metadata(DocumentedController.list)
        assert list_meta.tags == ["Public"]
        assert list_meta.description == "List all items"
        
        create_meta = get_route_metadata(DocumentedController.create)
        assert create_meta.tags == ["Admin"]
        assert create_meta.description == "Create new item"
    
    def test_controller_with_request_response_decorators(self):
        """Test controller con @request y @response decorators."""
        class AdvancedHTTPController:
            @get("/info")
            def get_info(self, req=request(description="HTTP Request")):
                """Endpoint que recibe Request object completo."""
                return {"ip": "req.ip"}
            
            @get("/download/:id")
            def download(self, res=response(description="HTTP Response"), id: int = 0):
                """Endpoint que manipula Response object directamente."""
                return None  # Response se maneja directamente
            
            @get("/profile")
            def get_profile(
                self,
                session_id=cookie("session_id", default="anonymous"),
                token=header("Authorization", required=False)
            ):
                """Endpoint con cookie y header."""
                return {"session": session_id}
        
        # Test @request
        info_meta = get_route_metadata(AdvancedHTTPController.get_info)
        assert info_meta is not None
        assert info_meta.method == HTTPMethod.GET
        
        # Test @response
        download_meta = get_route_metadata(AdvancedHTTPController.download)
        assert download_meta is not None
        assert download_meta.method == HTTPMethod.GET
        
        # Test @cookie + @header
        profile_meta = get_route_metadata(AdvancedHTTPController.get_profile)
        assert profile_meta is not None
        assert profile_meta.method == HTTPMethod.GET


# ================================
# Tests Edge Cases
# ================================

class TestEdgeCases:
    """Tests de casos límite."""
    
    def test_empty_path(self):
        """Test decorator con path vacío se normaliza a /."""
        @get("")
        def root():
            return {}
        
        metadata = get_route_metadata(root)
        assert metadata.path == "/"
    
    def test_multiple_decorators_on_same_function(self):
        """Test función con múltiples decorators (primero se aplica el más cercano)."""
        @post("/test2")  # Se aplica segundo (más externo)
        @get("/test1")   # Se aplica primero (más cercano a función)
        def multi_decorated():
            return {}
        
        # El decorator más externo (@post) envuelve al interno (@get)
        # Por tanto, @post sobrescribe completamente
        metadata = get_route_metadata(multi_decorated)
        assert metadata.method == HTTPMethod.POST
    
    def test_route_with_complex_path(self):
        """Test ruta con path complejo."""
        @get("/api/v1/users/:userId/posts/:postId/comments")
        def complex_route(userId: int, postId: int):
            return []
        
        metadata = get_route_metadata(complex_route)
        assert metadata.path == "/api/v1/users/:userId/posts/:postId/comments"
    
    def test_controller_with_no_methods(self):
        """Test controller sin métodos."""
        class EmptyController:
            pass
        
        routes = get_all_routes(EmptyController)
        assert routes == {}
    
    def test_route_metadata_immutability(self):
        """Test que RouteMetadata fields no son inmutables por defecto."""
        metadata = RouteMetadata(method=HTTPMethod.GET)
        
        # Dataclasses son mutables por defecto (a menos que frozen=True)
        metadata.path = "/changed"
        assert metadata.path == "/changed"


# ================================
# Run Tests
# ================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
