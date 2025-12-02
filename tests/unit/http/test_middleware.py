"""
Tests for HTTP Middleware Pipeline

Test Suite: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02
"""

import pytest
from src.runtime.http.middleware import (
    Middleware,
    MiddlewareChain,
    LoggingMiddleware,
    AuthMiddleware,
    CorsMiddleware,
    ErrorHandlerMiddleware
)
from src.runtime.http.request import Request, HttpMethod
from src.runtime.http.response import Response, ok, unauthorized


class TestMiddlewareChain:
    """Test suite for MiddlewareChain."""
    
    def test_empty_chain(self):
        """Test chain with no middleware (only handler)."""
        def handler(req: Request) -> Response:
            return ok({"message": "success"})
        
        chain = MiddlewareChain([], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        assert response.status == 200
        assert response.body == {"message": "success"}
    
    def test_single_middleware(self):
        """Test chain with single middleware."""
        class CounterMiddleware:
            def __init__(self):
                self.count = 0
            
            def handle(self, request, next):
                self.count += 1
                return next(request)
        
        counter = CounterMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({"count": counter.count})
        
        chain = MiddlewareChain([counter], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        assert counter.count == 1
    
    def test_multiple_middleware_order(self):
        """Test execution order: [M1, M2, Handler, M2, M1]."""
        order = []
        
        class TrackingMiddleware:
            def __init__(self, name):
                self.name = name
            
            def handle(self, request, next):
                order.append(f"{self.name}_before")
                response = next(request)
                order.append(f"{self.name}_after")
                return response
        
        m1 = TrackingMiddleware("M1")
        m2 = TrackingMiddleware("M2")
        
        def handler(req: Request) -> Response:
            order.append("HANDLER")
            return ok({})
        
        chain = MiddlewareChain([m1, m2], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        chain.execute(request)
        
        # Expected: M1_before, M2_before, HANDLER, M2_after, M1_after
        assert order == [
            "M1_before",
            "M2_before",
            "HANDLER",
            "M2_after",
            "M1_after"
        ]
    
    def test_middleware_short_circuit(self):
        """Test middleware that doesn't call next (short-circuit)."""
        class ShortCircuitMiddleware:
            def handle(self, request, next):
                # Don't call next(), return immediately
                return unauthorized("Access denied")
        
        def handler(req: Request) -> Response:
            # Should never be called
            return ok({"message": "should not reach here"})
        
        chain = MiddlewareChain([ShortCircuitMiddleware()], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        
        assert response.status == 401
        assert "Access denied" in response.body["error"]
    
    def test_middleware_modifies_request(self):
        """Test middleware modifying request."""
        class AddHeaderMiddleware:
            def handle(self, request, next):
                request.headers["X-Modified"] = "true"
                return next(request)
        
        def handler(req: Request) -> Response:
            has_header = req.get_header("X-Modified") == "true"
            return ok({"modified": has_header})
        
        chain = MiddlewareChain([AddHeaderMiddleware()], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        assert response.body["modified"] is True
    
    def test_middleware_modifies_response(self):
        """Test middleware modifying response."""
        class AddResponseHeaderMiddleware:
            def handle(self, request, next):
                response = next(request)
                response.set_header("X-Response-Modified", "true")
                return response
        
        def handler(req: Request) -> Response:
            return ok({"message": "success"})
        
        chain = MiddlewareChain([AddResponseHeaderMiddleware()], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        assert response.headers["X-Response-Modified"] == "true"
    
    def test_middleware_error_propagation(self):
        """Test error propagation through middleware."""
        class ErrorMiddleware:
            def handle(self, request, next):
                raise RuntimeError("Middleware error")
        
        def handler(req: Request) -> Response:
            return ok({})
        
        chain = MiddlewareChain([ErrorMiddleware()], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        with pytest.raises(RuntimeError, match="Middleware error"):
            chain.execute(request)


class TestLoggingMiddleware:
    """Test suite for LoggingMiddleware."""
    
    def test_logging_middleware_executes(self):
        """Test that logging middleware executes without errors."""
        middleware = LoggingMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({"message": "success"})
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        assert response.status == 200
    
    def test_logging_middleware_calls_next(self):
        """Test that logging middleware calls next."""
        middleware = LoggingMiddleware()
        handler_called = False
        
        def handler(req: Request) -> Response:
            nonlocal handler_called
            handler_called = True
            return ok({})
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        chain.execute(request)
        assert handler_called is True


class TestAuthMiddleware:
    """Test suite for AuthMiddleware."""
    
    def test_auth_with_valid_token(self):
        """Test auth middleware with valid token."""
        middleware = AuthMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({"authenticated": True})
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(
            method=HttpMethod.GET,
            path="/test",
            headers={"Authorization": "Bearer valid-token"}
        )
        
        response = chain.execute(request)
        assert response.status == 200
    
    def test_auth_without_token(self):
        """Test auth middleware without token (should return 401)."""
        middleware = AuthMiddleware()
        
        def handler(req: Request) -> Response:
            # Should not be called
            return ok({"authenticated": True})
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        assert response.status == 401
        assert "Missing authorization header" in response.body["error"]
    
    def test_auth_with_invalid_token_format(self):
        """Test auth middleware with invalid token format."""
        middleware = AuthMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({})
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(
            method=HttpMethod.GET,
            path="/test",
            headers={"Authorization": "InvalidFormat"}
        )
        
        response = chain.execute(request)
        assert response.status == 401
        assert "Invalid authorization format" in response.body["error"]


class TestCorsMiddleware:
    """Test suite for CorsMiddleware."""
    
    def test_cors_adds_headers(self):
        """Test that CORS middleware adds headers."""
        middleware = CorsMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({})
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        
        assert "Access-Control-Allow-Origin" in response.headers
        assert "Access-Control-Allow-Methods" in response.headers
        assert "Access-Control-Allow-Headers" in response.headers
    
    def test_cors_default_values(self):
        """Test CORS default header values."""
        middleware = CorsMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({})
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        
        assert response.headers["Access-Control-Allow-Origin"] == "*"
        assert "GET" in response.headers["Access-Control-Allow-Methods"]
        assert "POST" in response.headers["Access-Control-Allow-Methods"]


class TestErrorHandlerMiddleware:
    """Test suite for ErrorHandlerMiddleware."""
    
    def test_error_handler_catches_exception(self):
        """Test that error handler catches exceptions."""
        middleware = ErrorHandlerMiddleware()
        
        def handler(req: Request) -> Response:
            raise ValueError("Test error")
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        
        assert response.status == 500
        assert "error" in response.body
    
    def test_error_handler_passes_success(self):
        """Test that error handler passes successful responses."""
        middleware = ErrorHandlerMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({"message": "success"})
        
        chain = MiddlewareChain([middleware], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        
        assert response.status == 200
        assert response.body == {"message": "success"}


class TestMiddlewareComposition:
    """Test suite for multiple middleware composition."""
    
    def test_combined_middleware(self):
        """Test combining multiple middleware."""
        logging_mw = LoggingMiddleware()
        cors_mw = CorsMiddleware()
        error_mw = ErrorHandlerMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({"message": "success"})
        
        chain = MiddlewareChain([logging_mw, cors_mw, error_mw], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        
        # Should have CORS headers
        assert "Access-Control-Allow-Origin" in response.headers
        # Should be successful
        assert response.status == 200
    
    def test_auth_then_error_handler(self):
        """Test auth middleware with error handler."""
        auth_mw = AuthMiddleware()
        error_mw = ErrorHandlerMiddleware()
        
        def handler(req: Request) -> Response:
            return ok({})
        
        chain = MiddlewareChain([error_mw, auth_mw], handler)
        request = Request(method=HttpMethod.GET, path="/test")
        
        response = chain.execute(request)
        
        # Auth should fail (no token)
        assert response.status == 401


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
