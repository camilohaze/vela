"""
Tests for HTTP Router (Radix Tree)

Test Suite: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02
"""

import pytest
from src.runtime.http.router import Router, RouteNode, RouteGroup
from src.runtime.http.request import Request, HttpMethod
from src.runtime.http.response import Response, ok, not_found


def list_users_handler(req: Request) -> Response:
    """Handler for GET /users."""
    return ok({"users": ["Alice", "Bob"]})


def get_user_handler(req: Request) -> Response:
    """Handler for GET /users/:id."""
    user_id = req.get_param('id')
    return ok({"id": user_id, "name": "Alice"})


def create_user_handler(req: Request) -> Response:
    """Handler for POST /users."""
    return ok({"id": 123, "created": True})


def update_user_handler(req: Request) -> Response:
    """Handler for PUT /users/:id."""
    user_id = req.get_param('id')
    return ok({"id": user_id, "updated": True})


def delete_user_handler(req: Request) -> Response:
    """Handler for DELETE /users/:id."""
    user_id = req.get_param('id')
    return ok({"id": user_id, "deleted": True})


class TestRouter:
    """Test suite for Router class."""
    
    def test_router_creation(self):
        """Test router initialization."""
        router = Router()
        
        assert router.root is not None
        assert router.route_count == 0
        assert len(router.global_middleware) == 0
    
    def test_register_static_route(self):
        """Test registering static route."""
        router = Router()
        router.get("/users", list_users_handler)
        
        assert router.route_count == 1
    
    def test_register_param_route(self):
        """Test registering route with parameter."""
        router = Router()
        router.get("/users/:id", get_user_handler)
        
        assert router.route_count == 1
    
    def test_register_multiple_routes(self):
        """Test registering multiple routes."""
        router = Router()
        router.get("/users", list_users_handler)
        router.get("/users/:id", get_user_handler)
        router.post("/users", create_user_handler)
        
        assert router.route_count == 3
    
    def test_register_duplicate_route_raises_error(self):
        """Test that duplicate route registration raises error."""
        router = Router()
        router.get("/users", list_users_handler)
        
        with pytest.raises(ValueError, match="Route already exists"):
            router.get("/users", list_users_handler)
    
    def test_match_static_route(self):
        """Test matching static route."""
        router = Router()
        router.get("/users", list_users_handler)
        
        match = router.match(HttpMethod.GET, "/users")
        
        assert match is not None
        assert match.matched is True
        assert match.params == {}
    
    def test_match_param_route(self):
        """Test matching route with parameter."""
        router = Router()
        router.get("/users/:id", get_user_handler)
        
        match = router.match(HttpMethod.GET, "/users/123")
        
        assert match is not None
        assert match.matched is True
        assert match.params == {"id": "123"}
    
    def test_match_multiple_params(self):
        """Test matching route with multiple parameters."""
        def handler(req: Request) -> Response:
            return ok({
                "postId": req.get_param('postId'),
                "commentId": req.get_param('commentId')
            })
        
        router = Router()
        router.get("/posts/:postId/comments/:commentId", handler)
        
        match = router.match(HttpMethod.GET, "/posts/1/comments/5")
        
        assert match is not None
        assert match.params == {"postId": "1", "commentId": "5"}
    
    def test_match_priority_static_over_param(self):
        """Test that static segments have priority over parameters."""
        def new_handler(req: Request) -> Response:
            return ok({"route": "new"})
        
        router = Router()
        router.get("/users/new", new_handler)
        router.get("/users/:id", get_user_handler)
        
        # Static match should win
        match = router.match(HttpMethod.GET, "/users/new")
        assert match.params == {}  # Static, no params
        
        # Param match
        match2 = router.match(HttpMethod.GET, "/users/123")
        assert match2.params == {"id": "123"}
    
    def test_match_wildcard(self):
        """Test matching wildcard route."""
        def files_handler(req: Request) -> Response:
            return ok({"path": req.get_param('path')})
        
        router = Router()
        router.get("/files/*path", files_handler)
        
        match = router.match(HttpMethod.GET, "/files/images/photo.jpg")
        
        assert match is not None
        assert match.params == {"path": "images/photo.jpg"}
    
    def test_match_not_found(self):
        """Test matching non-existent route."""
        router = Router()
        router.get("/users", list_users_handler)
        
        match = router.match(HttpMethod.GET, "/posts")
        
        assert match is None
    
    def test_match_wrong_method(self):
        """Test matching wrong HTTP method."""
        router = Router()
        router.get("/users", list_users_handler)
        
        match = router.match(HttpMethod.POST, "/users")
        
        assert match is None
    
    def test_handle_success(self):
        """Test handling successful request."""
        router = Router()
        router.get("/users", list_users_handler)
        
        response = router.handle(HttpMethod.GET, "/users")
        
        assert response.status == 200
        assert response.body == {"users": ["Alice", "Bob"]}
    
    def test_handle_with_path_params(self):
        """Test handling request with path parameters."""
        router = Router()
        router.get("/users/:id", get_user_handler)
        
        response = router.handle(HttpMethod.GET, "/users/123")
        
        assert response.status == 200
        assert response.body == {"id": "123", "name": "Alice"}
    
    def test_handle_with_query_params(self):
        """Test handling request with query parameters."""
        def handler(req: Request) -> Response:
            page = req.get_query('page')
            limit = req.get_query('limit')
            return ok({"page": page, "limit": limit})
        
        router = Router()
        router.get("/users", handler)
        
        response = router.handle(HttpMethod.GET, "/users", "page=1&limit=10")
        
        assert response.status == 200
        assert response.body == {"page": "1", "limit": "10"}
    
    def test_handle_not_found(self):
        """Test handling non-existent route."""
        router = Router()
        router.get("/users", list_users_handler)
        
        response = router.handle(HttpMethod.GET, "/posts")
        
        assert response.status == 404
        assert "not found" in response.body["error"].lower()
    
    def test_handle_all_http_methods(self):
        """Test handling all HTTP methods."""
        router = Router()
        router.get("/users", list_users_handler)
        router.post("/users", create_user_handler)
        router.put("/users/:id", update_user_handler)
        router.delete("/users/:id", delete_user_handler)
        
        resp_get = router.handle(HttpMethod.GET, "/users")
        assert resp_get.status == 200
        
        resp_post = router.handle(HttpMethod.POST, "/users")
        assert resp_post.status == 200
        
        resp_put = router.handle(HttpMethod.PUT, "/users/123")
        assert resp_put.status == 200
        
        resp_delete = router.handle(HttpMethod.DELETE, "/users/123")
        assert resp_delete.status == 200


class TestRouteGroup:
    """Test suite for RouteGroup."""
    
    def test_group_creation(self):
        """Test route group creation."""
        router = Router()
        group = router.group("/api/v1")
        
        assert group.prefix == "/api/v1"
        assert group.router == router
    
    def test_group_registration(self):
        """Test registering routes in group."""
        router = Router()
        api_v1 = router.group("/api/v1")
        
        api_v1.get("/users", list_users_handler)
        
        # Should be registered as /api/v1/users
        match = router.match(HttpMethod.GET, "/api/v1/users")
        assert match is not None
    
    def test_group_with_multiple_routes(self):
        """Test group with multiple routes."""
        router = Router()
        api_v1 = router.group("/api/v1")
        
        api_v1.get("/users", list_users_handler)
        api_v1.get("/users/:id", get_user_handler)
        api_v1.post("/users", create_user_handler)
        
        assert router.route_count == 3
        
        # Test all routes work with prefix
        match1 = router.match(HttpMethod.GET, "/api/v1/users")
        assert match1 is not None
        
        match2 = router.match(HttpMethod.GET, "/api/v1/users/123")
        assert match2 is not None
        assert match2.params == {"id": "123"}
        
        match3 = router.match(HttpMethod.POST, "/api/v1/users")
        assert match3 is not None
    
    def test_group_with_middleware(self):
        """Test group with shared middleware."""
        from src.runtime.http.middleware import LoggingMiddleware
        
        router = Router()
        api_v1 = router.group("/api/v1", [LoggingMiddleware()])
        
        api_v1.get("/users", list_users_handler)
        
        match = router.match(HttpMethod.GET, "/api/v1/users")
        assert match is not None
        assert len(match.route.middleware) == 1


class TestGlobalMiddleware:
    """Test suite for global middleware."""
    
    def test_add_global_middleware(self):
        """Test adding global middleware."""
        from src.runtime.http.middleware import LoggingMiddleware
        
        router = Router()
        router.use(LoggingMiddleware())
        
        assert len(router.global_middleware) == 1
    
    def test_global_middleware_executes(self):
        """Test that global middleware executes."""
        # Custom middleware that modifies request
        class TestMiddleware:
            def handle(self, request, next):
                # Add custom header
                request.headers["X-Test"] = "executed"
                return next(request)
        
        def handler(req: Request) -> Response:
            # Check if middleware executed
            has_header = req.get_header("X-Test") == "executed"
            return ok({"middleware_executed": has_header})
        
        router = Router()
        router.use(TestMiddleware())
        router.get("/test", handler)
        
        response = router.handle(HttpMethod.GET, "/test")
        assert response.body["middleware_executed"] is True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
