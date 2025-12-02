"""
Tests for @mock decorator.

Jira: TASK-035I
Historia: VELA-575
"""

import pytest
from src.runtime.di.testing.mock import (
    mock, is_mock, get_mock_target, get_mock_name, create_mock
)


# Test fixtures

class UserRepository:
    """Target class for mocking."""
    
    def find_by_id(self, user_id: int):
        return {"id": user_id, "name": "Real User"}


class TestMockDecorator:
    """Test suite for @mock decorator."""
    
    def test_mock_basic(self):
        """Test basic @mock decorator."""
        
        @mock(UserRepository)
        class MockUserRepository:
            def find_by_id(self, user_id: int):
                return {"id": user_id, "name": "Mock User"}
        
        # Check metadata
        assert hasattr(MockUserRepository, '__mock_target__')
        assert hasattr(MockUserRepository, '__mock_name__')
        assert hasattr(MockUserRepository, '__is_mock__')
        
        assert MockUserRepository.__mock_target__ == UserRepository
        assert MockUserRepository.__mock_name__ == 'UserRepository'
        assert MockUserRepository.__is_mock__ is True
    
    def test_mock_with_custom_name(self):
        """Test @mock with custom name."""
        
        @mock(UserRepository, name='TestRepo')
        class CustomMock:
            pass
        
        assert get_mock_name(CustomMock) == 'TestRepo'
    
    def test_mock_without_target(self):
        """Test @mock without explicit target."""
        
        @mock
        class StandaloneMock:
            pass
        
        assert is_mock(StandaloneMock)
        assert get_mock_target(StandaloneMock) is None
        assert get_mock_name(StandaloneMock) == 'StandaloneMock'
    
    def test_mock_instance(self):
        """Test mock instance."""
        
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        instance = MockUserRepository()
        
        assert is_mock(instance)
        assert get_mock_target(instance) == UserRepository
    
    def test_mock_repr(self):
        """Test mock repr."""
        
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        instance = MockUserRepository()
        repr_str = repr(instance)
        
        assert 'Mock' in repr_str
        assert 'UserRepository' in repr_str
    
    def test_mock_is_mock_instance_method(self):
        """Test is_mock_instance() method."""
        
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        instance = MockUserRepository()
        
        assert hasattr(instance, 'is_mock_instance')
        assert instance.is_mock_instance() is True


class TestIsMock:
    """Test suite for is_mock() function."""
    
    def test_is_mock_class(self):
        """Test is_mock() on class."""
        
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        assert is_mock(MockUserRepository) is True
        assert is_mock(UserRepository) is False
    
    def test_is_mock_instance(self):
        """Test is_mock() on instance."""
        
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        mock_instance = MockUserRepository()
        real_instance = UserRepository()
        
        assert is_mock(mock_instance) is True
        assert is_mock(real_instance) is False


class TestGetMockTarget:
    """Test suite for get_mock_target() function."""
    
    def test_get_mock_target_class(self):
        """Test get_mock_target() on class."""
        
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        assert get_mock_target(MockUserRepository) == UserRepository
    
    def test_get_mock_target_instance(self):
        """Test get_mock_target() on instance."""
        
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        instance = MockUserRepository()
        assert get_mock_target(instance) == UserRepository
    
    def test_get_mock_target_none(self):
        """Test get_mock_target() on non-mock."""
        
        assert get_mock_target(UserRepository) is None
        assert get_mock_target(UserRepository()) is None


class TestGetMockName:
    """Test suite for get_mock_name() function."""
    
    def test_get_mock_name_default(self):
        """Test get_mock_name() with default name."""
        
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        assert get_mock_name(MockUserRepository) == 'UserRepository'
    
    def test_get_mock_name_custom(self):
        """Test get_mock_name() with custom name."""
        
        @mock(UserRepository, name='CustomName')
        class MockUserRepository:
            pass
        
        assert get_mock_name(MockUserRepository) == 'CustomName'
    
    def test_get_mock_name_instance(self):
        """Test get_mock_name() on instance."""
        
        @mock(UserRepository, name='TestRepo')
        class MockUserRepository:
            pass
        
        instance = MockUserRepository()
        assert get_mock_name(instance) == 'TestRepo'


class TestCreateMock:
    """Test suite for create_mock() function."""
    
    def test_create_mock_basic(self):
        """Test basic create_mock()."""
        
        MockRepo = create_mock(
            UserRepository,
            find_by_id=lambda self, user_id: {"id": user_id, "name": "Created Mock"}
        )
        
        # Check it's a mock
        assert is_mock(MockRepo)
        assert get_mock_target(MockRepo) == UserRepository
        
        # Check methods work
        instance = MockRepo()
        result = instance.find_by_id(999)
        assert result == {"id": 999, "name": "Created Mock"}
    
    def test_create_mock_multiple_methods(self):
        """Test create_mock() with multiple methods."""
        
        MockRepo = create_mock(
            UserRepository,
            find_by_id=lambda self, user_id: {"id": user_id},
            save=lambda self, user: True,
            delete=lambda self, user_id: True
        )
        
        instance = MockRepo()
        
        assert hasattr(instance, 'find_by_id')
        assert hasattr(instance, 'save')
        assert hasattr(instance, 'delete')
        
        assert instance.save({"name": "test"}) is True
        assert instance.delete(123) is True
    
    def test_create_mock_class_name(self):
        """Test create_mock() generates correct class name."""
        
        MockRepo = create_mock(UserRepository)
        
        assert MockRepo.__name__ == 'MockUserRepository'


class TestMockWithKwargs:
    """Test suite for @mock with additional kwargs."""
    
    def test_mock_with_custom_kwargs(self):
        """Test @mock with custom keyword arguments."""
        
        @mock(UserRepository, strict=True, version='1.0')
        class StrictMock:
            pass
        
        # Check custom metadata
        assert hasattr(StrictMock, '__mock_strict__')
        assert hasattr(StrictMock, '__mock_version__')
        
        assert StrictMock.__mock_strict__ is True
        assert StrictMock.__mock_version__ == '1.0'


class TestMockIntegration:
    """Integration tests for @mock decorator."""
    
    def test_mock_with_injector(self):
        """Test @mock with TestInjector."""
        from src.runtime.di.testing.test_injector import TestInjector
        
        @mock(UserRepository)
        class MockUserRepository:
            def find_by_id(self, user_id: int):
                return {"id": user_id, "name": "Mocked"}
        
        injector = TestInjector()
        mock_instance = MockUserRepository()
        
        # Override with mock
        injector.override(UserRepository, mock_instance)
        
        # Resolve should return mock
        result = injector.resolve(UserRepository)
        assert result is mock_instance
    
    def test_mock_preserves_interface(self):
        """Test @mock preserves interface."""
        
        @mock(UserRepository)
        class MockUserRepository:
            def find_by_id(self, user_id: int):
                return {"id": user_id, "name": "Mock"}
        
        mock = MockUserRepository()
        
        # Should have same methods as target
        assert hasattr(mock, 'find_by_id')
        assert callable(mock.find_by_id)
        
        # Method should work
        result = mock.find_by_id(123)
        assert result['id'] == 123


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
