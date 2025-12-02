"""
Tests for HTTP Route Matching

Test Suite: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02
"""

import pytest
from src.runtime.http.route import Route, RouteMatch, get, post, put, delete, patch
from src.runtime.http.request import Request, HttpMethod
from src.runtime.http.response import ok


def dummy_handler(req: Request):
    """Dummy handler for testing."""
    return ok({"message": "test"})


class TestRoute:
    """Test suite for Route class."""
    
    def test_route_creation(self):
        """Test route creation."""
        route = Route(HttpMethod.GET, "/users/:id", dummy_handler)
        
        assert route.method == HttpMethod.GET
        assert route.path == "/users/:id"
        assert route.handler == dummy_handler
        assert route.middleware == []
    
    def test_route_with_middleware(self):
        """Test route creation with middleware."""
        from src.runtime.http.middleware import LoggingMiddleware
        
        middleware = [LoggingMiddleware()]
        route = Route(HttpMethod.GET, "/users", dummy_handler, middleware)
        
        assert len(route.middleware) == 1
    
    def test_parse_static_segments(self):
        """Test parsing static path segments."""
        route = Route(HttpMethod.GET, "/users/list", dummy_handler)
        
        assert route.segments == ["users", "list"]
        assert route.param_names == []
    
    def test_parse_param_segments(self):
        """Test parsing path with parameters."""
        route = Route(HttpMethod.GET, "/users/:id", dummy_handler)
        
        assert route.segments == ["users", ":id"]
        assert route.param_names == ["id"]
    
    def test_parse_multiple_params(self):
        """Test parsing path with multiple parameters."""
        route = Route(HttpMethod.GET, "/posts/:postId/comments/:commentId", dummy_handler)
        
        assert route.segments == ["posts", ":postId", "comments", ":commentId"]
        assert route.param_names == ["postId", "commentId"]
    
    def test_parse_wildcard(self):
        """Test parsing wildcard path."""
        route = Route(HttpMethod.GET, "/files/*path", dummy_handler)
        
        assert route.segments == ["files", "*path"]
        assert route.param_names == ["path"]
    
    def test_match_static_path_success(self):
        """Test matching static path successfully."""
        route = Route(HttpMethod.GET, "/users/list", dummy_handler)
        match = route.match(HttpMethod.GET, "/users/list")
        
        assert match.matched is True
        assert match.params == {}
        assert match.route == route
    
    def test_match_static_path_fail_wrong_path(self):
        """Test matching fails with wrong path."""
        route = Route(HttpMethod.GET, "/users/list", dummy_handler)
        match = route.match(HttpMethod.GET, "/posts/list")
        
        assert match.matched is False
    
    def test_match_static_path_fail_wrong_method(self):
        """Test matching fails with wrong HTTP method."""
        route = Route(HttpMethod.GET, "/users", dummy_handler)
        match = route.match(HttpMethod.POST, "/users")
        
        assert match.matched is False
    
    def test_match_param_path_success(self):
        """Test matching path with parameter."""
        route = Route(HttpMethod.GET, "/users/:id", dummy_handler)
        match = route.match(HttpMethod.GET, "/users/123")
        
        assert match.matched is True
        assert match.params == {"id": "123"}
    
    def test_match_param_path_different_values(self):
        """Test matching parameter path with different values."""
        route = Route(HttpMethod.GET, "/users/:id", dummy_handler)
        
        match1 = route.match(HttpMethod.GET, "/users/123")
        assert match1.params == {"id": "123"}
        
        match2 = route.match(HttpMethod.GET, "/users/abc")
        assert match2.params == {"id": "abc"}
        
        match3 = route.match(HttpMethod.GET, "/users/user-456")
        assert match3.params == {"id": "user-456"}
    
    def test_match_multiple_params(self):
        """Test matching path with multiple parameters."""
        route = Route(HttpMethod.GET, "/posts/:postId/comments/:commentId", dummy_handler)
        match = route.match(HttpMethod.GET, "/posts/1/comments/5")
        
        assert match.matched is True
        assert match.params == {"postId": "1", "commentId": "5"}
    
    def test_match_wildcard(self):
        """Test matching wildcard path."""
        route = Route(HttpMethod.GET, "/files/*path", dummy_handler)
        
        match = route.match(HttpMethod.GET, "/files/images/photo.jpg")
        assert match.matched is True
        assert match.params == {"path": "images/photo.jpg"}
    
    def test_match_wildcard_nested(self):
        """Test matching wildcard with deeply nested path."""
        route = Route(HttpMethod.GET, "/files/*path", dummy_handler)
        
        match = route.match(HttpMethod.GET, "/files/docs/2024/report.pdf")
        assert match.matched is True
        assert match.params == {"path": "docs/2024/report.pdf"}
    
    def test_route_equality(self):
        """Test route equality comparison."""
        route1 = Route(HttpMethod.GET, "/users", dummy_handler)
        route2 = Route(HttpMethod.GET, "/users", dummy_handler)
        route3 = Route(HttpMethod.POST, "/users", dummy_handler)
        
        assert route1 == route2
        assert route1 != route3
    
    def test_route_repr(self):
        """Test route string representation."""
        route = Route(HttpMethod.GET, "/users/:id", dummy_handler)
        assert repr(route) == "Route(GET /users/:id)"


class TestRouteHelpers:
    """Test suite for route helper functions."""
    
    def test_get_helper(self):
        """Test get() helper function."""
        route = get("/users", dummy_handler)
        
        assert route.method == HttpMethod.GET
        assert route.path == "/users"
    
    def test_post_helper(self):
        """Test post() helper function."""
        route = post("/users", dummy_handler)
        
        assert route.method == HttpMethod.POST
        assert route.path == "/users"
    
    def test_put_helper(self):
        """Test put() helper function."""
        route = put("/users/:id", dummy_handler)
        
        assert route.method == HttpMethod.PUT
        assert route.path == "/users/:id"
    
    def test_delete_helper(self):
        """Test delete() helper function."""
        route = delete("/users/:id", dummy_handler)
        
        assert route.method == HttpMethod.DELETE
        assert route.path == "/users/:id"
    
    def test_patch_helper(self):
        """Test patch() helper function."""
        route = patch("/users/:id", dummy_handler)
        
        assert route.method == HttpMethod.PATCH
        assert route.path == "/users/:id"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
