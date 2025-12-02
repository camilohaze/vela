"""
HTTP Request Type

Implementación de: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02

Descripción:
Tipo Request para HTTP con:
- Path parameters (extraídos del routing)
- Query parameters (parseados desde query string)
- Headers
- Body
- HTTP method
- URL path
"""

from dataclasses import dataclass, field
from typing import Dict, Any, Optional, List
from enum import Enum


class HttpMethod(Enum):
    """HTTP methods soportados."""
    GET = "GET"
    POST = "POST"
    PUT = "PUT"
    DELETE = "DELETE"
    PATCH = "PATCH"
    HEAD = "HEAD"
    OPTIONS = "OPTIONS"


@dataclass
class Request:
    """
    HTTP Request con path params, query params, headers y body.
    
    Inspirado en:
    - Express.js: req.params, req.query, req.body
    - NestJS: @Request() decorator
    - FastAPI: Request object
    """
    
    method: HttpMethod
    path: str
    params: Dict[str, Any] = field(default_factory=dict)  # Path params: /users/:id → {id: 123}
    query: Dict[str, Any] = field(default_factory=dict)   # Query params: ?page=1 → {page: 1}
    headers: Dict[str, str] = field(default_factory=dict)
    body: Optional[Any] = None
    
    def get_param(self, name: str, default: Any = None) -> Any:
        """
        Get path parameter by name.
        
        Args:
            name: Parameter name
            default: Default value if not found
            
        Returns:
            Parameter value or default
            
        Example:
            # URL: /users/123
            request.get_param('id')  # → 123
        """
        return self.params.get(name, default)
    
    def get_query(self, name: str, default: Any = None) -> Any:
        """
        Get query parameter by name.
        
        Args:
            name: Query parameter name
            default: Default value if not found
            
        Returns:
            Query value or default
            
        Example:
            # URL: /users?page=1&limit=10
            request.get_query('page')  # → 1
            request.get_query('sort', 'asc')  # → 'asc' (default)
        """
        return self.query.get(name, default)
    
    def get_header(self, name: str, default: Optional[str] = None) -> Optional[str]:
        """
        Get header by name (case-insensitive).
        
        Args:
            name: Header name
            default: Default value if not found
            
        Returns:
            Header value or default
            
        Example:
            request.get_header('Content-Type')  # → 'application/json'
        """
        # Case-insensitive lookup
        for key, value in self.headers.items():
            if key.lower() == name.lower():
                return value
        return default
    
    @property
    def content_type(self) -> Optional[str]:
        """Get Content-Type header."""
        return self.get_header('Content-Type')
    
    @property
    def authorization(self) -> Optional[str]:
        """Get Authorization header."""
        return self.get_header('Authorization')
    
    def is_json(self) -> bool:
        """Check if request is JSON."""
        ct = self.content_type
        return ct is not None and 'application/json' in ct.lower()
    
    def is_form(self) -> bool:
        """Check if request is form data."""
        ct = self.content_type
        return ct is not None and 'application/x-www-form-urlencoded' in ct.lower()
    
    def __repr__(self) -> str:
        return f"Request({self.method.value} {self.path})"


def parse_query_string(query_string: str) -> Dict[str, Any]:
    """
    Parse query string to dictionary.
    
    Soporta:
    - Single values: ?key=value → {key: value}
    - Multiple values: ?key=a&key=b → {key: [a, b]}
    - Empty values: ?key= → {key: ""}
    - No values: ?key → {key: None}
    
    Args:
        query_string: Query string sin el '?' inicial
        
    Returns:
        Dictionary con query parameters
        
    Example:
        parse_query_string("page=1&limit=10&tags=python&tags=rust")
        # → {page: "1", limit: "10", tags: ["python", "rust"]}
    """
    if not query_string:
        return {}
    
    result: Dict[str, Any] = {}
    
    for pair in query_string.split('&'):
        if '=' in pair:
            key, value = pair.split('=', 1)
        else:
            key, value = pair, None
        
        # URL decode (simple version)
        key = key.replace('+', ' ')
        if value is not None:
            value = value.replace('+', ' ')
        
        # Handle multiple values → List
        if key in result:
            # Convert to list if first duplicate
            if not isinstance(result[key], list):
                result[key] = [result[key]]
            result[key].append(value)
        else:
            result[key] = value
    
    return result


if __name__ == "__main__":
    # Test básico
    req = Request(
        method=HttpMethod.GET,
        path="/users/123",
        params={"id": "123"},
        query={"page": "1", "limit": "10"},
        headers={"Content-Type": "application/json"}
    )
    
    print(f"Request: {req}")
    print(f"ID param: {req.get_param('id')}")
    print(f"Page query: {req.get_query('page')}")
    print(f"Content-Type: {req.content_type}")
    print(f"Is JSON: {req.is_json()}")
    
    # Test query parsing
    query = parse_query_string("page=1&limit=10&tags=python&tags=rust")
    print(f"\nQuery parsed: {query}")
