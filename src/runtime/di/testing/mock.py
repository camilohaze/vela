"""
@mock Decorator - Marks classes as mock providers.

Features:
- Declarative mock creation
- Interface preservation
- Auto-registration with TestInjector
- Type-safe mocking

Jira: TASK-035I
Historia: VELA-575
"""

from typing import Type, Optional, Any, Callable
from functools import wraps


def mock(
    target: Optional[Type] = None,
    *,
    name: Optional[str] = None,
    **mock_kwargs
) -> Callable:
    """
    Decorator to mark a class as a mock provider.
    
    Usage:
        # Mock a class
        @mock(UserRepository)
        class MockUserRepository:
            def find_by_id(self, user_id: int):
                return User(id=user_id, name="Test User")
        
        # Use in test
        injector.register(MockUserRepository)
        
        # Or with explicit name
        @mock(name='UserRepository')
        class MockUserRepository:
            pass
    
    Args:
        target: Target class being mocked (optional)
        name: Explicit name for the mock (optional)
        **mock_kwargs: Additional metadata for the mock
    
    Returns:
        Decorator function
    
    Metadata Added:
        __mock_target__: Target class being mocked
        __mock_name__: Name of the mock
        __is_mock__: True (marker)
    """
    
    def decorator(cls: Type) -> Type:
        """
        Apply mock decorator to class.
        
        Args:
            cls: Class to decorate
        
        Returns:
            Decorated class with mock metadata
        """
        # Store mock metadata
        cls.__mock_target__ = target
        cls.__mock_name__ = name or (target.__name__ if target else cls.__name__)
        cls.__is_mock__ = True
        
        # Store additional kwargs
        for key, value in mock_kwargs.items():
            setattr(cls, f"__mock_{key}__", value)
        
        # Preserve original class name in repr
        original_repr = cls.__repr__ if hasattr(cls, '__repr__') else None
        
        def mock_repr(self) -> str:
            """Custom repr for mock instances."""
            target_name = cls.__mock_target__.__name__ if cls.__mock_target__ else 'Unknown'
            return f"<Mock({target_name}) instance at {hex(id(self))}>"
        
        # Only override if no custom repr
        if original_repr is None or original_repr == object.__repr__:
            cls.__repr__ = mock_repr
        
        # Add helper method to check if instance is a mock
        def is_mock_instance(self) -> bool:
            """Check if this is a mock instance."""
            return True
        
        cls.is_mock_instance = is_mock_instance
        
        return cls
    
    # Handle both @mock and @mock(Target) syntax
    if target is not None and isinstance(target, type):
        # @mock(UserRepository) - target is the class being mocked
        return decorator
    elif target is not None:
        # @mock on a class directly (no args)
        # In this case, 'target' is actually the decorated class
        cls = target
        target = None  # No explicit target
        return decorator(cls)
    else:
        # @mock() - return decorator
        return decorator


def is_mock(cls_or_instance: Any) -> bool:
    """
    Check if a class or instance is a mock.
    
    Args:
        cls_or_instance: Class or instance to check
    
    Returns:
        True if it's a mock
    
    Example:
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        assert is_mock(MockUserRepository)  # True
        assert is_mock(MockUserRepository())  # True
        assert not is_mock(UserRepository)  # False
    """
    if isinstance(cls_or_instance, type):
        # It's a class
        return getattr(cls_or_instance, '__is_mock__', False)
    else:
        # It's an instance
        return getattr(cls_or_instance.__class__, '__is_mock__', False)


def get_mock_target(cls_or_instance: Any) -> Optional[Type]:
    """
    Get the target class that a mock is mocking.
    
    Args:
        cls_or_instance: Mock class or instance
    
    Returns:
        Target class or None if not a mock
    
    Example:
        @mock(UserRepository)
        class MockUserRepository:
            pass
        
        assert get_mock_target(MockUserRepository) == UserRepository
    """
    if isinstance(cls_or_instance, type):
        # It's a class
        return getattr(cls_or_instance, '__mock_target__', None)
    else:
        # It's an instance
        return getattr(cls_or_instance.__class__, '__mock_target__', None)


def get_mock_name(cls_or_instance: Any) -> Optional[str]:
    """
    Get the name of a mock.
    
    Args:
        cls_or_instance: Mock class or instance
    
    Returns:
        Mock name or None if not a mock
    
    Example:
        @mock(UserRepository, name='TestRepo')
        class MockUserRepository:
            pass
        
        assert get_mock_name(MockUserRepository) == 'TestRepo'
    """
    if isinstance(cls_or_instance, type):
        # It's a class
        return getattr(cls_or_instance, '__mock_name__', None)
    else:
        # It's an instance
        return getattr(cls_or_instance.__class__, '__mock_name__', None)


def create_mock(target: Type, **methods) -> Type:
    """
    Factory function to create a mock class dynamically.
    
    Args:
        target: Target class to mock
        **methods: Methods to add to the mock
    
    Returns:
        Mock class
    
    Example:
        MockUserRepository = create_mock(
            UserRepository,
            find_by_id=lambda self, id: User(id=id, name="Test")
        )
        
        injector.override(UserRepository, MockUserRepository())
    """
    # Create mock class dynamically
    mock_class_name = f"Mock{target.__name__}"
    
    # Build class dict
    class_dict = {'__module__': target.__module__}
    
    # Add methods
    for method_name, method_impl in methods.items():
        class_dict[method_name] = method_impl
    
    # Create class
    mock_cls = type(mock_class_name, (), class_dict)
    
    # Apply @mock decorator
    mock_cls = mock(target)(mock_cls)
    
    return mock_cls


if __name__ == "__main__":
    # Example usage
    
    # Example 1: Basic mock
    print("=== Example 1: Basic Mock ===")
    
    class UserRepository:
        def find_by_id(self, user_id: int):
            return {"id": user_id, "name": "Real User"}
    
    @mock(UserRepository)
    class MockUserRepository:
        def find_by_id(self, user_id: int):
            return {"id": user_id, "name": "Mock User"}
    
    print(f"Is mock? {is_mock(MockUserRepository)}")  # True
    print(f"Target: {get_mock_target(MockUserRepository)}")  # <class 'UserRepository'>
    print(f"Name: {get_mock_name(MockUserRepository)}")  # UserRepository
    
    instance = MockUserRepository()
    print(f"Instance is mock? {is_mock(instance)}")  # True
    print(f"Repr: {instance}")  # <Mock(UserRepository) instance at 0x...>
    
    # Example 2: Mock with custom name
    print("\n=== Example 2: Mock with Custom Name ===")
    
    @mock(UserRepository, name='TestUserRepo')
    class CustomMockRepository:
        pass
    
    print(f"Mock name: {get_mock_name(CustomMockRepository)}")  # TestUserRepo
    
    # Example 3: Create mock dynamically
    print("\n=== Example 3: Dynamic Mock ===")
    
    DynamicMock = create_mock(
        UserRepository,
        find_by_id=lambda self, user_id: {"id": user_id, "name": "Dynamic Mock"}
    )
    
    print(f"Is mock? {is_mock(DynamicMock)}")  # True
    print(f"Target: {get_mock_target(DynamicMock)}")  # <class 'UserRepository'>
    
    dynamic_instance = DynamicMock()
    result = dynamic_instance.find_by_id(999)
    print(f"Result: {result}")  # {"id": 999, "name": "Dynamic Mock"}
    
    # Example 4: Mock without target (standalone)
    print("\n=== Example 4: Standalone Mock ===")
    
    @mock
    class StandaloneMock:
        def do_something(self):
            return "mocked"
    
    print(f"Is mock? {is_mock(StandaloneMock)}")  # True
    print(f"Target: {get_mock_target(StandaloneMock)}")  # None
    print(f"Name: {get_mock_name(StandaloneMock)}")  # StandaloneMock
