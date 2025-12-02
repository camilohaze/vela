"""
System Tests - End-to-End Integration

Tests de integración completos que validan:
- User CRUD flow completo
- Auth flow (login, token validation, logout)
- Middleware chain execution
- Error handling end-to-end
- Performance benchmarks

Estos tests validan el sistema completo: DI + HTTP + Middleware + Guards

Jira: VELA-575, TASK-035J
"""

import pytest
import time
from src.runtime.di import Injector, injectable, Scope
from src.runtime.http import (
    Request,
    Response,
    HttpMethod,
    ok,
    created,
    no_content,
    bad_request,
    unauthorized,
    not_found
)
from tests.system.fixtures.services import (
    DatabaseConnection,
    UserRepository,
    UserService,
    AuthService,
    RequestContext
)


# ============================================================================
# FIXTURES DE INTEGRACIÓN
# ============================================================================

@pytest.fixture
def integrated_injector():
    """
    Injector configurado con TODOS los servicios para tests end-to-end.
    """
    injector = Injector()
    
    # Core services
    injector.register(DatabaseConnection, scope=Scope.SINGLETON)
    injector.register(UserRepository)
    injector.register(UserService)
    injector.register(AuthService)
    injector.register(RequestContext, scope=Scope.SCOPED)
    
    return injector


@pytest.fixture
def user_controller(integrated_injector):
    """
    UserController completo con DI para tests end-to-end.
    """
    
    @injectable
    class UserController:
        def __init__(self, service: UserService):
            self.service = service
        
        def create_user(self, request: Request) -> Response:
            """POST /users - Crear usuario."""
            try:
                name = request.body.get("name")
                email = request.body.get("email")
                
                if not name or not email:
                    return bad_request({"error": "Name and email required"})
                
                user = self.service.create_user(name, email)
                return created(user)
            except ValueError as e:
                return bad_request({"error": str(e)})
        
        def get_user(self, request: Request) -> Response:
            """GET /users/:id - Obtener usuario."""
            user_id = int(request.params.get("id"))
            user = self.service.get_user(user_id)
            
            if user:
                return ok(user)
            return not_found({"error": "User not found"})
        
        def list_users(self, request: Request) -> Response:
            """GET /users - Listar usuarios."""
            users = self.service.list_users()
            return ok({"users": users, "total": len(users)})
        
        def update_user(self, request: Request) -> Response:
            """PUT /users/:id - Actualizar usuario."""
            try:
                user_id = int(request.params.get("id"))
                name = request.body.get("name")
                email = request.body.get("email")
                
                user = self.service.update_user(user_id, name, email)
                
                if user:
                    return ok(user)
                return not_found({"error": "User not found"})
            except ValueError as e:
                return bad_request({"error": str(e)})
        
        def delete_user(self, request: Request) -> Response:
            """DELETE /users/:id - Eliminar usuario."""
            user_id = int(request.params.get("id"))
            deleted = self.service.delete_user(user_id)
            
            if deleted:
                return no_content()
            return not_found({"error": "User not found"})
    
    integrated_injector.register(UserController)
    return integrated_injector.get(UserController)


@pytest.fixture
def auth_controller(integrated_injector):
    """
    AuthController completo con DI para tests end-to-end.
    """
    
    @injectable
    class AuthController:
        def __init__(self, auth_service: AuthService):
            self.auth_service = auth_service
        
        def login(self, request: Request) -> Response:
            """POST /auth/login - Login."""
            email = request.body.get("email")
            password = request.body.get("password")
            
            if not email or not password:
                return bad_request({"error": "Email and password required"})
            
            result = self.auth_service.login(email, password)
            
            if result:
                return ok(result)
            return unauthorized({"error": "Invalid credentials"})
        
        def logout(self, request: Request) -> Response:
            """POST /auth/logout - Logout."""
            token = request.headers.get("Authorization", "").replace("Bearer ", "")
            
            if not token:
                return bad_request({"error": "Token required"})
            
            success = self.auth_service.logout(token)
            
            if success:
                return no_content()
            return bad_request({"error": "Invalid token"})
        
        def me(self, request: Request) -> Response:
            """GET /auth/me - Get current user."""
            token = request.headers.get("Authorization", "").replace("Bearer ", "")
            
            if not token:
                return unauthorized({"error": "Token required"})
            
            user = self.auth_service.get_current_user(token)
            
            if user:
                return ok(user)
            return unauthorized({"error": "Invalid token"})
    
    integrated_injector.register(AuthController)
    return integrated_injector.get(AuthController)


# ============================================================================
# TESTS DE USER CRUD END-TO-END
# ============================================================================

class TestUserCRUDFlow:
    """Tests del flow completo de User CRUD."""
    
    def test_complete_user_crud_flow(self, user_controller):
        """
        Test: Flow completo de CRUD.
        
        Escenario:
        1. POST /users - Crear usuario
        2. GET /users/:id - Leer usuario
        3. GET /users - Listar usuarios
        4. PUT /users/:id - Actualizar usuario
        5. DELETE /users/:id - Eliminar usuario
        6. GET /users/:id - Verificar eliminación (404)
        """
        
        # 1. CREATE: POST /users
        create_request = Request(
            method=HttpMethod.POST,
            path="/users",
            body={"name": "Alice", "email": "alice@example.com"}
        )
        
        create_response = user_controller.create_user(create_request)
        
        # Assert: Created
        assert create_response.status == 201
        assert create_response.body["name"] == "Alice"
        assert create_response.body["email"] == "alice@example.com"
        assert "id" in create_response.body
        
        user_id = create_response.body["id"]
        
        # 2. READ: GET /users/:id
        get_request = Request(
            method=HttpMethod.GET,
            path=f"/users/{user_id}",
            params={"id": str(user_id)}
        )
        
        get_response = user_controller.get_user(get_request)
        
        # Assert: OK
        assert get_response.status == 200
        assert get_response.body["id"] == user_id
        assert get_response.body["name"] == "Alice"
        
        # 3. LIST: GET /users
        list_request = Request(
            method=HttpMethod.GET,
            path="/users"
        )
        
        list_response = user_controller.list_users(list_request)
        
        # Assert: OK, contiene el usuario
        assert list_response.status == 200
        assert list_response.body["total"] >= 1
        assert any(u["id"] == user_id for u in list_response.body["users"])
        
        # 4. UPDATE: PUT /users/:id
        update_request = Request(
            method=HttpMethod.PUT,
            path=f"/users/{user_id}",
            params={"id": str(user_id)},
            body={"name": "Bob"}
        )
        
        update_response = user_controller.update_user(update_request)
        
        # Assert: OK, nombre actualizado
        assert update_response.status == 200
        assert update_response.body["name"] == "Bob"
        
        # 5. DELETE: DELETE /users/:id
        delete_request = Request(
            method=HttpMethod.DELETE,
            path=f"/users/{user_id}",
            params={"id": str(user_id)}
        )
        
        delete_response = user_controller.delete_user(delete_request)
        
        # Assert: No Content
        assert delete_response.status == 204
        
        # 6. VERIFY DELETE: GET /users/:id
        verify_request = Request(
            method=HttpMethod.GET,
            path=f"/users/{user_id}",
            params={"id": str(user_id)}
        )
        
        verify_response = user_controller.get_user(verify_request)
        
        # Assert: Not Found
        assert verify_response.status == 404
    
    def test_create_user_validation_errors(self, user_controller):
        """Test: Validación de errores en creación."""
        
        # Test: Missing name
        request1 = Request(
            method=HttpMethod.POST,
            path="/users",
            body={"email": "test@example.com"}
        )
        
        response1 = user_controller.create_user(request1)
        assert response1.status == 400
        assert "error" in response1.body
        
        # Test: Missing email
        request2 = Request(
            method=HttpMethod.POST,
            path="/users",
            body={"name": "Test"}
        )
        
        response2 = user_controller.create_user(request2)
        assert response2.status == 400
        
        # Test: Invalid email format
        request3 = Request(
            method=HttpMethod.POST,
            path="/users",
            body={"name": "Test", "email": "invalid-email"}
        )
        
        response3 = user_controller.create_user(request3)
        assert response3.status == 400


# ============================================================================
# TESTS DE AUTH FLOW END-TO-END
# ============================================================================

class TestAuthFlow:
    """Tests del flow completo de autenticación."""
    
    def test_complete_auth_flow(self, user_controller, auth_controller):
        """
        Test: Flow completo de autenticación.
        
        Escenario:
        1. Crear usuario
        2. Login con credenciales
        3. GET /auth/me con token
        4. Logout
        5. GET /auth/me con token inválido (debe fallar)
        """
        
        # 1. Crear usuario
        create_request = Request(
            method=HttpMethod.POST,
            path="/users",
            body={"name": "Alice", "email": "alice@example.com"}
        )
        
        create_response = user_controller.create_user(create_request)
        assert create_response.status == 201
        
        # 2. Login
        login_request = Request(
            method=HttpMethod.POST,
            path="/auth/login",
            body={"email": "alice@example.com", "password": "secret123"}
        )
        
        login_response = auth_controller.login(login_request)
        
        # Assert: Login successful
        assert login_response.status == 200
        assert "token" in login_response.body
        assert "user" in login_response.body
        
        token = login_response.body["token"]
        
        # 3. GET /auth/me con token
        me_request = Request(
            method=HttpMethod.GET,
            path="/auth/me",
            headers={"Authorization": f"Bearer {token}"}
        )
        
        me_response = auth_controller.me(me_request)
        
        # Assert: User info returned
        assert me_response.status == 200
        assert me_response.body["email"] == "alice@example.com"
        
        # 4. Logout
        logout_request = Request(
            method=HttpMethod.POST,
            path="/auth/logout",
            headers={"Authorization": f"Bearer {token}"}
        )
        
        logout_response = auth_controller.logout(logout_request)
        
        # Assert: Logout successful
        assert logout_response.status == 204
        
        # 5. GET /auth/me con token inválido
        invalid_me_request = Request(
            method=HttpMethod.GET,
            path="/auth/me",
            headers={"Authorization": f"Bearer {token}"}
        )
        
        invalid_me_response = auth_controller.me(invalid_me_request)
        
        # Assert: Unauthorized (token ya no válido)
        assert invalid_me_response.status == 401
    
    def test_login_invalid_credentials(self, auth_controller):
        """Test: Login con credenciales inválidas."""
        
        request = Request(
            method=HttpMethod.POST,
            path="/auth/login",
            body={"email": "nonexistent@example.com", "password": "wrong"}
        )
        
        response = auth_controller.login(request)
        
        # Assert: Unauthorized
        assert response.status == 401
        assert "error" in response.body


# ============================================================================
# TESTS DE MIDDLEWARE CHAIN
# ============================================================================

class TestMiddlewareChain:
    """Tests de middleware chain execution."""
    
    def test_middleware_executes_in_order(self, integrated_injector):
        """Test: Middleware se ejecuta en orden."""
        
        execution_order = []
        
        class LoggerMiddleware:
            def process(self, request: Request) -> Request:
                execution_order.append("logger")
                return request
        
        class AuthMiddleware:
            def process(self, request: Request) -> Request:
                execution_order.append("auth")
                return request
        
        class CorsMiddleware:
            def process(self, request: Request) -> Request:
                execution_order.append("cors")
                return request
        
        # Simular middleware chain
        request = Request(method=HttpMethod.GET, path="/test")
        
        # Act: Ejecutar middleware en orden
        logger = LoggerMiddleware()
        auth = AuthMiddleware()
        cors = CorsMiddleware()
        
        request = logger.process(request)
        request = auth.process(request)
        request = cors.process(request)
        
        # Assert: Orden correcto
        assert execution_order == ["logger", "auth", "cors"]
    
    def test_middleware_can_short_circuit(self):
        """Test: Middleware puede short-circuit (abortar pipeline)."""
        
        class AuthMiddleware:
            def process(self, request: Request) -> Response | Request:
                token = request.headers.get("Authorization", "")
                
                if not token:
                    # Short-circuit: retornar Response directamente
                    return unauthorized({"error": "Missing token"})
                
                return request
        
        # Request sin token
        request = Request(
            method=HttpMethod.GET,
            path="/protected",
            headers={}
        )
        
        auth = AuthMiddleware()
        
        # Act
        result = auth.process(request)
        
        # Assert: Short-circuit occurred
        assert isinstance(result, Response)
        assert result.status == 401


# ============================================================================
# TESTS DE ERROR HANDLING END-TO-END
# ============================================================================

class TestErrorHandling:
    """Tests de error handling end-to-end."""
    
    def test_404_not_found_user(self, user_controller):
        """Test: 404 cuando usuario no existe."""
        
        request = Request(
            method=HttpMethod.GET,
            path="/users/999999",
            params={"id": "999999"}
        )
        
        response = user_controller.get_user(request)
        
        # Assert
        assert response.status == 404
        assert "error" in response.body
    
    def test_400_bad_request_validation(self, user_controller):
        """Test: 400 cuando datos inválidos."""
        
        request = Request(
            method=HttpMethod.POST,
            path="/users",
            body={"name": "Test"}  # Missing email
        )
        
        response = user_controller.create_user(request)
        
        # Assert
        assert response.status == 400
        assert "error" in response.body
    
    def test_401_unauthorized_missing_token(self, auth_controller):
        """Test: 401 cuando falta token."""
        
        request = Request(
            method=HttpMethod.GET,
            path="/auth/me",
            headers={}
        )
        
        response = auth_controller.me(request)
        
        # Assert
        assert response.status == 401
        assert "error" in response.body


# ============================================================================
# TESTS DE PERFORMANCE
# ============================================================================

class TestPerformance:
    """Tests de performance y benchmarks."""
    
    def test_user_creation_performance(self, user_controller):
        """Test: Performance de creación de usuarios."""
        
        # Crear 100 usuarios
        start_time = time.time()
        
        for i in range(100):
            request = Request(
                method=HttpMethod.POST,
                path="/users",
                body={"name": f"User{i}", "email": f"user{i}@example.com"}
            )
            
            response = user_controller.create_user(request)
            assert response.status == 201
        
        elapsed_time = time.time() - start_time
        
        # Assert: < 1s para 100 usuarios
        assert elapsed_time < 1.0, f"Too slow: {elapsed_time:.3f}s"
        
        # Log performance
        requests_per_second = 100 / elapsed_time
        print(f"\nPerformance: {requests_per_second:.2f} requests/sec")
    
    def test_di_resolution_performance(self, integrated_injector):
        """Test: Performance de DI resolution."""
        
        @injectable
        class TestService:
            def __init__(self, db: DatabaseConnection):
                self.db = db
        
        integrated_injector.register(TestService)
        
        # Resolver 1000 veces
        start_time = time.time()
        
        for _ in range(1000):
            service = integrated_injector.get(TestService)
            assert service is not None
        
        elapsed_time = time.time() - start_time
        
        # Assert: < 0.5s para 1000 resolutions
        assert elapsed_time < 0.5, f"Too slow: {elapsed_time:.3f}s"
        
        # Log performance
        resolutions_per_second = 1000 / elapsed_time
        print(f"\nDI Resolution: {resolutions_per_second:.2f} resolutions/sec")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
