"""
Tests for TestContainer.

Jira: TASK-035I
Historia: VELA-575
"""

import pytest
from src.runtime.di.testing.container import TestContainer, create_test_container
from src.runtime.di.testing.test_injector import TestInjector
from src.runtime.di.injectable import injectable
from src.runtime.di.inject import inject


# Test fixtures

class Database:
    """Test database."""
    
    def __init__(self, connection_string: str = "default"):
        self.connection_string = connection_string
        self.closed = False
    
    def close(self):
        self.closed = True


class UserRepository:
    """Test repository."""
    
    def __init__(self, db: Database = inject(Database)):
        self.db = db
    
    def find_by_id(self, user_id: int):
        return {"id": user_id, "name": "User"}


@injectable
class UserService:
    """Test service."""
    
    def __init__(self, repo: UserRepository = inject(UserRepository)):
        self.repo = repo
    
    def get_user(self, user_id: int):
        return self.repo.find_by_id(user_id)


class TestTestContainerBasic:
    """Test suite for basic TestContainer functionality."""
    
    def test_container_creation(self):
        """Test creating a TestContainer."""
        container = TestContainer()
        
        assert container is not None
        assert isinstance(container.injector, TestInjector)
    
    def test_container_register(self):
        """Test registering providers."""
        container = TestContainer()
        
        # Register
        result = container.register(UserService)
        
        # Should return self for chaining
        assert result is container
    
    def test_container_resolve(self):
        """Test resolving dependencies."""
        container = TestContainer()
        
        # Register
        container.register(Database)
        container.register(UserRepository)
        container.register(UserService)
        
        # Resolve
        service = container.resolve(UserService)
        
        assert service is not None
        assert isinstance(service, UserService)


class TestTestContainerOverride:
    """Test suite for TestContainer.override()."""
    
    def test_override_basic(self):
        """Test basic override."""
        container = TestContainer()
        
        # Mock
        class MockRepository:
            def find_by_id(self, user_id: int):
                return {"id": user_id, "name": "Mock"}
        
        mock = MockRepository()
        
        # Override
        result = container.override(UserRepository, mock)
        
        # Should return self
        assert result is container
        
        # Resolve should return mock
        container.register(UserService)
        service = container.resolve(UserService)
        assert service.repo is mock
    
    def test_override_value(self):
        """Test overriding value."""
        container = TestContainer()
        
        # Override
        container.register_value('api_key', 'test-key')
        
        # Verify
        value = container.resolve('api_key')
        assert value == 'test-key'


class TestTestContainerFactory:
    """Test suite for TestContainer.register_factory()."""
    
    def test_register_factory(self):
        """Test registering factory."""
        container = TestContainer()
        
        # Factory
        def db_factory():
            return Database('factory_db')
        
        # Register
        container.register_factory(Database, db_factory)
        
        # Resolve
        db = container.resolve(Database)
        assert db.connection_string == 'factory_db'


class TestTestContainerLifecycle:
    """Test suite for TestContainer lifecycle hooks."""
    
    def test_on_setup(self):
        """Test setup callback."""
        container = TestContainer()
        
        setup_called = []
        
        def setup():
            setup_called.append(True)
        
        container.on_setup(setup)
        
        # Setup not called yet
        assert len(setup_called) == 0
        
        # Compile to trigger setup
        container.compile()
        
        # Setup should be called
        assert len(setup_called) == 1
    
    def test_on_cleanup(self):
        """Test cleanup callback."""
        container = TestContainer()
        
        cleanup_called = []
        
        def cleanup():
            cleanup_called.append(True)
        
        container.on_cleanup(cleanup)
        
        # Cleanup not called yet
        assert len(cleanup_called) == 0
        
        # Dispose to trigger cleanup
        container.dispose()
        
        # Cleanup should be called
        assert len(cleanup_called) == 1
    
    def test_multiple_setup_callbacks(self):
        """Test multiple setup callbacks."""
        container = TestContainer()
        
        order = []
        
        container.on_setup(lambda: order.append(1))
        container.on_setup(lambda: order.append(2))
        container.on_setup(lambda: order.append(3))
        
        container.compile()
        
        # All callbacks should run in order
        assert order == [1, 2, 3]
    
    def test_cleanup_in_reverse_order(self):
        """Test cleanup callbacks run in reverse order."""
        container = TestContainer()
        
        order = []
        
        container.on_cleanup(lambda: order.append(1))
        container.on_cleanup(lambda: order.append(2))
        container.on_cleanup(lambda: order.append(3))
        
        container.dispose()
        
        # Cleanup should run in reverse
        assert order == [3, 2, 1]
    
    def test_cleanup_continues_on_error(self):
        """Test cleanup continues even if callback fails."""
        container = TestContainer()
        
        cleanup_called = []
        
        def failing_cleanup():
            raise Exception("Cleanup failed")
        
        def successful_cleanup():
            cleanup_called.append(True)
        
        container.on_cleanup(failing_cleanup)
        container.on_cleanup(successful_cleanup)
        
        # Should not raise
        container.dispose()
        
        # Successful cleanup should still run
        assert len(cleanup_called) == 1


class TestTestContainerContextManager:
    """Test suite for TestContainer context manager."""
    
    def test_context_manager_basic(self):
        """Test basic context manager usage."""
        with create_test_container() as container:
            container.register(Database)
            db = container.resolve(Database)
            assert db is not None
    
    def test_context_manager_cleanup(self):
        """Test context manager auto-cleanup."""
        cleanup_called = []
        
        with create_test_container() as container:
            container.on_cleanup(lambda: cleanup_called.append(True))
        
        # Cleanup should have been called
        assert len(cleanup_called) == 1
    
    def test_context_manager_exception(self):
        """Test context manager cleanup on exception."""
        cleanup_called = []
        
        try:
            with create_test_container() as container:
                container.on_cleanup(lambda: cleanup_called.append(True))
                raise ValueError("Test error")
        except ValueError:
            pass
        
        # Cleanup should still run
        assert len(cleanup_called) == 1
    
    def test_manual_context_manager(self):
        """Test manual context manager usage."""
        container = TestContainer()
        cleanup_called = []
        
        container.on_cleanup(lambda: cleanup_called.append(True))
        
        # Manual enter/exit
        container.__enter__()
        container.__exit__(None, None, None)
        
        # Cleanup should run
        assert len(cleanup_called) == 1


class TestTestContainerIsolation:
    """Test suite for TestContainer isolation."""
    
    def test_containers_are_isolated(self):
        """Test containers don't share state."""
        # Container 1
        with create_test_container() as container1:
            container1.register_value('key', 'value1')
            value1 = container1.resolve('key')
        
        # Container 2 (separate)
        with create_test_container() as container2:
            # Should not have 'key' from container1
            with pytest.raises(Exception):
                container2.resolve('key')
    
    def test_dispose_resets_state(self):
        """Test dispose() resets container state."""
        container = TestContainer()
        
        # Register
        container.register_value('key', 'value')
        container.compile()
        
        # Dispose
        container.dispose()
        
        # Should not be able to resolve
        with pytest.raises(Exception):
            container.resolve('key')


class TestTestContainerFluentAPI:
    """Test suite for TestContainer fluent API."""
    
    def test_fluent_chaining(self):
        """Test fluent API chaining."""
        container = TestContainer()
        
        # Chain multiple operations
        result = (container
                  .register(Database)
                  .register(UserRepository)
                  .register(UserService)
                  .on_setup(lambda: None)
                  .on_cleanup(lambda: None))
        
        # Should return self
        assert result is container
        
        # Should work
        service = container.resolve(UserService)
        assert service is not None


class TestTestContainerCompile:
    """Test suite for TestContainer.compile()."""
    
    def test_compile_runs_setup(self):
        """Test compile() runs setup callbacks."""
        container = TestContainer()
        
        setup_called = []
        container.on_setup(lambda: setup_called.append(True))
        
        # Compile
        container.compile()
        
        assert len(setup_called) == 1
    
    def test_compile_idempotent(self):
        """Test compile() can be called multiple times."""
        container = TestContainer()
        
        setup_count = []
        container.on_setup(lambda: setup_count.append(1))
        
        # Multiple compiles
        container.compile()
        container.compile()
        container.compile()
        
        # Setup should only run once
        assert len(setup_count) == 1
    
    def test_auto_compile_on_resolve(self):
        """Test container auto-compiles on first resolve."""
        container = TestContainer()
        
        setup_called = []
        container.on_setup(lambda: setup_called.append(True))
        
        # Resolve (should auto-compile)
        container.register_value('key', 'value')
        container.resolve('key')
        
        # Setup should have run
        assert len(setup_called) == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
