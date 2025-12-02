"""
pytest Fixtures for DI Testing.

Provides reusable fixtures:
- injector: Function-scoped TestInjector
- test_container: Function-scoped TestContainer
- module_injector: Module-scoped TestInjector

Jira: TASK-035I
Historia: VELA-575
"""

import pytest
from typing import Generator
from .test_injector import TestInjector
from .container import TestContainer, create_test_container


@pytest.fixture
def injector() -> Generator[TestInjector, None, None]:
    """
    Provide a TestInjector that resets after each test.
    
    Scope: function (default)
    
    Usage:
        def test_user_service(injector):
            injector.register(UserService)
            service = injector.resolve(UserService)
            assert service is not None
    
    Yields:
        TestInjector instance
    """
    test_injector = TestInjector()
    yield test_injector
    # Auto-reset after test
    test_injector.reset()


@pytest.fixture
def test_container() -> Generator[TestContainer, None, None]:
    """
    Provide a TestContainer with auto-cleanup.
    
    Scope: function (default)
    
    Usage:
        def test_with_container(test_container):
            test_container.register(UserService)
            service = test_container.resolve(UserService)
            assert service is not None
    
    Yields:
        TestContainer instance
    """
    with create_test_container() as container:
        yield container


@pytest.fixture(scope="module")
def module_injector() -> Generator[TestInjector, None, None]:
    """
    Provide a module-scoped TestInjector.
    
    Scope: module (shared across all tests in the module)
    
    Use for expensive setup that can be shared.
    
    Usage:
        # In conftest.py or test file
        def test_1(module_injector):
            module_injector.register(ExpensiveService)
            # ...
        
        def test_2(module_injector):
            # Same injector as test_1
            service = module_injector.resolve(ExpensiveService)
    
    Yields:
        TestInjector instance
    """
    test_injector = TestInjector()
    yield test_injector
    # Cleanup after all tests in module
    test_injector.reset()


@pytest.fixture(scope="class")
def class_injector() -> Generator[TestInjector, None, None]:
    """
    Provide a class-scoped TestInjector.
    
    Scope: class (shared across all test methods in a test class)
    
    Usage:
        class TestUserService:
            def test_create(self, class_injector):
                class_injector.register(UserService)
                # ...
            
            def test_update(self, class_injector):
                # Same injector as test_create
                service = class_injector.resolve(UserService)
    
    Yields:
        TestInjector instance
    """
    test_injector = TestInjector()
    yield test_injector
    # Cleanup after all tests in class
    test_injector.reset()


@pytest.fixture
def isolated_injector() -> Generator[TestInjector, None, None]:
    """
    Provide a completely isolated TestInjector with no shared state.
    
    Use when you need absolute isolation between tests.
    
    Usage:
        def test_isolated(isolated_injector):
            # Fresh injector with no shared state
            isolated_injector.register(UserService)
    
    Yields:
        Fresh TestInjector instance
    """
    # Create new instance (no shared state)
    test_injector = TestInjector()
    yield test_injector
    # Full cleanup
    test_injector.reset()


# Helper fixtures for common scenarios

@pytest.fixture
def mock_repository(injector):
    """
    Example fixture providing a mock repository.
    
    Users can create similar fixtures in their conftest.py.
    
    Usage:
        def test_with_mock(mock_repository, injector):
            service = injector.resolve(UserService)
            # service.repo is mock_repository
    """
    from .mock import mock
    
    @mock
    class MockUserRepository:
        def find_by_id(self, user_id: int):
            return {"id": user_id, "name": "Mock User"}
    
    mock_instance = MockUserRepository()
    # Override in injector
    injector.override('UserRepository', mock_instance)
    
    return mock_instance


if __name__ == "__main__":
    # Example: How to use fixtures in tests
    
    print("=== pytest Fixtures Usage Examples ===\n")
    
    print("""
# conftest.py (project root)
import pytest
from vela.runtime.di.testing import TestInjector, create_test_container

@pytest.fixture
def injector():
    test_injector = TestInjector()
    yield test_injector
    test_injector.reset()

@pytest.fixture
def test_container():
    with create_test_container() as container:
        yield container

# test_user_service.py
def test_user_service_creation(injector):
    '''Test with function-scoped injector.'''
    injector.register(UserService)
    service = injector.resolve(UserService)
    assert service is not None

def test_with_container(test_container):
    '''Test with TestContainer.'''
    test_container.register(UserService)
    test_container.register(UserRepository)
    
    service = test_container.resolve(UserService)
    assert service is not None

def test_with_override(injector):
    '''Test with dependency override.'''
    # Mock repository
    mock_repo = MockUserRepository()
    injector.override(UserRepository, mock_repo)
    
    # Register service
    injector.register(UserService)
    service = injector.resolve(UserService)
    
    # Service uses mock
    assert service.repo is mock_repo

class TestUserServiceClass:
    '''Test class using class-scoped injector.'''
    
    def test_create(self, class_injector):
        class_injector.register(UserService)
        service = class_injector.resolve(UserService)
        assert service is not None
    
    def test_find(self, class_injector):
        # Same injector as test_create
        service = class_injector.resolve(UserService)
        user = service.find_user(123)
        assert user is not None

# test_integration.py
def test_full_integration(test_container):
    '''Integration test with real dependencies.'''
    test_container.register(Database)
    test_container.register(UserRepository)
    test_container.register(UserService)
    
    # Full workflow
    service = test_container.resolve(UserService)
    user = service.create_user("test@example.com")
    assert user.email == "test@example.com"
    
    # Auto-cleanup happens when test ends
""")
    
    print("\n=== Fixture Scopes ===\n")
    print("""
| Fixture          | Scope    | When to Use                              |
|------------------|----------|------------------------------------------|
| injector         | function | Default, fresh for each test             |
| test_container   | function | Isolated container per test              |
| class_injector   | class    | Share state across test class methods    |
| module_injector  | module   | Share expensive setup across module      |
| isolated_injector| function | Absolute isolation (paranoid mode)       |
""")
    
    print("\n=== Best Practices ===\n")
    print("""
1. **Use `injector` for unit tests** (fast, isolated)
2. **Use `test_container` for integration tests** (lifecycle management)
3. **Use `module_injector` sparingly** (only for expensive setup like DB)
4. **Create custom fixtures** in conftest.py for common mocks
5. **Compose fixtures** using pytest's dependency injection
6. **Override at test level** when possible (keeps tests explicit)
""")
