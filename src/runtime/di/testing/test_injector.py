"""
TestInjector - Extended Injector with testing capabilities.

Features:
- override(token, value): Replace providers for testing
- spy(token): Track calls to dependencies
- reset(): Clear overrides and restore original state
- snapshot() / restore(): Save and restore injector state

Jira: TASK-035I
Historia: VELA-575
"""

from typing import Type, Any, Optional, Dict, List, Callable, Union
from ..injector import Injector
from ..scopes import Scope

# Type alias for injection tokens
InjectionToken = Union[Type, str]


class SpyProxy:
    """
    Proxy that tracks method calls on a dependency.
    
    Usage:
        spy = injector.spy(UserRepository)
        service = injector.resolve(UserService)
        service.find_user(123)
        
        # Verify calls
        assert spy.called('find_by_id', args=(123,))
    """
    
    def __init__(self, target: Any):
        """
        Initialize spy proxy.
        
        Args:
            target: Object to spy on
        """
        self._target = target
        self._calls: List[Dict[str, Any]] = []
        self._method_proxies: Dict[str, Callable] = {}
    
    def __getattr__(self, name: str) -> Any:
        """Intercept attribute access to track calls."""
        attr = getattr(self._target, name)
        
        # If it's a callable, wrap it
        if callable(attr):
            def method_spy(*args, **kwargs):
                # Record call
                self._calls.append({
                    'method': name,
                    'args': args,
                    'kwargs': kwargs
                })
                
                # Call original
                return attr(*args, **kwargs)
            
            return method_spy
        
        return attr
    
    def called(self, method: str, *, args: tuple = None, kwargs: dict = None) -> bool:
        """
        Check if method was called with specific arguments.
        
        Args:
            method: Method name
            args: Expected positional arguments (optional)
            kwargs: Expected keyword arguments (optional)
        
        Returns:
            True if method was called (with matching args if specified)
        """
        for call in self._calls:
            if call['method'] != method:
                continue
            
            # If no args specified, just check method name
            if args is None and kwargs is None:
                return True
            
            # Check args
            if args is not None and call['args'] != args:
                continue
            
            # Check kwargs
            if kwargs is not None and call['kwargs'] != kwargs:
                continue
            
            return True
        
        return False
    
    def call_count(self, method: str) -> int:
        """
        Get number of times method was called.
        
        Args:
            method: Method name
        
        Returns:
            Number of calls
        """
        return sum(1 for call in self._calls if call['method'] == method)
    
    def get_calls(self, method: Optional[str] = None) -> List[Dict[str, Any]]:
        """
        Get all calls (optionally filtered by method).
        
        Args:
            method: Method name to filter by (optional)
        
        Returns:
            List of call records
        """
        if method is None:
            return self._calls.copy()
        
        return [call for call in self._calls if call['method'] == method]
    
    def reset_calls(self) -> None:
        """Clear call history."""
        self._calls.clear()


class TestInjector:
    """
    Testing wrapper for Injector with additional capabilities.
    
    Adds:
    - override(): Replace providers for testing
    - spy(): Track calls to dependencies
    - reset(): Clear overrides
    - snapshot/restore(): Save/restore state
    
    Example:
        injector = TestInjector()
        
        # Override dependency
        mock_repo = MockUserRepository()
        injector.override(UserRepository, mock_repo)
        
        # Resolve service (uses mock)
        service = injector.resolve(UserService)
        assert service.repo is mock_repo
    """
    
    def __init__(self):
        """Initialize test injector."""
        self._injector = Injector()
        self._overrides: Dict[InjectionToken, Any] = {}
        self._spies: Dict[InjectionToken, SpyProxy] = {}
        self._snapshots: List[Dict[str, Any]] = []
    
    # Delegate core methods to wrapped injector
    
    def register(self, *args, **kwargs):
        """Register provider."""
        return self._injector.register(*args, **kwargs)
    
    def get(self, token: Type, **kwargs):
        """Get dependency (checks overrides first)."""
        # Check if overridden
        normalized = self._normalize_token(token)
        if normalized in self._overrides:
            return self._overrides[normalized]
        return self._injector.get(token, **kwargs)
    
    def resolve(self, token: Type):
        """Resolve dependency (alias for get)."""
        return self.get(token)
    
    def override(self, token: InjectionToken, value: Any) -> 'TestInjector':
        """
        Override a provider for testing.
        
        Args:
            token: Token to override (class or string)
            value: Value to use instead (instance or factory)
        
        Returns:
            Self for method chaining
        
        Example:
            injector.override(UserRepository, MockUserRepository())
            injector.override('api_key', 'test-key-123')
        """
        # Normalize token
        normalized_token = self._normalize_token(token)
        
        # Store override (get() will check this first)
        self._overrides[normalized_token] = value
        
        return self
    
    def spy(self, token: InjectionToken, instance: Optional[Any] = None) -> SpyProxy:
        """
        Create a spy that tracks calls to a dependency.
        
        Args:
            token: Token to spy on
            instance: Specific instance to spy on (optional, will resolve if not provided)
        
        Returns:
            SpyProxy that wraps the dependency
        
        Example:
            spy = injector.spy(UserRepository)
            service = injector.get(UserService)
            service.find_user(123)
            
            assert spy.called('find_by_id', args=(123,))
        """
        # Normalize token
        normalized_token = self._normalize_token(token)
        
        # Get instance to spy on
        if instance is None:
            # Use parent's get() method
            if isinstance(token, str):
                raise ValueError("Cannot spy on string token without providing instance")
            instance = self.get(token)
        
        # Create spy
        spy = SpyProxy(instance)
        self._spies[normalized_token] = spy
        
        # Override with spy
        self.override(token, spy)
        
        return spy
    
    def reset(self) -> None:
        """
        Clear all overrides and restore original state.
        
        Example:
            injector.override(UserRepository, mock_repo)
            # ... tests ...
            injector.reset()  # Back to original state
        """
        # Clear overrides
        self._overrides.clear()
        
        # Clear spies
        self._spies.clear()
        
        # Reset wrapped injector
        self._injector = Injector()
    
    def snapshot(self) -> int:
        """
        Save current state of the injector.
        
        Returns:
            Snapshot ID (index in snapshots list)
        
        Example:
            snapshot_id = injector.snapshot()
            injector.override(UserRepository, mock)
            # ... tests ...
            injector.restore(snapshot_id)  # Back to snapshot state
        """
        import copy
        import pickle
        
        # Serialize injector state (simple approach: pickle)
        try:
            injector_state = pickle.dumps(self._injector._registry)
        except:
            # If pickle fails, just store reference (won't be perfect but better than nothing)
            injector_state = None
        
        snapshot = {
            'injector_state': injector_state,
            'overrides': self._overrides.copy(),
            'spies': self._spies.copy()
        }
        
        self._snapshots.append(snapshot)
        return len(self._snapshots) - 1
    
    def restore(self, snapshot_id: int) -> None:
        """
        Restore injector to a previous snapshot.
        
        Args:
            snapshot_id: ID returned by snapshot()
        
        Raises:
            IndexError: If snapshot_id is invalid
        
        Example:
            snapshot_id = injector.snapshot()
            injector.override(UserRepository, mock)
            injector.restore(snapshot_id)  # Restored
        """
        if snapshot_id < 0 or snapshot_id >= len(self._snapshots):
            raise IndexError(f"Invalid snapshot ID: {snapshot_id}")
        
        import pickle
        snapshot = self._snapshots[snapshot_id]
        
        # Restore state
        if snapshot['injector_state'] is not None:
            try:
                self._injector._registry = pickle.loads(snapshot['injector_state'])
            except:
                # If restore fails, create new injector
                self._injector = Injector()
        
        self._overrides = snapshot['overrides'].copy()
        self._spies = snapshot['spies'].copy()
    
    def get_override(self, token: InjectionToken) -> Optional[Any]:
        """
        Get override value for a token.
        
        Args:
            token: Token to check
        
        Returns:
            Override value or None if not overridden
        """
        token = self._normalize_token(token)
        return self._overrides.get(token)
    
    def is_overridden(self, token: InjectionToken) -> bool:
        """
        Check if a token has been overridden.
        
        Args:
            token: Token to check
        
        Returns:
            True if token is overridden
        """
        token = self._normalize_token(token)
        return token in self._overrides
    
    def get_spy(self, token: InjectionToken) -> Optional[SpyProxy]:
        """
        Get spy for a token.
        
        Args:
            token: Token to check
        
        Returns:
            SpyProxy or None if not spied
        """
        token = self._normalize_token(token)
        return self._spies.get(token)
    
    def _normalize_token(self, token: InjectionToken) -> InjectionToken:
        """
        Normalize token to string for internal storage.
        
        Args:
            token: Token to normalize
        
        Returns:
            Normalized token (string or type)
        """
        if isinstance(token, type):
            return token
        return str(token)


if __name__ == "__main__":
    # Example usage
    from ..injectable import injectable
    from ..inject import inject
    
    class UserRepository:
        def find_by_id(self, user_id: int):
            return {"id": user_id, "name": "Real User"}
    
    @injectable
    class UserService:
        def __init__(self, repo: UserRepository = inject(UserRepository)):
            self.repo = repo
        
        def get_user(self, user_id: int):
            return self.repo.find_by_id(user_id)
    
    # Test with override
    print("=== Test 1: Override ===")
    injector = TestInjector()
    
    class MockUserRepository:
        def find_by_id(self, user_id: int):
            return {"id": user_id, "name": "Mock User"}
    
    mock_repo = MockUserRepository()
    injector.override(UserRepository, mock_repo)
    injector.register(UserService)
    
    service = injector.resolve(UserService)
    user = service.get_user(123)
    print(f"User: {user}")  # {"id": 123, "name": "Mock User"}
    
    # Test with spy
    print("\n=== Test 2: Spy ===")
    injector2 = TestInjector()
    injector2.register(UserRepository)
    injector2.register(UserService)
    
    spy = injector2.spy(UserRepository)
    service2 = injector2.resolve(UserService)
    service2.get_user(456)
    
    print(f"Called find_by_id? {spy.called('find_by_id', args=(456,))}")  # True
    print(f"Call count: {spy.call_count('find_by_id')}")  # 1
    
    # Test reset
    print("\n=== Test 3: Reset ===")
    injector3 = TestInjector()
    injector3.override('api_key', 'test-key')
    print(f"Overridden: {injector3.is_overridden('api_key')}")  # True
    
    injector3.reset()
    print(f"After reset: {injector3.is_overridden('api_key')}")  # False
