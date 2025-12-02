"""
TestContainer - Isolated test environment with auto-cleanup.

Features:
- Context manager for automatic cleanup
- Isolated DI container per test
- Lifecycle hooks (setup/teardown)
- Resource management

Jira: TASK-035I
Historia: VELA-575
"""

from typing import List, Type, Optional, Any, Callable, Union
from contextlib import contextmanager
from .test_injector import TestInjector, InjectionToken


class TestContainer:
    """
    Isolated test container with auto-cleanup.
    
    Provides:
    - TestInjector instance
    - Automatic resource cleanup
    - Lifecycle hooks
    - Fluent API for configuration
    
    Example:
        with create_test_container() as container:
            container.register(UserService)
            container.register(UserRepository)
            
            service = container.resolve(UserService)
            # ... test ...
        # Auto-cleanup happens here
    """
    
    def __init__(self):
        """Initialize test container."""
        self._injector = TestInjector()
        self._cleanup_callbacks: List[Callable[[], None]] = []
        self._setup_callbacks: List[Callable[[], None]] = []
        self._compiled = False
    
    @property
    def injector(self) -> TestInjector:
        """Get the underlying TestInjector."""
        return self._injector
    
    def register(self, cls: Type, *, scope: Optional[str] = None) -> 'TestContainer':
        """
        Register a provider.
        
        Args:
            cls: Class to register
            scope: Optional scope (SINGLETON, TRANSIENT, SCOPED)
        
        Returns:
            Self for method chaining
        
        Example:
            container.register(UserService).register(UserRepository)
        """
        if scope:
            self._injector.register(cls, scope=scope)
        else:
            self._injector.register(cls)
        return self
    
    def register_value(self, token: InjectionToken, value: Any) -> 'TestContainer':
        """
        Register a value provider.
        
        Args:
            token: Injection token
            value: Value to provide
        
        Returns:
            Self for method chaining
        
        Example:
            container.register_value('api_key', 'test-key-123')
        """
        self._injector.register_value(token, value)
        return self
    
    def register_factory(
        self,
        token: InjectionToken,
        factory: Callable,
        *,
        scope: Optional[str] = None
    ) -> 'TestContainer':
        """
        Register a factory provider.
        
        Args:
            token: Injection token
            factory: Factory function
            scope: Optional scope
        
        Returns:
            Self for method chaining
        
        Example:
            container.register_factory(
                Database,
                lambda: Database('test_db')
            )
        """
        if scope:
            self._injector.register_factory(token, factory, scope=scope)
        else:
            self._injector.register_factory(token, factory)
        return self
    
    def override(self, token: InjectionToken, value: Any) -> 'TestContainer':
        """
        Override a provider for testing.
        
        Args:
            token: Token to override
            value: Value to use instead
        
        Returns:
            Self for method chaining
        
        Example:
            container.override(UserRepository, MockUserRepository())
        """
        self._injector.override(token, value)
        return self
    
    def spy(self, token: InjectionToken, instance: Optional[Any] = None) -> 'TestContainer':
        """
        Create a spy for a dependency.
        
        Args:
            token: Token to spy on
            instance: Optional instance to spy on
        
        Returns:
            Self for method chaining
        
        Example:
            spy = container.spy(UserRepository).injector.get_spy(UserRepository)
        """
        self._injector.spy(token, instance)
        return self
    
    def resolve(self, token: InjectionToken) -> Any:
        """
        Resolve a dependency.
        
        Args:
            token: Token to resolve
        
        Returns:
            Resolved instance
        
        Example:
            service = container.resolve(UserService)
        """
        if not self._compiled:
            self.compile()
        
        return self._injector.resolve(token)
    
    def on_setup(self, callback: Callable[[], None]) -> 'TestContainer':
        """
        Register a setup callback (runs before first resolve).
        
        Args:
            callback: Setup function
        
        Returns:
            Self for method chaining
        
        Example:
            container.on_setup(lambda: print("Setting up..."))
        """
        self._setup_callbacks.append(callback)
        return self
    
    def on_cleanup(self, callback: Callable[[], None]) -> 'TestContainer':
        """
        Register a cleanup callback (runs on dispose).
        
        Args:
            callback: Cleanup function
        
        Returns:
            Self for method chaining
        
        Example:
            container.on_cleanup(lambda: db.close())
        """
        self._cleanup_callbacks.append(callback)
        return self
    
    def compile(self) -> 'TestContainer':
        """
        Compile the container (run setup callbacks).
        
        Returns:
            Self for method chaining
        
        Example:
            container.register(UserService).compile()
        """
        if self._compiled:
            return self
        
        # Run setup callbacks
        for callback in self._setup_callbacks:
            callback()
        
        self._compiled = True
        return self
    
    def dispose(self) -> None:
        """
        Dispose the container (run cleanup callbacks).
        
        Example:
            container.dispose()  # Or use context manager
        """
        # Run cleanup callbacks in reverse order
        for callback in reversed(self._cleanup_callbacks):
            try:
                callback()
            except Exception as e:
                # Log but don't stop other cleanups
                print(f"Warning: Cleanup callback failed: {e}")
        
        # Clear cleanup callbacks
        self._cleanup_callbacks.clear()
        
        # Reset injector
        self._injector.reset()
        
        # Mark as not compiled
        self._compiled = False
    
    def __enter__(self) -> 'TestContainer':
        """Enter context manager."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit context manager (auto-cleanup)."""
        self.dispose()


@contextmanager
def create_test_container():
    """
    Factory function to create a test container with context manager.
    
    Yields:
        TestContainer instance
    
    Example:
        with create_test_container() as container:
            container.register(UserService)
            service = container.resolve(UserService)
            # ... test ...
        # Auto-cleanup
    """
    container = TestContainer()
    try:
        yield container
    finally:
        container.dispose()


if __name__ == "__main__":
    # Example usage
    from ..injectable import injectable
    from ..inject import inject
    
    class Database:
        def __init__(self, connection_string: str):
            self.connection_string = connection_string
            print(f"Database connected: {connection_string}")
        
        def close(self):
            print(f"Database closed: {self.connection_string}")
    
    class UserRepository:
        def __init__(self, db: Database = inject(Database)):
            self.db = db
        
        def find_by_id(self, user_id: int):
            return {"id": user_id, "name": "User from DB"}
    
    @injectable
    class UserService:
        def __init__(self, repo: UserRepository = inject(UserRepository)):
            self.repo = repo
        
        def get_user(self, user_id: int):
            return self.repo.find_by_id(user_id)
    
    # Example 1: Basic usage
    print("=== Example 1: Basic Container ===")
    with create_test_container() as container:
        container.register_value(Database, Database('test_db'))
        container.register(UserRepository)
        container.register(UserService)
        
        service = container.resolve(UserService)
        user = service.get_user(123)
        print(f"User: {user}")
    
    print("\n=== Example 2: With Override ===")
    
    class MockUserRepository:
        def find_by_id(self, user_id: int):
            return {"id": user_id, "name": "Mock User"}
    
    with create_test_container() as container:
        container.register(UserService)
        container.override(UserRepository, MockUserRepository())
        
        service = container.resolve(UserService)
        user = service.get_user(456)
        print(f"User: {user}")
    
    # Example 3: With lifecycle hooks
    print("\n=== Example 3: Lifecycle Hooks ===")
    
    db_instance = None
    
    def setup():
        global db_instance
        db_instance = Database('lifecycle_test')
        print("Setup complete")
    
    def cleanup():
        global db_instance
        if db_instance:
            db_instance.close()
        print("Cleanup complete")
    
    with create_test_container() as container:
        container.on_setup(setup)
        container.on_cleanup(cleanup)
        container.register_value(Database, lambda: db_instance)
        
        # Compile to run setup
        container.compile()
    
    # Example 4: Fluent API
    print("\n=== Example 4: Fluent API ===")
    
    with create_test_container() as container:
        (container
            .register_value(Database, Database('fluent_db'))
            .register(UserRepository)
            .register(UserService)
            .on_cleanup(lambda: print("Fluent cleanup")))
        
        service = container.resolve(UserService)
        user = service.get_user(789)
        print(f"User: {user}")
