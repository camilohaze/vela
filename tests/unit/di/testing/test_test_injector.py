"""
Tests for TestInjector class.

Jira: TASK-035I
Historia: VELA-575
"""

import pytest
from src.runtime.di.testing.test_injector import TestInjector, SpyProxy
from src.runtime.di.injectable import injectable
from src.runtime.di.inject import inject


# Test fixtures

class UserRepository:
    """Test repository."""
    
    def find_by_id(self, user_id: int):
        return {"id": user_id, "name": "Real User"}
    
    def save(self, user):
        return True


class MockUserRepository:
    """Mock repository."""
    
    def find_by_id(self, user_id: int):
        return {"id": user_id, "name": "Mock User"}
    
    def save(self, user):
        self.save_called = True
        return True


@injectable
class UserService:
    """Test service."""
    
    def __init__(self, repo: UserRepository = inject(UserRepository)):
        self.repo = repo
    
    def get_user(self, user_id: int):
        return self.repo.find_by_id(user_id)


# TestInjector Tests

class TestTestInjectorOverride:
    """Test suite for TestInjector.override()."""
    
    def test_override_basic(self):
        """Test basic override functionality."""
        injector = TestInjector()
        
        # Register original
        injector.register(UserRepository)
        
        # Override
        mock_repo = MockUserRepository()
        injector.override(UserRepository, mock_repo)
        
        # Resolve should return mock
        result = injector.resolve(UserRepository)
        assert result is mock_repo
    
    def test_override_with_service(self):
        """Test override with dependent service."""
        injector = TestInjector()
        
        # Override repository
        mock_repo = MockUserRepository()
        injector.override(UserRepository, mock_repo)
        
        # Register service (depends on repository)
        injector.register(UserService)
        
        # Service should use mock
        service = injector.resolve(UserService)
        assert service.repo is mock_repo
    
    def test_override_string_token(self):
        """Test override with string token."""
        injector = TestInjector()
        
        # Override string token
        injector.override('api_key', 'test-key-123')
        
        # Verify
        assert injector.is_overridden('api_key')
        result = injector.get_override('api_key')
        assert result == 'test-key-123'
    
    def test_override_multiple_tokens(self):
        """Test overriding multiple tokens."""
        injector = TestInjector()
        
        # Override multiple
        injector.override('token1', 'value1')
        injector.override('token2', 'value2')
        injector.override(UserRepository, MockUserRepository())
        
        # All should be overridden
        assert injector.is_overridden('token1')
        assert injector.is_overridden('token2')
        assert injector.is_overridden(UserRepository)
    
    def test_override_chaining(self):
        """Test method chaining."""
        injector = TestInjector()
        
        # Chain overrides
        result = (injector
                  .override('token1', 'value1')
                  .override('token2', 'value2'))
        
        assert result is injector
        assert injector.is_overridden('token1')
        assert injector.is_overridden('token2')


class TestTestInjectorSpy:
    """Test suite for TestInjector.spy()."""
    
    def test_spy_basic(self):
        """Test basic spy functionality."""
        injector = TestInjector()
        injector.register(UserRepository)
        
        # Create spy
        spy = injector.spy(UserRepository)
        
        assert isinstance(spy, SpyProxy)
    
    def test_spy_tracks_calls(self):
        """Test spy tracks method calls."""
        injector = TestInjector()
        injector.register(UserRepository)
        injector.register(UserService)
        
        # Spy on repository
        spy = injector.spy(UserRepository)
        
        # Use service (which calls repository)
        service = injector.resolve(UserService)
        service.get_user(123)
        
        # Verify spy tracked call
        assert spy.called('find_by_id', args=(123,))
        assert spy.call_count('find_by_id') == 1
    
    def test_spy_tracks_multiple_calls(self):
        """Test spy tracks multiple calls."""
        injector = TestInjector()
        repo = UserRepository()
        
        # Spy on instance
        spy = injector.spy(UserRepository, repo)
        
        # Make multiple calls
        spy.find_by_id(1)
        spy.find_by_id(2)
        spy.save({"name": "test"})
        
        # Verify all tracked
        assert spy.call_count('find_by_id') == 2
        assert spy.call_count('save') == 1
        assert spy.called('find_by_id', args=(1,))
        assert spy.called('find_by_id', args=(2,))
    
    def test_spy_get_calls(self):
        """Test getting call history."""
        injector = TestInjector()
        repo = UserRepository()
        spy = injector.spy(UserRepository, repo)
        
        # Make calls
        spy.find_by_id(123)
        spy.save({"name": "test"})
        
        # Get calls
        all_calls = spy.get_calls()
        assert len(all_calls) == 2
        
        find_calls = spy.get_calls('find_by_id')
        assert len(find_calls) == 1
        assert find_calls[0]['method'] == 'find_by_id'
        assert find_calls[0]['args'] == (123,)
    
    def test_spy_reset_calls(self):
        """Test resetting spy call history."""
        injector = TestInjector()
        repo = UserRepository()
        spy = injector.spy(UserRepository, repo)
        
        # Make call
        spy.find_by_id(123)
        assert spy.call_count('find_by_id') == 1
        
        # Reset
        spy.reset_calls()
        assert spy.call_count('find_by_id') == 0


class TestTestInjectorReset:
    """Test suite for TestInjector.reset()."""
    
    def test_reset_clears_overrides(self):
        """Test reset clears all overrides."""
        injector = TestInjector()
        
        # Override
        injector.override('token1', 'value1')
        injector.override(UserRepository, MockUserRepository())
        
        assert injector.is_overridden('token1')
        assert injector.is_overridden(UserRepository)
        
        # Reset
        injector.reset()
        
        assert not injector.is_overridden('token1')
        assert not injector.is_overridden(UserRepository)
    
    def test_reset_clears_spies(self):
        """Test reset clears all spies."""
        injector = TestInjector()
        injector.register(UserRepository)
        
        # Spy
        spy = injector.spy(UserRepository)
        assert injector.get_spy(UserRepository) is spy
        
        # Reset
        injector.reset()
        
        assert injector.get_spy(UserRepository) is None
    
    def test_reset_clears_providers(self):
        """Test reset clears all providers."""
        injector = TestInjector()
        
        # Register
        injector.register(UserRepository)
        injector.register(UserService)
        
        # Reset
        injector.reset()
        
        # Should not be able to resolve
        with pytest.raises(Exception):
            injector.resolve(UserRepository)


class TestTestInjectorSnapshot:
    """Test suite for TestInjector snapshot/restore."""
    
    def test_snapshot_basic(self):
        """Test basic snapshot functionality."""
        injector = TestInjector()
        
        # Initial state
        injector.register(UserRepository)
        
        # Snapshot
        snapshot_id = injector.snapshot()
        assert isinstance(snapshot_id, int)
        assert snapshot_id == 0
    
    def test_snapshot_restore(self):
        """Test snapshot and restore."""
        injector = TestInjector()
        
        # Initial state
        injector.register(UserRepository)
        injector.override('api_key', 'original')
        
        # Snapshot
        snapshot_id = injector.snapshot()
        
        # Modify state
        injector.override('api_key', 'modified')
        injector.override('new_token', 'new_value')
        
        assert injector.get_override('api_key') == 'modified'
        assert injector.is_overridden('new_token')
        
        # Restore
        injector.restore(snapshot_id)
        
        # Should be back to original
        assert injector.get_override('api_key') == 'original'
        assert not injector.is_overridden('new_token')
    
    def test_snapshot_multiple(self):
        """Test multiple snapshots."""
        injector = TestInjector()
        
        # State 1
        injector.override('key', 'value1')
        snapshot1 = injector.snapshot()
        
        # State 2
        injector.override('key', 'value2')
        snapshot2 = injector.snapshot()
        
        # State 3
        injector.override('key', 'value3')
        
        # Restore to state 2
        injector.restore(snapshot2)
        assert injector.get_override('key') == 'value2'
        
        # Restore to state 1
        injector.restore(snapshot1)
        assert injector.get_override('key') == 'value1'
    
    def test_snapshot_invalid_id(self):
        """Test restoring invalid snapshot."""
        injector = TestInjector()
        
        with pytest.raises(IndexError):
            injector.restore(999)


class TestSpyProxy:
    """Test suite for SpyProxy class."""
    
    def test_spy_proxy_wraps_instance(self):
        """Test SpyProxy wraps target instance."""
        repo = UserRepository()
        spy = SpyProxy(repo)
        
        # Should be able to call methods
        result = spy.find_by_id(123)
        assert result == {"id": 123, "name": "Real User"}
    
    def test_spy_proxy_tracks_kwargs(self):
        """Test SpyProxy tracks keyword arguments."""
        class Service:
            def method(self, a, b=None, c=None):
                return a + (b or 0) + (c or 0)
        
        service = Service()
        spy = SpyProxy(service)
        
        # Call with kwargs
        spy.method(1, b=2, c=3)
        
        # Verify
        assert spy.called('method', args=(1,), kwargs={'b': 2, 'c': 3})
    
    def test_spy_proxy_non_callable_attributes(self):
        """Test SpyProxy handles non-callable attributes."""
        class Service:
            value = 42
            
            def method(self):
                return self.value
        
        service = Service()
        spy = SpyProxy(service)
        
        # Access non-callable attribute
        assert spy.value == 42


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
