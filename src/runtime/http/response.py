"""
HTTP Response Type

Implementación de: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02

Descripción:
Tipo Response para HTTP con:
- Status code
- Headers
- Body (JSON, text, etc.)
- Helper methods para responses comunes
"""

from dataclasses import dataclass, field
from typing import Dict, Any, Optional
import json


@dataclass
class Response:
    """
    HTTP Response con status, headers y body.
    
    Inspirado en:
    - Express.js: res.status(), res.json(), res.send()
    - NestJS: @HttpCode(), @Header()
    - FastAPI: Response object
    """
    
    status: int = 200
    body: Optional[Any] = None
    headers: Dict[str, str] = field(default_factory=dict)
    
    def set_header(self, name: str, value: str) -> 'Response':
        """
        Set response header.
        
        Args:
            name: Header name
            value: Header value
            
        Returns:
            Self (for chaining)
            
        Example:
            Response().set_header('Content-Type', 'application/json')
        """
        self.headers[name] = value
        return self
    
    def set_status(self, status: int) -> 'Response':
        """
        Set status code.
        
        Args:
            status: HTTP status code
            
        Returns:
            Self (for chaining)
            
        Example:
            Response().set_status(404).json({"error": "Not found"})
        """
        self.status = status
        return self
    
    def json(self, data: Any) -> 'Response':
        """
        Set body as JSON and Content-Type header.
        
        Args:
            data: Data to serialize to JSON
            
        Returns:
            Self (for chaining)
            
        Example:
            Response().json({"message": "Success"})
        """
        self.body = data
        self.set_header('Content-Type', 'application/json')
        return self
    
    def text(self, data: str) -> 'Response':
        """
        Set body as plain text.
        
        Args:
            data: Text content
            
        Returns:
            Self (for chaining)
            
        Example:
            Response().text("Hello, world!")
        """
        self.body = data
        self.set_header('Content-Type', 'text/plain')
        return self
    
    def html(self, data: str) -> 'Response':
        """
        Set body as HTML.
        
        Args:
            data: HTML content
            
        Returns:
            Self (for chaining)
            
        Example:
            Response().html("<h1>Hello</h1>")
        """
        self.body = data
        self.set_header('Content-Type', 'text/html')
        return self
    
    def to_dict(self) -> Dict[str, Any]:
        """
        Convert Response to dictionary.
        
        Returns:
            Dictionary representation
        """
        return {
            'status': self.status,
            'headers': self.headers,
            'body': self.body
        }
    
    def serialize_body(self) -> str:
        """
        Serialize body to string.
        
        Returns:
            Serialized body
        """
        if self.body is None:
            return ""
        
        content_type = self.headers.get('Content-Type', '')
        
        if 'application/json' in content_type:
            return json.dumps(self.body)
        else:
            return str(self.body)
    
    def __repr__(self) -> str:
        return f"Response({self.status}, {len(self.headers)} headers)"


# Factory functions para responses comunes
def ok(data: Any = None) -> Response:
    """
    Create 200 OK response.
    
    Args:
        data: Response data
        
    Returns:
        Response with status 200
        
    Example:
        ok({"message": "Success"})
    """
    return Response(status=200).json(data) if data else Response(status=200)


def created(data: Any = None) -> Response:
    """
    Create 201 Created response.
    
    Args:
        data: Created resource
        
    Returns:
        Response with status 201
        
    Example:
        created({"id": 123, "name": "John"})
    """
    return Response(status=201).json(data) if data else Response(status=201)


def no_content() -> Response:
    """
    Create 204 No Content response.
    
    Returns:
        Response with status 204
        
    Example:
        no_content()  # → 204 No Content
    """
    return Response(status=204)


def bad_request(message: str = "Bad Request") -> Response:
    """
    Create 400 Bad Request response.
    
    Args:
        message: Error message
        
    Returns:
        Response with status 400
        
    Example:
        bad_request("Invalid email format")
    """
    return Response(status=400).json({"error": message})


def unauthorized(message: str = "Unauthorized") -> Response:
    """
    Create 401 Unauthorized response.
    
    Args:
        message: Error message
        
    Returns:
        Response with status 401
        
    Example:
        unauthorized("Invalid token")
    """
    return Response(status=401).json({"error": message})


def forbidden(message: str = "Forbidden") -> Response:
    """
    Create 403 Forbidden response.
    
    Args:
        message: Error message
        
    Returns:
        Response with status 403
        
    Example:
        forbidden("Access denied")
    """
    return Response(status=403).json({"error": message})


def not_found(message: str = "Not Found") -> Response:
    """
    Create 404 Not Found response.
    
    Args:
        message: Error message
        
    Returns:
        Response with status 404
        
    Example:
        not_found("User not found")
    """
    return Response(status=404).json({"error": message})


def internal_server_error(message: str = "Internal Server Error") -> Response:
    """
    Create 500 Internal Server Error response.
    
    Args:
        message: Error message
        
    Returns:
        Response with status 500
        
    Example:
        internal_server_error("Database connection failed")
    """
    return Response(status=500).json({"error": message})


if __name__ == "__main__":
    # Test básico
    resp = Response(status=200).json({"message": "Success"})
    print(f"Response: {resp}")
    print(f"Body serialized: {resp.serialize_body()}")
    print(f"Dict: {resp.to_dict()}")
    
    # Test factory functions
    print(f"\nOK: {ok({'data': 123})}")
    print(f"Created: {created({'id': 1})}")
    print(f"Bad Request: {bad_request('Invalid input')}")
    print(f"Not Found: {not_found('User not found')}")
