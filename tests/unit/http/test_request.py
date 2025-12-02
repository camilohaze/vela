"""
Tests for HTTP Request and Query Parsing

Test Suite: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02
"""

import pytest
from src.runtime.http.request import (
    Request,
    HttpMethod,
    parse_query_string
)


class TestRequest:
    """Test suite for Request type."""
    
    def test_request_creation(self):
        """Test request creation with all fields."""
        req = Request(
            method=HttpMethod.GET,
            path="/users/123",
            params={"id": "123"},
            query={"page": "1"},
            headers={"Content-Type": "application/json"},
            body={"test": "data"}
        )
        
        assert req.method == HttpMethod.GET
        assert req.path == "/users/123"
        assert req.params == {"id": "123"}
        assert req.query == {"page": "1"}
        assert req.headers == {"Content-Type": "application/json"}
        assert req.body == {"test": "data"}
    
    def test_get_param(self):
        """Test getting path parameter."""
        req = Request(
            method=HttpMethod.GET,
            path="/users/123",
            params={"id": "123", "name": "alice"}
        )
        
        assert req.get_param("id") == "123"
        assert req.get_param("name") == "alice"
        assert req.get_param("missing") is None
        assert req.get_param("missing", "default") == "default"
    
    def test_get_query(self):
        """Test getting query parameter."""
        req = Request(
            method=HttpMethod.GET,
            path="/users",
            query={"page": "1", "limit": "10"}
        )
        
        assert req.get_query("page") == "1"
        assert req.get_query("limit") == "10"
        assert req.get_query("missing") is None
        assert req.get_query("missing", "default") == "default"
    
    def test_get_header_case_insensitive(self):
        """Test header retrieval is case-insensitive."""
        req = Request(
            method=HttpMethod.GET,
            path="/users",
            headers={"Content-Type": "application/json"}
        )
        
        assert req.get_header("Content-Type") == "application/json"
        assert req.get_header("content-type") == "application/json"
        assert req.get_header("CONTENT-TYPE") == "application/json"
        assert req.get_header("missing") is None
    
    def test_content_type_property(self):
        """Test content_type property."""
        req = Request(
            method=HttpMethod.POST,
            path="/users",
            headers={"Content-Type": "application/json"}
        )
        
        assert req.content_type == "application/json"
    
    def test_authorization_property(self):
        """Test authorization property."""
        req = Request(
            method=HttpMethod.GET,
            path="/users",
            headers={"Authorization": "Bearer token123"}
        )
        
        assert req.authorization == "Bearer token123"
    
    def test_is_json(self):
        """Test is_json() method."""
        req_json = Request(
            method=HttpMethod.POST,
            path="/users",
            headers={"Content-Type": "application/json"}
        )
        assert req_json.is_json() is True
        
        req_form = Request(
            method=HttpMethod.POST,
            path="/users",
            headers={"Content-Type": "application/x-www-form-urlencoded"}
        )
        assert req_form.is_json() is False
        
        req_no_header = Request(
            method=HttpMethod.POST,
            path="/users"
        )
        assert req_no_header.is_json() is False
    
    def test_is_form(self):
        """Test is_form() method."""
        req_form = Request(
            method=HttpMethod.POST,
            path="/users",
            headers={"Content-Type": "application/x-www-form-urlencoded"}
        )
        assert req_form.is_form() is True
        
        req_json = Request(
            method=HttpMethod.POST,
            path="/users",
            headers={"Content-Type": "application/json"}
        )
        assert req_json.is_form() is False


class TestParseQueryString:
    """Test suite for query string parsing."""
    
    def test_empty_query_string(self):
        """Test parsing empty query string."""
        assert parse_query_string("") == {}
        assert parse_query_string(None) == {}
    
    def test_single_value(self):
        """Test parsing single key=value."""
        result = parse_query_string("page=1")
        assert result == {"page": "1"}
    
    def test_multiple_values(self):
        """Test parsing multiple key=value pairs."""
        result = parse_query_string("page=1&limit=10&sort=asc")
        assert result == {"page": "1", "limit": "10", "sort": "asc"}
    
    def test_multiple_same_key(self):
        """Test parsing multiple values for same key → list."""
        result = parse_query_string("tags=python&tags=rust&tags=go")
        assert result == {"tags": ["python", "rust", "go"]}
    
    def test_empty_value(self):
        """Test parsing key with empty value."""
        result = parse_query_string("key=")
        assert result == {"key": ""}
    
    def test_no_value(self):
        """Test parsing key without value."""
        result = parse_query_string("key")
        assert result == {"key": None}
    
    def test_space_encoding(self):
        """Test URL-encoded spaces (+ → space)."""
        result = parse_query_string("name=John+Doe")
        assert result == {"name": "John Doe"}
    
    def test_mixed_single_and_multiple(self):
        """Test mix of single and multiple values."""
        result = parse_query_string("page=1&tags=a&tags=b&limit=10")
        assert result == {
            "page": "1",
            "tags": ["a", "b"],
            "limit": "10"
        }
    
    def test_complex_query(self):
        """Test complex query string."""
        result = parse_query_string("q=search+term&page=2&tags=python&tags=django&sort=date&order=desc")
        assert result == {
            "q": "search term",
            "page": "2",
            "tags": ["python", "django"],
            "sort": "date",
            "order": "desc"
        }


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
